"""Keep protocol-only dependencies separate from native component execution."""

from __future__ import annotations

import copy
import tomllib
from pathlib import Path

HOST_OWNERS = ("sifr_analysis", "sifr_semantic_services")
PROVIDERS = ("sifr_sql_mysql", "sifr_sql_postgresql", "sifr_sql_sqlite")
PROTOCOL_OWNERS = (
    "sifr_compiler_services", "sifr_driver", "sifr_frontend", "sifr_package",
    "sifr_sql_contract", *PROVIDERS,
)


class FeatureBoundaryError(ValueError):
    """A component dependency enables the wrong capability."""


def load_manifests(root: Path) -> dict[str, dict]:
    paths = ["Cargo.toml", "crates/sifr_compiler_component/Cargo.toml"]
    paths.extend(f"crates/{name}/Cargo.toml" for name in (*HOST_OWNERS, *PROTOCOL_OWNERS))
    return {path: tomllib.loads((root / path).read_text()) for path in paths}


def validate_manifests(manifests: dict[str, dict]) -> None:
    workspace = manifests["Cargo.toml"]["workspace"]["dependencies"]["sifr_compiler_component"]
    if workspace.get("default-features") is not False or workspace.get("features", []):
        raise FeatureBoundaryError("workspace component dependencies must be protocol-only")
    component = manifests["crates/sifr_compiler_component/Cargo.toml"]
    if component.get("features") != {"default": ["host"], "host": ["dep:wasmtime"]}:
        raise FeatureBoundaryError("the native host capability and default must be exact")
    if "wasmtime" in component.get("dependencies", {}):
        raise FeatureBoundaryError("Wasmtime must not be an unconditional dependency")
    wasmtime = component["target"]['cfg(not(target_family = "wasm"))']["dependencies"]["wasmtime"]
    if wasmtime != {"workspace": True, "optional": True}:
        raise FeatureBoundaryError("native Wasmtime must be optional and workspace-qualified")
    for name in (*HOST_OWNERS, *PROTOCOL_OWNERS):
        manifest = manifests[f"crates/{name}/Cargo.toml"]
        dependency = manifest["dependencies"]["sifr_compiler_component"]
        expected_features = ["host"] if name in HOST_OWNERS else []
        if (
            dependency.get("workspace") is not True
            or dependency.get("default-features", False) is not False
            or dependency.get("features", []) != expected_features
        ):
            raise FeatureBoundaryError(f"{name} has the wrong native host grant")
    for name in PROVIDERS:
        dependency = manifests[f"crates/{name}/Cargo.toml"].get(
            "dev-dependencies", {}
        ).get("sifr_compiler_component")
        if dependency != {"workspace": True, "features": ["host"]}:
            raise FeatureBoundaryError(f"{name} integration tests must select the native host")


def validate_feature_boundary(root: Path) -> None:
    validate_manifests(load_manifests(root))


def self_test(root: Path) -> int:
    original = load_manifests(root)
    validate_manifests(original)
    mutations = []
    changed = copy.deepcopy(original)
    changed["Cargo.toml"]["workspace"]["dependencies"]["sifr_compiler_component"]["default-features"] = True
    mutations.append(("workspace-defaults", changed))
    changed = copy.deepcopy(original)
    changed["crates/sifr_compiler_component/Cargo.toml"]["features"]["host"] = []
    mutations.append(("missing-engine", changed))
    changed = copy.deepcopy(original)
    changed["crates/sifr_compiler_component/Cargo.toml"]["features"]["default"] = []
    mutations.append(("default-host-compatibility", changed))
    changed = copy.deepcopy(original)
    changed["crates/sifr_compiler_component/Cargo.toml"]["target"][
        'cfg(not(target_family = "wasm"))'
    ]["dependencies"]["wasmtime"]["optional"] = False
    mutations.append(("unconditional-engine", changed))
    for name in HOST_OWNERS:
        changed = copy.deepcopy(original)
        del changed[f"crates/{name}/Cargo.toml"]["dependencies"]["sifr_compiler_component"]["features"]
        mutations.append((f"missing-host-{name}", changed))
    for name in PROVIDERS:
        changed = copy.deepcopy(original)
        del changed[f"crates/{name}/Cargo.toml"]["dev-dependencies"]["sifr_compiler_component"]
        mutations.append((f"missing-test-host-{name}", changed))
    changed = copy.deepcopy(original)
    changed["crates/sifr_sql_contract/Cargo.toml"]["dependencies"]["sifr_compiler_component"]["features"] = ["host"]
    mutations.append(("protocol-host-leak", changed))
    for label, changed in mutations:
        try:
            validate_manifests(changed)
        except FeatureBoundaryError:
            continue
        raise FeatureBoundaryError(f"feature boundary accepted mutation: {label}")
    return len(mutations)
