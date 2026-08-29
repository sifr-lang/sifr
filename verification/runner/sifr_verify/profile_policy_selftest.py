"""Profile policy self-tests for the verification runner."""

from __future__ import annotations

import copy
from typing import Any

from .cargo_setup import cargo_setup_command
from .errors import SchemaError
from .profiles import load_all_profiles
from .schemas import load_schema, validate_data


def profile_schema_self_test() -> None:
    profiles = load_all_profiles()
    profile_schema = load_schema("profile.schema.json")

    def require_schema_rejection(payload: dict[str, Any], description: str) -> None:
        try:
            validate_data(payload, profile_schema, source=f"{description} self-test")
        except SchemaError:
            return
        raise AssertionError(f"{description} was accepted")

    expected = {"create-pr", "merge", "nightly", "python-interop-live", "release"}
    if set(profiles) != expected:
        raise AssertionError(f"unexpected profiles: {sorted(profiles)}")
    for profile_name, profile in profiles.items():
        if profile.get("schema_version") != 2:
            raise AssertionError(f"{profile_name} must use profile schema v2")
        if "legacy_facade" in profile or "execution_mode" in profile:
            raise AssertionError(f"{profile_name} contains removed profile fields")
    live = profiles["python-interop-live"]
    invalid_v1 = {**live, "schema_version": 1}
    require_schema_rejection(invalid_v1, "profile schema version 1")

    invalid_mutations = []
    empty_suites = copy.deepcopy(live)
    empty_suites["selected_areas"][0]["suites"] = []
    invalid_mutations.append(("empty selected-area suites", empty_suites))
    duplicate_suites = copy.deepcopy(live)
    duplicate_suites["selected_areas"][0]["suites"].append("live-policy")
    invalid_mutations.append(("duplicate selected-area suite", duplicate_suites))
    for field in ("toolchain_steps", "guardrail_steps"):
        duplicate_step = copy.deepcopy(profiles["create-pr"])
        duplicate_step[field].append(duplicate_step[field][0])
        invalid_mutations.append((f"duplicate {field}", duplicate_step))
    for description, payload in invalid_mutations:
        require_schema_rejection(payload, description)
    if live["selected_areas"] != [
        {
            "area": "python_interop",
            "suites": ["live-policy", "live-examples"],
            "resource_classes": ["container-runtime", "network", "platform-specific"],
        }
    ]:
        raise AssertionError(
            "python-interop-live must select only the two live Python interop suites"
        )
    if live["toolchain_steps"] or live["guardrail_steps"]:
        raise AssertionError(
            "python-interop-live must not select toolchain or guardrail steps"
        )
    if cargo_setup_command(profiles["python-interop-live"]) != [
        "cargo",
        "fetch",
        "--locked",
    ]:
        raise AssertionError(
            "python-interop-live has a noncanonical Cargo setup command"
        )
    for profile_name in sorted(expected):
        default_budget = profiles[profile_name].get("default_step_budget")
        if not isinstance(default_budget, dict):
            raise AssertionError(f"{profile_name} has no default step budget")
        if (
            default_budget.get("enforcement") != "blocking"
            or int(default_budget.get("budget_ms", 0)) <= 0
        ):
            raise AssertionError(
                f"{profile_name} has a non-blocking default step budget: {default_budget}"
            )
        setup_budget = (
            profiles[profile_name].get("step_budgets", {}).get("cargo_cache_setup")
        )
        if setup_budget != {
            "budget_ms": 300_000,
            "enforcement": "blocking",
        }:
            raise AssertionError(
                f"{profile_name} has a noncanonical Cargo setup budget: {setup_budget}"
            )
    _python_interop_budget_policy_self_test(profiles)
    create_pr_step_budgets = profiles["create-pr"].get("step_budgets", {})
    _create_pr_fixed_budget_policy_self_test(create_pr_step_budgets)
    required_blocking_steps = {
        "area_generated_code_quality",
        "area_rust_interop",
        "toolchain_cargo_test_sifr_smoke",
        "area_runtime_platform",
        "toolchain_e2e_pass",
    }
    missing_step_budgets = sorted(
        required_blocking_steps.difference(create_pr_step_budgets)
    )
    if missing_step_budgets:
        raise AssertionError(f"create-pr step budgets missing: {missing_step_budgets}")
    for step in sorted(required_blocking_steps):
        budget = create_pr_step_budgets[step]
        if (
            budget.get("enforcement") != "blocking"
            or int(budget.get("budget_ms", 0)) <= 0
        ):
            raise AssertionError(
                f"create-pr step budget is not blocking/positive: {step}={budget}"
            )
    _profile_coverage_self_test(profiles)


def _python_interop_budget_policy_self_test(
    profiles: dict[str, dict[str, Any]],
) -> None:
    expected_budgets = {
        "create-pr": (2_400_000, 2_700_000),
        "merge": (2_400_000, 4_500_000),
        "nightly": (2_400_000, 4_500_000),
        "release": (2_400_000, 4_500_000),
    }
    for profile_name, (warm_budget_ms, cold_budget_ms) in (
        expected_budgets.items()
    ):
        python_interop_budget = profiles[profile_name].get("step_budgets", {}).get(
            "area_python_interop"
        )
        expected_budget = {
            "warm_budget_ms": warm_budget_ms,
            "cold_budget_ms": cold_budget_ms,
            "cache_classifier": "successful-input-receipt",
            "enforcement": "blocking",
        }
        if python_interop_budget != expected_budget:
            raise AssertionError(
                f"{profile_name} Python interop cache budget drifted: "
                f"{python_interop_budget}"
            )


def _create_pr_fixed_budget_policy_self_test(
    step_budgets: dict[str, Any],
) -> None:
    expected_budgets = {
        "area_rust_interop": 20_000,
        "area_runtime_platform": 300_000,
    }
    for step_name, budget_ms in expected_budgets.items():
        budget = step_budgets.get(step_name)
        expected = {"budget_ms": budget_ms, "enforcement": "blocking"}
        if budget != expected:
            raise AssertionError(f"create-pr {step_name} budget drifted: {budget}")


def _profile_coverage_self_test(profiles: dict[str, dict[str, Any]]) -> None:
    required_guardrails = {
        "hir-maintainability",
        "file-size",
        "maintainability-ratchets",
        "demo-emitted-freshness",
        "source-crate-dependency-direction",
        "submodule-ownership",
        "method-dispatch-authority",
        "unsafe-abi-contracts",
        "codegen-invariant-contracts",
        "sysroot-resource-certification",
        "stdlib-native-intrinsic-allowlist",
        "stdlib-native-adapter-reachability",
        "stdlib-manifest-schema",
        "stdlib-bootstrap-ordering",
        "driver-maintainability",
        "verification-hardening-self-test",
        "verification-runner-foundation",
    }
    required_area_suites = {
        "core_language": {"audit-fixtures"},
        "project_workspace": {"audit-fixtures"},
        "stdlib_parity": {
            "module-merge-check",
            "audit-fixtures",
            "complexity-resource",
            "module-inventory",
        },
        "developer_tooling": {"typescript-go-transfer", "diagnostic-rules"},
        "package_management": {"guardrails", "offline-merge-smoke"},
        "performance": {"frontend-syntax-guardrails"},
    }
    for profile_name in ("create-pr", "merge", "nightly", "release"):
        profile = profiles[profile_name]
        selected = {
            str(selection["area"]): {str(suite) for suite in selection["suites"]}
            for selection in profile["selected_areas"]
            if isinstance(selection, dict)
        }
        for area, suites in required_area_suites.items():
            missing = sorted(suites.difference(selected.get(area, set())))
            if missing:
                raise AssertionError(
                    f"{profile_name} lost canonical {area} coverage: {missing}"
                )
        guardrails = set(profile["guardrail_steps"])
        if not required_guardrails.issubset(guardrails):
            raise AssertionError(f"{profile_name} lost canonical guardrail coverage")
        toolchain = set(profile["toolchain_steps"])
        if "e2e-pass" not in toolchain or not any(
            str(step).startswith("cargo-test-sifr-") for step in toolchain
        ):
            raise AssertionError(f"{profile_name} lost crate-test or e2e coverage")
    for profile_name in ("nightly", "release"):
        if (
            "hardening-determinism-scale"
            not in profiles[profile_name]["guardrail_steps"]
        ):
            raise AssertionError(f"{profile_name} lost determinism-scale coverage")
