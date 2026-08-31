#!/usr/bin/env python3
"""Validate the Cargo-locked host-tool graph and direct command contract."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
CONTRACT = REPO_ROOT / "verification/areas/sql_platform/data/host_tool_qualification.json"


class ContractError(ValueError):
    """The host-tool qualification is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def validate(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "schema_version must be 1")
    require(payload.get("graph_owner") == "sifr_package", "host-tool graph owner drift")
    require(payload.get("dispatch_owner") == "sifr", "host-tool dispatch owner drift")
    require(set(payload.get("lock_authority", [])) == {"Cargo.lock", "sifr-tools.lock.json"},
            "host-tool lock authority drift")
    require(payload.get("metadata_authority") == "cargo metadata --frozen",
            "host-tool metadata authority drift")
    execution = payload.get("execution", {})
    require(execution == {
        "build_target": "host",
        "target_argument_allowed": True,
        "target_source": "sifr-build-target",
        "direct_namespace": True,
        "application_graph_separate": True,
        "build_then_direct_exec": True,
        "cargo_program_absolute": True,
        "native_sandbox_required": True,
        "output_limit_bytes": 10 * 1024 * 1024,
    }, "host-tool execution contract drift")
    require(set(payload.get("entrypoint_identity", [])) == {
        "cargo-package-id", "package-name", "package-version", "package-source",
        "package-checksum", "binary-entrypoint", "capabilities", "tools-manifest-fingerprint",
        "lockfile-fingerprint", "persisted-tool-graph", "executable-sha256",
    }, "host-tool identity is incomplete")
    require(set(payload.get("capabilities", [])) == {
        "credentials", "environment", "network", "project-read", "project-write", "subprocess",
    }, "host-tool capability vocabulary drift")
    require(set(payload.get("negative_contracts", [])) == {
        "unknown-namespace", "reserved-namespace", "duplicate-namespace", "unknown-capability",
        "missing-entrypoint", "non-direct-tool-package", "lockfile-hash-drift",
        "target-graph-contamination", "missing-persisted-lock", "tools-manifest-drift",
        "path-source-drift", "capability-denial",
    }, "host-tool negative inventory is incomplete")
    provision = payload.get("sql_test_provision", {})
    require(provision.get("manifest_version") == 1, "provision manifest version drift")
    require(provision.get("inline_credentials") is False,
            "provision manifest must reject inline credentials")
    require(provision.get("credential_helper_uses_shell") is False,
            "credential helpers must not use a shell command string")
    isolation = payload.get("inherited_failure_isolation", {})
    require(isolation == {
        "sql_initialization_is_fatal": False,
        "diagnostic_is_required": True,
        "non_sql_analysis_is_preserved": True,
    }, "SQL initialization failure isolation drift")
    validate_evidence_paths(payload)


def validate_evidence_paths(payload: dict[str, Any]) -> None:
    evidence = set(payload.get("behavioral_evidence", []))
    require(evidence == {
        "crates/sifr/tests/host_tool_cli.rs",
        "crates/sifr_analysis/src/host/tests/sql_editor_tests.rs",
        "crates/sifr_package/src/host_tools_tests.rs",
        "crates/sifr_sql_contract/tests/provision_contract.rs",
        "crates/sifr_sql_mysql_tools/src/command.rs",
        "crates/sifr_sql_mysql_tools/src/provision.rs",
    }, "host-tool behavioral evidence inventory drift")
    for relative in evidence:
        path = REPO_ROOT / relative
        require(path.is_file(), f"behavioral evidence file does not exist: {relative}")


def self_test(payload: dict[str, Any]) -> None:
    mutations: list[dict[str, Any]] = []
    candidate = copy.deepcopy(payload)
    candidate["entrypoint_identity"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["capabilities"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["negative_contracts"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["execution"]["target_argument_allowed"] = False
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["inherited_failure_isolation"]["sql_initialization_is_fatal"] = True
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["behavioral_evidence"].pop()
    mutations.append(candidate)
    accepted = 0
    for candidate in mutations:
        try:
            validate(candidate)
        except ContractError:
            continue
        accepted += 1
    require(accepted == 0, f"host-tool mutations accepted: {accepted}")
    print(f"SQL host-tool self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(CONTRACT.read_text(encoding="utf-8"))
    validate(payload)
    if args.self_test:
        self_test(payload)
    else:
        print("SQL host-tool qualification ok: identities=11 negatives=12 capabilities=6")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, ContractError) as error:
        print(f"SQL host-tool qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
