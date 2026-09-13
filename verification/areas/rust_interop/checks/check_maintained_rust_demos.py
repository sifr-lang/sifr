"""Compile tracked positive idiomatic Rust demos; never execute their programs."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[4]
TEST_REFERENCES = {f"demos/{name}/idiomatic.rs" for name in
                   ("test_runner", "test_runner_imports", "test_imports_and_constants",
                    "temp_workspace_isolation/parallel_tests/a",
                    "temp_workspace_isolation/parallel_tests/b")}


def discover(listing: bytes) -> tuple[list[str], list[str]]:
    positive, negative = [], []
    for raw in listing.split(b"\0"):
        if not raw:
            continue
        path = raw.decode()
        parts = Path(path).parts
        if not parts or parts[0] != "demos" or parts[-1] != "idiomatic.rs":
            continue
        if any(part in ("..", ".") for part in parts) or Path(path).is_absolute():
            raise ValueError(f"invalid tracked demo path: {path}")
        (negative if "negative_cases" in parts else positive).append(path)
    if len(set(positive + negative)) != len(positive + negative):
        raise ValueError("duplicate tracked demo")
    if not positive:
        raise ValueError("no tracked positive Rust demos")
    return sorted(positive), sorted(negative)


def tracked_sources() -> tuple[list[str], list[str], dict[str, str]]:
    entries = subprocess.check_output(["git", "ls-files", "--stage", "-z", "--", "demos"], cwd=ROOT)
    paths, hashes = [], {}
    for entry in entries.split(b"\0"):
        if not entry:
            continue
        header, raw = entry.split(b"\t", 1)
        path = raw.decode()
        if not path.endswith(".rs"):
            continue
        mode, _, stage = header.split()
        if mode not in (b"100644", b"100755") or stage != b"0":
            raise ValueError(f"non-regular or conflicted tracked Rust source: {path}")
        source = ROOT / path
        if source.resolve() != source or not stat.S_ISREG(source.lstat().st_mode):
            raise ValueError(f"symlink or non-regular Rust source: {path}")
        hashes[path] = hashlib.sha256(source.read_bytes()).hexdigest()
        paths.append(raw)
    positive, negative = discover(b"\0".join(paths))
    return positive, negative, hashes


def select_dependencies(source: str, workspace: dict, packages: list[dict]) -> dict:
    selected = {}
    # Standard-library import trees cannot identify registry dependencies.
    # Match qualified paths from their root, not a suffix such as std::time.
    references = re.sub(r"\buse\s+(?:::)?std\s*::[^;]*;", "", source)
    roots = set(re.findall(r"(?<![\w:])([A-Za-z_]\w*)\s*::(?:\s*[A-Za-z_]\w*\s*::)*", references))
    # Discover the real maintained candidate names, not a manually incomplete
    # allow-list. Workspace aliases take precedence over registry spellings.
    names = {row["name"].replace("-", "_"): row["name"] for row in packages if "source" in row}
    for spec in workspace.values():
        if isinstance(spec, dict) and "package" in spec:
            names[spec["package"].replace("-", "_")] = spec["package"]
    names.update({name.replace("-", "_"): name for name in workspace})
    # This existing reference import has no current maintained selection; keep
    # it explicit so reaching it reports the input gap, not a guessed version.
    names.update(fs2="fs2", sifr_stdlib="sifr_stdlib")
    for rust_name, name in sorted(names.items()):
        if rust_name not in roots:
            continue
        if name == "sifr_stdlib":
            selected[name] = {"path": str(ROOT / "crates/sifr_stdlib"), "features": ["runtime-observability"]}
        elif name in workspace:
            selected[name] = workspace[name]
        else:
            aliases = [spec for spec in workspace.values()
                       if isinstance(spec, dict) and spec.get("package") == name]
            if aliases:
                if any(spec != aliases[0] for spec in aliases):
                    raise ValueError(f"demo crate {name} has conflicting maintained alias policies")
                # The reference may spell the package name rather than the
                # workspace's versioned alias. Preserve its complete policy,
                # particularly default-features=false; version alone is not it.
                selected[name] = aliases[0]
                continue
            raise ValueError(f"demo crate {name} has no canonical maintained manifest policy; a lock version is not authority")
    return selected


def inline(value: object) -> str:
    if isinstance(value, dict):
        return "{ " + ", ".join(f"{key} = {inline(item)}" for key, item in value.items()) + " }"
    return json.dumps(value)


def registry_identities(packages: list[dict]) -> set[tuple[str, str, str, str]]:
    return {(row["name"], row["version"], row["source"], row["checksum"])
            for row in packages if row.get("source", "").startswith("registry+")}


def cargo_commands(cargo: str, manifest: Path, section: str) -> list[list[str]]:
    # Metadata may prune the exact root-lock seed for this genuine consumer.
    # generate-lockfile would instead select fresh unrelated transitive versions.
    return [[cargo, "metadata", "--offline", "--format-version", "1", "--manifest-path", str(manifest)],
            [cargo, "check", "--locked", "--offline", "-j2", "--manifest-path", str(manifest),
             "--tests" if section == "test" else "--bins"]]



def dependency_group_key(dependencies: dict, section: str) -> str:
    return json.dumps([section, dependencies], sort_keys=True, separators=(",", ":"))


def compile_demos(cargo: str, destination: Path) -> int:
    positive, negative, source_hashes = tracked_sources()
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["dependencies"]
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    root_inputs = {name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
                   for name in ("Cargo.toml", "Cargo.lock")}
    destination.mkdir(parents=True, exist_ok=False)
    report = {"tracked_positive": positive, "intentional_negative_cases": negative,
              "excluded_from_positive_compile": "negative_cases are negative fixtures, not claimed passing demos",
              "tracked_rust_sources_sha256": source_hashes,
              "root_inputs_sha256": root_inputs,
              "qualification": "Cargo check only; no execution or runtime/parity claim",
              "results": [], "status": "FAIL"}
    try:
        groups = {}
        for index, relative in enumerate(positive):
            source = ROOT / relative
            row = {"path": relative, "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest()}
            report["results"].append(row)
            try:
                dependencies = select_dependencies(source.read_text(), workspace, packages)
            except ValueError as error:
                row.update(status="BLOCKED_INPUT", error=str(error))
                return 1
            section = "test" if relative in TEST_REFERENCES else "bin"
            key = dependency_group_key(dependencies, section)
            group = groups.setdefault(key, {"selected_dependencies": dependencies,
                                            "section": section, "members": []})
            group["members"].append((index, row))
        report["groups"] = []
        for group_index, group in enumerate(groups.values()):
            dependencies, section = group["selected_dependencies"], group["section"]
            members = group["members"]
            project = destination / f"group-{group_index:03d}"
            project.mkdir()
            manifest = ('[package]\nname = "sifr-idiomatic-demo"\nversion = "0.0.0"\nedition = "2024"\n'
                        '[workspace]\nresolver = "3"\n[dependencies]\n' +
                        "".join(f"{name} = {inline(spec)}\n" for name, spec in dependencies.items()))
            if dependencies:
                native = ROOT / "crates/sifr_runtime/third_party/libsqlite3-sys"
                manifest += ('[patch.crates-io]\nlibsqlite3-sys = { path = '
                             + json.dumps(str(native)) + ' }\n')
            for index, row in members:
                manifest += (f'[[{section}]]\nname = "idiomatic_{index:03d}"\n'
                             f'path = {json.dumps(str(ROOT / row["path"]))}\n')
                row.update(group=group_index, selected_dependencies=dependencies)
            (project / "Cargo.toml").write_text(manifest)
            # Only identical complete dependency policies share a Cargo root.
            # Every source remains a separate target; test-only references stay
            # separate from binaries. No feature union crosses policy groups.
            shutil.copyfile(ROOT / "Cargo.lock", project / "Cargo.lock")
            commands = cargo_commands(cargo, project / "Cargo.toml", section)
            receipt = {"members": [row["path"] for _, row in members],
                       "selected_dependencies": dependencies, "section": section,
                       "manifest_sha256": hashlib.sha256(manifest.encode()).hexdigest(),
                       "root_lock_seed_sha256": root_inputs["Cargo.lock"], "commands": []}
            report["groups"].append(receipt)
            for argv in commands:
                output = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, timeout=180, check=False)
                receipt["commands"].append({"argv": argv, "exit": output.returncode,
                                            "stdout": output.stdout, "stderr": output.stderr})
                if output.returncode:
                    for _, row in members:
                        row["status"] = "FAIL"
                    return 1
                lock = project / "Cargo.lock"
                actual = registry_identities(tomllib.loads(lock.read_text())["package"])
                drift = actual - registry_identities(packages)
                if drift:
                    receipt.update(status="BLOCKED_INPUT", unexpected_registry_identities=sorted(drift))
                    return 1
                digest = hashlib.sha256(lock.read_bytes()).hexdigest()
                if "prepared_lock_sha256" in receipt and receipt["prepared_lock_sha256"] != digest:
                    raise ValueError(f"locked demo check changed {lock}")
                receipt["prepared_lock_sha256"] = digest
            receipt["status"] = "PASS"
            for _, row in members:
                row["status"] = "PASS"
        report["status"] = "PASS"
        return 0
    finally:
        try:
            report["sources_unchanged"] = (tracked_sources() == (positive, negative, source_hashes)
                and all(hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest
                        for name, digest in root_inputs.items()))
        except (OSError, ValueError, subprocess.CalledProcessError) as error:
            report.update(sources_unchanged=False, input_error=str(error))
        if not report["sources_unchanged"]:
            report["status"] = "FAIL"
        (destination / "result.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
        print(f"maintained idiomatic demo compile: {report['status']}; completed={sum(row.get('status') == 'PASS' for row in report['results'])}/{len(positive)}; report={destination / 'result.json'}")
        if not report["sources_unchanged"]:
            raise ValueError("tracked Rust demo inputs changed during compilation")


def self_test() -> None:
    policy = {"dep": {"version": "1", "default-features": False, "features": ["one"]}}
    same_policy = {"dep": {"features": ["one"], "version": "1", "default-features": False}}
    assert dependency_group_key(policy, "bin") == dependency_group_key(same_policy, "bin")
    assert dependency_group_key(policy, "bin") != dependency_group_key(policy, "test")
    for changed in (
        {"dep": {"version": "2", "default-features": False, "features": ["one"]}},
        {"dep": {"version": "1", "default-features": True, "features": ["one"]}},
        {"dep": {"version": "1", "default-features": False, "features": ["two"]}},
        {"alias": {"package": "dep", "version": "1", "default-features": False, "features": ["one"]}},
    ):
        assert dependency_group_key(policy, "bin") != dependency_group_key(changed, "bin")
    positive, negative = discover(b"demos/a/idiomatic.rs\0demos/a/negative_cases/x/idiomatic.rs\0demos/a/emitted.rs\0")
    assert positive == ["demos/a/idiomatic.rs"] and len(negative) == 1
    assert select_dependencies("use itertools::Itertools;", {"itertools": {"version": "0.15.0", "default-features": False}}, []) == {"itertools": {"version": "0.15.0", "default-features": False}}
    assert select_dependencies("use serde_json::Value;", {"serde_json": {"version": "1.0.151"}}, []) == {"serde_json": {"version": "1.0.151"}}
    assert select_dependencies("zip::ZipWriter::new(file)", {"zip": {"version": "8"}}, []) == {"zip": {"version": "8"}}
    assert select_dependencies("new_crate::call()", {"new-crate": {"version": "1"}}, []) == {"new-crate": {"version": "1"}}
    time_lock = [{"name": "time", "version": "1", "source": "registry+official"}]
    assert select_dependencies("std::time::Instant::now(); std :: time :: Instant::now(); use std::{time::Instant};", {}, time_lock) == {}
    zip_policy = {"package": "zip", "version": "8.6.0", "default-features": False, "features": ["deflate"]}
    zip_lock = [{"name": "zip", "version": "8.6.0", "source": "registry+official"}]
    assert select_dependencies("zip::ZipWriter::new(file)", {"zip_8_6": zip_policy}, zip_lock) == {"zip": zip_policy}
    assert select_dependencies("zip::ZipWriter::new(file)", {"zip_8_6": zip_policy}, []) == {"zip": zip_policy}
    try:
        select_dependencies("zip::ZipWriter::new(file)", {"zip_8_6": zip_policy, "zip_other": dict(zip_policy, features=["aes-crypto"])}, zip_lock)
    except ValueError:
        pass
    else:
        raise AssertionError("conflicting alias feature policies accepted")
    try:
        select_dependencies("use fs2::total_space;", {}, [])
    except ValueError:
        pass
    else:
        raise AssertionError("missing maintained dependency selection accepted")
    try:
        select_dependencies("transitive::call()", {}, [{"name": "transitive", "version": "1", "source": "registry+official"}])
    except ValueError:
        pass
    else:
        raise AssertionError("a lock-only identity was substituted for manifest authority")
    assert TEST_REFERENCES and all(path in discover((path + "\0").encode())[0] for path in TEST_REFERENCES)
    for sibling in ("a", "b"):
        test_reference = f"demos/temp_workspace_isolation/parallel_tests/{sibling}/idiomatic.rs"
        runnable_reference = f"demos/temp_workspace_isolation/parallel_runs/{sibling}/idiomatic.rs"
        assert test_reference in TEST_REFERENCES
        assert runnable_reference not in TEST_REFERENCES
        positive, negative = discover((test_reference + "\0" + runnable_reference + "\0"
            + "demos/temp_workspace_isolation/negative_cases/reachable_parse_error/idiomatic.rs\0").encode())
        assert test_reference in positive and runnable_reference in positive and len(negative) == 1
        assert not TEST_REFERENCES.intersection(negative)
    assert registry_identities([{"name": "local", "version": "0"}]) == set()
    identity = {"name": "dep", "version": "1", "source": "registry+official", "checksum": "abc"}
    assert registry_identities([identity]) == {("dep", "1", "registry+official", "abc")}
    commands = cargo_commands("/pinned/cargo", Path("/owned/Cargo.toml"), "bin")
    assert commands[0][1:6] == ["metadata", "--offline", "--format-version", "1", "--manifest-path"]
    assert "--locked" not in commands[0] and "--locked" in commands[1]
    assert commands[1][-1] == "--bins"
    assert cargo_commands("/pinned/cargo", Path("/owned/Cargo.toml"), "test")[1][-1] == "--tests"
    for invalid in (b"", b"demos/../idiomatic.rs\0", b"demos/a/idiomatic.rs\0demos/a/idiomatic.rs\0"):
        try:
            discover(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid or empty discovery accepted")
    print("maintained Rust demo discovery/manifest self-test: PASS; no compile claim")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--inventory-only", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if args.inventory_only:
        positive, negative, _ = tracked_sources()
        print(json.dumps({"positive": positive, "negative": negative}, indent=2))
        return 0
    cargo = shutil.which("cargo")
    if cargo is None:
        raise ValueError("pinned Cargo unavailable")
    output = args.output or Path(tempfile.mkdtemp(prefix="sifr-demo-compile-parent-", dir=ROOT / "target")) / "compile"
    return compile_demos(cargo, output)


if __name__ == "__main__":
    raise SystemExit(main())
