"""Bounded controlled sampling and contamination evidence."""

from __future__ import annotations

import json
import os
from collections.abc import Callable
from pathlib import Path
from typing import Any

from benchmark_manifest import RUNNER_VERSION, BenchmarkCase, BenchmarkError
from host_control import HostActivityMonitor


def run_controlled_case(
    case: BenchmarkCase,
    run_root: Path,
    sample_scale: str,
    *,
    require_controlled_host: bool,
    control_mode: str = "latency",
    run_case_fn: Callable[[BenchmarkCase, Path, str], dict[str, Any]],
    retry_admission_fn: Callable[[], dict[str, Any]] | None,
    monitor_factory: Callable[..., Any] = HostActivityMonitor,
    repo_root: Path | None = None,
) -> dict[str, Any]:
    attempts: list[dict[str, Any]] = []
    for attempt_index in range(1, 4):
        retry_admission = None
        if attempt_index > 1 and require_controlled_host:
            if retry_admission_fn is None:
                raise BenchmarkError(
                    "controlled sampling retry requires host re-admission"
                )
            retry_admission = retry_admission_fn()
        attempt_root = run_root / "attempts" / str(attempt_index)
        with monitor_factory(control_mode=control_mode) as monitor:
            result = run_case_fn(case, attempt_root, sample_scale)
        rejection_reasons = (
            monitor.rejection_reasons() if require_controlled_host else []
        )
        advisory_reasons: list[str] = []
        if control_mode == "work":
            raw_variation = result["metrics"].get(
                "instructions_coefficient_variation"
            )
            stability_metric = "instructions_coefficient_variation"
            stability_limit = case.work_stability_limit
        else:
            raw_variation = result["metrics"].get("coefficient_variation")
            stability_metric = "coefficient_variation"
            stability_limit = case.stability_limit
        if raw_variation is None:
            rejection_reasons.append(f"{stability_metric}-unavailable")
            coefficient_variation = None
        else:
            coefficient_variation = float(raw_variation)
        if coefficient_variation is not None and coefficient_variation > stability_limit:
            rejection_reasons.append("unstable-samples")
        if (
            control_mode == "work"
            and coefficient_variation is not None
            and coefficient_variation <= stability_limit
            and "external-cpu-pressure" in rejection_reasons
        ):
            # Retired instructions measure completed work rather than elapsed time.
            # Admission reserves capacity before the attempt; transient pressure
            # during a demonstrably stable work sample is useful telemetry, but it
            # must not make a local desktop require perfect idleness.
            rejection_reasons.remove("external-cpu-pressure")
            advisory_reasons.append("external-cpu-pressure")
        rejection_reasons = sorted(set(rejection_reasons))
        attempt_evidence = {
            "attempt": attempt_index,
            "control_mode": control_mode,
            "stability_metric": stability_metric,
            "coefficient_variation": coefficient_variation,
            "stability_limit": stability_limit,
            "host_snapshots": monitor.snapshots,
            "rejection_reasons": rejection_reasons,
        }
        if advisory_reasons:
            attempt_evidence["advisory_reasons"] = sorted(set(advisory_reasons))
        if retry_admission is not None:
            attempt_evidence["retry_admission"] = retry_admission
        attempts.append(attempt_evidence)
        if not rejection_reasons:
            result["control"] = {
                "status": "controlled" if require_controlled_host else "record-only",
                "mode": control_mode,
                "accepted_attempt": attempt_index,
                "attempts": attempts,
            }
            return result
    reasons = sorted(
        {reason for attempt in attempts for reason in attempt["rejection_reasons"]}
    )
    failure_path = run_root / "control-failures" / f"{case.id}.json"
    write_json(
        failure_path,
        {
            "schema_version": 1,
            "runner_version": RUNNER_VERSION,
            "case_id": case.id,
            "status": "rejected",
            "attempts": attempts,
            "rejection_reasons": reasons,
        },
    )
    failure_display = display_path(failure_path, repo_root)
    raise BenchmarkError(
        f"benchmark {case.id} did not produce a stable controlled sample after 3 attempts: "
        f"{', '.join(reasons)}; evidence={failure_display}"
    )


def run_self_test(output_root: Path) -> None:
    case = BenchmarkCase({"id": "controlled-retry-self-test", "stability_limit": 0.10})
    coefficients = iter([0.20, 0.05])

    def fake_run(_case: BenchmarkCase, _root: Path, _scale: str) -> dict[str, Any]:
        return {"metrics": {"coefficient_variation": next(coefficients)}}

    class FakeMonitor:
        def __init__(self, **_kwargs: object) -> None:
            self.snapshots = [{"self_test": True}]

        def __enter__(self) -> Any:
            return self

        def __exit__(self, *_args: object) -> None:
            return None

        def rejection_reasons(self) -> list[str]:
            return []

    retry_admissions: list[dict[str, Any]] = []

    def fake_retry_admission() -> dict[str, Any]:
        admission = {"status": "controlled", "self_test": True}
        retry_admissions.append(admission)
        return admission

    accepted = run_controlled_case(
        case,
        output_root,
        "manifest",
        require_controlled_host=True,
        run_case_fn=fake_run,
        retry_admission_fn=fake_retry_admission,
        monitor_factory=FakeMonitor,
    )
    if accepted["control"]["accepted_attempt"] != 2:
        raise BenchmarkError(
            "controlled retry self-test did not accept the stable retry"
        )
    if retry_admissions != [{"status": "controlled", "self_test": True}]:
        raise BenchmarkError(
            "controlled retry self-test did not reacquire the host exactly once"
        )
    if accepted["control"]["attempts"][1].get("retry_admission") != retry_admissions[0]:
        raise BenchmarkError(
            "controlled retry self-test did not record retry admission evidence"
        )

    def stable_work_run(
        _case: BenchmarkCase, _root: Path, _scale: str
    ) -> dict[str, Any]:
        return {
            "metrics": {
                "coefficient_variation": 0.50,
                "instructions_coefficient_variation": 0.001,
            }
        }

    work_accepted = run_controlled_case(
        case,
        output_root,
        "manifest",
        require_controlled_host=True,
        control_mode="work",
        run_case_fn=stable_work_run,
        retry_admission_fn=fake_retry_admission,
        monitor_factory=FakeMonitor,
    )
    if work_accepted["control"]["attempts"][0]["stability_metric"] != (
        "instructions_coefficient_variation"
    ):
        raise BenchmarkError(
            "controlled sampling self-test did not select work stability"
        )

    class PressuredMonitor(FakeMonitor):
        def rejection_reasons(self) -> list[str]:
            return ["external-cpu-pressure"]

    pressured_work_accepted = run_controlled_case(
        case,
        output_root,
        "manifest",
        require_controlled_host=True,
        control_mode="work",
        run_case_fn=stable_work_run,
        retry_admission_fn=fake_retry_admission,
        monitor_factory=PressuredMonitor,
    )
    pressured_attempt = pressured_work_accepted["control"]["attempts"][0]
    if pressured_attempt["rejection_reasons"]:
        raise BenchmarkError(
            "stable work sampling self-test rejected transient CPU pressure"
        )
    if pressured_attempt.get("advisory_reasons") != ["external-cpu-pressure"]:
        raise BenchmarkError(
            "stable work sampling self-test did not preserve pressure telemetry"
        )

    def unstable_run(_case: BenchmarkCase, _root: Path, _scale: str) -> dict[str, Any]:
        return {"metrics": {"coefficient_variation": 0.20}}

    assert_fails(
        lambda: run_controlled_case(
            case,
            output_root,
            "manifest",
            require_controlled_host=True,
            run_case_fn=unstable_run,
            retry_admission_fn=fake_retry_admission,
            monitor_factory=FakeMonitor,
        ),
        "after 3 attempts",
    )
    if len(retry_admissions) != 3:
        raise BenchmarkError(
            "controlled retry exhaustion did not reacquire before both retries"
        )
    if not output_root.joinpath("control-failures", f"{case.id}.json").is_file():
        raise BenchmarkError(
            "controlled retry self-test did not persist failure evidence"
        )


def assert_fails(action: Any, expected: str) -> None:
    try:
        action()
    except BenchmarkError as error:
        if expected not in str(error):
            raise BenchmarkError(
                f"controlled sampling self-test failed with wrong diagnostic: {error}"
            ) from error
        return
    raise BenchmarkError(
        f"controlled sampling self-test did not fail; expected {expected!r}"
    )


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temporary.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    temporary.replace(path)


def display_path(path: Path, repo_root: Path | None) -> Path:
    if repo_root is None:
        return path
    try:
        return path.relative_to(repo_root)
    except ValueError:
        return path
