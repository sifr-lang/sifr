#!/usr/bin/env python3
"""Resolve validation-lane metadata for local-first test execution."""

from __future__ import annotations

import argparse
import json
import shlex
import sys
from pathlib import Path
from typing import Any, NoReturn

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = REPO_ROOT / "verification" / "validation_lanes" / "manifest.json"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        default=str(DEFAULT_MANIFEST),
        help="Path to the validation-lane manifest JSON.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    for name in ("profile", "shell", "summary"):
        subparser = subparsers.add_parser(name)
        subparser.add_argument("--profile", required=True, help="Requested validation profile.")

    return parser.parse_args()


def load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit(f"invalid manifest: expected JSON object at {path}")
    return payload


def load_lane_manifest(path: Path) -> dict[str, dict[str, Any]]:
    payload = load_json(path)
    lanes = payload.get("lanes", [])
    if not isinstance(lanes, list):
        raise SystemExit("invalid lane manifest: 'lanes' must be a list")

    lane_map: dict[str, dict[str, Any]] = {}
    for lane in lanes:
        if not isinstance(lane, dict):
            raise SystemExit("invalid lane manifest: lane entries must be objects")
        name = lane.get("name")
        if not isinstance(name, str) or not name:
            raise SystemExit("invalid lane manifest: each lane needs a string 'name'")
        lane_map[name] = lane
    return lane_map


def _raise_with_code(message: str, code: int) -> NoReturn:
    print(message, file=sys.stderr)
    raise SystemExit(code)


def resolve_profile(profile: str, lanes: dict[str, dict[str, Any]]) -> str:
    if profile not in lanes:
        supported = ", ".join(sorted(lanes))
        _raise_with_code(f"unsupported profile: {profile} (supported: {supported})", 2)
    return profile


def load_fixture_count(path: Path) -> int:
    payload = load_json(path)
    fixture_names = payload.get("fixture_names")
    if not isinstance(fixture_names, list) or not all(isinstance(name, str) for name in fixture_names):
        raise SystemExit(f"invalid fixture manifest: {path}")
    return len(fixture_names)


def resolve_fixture_manifest_path(raw_path: str) -> Path:
    fixture_manifest_path = (REPO_ROOT / raw_path).resolve()
    if not fixture_manifest_path.is_file():
        raise SystemExit(f"fixture manifest not found: {fixture_manifest_path}")
    return fixture_manifest_path


def shell_quote(value: Any) -> str:
    return shlex.quote("" if value is None else str(value))


def emit_shell(profile: str, lane: dict[str, Any], manifest_path: Path) -> None:
    lane_dir = manifest_path.parent
    matrix_suites = lane.get("matrix_suites", [])
    if not isinstance(matrix_suites, list):
        raise SystemExit("invalid lane manifest: 'matrix_suites' must be a list")
    hardening_suites = lane.get("hardening_suites", [])
    if not isinstance(hardening_suites, list):
        raise SystemExit("invalid lane manifest: 'hardening_suites' must be a list")
    tooling_suites = lane.get("tooling_suites", [])
    if not isinstance(tooling_suites, list):
        raise SystemExit("invalid lane manifest: 'tooling_suites' must be a list")
    extra_checks = lane.get("extra_checks", [])
    if not isinstance(extra_checks, list):
        raise SystemExit("invalid lane manifest: 'extra_checks' must be a list")

    e2e = lane.get("e2e", {})
    if not isinstance(e2e, dict):
        raise SystemExit("invalid lane manifest: 'e2e' must be an object")
    fixture_manifest = e2e.get("fixture_manifest")
    fixture_manifest_abs = ""
    if isinstance(fixture_manifest, str) and fixture_manifest:
        fixture_manifest_abs = str(resolve_fixture_manifest_path(fixture_manifest))

    values = {
        "RESOLVED_PROFILE": profile,
        "LANE_NAME": lane["name"],
        "LANE_DESCRIPTION": lane.get("description", ""),
        "WARM_TARGET_MINUTES": lane.get("warm_wall_time_target_minutes", ""),
        "COLD_TARGET_MINUTES": lane.get("cold_wall_time_target_minutes", ""),
        "THERMAL_POLICY": lane.get("thermal_policy", ""),
        "MEMORY_POLICY": lane.get("memory_policy", ""),
        "CONTRACT_SUITES": ",".join(matrix_suites),
        "RUN_FRONTEND_MODE_PARITY": "1" if "frontend_mode_parity" in matrix_suites else "0",
        "RUN_PHASE23_GRAPH_ISOLATION": "1" if "phase23_graph_isolation" in matrix_suites else "0",
        "RUN_PHASE24_HIR_ANALYSIS": "1" if "phase24_hir_analysis" in matrix_suites else "0",
        "RUN_PHASE25_CFG_FLOW": "1" if "phase25_cfg_flow" in matrix_suites else "0",
        "TOOLING_SUITES": ",".join(tooling_suites),
        "DISTRIBUTION_MODE": lane.get("distribution", "none"),
        "GENERATED_CODE_QUALITY_MODE": lane.get("generated_code_quality", "none"),
        "PERFORMANCE_BUDGET_MODE": lane.get("performance_budget", "none"),
        "CRATE_TEST_MODE": lane.get("crate_tests", "full"),
        "RUN_HARDENING": "1" if hardening_suites else "0",
        "HARDENING_SUITES": ",".join(hardening_suites),
        "RUN_E2E_REPORT_DETERMINISM": "1" if "e2e_report_determinism" in extra_checks else "0",
        "RUN_E2E_SEQUENTIAL_PARALLEL_EQUIVALENCE": "1"
        if "e2e_sequential_parallel_equivalence" in extra_checks
        else "0",
        "E2E_PROFILE": profile,
        "E2E_FIXTURE_MANIFEST": fixture_manifest_abs,
        "E2E_SIFR_JOBS": e2e.get("sifr_jobs", 1),
        "E2E_RUST_JOBS": e2e.get("rust_jobs", 1),
        "E2E_RUN_JOBS": e2e.get("run_jobs", 1),
        "E2E_CARGO_BUILD_JOBS": e2e.get("cargo_build_jobs", 1),
        "E2E_MAX_GROUP_FIXTURES": e2e.get("max_group_fixtures", ""),
        "E2E_DISABLE_CACHE": "1" if e2e.get("disable_cache", False) else "0",
        "LANE_MANIFEST_DIR": str(lane_dir),
    }
    for key, value in values.items():
        print(f"{key}={shell_quote(value)}")


def emit_summary(requested_profile: str, resolved_profile: str, lane: dict[str, Any], manifest_path: Path) -> None:
    e2e = lane.get("e2e", {})
    fixture_manifest = e2e.get("fixture_manifest")
    fixture_count = "full-corpus"
    fixture_manifest_display = "none"
    if isinstance(fixture_manifest, str) and fixture_manifest:
        fixture_manifest_path = resolve_fixture_manifest_path(fixture_manifest)
        fixture_count = str(load_fixture_count(fixture_manifest_path))
        fixture_manifest_display = str(fixture_manifest_path.relative_to(REPO_ROOT))

    matrix_suites = lane.get("matrix_suites", [])
    hardening_suites = lane.get("hardening_suites", [])
    tooling_suites = lane.get("tooling_suites", [])
    extra_checks = lane.get("extra_checks", [])

    print("Validation lane summary")
    print(f"  requested_profile={requested_profile}")
    print(f"  resolved_profile={resolved_profile}")
    print(f"  lane={lane['name']}")
    print(f"  description={lane.get('description', '')}")
    print(
        "  budgets="
        f"warm<={lane.get('warm_wall_time_target_minutes', '?')}m "
        f"cold<={lane.get('cold_wall_time_target_minutes', '?')}m"
    )
    print(
        "  policies="
        f"thermal={lane.get('thermal_policy', 'unknown')} "
        f"memory={lane.get('memory_policy', 'unknown')}"
    )
    print(
        "  matrix_suites="
        + (", ".join(matrix_suites) if matrix_suites else "none")
    )
    print(
        "  representative_e2e="
        f"{fixture_count} fixtures "
        f"(manifest={fixture_manifest_display})"
    )
    print(
        "  hardening_suites="
        + (", ".join(hardening_suites) if hardening_suites else "none")
    )
    print("  tooling_suites=" + (", ".join(tooling_suites) if tooling_suites else "none"))
    print(f"  distribution={lane.get('distribution', 'none')}")
    print(f"  generated_code_quality={lane.get('generated_code_quality', 'none')}")
    print(f"  performance_budget={lane.get('performance_budget', 'none')}")
    print(f"  crate_tests={lane.get('crate_tests', 'full')}")
    print("  extra_checks=" + (", ".join(extra_checks) if extra_checks else "none"))
    print(f"  manifest={manifest_path.relative_to(REPO_ROOT)}")


def main() -> None:
    args = parse_args()
    manifest_path = Path(args.manifest).resolve()
    lanes = load_lane_manifest(manifest_path)
    requested_profile = args.profile
    resolved_profile = resolve_profile(requested_profile, lanes)
    lane = lanes[resolved_profile]

    if args.command == "profile":
        print(resolved_profile)
        return
    if args.command == "shell":
        emit_shell(resolved_profile, lane, manifest_path)
        return
    if args.command == "summary":
        emit_summary(requested_profile, resolved_profile, lane, manifest_path)
        return
    raise SystemExit(f"unsupported command: {args.command}")


if __name__ == "__main__":
    main()
