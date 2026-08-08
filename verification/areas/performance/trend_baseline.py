"""Governed approved-reference trend baseline capture."""

from __future__ import annotations

import hashlib
import subprocess
from pathlib import Path
from typing import Any


APPROVED_PROFILE = "approved-reference"
APPROVED_THERMAL_POLICY = "controlled-host"
APPROVAL_OWNER = "compiler/performance"


class TrendBaselineError(Exception):
    pass


def validate_capture_request(
    *,
    capture_requested: bool,
    capture_budget_baseline: bool,
    require_controlled_host: bool,
    sample_scale: str,
    selected_count: int,
    manifest_count: int,
    groups: set[str],
    case_ids: set[str],
    case_limit: int,
    approval_owner: str,
    profile: str,
    thermal_policy: str,
    repo_root: Path,
) -> None:
    if not capture_requested:
        return
    if capture_budget_baseline:
        raise TrendBaselineError(
            "trend and budget baselines cannot be captured in the same invocation"
        )
    if not require_controlled_host:
        raise TrendBaselineError(
            "approved trend baseline capture requires --require-controlled-host"
        )
    if sample_scale != "manifest":
        raise TrendBaselineError(
            "approved trend baseline capture requires manifest sample counts"
        )
    if groups or case_ids or case_limit or selected_count != manifest_count:
        raise TrendBaselineError(
            "approved trend baseline capture requires the complete benchmark manifest"
        )
    if approval_owner != APPROVAL_OWNER:
        raise TrendBaselineError(
            f"approved trend baseline capture requires --reference-approval {APPROVAL_OWNER}"
        )
    if profile != APPROVED_PROFILE:
        raise TrendBaselineError(
            f"approved trend baseline capture requires SIFR_VALIDATION_PROFILE={APPROVED_PROFILE}"
        )
    if thermal_policy != APPROVED_THERMAL_POLICY:
        raise TrendBaselineError(
            f"approved trend baseline capture requires SIFR_THERMAL_POLICY={APPROVED_THERMAL_POLICY}"
        )
    status = command_output(["git", "status", "--porcelain"], repo_root)
    if status != "":
        raise TrendBaselineError(
            "approved trend baseline capture requires a clean worktree"
        )


def baseline_from_reference_run(
    run_report: dict[str, Any],
    manifest_path: Path,
    evidence_path: Path,
    *,
    repo_root: Path,
    approval_owner: str,
) -> dict[str, Any]:
    metadata = require_object(run_report, "metadata")
    host_control = require_object(metadata, "host_control")
    if host_control.get("status") != "controlled":
        raise TrendBaselineError(
            "approved trend baseline run did not record controlled host admission"
        )
    captured_at = require_int(metadata, "captured_at_unix", "run metadata")
    results = run_report.get("results")
    if not isinstance(results, list) or not results:
        raise TrendBaselineError("approved trend baseline run has no results")
    baseline_results = [baseline_result(result, captured_at) for result in results]
    source_commit = command_output(["git", "rev-parse", "HEAD"], repo_root)
    if len(source_commit) != 40:
        raise TrendBaselineError(
            "approved trend baseline capture could not resolve the source commit"
        )
    governed_metadata = {
        key: value
        for key, value in metadata.items()
        if key not in {"host_control", "cache_state_before", "cache_state_after"}
    }
    return {
        "schema_version": 1,
        "runner_version": run_report.get("runner_version"),
        "manifest_sha256": sha256(manifest_path),
        "metadata": governed_metadata,
        "results": baseline_results,
        "deferrals": [],
        "renames": [],
        "source_baseline": "approved-reference-run",
        "reference_capture": {
            "approval_owner": approval_owner,
            "controlled_host": True,
            "evidence_sha256": sha256(evidence_path),
            "invocation_id": require_string(run_report, "invocation_id", "run report"),
            "run_id": require_string(run_report, "run_id", "run report"),
            "source_commit": source_commit,
            "profile": require_string(metadata, "profile", "run metadata"),
            "thermal_policy": require_string(
                metadata, "thermal_policy", "run metadata"
            ),
            "host_control_policy": require_object(host_control, "policy"),
            "observation_count": require_int(
                host_control, "observation_count", "host control"
            ),
            "rejected_observations": require_int(
                host_control, "rejected_observation_count", "host control"
            ),
        },
    }


def baseline_result(raw: Any, captured_at: int) -> dict[str, Any]:
    if not isinstance(raw, dict):
        raise TrendBaselineError(
            "approved trend baseline result entries must be objects"
        )
    return {
        "id": require_string(raw, "id", "benchmark result"),
        "sample_count": require_int(raw, "sample_count", "benchmark result"),
        "samples_ms": raw.get("samples_ms"),
        "metrics": require_object(raw, "metrics"),
        "cache": require_object(raw, "cache"),
        "baseline_captured_at_unix": captured_at,
    }


def run_self_test() -> None:
    common = {
        "capture_requested": True,
        "capture_budget_baseline": False,
        "require_controlled_host": True,
        "sample_scale": "manifest",
        "selected_count": 2,
        "manifest_count": 2,
        "groups": set(),
        "case_ids": set(),
        "case_limit": 0,
        "approval_owner": APPROVAL_OWNER,
        "profile": APPROVED_PROFILE,
        "thermal_policy": APPROVED_THERMAL_POLICY,
    }
    assert_fails(
        lambda: validate_capture_request(
            **(common | {"require_controlled_host": False}), repo_root=Path.cwd()
        ),
        "requires --require-controlled-host",
    )
    assert_fails(
        lambda: validate_capture_request(
            **(common | {"selected_count": 1}), repo_root=Path.cwd()
        ),
        "complete benchmark manifest",
    )


def assert_fails(action: Any, expected: str) -> None:
    try:
        action()
    except TrendBaselineError as error:
        if expected not in str(error):
            raise TrendBaselineError(
                f"trend baseline self-test failed with wrong diagnostic: {error}"
            ) from error
        return
    raise TrendBaselineError(
        f"trend baseline self-test did not fail; expected {expected!r}"
    )


def require_object(raw: dict[str, Any], field: str) -> dict[str, Any]:
    value = raw.get(field)
    if not isinstance(value, dict):
        raise TrendBaselineError(f"field {field} must be an object")
    return value


def require_string(raw: dict[str, Any], field: str, owner: str) -> str:
    value = raw.get(field)
    if not isinstance(value, str) or not value:
        raise TrendBaselineError(f"{owner} field {field} must be a non-empty string")
    return value


def require_int(raw: dict[str, Any], field: str, owner: str) -> int:
    value = raw.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TrendBaselineError(f"{owner} field {field} must be an integer")
    return value


def command_output(command: list[str], cwd: Path) -> str:
    try:
        completed = subprocess.run(
            command, cwd=cwd, text=True, capture_output=True, timeout=30, check=False
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise TrendBaselineError(
            f"failed to run {' '.join(command)}: {error}"
        ) from error
    if completed.returncode != 0:
        raise TrendBaselineError(
            f"command failed ({' '.join(command)}): {completed.stderr.strip()}"
        )
    return completed.stdout.strip()


def sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()
