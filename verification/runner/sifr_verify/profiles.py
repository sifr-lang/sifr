"""Profile loading, policy helpers, and shell exports."""

from __future__ import annotations

import json
import os
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

from .errors import VerificationError
from .paths import PROFILES_DIR, REPO_ROOT
from .schemas import load_json, load_schema, validate_data


class ProfileError(VerificationError):
    """Profile data or profile lookup failed."""


_WORKSPACE_PACKAGE_NAMES: set[str] | None = None


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
    validate_selected_area_suites(payload)
    validate_crate_test_membership(payload)
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


def validate_selected_area_suites(profile: dict[str, Any]) -> None:
    area_suites: dict[str, set[str]] = {}
    for manifest_path in sorted((REPO_ROOT / "verification" / "areas").glob("*/manifest.json")):
        manifest = load_json(manifest_path)
        if not isinstance(manifest, dict):
            continue
        name = manifest.get("name")
        suites = manifest.get("suites", [])
        if isinstance(name, str) and isinstance(suites, list):
            area_suites[name] = {
                str(suite.get("name"))
                for suite in suites
                if isinstance(suite, dict) and isinstance(suite.get("name"), str)
            }
    for selection in profile.get("selected_areas", []):
        area = selection.get("area") if isinstance(selection, dict) else None
        if not isinstance(area, str) or area not in area_suites:
            raise ProfileError(f"profile {profile.get('name')} selects unknown area: {area}")
        for suite in selection.get("suites", []):
            if suite not in area_suites[area]:
                raise ProfileError(
                    f"profile {profile.get('name')} selects unknown suite {area}:{suite}"
                )


def validate_crate_test_membership(profile: dict[str, Any]) -> None:
    membership = profile.get("crate_test_membership")
    if membership is None:
        return
    suites = membership.get("suites") if isinstance(membership, dict) else None
    if not isinstance(suites, list) or not suites:
        raise ProfileError(f"profile {profile.get('name')} crate_test_membership has no suites")
    workspace_packages = workspace_package_names()
    seen: set[str] = set()
    for suite in suites:
        if not isinstance(suite, dict):
            raise ProfileError(f"profile {profile.get('name')} has invalid crate test suite")
        suite_id = str(suite.get("id", ""))
        if suite_id in seen:
            raise ProfileError(f"profile {profile.get('name')} has duplicate crate test suite {suite_id}")
        seen.add(suite_id)
        command = suite.get("command", [])
        if not isinstance(command, list) or not command or not all(isinstance(arg, str) for arg in command):
            raise ProfileError(f"profile {profile.get('name')} crate test {suite_id} has invalid command")
        if command[0] != "test":
            raise ProfileError(f"profile {profile.get('name')} crate test {suite_id} must use cargo test")
        package = suite.get("package")
        if not isinstance(package, str) or not package:
            raise ProfileError(f"profile {profile.get('name')} crate test {suite_id} has invalid package")
        if package not in workspace_packages:
            raise ProfileError(
                f"profile {profile.get('name')} crate test {suite_id} references unknown package {package}"
            )
        command_package = package_from_cargo_test_command(command)
        if command_package != package:
            raise ProfileError(
                f"profile {profile.get('name')} crate test {suite_id} package {package} "
                f"does not match command package {command_package}"
            )
        status = suite.get("status")
        modes = suite.get("modes", [])
        if status == "red-blocker" and suite.get("executed_in_merge") is not False:
            raise ProfileError(f"profile {profile.get('name')} red-blocker {suite_id} must not execute in merge")
        if status == "red-blocker" and not suite.get("must_be_executed_by"):
            raise ProfileError(f"profile {profile.get('name')} red-blocker {suite_id} has no execution deadline")
        if "full" in modes and suite.get("executed_in_merge") is False and status != "red-blocker":
            raise ProfileError(
                f"profile {profile.get('name')} full-mode crate test {suite_id} "
                "must execute in merge unless it is a red-blocker"
            )


def workspace_package_names() -> set[str]:
    global _WORKSPACE_PACKAGE_NAMES
    if _WORKSPACE_PACKAGE_NAMES is not None:
        return _WORKSPACE_PACKAGE_NAMES
    proc = subprocess.run(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        cwd=REPO_ROOT,
        env={**os.environ, "CARGO_NET_OFFLINE": "true"},
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip()
        raise ProfileError(f"cargo metadata --locked failed while validating crate membership: {detail}")
    payload = json.loads(proc.stdout)
    packages = payload.get("packages") if isinstance(payload, dict) else None
    if not isinstance(packages, list):
        raise ProfileError("cargo metadata returned no packages while validating crate membership")
    _WORKSPACE_PACKAGE_NAMES = {
        package["name"]
        for package in packages
        if isinstance(package, dict) and isinstance(package.get("name"), str)
    }
    return _WORKSPACE_PACKAGE_NAMES


def package_from_cargo_test_command(command: list[str]) -> str | None:
    for index, arg in enumerate(command):
        if arg in {"-p", "--package"} and index + 1 < len(command):
            return command[index + 1]
        if arg.startswith("--package="):
            return arg.removeprefix("--package=")
    return None


def crate_test_suites_for_mode(profile: dict[str, Any], mode: str) -> list[dict[str, Any]]:
    if mode not in {"smoke", "full"}:
        raise ProfileError(f"unsupported crate test mode: {mode}")
    membership = profile.get("crate_test_membership")
    suites = membership.get("suites") if isinstance(membership, dict) else None
    if not isinstance(suites, list) or not suites:
        raise ProfileError(f"profile {profile.get('name')} has no crate_test_membership suites")
    return [
        suite
        for suite in suites
        if isinstance(suite, dict) and mode in suite.get("modes", [])
    ]


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


def build_profile_plan(profile_name: str) -> dict[str, Any]:
    profile = load_profile(profile_name)
    legacy = legacy_facade(profile)
    selected_areas = [
        {
            "area": selection["area"],
            "suites": list(selection["suites"]),
            "resource_classes": list(selection["resource_classes"]),
        }
        for selection in profile["selected_areas"]
    ]
    e2e = legacy["e2e"]
    fixture_manifest = resolve_fixture_manifest(str(e2e.get("fixture_manifest", "")))
    return {
        "schema_version": 1,
        "profile": profile["name"],
        "description": profile.get("description", ""),
        "network_policy": profile.get("network_policy", {}),
        "cargo_policy": profile.get("cargo_policy", {}),
        "reference_host": profile.get("reference_host", {}),
        "execution_sandbox": profile.get("execution_sandbox", {}),
        "budgets": profile["budgets"],
        "selected_areas": selected_areas,
        "legacy_facade": {
            "matrix_suites": list(legacy["matrix_suites"]),
            "tooling_suites": list(legacy["tooling_suites"]),
            "distribution": legacy["distribution"],
            "generated_code_quality": legacy["generated_code_quality"],
            "performance_budget": legacy["performance_budget"],
            "crate_tests": legacy["crate_tests"],
            "hardening_suites": list(legacy["hardening_suites"]),
            "extra_checks": list(legacy["extra_checks"]),
        },
        "crate_test_membership": profile.get("crate_test_membership", {}),
        "e2e": {
            "fixture_manifest": "" if fixture_manifest is None else str(fixture_manifest.relative_to(REPO_ROOT)),
            "fixture_count": 0 if fixture_manifest is None else fixture_count(fixture_manifest),
            "sifr_jobs": e2e["sifr_jobs"],
            "rust_jobs": e2e["rust_jobs"],
            "run_jobs": e2e["run_jobs"],
            "cargo_build_jobs": e2e["cargo_build_jobs"],
            "max_group_fixtures": e2e["max_group_fixtures"],
            "disable_cache": e2e["disable_cache"],
        },
    }


def print_plan(profile_name: str) -> None:
    print(json.dumps(build_profile_plan(profile_name), indent=2, sort_keys=True))


def compare_plans(local_path: str, ci_path: str) -> int:
    local = load_json(Path(local_path))
    ci = load_json(Path(ci_path))
    keys = [
        "profile",
        "selected_areas",
        "legacy_facade",
        "crate_test_membership",
        "e2e",
        "network_policy",
        "cargo_policy",
        "reference_host",
        "execution_sandbox",
    ]
    mismatches = [key for key in keys if local.get(key) != ci.get(key)]
    if mismatches:
        print("profile plan mismatch: " + ", ".join(mismatches), file=sys.stderr)
        return 1
    print(f"profile plans equivalent: {local.get('profile')}")
    return 0


def run_command(argv: list[str]) -> int:
    if not argv:
        print("profiles command requires one of: profile, shell, summary, check, plan, compare-plans, run", file=sys.stderr)
        return 2
    command = argv[0]
    profile_name = _profile_arg(argv[1:]) if command in {"profile", "shell", "summary", "plan", "run"} else ""
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
    if command == "plan":
        print_plan(profile_name)
        return 0
    if command == "compare-plans":
        return compare_plans(_path_arg(argv[1:], "--local"), _path_arg(argv[1:], "--ci"))
    if command == "check":
        profiles = load_all_profiles()
        print(f"verification profiles valid: {', '.join(sorted(profiles))}")
        return 0
    if command == "run":
        from .profile_runner import run_profile

        return run_profile(profile_name, _forward_args(argv[1:]))
    print(f"unsupported profiles command: {command}", file=sys.stderr)
    return 2


def _profile_arg(argv: list[str]) -> str:
    for index, value in enumerate(argv):
        if value == "--profile" and index + 1 < len(argv):
            return argv[index + 1]
    raise ProfileError("--profile is required")


def _forward_args(argv: list[str]) -> list[str]:
    if "--" not in argv:
        return []
    separator = argv.index("--")
    return argv[separator + 1 :]


def _path_arg(argv: list[str], flag: str) -> str:
    for index, value in enumerate(argv):
        if value == flag and index + 1 < len(argv):
            return argv[index + 1]
    raise ProfileError(f"{flag} is required")
