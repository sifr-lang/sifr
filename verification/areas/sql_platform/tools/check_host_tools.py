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
    require(payload.get("lock_authority") == "Cargo.lock", "host-tool lock authority drift")
    require(payload.get("metadata_authority") == "cargo metadata --frozen",
            "host-tool metadata authority drift")
    execution = payload.get("execution", {})
    require(execution == {
        "build_target": "host",
        "target_argument_allowed": True,
        "target_source": "sifr-build-target",
        "direct_namespace": True,
        "application_graph_separate": True,
    }, "host-tool execution contract drift")
    require(set(payload.get("entrypoint_identity", [])) == {
        "cargo-package-id", "package-name", "package-version", "package-source",
        "package-checksum", "binary-entrypoint", "capabilities", "tools-manifest-fingerprint",
        "lockfile-fingerprint",
    }, "host-tool identity is incomplete")
    require(set(payload.get("capabilities", [])) == {
        "credentials", "environment", "network", "project-read", "project-write", "subprocess",
    }, "host-tool capability vocabulary drift")
    require(set(payload.get("negative_contracts", [])) == {
        "unknown-namespace", "reserved-namespace", "duplicate-namespace", "unknown-capability",
        "missing-entrypoint", "non-direct-tool-package", "lockfile-hash-drift",
        "target-graph-contamination",
    }, "host-tool negative inventory is incomplete")
    provision = payload.get("sql_test_provision", {})
    require(provision.get("manifest_version") == 1, "provision manifest version drift")
    require(provision.get("inline_credentials") is False,
            "provision manifest must reject inline credentials")
    isolation = payload.get("inherited_failure_isolation", {})
    require(isolation == {
        "sql_initialization_is_fatal": False,
        "diagnostic_is_required": True,
        "non_sql_analysis_is_preserved": True,
    }, "SQL initialization failure isolation drift")
    validate_sources()


def validate_sources() -> None:
    graph = (REPO_ROOT / "crates/sifr_package/src/host_tools.rs").read_text(encoding="utf-8")
    metadata = (REPO_ROOT / "crates/sifr_package/src/cargo/metadata.rs").read_text(encoding="utf-8")
    derive = (REPO_ROOT / "crates/sifr_package/src/graph/derive.rs").read_text(encoding="utf-8")
    cli = (REPO_ROOT / "crates/sifr/src/host_tool_cli.rs").read_text(encoding="utf-8")
    command = (REPO_ROOT / "crates/sifr/src/cli_model_and_entrypoint.rs").read_text(encoding="utf-8")
    provision = (REPO_ROOT / "crates/sifr_sql_contract/src/provision.rs").read_text(encoding="utf-8")
    construction = (REPO_ROOT / "crates/sifr_analysis/src/host/construction.rs").read_text(encoding="utf-8")
    runtime = (REPO_ROOT / "crates/sifr_analysis/src/sql_editor_runtime.rs").read_text(encoding="utf-8")
    for token in (
        "HostToolGraph", "resolve_host_tool_graph", "verify_host_tool_graph",
        "lockfile_fingerprint", "tools_manifest_fingerprint", "package_checksum",
        "target_contamination_diagnostics",
        "RESERVED_TOOL_NAMESPACES", "HOST_TOOL_CAPABILITIES", "--locked",
    ):
        require(token in graph, f"host-tool graph is missing {token}")
    require("workspace_sifr" in metadata and "tools-package" in metadata,
            "Cargo workspace tool metadata is not normalized")
    require("workspace_sifr.tools_package" in derive,
            "tools member is not isolated from the application Sifr graph")
    for token in (
        "cmd_host_tool", "SIFR_TOOL_CAPABILITIES", "SIFR_TOOL_PACKAGE_CHECKSUM",
        "SIFR_TOOL_LOCKFILE_FINGERPRINT", "render_connection_manifest",
    ):
        require(token in cli, f"host-tool CLI is missing {token}")
    require("external_subcommand" in command and "HostTool" in command,
            "direct tool namespaces are not routed")
    require("TestConnectionManifest" in provision and "ProvisionedCredential" in provision,
            "structured test connection manifest is missing")
    require("from_initialization_failure" in construction,
            "SQL profile failure still aborts analysis host construction")
    require("initialization_diagnostics" in runtime,
            "SQL initialization diagnostics are not surfaced")


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
        print("SQL host-tool qualification ok: identities=9 negatives=8 capabilities=6")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, ContractError) as error:
        print(f"SQL host-tool qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
