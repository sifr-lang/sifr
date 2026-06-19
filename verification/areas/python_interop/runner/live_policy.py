from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path
from typing import Any, Callable

from env import RunnerPaths

REQUIRED_POLICY_KEYS = {
    "schema_version",
    "profile",
    "status",
    "required_resource_classes",
    "offline_profiles",
    "offline_forbidden_suites",
    "container_runtime",
    "result_statuses",
    "artifact_root",
    "notes",
}
REQUIRED_CONTAINER_KEYS = {
    "provider",
    "docker_required",
    "image_pull_policy",
    "cleanup_required",
    "timeout_seconds",
}
EXPECTED_STATUSES = {"policy-passed", "live-passed", "structured-skip", "live-failed"}


def build_live_policy_report(paths: RunnerPaths) -> dict[str, Any]:
    policy = _load_json(paths.area_root / "live_policy.json")
    _validate_policy_shape(policy)
    manifest = _load_json(paths.area_root / "manifest.json")
    _validate_live_manifest(manifest, policy)
    profile = _load_json(paths.repo_root / "verification" / "profiles" / f"{policy['profile']}.json")
    _validate_live_profile(profile, policy)
    _validate_offline_profiles(paths.repo_root, policy)
    return {
        "schema_version": 1,
        "area": "python_interop",
        "status": "policy-passed",
        "policy": policy,
        "summary": {
            "total_variants": 1,
            "total_failures": 0,
            "blocking_failures": 0,
            "non_blocking_failures": 0,
            "skipped": 0,
        },
    }


def run_live_policy_self_tests(paths: RunnerPaths) -> None:
    policy = _load_json(paths.area_root / "live_policy.json")
    manifest = _load_json(paths.area_root / "manifest.json")
    profile = _load_json(paths.repo_root / "verification" / "profiles" / f"{policy['profile']}.json")
    build_live_policy_report(paths)

    missing_key_policy = dict(policy)
    del missing_key_policy["result_statuses"]
    _expect_policy_failure(
        lambda: _validate_policy_shape(missing_key_policy),
        "missing keys: result_statuses",
    )

    drifted_status_policy = dict(policy)
    drifted_status_policy["result_statuses"] = ["policy-passed"]
    _expect_policy_failure(
        lambda: _validate_policy_shape(drifted_status_policy),
        "result statuses drifted",
    )

    poisoned_profile = deepcopy(profile)
    poisoned_profile["network_policy"]["mode"] = "offline"
    _expect_policy_failure(
        lambda: _validate_live_profile(poisoned_profile, policy),
        "network policy must be live",
    )

    poisoned_offline = _load_json(paths.repo_root / "verification" / "profiles" / "create-pr.json")
    poisoned_offline["resource_policy"]["classes"].append("container-runtime")
    _expect_policy_failure(
        lambda: _validate_offline_profile_payload(
            "create-pr",
            poisoned_offline,
            _string_set(policy, "offline_forbidden_suites"),
        ),
        "must not declare container-runtime",
    )

    poisoned_manifest = deepcopy(manifest)
    for suite in poisoned_manifest["suites"]:
        if suite["name"] == "live-policy":
            suite["network_mode"] = "offline"
            break
    _expect_policy_failure(
        lambda: _validate_live_manifest(poisoned_manifest, policy),
        "must declare live network mode",
    )


def _load_json(path: Path) -> dict[str, Any]:
    if not path.is_file():
        raise SystemExit(f"missing python interop live policy file: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit(f"python interop live policy JSON must be an object: {path}")
    return payload


def _validate_policy_shape(policy: dict[str, Any]) -> None:
    missing = sorted(REQUIRED_POLICY_KEYS.difference(policy))
    if missing:
        raise SystemExit(f"python interop live policy missing keys: {', '.join(missing)}")
    if policy["schema_version"] != 1:
        raise SystemExit("python interop live policy schema_version must be 1")
    if policy["status"] != "policy-only":
        raise SystemExit("python interop live policy status must be policy-only")
    required_classes = _string_set(policy, "required_resource_classes")
    if {"container-runtime", "network", "platform-specific"}.difference(required_classes):
        raise SystemExit("python interop live policy must require container-runtime, network, and platform-specific")
    statuses = _string_set(policy, "result_statuses")
    if statuses != EXPECTED_STATUSES:
        raise SystemExit(f"python interop live policy result statuses drifted: {sorted(statuses)}")
    container = policy["container_runtime"]
    if not isinstance(container, dict):
        raise SystemExit("python interop live policy container_runtime must be an object")
    missing_container = sorted(REQUIRED_CONTAINER_KEYS.difference(container))
    if missing_container:
        raise SystemExit(
            "python interop live policy container_runtime missing keys: "
            + ", ".join(missing_container)
        )
    if container["provider"] != "testcontainers":
        raise SystemExit("python interop live policy provider must be testcontainers")
    if container["docker_required"] is not True or container["cleanup_required"] is not True:
        raise SystemExit("python interop live policy must require Docker and cleanup")
    if not isinstance(container["timeout_seconds"], int) or container["timeout_seconds"] <= 0:
        raise SystemExit("python interop live policy timeout_seconds must be a positive integer")


def _validate_live_profile(profile: dict[str, Any], policy: dict[str, Any]) -> None:
    if profile.get("execution_mode") != "selected-areas-only":
        raise SystemExit("python-interop-live profile must use selected-areas-only execution")
    resource_policy = profile.get("resource_policy", {})
    classes = set(resource_policy.get("classes", [])) if isinstance(resource_policy, dict) else set()
    required_classes = _string_set(policy, "required_resource_classes")
    missing = sorted(required_classes.difference(classes))
    if missing:
        raise SystemExit(f"python-interop-live profile is missing resource classes: {', '.join(missing)}")
    network_policy = profile.get("network_policy", {})
    if not isinstance(network_policy, dict) or network_policy.get("mode") != "live":
        raise SystemExit("python-interop-live profile network policy must be live")
    if network_policy.get("live_network_allowed") is not True:
        raise SystemExit("python-interop-live profile must explicitly allow live network")
    selected_live_policy = False
    for selection in profile.get("selected_areas", []):
        if not isinstance(selection, dict) or selection.get("area") != "python_interop":
            continue
        suites = set(selection.get("suites", []))
        selected_live_policy = "live-policy" in suites
        selected_classes = set(selection.get("resource_classes", []))
        missing_selection = sorted(required_classes.difference(selected_classes))
        if missing_selection:
            raise SystemExit(
                "python-interop-live profile python_interop selection is missing resource classes: "
                + ", ".join(missing_selection)
            )
    if not selected_live_policy:
        raise SystemExit("python-interop-live profile must select python_interop:live-policy")


def _validate_live_manifest(manifest: dict[str, Any], policy: dict[str, Any]) -> None:
    required_classes = _string_set(policy, "required_resource_classes")
    suites = manifest.get("suites", [])
    if not isinstance(suites, list):
        raise SystemExit("python interop manifest suites must be an array")
    live_suite = next(
        (suite for suite in suites if isinstance(suite, dict) and suite.get("name") == "live-policy"),
        None,
    )
    if not isinstance(live_suite, dict):
        raise SystemExit("python interop manifest must declare live-policy suite")
    if live_suite.get("network_mode") != "live":
        raise SystemExit("python interop live-policy suite must declare live network mode")
    suite_classes = set(live_suite.get("resource_classes", []))
    missing = sorted(required_classes.difference(suite_classes))
    if missing:
        raise SystemExit(f"python interop live-policy suite is missing resource classes: {', '.join(missing)}")
    if not isinstance(live_suite.get("timeout_seconds"), int) or live_suite["timeout_seconds"] <= 0:
        raise SystemExit("python interop live-policy suite timeout_seconds must be a positive integer")
    cases = live_suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit("python interop live-policy suite must contain exactly one policy case")
    case = cases[0]
    if not isinstance(case, dict) or case.get("command") != "python-interop-live-policy":
        raise SystemExit("python interop live-policy suite must dispatch python-interop-live-policy")


def _validate_offline_profiles(repo_root: Path, policy: dict[str, Any]) -> None:
    forbidden = _string_set(policy, "offline_forbidden_suites")
    for profile_name in _string_list(policy, "offline_profiles"):
        profile = _load_json(repo_root / "verification" / "profiles" / f"{profile_name}.json")
        _validate_offline_profile_payload(profile_name, profile, forbidden)


def _validate_offline_profile_payload(
    profile_name: str,
    profile: dict[str, Any],
    forbidden_suites: set[str],
) -> None:
    network_policy = profile.get("network_policy", {})
    if not isinstance(network_policy, dict) or network_policy.get("mode") != "offline":
        raise SystemExit(f"offline profile {profile_name} must keep network mode offline")
    if network_policy.get("live_network_allowed") is not False:
        raise SystemExit(f"offline profile {profile_name} must forbid live network")
    resource_policy = profile.get("resource_policy", {})
    policy_classes = set(resource_policy.get("classes", [])) if isinstance(resource_policy, dict) else set()
    if "container-runtime" in policy_classes:
        raise SystemExit(f"offline profile {profile_name} must not declare container-runtime")
    for selection in profile.get("selected_areas", []):
        if not isinstance(selection, dict):
            continue
        selection_classes = set(selection.get("resource_classes", []))
        if "container-runtime" in selection_classes:
            raise SystemExit(f"offline profile {profile_name} must not select container-runtime")
        if selection.get("area") != "python_interop":
            continue
        suites = set(selection.get("suites", []))
        overlap = sorted(forbidden_suites.intersection(suites))
        if overlap:
            raise SystemExit(
                f"offline profile {profile_name} selects live python interop suites: "
                + ", ".join(overlap)
            )


def _string_set(payload: dict[str, Any], key: str) -> set[str]:
    return set(_string_list(payload, key))


def _string_list(payload: dict[str, Any], key: str) -> list[str]:
    values = payload.get(key)
    if not isinstance(values, list) or not all(isinstance(value, str) for value in values):
        raise SystemExit(f"python interop live policy {key} must be a string array")
    return values


def _expect_policy_failure(callback: Callable[[], None], expected: str) -> None:
    try:
        callback()
    except SystemExit as exc:
        if expected not in str(exc):
            raise
    else:
        raise SystemExit(f"negative live-policy self-test failed: expected {expected!r}")
