"""Declaration correspondence and bounded published-observation capabilities."""
from collections import Counter
import re
from pathlib import Path


def nodes(value):
    if isinstance(value, dict):
        yield value
        for child in value.values():
            yield from nodes(child)
    elif isinstance(value, list):
        for child in value:
            yield from nodes(child)


def alpha(value):
    """Compare binder/parameter positions while retaining full originals separately."""
    if isinstance(value, list):
        return [alpha(v) for v in value]
    if isinstance(value, dict):
        return {k:alpha(v) for k,v in value.items() if k != "origin" and not (k in ("name", "parameter") and "index" in value)}
    return value


def bind_dynamic_origins(values, local_owner, sysroot, prepared_files, artifact_owners, b):
    for node in nodes(values):
        if "dynamic" not in node:
            continue
        for predicate in node["dynamic"]["predicates"]:
            fact=predicate["fact"]
            for definition in [fact["definition"]]+([fact["trait_owner"]] if fact["kind"]=="Projection" else []):
                paths=definition.pop("origin_paths")
                if paths is None:
                    definition["owner"]=local_owner
                    continue
                owners={("sysroot:"+b.RUST_COMMIT) if Path(name).resolve().is_relative_to(sysroot) else artifact_owners.get(str(Path(name).resolve())) for name in paths}
                if None in owners or len(owners)!=1 or any(str(Path(name).resolve()) not in prepared_files for name in paths):
                    raise b.Unsupported("unbound existential definition/external closure")
                definition["owner"]=owners.pop()


def validate_dynamic(node, b):
    dynamic=node["dynamic"]
    if set(dynamic)!={"predicates","principal","projections","auto_traits","object_region"}:
        raise b.Unsupported("incomplete Dynamic semantic fields")
    principal=[];projections=[];auto_traits=[]
    for index,predicate in enumerate(dynamic["predicates"]):
        if set(predicate)!={"binders","fact"} or not isinstance(predicate["binders"],list):
            raise b.Unsupported("incomplete existential binder facts")
        fact=predicate["fact"];kind=fact.get("kind")
        fields={"kind","definition"}
        if kind in ("Trait","Projection"):
            fields|={"arguments","existential_self"}
            if fact.get("existential_self")!="omitted" or not isinstance(fact.get("arguments"),list):
                raise b.Unsupported("changed existential Self/ordered arguments")
            if any(set(a) not in ({"lifetime"},{"type"},{"const"}) for a in fact["arguments"]):
                raise b.Unsupported("unknown existential argument kind")
        if kind=="Trait":principal.append(index)
        elif kind=="Projection":
            projections.append(index);fields|={"trait_owner","term"}
            term=fact.get("term",{})
            if set(term)!={"kind","payload"} or term["kind"] not in ("type","const"):
                raise b.Unsupported("incomplete existential projection term")
        elif kind=="AutoTrait":auto_traits.append(index)
        else:raise b.Unsupported("unknown existential predicate kind")
        if set(fact)!=fields:
            raise b.Unsupported("incomplete existential predicate facts")
        for identity in [fact["definition"]]+([fact["trait_owner"]] if kind=="Projection" else []):
            if set(identity)!={"canonical","structural","crate","kind","owner"} or any(not isinstance(v,str) or not v for v in identity.values()):
                raise b.Unsupported("incomplete existential definition ownership")
    if len(principal)>1 or dynamic["principal"]!=(principal[0] if principal else None) or dynamic["projections"]!=projections or dynamic["auto_traits"]!=auto_traits:
        raise b.Unsupported("ambiguous existential principal/projection/auto-trait presence")
    if not isinstance(dynamic["object_region"],dict) or "kind" not in dynamic["object_region"]:
        raise b.Unsupported("missing published trait-object region")


def supported(value, *, declaration, b):
    for node in nodes(value):
        if "dynamic" in node:
            validate_dynamic(node,b)
        if any(key.startswith("unsupported") for key in node):
            raise b.Unsupported("unsupported required declaration/body semantic fact")
        if declaration and node.get("kind") == "ReErased":
            raise b.Unsupported("declaration lifetime erasure")
        if node.get("kind") == "constant":
            raise b.Unsupported("unsupported const generic impl shape")


def validate_inventory_body(capture, authority, b):
    if capture["original_capture"]!=authority["original_capture"]:
        raise b.Unsupported("independent inventory original capture provenance conflict")
    expected_capabilities=capabilities(capture,("call-count","resolved-module-fanout","declaration-signatures"),b)
    if capture["consumer_capabilities"]!=expected_capabilities:
        raise b.Unsupported("independent inventory consumer capability observation conflict")
    declarations = {d["owner"]:d for d in capture["declarations"]}
    for original in authority["inventory"]["owners"]:
        d = declarations[original["owner"]]
        if d["declaration_facts"] != original["declaration_facts"]:
            raise b.Unsupported("independent inventory declaration fact conflict")
        supported(d["declaration_facts"], declaration=True, b=b)
        body = original["published_body"]
        supported(body["types"],declaration=False,b=b)
        if any(t["type"] is None for t in body["types"]):
            raise b.Unsupported("unknown required body declaration type")
        if d["body_type_dependencies"] != body["types"]:
            raise b.Unsupported("independent inventory body type dependency conflict")
        if d["hir_body_tokens"] != body["tokens"] or len(d["typed_sites"]) != len(body["sites"]):
            raise b.Unsupported("independent inventory body/call completeness conflict")
        for site, expected in zip(d["typed_sites"], body["sites"]):
            if site != expected:
                raise b.Unsupported("independent inventory typed call fact conflict")
            supported([site["published_arguments"],site["published_result"],site["published_receiver"],site["adjustments"]], declaration=False, b=b)
            if site["region_stage"] != "rustc-hir-typeck-writeback-after-analysis":
                raise b.Unsupported("unknown body erasure producer/stage")
        for key, expected in body["catalog"].items():
            actual = capture["callable_catalog"].get(key)
            if actual is None or actual != expected:
                raise b.Unsupported("independent inventory callable catalog conflict")


def source_correspondence(compiler, resolver, b):
    """RA resolves declaration parameters/bounds; compiler owns generated regions."""
    if any("unsupported" in p for p in resolver["parameters"]):
        raise b.Unsupported("unsupported RA declaration parameter")
    own = compiler["generics"]["own"]
    expected = [{k:p[k] for k in ("index","name","kind")} for p in own]
    # Trait method indices include inherited Self; RA params are method-own.
    expected = [{**p,"index":i} for i,p in enumerate(expected)]
    if expected != resolver["parameters"]:
        raise b.Unsupported("original RA/compiler declaration parameter ownership conflict")
    tokens, source = compiler["source_tokens"], resolver["source_tokens"]
    positions = [i for i in range(len(source)-len(tokens)+1) if source[i:i+len(tokens)] == tokens]
    if len(positions) != 1:
        raise b.Unsupported("ambiguous original RA/compiler declaration source correspondence")
    expected_lifetimes=Counter(token for token in tokens if re.fullmatch(r"'[A-Za-z_][A-Za-z_0-9]*",token))
    if expected_lifetimes != Counter(n["syntax"] for n in resolver["lifetimes"]):
        raise b.Unsupported("incomplete original declaration lifetime source occurrences")
    regions = [n for n in nodes(compiler) if n.get("kind") in ("ReEarlyParam", "ReStatic")]
    named = {r["name"] for r in regions if r["kind"]=="ReEarlyParam"}
    resolved = {n["disposition"].get("name") for n in resolver["lifetimes"] if n["disposition"]["kind"]=="parameter"}
    if not named.issubset(resolved):
        raise b.Unsupported("unresolved original declaration lifetime correspondence")
    # Each source lifetime occurrence has a semantic disposition and exact range.
    if any(n["disposition"]["kind"] not in ("parameter","static","placeholder") for n in resolver["lifetimes"]):
        raise b.Unsupported("unknown declaration lifetime source disposition")
    for lifetime in resolver["lifetimes"]:
        disposition=lifetime["disposition"]
        if disposition["kind"]=="parameter":
            if disposition["name"]!=lifetime["syntax"]:
                raise b.Unsupported("original lifetime parameter source correspondence conflict")
            if disposition["owner_relation"]=="own" and not any(p["kind"]=="lifetime" and p["name"]==disposition["name"] and p["index"]==disposition["index"] for p in expected):
                raise b.Unsupported("original lifetime parameter ownership correspondence conflict")
        elif lifetime["syntax"]!={"static":"'static","placeholder":"'_"}[disposition["kind"]]:
            raise b.Unsupported("original lifetime static/placeholder source correspondence conflict")
    traits = {n["trait"] for n in nodes(compiler["predicates"]) if "trait" in n}
    traits.update(p["fact"]["definition"]["canonical"] for n in nodes(compiler.get("field_types",[])) if "dynamic" in n for p in n["dynamic"]["predicates"] if p["fact"]["kind"] in ("Trait","AutoTrait"))
    if any(bound["trait"] not in traits for bound in resolver["bounds"]):
        raise b.Unsupported("original resolved declaration trait bound conflict")
    return {"kind":"ra-semantic-declaration-and-source-correspondence","source_token_start":positions[0],"parameters":expected,"lifetimes":resolver["lifetimes"],"compiler_declaration":compiler}


def validate_bridge(declaration, member, invocation, b):
    facts=declaration["declaration_facts"]
    if invocation["declaration_facts"]["identity"] != facts["original"]["identity"]:
        raise b.Unsupported("original ADT semantic identity conflict")
    adt = source_correspondence(facts["original"],invocation["declaration_facts"],b)
    bridge=facts["trait_bridge"]
    trait=member["trait_declaration_facts"]
    if trait["identity"] != bridge["trait_method"] or bridge["trait"] != declaration["trait_identity"]:
        raise b.Unsupported("resolved original trait method identity conflict")
    original=source_correspondence(bridge["original"],trait,b)
    if alpha(bridge["substituted_signature"]) != alpha(facts["signature"]):
        raise b.Unsupported("unproved compiler trait/Self/receiver signature substitution")
    generated_clauses={b.encoded(alpha(p)) for p in facts["predicates"]["own"]}
    if any(b.encoded(alpha(p)) not in generated_clauses for p in bridge["substituted_own_predicates"]):
        raise b.Unsupported("unproved compiler method-own trait bound substitution")
    if len(bridge["alpha_parameters"]) != len(facts["generics"]["own"]):
        raise b.Unsupported("incomplete method-own alpha parameter relation")
    for mapping, parameter in zip(bridge["alpha_parameters"],facts["generics"]["own"]):
        if any(mapping[k]!=parameter[v] for k,v in (("generated_index","index"),("generated_name","name"),("kind","kind"))):
            raise b.Unsupported("unproved generated method parameter alpha renaming")
    return {"original_adt":adt,"original_trait_method":original,"generated_authority":"authenticated-compiler-declaration","compiler_trait_substitution":bridge}


def region_paths(value, path=()):
    if isinstance(value,dict):
        if value.get("kind")=="ReErased":
            yield list(path)
        for key,child in value.items():
            yield from region_paths(child,path+(key,))
    elif isinstance(value,list):
        for index,child in enumerate(value):
            yield from region_paths(child,path+(index,))


def capabilities(capture, required, b):
    allowed={"call-count","resolved-module-fanout","declaration-signatures"}
    erased=[]
    origin=capture["original_capture"]
    stage="rustc-hir-typeck-writeback-after-analysis"
    for declaration in capture["declarations"]:
        def record(surface,ordinal,semantic,*,target=None,adjustment=None):
            supported(semantic,declaration=False,b=b)
            for path in region_paths(semantic):
                location={"surface":surface,"ordinal":ordinal,"path":path}
                if adjustment is not None:location["adjustment_ordinal"]=adjustment
                erased.append({"original_capture":origin,"producer":"pinned-rustc-helper","stage":stage,
                    "owner":declaration["owner"],"invocation":declaration["expansion_chain"],
                    "location":location,"target":target,"observation":"published-ReErased","lost_relation":"unresolved"})
        for index,dependency in enumerate(declaration["body_type_dependencies"]):
            if dependency["region_stage"]!=stage:
                raise b.Unsupported("unknown body dependency erasure producer/stage")
            record("body-type-dependency",index,dependency["type"])
        for index,site in enumerate(declaration["typed_sites"]):
            if site["region_stage"]!=stage:
                raise b.Unsupported("unknown body call erasure producer/stage")
            for field in ("published_arguments","published_result","published_receiver"):
                record(field,index,site[field],target=site["target"])
            for adjustment,value in enumerate(site["adjustments"] or []):
                record("adjustment-target",index,value["published_target"],target=site["target"],adjustment=adjustment)
    if not required or not set(required).issubset(allowed):
        raise b.Unsupported("required consumer capability unresolved; ownership/lifetime/deletion needs separate proof")
    return {"required":sorted(required),"available":sorted(allowed),"body_erasure_observations":erased,"body_region_authority":"published-pinned-compiler-RegionKind","lifetime_sensitive_proof":"unresolved" if erased else "not-requested"}


def invocation_correspondence(capture, common, inputs, b):
    for invocation in common["invocations"]:
        selected=[d for d in capture["declarations"] if d["receiver_identity"]==invocation["receiver"] and d["expansion_chain"][0]["macro_identity"]==invocation["macro"]]
        if not selected:raise b.Unsupported("invocation-owned missing complete compiler outputs")
        site=invocation["invocation_site"]
        if "include_source_mapping" not in invocation:
            raise b.Unsupported("missing original invocation include source disposition")
        mapping=invocation["include_source_mapping"]
        compiler_included=any(c["macro_identity"]=="core::macros::builtin::include" for c in selected[0]["expansion_chain"])
        if compiler_included and selected[0]["expansion_chain"][0]["call_site"]["quality"]=="exact-source" and mapping is None:
            raise b.Unsupported("missing exact original included invocation source authority")
        if mapping is not None:
            if set(mapping)!={"kind","receiver","file","source_range","expanded_source_tokens"} or mapping["kind"]!="ra-public-include-token-descent-and-to-def" or site is None or mapping["receiver"]!=invocation["receiver"] or mapping["file"]!=site["file"] or mapping["source_range"]!=invocation["source_range"] or mapping["expanded_source_tokens"]!=invocation["declaration_facts"]["source_tokens"]:
                raise b.Unsupported("invocation semantic include source correspondence conflict")
        if site is not None:
            actual_path=Path(inputs["input_root"])/selected[0]["expansion_chain"][0]["call_site"]["file"].removeprefix("checkout:/")
            if Path(site["file"]).resolve()!=actual_path.resolve():
                raise b.Unsupported("invocation original source file identity conflict")
            start,end=map(int,invocation["source_range"].split(".."))
            if actual_path.read_bytes()[start:end]!=invocation["declaration_source"].encode():
                raise b.Unsupported("invocation original source range conflict")
        tokens=invocation["declaration_facts"]["source_tokens"]
        attributes=[];cursor=0
        while cursor<len(tokens):
            if tokens[cursor].startswith(("//","/*")):cursor+=1;continue
            if tokens[cursor:cursor+2]!=["#","["]:break
            start=cursor;depth=0
            while cursor<len(tokens):
                if tokens[cursor]=="[":depth+=1
                elif tokens[cursor]=="]":
                    depth-=1
                    if depth==0:
                        cursor+=1;break
                cursor+=1
            attributes.append(tokens[start:cursor])
        ordinal=invocation["attribute_ordinal"]
        if not 0<=ordinal<len(attributes):raise b.Unsupported("invocation attribute ordinal out of source range")
        attribute_tokens=attributes[ordinal]
        opening=attribute_tokens.index("(")+1
        closing=len(attribute_tokens)-1-attribute_tokens[::-1].index(")")
        groups=[];group=[]
        for token in attribute_tokens[opening:closing]+[","]:
            if token==",":
                if group:groups.append(group);group=[]
            else:group.append(token)
        if not 0<=invocation["derive_ordinal"]<len(groups):raise b.Unsupported("invocation derive ordinal out of source range")
        expected_path=groups[invocation["derive_ordinal"]]
        for d in selected:
            chain=d["expansion_chain"]
            actual=chain[0]["call_site"]
            if chain[0]["call_site_tokens"]!=expected_path or chain[1]["call_site_tokens"]!=attribute_tokens:
                raise b.Unsupported("invocation ordinal/attribute/compiler source token conflict")
            if site is None:
                if actual["quality"]!="coarse-generated-anchor" or len(chain)<3:
                    raise b.Unsupported("ambiguous invocation macro source correspondence")
            else:
                if actual["quality"]!="exact-source" or any(site[k]!=actual[k] for k in ("start","end")) or not site["file"].endswith(actual["file"].removeprefix("checkout:/")):
                    raise b.Unsupported("invocation source callsite/derive ordinal conflict")
                attribute=chain[1]["call_site"]
                if site["attribute_start"]!=attribute["start"] or site["attribute_end"]!=attribute["end"]:
                    raise b.Unsupported("invocation attribute source ordinal conflict")
