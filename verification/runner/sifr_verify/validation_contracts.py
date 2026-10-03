"""Stage claims derived from canonical profiles and area manifests.

This inventory describes selection, not execution. A full label cannot create
coverage that the underlying manifest does not select.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from .errors import VerificationError
from .paths import REPO_ROOT
from .profiles import build_profile_plan, load_profile, resolve_fixture_manifest
from .schemas import load_json, load_schema, validate_data

POLICY = REPO_ROOT / "verification/policy/validation_contracts.json"


class ContractError(VerificationError):
    """A validation claim has no complete canonical contract."""


def load_policy(path: Path = POLICY) -> dict:
    policy = load_json(path)
    validate_data(policy, load_schema("validation_contracts.schema.json"), source=str(path))
    if policy["schema_version"] != 1:
        raise ContractError("unsupported validation contracts version")
    expected = {"create-pr", "merge", "nightly", "release", "artifact-qualification",
                "python-interop-live", "cloud"}
    stages = {row["stage"] for row in policy["stages"]}
    if stages != expected or len(stages) != len(policy["stages"]):
        raise ContractError("stage contracts must declare every stage exactly once")
    for row in policy["stages"]:
        for key in ("owner", "cadence", "boundary", "execution_policy"):
            if not row[key].strip():
                raise ContractError(f"{row['stage']}: missing {key}")
        if row["required_outcome"] != "passed":
            raise ContractError("required work must pass; skips cannot qualify")
        if row["stage"] in {"merge", "cloud"} and row["profile"] != "merge":
            raise ContractError("merge/cloud correctness must inherit the live merge inventory")
        if row["stage"] == "artifact-qualification":
            if row["profile"] or not row["entrypoints"]:
                raise ContractError("artifact qualification must execute exact-package consumers")
        elif not row["profile"] or row["entrypoints"]:
            raise ContractError("source stages must select one canonical profile")
        for entry in row["entrypoints"]:
            path = Path(entry)
            if path.is_absolute() or ".." in path.parts or not (REPO_ROOT / path).is_file():
                raise ContractError(f"invalid artifact consumer: {entry}")
    return policy


def selected_inventory(profile: dict, root: Path = REPO_ROOT) -> list[dict]:
    """Expand real selected case IDs without fabricating runtime observations."""
    rows = []
    for selection in profile["selected_areas"]:
        manifest = load_json(root / "verification/areas" / selection["area"] / "manifest.json")
        suites = {suite["name"]: suite for suite in manifest["suites"]}
        if len(suites) != len(manifest["suites"]):
            raise ContractError(f"duplicate suite in {manifest['name']}")
        for name in selection["suites"]:
            suite = suites[name]
            cases = [case["id"] for case in suite["cases"]]
            if not cases or len(cases) != len(set(cases)):
                raise ContractError(f"empty/duplicate case IDs in {manifest['name']}:{name}")
            rows.append({"id": f"{manifest['name']}:{name}", "owner": manifest["owner"],
                         "cases": cases, "inventory_case_count": len(cases),
                         "timeout_seconds": suite.get("timeout_seconds", manifest["timeout_seconds"]),
                         "resource_classes": suite.get("resource_classes", manifest["resource_classes"]),
                         "network_mode": suite.get("network_mode", manifest.get("network_mode", "offline")),
                         "skip_policy": manifest.get("skip_policy", {"allowed": False})})
    return rows


def stage_plan(stage: str) -> dict:
    policy = load_policy()
    matches = [row for row in policy["stages"] if row["stage"] == stage]
    if not matches:
        raise ContractError(f"unknown validation stage: {stage}")
    contract = matches[0]
    result = {"schema_version": 1, "stage": stage, "contract": contract,
              "supported_environments": load_json(REPO_ROOT / policy["supported_environments"]),
              "selection_state": "selected", "execution_state": "not-executed"}
    if not contract["profile"]:
        result["artifact_consumers"] = contract["entrypoints"]
        result["required_bindings"] = ["source", "submodules", "version", "target", "toolchain",
                                       "artifact_hashes", "producer", "workflow"]
        return result
    profile = load_profile(contract["profile"])
    result["profile_plan"] = build_profile_plan(contract["profile"])
    result["selected_inventory"] = selected_inventory(profile)
    path = resolve_fixture_manifest(profile["e2e"]["fixture_manifest"])
    full = sorted(p.stem for p in (REPO_ROOT / "crates/sifr/tests/e2e/pass").glob("*.sifr"))
    selected = load_json(path)["fixture_names"] if path else full
    if len(selected) != len(set(selected)) or not set(selected).issubset(full):
        raise ContractError("E2E selection contains duplicate or unknown fixture IDs")
    result["e2e_inventory"] = {"selected_ids": selected, "selected_count": len(selected),
                               "total_ids": full, "total_count": len(full)}
    # A suite can internally expand an adapter case into many observations.
    # These are manifest IDs, not a claim that the adapter's inner corpus ran.
    result["case_id_semantics"] = "declared adapter cases; runtime expansions require execution evidence"
    return result


def check_contracts() -> list[str]:
    policy = load_policy()
    for stage in policy["stages"]:
        stage_plan(stage["stage"])
    for key in ("supported_environments", "guarantees", "surfaces", "profile_assignments"):
        if not (REPO_ROOT / policy[key]).is_file():
            raise ContractError(f"missing canonical {key}: {policy[key]}")
    # Existing readiness owns guarantee/surface semantics; do not fork its rules.
    from .profile_commands import run_command
    import sys
    import os
    run_command([sys.executable, "verification/areas/coverage_matrix/checks/coverage_matrix.py"],
                env=dict(os.environ, SIFR_COVERAGE_MATRIX_STRICT="1"))
    run_command([sys.executable, "verification/areas/coverage_matrix/checks/profile_assignment_matrix.py"])
    return [row["stage"] for row in policy["stages"]]


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="sifr_verify contracts")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("check")
    plan = sub.add_parser("plan")
    plan.add_argument("--stage", required=True)
    args = parser.parse_args(argv)
    if args.command == "check":
        print("validation contracts valid: " + ", ".join(check_contracts()))
    else:
        print(json.dumps(stage_plan(args.stage), indent=2, sort_keys=True))
    return 0
