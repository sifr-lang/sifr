#!/usr/bin/env python3
"""Validate the compiler component qualification contract."""

from __future__ import annotations

import argparse
import copy
import json
import tomllib
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[4]
RECORD_PATH = REPO_ROOT / "verification/areas/sql_platform/data/compiler_component_qualification.json"
EXPECTED_TARGETS = {
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
}
EXPECTED_DENIALS = {
    "clock",
    "environment",
    "filesystem",
    "linker",
    "native-library",
    "network",
    "process",
    "random",
    "rust-source",
    "shared-memory",
    "thread",
    "wasi",
}
EXPECTED_CONTRACTS = {
    "cache",
    "dependencies",
    "determinism",
    "diagnostics",
    "malformed-output",
    "parsing",
    "source-maps",
    "typed-holes",
}


class QualificationError(ValueError):
    """The compiler component qualification record is invalid."""


def validate(payload: object) -> None:
    if not isinstance(payload, dict) or payload.get("schema_version") != 1:
        raise QualificationError("qualification schema_version must be 1")
    if payload.get("protocol_major") != 1:
        raise QualificationError("qualification protocol_major must be 1")
    engine = payload.get("engine")
    if not isinstance(engine, dict):
        raise QualificationError("qualification has no engine record")
    if engine.get("crate") != "wasmtime" or engine.get("version") != "48.0.1":
        raise QualificationError("qualification must pin Wasmtime 48.0.1")
    if engine.get("default_features") is not False:
        raise QualificationError("Wasmtime default features must be disabled")
    if engine.get("features") != ["component-model", "cranelift", "runtime", "std"]:
        raise QualificationError("Wasmtime feature selection is not exact")
    cargo = tomllib.loads((REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    wasmtime = cargo["workspace"]["dependencies"].get("wasmtime")
    if not isinstance(wasmtime, dict):
        raise QualificationError("workspace Wasmtime dependency is absent")
    if wasmtime.get("version") != "=48.0.1" or wasmtime.get("default-features") is not False:
        raise QualificationError("workspace Wasmtime dependency is not exact")
    if wasmtime.get("features") != engine.get("features"):
        raise QualificationError("workspace and qualification Wasmtime features differ")
    wit = payload.get("wit")
    if not isinstance(wit, str) or not (REPO_ROOT / wit).is_file():
        raise QualificationError("compiler-owned WIT file is absent")
    wit_text = (REPO_ROOT / wit).read_text(encoding="utf-8")
    if "export analyze:" not in wit_text or "import " in wit_text or "wasi:" in wit_text:
        raise QualificationError("compiler-owned WIT boundary is not closed")
    if set(payload.get("host_targets", [])) != EXPECTED_TARGETS:
        raise QualificationError("component host target matrix is incomplete")
    component_source = (
        REPO_ROOT / "crates/sifr_compiler_component/src/lib.rs"
    ).read_text(encoding="utf-8")
    if any(f'"{target}"' not in component_source for target in EXPECTED_TARGETS):
        raise QualificationError("component host code and qualification targets differ")
    workflow = (REPO_ROOT / ".github/workflows/local-first-validation.yml").read_text(
        encoding="utf-8"
    )
    if "compiler-component-targets:" not in workflow or any(
        target not in workflow for target in EXPECTED_TARGETS
    ):
        raise QualificationError("native component qualification matrix is incomplete")
    host_source = (
        REPO_ROOT / "crates/sifr_compiler_component/src/host.rs"
    ).read_text(encoding="utf-8")
    required_host_controls = {
        "config.consume_fuel(true)",
        "config.wasm_memory64(false)",
        "config.wasm_multi_memory(false)",
        "StoreLimitsBuilder::new()",
        "Linker::<HostState>::new(&self.engine)",
    }
    if any(control not in host_source for control in required_host_controls):
        raise QualificationError("component host sandbox controls are incomplete")
    if "wasmtime_wasi" in host_source:
        raise QualificationError("component host must not link WASI")
    if set(payload.get("denied_capabilities", [])) != EXPECTED_DENIALS:
        raise QualificationError("component capability denial list is incomplete")
    limits = payload.get("limits")
    if not isinstance(limits, list) or len(limits) != len(set(limits)) or len(limits) < 8:
        raise QualificationError("component resource-limit list is incomplete")
    fixture = payload.get("fixture")
    if not isinstance(fixture, dict) or fixture.get("sql") is not False:
        raise QualificationError("qualification requires a non-SQL fixture")
    if fixture.get("processor") != "fixture.words":
        raise QualificationError("non-SQL fixture processor identity is invalid")
    if set(fixture.get("contracts", [])) != EXPECTED_CONTRACTS:
        raise QualificationError("non-SQL fixture contract coverage is incomplete")
    tests = (REPO_ROOT / "crates/sifr_compiler_component/src/tests.rs").read_text(
        encoding="utf-8"
    )
    evidence = fixture.get("evidence", "").rsplit("::", maxsplit=1)[-1]
    if not evidence or f"fn {evidence}()" not in tests:
        raise QualificationError("non-SQL fixture evidence does not resolve")


def run_self_test(payload: dict[str, object]) -> None:
    mutations: list[tuple[str, object]] = []
    wrong_version = copy.deepcopy(payload)
    wrong_version["protocol_major"] = 0
    mutations.append(("protocol", wrong_version))
    default_features = copy.deepcopy(payload)
    default_features["engine"]["default_features"] = True  # type: ignore[index]
    mutations.append(("default-features", default_features))
    missing_target = copy.deepcopy(payload)
    missing_target["host_targets"] = missing_target["host_targets"][:-1]  # type: ignore[index]
    mutations.append(("target", missing_target))
    missing_denial = copy.deepcopy(payload)
    missing_denial["denied_capabilities"] = missing_denial["denied_capabilities"][:-1]  # type: ignore[index]
    mutations.append(("capability", missing_denial))
    sql_fixture = copy.deepcopy(payload)
    sql_fixture["fixture"]["sql"] = True  # type: ignore[index]
    mutations.append(("fixture", sql_fixture))
    for name, mutation in mutations:
        try:
            validate(mutation)
        except QualificationError:
            continue
        raise QualificationError(f"mutation did not fail: {name}")
    print(f"compiler component qualification self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(RECORD_PATH.read_text(encoding="utf-8"))
    validate(payload)
    if args.self_test:
        run_self_test(payload)
    else:
        print(
            "compiler component qualification ok: "
            f"targets={len(payload['host_targets'])} "
            f"denials={len(payload['denied_capabilities'])}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
