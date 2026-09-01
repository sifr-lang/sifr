#!/usr/bin/env python3
"""Validate the compiler component qualification contract."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import tomllib
from pathlib import Path

from wasi_virt_inputs import (
    WASI_VIRT_COMMIT,
    WASI_VIRT_SOURCE_SHA256,
    WASI_VIRT_VERSION,
)

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
EXPECTED_DETERMINISM_CONTROLS = {
    "nan-canonicalization",
    "relaxed-simd-disabled",
}


class QualificationError(ValueError):
    """The compiler component qualification record is invalid."""


def validate(payload: object, *, host_source_override: str | None = None) -> None:
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
    if engine.get("features") != [
        "component-model",
        "cranelift",
        "gc-null",
        "runtime",
        "std",
    ]:
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
    host_source = host_source_override or (
        REPO_ROOT / "crates/sifr_compiler_component/src/host.rs"
    ).read_text(encoding="utf-8")
    required_host_controls = {
        "config.consume_fuel(true)",
        "config.cranelift_nan_canonicalization(true)",
        "config.wasm_exceptions(true)",
        "config.wasm_relaxed_simd(false)",
        "config.wasm_memory64(false)",
        "config.wasm_multi_memory(true)",
        "StoreLimitsBuilder::new()",
        ".instances(self.limits.max_instances)",
        ".memories(self.limits.max_memories)",
        ".tables(self.limits.max_tables)",
        "Linker::<HostState>::new(&self.engine)",
    }
    if any(control not in host_source for control in required_host_controls):
        raise QualificationError("component host sandbox controls are incomplete")
    if set(payload.get("determinism_controls", [])) != EXPECTED_DETERMINISM_CONTROLS:
        raise QualificationError("component determinism controls are incomplete")
    if "wasmtime_wasi" in host_source:
        raise QualificationError("component host must not link WASI")
    if set(payload.get("denied_capabilities", [])) != EXPECTED_DENIALS:
        raise QualificationError("component capability denial list is incomplete")
    limits = payload.get("limits")
    if not isinstance(limits, list) or len(limits) != len(set(limits)) or len(limits) < 8:
        raise QualificationError("component resource-limit list is incomplete")
    participants = payload.get("sql_participants")
    if not isinstance(participants, list) or {
        row.get("crate") for row in participants if isinstance(row, dict)
    } != {"sifr_sql_mysql", "sifr_sql_postgresql", "sifr_sql_sqlite"}:
        raise QualificationError("SQL component participant set is incomplete")
    for participant in participants:
        artifacts = participant.get("artifacts")
        if not isinstance(artifacts, list) or not artifacts:
            raise QualificationError("SQL component artifact set is empty")
        if any(not (REPO_ROOT / str(path)).is_file() for path in artifacts):
            raise QualificationError("SQL component artifact is absent")
        manifest_path = REPO_ROOT / str(participant.get("manifest", ""))
        if not manifest_path.is_file():
            raise QualificationError("SQL component artifact manifest is absent")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        if (
            manifest.get("target") != "wasm32-wasip2"
            or manifest.get("wit_world") != "embedded-language-provider"
            or manifest.get("protocol_major") != 1
        ):
            raise QualificationError("SQL component artifact manifest boundary has drifted")
        virtualization = manifest.get("wasi_virtualization")
        if virtualization is None:
            toolchain = manifest.get("toolchain", {})
            virtualization = {
                "name": "wasi-virt",
                "version": toolchain.get("wasi_virt"),
                "commit": toolchain.get("wasi_virt_commit"),
                "source_content_sha256": toolchain.get("wasi_virt_source_sha256"),
            }
        if virtualization != {
            "name": "wasi-virt",
            "version": WASI_VIRT_VERSION,
            "commit": WASI_VIRT_COMMIT,
            "source_content_sha256": WASI_VIRT_SOURCE_SHA256,
        }:
            raise QualificationError("SQL component WASI-Virt identity has drifted")
        manifest_artifacts = manifest.get("artifacts")
        if not isinstance(manifest_artifacts, list) or not manifest_artifacts:
            raise QualificationError("SQL component artifact manifest has no artifacts")
        recorded_paths = set()
        for row in manifest_artifacts:
            if not isinstance(row, dict):
                raise QualificationError("SQL component artifact record is invalid")
            artifact_path = manifest_path.parent / str(row.get("path", ""))
            if not artifact_path.is_file():
                raise QualificationError("recorded SQL component artifact is absent")
            payload_bytes = artifact_path.read_bytes()
            if row.get("sha256") != hashlib.sha256(payload_bytes).hexdigest() or row.get(
                "size_bytes"
            ) != len(payload_bytes):
                raise QualificationError("SQL component artifact digest or size has drifted")
            recorded_paths.add(artifact_path.resolve())
        qualified_paths = {(REPO_ROOT / str(path)).resolve() for path in artifacts}
        if recorded_paths != qualified_paths:
            raise QualificationError("SQL component artifact manifest coverage has drifted")
        evidence = str(participant.get("evidence", ""))
        parts = evidence.split("::", maxsplit=1)
        if len(parts) != 2 or not (REPO_ROOT / parts[0]).is_file():
            raise QualificationError("SQL component evidence is invalid")
        source = (REPO_ROOT / parts[0]).read_text(encoding="utf-8")
        if f"fn {parts[1]}()" not in source:
            raise QualificationError("SQL component evidence does not resolve")
    fixture = payload.get("fixture")
    if not isinstance(fixture, dict) or fixture.get("sql") is not False:
        raise QualificationError("qualification requires a non-SQL fixture")
    if fixture.get("processor") != "fixture.words":
        raise QualificationError("non-SQL fixture processor identity is invalid")
    source_path = REPO_ROOT / str(fixture.get("source", ""))
    artifact_path = REPO_ROOT / str(fixture.get("artifact", ""))
    if not source_path.is_file() or not artifact_path.is_file():
        raise QualificationError("non-SQL fixture source or component artifact is absent")
    artifact_digest = hashlib.sha256(artifact_path.read_bytes()).hexdigest()
    if fixture.get("artifact_sha256") != artifact_digest:
        raise QualificationError("non-SQL fixture component artifact digest drifted")
    if fixture.get("build_tooling") != {
        "rust_target": "wasm32-unknown-unknown",
        "wit_bindgen": "0.57.1",
        "wit_component": "0.254.0",
    }:
        raise QualificationError("non-SQL fixture build tooling is not exact")
    fixture_manifest = tomllib.loads(
        source_path.parent.parent.joinpath("Cargo.toml").read_text(encoding="utf-8")
    )
    fixture_dependencies = fixture_manifest.get("dependencies", {})
    if fixture_dependencies.get("wit-bindgen") != "=0.57.1":
        raise QualificationError("non-SQL fixture wit-bindgen dependency drifted")
    wit_component = fixture_dependencies.get("wit-component")
    if not isinstance(wit_component, dict) or wit_component.get("version") != "=0.254.0":
        raise QualificationError("non-SQL fixture wit-component dependency drifted")
    fixture_source = source_path.read_text(encoding="utf-8")
    required_fixture_mechanisms = {
        "serde_json::from_slice(request)",
        "ty: hole.ty.clone()",
        "Sha256::digest(text.as_bytes())",
        "source_map: vec![SourceMapEntry",
        "plan.stable_fingerprint = hex_digest",
    }
    if any(mechanism not in fixture_source for mechanism in required_fixture_mechanisms):
        raise QualificationError("non-SQL fixture does not derive a complete plan in the guest")
    contracts = fixture.get("contracts")
    if not isinstance(contracts, dict) or set(contracts) != EXPECTED_CONTRACTS:
        raise QualificationError("non-SQL fixture contract coverage is incomplete")
    tests = (REPO_ROOT / "crates/sifr_compiler_component/src/tests.rs").read_text(
        encoding="utf-8"
    )
    if str(fixture.get("artifact", "")).rsplit("/", maxsplit=1)[-1] not in tests:
        raise QualificationError("non-SQL fixture test does not execute the qualified artifact")
    for contract, evidence_path in contracts.items():
        if not isinstance(evidence_path, str):
            raise QualificationError(f"non-SQL fixture evidence for {contract} is invalid")
        evidence = evidence_path.rsplit("::", maxsplit=1)[-1]
        if not evidence or f"fn {evidence}()" not in tests:
            raise QualificationError(f"non-SQL fixture evidence for {contract} does not resolve")


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
    missing_determinism = copy.deepcopy(payload)
    missing_determinism["determinism_controls"] = missing_determinism[
        "determinism_controls"
    ][:-1]
    mutations.append(("determinism", missing_determinism))
    missing_contract = copy.deepcopy(payload)
    del missing_contract["fixture"]["contracts"]["typed-holes"]  # type: ignore[index]
    mutations.append(("contract", missing_contract))
    sql_fixture = copy.deepcopy(payload)
    sql_fixture["fixture"]["sql"] = True  # type: ignore[index]
    mutations.append(("fixture", sql_fixture))
    missing_sql_participant = copy.deepcopy(payload)
    missing_sql_participant["sql_participants"] = missing_sql_participant[
        "sql_participants"
    ][:-1]
    mutations.append(("sql-participant", missing_sql_participant))
    missing_manifest = copy.deepcopy(payload)
    del missing_manifest["sql_participants"][0]["manifest"]  # type: ignore[index]
    mutations.append(("component-manifest", missing_manifest))
    for name, mutation in mutations:
        try:
            validate(mutation)
        except QualificationError:
            continue
        raise QualificationError(f"mutation did not fail: {name}")
    host_source = (
        REPO_ROOT / "crates/sifr_compiler_component/src/host.rs"
    ).read_text(encoding="utf-8")
    relaxed_simd = host_source.replace(
        "config.wasm_relaxed_simd(false);",
        "config.wasm_relaxed_simd(true);",
    )
    try:
        validate(payload, host_source_override=relaxed_simd)
    except QualificationError:
        pass
    else:
        raise QualificationError("mutation did not fail: relaxed-simd")
    mutations.append(("relaxed-simd", payload))
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
