"""Shared fixture helpers for protected stable publication self-tests."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path
from typing import Any

from .common import write_canonical_json
from .stable_prepare_selftest import prepare
from .stable_publish import stage_stable_publication

REPO_ROOT = Path(__file__).resolve().parents[4]


def fixture_paths(context: dict[str, Any]) -> dict[str, Path]:
    candidate = context["evidence_root"] / context["candidate_path"]
    return {
        "prepare": context["root"] / "stable-prepare.json",
        "qualification": candidate / "qualification-artifact-index.json",
        "plan": candidate / "stable-release-plan.json",
    }


def stage_call(
    context: dict[str, Any],
    paths: dict[str, Path],
    name: str,
) -> Path:
    output = context["root"] / name
    stage_stable_publication(
        prepare_summary_path=paths["prepare"],
        qualification_index_path=paths["qualification"],
        artifact_root=context["artifact_root"],
        plan_path=paths["plan"],
        dispatcher_root=context["dispatcher_root"],
        output_root=output,
    )
    return output


def stage_fixture(context: dict[str, Any]) -> dict[str, Any]:
    paths = fixture_paths(context)
    summary = prepare(context)
    write_canonical_json(paths["prepare"], summary, refuse_existing=True)
    staged = stage_call(context, paths, "staged")
    paths.update({"summary": summary, "staged": staged})
    return paths


def run_command(
    command: list[str],
    *,
    env: dict[str, str] | None = None,
) -> None:
    completed = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=env,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if completed.returncode != 0:
        raise AssertionError(completed.stderr)
