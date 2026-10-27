"""Qualify native SQL executables independently of the optional embedded compiler API."""
from __future__ import annotations

import argparse
import copy
import importlib.util
import json
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
TOOLS = tuple(f"sifr_sql_{family}_tools" for family in ("mysql", "postgresql", "sqlite"))


def validate(manifests: dict[str, dict]) -> None:
    for name in TOOLS:
        manifest = manifests[name]
        if manifest.get("features") != {
            "default": [], "compiler": ["dep:sifr_semantic_services"],
        }:
            raise ValueError(f"{name}: native default or optional compiler adapter drift")
        if manifest["dependencies"].get("sifr_semantic_services") != {
            "workspace": True, "optional": True,
        }:
            raise ValueError(f"{name}: compiler must be optional and workspace qualified")
        bins = manifest.get("bin", [])
        if len(bins) != 1 or bins[0].get("required-features", []) != []:
            raise ValueError(f"{name}: ordinary binary must not require the embedded compiler")


def validate_tree(text: str) -> None:
    names = {line.split()[0] for line in text.splitlines() if line.strip()}
    if not set(TOOLS) <= names:
        raise ValueError("native tool tree omitted a selected tool library")
    forbidden = {name for name in names if name in {
        "sifr_semantic_services", "sifr_frontend", "sifr_lowering", "sifr_codegen",
    } or name.startswith(("wasmtime", "cranelift", "pulley"))}
    if forbidden:
        raise ValueError(f"native tool graph contains compiler capabilities: {sorted(forbidden)}")


def self_test(manifests: dict[str, dict]) -> int:
    mutations = []
    for name in TOOLS:
        for label, change in (
            ("default", lambda m: m["features"].update(default=["compiler"])),
            ("feature-edge", lambda m: m["features"].update(compiler=[])),
            ("required-bin", lambda m: m["bin"][0].update({"required-features": ["compiler"]})),
            ("unconditional", lambda m: m["dependencies"]["sifr_semantic_services"].update(optional=False)),
        ):
            mutated = copy.deepcopy(manifests)
            change(mutated[name])
            mutations.append((name + "-" + label, mutated))
    for label, mutated in mutations:
        try:
            validate(mutated)
        except ValueError:
            continue
        raise ValueError(f"accepted manifest mutation: {label}")
    baseline = "\n".join(f"{name} v0.0.0" for name in TOOLS)
    validate_tree(baseline)
    trees = [baseline.replace(TOOLS[0], "missing-tool")]
    trees.extend(baseline + "\n" + name + " v0.0.0" for name in (
        "sifr_semantic_services", "sifr_frontend", "sifr_lowering", "sifr_codegen",
        "wasmtime", "wasmtime-internal-component-macro", "cranelift-codegen", "pulley-interpreter",
    ))
    for text in trees:
        try:
            validate_tree(text)
        except ValueError:
            continue
        raise ValueError("accepted dependency-tree mutation")
    return len(mutations) + len(trees)


def execution_commands() -> dict[str, list[str]]:
    path = ROOT / "verification/areas/sql_platform/runner.py"
    spec = importlib.util.spec_from_file_location("_native_boundary_runner", path)
    if spec is None or spec.loader is None:
        raise ValueError("cannot load SQL execution contracts")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.COMMANDS


def validate_commands(commands: dict[str, list[str]]) -> None:
    for name, command in commands.items():
        if command[:2] != ["cargo", "test"]:
            continue
        selected = set(TOOLS).intersection(command)
        if not selected:
            continue
        features = set()
        for index, value in enumerate(command[:-1]):
            if value == "--features":
                features.update(command[index + 1].split(","))
        compiler_features = {f"{tool}/compiler" for tool in selected}
        if name == "sql-native-tool-tests":
            if features.intersection(compiler_features) or "--all-features" in command:
                raise ValueError("native contract must exclude embedded compiler features")
        elif not compiler_features <= features:
            raise ValueError(f"{name}: existing embedded compiler coverage is missing")


def validate_protocol_tree(text: str) -> None:
    names = {line.split()[0] for line in text.splitlines() if line.strip()}
    if "sifr_sql_tool" not in names:
        raise ValueError("protocol test graph omitted sifr_sql_tool")
    forbidden = {name for name in names if name in {
        "sifr", "sifr_semantic_services", "sifr_frontend", "sifr_lowering", "sifr_codegen",
    } or name.startswith(("wasmtime", "cranelift", "pulley"))}
    if forbidden:
        raise ValueError(f"protocol test graph contains compiler capabilities: {sorted(forbidden)}")


def validate_protocol_ownership(
    protocol_manifest: dict, commands: dict[str, list[str]], area_manifest: dict,
) -> None:
    if "sifr_semantic_services" in protocol_manifest.get("dev-dependencies", {}):
        raise ValueError("compiler-dependent migration tests belong to the compiler owner")
    expected = ["cargo", "test", "--locked", "-p", "sifr", "--test", "sql_migration_source"]
    if commands.get("sql-migration-source-tests") != expected:
        raise ValueError("canonical compiler migration test execution is missing")
    selected = [
        case for suite in area_manifest["suites"] if suite["name"] == "schema-tools"
        for case in suite["cases"] if case["command"] == "sql-migration-source-tests"
    ]
    if len(selected) != 1 or selected[0]["entry"] != "crates/sifr/tests/sql_migration_source.rs":
        raise ValueError("compiler migration test must remain enrolled in schema-tools")


def protocol_self_test(protocol_manifest: dict, commands: dict, area_manifest: dict) -> int:
    mutations = []
    mutated = copy.deepcopy(protocol_manifest)
    mutated.setdefault("dev-dependencies", {})["sifr_semantic_services"] = {"workspace": True}
    mutations.append((mutated, commands, area_manifest))
    mutated = copy.deepcopy(commands)
    mutated.pop("sql-migration-source-tests")
    mutations.append((protocol_manifest, mutated, area_manifest))
    for removal in (True, False):
        mutated = copy.deepcopy(area_manifest)
        suite = next(suite for suite in mutated["suites"] if suite["name"] == "schema-tools")
        case = next(case for case in suite["cases"] if case["command"] == "sql-migration-source-tests")
        if removal:
            suite["cases"].remove(case)
        else:
            case["entry"] = "crates/sifr_sql_tool/tests/migration_source.rs"
        mutations.append((protocol_manifest, commands, mutated))
    for values in mutations:
        try:
            validate_protocol_ownership(*values)
        except ValueError:
            continue
        raise ValueError("accepted protocol ownership mutation")
    baseline = "sifr_sql_tool v0.0.0"
    validate_protocol_tree(baseline)
    trees = ["missing-tool v0.0.0"]
    trees.extend(baseline + "\n" + name + " v0.0.0" for name in (
        "sifr", "sifr_semantic_services", "sifr_frontend", "sifr_lowering", "sifr_codegen",
        "wasmtime", "wasmtime-internal-component-macro", "cranelift-codegen", "pulley-interpreter",
    ))
    for text in trees:
        try:
            validate_protocol_tree(text)
        except ValueError:
            continue
        raise ValueError("accepted protocol dependency-tree mutation")
    return len(mutations) + len(trees)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    manifests = {
        name: tomllib.loads((ROOT / "crates" / name / "Cargo.toml").read_text())
        for name in TOOLS
    }
    validate(manifests)
    mutations = self_test(manifests) if args.self_test else 0
    commands = execution_commands()
    validate_commands(commands)
    protocol_manifest = tomllib.loads((ROOT / "crates/sifr_sql_tool/Cargo.toml").read_text())
    area_manifest = json.loads((ROOT / "verification/areas/sql_platform/manifest.json").read_text())
    validate_protocol_ownership(protocol_manifest, commands, area_manifest)
    if args.self_test:
        mutations += protocol_self_test(protocol_manifest, commands, area_manifest)
    if args.self_test:
        for name, command in commands.items():
            if command[:2] != ["cargo", "test"] or not set(TOOLS).intersection(command):
                continue
            mutated = copy.deepcopy(commands)
            if name == "sql-native-tool-tests":
                mutated[name].extend(("--features", f"{TOOLS[0]}/compiler"))
            else:
                index = mutated[name].index("--features")
                del mutated[name][index:index + 2]
            try:
                validate_commands(mutated)
            except ValueError:
                mutations += 1
            else:
                raise ValueError(f"accepted test-coverage mutation: {name}")
    command = ["cargo", "tree", "--locked", "--offline",
               "--edges", "normal,build", "--prefix", "none", "--format", "{p}"]
    for name in TOOLS:
        command.extend(("-p", name))
    result = subprocess.run(command, cwd=ROOT, check=True, text=True, capture_output=True)
    validate_tree(result.stdout)
    protocol_command = [
        "cargo", "tree", "--locked", "--offline", "-p", "sifr_sql_tool",
        "--features", "__compiler_fixture", "--edges", "normal,build,dev",
        "--prefix", "none", "--format", "{p}",
    ]
    protocol_result = subprocess.run(protocol_command, cwd=ROOT, check=True, text=True, capture_output=True)
    validate_protocol_tree(protocol_result.stdout)
    print(f"SQL native tool boundary ok: tools={len(TOOLS)} mutations={mutations}")


if __name__ == "__main__":
    main()
