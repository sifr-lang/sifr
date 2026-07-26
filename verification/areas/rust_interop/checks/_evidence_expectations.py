"""Validate evidence outcomes independently from fixture inventory topology."""

from __future__ import annotations

from typing import Any

EXPECTED_RESULTS = {
    "diagnostic",
    "future-owned",
    "future-owned-diagnostic",
    "pass",
    "runtime-error-state",
}
RUNTIME_ERROR_STATES = {"closed", "poisoned"}


def validate_evidence_expectation(
    failures: list[str],
    *,
    fixture_id: str,
    side: str,
    raw_path: str,
    text: str,
    evidence: dict[str, Any],
    status: Any,
    execution_kind: Any,
    required_diagnostics: set[str],
) -> None:
    """Require an outcome model that the selected execution kind can prove."""
    label = f"{fixture_id}: evidence.{side}"
    expected_result = evidence.get("expected_result")
    if expected_result not in EXPECTED_RESULTS:
        failures.append(f"{label}.expected_result is invalid")
        return
    if f"# expected-result: {expected_result}" not in text:
        failures.append(f"{fixture_id}: {raw_path} missing expected-result header")
    if expected_result.startswith("future-owned") and status == "passing":
        failures.append(f"{fixture_id}: passing {side} evidence cannot be marked future-owned")
    if status != "passing" and not expected_result.startswith("future-owned"):
        failures.append(f"{fixture_id}: non-passing {side} evidence must be marked future-owned")

    expected_diagnostic = evidence.get("expected_diagnostic")
    expected_runtime_state = evidence.get("expected_runtime_state")
    if expected_result in {"diagnostic", "future-owned-diagnostic"}:
        if execution_kind == "runtime-observed" and expected_result == "diagnostic":
            failures.append(
                f"{label} runtime-observed evidence cannot claim a compiler diagnostic"
            )
        if expected_diagnostic not in required_diagnostics:
            failures.append(
                f"{label}.expected_diagnostic must be a reserved SIFR-RUST code"
            )
        elif f"# expected-diagnostic: {expected_diagnostic}" not in text:
            failures.append(
                f"{fixture_id}: {raw_path} missing expected diagnostic marker "
                f"{expected_diagnostic}"
            )
    elif expected_diagnostic is not None:
        failures.append(f"{label}.expected_diagnostic is allowed only for diagnostics")

    if expected_result == "runtime-error-state":
        if execution_kind != "runtime-observed":
            failures.append(
                f"{label} runtime-error-state requires runtime-observed execution"
            )
        if expected_runtime_state not in RUNTIME_ERROR_STATES:
            failures.append(
                f"{label}.expected_runtime_state must be closed or poisoned"
            )
        elif f"# expected-runtime-state: {expected_runtime_state}" not in text:
            failures.append(
                f"{fixture_id}: {raw_path} missing expected runtime state marker "
                f"{expected_runtime_state}"
            )
    elif expected_runtime_state is not None:
        failures.append(
            f"{label}.expected_runtime_state is allowed only for runtime-error-state"
        )


def run_self_test(required_diagnostics: set[str]) -> tuple[int, str | None]:
    """Mutation-test the cross-field outcome rules."""
    runtime_text = (
        "# expected-result: runtime-error-state\n"
        "# expected-runtime-state: closed\n"
    )
    valid_runtime = {
        "expected_result": "runtime-error-state",
        "expected_runtime_state": "closed",
    }
    control: list[str] = []
    validate_evidence_expectation(
        control,
        fixture_id="control",
        side="negative",
        raw_path="negative/control.sifr",
        text=runtime_text,
        evidence=valid_runtime,
        status="passing",
        execution_kind="runtime-observed",
        required_diagnostics=required_diagnostics,
    )
    if control:
        return 0, f"valid runtime error expectation was rejected: {control}"

    diagnostic = sorted(required_diagnostics)[0]
    cases = (
        (
            "runtime diagnostic",
            {
                "expected_result": "diagnostic",
                "expected_diagnostic": diagnostic,
            },
            f"# expected-result: diagnostic\n# expected-diagnostic: {diagnostic}\n",
            "passing",
            "runtime-observed",
            "cannot claim a compiler diagnostic",
        ),
        (
            "non-runtime error state",
            valid_runtime,
            runtime_text,
            "passing",
            "contract-only",
            "requires runtime-observed execution",
        ),
        (
            "missing runtime error",
            {"expected_result": "runtime-error-state"},
            "# expected-result: runtime-error-state\n",
            "passing",
            "runtime-observed",
            "expected_runtime_state must be closed or poisoned",
        ),
        (
            "runtime error on pass",
            {
                "expected_result": "pass",
                "expected_runtime_state": "closed",
            },
            "# expected-result: pass\n",
            "passing",
            "runtime-observed",
            "allowed only for runtime-error-state",
        ),
        (
            "diagnostic on pass",
            {
                "expected_result": "pass",
                "expected_diagnostic": diagnostic,
            },
            "# expected-result: pass\n",
            "passing",
            "contract-only",
            "allowed only for diagnostics",
        ),
        (
            "unknown result",
            {"expected_result": "unknown"},
            "# expected-result: unknown\n",
            "passing",
            "contract-only",
            "expected_result is invalid",
        ),
    )
    for name, evidence, text, status, execution_kind, expected in cases:
        failures: list[str] = []
        validate_evidence_expectation(
            failures,
            fixture_id=name,
            side="negative",
            raw_path="negative/mutation.sifr",
            text=text,
            evidence=evidence,
            status=status,
            execution_kind=execution_kind,
            required_diagnostics=required_diagnostics,
        )
        if not any(expected in failure for failure in failures):
            return len(cases), f"{name} did not report {expected!r}: {failures}"
    return len(cases) + 1, None
