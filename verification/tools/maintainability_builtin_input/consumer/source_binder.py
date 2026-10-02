"""Bounded original syn source-binder feasibility, without semantic export."""
from __future__ import annotations
import copy
from dataclasses import dataclass
import json
import os
from pathlib import Path
import time
import sys
import tarfile
import shutil

SCHEMA = "sifr-maintainability-source-binder-feasibility-v1"
FRAGMENT = "sifr-maintainability-source-binder-v1"
_AUTHORITY = object()
_REGISTERED = {}


@dataclass(frozen=True)
class OriginalAuthority:
    """Caller-held originals remain outside the projected records."""
    originals: bytes
    inputs: bytes
    seal: object


def _authority(originals, inputs, api):
    authority = OriginalAuthority(api.encoded(originals), api.encoded(inputs), _AUTHORITY)
    _REGISTERED[id(authority)] = authority
    return authority


def require(condition, reason, api):
    if not condition:
        raise api.Unsupported(reason)


def unique(values, label, api):
    require(len(values) == 1, "missing/ambiguous " + label, api)
    return values[0]


def stable(identity):
    return identity["crate"], identity["hash"]


def path_at(root, value):
    p = Path(value)
    return p if p.is_absolute() else root / p


def _originals(output, api):
    return {name: json.loads((output / name).read_text()) for name in (
        "original-build.json", "dependency-invocation.json", "caller-invocation.json",
        "syn-raw.json", "caller-raw.json", "ra-source.json", "independent-inventory.json")}


def _inputs(root, metadata, messages, invocations, identity, output, api):
    files = {str(root / "Cargo.lock"): api.digest((root / "Cargo.lock").read_bytes())}
    directories = {}
    slots = {}
    package_roots = {Path(p["manifest_path"]).parent for p in metadata["packages"]}
    ancestors = {root,*root.parents}
    for directory in package_roots:
        for ancestor in (directory,*directory.parents):
            manifest=ancestor/"Cargo.toml"
            if manifest.is_file(): files[str(manifest)]=api.digest(manifest.read_bytes())
    cargo_home=Path(os.environ.get("CARGO_HOME",str(Path.home()/".cargo")))
    rustup_home=Path(os.environ.get("RUSTUP_HOME",str(Path.home()/".rustup")))
    candidates={cargo_home/"config",cargo_home/"config.toml",rustup_home/"settings.toml"}
    for ancestor in ancestors:
        candidates.update(ancestor/name for name in (".cargo/config",".cargo/config.toml","rust-toolchain","rust-toolchain.toml"))
    for p in candidates:
        require(not p.exists() or p.is_file(), "unsupported original manifest/toolchain/config input", api)
        slots[str(p)]=api.digest(p.read_bytes()) if p.is_file() else None
        if p.is_file():files[str(p)]=slots[str(p)]
    packages = {p["id"]:p for p in metadata["packages"]}
    built = {m["package_id"] for m in messages if m.get("reason") == "compiler-artifact"}
    for package in built:
        package_root = Path(packages[package]["manifest_path"]).parent
        members = sorted(str(p.relative_to(package_root)) for p in package_root.rglob("*") if p.is_file() and not set(p.relative_to(package_root).parts).intersection({".git", "target", "__pycache__"}))
        directories[str(package_root)] = {"members":members,"exclude_operational":True}
        for member in members:
            p = package_root / member
            files[str(p)] = api.digest(p.read_bytes())
    build_environment = {}
    for message in messages:
        if message.get("reason") == "compiler-artifact":
            for name in message["filenames"]:
                p = Path(name); files[str(p)] = api.digest(p.read_bytes())
        if message.get("reason") != "build-script-executed":
            continue
        out = Path(message["out_dir"])
        build_output = out.parent / "output"
        require(build_output.is_file(), "missing build-script original input declarations", api)
        for p in [build_output, *(p for p in out.rglob("*") if p.is_file())]:
            files[str(p)] = api.digest(p.read_bytes())
        for line in build_output.read_text().splitlines():
            if line.startswith(("cargo:rerun-if-env-changed=", "cargo::rerun-if-env-changed=")):
                key = line.split("=",1)[1]; build_environment[key] = os.environ.get(key)
            if line.startswith(("cargo:rerun-if-changed=", "cargo::rerun-if-changed=")):
                p = Path(line.split("=",1)[1])
                if not p.is_absolute(): p = Path(packages[message["package_id"]]["manifest_path"]).parent / p
                require(p.exists(), "missing original build-script input " + str(p), api)
                if p.is_dir():
                    members = sorted(str(v.relative_to(p)) for v in p.rglob("*") if v.is_file())
                    directories[str(p)] = {"members":members,"exclude_operational":False}
                    for member in members: files[str(p / member)] = api.digest((p / member).read_bytes())
                else: files[str(p)] = api.digest(p.read_bytes())
    for invocation in invocations:
        require("RUSTC_BOOTSTRAP" not in invocation["environment"], "original compiler bootstrap leak", api)
        compiler = Path(invocation["compiler"]); files[str(compiler)] = api.digest(compiler.read_bytes())
        for arg in invocation["args"]:
            if "=" in arg and arg.split("=",1)[1].endswith((".rmeta",".rlib",".so")):
                p = Path(arg.split("=",1)[1]); files[str(p)] = api.digest(p.read_bytes())
    for p in (root / ".cargo").glob("config*"):
        if p.is_file(): files[str(p)] = api.digest(p.read_bytes())
    files.update(identity["selected_sysroot_metadata"])
    for p in (api.TOOL / "fixtures/source_binder").glob("*.rs"):
        files[str(p)] = api.digest(p.read_bytes())
    for name in ("original-build.json", "dependency-invocation.json", "caller-invocation.json", "syn-raw.json", "caller-raw.json", "ra-source.json", "independent-inventory.json"):
        p = output / name
        if p.is_file(): files[str(p)] = api.digest(p.read_bytes())
    runtime_files={str(Path(sys.executable).resolve())}
    for module in tuple(sys.modules.values()):
        for attribute in ("__file__","__cached__"):
            name=getattr(module,attribute,None)
            if name and Path(name).is_file():runtime_files.add(str(Path(name).resolve()))
    cargo=Path(api.run(["rustup","which","cargo"]).stdout.strip())
    runtime_files.add(str(cargo))
    launchers=[]; executable_selections={}
    for command in ("cargo","rustc","rustup","git","ldd"):
        selected=shutil.which(command)
        require(selected is not None, "missing original executable launcher: "+command, api)
        p=Path(selected).resolve();runtime_files.add(str(p))
        executable_selections[command]={"path":selected,"resolved":str(p)}
        if command != "ldd":launchers.append(p)
    for binary in {Path(sys.executable),cargo,*launchers}:
        linked=api.run(["ldd",str(binary)]).stdout
        for line in linked.splitlines():
            for word in line.split():
                if word.startswith("/") and Path(word).is_file():runtime_files.add(str(Path(word).resolve()))
    for name in runtime_files:files[name]=api.digest(Path(name).read_bytes())
    return {"source_candidate":api.run(["git","rev-parse","HEAD"],cwd=root).stdout.strip(),"host":list(os.uname()),"python_runtime":sys.version,"root":str(root), "tool":identity, "files":files, "directories":directories,
            "optional_input_slots":slots,"executable_selections":executable_selections,"build_environment":build_environment,
            "parent_environment":{k:api.digest(v.encode()) for k,v in os.environ.items() if not k.startswith("SIFR_BUILTIN_")}}


def archive_inventory(archive, package, api):
    archive=Path(archive)
    require(api.digest(archive.read_bytes()) == "12df2e0110f65b775f769bb17ef989067a1d931b2eb822bd4346631eeada89f9", "original archive authority conflict", api)
    root=Path(package["manifest_path"]).parent
    inventory={}
    with tarfile.open(archive,"r:gz") as source:
        for member in source.getmembers():
            if member.isdir(): continue
            require(member.isfile() and member.name.startswith("syn-3.0.5/"), "unsupported original archive member", api)
            relative=member.name.removeprefix("syn-3.0.5/")
            require(relative not in inventory and ".." not in Path(relative).parts, "ambiguous original archive member", api)
            content=source.extractfile(member)
            require(content is not None, "missing original archive member bytes", api)
            inventory[relative]=api.digest(content.read())
            p=root/relative
            require(p.is_file() and api.digest(p.read_bytes())==inventory[relative], "original archive/extracted source conflict: "+relative, api)
    extras=sorted(str(p.relative_to(root)) for p in root.rglob("*") if p.is_file() and str(p.relative_to(root)) not in inventory)
    require(set(extras) <= {".cargo-ok",".cargo-checksum.json"}, "unaccounted original extracted source files", api)
    return {"files":inventory,"cargo_operational_files":extras}


def _validate_originals(originals, inputs, api):
    build = originals["original-build.json"]
    require(build["schema"] == "sifr-maintainability-original-build-v1", "unknown original successful-build record", api)
    require(build["status"] == 0 and build["command"] == ["cargo","check","--locked","--lib","-p","sifr_codegen","--target","x86_64-unknown-linux-gnu","--message-format=json"], "missing successful unchanged selected Cargo control", api)
    messages = build["messages"]
    require(unique([m for m in messages if m.get("reason") == "build-finished"], "original build-finished event", api)["success"] is True, "failed original compiler graph", api)
    invocation = originals["dependency-invocation.json"]
    require(invocation["schema"] == "sifr-maintainability-original-invocation-v1" and invocation["compiler_status"] == 0 and invocation["environment"]["CARGO_PKG_NAME"] == "syn" and invocation["environment"]["CARGO_PKG_VERSION"] == "3.0.5" and "RUSTC_BOOTSTRAP" not in invocation["environment"], "missing authentic original dependency invocation", api)
    extern = unique([a.split("=",1)[1] for a in originals["caller-invocation.json"]["args"] if a.startswith("syn=")], "actual caller syn extern", api)
    artifact = unique([m for m in messages if m.get("reason") == "compiler-artifact" and m["target"]["name"] == "syn" and extern in m["filenames"]], "actual caller original syn artifact", api)
    require(artifact["package_id"] == build["syn_package"]["id"], "dependency package/artifact mismatch", api)
    require(build["syn_package"]["source"] == "registry+https://github.com/rust-lang/crates.io-index" and build["archive_sha256"] == "12df2e0110f65b775f769bb17ef989067a1d931b2eb822bd4346631eeada89f9", "original locked dependency archive mismatch", api)
    require(build["archive_inventory"] == archive_inventory(build["archive"],build["syn_package"],api), "original archive inventory drift", api)
    raw = originals["syn-raw.json"]; caller = originals["caller-raw.json"]
    for value in (raw, caller):
        require(value["schema"] == "sifr-maintainability-source-binder-capture-v1" and value["stage"] == "rustc-after-analysis" and value["semantic_export"] is False and value["fragment"] == FRAGMENT, "missing original local HIR capture/stage", api)
    require(api.encoded(originals["independent-inventory.json"]) == api.encoded(inventory(raw)), "independent original inventory conflict", api)
    require(originals["ra-source.json"]["schema"] == "sifr-maintainability-source-binder-ra-v1", "missing original RA/source authority", api)
    return artifact


def inventory(raw):
    owners = [{"owner":stable(o["identity"]), "parameters":[stable(p["identity"]) for p in o["parameters"]],
             "binders":[b["id"] for b in o["binders"]], "occurrences":[l["hir"] for l in o["lifetime_occurrences"]],
             "traits":[t["hir"] for t in o["trait_constraints"]]} for o in raw["declaration_owners"]]
    return {"schema":"sifr-maintainability-original-inventory-v1","owners":owners}


def _project(originals, inputs, api):
    _validate_originals(originals, inputs, api)
    raw = originals["syn-raw.json"]
    ra = unique(originals["ra-source.json"]["calls"], "original RA callable", api)
    require(ra["intrinsic_cfg"] == [{"kind":"intrinsic-true","authority":"pinned-cfg::CfgOptions::default"}], "unknown/missing RA intrinsic cfg disposition", api)
    require(ra["callee_cfg"] == raw["cfg"], "original dependency compiler/RA cfg-feature context conflict", api)
    require(ra["owner_roundtrip"] and ra["parent_roundtrip"] and not ra["contains_unknown"] and ra["impl_trait"] is None, "wrong original RA owner/trait/receiver", api)
    caller = unique([c for c in originals["caller-raw.json"]["calls"] if c["source"]["kind"] == "original" and [c["source"]["start"],c["source"]["end"]] == ra["caller_range"] and str(path_at(Path(inputs["root"]),c["source"]["file"])) == ra["caller_file"]], "original compiler callable occurrence", api)
    owner = unique([o for o in raw["declaration_owners"] if stable(o["identity"]) == stable(caller["target"])], "original dependency-local resolved owner", api)
    parent = unique([o for o in raw["declaration_owners"] if stable(o["identity"]) == stable(owner["parent"]) == stable(caller["parent"])], "original dependency-local parent", api)
    require(owner["source"]["file"] == parent["source"]["file"] == ra["owner_file"] and ra["owner_range"][0] <= owner["source"]["start"] < owner["source"]["end"] == ra["owner_range"][1], "wrong original owner exact source interval", api)
    require(ra["parent_range"][0] <= parent["source"]["start"] < parent["source"]["end"] == ra["parent_range"][1], "wrong original parent exact source interval", api)
    own = owner["generics"]["own"]
    inherited = owner["generics"]["parent"]["own"]
    for compiler, semantic, offset in ((own,ra["parameters"],owner["generics"]["parent_count"]),(inherited,ra["inherited_parameters"],0)):
        require(len(compiler) == len(semantic), "original compiler/RA generic arity conflict", api)
        for parameter, resolved in zip(compiler,semantic):
            require(parameter["kind"] == resolved["kind"] and parameter["index"] == resolved["ordinal"] + offset and parameter["name"].lstrip("'") == resolved["name"].lstrip("'"), "original compiler/RA generic parameter correspondence conflict", api)
    source = unique([s for s in raw["source_files"] if s["file"] == ra["owner_file"]], "owning original compiler SourceFile", api)
    physical = Path(ra["owner_file"]).read_bytes()
    # SourceMap supplies original positions; source contents supply the normalization witness.
    normalized = physical.decode("utf-8-sig").replace("\r\n","\n")
    require(normalized == source["normalized_text"], "original normalized source bytes conflict", api)
    correspondences = []
    lifetime_params = {stable(p["identity"]):p for p in owner["parameters"] + parent["parameters"] if p["identity"]["kind"] == "LifetimeParam"}
    named = [p for p in owner["parameters"] if p["origin"] == "Binder" and p["identity"]["kind"] == "LifetimeParam"]
    for parameter in named:
        binder = unique([b for b in owner["binders"] if parameter["identity"] in b["parameters"]], "compiler binder declaration", api)
        require(binder["compiler_map_disposition"] == "present" and binder["variables"], "missing supported compiler binder variables", api)
        records = [(parameter["source"], parameter["hir"], "declaration", None)]
        records += [(l["source"],l["hir"],"use",l) for l in owner["lifetime_occurrences"] if l["target"] and stable(l["target"]) == stable(parameter["identity"])]
        for span, hir, role, lifetime in records:
            require(span["kind"] == "original" and span["hygiene"] == "#0", "generated/coarse source is not exact original authority", api)
            occurrence = unique([l for l in ra["lifetime_occurrences"] if l["kind"] == "LIFETIME" and l["range"] == [span["start"],span["end"]]], "exact RA lifetime occurrence", api)
            require(physical[span["start"]:span["end"]].decode() == occurrence["token"] == span["snippet"], "literal original lifetime token conflict", api)
            if lifetime:
                require(lifetime["resolved"]["kind"] == "LateBound" and stable(lifetime["resolved"]["target"]) == stable(parameter["identity"]), "missing genuine compiler late-bound declaration relation", api)
            require(binder["variables"][binder["parameters"].index(parameter["identity"])]["declaration"] == parameter["identity"], "ordered compiler bound variable declaration conflict", api)
            correspondences.append({"hir":hir,"role":role,"target":stable(parameter["identity"]),"binder":binder["id"],"binder_parameter_ordinal":binder["parameters"].index(parameter["identity"]),"ra":occurrence,"source":span,"resolved":lifetime["resolved"] if lifetime else {"kind":"GenericParamDeclaration"}})
    for lifetime in owner["lifetime_occurrences"]:
        if lifetime["syntax"] == "Implicit":
            require(lifetime["target"] and stable(lifetime["target"]) in lifetime_params, "missing explicit elided declaration disposition", api)
            continue
        require(lifetime["target"] and stable(lifetime["target"]) in lifetime_params, "unowned named original lifetime", api)
        if stable(lifetime["target"]) not in {stable(p["identity"]) for p in named}:
            span = lifetime["source"]
            occurrence = unique([l for l in ra["lifetime_occurrences"] if l["range"] == [span["start"],span["end"]]], "exact inherited RA occurrence", api)
            require(occurrence["resolved"] and occurrence["resolved"]["relation"] == "inherited" and lifetime["resolved"]["kind"] == "EarlyBound", "inherited lifetime compiler/RA conflict", api)
            inherited_params = [p for p in parent["parameters"] if p["origin"] == "Generics"]
            parameter_ordinal = next(i for i,p in enumerate(inherited_params) if stable(p["identity"]) == stable(lifetime["target"]))
            require(occurrence["resolved"]["ordinal"] == parameter_ordinal, "resolved inherited parameter ordinal conflict", api)
            correspondences.append({"hir":lifetime["hir"],"role":"inherited-use","target":stable(lifetime["target"]),"ra":occurrence,"source":span,"resolved":lifetime["resolved"]})
    constraints = []
    for trait in owner["trait_constraints"]:
        span = trait["source"]
        r = unique([t for t in ra["trait_constraints"] if t["range"] == [span["start"],span["end"]]], "exact original resolved trait path", api)
        require(trait["resolved"] and r["roundtrip"] and trait["canonical_path"] == r["identity"], "wrong resolved compiler/RA trait", api)
        declaration = trait["declaration_source"]
        require(declaration and declaration["kind"] == "original" and declaration["hygiene"] == "#0", "missing original compiler trait declaration source", api)
        compiler_file = Path(declaration["file"])
        ra_file = Path(r["source_file"])
        require(compiler_file.resolve() == ra_file.resolve() and r["source_range"][0] <= declaration["start"] < declaration["end"] <= r["source_range"][1], "resolved trait declaration source correspondence conflict", api)
        name = trait["declaration_name_source"]
        require(name and name["kind"] == "original" and [name["start"],name["end"]] == r["name_range"] and name["file"] == declaration["file"], "resolved trait exact declaration Name node conflict", api)
        require(ra_file.read_bytes()[name["start"]:name["end"]].decode() == name["snippet"] == r["name_token"], "resolved original trait declaration token conflict", api)
        require(str(ra_file) in inputs["tool"]["rust_source_files"], "trait source lacks official selected rust-src authority", api)
        constraints.append({"compiler":trait,"ra":r})
    require(named and len(correspondences) == len(ra["lifetime_occurrences"]), "incomplete bounded original occurrence inventory", api)
    return {"schema":SCHEMA,"fragment":FRAGMENT,"context":{"package":"sifr_codegen","target":"x86_64-unknown-linux-gnu","test":False},"original_build_authority":originals["original-build.json"],"dependency_invocation":originals["dependency-invocation.json"],"source_files":[source],"declaration_owners":[owner,parent],"binders":owner["binders"],"lifetime_occurrences":owner["lifetime_occurrences"],"trait_constraints":constraints,"source_correspondences":correspondences,"independent_inventory":originals["independent-inventory.json"],"semantic_export":False}


def _verify(proof, receipt, authority, api):
    require(isinstance(authority,OriginalAuthority) and authority.seal is _AUTHORITY and _REGISTERED.get(id(authority)) is authority, "unauthenticated or replaced original bridge authority", api)
    inputs = json.loads(authority.inputs)
    require(api.run(["git","rev-parse","HEAD"],cwd=inputs["root"]).stdout.strip() == inputs["source_candidate"] and list(os.uname()) == inputs["host"] and sys.version == inputs["python_runtime"], "original candidate/host/Python runtime drift", api)
    require(set(receipt) == {"schema","inputs","proof_digest","capture_status","semantic_export","timing_seconds"} and receipt["schema"] == "sifr-maintainability-source-binder-receipt-v1", "closed original bridge receipt schema mismatch", api)
    require(receipt["inputs"] == inputs and receipt["capture_status"] == 0 and receipt["semantic_export"] is False, "original bridge context/capture compiler failure", api)
    for name, sha in inputs["files"].items():
        require(Path(name).is_file() and api.digest(Path(name).read_bytes()) == sha, "original bridge input/source/artifact drift: " + name, api)
    for command, expected in inputs["executable_selections"].items():
        current=shutil.which(command)
        require(current is not None and {"path":current,"resolved":str(Path(current).resolve())} == expected, "original executable selection drift: "+command, api)
    for name, expected in inputs["optional_input_slots"].items():
        path=Path(name)
        actual=api.digest(path.read_bytes()) if path.is_file() else None
        require(actual == expected and (not path.exists() or path.is_file()), "original toolchain/config presence or content drift: "+name, api)
    for name, inventory in inputs["directories"].items():
        root=Path(name)
        actual=sorted(str(p.relative_to(root)) for p in root.rglob("*") if p.is_file() and (not inventory["exclude_operational"] or not set(p.relative_to(root).parts).intersection({".git","target","__pycache__"})))
        require(actual == inventory["members"], "original source directory input drift: " + name, api)
    require({k:api.digest(v.encode()) for k,v in os.environ.items() if not k.startswith("SIFR_BUILTIN_")} == inputs["parent_environment"], "original parent environment drift", api)
    for key,value in inputs["build_environment"].items(): require(os.environ.get(key) == value, "build-script environment drift", api)
    for name,sha in inputs["tool"]["consumer_sources"].items(): require(api.digest((api.ROOT / "scripts" / name).read_bytes()) == sha, "original consumer source drift", api)
    for name,sha in inputs["tool"]["helper_sources"].items(): require(api.digest((api.TOOL / name).read_bytes()) == sha, "original helper/schema source drift", api)
    for name,sha in inputs["tool"]["helper_build"]["executables_and_runtime"].items(): require(api.digest(Path(name).read_bytes()) == sha, "original helper executable/runtime drift", api)
    resolver=inputs["resolver_root"]
    require(api.run(["git","-C",resolver,"rev-parse","HEAD"]).stdout.strip() == api.RA_COMMIT and api.run(["git","-C",resolver,"rev-parse","HEAD^{tree}"]).stdout.strip() == api.RA_TREE, "original RA source pin drift", api)
    api.run(["git","-C",resolver,"diff","--exit-code","HEAD"])
    schema = json.loads((api.TOOL / "schema/source-binder-feasibility-v1.json").read_text())
    require(isinstance(proof,dict) and set(proof) == set(schema["required"]), "closed original bridge schema mismatch", api)
    for name,rule in schema["properties"].items():
        value=proof[name]
        if "const" in rule:require(value == rule["const"], "original bridge schema variant mismatch: " + name, api)
        if rule.get("type")=="array":require(isinstance(value,list) and rule["minItems"] <= len(value) <= rule["maxItems"], "original bridge bounded inventory mismatch: " + name, api)
        if rule.get("type")=="object":
            require(isinstance(value,dict), "original bridge schema type mismatch: " + name, api)
            if "properties" in rule:
                require(set(value)==set(rule["required"]),"closed original inventory schema mismatch",api)
                require(value["schema"]==rule["properties"]["schema"]["const"],"unknown original inventory schema",api)
                bound=rule["properties"]["owners"]
                require(isinstance(value["owners"],list) and bound["minItems"]<=len(value["owners"])<=bound["maxItems"],"bounded original inventory universe mismatch",api)
    originals = json.loads(authority.originals)
    expected = _project(originals,inputs,api)
    require(api.encoded(proof) == api.encoded(expected), "projected original bridge owner/binder/use/trait/inventory conflict", api)
    require(receipt["proof_digest"] == api.digest(api.encoded(proof)), "original bridge receipt digest conflict", api)
    return {"feasibility":True,"source_correspondences":len(proof["source_correspondences"]),"semantic_export":False}


def capture(output, target, identity, api):
    output, target = Path(output).resolve(), Path(target).resolve()
    output.mkdir(parents=True,exist_ok=True)
    start = time.monotonic()
    if (output / "success.json").is_file():
        binding=json.loads((output / "success.json").read_text())
        for name,sha in binding.items(): require(api.digest((output/name).read_bytes()) == sha,"cached original authority drift",api)
        originals=_originals(output,api)
        receipt=json.loads((output / "receipt.json").read_text())
        require(receipt["inputs"]["tool"] == identity, "cached original helper/component/input identity drift", api)
        authority=_authority(originals,receipt["inputs"],api)
        proof=json.loads((output / "proof.json").read_text())
        verify(proof,receipt,authority,api)
        print("source-binder prepared cache HIT",receipt["timing_seconds"])
        return proof,receipt,authority
    root=api.ROOT
    store=output / "invocations";store.mkdir()
    env=os.environ.copy()
    for key in ("RUSTC_BOOTSTRAP","RUSTC_WRAPPER","RUSTC_WORKSPACE_WRAPPER"):env.pop(key,None)
    env.update(CARGO_INCREMENTAL="0",CARGO_BUILD_JOBS="2",CARGO_TARGET_DIR=str(target),RUSTC_WRAPPER=str(api.TOOL / "consumer/dependency_invocation.py"),SIFR_BUILTIN_DEPENDENCY_INVOCATIONS=str(store))
    command=["cargo","check","--locked","--lib","-p","sifr_codegen","--target","x86_64-unknown-linux-gnu","--message-format=json"]
    result=api.run(command,env=env,log=output / "original-control-warm.log")
    rebuild=False
    if not list(store.glob("syn-*.json")):
        # Rebuild only the owned selected dependency to obtain actual original argv.
        api.run(["cargo","clean","--package","syn@3.0.5","--target","x86_64-unknown-linux-gnu"],env=env,log=output / "owned-syn-rebuild.log")
        result=api.run(command,env=env,log=output / "original-control-rebuild.log")
        rebuild=True
    messages=[json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]
    metadata_result=api.run(["cargo","metadata","--locked","--format-version","1","--filter-platform","x86_64-unknown-linux-gnu"],env=env,log=output / "metadata.log")
    metadata=json.loads(metadata_result.stdout)
    selected=[]
    for crate,name in (("syn","dependency-invocation.json"),("sifr_codegen","caller-invocation.json")):
        invocation=unique([json.loads(p.read_text()) for p in store.glob(crate+"-*.json")], "actual original "+crate+" invocation", api)
        (output / name).write_bytes(api.encoded(invocation));selected.append(invocation)
    syn=unique([p for p in metadata["packages"] if p["name"]=="syn" and p["version"]=="3.0.5"],"selected original syn package",api)
    archive=unique(list(Path.home().glob(".cargo/registry/cache/*/syn-3.0.5.crate")),"original syn archive",api)
    build={"schema":"sifr-maintainability-original-build-v1","command":command,"status":result.returncode,"messages":messages,"syn_package":syn,"archive":str(archive),"archive_sha256":api.digest(archive.read_bytes()),"metadata":metadata,"metadata_sha256":api.digest(api.encoded(metadata)),"owned_dependency_rebuild":rebuild}
    build["archive_inventory"]=archive_inventory(archive,syn,api)
    (output / "original-build.json").write_bytes(api.encoded(build))
    before=_inputs(root,metadata,messages,selected,identity,output,api)
    before["files"][str(archive)]=build["archive_sha256"]
    analysis_times={}
    for invocation,name in zip(selected,("syn-raw.json","caller-raw.json")):
        replay=invocation["environment"].copy()
        replay["SIFR_BUILTIN_SOURCE_BINDER"]=str(output/name)
        if name=="caller-raw.json":replay["SIFR_BUILTIN_SOURCE_BINDER_CALL_SUFFIX"]="crates/sifr_codegen/src/inline_syntax.rs"
        analysis=api.run([str(target / "debug/sifr_maintainability_builtin_input"),*invocation["args"]],cwd=invocation["cwd"],env=replay,log=output / (name+".log"))
        require((output/name).is_file(), "original local HIR unavailable after compiler analysis",api)
        analysis_times[name]=analysis.elapsed_seconds
    raw=json.loads((output/"syn-raw.json").read_text());caller=json.loads((output/"caller-raw.json").read_text())
    (output/"independent-inventory.json").write_bytes(api.encoded(inventory(raw)))
    (output/"ra-context.json").write_bytes(api.encoded({"context":{"crate":"sifr_codegen"},"cfg":caller["cfg"],"dependency_cfg":raw["cfg"],"dependency_root":next(t["src_path"] for t in syn["targets"] if "lib" in t["kind"])}))
    resolver_env=os.environ.copy()
    for key in ("RUSTC_BOOTSTRAP","RUSTC_WRAPPER","RUSTC_WORKSPACE_WRAPPER"):resolver_env.pop(key,None)
    resolver_env.update(CARGO_INCREMENTAL="0",CARGO_BUILD_JOBS="2",CARGO_TARGET_DIR=str(target),SIFR_BUILTIN_SOURCE_BINDER_RA="1")
    resolver=api.run([str(target / "debug/ra_common"),str(root),"sifr_codegen","crates/sifr_codegen/src/inline_syntax.rs",str(output/"ra-context.json")],env=resolver_env,log=output / "ra-source.log")
    (output/"ra-source.json").write_text(resolver.stdout)
    for name,sha in before["files"].items():require(api.digest(Path(name).read_bytes())==sha,"original input drift across capture/RA stages: "+name,api)
    inputs=_inputs(root,metadata,messages,selected,identity,output,api)
    inputs["files"][str(archive)]=build["archive_sha256"]
    sysroot=Path(api.run(["rustc","--print","sysroot"]).stdout.strip())
    rust_source_files={str(p):api.digest(p.read_bytes()) for p in (sysroot / "lib/rustlib/src/rust").rglob("*") if p.is_file()}
    inputs["tool"]=copy.deepcopy(inputs["tool"])
    inputs["tool"]["rust_source_files"]=rust_source_files
    inputs["files"].update(rust_source_files)
    inputs["resolver_root"]=str(next(Path.home().glob(".cargo/git/checkouts/rust-analyzer-*/03fcb77")))
    component_path=Path(os.environ["SIFR_BUILTIN_COMPONENT_RECEIPT"])
    component=json.loads(component_path.read_text())
    for relative,record in component["inventory"].items():inputs["files"][str(sysroot/relative)]=record["sha256"]
    for p in (component_path,component_path.parent/"official-channel-manifest.toml",component_path.parent/"rustc-dev-1.98.1-x86_64-unknown-linux-gnu.tar.xz",sysroot/"lib/rustlib/multirust-channel-manifest.toml",sysroot/"libexec/rust-analyzer-proc-macro-srv"):
        inputs["files"][str(p)]=api.digest(p.read_bytes())
    originals=_originals(output,api)
    proof=_project(originals,inputs,api)
    receipt={"schema":"sifr-maintainability-source-binder-receipt-v1","inputs":inputs,"proof_digest":api.digest(api.encoded(proof)),"capture_status":0,"semantic_export":False,"timing_seconds":{"original_control":result.elapsed_seconds,"metadata":metadata_result.elapsed_seconds,"analysis":analysis_times,"resolver":resolver.elapsed_seconds,"total":time.monotonic()-start}}
    authority=_authority(originals,inputs,api)
    verify(proof,receipt,authority,api)
    (output/"proof.json").write_bytes(api.encoded(proof));(output/"receipt.json").write_bytes(api.encoded(receipt))
    (output/"success.json").write_bytes(api.encoded({name:api.digest((output/name).read_bytes()) for name in ("proof.json","receipt.json","original-build.json","dependency-invocation.json","caller-invocation.json","syn-raw.json","caller-raw.json","ra-source.json","independent-inventory.json")}))
    print("source-binder prepared cache MISS",receipt["timing_seconds"])
    return proof,receipt,authority


def verify(proof, receipt, authority, api):
    try:
        return _verify(proof,receipt,authority,api)
    except (KeyError,TypeError,ValueError,OSError,AttributeError) as error:
        raise api.Unsupported(f"malformed/unavailable original bridge authority: {error}") from error
