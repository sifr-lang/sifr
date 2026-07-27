"""Editor-specific stable qualification mutations."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from .common import canonical_json_bytes
from .qualification_fixture import build_evidence_bundle, create_fixture_source

REPO_ROOT = Path(__file__).resolve().parents[4]
EDITOR_REPORT_CASES = (
    "bad-editor-shape",
    "editor-candidate-range",
    "editor-target-binding",
)


def mutate_editor_report(case: str, path: Path) -> dict[str, Any]:
    report = json.loads(path.read_text(encoding="utf-8"))
    if case == "bad-editor-shape":
        report["unexpected"] = True
    elif case == "editor-candidate-range":
        report["compiler_compatibility"] = ">=0.1.1,<0.2.0"
    elif case == "editor-target-binding":
        report["target_report_sha256"] = "1" * 64
    else:
        raise AssertionError(f"unknown editor report mutation: {case}")
    path.write_bytes(canonical_json_bytes(report))
    return report


def update_editor_plan(
    case: str,
    plan: dict[str, Any],
    *,
    report: dict[str, Any],
    report_sha256: str,
) -> None:
    plan["vscode"]["validation_report_sha256"] = report_sha256
    if case == "editor-candidate-range":
        plan["vscode"]["compiler_compatibility"] = report["compiler_compatibility"]


def test_planner_rejects_rollback_range_drift() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-plan-rollback-range-") as directory:
        root = Path(directory)
        source_root = create_fixture_source(root)
        bundle = build_evidence_bundle(
            source_root=source_root,
            evidence_root=root / "evidence",
            result_root=(
                source_root / "target" / "verification" / "fixture-rollback-range"
            ),
            transition="normal",
        )
        qualification = json.loads(
            bundle["qualification_index"].read_text(encoding="utf-8")
        )
        artifact = next(
            row
            for row in qualification["artifacts"]
            if row["id"] == "editor-qualification-report"
        )
        report_path = (
            bundle["artifact_root"]
            / artifact["workflow_artifact_name"]
            / artifact["name"]
        )
        report = json.loads(report_path.read_text(encoding="utf-8"))
        report["compiler_compatibility"] = ">=0.1.0,<0.2.0"
        report_path.write_bytes(canonical_json_bytes(report))
        artifact["sha256"] = hashlib.sha256(report_path.read_bytes()).hexdigest()
        artifact["size_bytes"] = report_path.stat().st_size
        bundle["qualification_index"].write_bytes(canonical_json_bytes(qualification))
        plan = json.loads(bundle["plan_spec"].read_text(encoding="utf-8"))
        plan["qualification_artifact_index"]["sha256"] = hashlib.sha256(
            bundle["qualification_index"].read_bytes()
        ).hexdigest()
        plan["vscode"]["compiler_compatibility"] = report["compiler_compatibility"]
        plan["vscode"]["validation_report_sha256"] = artifact["sha256"]
        bundle["plan_spec"].write_bytes(canonical_json_bytes(plan))
        result = subprocess.run(
            planner_command(bundle, root / "rejected-plan.json"),
            cwd=REPO_ROOT,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if result.returncode == 0:
            raise AssertionError("planner accepted rollback range drift")
        if "Traceback" in result.stderr:
            raise AssertionError(f"planner leaked a raw traceback: {result.stderr}")


def planner_command(bundle: dict[str, Any], output: Path) -> list[str]:
    return [
        sys.executable,
        str(REPO_ROOT / "scripts" / "distribution" / "release_governance.py"),
        "plan-stable-release",
        "--spec",
        str(bundle["plan_spec"]),
        "--source-root",
        str(bundle["source_root"]),
        "--source-ref",
        str(bundle["source_ref"]),
        "--live-index",
        str(bundle["active_index"]),
        "--release-report",
        str(bundle["release_report"]),
        "--qualification-index",
        str(bundle["qualification_index"]),
        "--artifact-root",
        str(bundle["artifact_root"]),
        "--stable-support-claims",
        str(bundle["stable_support_claims"]),
        "--rust-validation-report",
        str(bundle["rust_validation_report"]),
        "--documentation-report",
        str(bundle["documentation_report"]),
        "--release-notes",
        str(bundle["release_notes"]),
        "--out",
        str(output),
    ]
