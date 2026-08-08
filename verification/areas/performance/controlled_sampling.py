"""Bounded controlled sampling and contamination evidence."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any, Callable

from benchmark_manifest import RUNNER_VERSION, BenchmarkCase, BenchmarkError
from host_control import HostActivityMonitor


def run_controlled_case(
    case: BenchmarkCase,
    run_root: Path,
    sample_scale: str,
    *,
    require_controlled_host: bool,
    run_case_fn: Callable[[BenchmarkCase, Path, str], dict[str, Any]],
    monitor_factory: Callable[[], Any] = HostActivityMonitor,
    repo_root: Path | None = None,
) -> dict[str, Any]:
    attempts: list[dict[str, Any]] = []
    for attempt_index in range(1, 4):
        with monitor_factory() as monitor:
            result = run_case_fn(case, run_root, sample_scale)
        rejection_reasons = (
            monitor.rejection_reasons() if require_controlled_host else []
        )
        coefficient_variation = float(result["metrics"]["coefficient_variation"])
        if coefficient_variation > case.stability_limit:
            rejection_reasons.append("unstable-samples")
        rejection_reasons = sorted(set(rejection_reasons))
        attempts.append(
            {
                "attempt": attempt_index,
                "coefficient_variation": coefficient_variation,
                "stability_limit": case.stability_limit,
                "host_snapshots": monitor.snapshots,
                "rejection_reasons": rejection_reasons,
            }
        )
        if not rejection_reasons:
            result["control"] = {
                "status": "controlled" if require_controlled_host else "record-only",
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
        snapshots = [{"self_test": True}]

        def __enter__(self) -> Any:
            return self

        def __exit__(self, *_args: object) -> None:
            return None

        def rejection_reasons(self) -> list[str]:
            return []

    accepted = run_controlled_case(
        case,
        output_root,
        "manifest",
        require_controlled_host=True,
        run_case_fn=fake_run,
        monitor_factory=FakeMonitor,
    )
    if accepted["control"]["accepted_attempt"] != 2:
        raise BenchmarkError(
            "controlled retry self-test did not accept the stable retry"
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
            monitor_factory=FakeMonitor,
        ),
        "after 3 attempts",
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
