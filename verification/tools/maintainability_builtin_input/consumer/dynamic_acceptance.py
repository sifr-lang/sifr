"""Real Dynamic producer assertions and redigested, original-authority negatives."""
import copy
import os
import shutil

import maintainability_builtin_input as b


def semantic_capture(capture):
    # Provenance identities bind each distinct run and are verified separately.
    # Compare all semantic facts, preserving paths/tokens/regions/observations.
    value = copy.deepcopy(capture)
    value.pop("original_capture")
    for observation in value["consumer_capabilities"]["body_erasure_observations"]:
        observation.pop("original_capture")
    return value


def fields(capture):
    owner = next(d for d in capture["declarations"] if "ObjectShapes" in d["owner"] and d["owner_kind"] == "AssocFn")
    return {f["identity"].rsplit("::", 1)[1]: f["type"] for f in owner["declaration_facts"]["original"]["field_types"]}


def dynamic(value):
    return next(n["dynamic"] for n in b.declaration_consumer.nodes(value) if "dynamic" in n)


def positive(test):
    declarations = test.extended["declarations"]
    for shape, targets in (
        ("AggregateRecord", ("debug_struct_fields_finish",)),
        ("AggregateTuple", ("debug_tuple_fields_finish",)),
        ("AggregateEnum", ("debug_struct_fields_finish", "debug_tuple_fields_finish", "write_str", "debug_tuple_field1_finish")),
    ):
        owner = next(d for d in declarations if shape + " as " in d["owner"] and d["owner_kind"] == "AssocFn")
        sites = [s for s in owner["typed_sites"] if s["target"]]
        for target in targets:
            site = next(s for s in sites if s["target"].endswith("::" + target))
            test.require(site["target"] in test.extended["callable_catalog"], "actual aggregate/unit/fixed formatter catalog " + target)
        objects = [n["dynamic"] for n in b.declaration_consumer.nodes(owner["body_type_dependencies"]) if "dynamic" in n]
        test.require(bool(objects), "aggregate Debug authentic body type dependencies")
        for obj in objects:
            predicate = obj["predicates"][obj["principal"]]
            test.require(predicate["fact"]["definition"]["canonical"] == "core::fmt::Debug", "aggregate Debug canonical principal")
            test.require(predicate["fact"]["definition"]["owner"] == "sysroot:" + b.RUST_COMMIT, "authenticated formatter trait closure")
            test.require(predicate["binders"] == [] and predicate["fact"]["arguments"] == [] and predicate["fact"]["existential_self"] == "omitted", "exact empty existential binder/argument and omitted Self")
            test.require(obj["projections"] == [] and obj["auto_traits"] == [] and obj["object_region"]["kind"] == "ReErased", "explicit absent predicates and actual erased object region")
        refs = [n for n in b.declaration_consumer.nodes(owner["body_type_dependencies"]) if "reference" in n and "dynamic" in n["reference"]]
        test.require(bool(refs) and all(n["region"]["kind"] == "ReErased" for n in refs), "aggregate enclosing reference region independently retained")
    source = fields(test.extended)
    parameterized = dynamic(source["parameterized"])
    test.require([p["fact"]["kind"] for p in parameterized["predicates"]] == ["Trait", "Projection", "AutoTrait", "AutoTrait"], "actual ordered existential predicate kinds")
    principal, projection = [p["fact"] for p in parameterized["predicates"][:2]]
    test.require(principal["definition"]["canonical"].endswith("::ObjectChild") and principal["definition"]["owner"].startswith("checkout:"), "resolved local principal and source closure")
    test.require([next(iter(a)) for a in principal["arguments"]] == ["lifetime", "type", "const"], "actual ordered lifetime/type/const arguments")
    test.require(principal["arguments"] == projection["arguments"] and principal["arguments"][0]["lifetime"] == source["parameterized"]["region"], "published projection arguments omit Self and retain parameter region")
    test.require(principal["arguments"][1]["type"] == {"builtin": "u8"} and principal["arguments"][2]["const"]["value"] == {"bits": "7", "bytes": 8}, "real type and const payload")
    test.require(projection["definition"]["canonical"].endswith("::ObjectBase::Output") and projection["trait_owner"]["canonical"].endswith("::ObjectBase"), "projection actual associated-item declaring trait")
    test.require(projection["trait_owner"] != principal["definition"] and projection["term"] == {"kind": "type", "payload": {"builtin": "u16"}}, "supertrait projection owner and recursive term")
    test.require(all(p["binders"] == [] for p in parameterized["predicates"]) and parameterized["principal"] == 0 and parameterized["projections"] == [1] and parameterized["auto_traits"] == [2, 3], "explicit binders and presence ordinals")
    test.require([p["fact"]["definition"]["canonical"] for p in parameterized["predicates"][2:]] == ["core::marker::Send", "core::marker::Sync"], "real ordered auto traits")
    auto = dynamic(source["auto_only"])
    test.require(auto["principal"] is None and auto["projections"] == [] and auto["auto_traits"] == [0, 1], "auto-only object genuine principal absence")
    test.require([p["fact"]["definition"]["canonical"] for p in auto["predicates"]] == ["core::marker::Send", "core::marker::Sync"], "auto-only actual predicates")
    static = dynamic(source["static_object"])
    test.require(static["object_region"] == {"kind": "ReStatic"} and source["static_object"]["region"]["kind"] == "ReEarlyParam", "object lifetime distinct from outer reference")
    observations = test.extended["consumer_capabilities"]["body_erasure_observations"]
    test.require({o["location"]["surface"] for o in observations} == {"body-type-dependency", "published_arguments", "published_result", "published_receiver", "adjustment-target"}, "all published body erasure surfaces")
    for observation in observations:
        owner = next(d for d in declarations if d["owner"] == observation["owner"])
        location = observation["location"]
        if location["surface"] == "body-type-dependency":
            value = owner["body_type_dependencies"][location["ordinal"]]["type"]
            test.require("adjustment_ordinal" not in location and observation["target"] is None, "type dependency distinct from call")
        else:
            site = owner["typed_sites"][location["ordinal"]]
            value = site["adjustments"][location["adjustment_ordinal"]]["published_target"] if location["surface"] == "adjustment-target" else site[location["surface"]]
            test.require(observation["target"] == site["target"], "exact original call/adjustment ordinal")
        for step in location["path"]:
            value = value[step]
        test.require(value["kind"] == "ReErased" and observation["original_capture"] == test.extended["original_capture"] and observation["invocation"] == owner["expansion_chain"], "exact authentic owner/invocation/nested region path")
        test.require(observation["producer"] == "pinned-rustc-helper" and observation["stage"] == "rustc-hir-typeck-writeback-after-analysis" and observation["lost_relation"] == "unresolved", "original published erasure producer/stage/disposition")
    for capability in ("ownership-equivalence", "deletion"):
        test.assertions += 1
        with test.assertRaisesRegex(b.Unsupported, "capability unresolved"):
            b.consume(test.extended, test.extended_receipt, test.extended_receipt["inputs"], test.extended_authority, (capability,))


def negatives(test):
    changes = (
        ("principal-identity", lambda d: d["predicates"][0]["fact"]["definition"].update(canonical="invented::Trait")),
        ("principal-presence", lambda d: d.update(principal=None)),
        ("argument-kind", lambda d: d["predicates"][0]["fact"]["arguments"].__setitem__(0, {"type": {"builtin": "u8"}})),
        ("argument-order", lambda d: d["predicates"][0]["fact"]["arguments"].reverse()),
        ("existential-Self", lambda d: d["predicates"][0]["fact"].update(existential_self="invented")),
        ("existential-binder", lambda d: d["predicates"][0]["binders"].append({"kind": "region", "origin": {"anonymous": True}})),
        ("projection-term", lambda d: d["predicates"][1]["fact"]["term"].update(payload={"builtin": "u8"})),
        ("projection-owner", lambda d: d["predicates"][1]["fact"].update(trait_owner=copy.deepcopy(d["predicates"][0]["fact"]["definition"]))),
        ("projection-identity", lambda d: d["predicates"][1]["fact"]["definition"].update(canonical="invented::Output")),
        ("auto-membership", lambda d: d["predicates"].pop()),
        ("object-region", lambda d: d.update(object_region={"kind": "ReStatic"})),
        ("empty-binder-omission", lambda d: d["predicates"][0].pop("binders")),
        ("empty-projections-omission", lambda d: d.pop("projections")),
        ("empty-auto-omission", lambda d: d.pop("auto_traits")),
        ("definition-closure", lambda d: d["predicates"][0]["fact"]["definition"].update(owner="invented:closure")),
    )
    for label, change in changes:
        test.mutation(lambda c, m: change(dynamic(fields(c)["parameterized"])), "dynamic-" + label)
    def omit(c, common):
        for owner in c["declarations"]:
            owner["body_type_dependencies"] = [d for d in owner["body_type_dependencies"] if not any("dynamic" in n for n in b.declaration_consumer.nodes(d))]
        c["consumer_capabilities"] = b.declaration_consumer.capabilities(c, ("call-count", "resolved-module-fanout", "declaration-signatures"), b)
    test.mutation(omit, "coordinated-Dynamic-type-dependency-omission")
    for key in ("projections", "auto_traits"):
        test.mutation(lambda c, m: dynamic(fields(c)["static_object"]).pop(key), "explicit-empty-" + key)
    for field in ("owner", "invocation", "stage", "original_capture"):
        def change(c, common):
            observation = c["consumer_capabilities"]["body_erasure_observations"][0]
            observation[field] = "invented"
        test.mutation(change, "erasure-" + field)
    test.mutation(lambda c, m: c["consumer_capabilities"]["body_erasure_observations"][0]["location"].update(path=["invented"]), "erasure-path-swap")
    test.mutation(lambda c, m: c["consumer_capabilities"]["body_erasure_observations"][0]["location"].update(ordinal=999), "erasure-ordinal-swap")
    test.mutation(lambda c, m: c["consumer_capabilities"]["body_erasure_observations"].pop(), "erasure-disposition-omission")


def source_binder_negative(test):
    root = test.evidence / "unsupported-source-binder-fixture"
    shutil.copytree(b.TOOL / "fixtures", root)
    source = root / "fixture_root/src/extension.rs"
    original = source.read_text()
    boundary = "    pub parameterized:"
    shape = "    pub bound: &'a (dyn for<'b> BoundObject<'b, Output=u16> + 'a),\n"
    test.require(original.count(boundary) == 1 and shape not in original, "isolate only the exact new ObjectShapes::bound source")
    source.write_text(original.replace(boundary, shape + boundary))
    output = test.evidence / "unsupported-source-binder-compiler-only"
    capture, receipt = b.capture_package(root, "builtin_fixture", "builtin_fixture", output, test.helper, test.target, test.identity, whole=True)
    authority = b.read_inventory(receipt, receipt["inputs"])
    test.require(bool(b.verify_capture(capture, receipt, receipt["inputs"], authority)), "genuine original successful compiler capture")
    fact = dynamic(fields(capture)["bound"])
    test.require([p["fact"]["kind"] for p in fact["predicates"]] == ["Trait", "Projection"], "real higher-ranked existential predicates")
    for predicate in fact["predicates"]:
        bound = predicate["fact"]["arguments"][0]["lifetime"]
        test.require(len(predicate["binders"]) == 1 and predicate["binders"][0]["kind"] == "region" and bound["kind"] == "ReBound" and bound["depth"] == {"debruijn": 0} and bound["variable"] == 0 and bound["origin"] == predicate["binders"][0]["origin"], "genuine compiler nonempty binder/depth/origin retained")
    env = os.environ.copy()
    env.pop("RUSTC_BOOTSTRAP", None)
    env.update(receipt["inputs"]["resolver_preparation_environment"])
    result = b.run([str(test.resolver), str(root), "builtin_fixture", "fixture_root/src/", str(output / "capture.json")], env=env, log=output / "source-authority-rejection.log", allowed=(1,))
    test.require("Error: unresolved declaration lifetime 'b in builtin_fixture::extension::ObjectShapes" in result.stderr and not result.stdout.strip(), "specific pinned RA source-authority rejection following compiler success")
    test.require(not any((output / name).exists() for name in ("successful.json", "common.json", "join.json")), "no successful combined receipt or accepted export/publication")
    (output / "rejection.json").write_bytes(b.encoded({"disposition": "compiler-only-source-authority-rejection", "compiler_capture_digest": receipt["capture_digest"], "binder_facts": fact, "resolver_returncode": result.returncode, "resolver_error": result.stderr, "combined_success": False}))



def method_binders(test):
    owner=next(d for d in test.extended["declarations"] if "TwoLifetimes" in d["owner"] and d["owner"].endswith("::hash"))
    signature=owner["declaration_facts"]["signature"]
    test.require(len(signature["binders"])==2 and all(v["kind"]=="region" for v in signature["binders"]), "genuine two generated-method reference binders")
    for index,parameter in enumerate(signature["parameters"]):
        region=parameter["region"]
        test.require(region["kind"]=="ReBound" and region["depth"]=={"debruijn":0} and region["variable"]==index and region["origin"]==signature["binders"][index]["origin"], "actual generated-method binder depth/variable/original origin")
    for label,field,value in (
        ("valid-wrong-bound-depth","depth",{"debruijn":1}),
        ("valid-wrong-bound-variable","variable",999),
    ):
        def change(c,common):
            signature=next(d for d in c["declarations"] if d["owner"]==owner["owner"])["declaration_facts"]["signature"]
            signature["parameters"][0]["region"][field]=value
        test.mutation(change,label)
