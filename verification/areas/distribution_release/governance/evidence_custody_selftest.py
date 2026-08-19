"""Focused mutation tests for candidate evidence-directory custody."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import Any

from .common import GovernanceError, canonical_json_bytes, sha256_bytes
from .evidence_custody import (
    require_comparison_base,
    validate_candidate_directory,
    validate_changed_path_set,
    validate_changed_path_sets,
)
from .planner import stage_stable_support_claims, validate_staged_support_claims
from .qualification_fixture import stable_claims
from .qualification_rust_fixture import rust_candidate_result
from .schema_contracts import qualification_index

REPO_ROOT = Path(__file__).resolve().parents[4]


def run_evidence_custody_mutations(
    *,
    valid_plan: Callable[[], dict[str, Any]],
    valid_report: Callable[[], dict[str, Any]],
    source_commit: str,
    retained_digest: str,
) -> None:
    _test_changed_paths()
    _test_stable_support_claim_staging()
    with tempfile.TemporaryDirectory() as directory:
        candidate_dir = Path(directory) / "0.1.0"
        candidate_dir.mkdir()
        report = valid_report()
        qualification_bytes = canonical_json_bytes(qualification_index())
        claims_bytes = canonical_json_bytes(stable_claims(variant="baseline"))
        rust_bytes = canonical_json_bytes(rust_candidate_result())
        rust_sha256 = sha256_bytes(rust_bytes)
        documentation_bytes = canonical_json_bytes(
            {
                "schema_version": 2,
                "kind": "stable-documentation-qualification",
                "report_id": "docs-a",
                "source_commit": source_commit,
                "suites": [
                    {"name": "structure", "status": "pass", "total_variants": 1},
                    {"name": "ga-release", "status": "pass", "total_variants": 1},
                ],
                "result_sha256": retained_digest,
                "status": "pass",
            }
        )
        release_notes_bytes = b"# Stable release notes\n"
        report["result_artifacts"] = [
            {
                "path": "target/verification/example.json",
                "sha256": retained_digest,
            },
            {
                "path": "target/verification/rust-interop-release-results.json",
                "sha256": rust_sha256,
            },
        ]
        for step in report["steps"]:
            for suite in step["suite_results"]:
                if suite["area"] == "rust_interop":
                    suite["result_artifact_sha256"] = rust_sha256
        report_bytes = canonical_json_bytes(report)
        plan = valid_plan()
        plan["release_profile_report"]["sha256"] = sha256_bytes(report_bytes)
        plan["qualification_artifact_index"]["sha256"] = sha256_bytes(
            qualification_bytes
        )
        plan["rust_interop"]["stable_support_claims_sha256"] = sha256_bytes(
            claims_bytes
        )
        plan["rust_interop"]["advertised_claim_ids"] = [
            "direct_crate_fixture",
            "bridge_fixture",
        ]
        plan["rust_interop"]["validation_report_sha256"] = rust_sha256
        plan["documentation_report"]["sha256"] = sha256_bytes(documentation_bytes)
        plan["release_notes_sha256"] = sha256_bytes(release_notes_bytes)
        files = {
            "stable-release-plan.json": canonical_json_bytes(plan),
            "release-profile-report.json": report_bytes,
            "qualification-artifact-index.json": qualification_bytes,
            "stable-support-claims.json": claims_bytes,
            "rust-validation-report.json": rust_bytes,
            "documentation-report.json": documentation_bytes,
            "release-notes.md": release_notes_bytes,
        }
        for name, contents in files.items():
            (candidate_dir / name).write_bytes(contents)
        validate_candidate_directory(candidate_dir)
        unexpected = candidate_dir / "nested"
        unexpected.mkdir()
        try:
            validate_candidate_directory(candidate_dir)
        except GovernanceError:
            pass
        else:
            raise AssertionError("nested candidate evidence passed custody")
        unexpected.rmdir()
        (candidate_dir / "release-profile-report.json").write_bytes(
            canonical_json_bytes({**report, "report_id": "tampered"})
        )
        try:
            validate_candidate_directory(candidate_dir)
        except GovernanceError:
            pass
        else:
            raise AssertionError("candidate report digest mismatch passed custody")


def _test_stable_support_claim_staging() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        source_root = root / "source"
        source_path = (
            source_root
            / "verification/areas/rust_interop/data/stable_support_claims.json"
        )
        source_path.parent.mkdir(parents=True)
        payload = stable_claims(variant="baseline")
        source_path.write_text(
            json.dumps(payload, indent=2, sort_keys=True),
            encoding="utf-8",
        )
        output = root / "evidence/stable-support-claims.json"
        stage_stable_support_claims(
            source_root=source_root,
            output_path=output,
        )
        if output.read_bytes() != canonical_json_bytes(payload):
            raise AssertionError("stable support claims staging was not canonical")
        cli_output = root / "evidence/cli-stable-support-claims.json"
        cli = subprocess.run(
            [
                sys.executable,
                str(REPO_ROOT / "scripts/distribution/release_governance.py"),
                "stage-stable-support-claims",
                "--source-root",
                str(source_root),
                "--out",
                str(cli_output),
            ],
            cwd=REPO_ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        if cli.returncode != 0 or cli_output.read_bytes() != canonical_json_bytes(
            payload
        ):
            raise AssertionError(
                f"stable support claims CLI staging failed: {cli.stderr}"
            )
        try:
            stage_stable_support_claims(
                source_root=source_root,
                output_path=output,
            )
        except GovernanceError:
            pass
        else:
            raise AssertionError("stable support claims staging overwrote evidence")
        output.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
        _require_governance_rejection(
            lambda: validate_staged_support_claims(output, source_root=source_root),
            "noncanonical staged stable support claims passed",
        )
        output.write_bytes(canonical_json_bytes(payload))
        source_path.write_bytes(
            canonical_json_bytes(stable_claims(variant="rust-claims"))
        )
        _require_governance_rejection(
            lambda: validate_staged_support_claims(output, source_root=source_root),
            "source-drifted stable support claims passed",
        )
        in_source = source_root / "candidate-claims.json"
        _require_governance_rejection(
            lambda: stage_stable_support_claims(
                source_root=source_root,
                output_path=in_source,
            ),
            "in-source stable support claims output passed",
        )
        source_path.unlink()
        linked_source = root / "linked-source-claims.json"
        linked_source.write_bytes(canonical_json_bytes(payload))
        source_path.symlink_to(linked_source)
        _require_governance_rejection(
            lambda: stage_stable_support_claims(
                source_root=source_root,
                output_path=root / "evidence/symlink-source-claims.json",
            ),
            "symlinked source stable support claims passed",
        )
        _require_governance_rejection(
            lambda: validate_staged_support_claims(
                output,
                source_root=source_root,
            ),
            "validator accepted symlinked source stable support claims",
        )


def _test_changed_paths() -> None:
    try:
        require_comparison_base("", base_ref="missing")
    except GovernanceError:
        pass
    else:
        raise AssertionError("missing evidence comparison base passed")
    candidate = "plans/releases/candidates/0.1.0/stable-release-plan.json"
    validate_changed_path_set({candidate})
    validate_changed_path_set({candidate, "plans/releases/README.md"})
    validate_changed_path_set({"verification/runner/sifr_verify/profiles.py"})
    validate_changed_path_sets(
        [
            {"verification/runner/sifr_verify/profiles.py"},
            {candidate},
        ]
    )
    for paths in (
        {candidate, "crates/sifr/src/main.rs"},
        {candidate, "plans/releases/candidates/0.1.1/stable-release-plan.json"},
        {"plans/releases/candidates/0.1.0/unexpected.json"},
    ):
        try:
            validate_changed_path_set(paths)
        except GovernanceError:
            continue
        raise AssertionError(f"invalid evidence custody paths passed: {paths}")


def _require_governance_rejection(action: Callable[[], Any], message: str) -> None:
    try:
        action()
    except GovernanceError:
        return
    raise AssertionError(message)
