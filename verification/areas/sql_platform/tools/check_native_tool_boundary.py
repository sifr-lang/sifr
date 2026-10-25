"""Qualify native SQL libraries independently of their default compiler adapter."""
from __future__ import annotations

import argparse
import copy
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
TOOLS = tuple(f"sifr_sql_{family}_tools" for family in ("mysql", "postgresql", "sqlite"))


def validate(manifests: dict[str, dict]) -> None:
    for name in TOOLS:
        manifest = manifests[name]
        if manifest.get("features") != {
            "default": ["compiler"], "compiler": ["dep:sifr_semantic_services"],
        }:
            raise ValueError(f"{name}: explicit default compiler adapter drift")
        if manifest["dependencies"].get("sifr_semantic_services") != {
            "workspace": True, "optional": True,
        }:
            raise ValueError(f"{name}: compiler must be optional and workspace qualified")
        bins = manifest.get("bin", [])
        if len(bins) != 1 or bins[0].get("required-features") != ["compiler"]:
            raise ValueError(f"{name}: ordinary binary must require its compiler adapter")


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
            ("default", lambda m: m["features"].update(default=[])),
            ("feature-edge", lambda m: m["features"].update(compiler=[])),
            ("required-bin", lambda m: m["bin"][0].update({"required-features": []})),
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
    command = ["cargo", "tree", "--locked", "--offline", "--no-default-features",
               "--edges", "normal,build", "--prefix", "none", "--format", "{p}"]
    for name in TOOLS:
        command.extend(("-p", name))
    result = subprocess.run(command, cwd=ROOT, check=True, text=True, capture_output=True)
    validate_tree(result.stdout)
    print(f"SQL native tool boundary ok: tools={len(TOOLS)} mutations={mutations}")


if __name__ == "__main__":
    main()
