#!/usr/bin/env python3
"""Pinned builtin compiler capability capture; it is not a semantic adapter export."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tomllib
import time

SCHEMA = "sifr-maintainability-builtin-capability-v1"
RUST_COMMIT = "48a229ceaefd4985c50990b14116b6d856af0985"
CARGO_COMMIT = "797e8a9bca276c1c9f9f738d2a20f484fa4eea9d"
RA_COMMIT = "03fcb77246f2568adb0e9b2fa60d19c6cc1686f4"
RA_TREE = "0081a116ddfb9f5c3673eba97df030bea907106f"
SERVER_DIGEST = "98f6311d05a2f4b9132dbdeeefed058204374587f5efcf869db0a94025c3fc2a"
SOURCE_DIGEST = "022d8a071204673d370771be04904f302fbebd7e7dada0aa8918f74512bac507"
ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "verification/tools/maintainability_builtin_input"


class Unsupported(RuntimeError):
    """Evidence cannot establish the complete affected capability."""


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def run(args, *, cwd=ROOT, env=None, log=None, allowed=(0,)):
    started = time.monotonic()
    result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True, timeout=1200)
    result.elapsed_seconds = time.monotonic() - started
    if log:
        Path(log).write_text(result.stdout + result.stderr)
    if result.returncode not in allowed:
        raise Unsupported(f"command failed ({result.returncode}): {args}: {result.stderr[-3000:]}")
    return result


def tool_identity(receipt_path, target=None):
    try:
        return _tool_identity(receipt_path, target)
    except (OSError, KeyError, ValueError, TypeError) as error:
        raise Unsupported(f"unreadable/malformed selected component/helper identity: {error}") from error


def _tool_identity(receipt_path, target=None):
    receipt_path = Path(receipt_path).resolve()
    receipt = json.loads(receipt_path.read_text())
    official_path = receipt_path.parent / "official-channel-manifest.toml"
    archive = receipt_path.parent / "rustc-dev-1.98.1-x86_64-unknown-linux-gnu.tar.xz"
    if receipt["manifest_url"] != "https://static.rust-lang.org/dist/channel-rust-1.98.1.toml" or digest(official_path.read_bytes()) != receipt["manifest_sha256"]:
        raise Unsupported("official component manifest provenance drift")
    official = tomllib.loads(official_path.read_text())["pkg"]["rustc-dev"]["target"]["x86_64-unknown-linux-gnu"]
    if official != receipt["component"] or digest(archive.read_bytes()) != official["xz_hash"]:
        raise Unsupported("official component archive provenance drift")
    rustc = run(["rustc", "-vV"]).stdout
    cargo = run(["cargo", "-vV"]).stdout
    if f"commit-hash: {RUST_COMMIT}" not in rustc or f"commit-hash: {CARGO_COMMIT}" not in cargo:
        raise Unsupported("wrong selected compiler/Cargo commit")
    if "host: x86_64-unknown-linux-gnu" not in rustc:
        raise Unsupported("uncovered compiler host")
    if receipt["rustc"] != rustc or receipt["cargo"] != cargo:
        raise Unsupported("component compiler context drift")
    sysroot = Path(run(["rustc", "--print", "sysroot"]).stdout.strip())
    installed_manifest = sysroot / "lib/rustlib/multirust-channel-manifest.toml"
    if digest(installed_manifest.read_bytes()) != receipt["installed_manifest_sha256"]:
        raise Unsupported("installed toolchain manifest drift")
    installed = sysroot / "lib/rustlib/manifest-rustc-dev-x86_64-unknown-linux-gnu"
    if not installed.is_file():
        raise Unsupported("missing exact rustc-dev component")
    inventory_files = {line.split(":", 1)[1] for line in installed.read_text().splitlines() if line.startswith("file:")}
    required_files = inventory_files | {str(p.relative_to(sysroot)) for p in (sysroot / "lib").glob("*.so*")}
    if set(receipt["inventory"]) != required_files:
        raise Unsupported("truncated or unexpected component/runtime inventory")
    if receipt["component"]["xz_hash"] != "97c49fd126d47aa0446488e2674af8b3062b1ec8d72d59ce83d43e45f34a5245":
        raise Unsupported("wrong official component archive")
    for relative, identity in receipt["inventory"].items():
        path = sysroot / relative
        if not path.is_file() or digest(path.read_bytes()) != identity["sha256"]:
            raise Unsupported(f"component/runtime drift: {relative}")
    source = sysroot / "lib/rustlib/src/rust"
    source_files = {str(p.relative_to(source)): digest(p.read_bytes()) for p in sorted(source.rglob("*")) if p.is_file()}
    if digest(encoded(source_files)) != SOURCE_DIGEST:
        raise Unsupported("selected rust-src drift")
    server = sysroot / "libexec/rust-analyzer-proc-macro-srv"
    if not server.is_file() or digest(server.read_bytes()) != SERVER_DIGEST:
        raise Unsupported("selected proc-macro server drift")
    environment = os.environ.copy()
    environment["RUST_ANALYZER_INTERNALS_DO_NOT_USE"] = "this is unstable"
    protocol = subprocess.run([str(server)], input=json.dumps({"ApiVersionCheck": {}}) + "\n", text=True, capture_output=True, env=environment)
    if protocol.returncode or json.loads(protocol.stdout) != {"ApiVersionCheck": 6}:
        raise Unsupported("selected server protocol mismatch")
    lock = tomllib.loads((TOOL / "Cargo.lock").read_text())
    resolver_sources = {p.get("source") for p in lock["package"] if p.get("source", "").startswith("git+")}
    if resolver_sources != {f"git+https://github.com/rust-lang/rust-analyzer.git?rev={RA_COMMIT}#{RA_COMMIT}"}:
        raise Unsupported("unpinned resolver lock graph")
    resolver = next(Path.home().glob(".cargo/git/checkouts/rust-analyzer-*/03fcb77"), None)
    if resolver is None or run(["git", "-C", str(resolver), "rev-parse", "HEAD"]).stdout.strip() != RA_COMMIT or run(["git", "-C", str(resolver), "rev-parse", "HEAD^{tree}"]).stdout.strip() != RA_TREE:
        raise Unsupported("missing or mismatched pinned resolver source")
    run(["git", "-C", str(resolver), "diff", "--exit-code", "HEAD"])
    lock_graph = sorted([{key: value for key, value in entry.items() if key in ("name", "version", "source", "checksum", "dependencies")} for entry in lock["package"]], key=lambda entry: (entry["name"], entry["version"]))
    identity = {"selected_sysroot_metadata": {str(path):digest(path.read_bytes()) for path in sorted((sysroot / "lib/rustlib/x86_64-unknown-linux-gnu/lib").glob("*")) if path.is_file()}, "consumer_sources": {name:digest((ROOT / "scripts" / name).read_bytes()) for name in ("maintainability_builtin_input.py", "maintainability_builtin_input_tests.py")}, "lock_graph": lock_graph, "rustc": rustc, "cargo": cargo, "component_receipt": digest(Path(receipt_path).read_bytes()), "server": SERVER_DIGEST, "rust_src": SOURCE_DIGEST, "resolver": RA_COMMIT, "lock": digest((TOOL / "Cargo.lock").read_bytes()), "helper_sources": {str(p.relative_to(TOOL)):digest(p.read_bytes()) for p in sorted(TOOL.rglob("*")) if p.is_file() and "fixtures" not in p.parts}}

    if target is not None:
        binding = json.loads((Path(target) / "builtin-build-receipt.json").read_text())
        if binding["source_identity"] != identity:
            raise Unsupported("helper source/compiler/lock graph drift; rebuild exact isolated companion")
        for path, sha256 in binding["executables_and_runtime"].items():
            if digest(Path(path).read_bytes()) != sha256:
                raise Unsupported(f"helper executable/dynamic runtime drift: {path}")
        identity["helper_build"] = binding
    return identity


def bind_helper_build(target, receipt_path, build_log):
    target = Path(target).resolve()
    identity = tool_identity(receipt_path)
    files = {}
    for name in ("sifr_maintainability_builtin_input", "ra_common"):
        executable = target / "debug" / name
        files[str(executable)] = digest(executable.read_bytes())
        result = run(["ldd", str(executable)])
        for path in re.findall(r"(?:=> )?(/[^\s]+)", result.stdout):
            files[path] = digest(Path(path).read_bytes())
    binding = {"source_identity": identity, "executables_and_runtime": files,
               "bootstrap": "RUSTC_BOOTSTRAP=sifr_maintainability_builtin_input",
               "command": "cargo build --locked --manifest-path verification/tools/maintainability_builtin_input/Cargo.toml",
               "build_log_digest": digest(Path(build_log).read_bytes())}
    (target / "builtin-build-receipt.json").write_bytes(encoded(binding))
    return binding


def normalize(value, root, sysroot=None):
    """Canonicalize explicit paths only; preserve compiler tokens and literals."""
    if isinstance(value, dict):
        return {normalize(k, root, sysroot): normalize(v, root, sysroot) for k, v in value.items()}
    if isinstance(value, list):
        return [normalize(v, root, sysroot) for v in value]
    if isinstance(value, str) and value.startswith(str(root) + "/"):
        return "checkout:/" + value[len(str(root)) + 1:]
    if isinstance(value, str) and sysroot and value.startswith(str(sysroot) + "/"):
        return "sysroot:/" + value[len(str(sysroot)) + 1:]
    return value


def normalized_body_tokens(tokens):
    if tokens is None:
        return None
    # AST and HIR printers differ in optional trailing struct/pattern commas.
    # Tokens remain intact in evidence; this explicit punctuation relation
    # never changes a literal or semantic operation.
    return [token for index, token in enumerate(tokens) if not (token == "," and index + 1 < len(tokens) and tokens[index + 1] == "}")]


def validate_schema(value):
    schema = json.loads((TOOL / "schema/capability-v1.json").read_text())
    def check(value, rule, path):
        types = {"object": dict, "array": list, "string": str}
        if "type" in rule and not isinstance(value, types[rule["type"]]):
            raise Unsupported(f"schema type mismatch: {path}")
        if "const" in rule and value != rule["const"] or "enum" in rule and value not in rule["enum"]:
            raise Unsupported(f"schema value mismatch: {path}")
        if isinstance(value, dict):
            if not set(rule.get("required", [])).issubset(value):
                raise Unsupported(f"schema missing record: {path}")
            for name, child in rule.get("properties", {}).items():
                if name in value:
                    check(value[name], child, path + "." + name)
        if isinstance(value, list):
            if len(value) < rule.get("minItems", 0):
                raise Unsupported(f"schema truncated sequence: {path}")
            for index, item in enumerate(value):
                check(item, rule.get("items", {}), path + f"[{index}]")
    check(value, schema, "capture")


def validate_resolution(site, catalog):
    target, implementation = site["target"], site["implementation"]
    disposition = site["disposition"]
    if disposition == "scalar-operation":
        if any(site[field] is not None for field in ("target", "trait", "signature", "implementation", "instance_kind", "target_owner", "implementation_owner")):
            raise Unsupported("scalar operation has an invented callable disposition")
        return
    if target not in catalog:
        raise Unsupported(f"invented/unregistered callable target: {target}")
    descriptor = catalog[target]
    for field in ("trait", "signature"):
        if site[field] != descriptor[field]:
            raise Unsupported(f"callable declaration {field} conflict: {target}")
    if not site["target_owner"] or site["target_owner"] != descriptor["owner"]:
        raise Unsupported(f"callable package ownership conflict: {target}")
    trait_member = descriptor["kind"] in {"trait-member", "trait-implementation"}
    if trait_member != (site["trait"] is not None):
        raise Unsupported(f"trait-associated target lost resolved trait origin: {target}")
    if implementation is not None:
        if implementation not in catalog or site["instance_kind"] != "static":
            raise Unsupported("invented or nonstatic concrete implementation")
        concrete = catalog[implementation]
        if not site["implementation_owner"] or site["implementation_owner"] != concrete["owner"]:
            raise Unsupported("concrete implementation ownership conflict")
        if concrete["trait"] != site["trait"]:
            raise Unsupported("target and concrete implementation trait conflict")
    elif site["implementation_owner"] is not None:
        raise Unsupported("invented implementation ownership")
    if disposition == "generic-trait-call":
        if not trait_member or not site["generics"] or implementation is not None or site["instance_kind"] is not None:
            raise Unsupported("invalid generic trait dispatch")
    elif disposition == "dynamic-trait-call":
        if not trait_member or implementation is not None or site["instance_kind"] != "dynamic":
            raise Unsupported("invalid dynamic trait dispatch")
    elif disposition == "resolved-call":
        if site["instance_kind"] not in {"static", "intrinsic", "compiler-shim"} or (site["instance_kind"] == "static" and implementation is None):
            raise Unsupported("unproved resolved/static call")
    else:
        raise Unsupported("unsupported callable disposition")


def read_inventory(receipt, inputs):
    """Load expected authority using the caller's original, independently held inputs."""
    reference = inputs.get("inventory_authority")
    if not reference or receipt.get("inventory_authority") != reference:
        raise Unsupported("missing/mismatched independent inventory authority")
    path = Path(reference["path"])
    if not path.is_file() or digest(path.read_bytes()) != reference["digest"]:
        raise Unsupported("changed/truncated independent inventory artifact")
    authority = json.loads(path.read_text())
    binding = {key:value for key,value in inputs.items() if key != "inventory_authority"}
    if authority["input_digest"] != digest(encoded(binding)):
        raise Unsupported("independent inventory producer/input binding conflict")
    return authority


def validate_inventory(capture, authority):
    if not authority or "inventory" not in authority:
        raise Unsupported("missing independent inventory authority")
    inventory = authority["inventory"]
    if capture.get("inventory_digest") != digest(encoded(inventory)):
        raise Unsupported("independent inventory digest conflict")
    if inventory.get("crate") != capture["context"].get("crate"):
        raise Unsupported("independent inventory selected crate conflict")
    if authority["context"] != capture["context"] or authority["cfg"] != capture["cfg"]:
        raise Unsupported("independent inventory context conflict")
    if inventory.get("schema") != "sifr-builtin-owner-inventory-v1" or inventory.get("enumeration") != "hir-crate-free-items-and-associated-items" or inventory.get("trait_impl_cross_check") is not True:
        raise Unsupported("unproved independent HIR/type inventory")
    owners, universe = inventory["owners"], inventory["universe"]
    expected = {entry["owner"]:entry for entry in owners}
    considered = {entry["structural_identity"]:entry for entry in universe}
    if not owners or not universe or len(expected) != len(owners) or len(considered) != len(universe):
        raise Unsupported("empty/duplicate independent inventory owner")
    impls = {entry["owner"] for entry in owners if entry["owner_kind"].startswith("Impl")}
    if impls != {entry["owner"] for entry in universe if entry["disposition"] == "selected"}:
        raise Unsupported("independent inventory preselection universe mismatch")
    for entry in universe:
        if entry["disposition"] not in {"selected", "nonselected"} or not entry["reason"] or not entry["source"]:
            raise Unsupported("unsupported independent inventory provenance")
        if entry["disposition"] == "selected" and (not entry["trait_impl_cross_checked"] or not entry["expansion_chain"] or not entry["expansion_chain"][0]["builtin"]):
            raise Unsupported("unknown independent inventory builtin provenance")
    declarations = capture["declarations"]
    actual = {entry["owner"]:entry for entry in declarations}
    if len(actual) != len(declarations) or actual.keys() != expected.keys():
        raise Unsupported("independent inventory missing/extra/duplicate export owner")
    fields = ("output_role", "structural_identity", "owner_kind", "parent", "receiver_identity", "trait_identity", "generic_bounds", "canonical_signature", "visibility", "hir_body", "expansion_chain", "hir_members")
    for owner, entry in expected.items():
        disposition = "impl-owner" if owner in impls else "actual-body" if entry["hir_body"] else "actual-bodyless-member"
        if entry["member_disposition"] != disposition:
            raise Unsupported("independent inventory body/member disposition conflict")
        if any(entry[field] != actual[owner][field] for field in fields):
            raise Unsupported("independent inventory ownership/member conflict: " + owner)
        if owner in impls:
            members = entry["hir_members"]
            children = [child["owner"] for child in owners if child["parent"] == owner]
            if len(set(members)) != len(members) or sorted(members) != sorted(children):
                raise Unsupported("independent inventory duplicate/missing member")
        elif entry["parent"] not in impls:
            raise Unsupported("independent inventory swapped member parent")
    return inventory


def validate_mapping(capture, authority):
    validate_schema(capture)
    validate_inventory(capture, authority)
    if capture.get("schema") != SCHEMA:
        raise Unsupported("wrong capability schema")
    if any("unsupported" in encoded(declaration["canonical_signature"]).decode() or isinstance(declaration["generic_bounds"], dict) for declaration in capture["declarations"]):
        raise Unsupported("unsupported canonical type/generic facts")
    catalog = capture["callable_catalog"]
    for path, descriptor in catalog.items():
        if descriptor["path"] != path or not descriptor["signature"] or not descriptor["owner"]:
            raise Unsupported("invalid compiler callable declaration catalog")
    declarations = capture["declarations"]
    ledger = capture["expanded_owner_ledger"]
    expanded = {entry["owner"]:entry for entry in ledger}
    owners = {declaration["owner"] for declaration in declarations}
    if len(expanded) != len(ledger) or owners != expanded.keys():
        raise Unsupported("missing/duplicate expanded AST declaration disposition")
    for declaration in declarations:
        if any(declaration[field] != expanded[declaration["owner"]][field] for field in ("token_sequence", "ast_body")):
            raise Unsupported("expanded AST owner/body witness conflict")
    for declaration in declarations:
        if declaration["owner_kind"] == "AssocFn":
            if declaration["parent"] not in owners:
                raise Unsupported("generated member has no actual impl owner")
        else:
            children = [child["owner"] for child in declarations if child["owner_kind"] == "AssocFn" and child["parent"] == declaration["owner"]]
            if len(children) != declaration["hir_impl_member_count"] or sorted(children) != sorted(declaration["hir_members"]):
                raise Unsupported(f"missing/unaccounted generated impl member: {declaration['owner']}")
    owners = set()
    invocations = {}
    for declaration in capture["declarations"]:
        owner = declaration["owner"]
        if owner in owners:
            raise Unsupported(f"duplicate declaration owner: {owner}")
        owners.add(owner)
        chains = declaration["expansion_chain"]
        if not chains or not chains[0]["macro"]:
            raise Unsupported(f"missing resolved expansion origin: {owner}")
        if chains[0]["macro_identity"] not in {"core::fmt::macros::Debug", "core::clone::Clone", "core::cmp::PartialEq", "core::marker::Copy"}:
            raise Unsupported(f"unproven derive kind: {owner} {chains[0]["macro_identity"]}")
        if not chains[0]["builtin"]:
            raise Unsupported(f"unadmitted non-builtin derive: {owner}")
        key = encoded(chains)
        invocations.setdefault(key, []).append(declaration)
        if not declaration["tokens"]:
            raise Unsupported(f"missing expanded tokens: {owner}")
        if normalized_body_tokens(declaration["ast_body_tokens"]) != normalized_body_tokens(declaration["hir_body_tokens"]):
            raise Unsupported(f"unaccounted AST/HIR body transformation: {owner}")
        if declaration["ast_body"] != declaration["hir_body"]:
            raise Unsupported(f"missing required actual body: {owner}")
        if declaration["ast_impl_member_count"] != declaration["hir_impl_member_count"]:
            raise Unsupported(f"unaccounted auxiliary members: {owner}")
        if declaration["owner_kind"] == "AssocFn" and not declaration["ast_body"]:
            raise Unsupported(f"empty generated method: {owner}")
        ast, typed = declaration["ast_sites"], declaration["typed_sites"]
        if len(ast) != len(typed):
            raise Unsupported(f"dropped/duplicate call or operator: {owner}")
        for index, (syntax, resolved) in enumerate(zip(ast, typed)):
            validate_resolution(resolved, catalog)
            for field in ("kind", "ancestor", "span", "expansion_chain", "token_sequence"):
                if syntax[field] != resolved[field]:
                    raise Unsupported(f"ambiguous AST/HIR structural join: {owner} {index} {field}")
            if syntax["ancestor"] is not None and not 0 <= syntax["ancestor"] < index:
                raise Unsupported(f"invalid structural ancestor: {owner} {index}")
            if syntax["span"]["quality"] != "coarse-generated-anchor":
                raise Unsupported(f"forged exact generated mapping: {owner} {index}")
            interval = syntax["token_interval"]
            if declaration["token_sequence"][interval[0]:interval[1]] != syntax["token_sequence"]:
                raise Unsupported(f"invalid generated token interval: {owner} {index}")
            if resolved["disposition"] == "unsupported-call":
                raise Unsupported(f"unresolved generated callable: {owner} {index}")
            if not resolved["result_type"]:
                raise Unsupported(f"unknown generated type: {owner} {index}")
    if not invocations:
        raise Unsupported("missing builtin invocation dispositions")
    return {"declarations": len(owners), "invocations": len(invocations), "typed_sites": sum(len(d["typed_sites"]) for d in capture["declarations"])}



def add_intervals(capture):
    for declaration in capture["declarations"]:
        sequence = declaration["token_sequence"]
        cursor = {None: 0}
        intervals = []
        for index, site in enumerate(declaration["ast_sites"]):
            parent = site["ancestor"]
            start, limit = (0, len(sequence)) if parent is None else intervals[parent]
            start = max(start, cursor.get(parent, start))
            tokens = site["token_sequence"]
            position = next((pos for pos in range(start, limit - len(tokens) + 1) if sequence[pos:pos + len(tokens)] == tokens), None)
            if position is None:
                raise Unsupported(f"missing structural token interval: {declaration['owner']} {index}")
            interval = [position, position + len(tokens)]
            site["token_interval"] = interval
            site["mapping"] = "owned-structural-one-to-one"
            intervals.append(interval)
            cursor[parent] = interval[1]
            cursor[index] = interval[0]


COMMON_DERIVES = {
    "core::fmt::macros::Debug": ("core::fmt::Debug", "fmt"),
    "core::clone::Clone": ("core::clone::Clone", "clone"),
    "core::cmp::PartialEq": ("core::cmp::PartialEq", "eq"),
    "core::marker::Copy": ("core::marker::Copy", None),
}


def validate_common_obligations(capture, common):
    """Invocation authority obligates common methods before examining compiler impls."""
    invocations = common["invocations"]
    identities = {}
    pairs = set()
    for invocation in invocations:
        key = encoded(invocation["identity"])
        pair = encoded([invocation["receiver"],invocation["macro"]])
        identity = [invocation[field] for field in ("receiver", "macro", "module", "source_range", "attribute_ordinal", "derive_ordinal")]
        if key in identities or pair in pairs or invocation["identity"] != identity:
            raise Unsupported("duplicate/swapped RA invocation ownership")
        receiver = invocation["receiver"].get("adt", "")
        if not receiver.startswith(invocation["module"] + "::") or receiver.rsplit("::",1)[-1] != invocation["source_name"] or not invocation["declaration_source"]:
            raise Unsupported("swapped RA invocation ADT/module source owner")
        if invocation["macro"] not in COMMON_DERIVES:
            raise Unsupported("unknown resolved RA builtin invocation")
        identities[key] = invocation
        pairs.add(pair)
    members = {}
    compiler = {encoded([d["receiver_identity"],d["trait_identity"],d["owner"].rsplit("::",1)[-1]]):d for d in capture["declarations"] if d["owner_kind"] == "AssocFn"}
    for member in common["common_members"]:
        if member["invocation"] is None:
            continue
        key = encoded(member["invocation"])
        if key not in identities:
            raise Unsupported("unknown RA common-member invocation")
        invocation = identities[key]
        trait_, method = COMMON_DERIVES[invocation["macro"]]
        if member["receiver_identity"] != invocation["receiver"] or member["trait_identity"] != trait_ or member["method"] != method or member["module"] != invocation["module"] or key in members:
            raise Unsupported("duplicate/swapped RA invocation common-member ownership")
        members[key] = member
        counterpart = encoded([invocation["receiver"],trait_,method])
        if counterpart not in compiler:
            raise Unsupported("invocation-owned missing compiler common impl/member")
        if compiler[counterpart]["expansion_chain"][0]["macro_identity"] != invocation["macro"]:
            raise Unsupported("invocation-owned compiler common origin conflict")
    for key, invocation in identities.items():
        if COMMON_DERIVES[invocation["macro"]][1] is not None and key not in members:
            raise Unsupported("invocation-owned missing RA common member")


def validate_join(capture, common, authority):
    validate_common_obligations(capture, common)
    validate_mapping(capture, authority)
    if common["producer"] != RA_COMMIT:
        raise Unsupported("wrong common-member producer")
    # Logical true/false are implicit language constants; RA lists true explicitly.
    resolver_cfg = sorted((atom for atom in common["cfg"] if atom["key"] not in ("true", "false")), key=lambda atom: encoded(atom))
    if resolver_cfg != capture["cfg"]:
        raise Unsupported(f"producer cfg conflict: compiler={capture['cfg']}, resolver={resolver_cfg}")
    compiler = {}
    for declaration in capture["declarations"]:
        if declaration["owner_kind"] != "AssocFn":
            continue
        key = encoded([declaration["receiver_identity"], declaration["trait_identity"], declaration["owner"].rsplit("::", 1)[-1]])
        if key in compiler:
            raise Unsupported("duplicate compiler common member")
        compiler[key] = declaration
    seen = set()
    joined = []
    for member in common["common_members"]:
        key = encoded([member["receiver_identity"], member["trait_identity"], member["method"]])
        # A resolver sees source implementations beyond this builtin-only surface.
        if key not in compiler:
            if member["invocation"] is not None:
                raise Unsupported("missing compiler common member/body record")
            continue
        if key in seen:
            raise Unsupported("ambiguous resolver common member")
        seen.add(key)
        declaration = compiler[key]
        normalized_bounds = []
        erased_bounds = []
        for bound in member["generic_bounds"]:
            kept = []
            for trait in bound["traits"]:
                if trait["trait"] == capture["lowering_erased_bound"]:
                    if trait["arguments"] != [bound["parameter"]] or trait["polarity"] != "Positive":
                        raise Unsupported("unsupported lowering-erased bound shape")
                    erased_bounds.append(trait)
                else:
                    kept.append(trait)
            normalized_bounds.append({"parameter": bound["parameter"], "traits": kept})
        for field, resolver_value in (("generic_bounds", normalized_bounds), ("visibility", member["visibility"])):
            if resolver_value != declaration[field]:
                raise Unsupported(f"common {field} conflict: {declaration['owner']}")
        if member["canonical_signature"] != declaration["canonical_signature"]:
            raise Unsupported(f"common source/signature conflict: {declaration['owner']}")
        joined.append({"owner": declaration["owner"], "receiver": declaration["receiver_identity"], "trait": declaration["trait_identity"], "signature": declaration["canonical_signature"], "compiler_generics": declaration["generic_bounds"], "resolver_generics": member["generic_bounds"], "generic_relation": {"kind": "compiler-lowering-erases-lang-item-bound", "resolved_lang_item": capture["lowering_erased_bound"], "erased": erased_bounds}})
    if seen != compiler.keys():
        raise Unsupported("missing one-to-one common member")
    invocation_join = validate_invocation_multisets(capture, common)
    by_owner = {owner:entry for entry in invocation_join for owner in entry["compiler_output_multiset"]}
    for joined_member in joined:
        entry = by_owner[joined_member["owner"]]
        joined_member["invocation"] = entry["invocation"]
        joined_member["compiler_output_multiset"] = entry["compiler_output_multiset"]
    return joined


def validate_invocation_multisets(capture, common):
    """Retain all invocation-owned outputs, including invocations with no common method."""
    outputs = {}
    for declaration in capture["declarations"]:
        key = encoded([declaration["receiver_identity"],declaration["expansion_chain"][0]["macro_identity"]])
        outputs.setdefault(key, []).append(declaration["owner"])
    obligations = {encoded([invocation["receiver"],invocation["macro"]]):invocation for invocation in common["invocations"]}
    if len(obligations) != len(common["invocations"]) or outputs.keys() != obligations.keys():
        raise Unsupported("missing/duplicate builtin invocation ownership join")
    return [{"invocation":obligations[key]["identity"],"compiler_output_multiset":sorted(owners)} for key,owners in sorted(outputs.items())]


def verify_capture(capture, receipt, input_identity, expected_inventory):
    """Admission requires the authority held by the caller from the real compiler run."""
    if expected_inventory is None:
        raise Unsupported("missing independent inventory expected authority")
    if receipt["inputs"] != input_identity:
        raise Unsupported("source/extern/configuration/context drift")
    current_environment = {key:digest(value.encode()) for key,value in os.environ.items() if not key.startswith("SIFR_BUILTIN_")}
    if current_environment != input_identity["parent_environment"]:
        raise Unsupported("parent compiler/preparation environment drift")
    for name, sha256 in input_identity["tool"]["consumer_sources"].items():
        if digest((ROOT / "scripts" / name).read_bytes()) != sha256:
            raise Unsupported("consumer/instrumentation source drift")
    root = Path(input_identity["input_root"])
    for name, sha256 in input_identity["files"].items():
        path = Path(name)
        if not path.is_absolute():
            path = root / path
        if not path.is_file() or digest(path.read_bytes()) != sha256:
            raise Unsupported(f"source/extern/filesystem input drift: {path}")
    for name, members in input_identity["directory_members"].items():
        path = root / name[len("checkout:/"):] if name.startswith("checkout:/") else Path(name)
        excluded = set()
        if isinstance(members, dict):
            excluded = {".git", "target", "__pycache__"} if members["exclude_operational"] else set()
            members = members["members"]
        if not path.is_dir() or sorted(str(p.relative_to(path)) for p in path.rglob("*") if p.is_file() and not set(p.relative_to(path).parts).intersection(excluded)) != members:
            raise Unsupported(f"build-script directory membership drift: {path}")
    for name, sha256 in input_identity["tool"]["helper_sources"].items():
        if digest((TOOL / name).read_bytes()) != sha256:
            raise Unsupported("helper input drift")
    for name, sha256 in input_identity["tool"]["helper_build"]["executables_and_runtime"].items():
        if digest(Path(name).read_bytes()) != sha256:
            raise Unsupported("helper executable/runtime drift")
    if digest(encoded(capture)) != receipt["capture_digest"]:
        raise Unsupported("producer evidence changed or truncated")
    inventory = read_inventory(receipt, input_identity)
    if inventory != expected_inventory:
        raise Unsupported("independent inventory replacement expected authority")
    return validate_mapping(capture, expected_inventory)


def compiler_wrapper():
    """Capture the selected invocation; stopping analysis never supplies a Cargo artifact."""
    args = sys.argv[2:]
    compiler = args.pop(0)
    selected = os.environ["SIFR_BUILTIN_SELECTED_CRATE"]
    if "--crate-name" in args and args[args.index("--crate-name") + 1] == selected:
        destination = Path(os.environ["SIFR_BUILTIN_CAPTURE_DIR"])
        destination.mkdir(parents=True, exist_ok=True)
        baseline = json.loads((destination / "baseline.json").read_text())
        (destination / "invocation.json").write_text(json.dumps({"version": 2, "cwd": os.getcwd(), "compiler": compiler, "args": args, "environment_removed": [key for key in baseline if key not in os.environ and not key.startswith("SIFR_BUILTIN_")], "environment_delta": {key: value for key, value in os.environ.items() if not key.startswith("SIFR_BUILTIN_") and baseline.get(key) != value}, "inherited_environment": {key: digest(value.encode()) for key, value in baseline.items() if not key.startswith("SIFR_BUILTIN_")}}, indent=2))
        # Real locked Cargo preparation must receive actual artifacts.
        return subprocess.run([compiler, *args]).returncode
    return subprocess.run([compiler, *args]).returncode


def capture_package(root, package, selected_crate, output, helper, target, identity):
    started = time.monotonic()
    root, output = Path(root).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    environment.pop("RUSTC_BOOTSTRAP", None)
    environment.pop("RUSTC_WORKSPACE_WRAPPER", None)
    environment.pop("RUSTC_WRAPPER", None)
    inherited = {key: digest(value.encode()) for key, value in environment.items() if not key.startswith("SIFR_BUILTIN_")}
    wrapper_identity = digest(encoded([str(root), str(output), inherited, digest(Path(__file__).read_bytes())]))
    wrapper = Path(target) / "builtin-wrappers" / wrapper_identity / "capture-wrapper"
    wrapper.parent.mkdir(parents=True, exist_ok=True)
    if not wrapper.exists():
        wrapper.symlink_to(Path(__file__).resolve())
    invocation_store = Path(target) / "builtin-invocations" / (selected_crate + "-" + wrapper_identity)
    invocation_store.mkdir(parents=True, exist_ok=True)
    environment.update({"CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "2", "CARGO_TARGET_DIR": str(target), "RUSTC_WORKSPACE_WRAPPER": str(wrapper), "SIFR_BUILTIN_SELECTED_CRATE": selected_crate, "SIFR_BUILTIN_CAPTURE_DIR": str(invocation_store), "SIFR_BUILTIN_HELPER": str(helper)})
    environment.pop("RUSTC_WRAPPER", None)
    (invocation_store / "baseline.json").write_bytes(encoded(environment))
    # Workspace wrapper participates in Cargo artifact identity; dependency metadata remains warm.
    result = run(["cargo", "check", "--locked", "--message-format=json", "-p", package, "--lib", "--target", "x86_64-unknown-linux-gnu"], cwd=root, env=environment, log=output / "cargo-preparation.log")
    if not (invocation_store / "invocation.json").is_file():
        raise Unsupported("selected exact Cargo invocation is unavailable")
    invocation = json.loads((invocation_store / "invocation.json").read_text())
    if invocation.get("version") != 2:
        raise Unsupported("obsolete compiler invocation context")
    inherited = {key: digest(value.encode()) for key, value in environment.items() if not key.startswith("SIFR_BUILTIN_")}
    if invocation["inherited_environment"] != inherited:
        raise Unsupported("inherited compiler environment drift")
    if Path(invocation["cwd"]).resolve() != root:
        raise Unsupported("selected invocation belongs to a different checkout")
    messages = [json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]
    selected = [message for message in messages if message.get("reason") == "compiler-artifact" and message["target"]["name"] == selected_crate and "lib" in message["target"]["kind"]]
    if len(selected) != 1:
        raise Unsupported("missing or ambiguous actual selected Cargo package/target")
    metadata_result = run(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", "x86_64-unknown-linux-gnu"], cwd=root, env=environment, log=output / "cargo-metadata.log")
    metadata = json.loads(metadata_result.stdout)
    package_roots = {entry["id"]: Path(entry["manifest_path"]).parent for entry in metadata["packages"]}
    build_files = [Path(entry["manifest_path"]) for entry in metadata["packages"]]
    build_environment = {}
    build_contexts = []
    directory_members = {}
    for message in messages:
        if message.get("reason") != "build-script-executed":
            continue
        build_contexts.append(message)
        output_dir = Path(message["out_dir"])
        build_output = output_dir.parent / "output"
        if not build_output.is_file():
            raise Unsupported("missing actual Cargo build-script input declarations")
        build_files.extend(path for path in output_dir.rglob("*") if path.is_file())
        build_files.append(build_output)
        for line in build_output.read_text().splitlines():
            if line.startswith(("cargo:rerun-if-changed=", "cargo::rerun-if-changed=")):
                path = Path(line.split("=", 1)[1])
                if not path.is_absolute():
                    path = package_roots[message["package_id"]] / path
                if not path.exists():
                    raise Unsupported(f"unreadable declared build-script input: {path}")
                if path.is_dir():
                    members = sorted(str(p.relative_to(path)) for p in path.rglob("*") if p.is_file())
                    directory_members[str(path)] = members
                    build_files.extend(path / member for member in members)
                else:
                    build_files.append(path)
            elif line.startswith(("cargo:rerun-if-env-changed=", "cargo::rerun-if-env-changed=")):
                key = line.split("=", 1)[1]
                build_environment[key] = environment.get(key)
    extern_files = [Path(arg.split("=", 1)[1]) for arg in invocation["args"] if "=" in arg and arg.split("=", 1)[1].endswith((".rmeta", ".rlib", ".so"))]
    prepared_files = {str(path): digest(path.read_bytes()) for path in set(build_files + extern_files)}
    if run([invocation["compiler"], "-vV"]).stdout != identity["rustc"]:
        raise Unsupported("selected Cargo compiler identity mismatch")
    prepared_files[invocation["compiler"]] = digest(Path(invocation["compiler"]).read_bytes())
    artifact_owners = {}
    package_sources = {}
    for message in messages:
        if message.get("reason") != "compiler-artifact":
            continue
        origin = message["package_id"].replace("path+file://" + str(root), "checkout:")
        for filename in message["filenames"]:
            path = Path(filename).resolve()
            artifact_owners[str(path)] = origin
            if path.suffix in {".rmeta", ".rlib", ".so"}:
                prepared_files[str(path)] = digest(path.read_bytes())
        package_sources[message["package_id"]] = package_roots[message["package_id"]]
    for package_root in package_sources.values():
        members = sorted(str(path.relative_to(package_root)) for path in package_root.rglob("*") if path.is_file() and not set(path.relative_to(package_root).parts).intersection({".git", "target", "__pycache__"}))
        directory_members[str(package_root)] = {"members": members, "exclude_operational": True}
        for member in members:
            path = package_root / member
            prepared_files[str(path)] = digest(path.read_bytes())
    prepared_files.update(identity["selected_sysroot_metadata"])
    for name, sha256 in prepared_files.items():
        if digest(Path(name).read_bytes()) != sha256:
            raise Unsupported(f"prepared input changed before analysis: {name}")
    (output / "invocation.json").write_bytes(encoded(invocation))
    for key in invocation["environment_removed"]:
        environment.pop(key, None)
    environment.update(invocation["environment_delta"])
    if "RUSTC_BOOTSTRAP" in environment:
        raise Unsupported("bootstrap leaked into analyzed context")
    sysroot = Path(run(["rustc", "--print", "sysroot"]).stdout.strip())
    environment["SIFR_BUILTIN_CAPTURE"] = str(output / "raw.json")
    environment["SIFR_BUILTIN_INVENTORY"] = str(output / "raw-inventory.json")
    environment["SIFR_BUILTIN_SOURCE_SUFFIX"] = "crates/sifr_codegen/src/rust_ir.rs" if package == "sifr_codegen" else "fixture_root/src/lib.rs"
    analysis_args = invocation["args"].copy()
    if "--out-dir" not in analysis_args:
        raise Unsupported("original selected output context is unavailable")
    analysis_output = output / "compiler-output"
    analysis_output.mkdir()
    analysis_args[analysis_args.index("--out-dir") + 1] = str(analysis_output)
    analysis = run([str(helper), *analysis_args], cwd=root, env=environment, log=output / "compiler-analysis.log")
    if not (output / "raw.json").is_file():
        raise Unsupported("selected typechecking/expansion failed; no complete capture")
    # Rebind all original Rust inputs and actual externally prepared metadata.
    raw = json.loads((output / "raw.json").read_text())
    for descriptor in raw["callable_catalog"].values():
        paths = descriptor.pop("origin_paths")
        if paths is None:
            descriptor["owner"] = selected[0]["package_id"].replace("path+file://" + str(root), "checkout:")
        else:
            owners = {("sysroot:" + RUST_COMMIT) if Path(name).resolve().is_relative_to(sysroot) else artifact_owners.get(str(Path(name).resolve())) for name in paths}
            if None in owners or len(owners) != 1 or any(str(Path(name).resolve()) not in prepared_files for name in paths):
                raise Unsupported("unbound callable catalog origin")
            descriptor["owner"] = owners.pop()
    for declaration in raw["declarations"]:
        for site in declaration["typed_sites"]:
            for field in ("target", "implementation"):
                paths = site.pop(field + "_origin_paths")
                if paths is None:
                    site[field + "_owner"] = selected[0]["package_id"].replace("path+file://" + str(root), "checkout:") if site[field] else None
                    continue
                owners = set()
                for name in paths:
                    path = Path(name).resolve()
                    if path.is_relative_to(sysroot):
                        owners.add("sysroot:" + RUST_COMMIT)
                    elif str(path) in artifact_owners:
                        owners.add(artifact_owners[str(path)])
                    else:
                        raise Unsupported(f"unreadable/unowned actual dependency metadata: {path}")
                    if str(path) not in prepared_files:
                        raise Unsupported(f"dependency metadata lacks original preparation binding: {path}")
                if len(owners) != 1:
                    raise Unsupported("ambiguous dependency callable ownership")
                site[field + "_owner"] = owners.pop()
    for path, sha256 in prepared_files.items():
        if digest(Path(path).read_bytes()) != sha256:
            raise Unsupported(f"prepared input changed during analysis: {path}")
    paths = [Path(path) for path in prepared_files]
    for source in raw.pop("source_files"):
        path = Path(source["file"])
        if not path.is_absolute():
            path = root / path
        if not path.is_file() or digest(path.read_bytes()) != digest(source["text"].encode()):
            raise Unsupported(f"compiler source missing or changed: {path}")
        paths.append(path)
    paths += [p for p in root.rglob("Cargo.toml") if "target" not in p.parts and "third_party" not in p.parts and "vendor" not in p.parts]
    paths += [root / "Cargo.lock"]
    paths += [p for p in (root / ".cargo").glob("config*") if p.is_file()]
    for arg in invocation["args"]:
        if "=" in arg and arg.split("=", 1)[1].endswith((".rmeta", ".rlib", ".so")):
            paths.append(Path(arg.split("=", 1)[1]))
    inputs = {str(p.relative_to(root)) if p.is_relative_to(root) else str(p): digest(p.read_bytes()) for p in sorted(set(paths))}
    config = {"resolver_preparation_environment": {key:environment[key] for key in ("CARGO_INCREMENTAL", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "RUSTC_WORKSPACE_WRAPPER", "SIFR_BUILTIN_SELECTED_CRATE", "SIFR_BUILTIN_CAPTURE_DIR")}, "parent_environment": {key:digest(value.encode()) for key,value in os.environ.items() if not key.startswith("SIFR_BUILTIN_")}, "input_root": str(root), "build_script_parent_environment": build_environment, "cargo_build_contexts": normalize(build_contexts, root), "cargo_metadata_digest": digest(encoded(normalize(metadata, root))), "directory_members": normalize(directory_members, root), "cargo_package_id": normalize(selected[0]["package_id"].replace("path+file://" + str(root), "checkout:"), root), "package": package, "target": "x86_64-unknown-linux-gnu", "test": False, "tool": identity, "files": inputs, "invocation": normalize(invocation, root)}
    sysroot = Path(run(["rustc", "--print", "sysroot"]).stdout.strip())
    capture = normalize(raw, root, sysroot)
    capture["context"] = {"crate":selected_crate,"package": config["cargo_package_id"], "target": config["target"], "test": config["test"]}
    for declaration in capture["declarations"]:
        kind = declaration["owner_kind"]
        declaration["output_role"] = "common-primary" if declaration["trait_identity"] == COMMON_DERIVES[declaration["expansion_chain"][0]["macro_identity"]][0] else "compiler-only-auxiliary"
        if kind == "AssocFn":
            declaration["declaration_disposition"] = "typed-generated-body"
        elif kind.startswith("Impl"):
            declaration["declaration_disposition"] = "bodyless-impl" if declaration["ast_impl_member_count"] == 0 else "impl-with-members"
        else:
            raise Unsupported(f"unaccounted generated auxiliary item: {declaration['owner']}")
    add_intervals(capture)
    inventory = normalize(json.loads((output / "raw-inventory.json").read_text()), root, sysroot)
    capture["inventory_digest"] = digest(encoded(inventory))
    authority = {"inventory":inventory,"context":capture["context"],"cfg":capture["cfg"],"input_digest":digest(encoded(config))}
    authority_path = output / "inventory-authority.json"
    authority_path.write_bytes(encoded(authority))
    reference = {"path":str(authority_path),"digest":digest(authority_path.read_bytes())}
    config["inventory_authority"] = reference
    counts = validate_mapping(capture, authority)
    receipt = {"inventory_authority":reference,"timing_seconds": {"locked_preparation": result.elapsed_seconds, "metadata": metadata_result.elapsed_seconds, "compiler_analysis": analysis.elapsed_seconds, "total": time.monotonic() - started}, "inputs": config, "capture_digest": digest(encoded(capture)), "counts": counts, "preparation_cargo_artifact_success": True, "capture_cargo_artifact_success": False, "cargo_status": result.returncode, "capture_stopped_after_analysis": True, "cfg_relation": "actual compiler cfg applied via public resolver CfgOverrides, then correspondence verified", "preparation_reuse": "capture-owned wrapper forces a fresh selected invocation; compatible dependency artifacts remain warm", "analysis_output_transform": "only --out-dir redirected to owned evidence; original Cargo args retained"}
    (output / "capture.json").write_bytes(encoded(capture))
    (output / "receipt.json").write_bytes(encoded(receipt))
    return capture, receipt


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--rustc-wrapper":
        sys.exit(compiler_wrapper())
    # Cargo calls the wrapper as [wrapper, rustc, ...].
    if len(sys.argv) > 1 and Path(sys.argv[1]).name.startswith("rustc"):
        sys.argv.insert(1, "--rustc-wrapper")
        sys.exit(compiler_wrapper())
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--component-receipt", required=True)
    parser.add_argument("--output-dir", required=True)
    parser.add_argument("--target-dir", required=True)
    parser.add_argument("--helper", required=True)
    args = parser.parse_args()
    try:
        identity = tool_identity(args.component_receipt, args.target_dir)
        capture, receipt = capture_package(ROOT, "sifr_codegen", "sifr_codegen", args.output_dir, args.helper, args.target_dir, identity)
        print(json.dumps(receipt["counts"], sort_keys=True))
    except (Unsupported, OSError, KeyError, ValueError) as error:
        print(f"unsupported builtin capability: {error}", file=sys.stderr)
        sys.exit(2)
