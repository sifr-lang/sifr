"""Execute explicit correctness groups, retaining each canonical result."""
from __future__ import annotations

from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import time

from .area_group_policy import lane_environment, partition_slots, reserve_bytes
from .batch_execution import execute_batch
from .demo_freshness_policy import requires_strict_freshness, validate_strict_freshness_evidence
from .paths import REPO_ROOT
from .profile_commands import CommandFailed
from .profile_results import AreaResultError, validate_area_result
from .resource_capacity import discover_capacity
from .step_budgets import prepare_step_budget


class StopAdmission(Exception):
    """Stop queued consumers; the batch owner cleans up admitted processes."""


def result_path(area, evidence_name):
    return REPO_ROOT / "target/verification/areas" / f"{area.replace('_', '-')}-{evidence_name}-results.json"


def compiler_digest(env):
    with Path(env["SIFR_GCQ_BIN"]).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def validate_result(runner, area, suites):
    payload = validate_area_result(result_path(area, runner.evidence_name),
                                   area=area, expected_suites=suites)
    if requires_strict_freshness(runner.profile, area):
        validate_strict_freshness_evidence(payload)
    if area == "python_interop" and not isinstance(payload.get("compiled_certification"), dict):
        raise AreaResultError("python_interop result lacks compiled certification")


def capacity_monitor(build):
    previous = float("-inf")
    def check():
        nonlocal previous
        now = time.monotonic()
        if now - previous < 1:
            return
        previous = now
        if discover_capacity().available_memory_bytes < reserve_bytes(build):
            raise CommandFailed(2, "area_capacity")
    return check


def run_area_group(runner, areas):
    # Imported lazily: the runner delegates here after its own module is loaded.
    from .profile_runner import StepResult
    names = {area: "area_" + area for area in areas}
    if set(names.values()).intersection(runner.completed_steps):
        raise ValueError("area group cannot execute an already completed category")
    if runner.preparation is None:
        raise ValueError("area group requires canonical preparation outcomes")
    selections = {item["area"]: list(item["suites"]) for item in runner.profile["selected_areas"]}
    blocked = {area: runner.preparation.blocked_by(names[area]) for area in areas}
    blocked = {area: reason for area, reason in blocked.items() if reason}
    if blocked and not runner.no_fail_fast:
        reason = next(iter(blocked.values()))
        blocked.update({area: "group prerequisite: " + reason for area in areas if area not in blocked})
    ready = [area for area in areas if area not in blocked]
    for area, reason in blocked.items():
        result_path(area, runner.evidence_name).unlink(missing_ok=True)
        runner.block_step(names[area], reason)
        runner.functional_exit_status = runner.functional_exit_status or 2
    if not ready:
        return 2

    partitions, parallel = partition_slots(ready, runner.build_allocation.slot_limit)
    parent = REPO_ROOT / "target/verification/area-groups"
    parent.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix=runner.evidence_name + "-", dir=parent))
    commands, environments, budgets, allocations = [], [], {}, {}
    outcomes, started, starts = {}, set(), {}
    stop_reason = None
    before = compiler_digest(runner.env)
    for area in ready:
        allocation, environment = lane_environment(runner.build_allocation, runner.env, partitions[area])
        allocations[area] = asdict(allocation)
        budgets[area] = prepare_step_budget(
            repo_root=REPO_ROOT, profile=runner.profile, profile_name=runner.evidence_name,
            name=names[area], env=environment)
        # A successful child that produces nothing must not inherit an old pass.
        result_path(area, runner.evidence_name).unlink(missing_ok=True)
        command = [sys.executable, "-m", "sifr_verify.area_group_worker",
                   "--profile", runner.profile_name, "--area", area]
        if runner.no_fail_fast:
            command.append("--no-fail-fast")
        commands.append(command)
        environments.append(environment)

    def save():
        (root / "outcomes.json").write_text(json.dumps({
            "areas": areas, "blocked": blocked, "allocations": allocations,
            "started": sorted(started), "outcomes": outcomes, "stop_reason": stop_reason,
        }, indent=2))

    def admitted(index):
        area = ready[index]
        started.add(area)
        starts[area] = time.monotonic()
        save()

    def completed(index, outcome):
        area = ready[index]
        if area in outcomes:
            raise ValueError("duplicate area outcome")
        for stream in ("stdout", "stderr"):
            data = getattr(outcome, stream)
            (root / f"{area}.{stream}").write_bytes(data)
            for line in data.decode("utf-8", errors="replace").splitlines():
                print(f"[child:{stream}] {json.dumps(line)}", flush=True)
        status = outcome.returncode
        cause = outcome.cause
        error = None
        if outcome.truncated:
            status, cause, error = 2, "output_truncated", "incomplete child output"
        elif cause != "exit" or status != 0:
            status = status or 2
        else:
            try:
                validate_result(runner, area, selections[area])
            except (AreaResultError, ValueError, OSError) as exc:
                status, error = 2, str(exc)
        outcomes[area] = {"status": status, "cause": cause, "error": error,
                          "elapsed_ms": round(outcome.elapsed_seconds * 1000)}
        save()
        if cause != "exit" or status < 0:
            runner.stop_requested = True
            raise StopAdmission(f"{area}: {cause}")
        if status and parallel == 1 and not runner.no_fail_fast:
            raise StopAdmission(f"{area}: failed before queued sibling admission")

    (root / "plan.json").write_text(json.dumps({
        "profile": runner.evidence_name, "areas": ready, "suites": {a: selections[a] for a in ready},
        "commands": commands, "allocations": allocations, "compiler_sha256": before,
        "aggregate_slots": runner.build_allocation.slot_limit, "max_parallel": parallel,
    }, indent=2))
    print("[sifr-area-group] " + json.dumps({"evidence": str(root), "areas": ready,
          "slots": partitions, "max_parallel": parallel}), flush=True)
    save()
    try:
        execute_batch(commands, cwd=REPO_ROOT, env=runner.env, env_overrides=environments,
                      max_parallel=parallel, started=admitted, completed=completed, own_sessions=True,
                      deadline_seconds=runner.env.get("SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS", "2400"),
                      limit_bytes=32 * 1024 * 1024, heartbeat=capacity_monitor(runner.build_allocation))
    except StopAdmission as exc:
        stop_reason = str(exc)
    except Exception as exc:
        stop_reason = repr(exc)
        runner.stop_requested = True
    try:
        if compiler_digest(runner.env) != before:
            raise ValueError("shared compiler changed during area execution")
    except (ValueError, OSError) as exc:
        stop_reason = str(exc)
        runner.stop_requested = True
        for area in ready:
            if area in outcomes:
                outcomes[area].update(status=2, cause="shared_compiler_changed", error=str(exc))
    failed = 2 if blocked else 0
    for area in ready:
        if area not in outcomes:
            if area not in started:
                runner.block_step(names[area], stop_reason or "missing area admission")
                runner.functional_exit_status = runner.functional_exit_status or 2
                failed = failed or 2
                continue
            outcomes[area] = {"status": 2, "cause": "incomplete",
                              "error": stop_reason or "missing area outcome",
                              "elapsed_ms": round((time.monotonic() - starts[area]) * 1000)}
        row = outcomes[area]
        runner.completed_steps.add(names[area])
        print(f"[sifr-lane-step] name={names[area]} elapsed_ms={row['elapsed_ms']} "
              f"status={'pass' if row['status'] == 0 else 'fail'}", flush=True)
        status = runner.finish_step(names[area], StepResult(row["status"], row["elapsed_ms"],
                                                          row["cause"]), budgets[area])
        failed = failed or status
    save()
    return failed
