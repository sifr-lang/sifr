"""Execution tests for stable publication protected-main preflight."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

from .common import sha256_file, write_canonical_json
from .stable_prepare_selftest import StablePrepareFixture, prepare

REPO_ROOT = Path(__file__).resolve().parents[4]


def test_orchestrator_rejects_unmerged_candidate() -> None:
    with StablePrepareFixture() as context:
        summary_path = context["root"] / "stable-prepare.json"
        write_canonical_json(summary_path, prepare(context), refuse_existing=True)
        fake_bin = context["root"] / "ancestry-bin"
        fake_bin.mkdir()
        fake_git = fake_bin / "git"
        fake_git.write_text(
            """#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == *"merge-base --is-ancestor"* ]]; then
  exit 1
fi
if [[ "$*" == *"rev-parse"* ]]; then
  printf '%s\\n' "${WORKFLOW_COMMIT}"
fi
""",
            encoding="utf-8",
        )
        fake_git.chmod(0o755)
        workflow_commit = "a" * 40
        environment = os.environ.copy()
        environment.update(
            {
                "PATH": f"{fake_bin}:{environment['PATH']}",
                "SITE_TOKEN": "fixture",
                "VSCE_BIN": "/usr/bin/true",
                "WORKFLOW_COMMIT": workflow_commit,
            }
        )
        command = [
            str(REPO_ROOT / "scripts/distribution/run_stable_publication.sh"),
            "--operation",
            "ga-activation",
            "--mode",
            "initial",
            "--repository",
            "sifr-lang/sifr",
            "--evidence-root",
            str(context["evidence_root"]),
            "--evidence-commit",
            context["evidence_commit"],
            "--candidate-path",
            context["candidate_path"],
            "--expected-plan-sha256",
            context["expected_plan_sha256"],
            "--source-root",
            str(context["source_root"]),
            "--prepare-summary",
            str(summary_path),
            "--expected-summary-sha256",
            sha256_file(summary_path),
            "--workflow-ref",
            "refs/heads/main",
            "--workflow-commit",
            workflow_commit,
            "--run-id",
            "99",
            "--run-attempt",
            "1",
            "--initiator",
            "initiator",
            "--approval-waiver",
            str(
                REPO_ROOT
                / "plans/releases/single-maintainer-approval-waiver.json"
            ),
            "--site-repository",
            "sifr-lang/sifr-website",
            "--site-workflow",
            "release-site.yml",
            "--site-workflow-ref",
            "stable-site",
            "--site-ruleset-id",
            "1",
            "--site-ruleset-updated-at",
            "2099-01-01T00:00:00Z",
            "--site-workflow-sha256",
            "b" * 64,
        ]
        completed = subprocess.run(
            command,
            cwd=REPO_ROOT,
            env=environment,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert completed.returncode != 0
        assert (
            "candidate source commit must be merged into protected main"
            in completed.stderr
        )
