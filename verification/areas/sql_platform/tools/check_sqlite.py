#!/usr/bin/env python3
"""Validate the closed SQLite provider qualification record."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[4]
RECORD = ROOT / "verification/areas/sql_platform/data/sqlite_qualification.json"


def validate(data: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if data.get("schema_version") != 1 or data.get("provider") != "sqlite":
        errors.append("SQLite qualification identity is invalid")
    libraries = data.get("supported_libraries", [])
    if libraries != [{"version": "3.53.2", "version_number": 3053002, "compile_flags": []}]:
        errors.append("SQLite supported library must be exactly the qualified 3.53.2 build")
    expected_tools = {
        "syntaqlite": "0.9.0",
        "rusqlite": "0.40.2",
        "libsqlite3-sys": "0.38.2",
        "sqlite": "3.53.2",
        "wasi-sdk": "33",
        "wit-bindgen": "0.61.1",
        "tokio": "1.53.1",
    }
    if data.get("toolchain") != expected_tools:
        errors.append("SQLite toolchain does not match the dependency baseline")
    root_cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    for crate, version in expected_tools.items():
        if crate in {"sqlite", "wasi-sdk"}:
            continue
        if not any(
            marker in root_cargo
            for marker in (
                f'{crate} = {{ version = "={version}"',
                f'{crate} = {{ version = "{version}"',
            )
        ):
            errors.append(f"root Cargo manifest does not lock {crate} {version}")
    cargo_config = (ROOT / ".cargo/config.toml").read_text(encoding="utf-8")
    if 'SYNTAQLITE_SQLITE_VERSION = { value = "3053002", force = true }' not in cargo_config:
        errors.append("Cargo does not pin Syntaqlite to SQLite 3.53.2")
    compiler = (ROOT / "crates/sifr_sql_sqlite/Cargo.toml").read_text(encoding="utf-8")
    runtime = (ROOT / "crates/sifr_sql_sqlite_runtime/Cargo.toml").read_text(encoding="utf-8")
    for feature in ["analysis", "fmt", "pin-cflags", "pin-version", "serde", "sqlite"]:
        if f'"{feature}"' not in compiler:
            errors.append(f"SQLite compiler feature '{feature}' is missing")
    for feature in data.get("runtime_features", []):
        if f'"{feature}"' not in runtime:
            errors.append(f"SQLite runtime feature '{feature}' is missing")
    surfaces = data.get("surfaces", [])
    expected_surfaces = {
        "grammar", "schema", "runtime", "tools", "migrations", "editor",
        "portable-requirements", "corruption-locking-performance",
    }
    if {surface.get("id") for surface in surfaces} != expected_surfaces:
        errors.append("SQLite qualification surface inventory is incomplete")
    for surface in surfaces:
        if not surface.get("owner") or not surface.get("evidence"):
            errors.append(f"SQLite surface '{surface.get('id')}' has no owner or evidence")
        for evidence in surface.get("evidence", []):
            if not (ROOT / str(evidence)).is_file():
                errors.append(f"SQLite evidence does not exist: {evidence}")
    if len(set(data.get("required_contracts", []))) != 16:
        errors.append("SQLite required contract inventory is incomplete or duplicated")
    for key in ("library_matrix", "documentation"):
        if not (ROOT / str(data.get(key, ""))).is_file():
            errors.append(f"SQLite {key} path does not exist")
    evidence_path = ROOT / str(data.get("library_evidence", ""))
    if not evidence_path.is_file():
        errors.append("SQLite checked-in library evidence does not exist")
    else:
        evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
        rows = evidence.get("libraries", [])
        if evidence.get("surface") != "all" or len(rows) != 1:
            errors.append("SQLite library evidence must cover every surface")
        elif rows[0].get("version_number") != 3053002 or rows[0].get("status") != "passed":
            errors.append("SQLite 3.53.2 library evidence is not passing")
        else:
            options = set(rows[0].get("runtime_compile_options", []))
            required_options = {
                "DEFAULT_FOREIGN_KEYS",
                "DEFAULT_RECURSIVE_TRIGGERS",
                "ENABLE_FTS5",
                "ENABLE_RTREE",
                "ENABLE_UNLOCK_NOTIFY",
                "MAX_VARIABLE_NUMBER=32766",
                "THREADSAFE=1",
            }
            if not required_options.issubset(options):
                errors.append("SQLite runtime compile-option evidence is incomplete")
    component_manifest_path = ROOT / str(data.get("component_manifest", ""))
    if not component_manifest_path.is_file():
        errors.append("SQLite component artifact manifest does not exist")
    else:
        component = json.loads(component_manifest_path.read_text(encoding="utf-8"))
        artifacts = component.get("artifacts", [])
        parser = component.get("parser", {})
        if component.get("target") != "wasm32-wasip2" or component.get("protocol_major") != 1:
            errors.append("SQLite component target or protocol is invalid")
        if parser != {
            "compile_flags": [],
            "name": "syntaqlite",
            "sqlite_version": "3.53.2",
            "sqlite_version_number": 3053002,
            "version": "0.9.0",
        }:
            errors.append("SQLite component parser identity is invalid")
        if len(artifacts) != 1:
            errors.append("SQLite component manifest must contain one artifact")
        else:
            artifact = artifacts[0]
            artifact_path = component_manifest_path.parent / str(artifact.get("path", ""))
            if not artifact_path.is_file():
                errors.append("SQLite component artifact does not exist")
            else:
                payload = artifact_path.read_bytes()
                if artifact.get("size_bytes") != len(payload):
                    errors.append("SQLite component artifact size is stale")
                if artifact.get("sha256") != hashlib.sha256(payload).hexdigest():
                    errors.append("SQLite component artifact digest is stale")
    return errors


def self_test(data: dict[str, Any]) -> int:
    mutations = []
    wrong_provider = copy.deepcopy(data)
    wrong_provider["provider"] = "postgresql"
    mutations.append(wrong_provider)
    wrong_version = copy.deepcopy(data)
    wrong_version["toolchain"]["rusqlite"] = "0.39.0"
    mutations.append(wrong_version)
    wrong_library = copy.deepcopy(data)
    wrong_library["supported_libraries"][0]["version_number"] = 3052000
    mutations.append(wrong_library)
    missing_evidence = copy.deepcopy(data)
    missing_evidence["surfaces"][0]["evidence"] = ["missing"]
    mutations.append(missing_evidence)
    missing_contract = copy.deepcopy(data)
    missing_contract["required_contracts"] = missing_contract["required_contracts"][:-1]
    mutations.append(missing_contract)
    if any(not validate(mutation) for mutation in mutations):
        raise SystemExit("SQLite checker accepted a required mutation")
    print(f"SQLite qualification mutations rejected: {len(mutations)}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    data = json.loads(RECORD.read_text(encoding="utf-8"))
    if args.self_test:
        return self_test(data)
    errors = validate(data)
    if errors:
        raise SystemExit("\n".join(errors))
    print("SQLite provider qualification ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
