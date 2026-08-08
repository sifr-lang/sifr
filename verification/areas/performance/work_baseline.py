"""Governed local retired-instruction baseline capture."""

from __future__ import annotations

import subprocess
from collections.abc import Callable
from pathlib import Path

APPROVAL_OWNER = "compiler/performance"


class WorkBaselineError(Exception):
    pass


def validate_capture_request(
    *,
    capture_requested: bool,
    capture_budget_baseline: bool,
    require_controlled_host: bool,
    control_mode: str,
    sample_scale: str,
    selected_count: int,
    manifest_count: int,
    groups: set[str],
    case_ids: set[str],
    case_limit: int,
    approval_owner: str,
    repo_root: Path,
    git_output: Callable[[list[str], Path], str] | None = None,
) -> str | None:
    if not capture_requested:
        return None
    if capture_budget_baseline:
        raise WorkBaselineError(
            "work and wall-latency baselines must be captured separately"
        )
    if not require_controlled_host or control_mode != "work":
        raise WorkBaselineError(
            "work baseline capture requires controlled work-mode admission"
        )
    if sample_scale != "manifest":
        raise WorkBaselineError(
            "work baseline capture requires manifest sample counts"
        )
    if groups or case_ids or case_limit or selected_count != manifest_count:
        raise WorkBaselineError(
            "work baseline capture requires the complete benchmark manifest"
        )
    if approval_owner != APPROVAL_OWNER:
        raise WorkBaselineError(
            f"work baseline capture requires --reference-approval {APPROVAL_OWNER}"
        )
    git_output = git_output or command_output
    if git_output(["git", "status", "--porcelain"], repo_root) != "":
        raise WorkBaselineError("work baseline capture requires a clean worktree")
    source_commit = git_output(["git", "rev-parse", "HEAD"], repo_root)
    validate_source_commit(source_commit)
    return source_commit


def validate_source_unchanged(
    expected_source_commit: str,
    repo_root: Path,
    git_output: Callable[[list[str], Path], str] | None = None,
) -> None:
    git_output = git_output or command_output
    if git_output(["git", "status", "--porcelain"], repo_root) != "":
        raise WorkBaselineError(
            "work baseline capture worktree changed during measurement"
        )
    source_commit = git_output(["git", "rev-parse", "HEAD"], repo_root)
    validate_source_commit(source_commit)
    if source_commit != expected_source_commit:
        raise WorkBaselineError(
            "work baseline capture source commit changed during measurement"
        )


def validate_source_commit(source_commit: str) -> None:
    if len(source_commit) != 40 or any(
        char not in "0123456789abcdef" for char in source_commit
    ):
        raise WorkBaselineError(
            "work baseline capture could not resolve a full source commit"
        )


def command_output(command: list[str], cwd: Path) -> str:
    completed = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        capture_output=True,
        timeout=30,
        check=False,
    )
    if completed.returncode != 0:
        raise WorkBaselineError(
            f"work baseline git command failed: {' '.join(command)}"
        )
    return completed.stdout.strip()


def run_self_test() -> None:
    common = {
        "capture_requested": True,
        "capture_budget_baseline": False,
        "require_controlled_host": True,
        "control_mode": "work",
        "sample_scale": "manifest",
        "selected_count": 2,
        "manifest_count": 2,
        "groups": set(),
        "case_ids": set(),
        "case_limit": 0,
        "approval_owner": APPROVAL_OWNER,
    }

    def clean_git(command: list[str], _cwd: Path) -> str:
        return "a" * 40 if command[1:3] == ["rev-parse", "HEAD"] else ""

    source = validate_capture_request(
        **common, repo_root=Path("/self-test"), git_output=clean_git
    )
    if source != "a" * 40:
        raise WorkBaselineError("work baseline self-test lost source binding")
    bad_mode = dict(common)
    bad_mode["control_mode"] = "latency"
    assert_fails(
        lambda: validate_capture_request(
            **bad_mode, repo_root=Path("/self-test"), git_output=clean_git
        ),
        "work-mode",
    )
    assert_fails(
        lambda: validate_capture_request(
            **(common | {"case_limit": 1}),
            repo_root=Path("/self-test"),
            git_output=clean_git,
        ),
        "complete benchmark manifest",
    )


def assert_fails(action: Callable[[], object], expected: str) -> None:
    try:
        action()
    except WorkBaselineError as error:
        if expected not in str(error):
            raise WorkBaselineError(
                f"work baseline self-test failed with wrong diagnostic: {error}"
            ) from error
        return
    raise WorkBaselineError(
        f"work baseline self-test did not fail; expected {expected!r}"
    )
