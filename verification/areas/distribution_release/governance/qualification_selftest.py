"""Stable qualification collector, workflow, and planner self-tests."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from .common import TARGETS, canonical_json_bytes
from .artifact_index import validate_qualification_artifact_index
from .qualification_fixture import build_evidence_bundle, create_fixture_source
from .planner import (
    RUST_CLAIMS_SCHEMA_VERSION,
    resolve_source_once,
    stable_claim_ids,
    validate_rust_candidate_result,
    validate_target_report,
)
from .schema_contracts import release_plan, release_report

REPO_ROOT = Path(__file__).resolve().parents[4]
COMMIT = "e" * 40
VERSION = "0.1.0"


def run_self_tests() -> int:
    tests = (
        test_artifact_collector,
        test_artifact_collector_rejects_drift,
        test_artifact_index_exact_custody,
        test_planner_evidence_contract,
        test_rust_candidate_result_contract,
        test_materialized_planner_contract,
        test_planner_rejects_drift_cases,
        test_plan_digest_sensitivity,
    )
    for test in tests:
        test()
        print(f"qualification-self-test pass: {test.__name__}")
    print(f"qualification self-tests ok: tests={len(tests)}")
    return 0


def load_collector() -> Any:
    path = REPO_ROOT / "scripts" / "distribution" / "collect_qualification_artifacts.py"
    spec = importlib.util.spec_from_file_location("qualification_collector", path)
    if spec is None or spec.loader is None:
        raise AssertionError("could not load qualification artifact collector")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def create_fixture(root: Path) -> tuple[Path, Path, Path, Path]:
    prefix = f"sifr-stable-candidate-{VERSION}-{COMMIT}-"
    artifact_root = root / "artifacts"
    metadata: dict[str, Any] = {"artifacts": []}
    for artifact_id, suffix in enumerate([*TARGETS, "assemble", "editor"], start=1):
        workflow_name = f"{prefix}{suffix}"
        directory = artifact_root / workflow_name
        directory.mkdir(parents=True)
        if suffix in TARGETS:
            archive = f"sifr-{VERSION}-{suffix}.tar.gz"
            files = (
                archive,
                f"{archive}.sha256",
                f"sifr-{VERSION}-{suffix}-sysroot.tar.gz",
                f"qualification-{suffix}.json",
            )
        elif suffix == "assemble":
            files = ("checksums.txt", f"sifr-installer-{VERSION}")
        else:
            files = ("qualification-editor.json", "sifr-vscode-0.1.0.vsix")
        for name in files:
            (directory / name).write_text(f"{workflow_name}:{name}\n", encoding="utf-8")
        metadata["artifacts"].append(
            {
                "id": artifact_id,
                "name": workflow_name,
                "expired": False,
                "expires_at": "2099-01-01T00:00:00Z",
                "workflow_run": {"id": 42, "head_sha": COMMIT},
            }
        )
    metadata_path = root / "run-artifacts.json"
    metadata_path.write_bytes(canonical_json_bytes(metadata))
    run_metadata_path = root / "run-metadata.json"
    run_metadata_path.write_bytes(
        canonical_json_bytes(
            {
                "id": 42,
                "run_attempt": 1,
                "event": "workflow_dispatch",
                "name": "release-qualification",
                "repository": {"full_name": "sifr-lang/sifr"},
            }
        )
    )
    submodules_path = root / "submodules.json"
    submodules_path.write_bytes(
        canonical_json_bytes({"editor_integrations": "f" * 40})
    )
    return artifact_root, metadata_path, run_metadata_path, submodules_path


def collect_fixture(root: Path) -> dict[str, Any]:
    artifact_root, metadata_path, run_metadata_path, submodules_path = create_fixture(root)
    return load_collector().collect_index(
        version=VERSION,
        source_commit=COMMIT,
        submodules_path=submodules_path,
        run_id=42,
        run_attempt=1,
        run_metadata_path=run_metadata_path,
        metadata_path=metadata_path,
        artifact_root=artifact_root,
    )


def test_artifact_collector() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-qualification-collector-") as directory:
        root = Path(directory)
        first = collect_fixture(root)
        if len(first["artifacts"]) != 20:
            raise AssertionError("collector did not emit the complete qualification artifact set")
        second = load_collector().collect_index(
            version=VERSION,
            source_commit=COMMIT,
            submodules_path=root / "submodules.json",
            run_id=42,
            run_attempt=1,
            run_metadata_path=root / "run-metadata.json",
            metadata_path=root / "run-artifacts.json",
            artifact_root=root / "artifacts",
        )
        if canonical_json_bytes(first) != canonical_json_bytes(second):
            raise AssertionError("qualification collection is not byte deterministic")


def test_artifact_collector_rejects_drift() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-qualification-drift-") as directory:
        root = Path(directory)
        artifact_root, metadata_path, run_metadata_path, submodules_path = create_fixture(root)
        target_dir = artifact_root / f"sifr-stable-candidate-{VERSION}-{COMMIT}-{TARGETS[0]}"
        (target_dir / f"sifr-{VERSION}-{TARGETS[0]}.tar.gz.sha256").unlink()
        try:
            load_collector().collect_index(
                version=VERSION,
                source_commit=COMMIT,
                submodules_path=submodules_path,
                run_id=42,
                run_attempt=1,
                run_metadata_path=run_metadata_path,
                metadata_path=metadata_path,
                artifact_root=artifact_root,
            )
        except ValueError:
            pass
        else:
            raise AssertionError("collector accepted an incomplete target artifact")


def test_artifact_index_exact_custody() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-qualification-index-") as directory:
        index = collect_fixture(Path(directory))
    mutations = []
    renamed = copy.deepcopy(index)
    renamed["artifacts"][0]["id"] = "renamed-artifact"
    mutations.append(renamed)
    traversal = copy.deepcopy(index)
    traversal["artifacts"][0]["workflow_artifact_name"] = "../outside"
    mutations.append(traversal)
    extra = copy.deepcopy(index)
    extra_report = copy.deepcopy(
        next(row for row in extra["artifacts"] if row["kind"] == "report")
    )
    extra_report["id"] = "extra-report"
    extra["artifacts"].append(extra_report)
    mutations.append(extra)
    for mutation in mutations:
        try:
            validate_qualification_artifact_index(mutation)
        except ValueError:
            continue
        raise AssertionError("qualification index accepted non-canonical custody")


def test_planner_evidence_contract() -> None:
    plan = release_plan()
    target = plan["targets"][0]
    report = {
        "schema_version": 2,
        "kind": "stable-target-qualification",
        "candidate_version": plan["version"],
        "source_commit": plan["source_commit"],
        "target": target["triple"],
        "builder": target["builder"],
        "binary_sha256": target["binary_sha256"],
        "sysroot_sha256": target["sysroot_sha256"],
        "archive_sha256": target["archive_sha256"],
        "checksum_sha256": target["checksum_sha256"],
        "sysroot_bundle_sha256": "a" * 64,
        "sifr_version": target["sifr_version"],
        "installer_version": target["installer_version"],
        "receipt_channel": target["receipt_channel"],
        "sysroot_version": target["sysroot_version"],
        "sysroot_target": target["sysroot_target"],
        "smoke_status": "pass",
        "self_version_sha256": "b" * 64,
    }
    validate_target_report(
        report,
        version=plan["version"],
        source_commit=plan["source_commit"],
        target=target["triple"],
    )
    invalid = copy.deepcopy(report)
    invalid["source_commit"] = "f" * 40
    try:
        validate_target_report(
            invalid,
            version=plan["version"],
            source_commit=plan["source_commit"],
            target=target["triple"],
        )
    except ValueError:
        pass
    else:
        raise AssertionError("planner accepted a target report for another source")
    claims = {
        "schema_version": RUST_CLAIMS_SCHEMA_VERSION,
        "role": "compatibility-derived-release-plan-input",
        "source_compatibility_matrix": (
            "verification/areas/rust_interop/data/"
            "rust_interop_compatibility_matrix.json"
        ),
        "public_document": "docs/rust-interop.mdx",
        "runtime_deferrals": ["runtime-future"],
        "claims": [
            {
                "id": "direct-crate",
                "category": "supported",
                "execution_kind": "cargo-probe",
                "capability": "direct crate",
            },
            {
                "id": "bridge-contract",
                "category": "supported-through-bridge",
                "execution_kind": "contract-only",
                "capability": "bridge contract",
            },
        ],
    }
    if stable_claim_ids(claims) != ["direct-crate", "bridge-contract"]:
        raise AssertionError("planner did not preserve exact stable claim order")
    try:
        resolve_source_once(REPO_ROOT, "main")
    except ValueError:
        pass
    else:
        raise AssertionError("planner accepted a floating source ref")


def test_rust_candidate_result_contract() -> None:
    result = rust_candidate_result()
    report = release_report()
    report["result_artifacts"] = [
        {
            "path": "target/verification/areas/rust-interop-release-results.json",
            "sha256": "a" * 64,
        }
    ]
    validate_rust_candidate_result(
        result,
        expected_digest="a" * 64,
        release_report=report,
    )
    invalid = copy.deepcopy(result)
    invalid["suites"][-1]["cases"][0]["variants"][0]["status"] = "fail"
    try:
        validate_rust_candidate_result(
            invalid,
            expected_digest="a" * 64,
            release_report=report,
        )
    except ValueError:
        pass
    else:
        raise AssertionError("planner accepted a failing Rust stable-candidate result")


def rust_candidate_result() -> dict[str, Any]:
    suite_names = (
        "matrix",
        "tiers",
        "compatibility-matrix",
        "stale-drafts",
        "stable-candidate",
    )
    suites = []
    for suite_name in suite_names:
        case_ids = (
            (
                "rust-interop-stable-candidate",
                "rust-interop-stable-candidate-self-test",
            )
            if suite_name == "stable-candidate"
            else (f"rust-interop-{suite_name}",)
        )
        cases = [
            {
                "id": case_id,
                "variants": [
                    {
                        "actual_exit_code": 0,
                        "expected_exit_code": 0,
                        "mismatches": [],
                        "status": "pass",
                    }
                ],
            }
            for case_id in case_ids
        ]
        suites.append(
            {
                "blocking": True,
                "cases": cases,
                "failed_cases": 0,
                "name": suite_name,
                "total_failures": 0,
                "total_variants": len(cases),
            }
        )
    return {
        "area": "rust_interop",
        "bless": False,
        "manifest": "verification/areas/rust_interop/manifest.json",
        "suites": suites,
        "summary": {
            "blocking_failures": 0,
            "non_blocking_failures": 0,
            "total_failures": 0,
            "total_variants": sum(suite["total_variants"] for suite in suites),
        },
    }


def test_materialized_planner_contract() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-stable-plan-") as directory:
        root = Path(directory)
        source_root = create_fixture_source(root)
        bundle = build_evidence_bundle(
            source_root=source_root,
            evidence_root=root / "evidence",
            result_root=source_root / "target" / "verification" / "fixture-results",
        )
        first = run_planner(bundle, root / "first-plan.json")
        second = run_planner(bundle, root / "second-plan.json")
        if first != second:
            raise AssertionError("identical planner inputs did not produce identical bytes")
        if json.loads(first)["source_commit"] != bundle["source_ref"]:
            raise AssertionError("planner output did not bind the resolved fixture source")

        missing_artifact = (
            bundle["artifact_root"]
            / (
                f"sifr-stable-candidate-{VERSION}-{bundle['source_ref']}-"
                f"{TARGETS[0]}"
            )
            / f"sifr-{VERSION}-{TARGETS[0]}.tar.gz"
        )
        missing_artifact.unlink()
        expect_planner_rejected(bundle, root / "missing-artifact-plan.json")

        inside_output = REPO_ROOT / "target" / "qualification-plan-must-not-exist.json"
        inside_output.unlink(missing_ok=True)
        expect_planner_rejected(bundle, inside_output)
        if inside_output.exists():
            raise AssertionError("planner wrote evidence inside the repository checkout")


def test_planner_rejects_drift_cases() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-plan-drift-") as directory:
        root = Path(directory)
        source_root = create_fixture_source(root)
        for case in (
            "expired",
            "cross-target",
            "stale-report",
            "other-source",
            "version-drift",
        ):
            case_root = root / case
            bundle = build_evidence_bundle(
                source_root=source_root,
                evidence_root=case_root / "evidence",
                result_root=(
                    source_root / "target" / "verification" / f"fixture-{case}"
                ),
            )
            if case == "stale-report":
                report = json.loads(bundle["release_report"].read_text(encoding="utf-8"))
                report["source"]["commit"] = "f" * 40
                rewrite_canonical(bundle["release_report"], report)
                refresh_plan_reference(
                    bundle,
                    "release_profile_report",
                    bundle["release_report"],
                )
            else:
                qualification = json.loads(
                    bundle["qualification_index"].read_text(encoding="utf-8")
                )
                if case == "expired":
                    qualification["workflow"]["expires_at"] = "2000-01-01T00:00:00Z"
                    for artifact in qualification["artifacts"]:
                        artifact["expires_at"] = "2000-01-01T00:00:00Z"
                elif case == "cross-target":
                    report_id = f"qualification-report-{TARGETS[0]}"
                    artifact = next(
                        row
                        for row in qualification["artifacts"]
                        if row["id"] == report_id
                    )
                    report_path = (
                        bundle["artifact_root"]
                        / artifact["workflow_artifact_name"]
                        / artifact["name"]
                    )
                    report = json.loads(report_path.read_text(encoding="utf-8"))
                    report["target"] = TARGETS[1]
                    rewrite_canonical(report_path, report)
                    artifact["sha256"] = hashlib.sha256(
                        report_path.read_bytes()
                    ).hexdigest()
                    artifact["size_bytes"] = report_path.stat().st_size
                elif case == "other-source":
                    qualification["source_commit"] = "f" * 40
                elif case == "version-drift":
                    qualification["candidate_version"] = "0.1.1"
                rewrite_canonical(bundle["qualification_index"], qualification)
                refresh_plan_reference(
                    bundle,
                    "qualification_artifact_index",
                    bundle["qualification_index"],
                )
            expect_planner_rejected(bundle, case_root / "rejected-plan.json")


def rewrite_canonical(path: Path, payload: Any) -> None:
    path.write_bytes(canonical_json_bytes(payload))


def refresh_plan_reference(
    bundle: dict[str, Any],
    reference: str,
    evidence_path: Path,
) -> None:
    plan = json.loads(bundle["plan_spec"].read_text(encoding="utf-8"))
    plan[reference]["sha256"] = hashlib.sha256(evidence_path.read_bytes()).hexdigest()
    rewrite_canonical(bundle["plan_spec"], plan)


def run_planner(bundle: dict[str, Any], output: Path) -> bytes:
    command = planner_command(bundle, output)
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise AssertionError(
            f"planner rejected valid fixture: {result.stdout}{result.stderr}"
        )
    return output.read_bytes()


def expect_planner_rejected(bundle: dict[str, Any], output: Path) -> None:
    result = subprocess.run(
        planner_command(bundle, output),
        cwd=REPO_ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode == 0:
        raise AssertionError("planner accepted invalid qualification evidence")
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


def test_plan_digest_sensitivity() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-plan-sensitivity-") as directory:
        root = Path(directory)
        baseline_root = root / "baseline"
        source_root = create_fixture_source(baseline_root)
        bundle = build_evidence_bundle(
            source_root=source_root,
            evidence_root=baseline_root / "evidence",
            result_root=source_root / "target" / "verification" / "fixture-results",
        )
        baseline = run_planner(bundle, baseline_root / "plan.json")
        baseline_digest = hashlib.sha256(baseline).hexdigest()
        for variant in (
            "source",
            "submodule",
            "lock",
            "target-artifact",
            "sysroot",
            "installer",
            "rust-claims",
            "vsix",
        ):
            variant_root = root / variant
            variant_source = create_fixture_source(variant_root, variant=variant)
            variant_bundle = build_evidence_bundle(
                source_root=variant_source,
                evidence_root=variant_root / "evidence",
                result_root=(
                    variant_source / "target" / "verification" / "fixture-results"
                ),
                variant=variant,
            )
            changed = run_planner(variant_bundle, variant_root / "plan.json")
            if hashlib.sha256(changed).hexdigest() == baseline_digest:
                raise AssertionError(
                    f"planner input variant {variant} did not change the plan digest"
                )


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
