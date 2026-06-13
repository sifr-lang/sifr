"""Profile loading, policy helpers, and shell exports."""

from __future__ import annotations

import shlex
import sys
from pathlib import Path
from typing import Any

from .errors import VerificationError
from .paths import PROFILES_DIR, REPO_ROOT
from .schemas import load_json, load_schema, validate_data


class ProfileError(VerificationError):
    """Profile data or profile lookup failed."""


def profile_path(profile: str, profiles_dir: Path = PROFILES_DIR) -> Path:
    return profiles_dir / f"{profile}.json"


def available_profiles(profiles_dir: Path = PROFILES_DIR) -> list[str]:
    return sorted(path.stem for path in profiles_dir.glob("*.json"))


def load_profile(profile: str, profiles_dir: Path = PROFILES_DIR) -> dict[str, Any]:
    path = profile_path(profile, profiles_dir)
    if not path.is_file():
        supported = ", ".join(available_profiles(profiles_dir))
        raise ProfileError(f"unsupported profile: {profile} (supported: {supported})")
    payload = load_json(path)
    if not isinstance(payload, dict):
        raise ProfileError(f"profile must be a JSON object: {path}")
    validate_data(payload, load_schema("profile.schema.json"), source=str(path.relative_to(REPO_ROOT)))
    if payload.get("name") != profile:
        raise ProfileError(f"profile name '{payload.get('name')}' must match file stem '{profile}'")
    return payload


def load_all_profiles(profiles_dir: Path = PROFILES_DIR) -> dict[str, dict[str, Any]]:
    profiles: dict[str, dict[str, Any]] = {}
    for name in available_profiles(profiles_dir):
        profiles[name] = load_profile(name, profiles_dir)
    return profiles


def selected_resource_classes(profile: dict[str, Any]) -> set[str]:
    classes: set[str] = set()
    resource_policy = profile.get("resource_policy", {})
    if isinstance(resource_policy, dict):
        raw_classes = resource_policy.get("classes", [])
        if isinstance(raw_classes, list):
            classes.update(item for item in raw_classes if isinstance(item, str))
    for selection in profile.get("selected_areas", []):
        if not isinstance(selection, dict):
            continue
        raw_classes = selection.get("resource_classes", [])
        if isinstance(raw_classes, list):
            classes.update(item for item in raw_classes if isinstance(item, str))
    return classes


def failure_reproduction_command(profile_name: str, case_id: str) -> str:
    return f"uv run --project verification python -m sifr_verify --profile {profile_name} --case {case_id}"


def shell_quote(value: Any) -> str:
    return shlex.quote("" if value is None else str(value))


def fixture_count(path: Path) -> int:
    payload = load_json(path)
    fixture_names = payload.get("fixture_names")
    if not isinstance(fixture_names, list) or not all(isinstance(name, str) for name in fixture_names):
        raise ProfileError(f"invalid fixture manifest: {path}")
    return len(fixture_names)


def resolve_fixture_manifest(raw_path: str) -> Path | None:
    if not raw_path:
        return None
    path = (REPO_ROOT / raw_path).resolve()
    if not path.is_file():
        raise ProfileError(f"fixture manifest not found: {path}")
    return path


def legacy_facade(profile: dict[str, Any]) -> dict[str, Any]:
    legacy = profile.get("legacy_facade")
    if not isinstance(legacy, dict):
        raise ProfileError(f"profile {profile.get('name')} is missing legacy_facade")
    return legacy


def shell_exports(profile: dict[str, Any]) -> dict[str, Any]:
    name = str(profile["name"])
    budgets = profile["budgets"]
    legacy = legacy_facade(profile)
    e2e = legacy["e2e"]
    matrix_suites = legacy["matrix_suites"]
    hardening_suites = legacy["hardening_suites"]
    tooling_suites = legacy["tooling_suites"]
    extra_checks = legacy["extra_checks"]
    fixture_manifest = resolve_fixture_manifest(str(e2e.get("fixture_manifest", "")))
    return {
        "RESOLVED_PROFILE": name,
        "LANE_NAME": name,
        "LANE_DESCRIPTION": profile.get("description", ""),
        "WARM_TARGET_MINUTES": budgets["warm_wall_time_minutes"],
        "COLD_TARGET_MINUTES": budgets["cold_wall_time_minutes"],
        "THERMAL_POLICY": legacy["thermal_policy"],
        "MEMORY_POLICY": legacy["memory_policy"],
        "CONTRACT_SUITES": ",".join(matrix_suites),
        "RUN_FRONTEND_MODE_PARITY": "1" if "frontend_mode_parity" in matrix_suites else "0",
        "RUN_PHASE23_GRAPH_ISOLATION": "1" if "phase23_graph_isolation" in matrix_suites else "0",
        "RUN_PHASE24_HIR_ANALYSIS": "1" if "phase24_hir_analysis" in matrix_suites else "0",
        "RUN_PHASE25_CFG_FLOW": "1" if "phase25_cfg_flow" in matrix_suites else "0",
        "TOOLING_SUITES": ",".join(tooling_suites),
        "DISTRIBUTION_MODE": legacy["distribution"],
        "GENERATED_CODE_QUALITY_MODE": legacy["generated_code_quality"],
        "PERFORMANCE_BUDGET_MODE": legacy["performance_budget"],
        "CRATE_TEST_MODE": legacy["crate_tests"],
        "RUN_HARDENING": "1" if hardening_suites else "0",
        "HARDENING_SUITES": ",".join(hardening_suites),
        "RUN_E2E_REPORT_DETERMINISM": "1" if "e2e_report_determinism" in extra_checks else "0",
        "RUN_E2E_SEQUENTIAL_PARALLEL_EQUIVALENCE": "1"
        if "e2e_sequential_parallel_equivalence" in extra_checks
        else "0",
        "E2E_PROFILE": name,
        "E2E_FIXTURE_MANIFEST": "" if fixture_manifest is None else str(fixture_manifest),
        "E2E_SIFR_JOBS": e2e["sifr_jobs"],
        "E2E_RUST_JOBS": e2e["rust_jobs"],
        "E2E_RUN_JOBS": e2e["run_jobs"],
        "E2E_CARGO_BUILD_JOBS": e2e["cargo_build_jobs"],
        "E2E_MAX_GROUP_FIXTURES": e2e["max_group_fixtures"],
        "E2E_DISABLE_CACHE": "1" if e2e["disable_cache"] else "0",
    }


def print_shell(profile_name: str) -> None:
    for key, value in shell_exports(load_profile(profile_name)).items():
        print(f"{key}={shell_quote(value)}")


def print_summary(requested_profile: str) -> None:
    profile = load_profile(requested_profile)
    legacy = legacy_facade(profile)
    e2e = legacy["e2e"]
    fixture_manifest = resolve_fixture_manifest(str(e2e.get("fixture_manifest", "")))
    fixture_count_display = "full-corpus"
    fixture_manifest_display = "none"
    if fixture_manifest is not None:
        fixture_count_display = str(fixture_count(fixture_manifest))
        fixture_manifest_display = str(fixture_manifest.relative_to(REPO_ROOT))

    print("Validation profile summary")
    print(f"  requested_profile={requested_profile}")
    print(f"  resolved_profile={profile['name']}")
    print(f"  description={profile.get('description', '')}")
    print(
        "  budgets="
        f"warm<={profile['budgets']['warm_wall_time_minutes']}m "
        f"cold<={profile['budgets']['cold_wall_time_minutes']}m"
    )
    print("  resource_classes=" + ", ".join(profile["resource_policy"]["classes"]))
    print("  matrix_suites=" + (", ".join(legacy["matrix_suites"]) if legacy["matrix_suites"] else "none"))
    print(f"  representative_e2e={fixture_count_display} fixtures (manifest={fixture_manifest_display})")
    print(
        "  hardening_suites="
        + (", ".join(legacy["hardening_suites"]) if legacy["hardening_suites"] else "none")
    )
    print("  tooling_suites=" + (", ".join(legacy["tooling_suites"]) if legacy["tooling_suites"] else "none"))
    print(f"  distribution={legacy['distribution']}")
    print(f"  generated_code_quality={legacy['generated_code_quality']}")
    print(f"  performance_budget={legacy['performance_budget']}")
    print(f"  crate_tests={legacy['crate_tests']}")
    print("  extra_checks=" + (", ".join(legacy["extra_checks"]) if legacy["extra_checks"] else "none"))
    print(f"  manifest={profile_path(str(profile['name'])).relative_to(REPO_ROOT)}")


def run_command(argv: list[str]) -> int:
    if not argv:
        print("profiles command requires one of: profile, shell, summary, check", file=sys.stderr)
        return 2
    command = argv[0]
    profile_name = _profile_arg(argv[1:]) if command in {"profile", "shell", "summary"} else ""
    if command == "profile":
        profile = load_profile(profile_name)
        print(profile["name"])
        return 0
    if command == "shell":
        print_shell(profile_name)
        return 0
    if command == "summary":
        print_summary(profile_name)
        return 0
    if command == "check":
        profiles = load_all_profiles()
        print(f"verification profiles valid: {', '.join(sorted(profiles))}")
        return 0
    print(f"unsupported profiles command: {command}", file=sys.stderr)
    return 2


def _profile_arg(argv: list[str]) -> str:
    for index, value in enumerate(argv):
        if value == "--profile" and index + 1 < len(argv):
            return argv[index + 1]
    raise ProfileError("--profile is required")
