#!/usr/bin/env python3
"""Validate the closed MySQL provider qualification record."""

from __future__ import annotations

import argparse
import copy
import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[4]
RECORD = ROOT / "verification/areas/sql_platform/data/mysql_qualification.json"
ROOT_CARGO = ROOT / "Cargo.toml"
RUNTIME_CARGO = ROOT / "crates/sifr_sql_mysql_runtime/Cargo.toml"


def validate(data: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if data.get("schema_version") != 1 or data.get("milestone") != "sql_16_mysql_provider":
        errors.append("MySQL qualification identity is invalid")
    series = data.get("supported_series", [])
    if [item.get("series") for item in series] != ["8.4", "9.7", "26.7"]:
        errors.append("MySQL supported series must be exactly 8.4, 9.7, and 26.7")
    if any(not str(item.get("image", "")).startswith("mysql:") for item in series):
        errors.append("every MySQL series needs an official image")
    expected_tools = {
        "lalrpop": "0.23.1",
        "lalrpop-util": "0.23.1",
        "mysql_async": "0.37.0",
        "mysql_common": "0.37.3",
        "tokio": "1.53.1",
        "rustls": "0.23.43",
    }
    if data.get("toolchain") != expected_tools:
        errors.append("MySQL toolchain does not match the stable dependency baseline")
    root_cargo = ROOT_CARGO.read_text(encoding="utf-8")
    for crate, version in expected_tools.items():
        keys = {crate, crate.replace("-", "_")}
        if not any(
            f'{key} = {{ version = "={version}"' in root_cargo
            or f'{key} = {{ version = "{version}"' in root_cargo
            for key in keys
        ):
            errors.append(f"root Cargo manifest does not lock {crate} {version}")
    runtime = RUNTIME_CARGO.read_text(encoding="utf-8")
    for feature in data.get("runtime_features", []):
        if f'"{feature}"' not in runtime:
            errors.append(f"MySQL runtime feature '{feature}' is missing")
    if "default-features = false" not in root_cargo:
        errors.append("MySQL driver default features are not disabled")
    if '"tracing"' in runtime:
        errors.append("MySQL runtime enables forbidden tracing")
    surfaces = data.get("surfaces", [])
    expected_surfaces = {"grammar", "schema", "runtime", "tools", "migrations", "editor", "portable-requirements"}
    if {surface.get("id") for surface in surfaces} != expected_surfaces:
        errors.append("MySQL qualification surface inventory is incomplete")
    for surface in surfaces:
        if not surface.get("owner") or not surface.get("evidence"):
            errors.append(f"MySQL surface '{surface.get('id')}' has no owner or evidence")
        for evidence in surface.get("evidence", []):
            if not (ROOT / str(evidence)).is_file():
                errors.append(f"MySQL evidence does not exist: {evidence}")
    if len(set(data.get("required_contracts", []))) != 14:
        errors.append("MySQL required contract inventory is incomplete or duplicated")
    for key in ("live_matrix", "documentation"):
        path = ROOT / str(data.get(key, ""))
        if not path.is_file():
            errors.append(f"MySQL {key} path does not exist")
    evidence_path = ROOT / str(data.get("live_evidence", ""))
    if not evidence_path.is_file():
        errors.append("MySQL checked-in live evidence does not exist")
    else:
        evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
        rows = evidence.get("servers", [])
        if evidence.get("surface") != "all":
            errors.append("MySQL live evidence must cover every surface")
        if [row.get("series") for row in rows] != ["8.4", "9.7", "26.7"]:
            errors.append("MySQL live evidence does not cover the supported series")
        for row, expected in zip(rows, series, strict=False):
            if row.get("image") != expected.get("image") or row.get("status") != "passed":
                errors.append(f"MySQL {row.get('series')} live evidence is not passing")
            if re.fullmatch(r"sha256:[0-9a-f]{64}", str(row.get("image_digest"))) is None:
                errors.append(f"MySQL {row.get('series')} image digest is invalid")
    return errors


def self_test(data: dict[str, Any]) -> int:
    mutations = []
    wrong_series = copy.deepcopy(data)
    wrong_series["supported_series"] = wrong_series["supported_series"][:-1]
    mutations.append(wrong_series)
    wrong_version = copy.deepcopy(data)
    wrong_version["toolchain"]["mysql_async"] = "0.36.0"
    mutations.append(wrong_version)
    missing_evidence = copy.deepcopy(data)
    missing_evidence["surfaces"][0]["evidence"] = ["missing"]
    mutations.append(missing_evidence)
    missing_contract = copy.deepcopy(data)
    missing_contract["required_contracts"] = missing_contract["required_contracts"][:-1]
    mutations.append(missing_contract)
    if any(not validate(mutation) for mutation in mutations):
        raise SystemExit("MySQL checker accepted a required mutation")
    print(f"MySQL qualification mutations rejected: {len(mutations)}")
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
    print("MySQL provider qualification ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
