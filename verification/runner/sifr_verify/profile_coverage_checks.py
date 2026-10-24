"""Preserve the canonical delivery-profile coverage obligations."""
from __future__ import annotations

from typing import Any

from .demo_freshness_policy import requires_strict_freshness, validate_freshness_ownership


def _profile_coverage_self_test(profiles: dict[str, dict[str, Any]]) -> None:
    required_guardrails = {
        "hir-maintainability",
        "file-size",
        "demo-emitted-freshness",
        "source-crate-dependency-direction",
        "submodule-ownership",
        "sysroot-resource-certification",
        "stdlib-native-intrinsic-allowlist",
        "stdlib-native-adapter-reachability",
        "stdlib-manifest-schema",
        "dx-metadata-inventory",
        "stdlib-bootstrap-ordering",
        "driver-maintainability",
        "method-dispatch-authority",
        "unsafe-abi-contracts",
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
        "developer_tooling": {"typescript-go-transfer", "diagnostic-rules", "architecture-policy"},
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
                raise AssertionError(f"{profile_name} lost canonical {area} coverage: {missing}")
        validate_freshness_ownership(profile)
        guardrails = set(profile["guardrail_steps"])
        if requires_strict_freshness(profile, "generated_code_quality"):
            guardrails.add("demo-emitted-freshness")
        if not required_guardrails.issubset(guardrails):
            raise AssertionError(f"{profile_name} lost canonical guardrail coverage")
        toolchain = set(profile["toolchain_steps"])
        if "e2e-pass" not in toolchain or not any(
            str(step).startswith("cargo-test-sifr-") for step in toolchain
        ):
            raise AssertionError(f"{profile_name} lost crate-test or e2e coverage")
    for profile_name in ("nightly", "release"):
        if "hardening-determinism-scale" not in profiles[profile_name]["guardrail_steps"]:
            raise AssertionError(f"{profile_name} lost determinism-scale coverage")


