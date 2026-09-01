"""Profile loading, policy helpers, and shell exports."""

from __future__ import annotations

import json
import os
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
E2E_PASS_FIXTURE_DIR = REPO_ROOT / "crates" / "sifr" / "tests" / "e2e" / "pass"
PYTHON_INTEROP_CAPABILITY_MATRIX = (
    REPO_ROOT / "verification" / "areas" / "python_interop" / "declaration_capabilities.json"
)
RUST_INTEROP_MANIFEST = REPO_ROOT / "verification" / "areas" / "rust_interop" / "manifest.json"
SQL_PLATFORM_MANIFEST = REPO_ROOT / "verification" / "areas" / "sql_platform" / "manifest.json"


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
    validate_data(
        payload,
        load_schema("profile.schema.json"),
        source=str(path.relative_to(REPO_ROOT)),
    )
    if payload.get("name") != profile:
        raise ProfileError(f"profile name '{payload.get('name')}' must match file stem '{profile}'")
    validate_selected_area_suites(payload)
    validate_toolchain_steps(payload)
    validate_crate_test_membership(payload)
    validate_step_budgets(payload)
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


def fixture_count(path: Path) -> int:
    payload = load_json(path)
    fixture_names = payload.get("fixture_names")
    if not isinstance(fixture_names, list) or not all(isinstance(name, str) for name in fixture_names):
        raise ProfileError(f"invalid fixture manifest: {path}")
    return len(fixture_names)


def full_pass_fixture_count() -> int:
    return sum(1 for path in E2E_PASS_FIXTURE_DIR.glob("*.sifr") if path.is_file())


def resolve_fixture_manifest(raw_path: str) -> Path | None:
    if not raw_path:
        return None
    path = (REPO_ROOT / raw_path).resolve()
    if not path.is_file():
        raise ProfileError(f"fixture manifest not found: {path}")
    return path


def selected_suites_for_area(profile: dict[str, Any], area_name: str) -> list[str]:
    suites: list[str] = []
    for selection in profile.get("selected_areas", []):
        if not isinstance(selection, dict) or selection.get("area") != area_name:
            continue
        raw_suites = selection.get("suites", [])
        if isinstance(raw_suites, list):
            suites.extend(str(suite) for suite in raw_suites)
    return suites


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
    seen_areas: set[str] = set()
    for selection in profile.get("selected_areas", []):
        area = selection.get("area") if isinstance(selection, dict) else None
        if not isinstance(area, str) or area not in area_suites:
            raise ProfileError(f"profile {profile.get('name')} selects unknown area: {area}")
        if area in seen_areas:
            raise ProfileError(f"profile {profile.get('name')} selects area {area} more than once")
        seen_areas.add(area)
        selected_suites = selection.get("suites", [])
        if len(selected_suites) != len(set(selected_suites)):
            raise ProfileError(f"profile {profile.get('name')} selects duplicate suites for {area}")
        for suite in selected_suites:
            if suite not in area_suites[area]:
                raise ProfileError(f"profile {profile.get('name')} selects unknown suite {area}:{suite}")
        if area == "python_interop" and profile.get("name") != "python-interop-live":
            required_suites = _compiled_evidence_suites()
            missing = sorted(required_suites.difference(selected_suites))
            if missing:
                raise ProfileError(
                    f"profile {profile.get('name')} omits required Python interop "
                    f"certification suites: {', '.join(missing)}"
                )
        if area == "rust_interop":
            required_suites = required_rust_interop_suites()
            missing = sorted(required_suites.difference(selected_suites))
            if missing:
                raise ProfileError(
                    f"profile {profile.get('name')} omits required Rust interop "
                    f"verification suites: {', '.join(missing)}"
                )
        if area == "sql_platform":
            required_suites = required_sql_platform_suites()
            missing = sorted(required_suites.difference(selected_suites))
            if missing:
                raise ProfileError(
                    f"profile {profile.get('name')} omits required SQL platform "
                    f"verification suites: {', '.join(missing)}"
                )
    if profile.get("name") != "python-interop-live" and "rust_interop" not in seen_areas:
        raise ProfileError(f"profile {profile.get('name')} omits the required Rust interop area")
    if profile.get("name") != "python-interop-live" and "sql_platform" not in seen_areas:
        raise ProfileError(f"profile {profile.get('name')} omits the required SQL platform area")


def required_rust_interop_suites() -> set[str]:
    manifest = load_json(RUST_INTEROP_MANIFEST)
    suites = manifest.get("suites") if isinstance(manifest, dict) else None
    if not isinstance(suites, list) or not suites:
        raise ProfileError("Rust interop area manifest has no suites")
    names = {str(suite["name"]) for suite in suites if isinstance(suite, dict) and isinstance(suite.get("name"), str)}
    if len(names) != len(suites):
        raise ProfileError("Rust interop area manifest has invalid or duplicate suites")
    return names


def required_sql_platform_suites() -> set[str]:
    manifest = load_json(SQL_PLATFORM_MANIFEST)
    suites = manifest.get("suites") if isinstance(manifest, dict) else None
    if not isinstance(suites, list) or not suites:
        raise ProfileError("SQL platform area manifest has no suites")
    names = {
        str(suite["name"])
        for suite in suites
        if isinstance(suite, dict)
        and isinstance(suite.get("name"), str)
        and suite.get("network_mode", manifest.get("network_mode")) == "offline"
    }
    declared_names = {
        str(suite["name"])
        for suite in suites
        if isinstance(suite, dict) and isinstance(suite.get("name"), str)
    }
    if len(declared_names) != len(suites):
        raise ProfileError("SQL platform area manifest has invalid or duplicate suites")
    return names


def validate_toolchain_steps(profile: dict[str, Any]) -> None:
    steps = profile.get("toolchain_steps", [])
    if len(steps) != len(set(steps)):
        raise ProfileError(f"profile {profile.get('name')} has duplicate toolchain steps")
    crate_modes = [step for step in steps if str(step).startswith("cargo-test-sifr-")]
    if len(crate_modes) > 1:
        raise ProfileError(f"profile {profile.get('name')} selects more than one crate-test mode")
    if "e2e-report-determinism" in steps or "e2e-sequential-parallel-equivalence" in steps:
        if "e2e-pass" not in steps:
            raise ProfileError(f"profile {profile.get('name')} selects an e2e check without e2e-pass")


def _compiled_evidence_suites() -> set[str]:
    matrix = load_json(PYTHON_INTEROP_CAPABILITY_MATRIX)
    if not isinstance(matrix, dict) or matrix.get("schema_version") != 2:
        raise ProfileError("invalid Python interop capability matrix for profile validation")
    capabilities = matrix.get("capabilities")
    if not isinstance(capabilities, list):
        raise ProfileError("Python interop capability matrix has no capability rows")
    suites: set[str] = set()
    for row in capabilities:
        if not isinstance(row, dict):
            raise ProfileError("Python interop capability matrix contains an invalid row")
        evidence = row.get("compiled_evidence")
        if evidence is None:
            continue
        if not isinstance(evidence, list):
            raise ProfileError("Python interop capability matrix contains invalid compiled evidence")
        for item in evidence:
            suite = item.get("suite") if isinstance(item, dict) else None
            if not isinstance(suite, str) or not suite:
                raise ProfileError("Python interop compiled evidence has no owning suite")
            suites.add(suite)
    if not suites:
        raise ProfileError("Python interop capability matrix has no compiled evidence suites")
    return suites


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


def validate_step_budgets(profile: dict[str, Any]) -> None:
    raw_budgets = profile.get("step_budgets")
    if raw_budgets is None:
        return
    if not isinstance(raw_budgets, dict):
        raise ProfileError(f"profile {profile.get('name')} step_budgets must be an object")
    allowed_step_names = canonical_step_names(profile)
    for step_name, budget in raw_budgets.items():
        if step_name not in allowed_step_names:
            raise ProfileError(f"profile {profile.get('name')} step_budgets has unknown step {step_name}")
        if not isinstance(budget, dict):
            raise ProfileError(f"profile {profile.get('name')} step budget {step_name} must be an object")
        fixed_keys = {"budget_ms", "enforcement"}
        cache_keys = {
            "warm_budget_ms",
            "cold_budget_ms",
            "cache_classifier",
            "enforcement",
        }
        if set(budget) not in {frozenset(fixed_keys), frozenset(cache_keys)}:
            raise ProfileError(
                f"profile {profile.get('name')} step budget {step_name} "
                "must define either a fixed budget or warm/cold cache budgets"
            )
        budget_keys = (
            ["budget_ms"]
            if "budget_ms" in budget
            else [
                "warm_budget_ms",
                "cold_budget_ms",
            ]
        )
        for budget_key in budget_keys:
            budget_ms = budget.get(budget_key)
            if isinstance(budget_ms, bool) or not isinstance(budget_ms, int) or budget_ms <= 0:
                raise ProfileError(f"profile {profile.get('name')} step budget {step_name} has invalid {budget_key}")
        if "cache_classifier" in budget:
            if budget.get("cache_classifier") != "successful-input-receipt":
                raise ProfileError(
                    f"profile {profile.get('name')} step budget {step_name} has invalid cache_classifier"
                )
            if int(budget["cold_budget_ms"]) < int(budget["warm_budget_ms"]):
                raise ProfileError(
                    f"profile {profile.get('name')} step budget {step_name} "
                    "cold budget must not be lower than warm budget"
                )
        enforcement = budget.get("enforcement")
        if enforcement not in {"advisory", "blocking"}:
            raise ProfileError(f"profile {profile.get('name')} step budget {step_name} has invalid enforcement")


def canonical_step_names(profile: dict[str, Any]) -> set[str]:
    names = {"cargo_cache_setup"}
    names.update(
        f"guardrail_{str(step).replace('-', '_')}"
        for step in profile.get("guardrail_steps", [])
    )
    names.update(
        f"area_{selection['area']}"
        for selection in profile.get("selected_areas", [])
        if isinstance(selection, dict) and isinstance(selection.get("area"), str)
    )
    names.update(
        f"toolchain_{str(step).replace('-', '_')}"
        for step in profile.get("toolchain_steps", [])
    )
    return names


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
        package["name"] for package in packages if isinstance(package, dict) and isinstance(package.get("name"), str)
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
    return [suite for suite in suites if isinstance(suite, dict) and mode in suite.get("modes", [])]


def crate_test_mode(profile: dict[str, Any]) -> str | None:
    modes = {
        "cargo-test-sifr-smoke": "smoke",
        "cargo-test-sifr-full": "full",
    }
    selected = [modes[step] for step in profile.get("toolchain_steps", []) if step in modes]
    if len(selected) > 1:
        raise ProfileError(f"profile {profile.get('name')} selects more than one crate-test mode")
    return selected[0] if selected else None


def print_summary(requested_profile: str) -> None:
    profile = load_profile(requested_profile)
    e2e = profile["e2e"]
    fixture_manifest = resolve_fixture_manifest(str(e2e.get("fixture_manifest", "")))
    fixture_count_display = f"full-corpus ({full_pass_fixture_count()})"
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
    print(
        "  selected_areas="
        + ", ".join(
            f"{selection['area']}:{'+'.join(selection['suites'])}"
            for selection in profile["selected_areas"]
        )
    )
    print(f"  e2e={fixture_count_display} fixtures (manifest={fixture_manifest_display})")
    print("  toolchain_steps=" + (", ".join(profile["toolchain_steps"]) or "none"))
    print("  guardrail_steps=" + (", ".join(profile["guardrail_steps"]) or "none"))
    print(f"  manifest={profile_path(str(profile['name'])).relative_to(REPO_ROOT)}")


def build_profile_plan(profile_name: str) -> dict[str, Any]:
    profile = load_profile(profile_name)
    selected_areas = [
        {
            "area": selection["area"],
            "suites": list(selection["suites"]),
            "resource_classes": list(selection["resource_classes"]),
        }
        for selection in profile["selected_areas"]
    ]
    e2e = profile["e2e"]
    fixture_manifest = resolve_fixture_manifest(str(e2e.get("fixture_manifest", "")))
    fixture_selection = "full-corpus" if fixture_manifest is None else "manifest"
    selected_fixture_count = full_pass_fixture_count() if fixture_manifest is None else fixture_count(fixture_manifest)
    return {
        "schema_version": 2,
        "profile": profile["name"],
        "description": profile.get("description", ""),
        "network_policy": profile.get("network_policy", {}),
        "cargo_policy": profile.get("cargo_policy", {}),
        "reference_host": profile.get("reference_host", {}),
        "execution_sandbox": profile.get("execution_sandbox", {}),
        "budgets": profile["budgets"],
        "step_budgets": profile.get("step_budgets", {}),
        "selected_areas": selected_areas,
        "toolchain_steps": list(profile["toolchain_steps"]),
        "guardrail_steps": list(profile["guardrail_steps"]),
        "crate_test_membership": profile.get("crate_test_membership", {}),
        "e2e": {
            "fixture_manifest": "" if fixture_manifest is None else str(fixture_manifest.relative_to(REPO_ROOT)),
            "fixture_selection": fixture_selection,
            "fixture_count": selected_fixture_count,
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
        "schema_version",
        "profile",
        "selected_areas",
        "toolchain_steps",
        "guardrail_steps",
        "crate_test_membership",
        "e2e",
        "network_policy",
        "cargo_policy",
        "reference_host",
        "execution_sandbox",
        "step_budgets",
    ]
    mismatches = [key for key in keys if local.get(key) != ci.get(key)]
    if mismatches:
        print("profile plan mismatch: " + ", ".join(mismatches), file=sys.stderr)
        return 1
    print(f"profile plans equivalent: {local.get('profile')}")
    return 0


def run_command(argv: list[str]) -> int:
    if not argv:
        print(
            "profiles command requires one of: profile, summary, check, plan, compare-plans, run",
            file=sys.stderr,
        )
        return 2
    command = argv[0]
    profile_name = _profile_arg(argv[1:]) if command in {"profile", "summary", "plan", "run"} else ""
    if command == "profile":
        profile = load_profile(profile_name)
        print(profile["name"])
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

        return run_profile(
            profile_name,
            _forward_args(argv[1:]),
            release_report_out=_optional_arg(argv[1:], "--release-report-out"),
        )
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


def _optional_arg(argv: list[str], flag: str) -> str | None:
    values = [argv[index + 1] for index, value in enumerate(argv) if value == flag and index + 1 < len(argv)]
    if len(values) > 1:
        raise ProfileError(f"{flag} may be provided only once")
    return values[0] if values else None
