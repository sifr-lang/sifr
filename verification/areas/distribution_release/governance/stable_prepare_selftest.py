"""Mutation tests for credential-free protected stable publication preparation."""

from __future__ import annotations

import copy
import json
import shutil
import stat
import subprocess
import tempfile
import zipfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

from .common import (
    GovernanceError,
    sha256_file,
)
from .qualification_fixture import (
    build_evidence_bundle,
    create_fixture_source,
)
from .qualification_fixture_support import (
    configure_git,
    git,
    git_output,
)
from .stable_prepare import (
    materialize_stable_prepare,
    validate_stable_prepare_summary,
)

REPO_ROOT = Path(__file__).resolve().parents[4]
NOW = datetime(2098, 12, 20, tzinfo=timezone.utc)
CANDIDATE_PATH = "plans/releases/candidates/0.1.0"
EVIDENCE_FILES = {
    "plan_spec": "stable-release-plan.json",
    "release_report": "release-profile-report.json",
    "qualification_index": "qualification-artifact-index.json",
    "stable_support_claims": "stable-support-claims.json",
    "rust_validation_report": "rust-validation-report.json",
    "documentation_report": "documentation-report.json",
    "release_notes": "release-notes.md",
}


def run_self_tests() -> int:
    tests = (
        test_materialized_prepare,
        test_materialized_normal_prepare,
        test_prepare_rejects_input_drift,
        test_summary_contract,
        test_cli_producer,
        test_safe_artifact_extractor,
    )
    for test in tests:
        test()
        print(f"stable-prepare pass: {test.__name__}")
    print(f"stable publication prepare self-tests ok: tests={len(tests)}")
    return 0


def test_materialized_prepare() -> None:
    with StablePrepareFixture() as context:
        summary = prepare(context)
        assert summary["operation"] == "ga-activation"
        assert summary["mode"] == "initial"
        assert summary["source"]["commit"] == context["source_commit"]
        assert summary["evidence"]["commit"] == context["evidence_commit"]
        assert summary["mutation"]["proposed_index"]["generation"] == 8
        assert len(summary["artifacts"]) == 20
        assert (
            summary["marketplace"]["vsix_sha256"]
            == summary["artifacts"]["vsix"]["sha256"]
        )


def test_materialized_normal_prepare() -> None:
    with StablePrepareFixture(transition="normal") as context:
        arguments = prepare_arguments(context)
        arguments["mode"] = "resume"
        summary = materialize_stable_prepare(**arguments)
        assert summary["operation"] == "normal"
        assert summary["mode"] == "resume"
        assert summary["mutation"]["previous_index"]["generation"] == 8
        assert summary["mutation"]["proposed_index"]["generation"] == 9
        assert summary["mutation"]["proposed_index"]["channels"]["stable"] == "0.1.0"


def test_prepare_rejects_input_drift() -> None:
    with StablePrepareFixture() as context:
        mutations: tuple[tuple[str, Callable[[dict[str, Any]], None]], ...] = (
            (
                "evidence commit",
                lambda value: value.update({"evidence_commit": "a" * 40}),
            ),
            (
                "candidate path",
                lambda value: value.update(
                    {"candidate_path": "plans/releases/candidates/0.1.1"}
                ),
            ),
            (
                "plan digest",
                lambda value: value.update({"expected_plan_sha256": "b" * 64}),
            ),
            (
                "operation",
                lambda value: value.update({"operation": "normal"}),
            ),
            (
                "generation",
                lambda value: value.update({"proposed_generation": 7}),
            ),
            (
                "publication window",
                lambda value: value.update(
                    {"now": datetime(2098, 12, 26, tzinfo=timezone.utc)}
                ),
            ),
        )
        for label, mutation in mutations:
            arguments = prepare_arguments(context)
            mutation(arguments)
            expect_rejected(
                lambda value: materialize_stable_prepare(**value),
                arguments,
                label=label,
            )

        source_marker = context["source_root"] / "untracked.txt"
        source_marker.write_text("dirty\n", encoding="utf-8")
        expect_rejected(
            lambda value: materialize_stable_prepare(**value),
            prepare_arguments(context),
            label="dirty source",
        )
        source_marker.unlink()

        evidence_marker = context["evidence_root"] / "untracked.txt"
        evidence_marker.write_text("dirty\n", encoding="utf-8")
        expect_rejected(
            lambda value: materialize_stable_prepare(**value),
            prepare_arguments(context),
            label="dirty evidence",
        )
        evidence_marker.unlink()

        qualification = json.loads(
            (
                context["evidence_root"]
                / context["candidate_path"]
                / "qualification-artifact-index.json"
            ).read_text(encoding="utf-8")
        )
        artifact = qualification["artifacts"][0]
        artifact_path = (
            context["artifact_root"]
            / artifact["workflow_artifact_name"]
            / artifact["name"]
        )
        artifact_path.write_bytes(artifact_path.read_bytes() + b"drift")
        expect_rejected(
            lambda value: materialize_stable_prepare(**value),
            prepare_arguments(context),
            label="transported artifact drift",
        )


def test_summary_contract() -> None:
    with StablePrepareFixture() as context:
        summary = prepare(context)
        mutations: tuple[Callable[[dict[str, Any]], None], ...] = (
            lambda value: value["evidence"].update(
                {"candidate_path": "plans/releases/candidates/0.1.1"}
            ),
            lambda value: value["source"].update({"commit": "a" * 40}),
            lambda value: value["release_report"].update({"sha256": "0" * 64}),
            lambda value: value["qualification"].update({"run_attempt": 2}),
            lambda value: value["live_index"].update({"generation": 6}),
            lambda value: value["artifacts"].pop("installer"),
            lambda value: value["artifacts"]["vsix"].update(
                {"workflow_artifact_id": 99}
            ),
            lambda value: value["marketplace"].update({"vsix_sha256": "a" * 64}),
            lambda value: value["site"].update({"repository": "example.invalid/site"}),
        )
        for mutation in mutations:
            changed = copy.deepcopy(summary)
            mutation(changed)
            expect_rejected(
                validate_stable_prepare_summary,
                changed,
                label="summary mutation",
            )


def test_cli_producer() -> None:
    with StablePrepareFixture() as context:
        output = context["root"] / "stable-prepare.json"
        arguments = prepare_arguments(context)
        command = [
            "python3",
            str(REPO_ROOT / "scripts/distribution/release_governance.py"),
            "prepare-stable-publication",
            "--operation",
            arguments["operation"],
            "--mode",
            arguments["mode"],
            "--evidence-root",
            str(arguments["evidence_root"]),
            "--evidence-commit",
            arguments["evidence_commit"],
            "--candidate-path",
            arguments["candidate_path"],
            "--expected-plan-sha256",
            arguments["expected_plan_sha256"],
            "--source-root",
            str(arguments["source_root"]),
            "--live-index",
            str(arguments["live_index_path"]),
            "--artifact-root",
            str(arguments["artifact_root"]),
            "--proposed-generation",
            str(arguments["proposed_generation"]),
            "--out",
            str(output),
        ]
        completed = subprocess.run(
            command,
            cwd=REPO_ROOT,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if completed.returncode != 0:
            raise AssertionError(completed.stderr)
        validate_stable_prepare_summary(json.loads(output.read_text(encoding="utf-8")))
        second = subprocess.run(
            command,
            cwd=REPO_ROOT,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if second.returncode == 0:
            raise AssertionError("CLI overwrote an existing prepare summary")


def test_safe_artifact_extractor() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-artifact-extractor-") as directory:
        root = Path(directory)
        script = REPO_ROOT / "scripts/distribution/extract_github_artifact.py"
        safe_archive = root / "safe.zip"
        with zipfile.ZipFile(safe_archive, "w") as archive:
            archive.writestr("artifact.bin", b"exact bytes")
        safe_destination = root / "safe"
        safe_destination.mkdir()
        subprocess.run(
            [
                "python3",
                str(script),
                str(safe_archive),
                str(safe_destination),
                "--expected-uncompressed-bytes",
                "11",
            ],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert (safe_destination / "artifact.bin").read_bytes() == b"exact bytes"

        traversal_archive = root / "traversal.zip"
        with zipfile.ZipFile(traversal_archive, "w") as archive:
            archive.writestr("../escaped.bin", b"unsafe")
        link_archive = root / "link.zip"
        link = zipfile.ZipInfo("link")
        link.create_system = 3
        link.external_attr = (stat.S_IFLNK | 0o777) << 16
        with zipfile.ZipFile(link_archive, "w") as archive:
            archive.writestr(link, "target")
        for index, archive in enumerate((traversal_archive, link_archive)):
            destination = root / f"rejected-{index}"
            destination.mkdir()
            completed = subprocess.run(
                [
                    "python3",
                    str(script),
                    str(archive),
                    str(destination),
                    "--expected-uncompressed-bytes",
                    "6",
                ],
                check=False,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            if completed.returncode == 0:
                raise AssertionError(f"unsafe artifact ZIP passed: {archive.name}")
            if any(destination.iterdir()):
                raise AssertionError(f"unsafe artifact ZIP wrote bytes: {archive.name}")

        mismatch_destination = root / "rejected-byte-count"
        mismatch_destination.mkdir()
        mismatch = subprocess.run(
            [
                "python3",
                str(script),
                str(safe_archive),
                str(mismatch_destination),
                "--expected-uncompressed-bytes",
                "12",
            ],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if mismatch.returncode == 0:
            raise AssertionError("artifact ZIP byte-count mismatch passed")
        if any(mismatch_destination.iterdir()):
            raise AssertionError("artifact ZIP byte-count mismatch wrote bytes")


class StablePrepareFixture:
    """Context manager that creates an exact source/evidence/artifact bundle."""

    def __init__(self, *, transition: str = "ga-activation") -> None:
        self.temporary: tempfile.TemporaryDirectory[str] | None = None
        self.transition = transition

    def __enter__(self) -> dict[str, Any]:
        self.temporary = tempfile.TemporaryDirectory(
            prefix="sifr-stable-prepare-self-test-"
        )
        root = Path(self.temporary.name)
        source_root = create_fixture_source(root)
        bundle = build_evidence_bundle(
            source_root=source_root,
            evidence_root=root / "qualification",
            result_root=source_root / "target/results",
            transition=self.transition,
        )
        evidence_root = root / "evidence"
        candidate_root = evidence_root / CANDIDATE_PATH
        candidate_root.mkdir(parents=True)
        for source_key, destination_name in EVIDENCE_FILES.items():
            shutil.copyfile(bundle[source_key], candidate_root / destination_name)
        git(evidence_root, "init")
        configure_git(evidence_root)
        git(evidence_root, "add", ".")
        git(evidence_root, "commit", "-m", "candidate evidence")
        return {
            "root": root,
            "source_root": source_root,
            "source_commit": git_output(source_root, "rev-parse", "HEAD"),
            "evidence_root": evidence_root,
            "evidence_commit": git_output(evidence_root, "rev-parse", "HEAD"),
            "candidate_path": CANDIDATE_PATH,
            "expected_plan_sha256": sha256_file(
                candidate_root / "stable-release-plan.json"
            ),
            "live_index_path": bundle["active_index"],
            "artifact_root": bundle["artifact_root"],
            "operation": self.transition,
            "proposed_generation": 8 if self.transition == "ga-activation" else 9,
        }

    def __exit__(self, *_: object) -> None:
        if self.temporary is not None:
            self.temporary.cleanup()


def prepare(context: dict[str, Any]) -> dict[str, Any]:
    return materialize_stable_prepare(**prepare_arguments(context))


def prepare_arguments(context: dict[str, Any]) -> dict[str, Any]:
    return {
        "operation": context["operation"],
        "mode": "initial",
        "evidence_root": context["evidence_root"],
        "evidence_commit": context["evidence_commit"],
        "candidate_path": context["candidate_path"],
        "expected_plan_sha256": context["expected_plan_sha256"],
        "source_root": context["source_root"],
        "live_index_path": context["live_index_path"],
        "artifact_root": context["artifact_root"],
        "proposed_generation": context["proposed_generation"],
        "now": NOW,
    }


def expect_rejected(
    validator: Callable[[Any], Any],
    payload: Any,
    *,
    label: str,
) -> None:
    try:
        validator(payload)
    except (GovernanceError, ValueError):
        return
    raise AssertionError(f"{label} unexpectedly passed")


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
