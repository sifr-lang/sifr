"""Self-tests for reusable protected stable-publication primitives."""

from __future__ import annotations

import copy
import json
import os
import zipfile
from collections.abc import Callable, Iterator
from contextlib import contextmanager
from pathlib import Path

from scripts.distribution.fetch_qualification_artifacts import (
    fetch_qualification_artifacts,
)
from scripts.distribution.revalidate_stable_publication import (
    revalidate_stable_publication,
)

from .common import (
    GovernanceError,
    load_json_strict,
    sha256_file,
    write_canonical_json,
)
from .generation import allocate_next_generation
from .stable_prepare_selftest import (
    StablePrepareFixture,
    prepare,
)


def run_self_tests() -> int:
    tests = (
        test_generation_allocation_burns_gaps,
        test_exact_artifact_refetch,
        test_protected_revalidation,
    )
    for test in tests:
        test()
        print(f"stable-publication-primitives pass: {test.__name__}")
    print(f"stable publication primitives self-tests ok: tests={len(tests)}")
    return 0


def test_generation_allocation_burns_gaps() -> None:
    with StablePrepareFixture() as context:
        root = context["root"]
        history = root / "history"
        history.mkdir()
        live = load_json_strict(context["live_index_path"], require_canonical=True)
        write_canonical_json(
            history / f"channels-generation-{live['generation']}.json",
            live,
            refuse_existing=True,
        )
        burned = copy.deepcopy(live)
        burned["generation"] = live["generation"] + 3
        write_canonical_json(
            history / f"channels-generation-{burned['generation']}.json",
            burned,
            refuse_existing=True,
        )
        assert (
            allocate_next_generation(
                live_index_path=context["live_index_path"],
                snapshot_root=history,
            )
            == burned["generation"] + 1
        )

        invalid_name = history / "channels-generation-invalid.json"
        invalid_name.write_text("{}\n", encoding="utf-8")
        _expect_governance_error(
            lambda: allocate_next_generation(
                live_index_path=context["live_index_path"],
                snapshot_root=history,
            )
        )
        invalid_name.unlink()

        mismatched = copy.deepcopy(live)
        mismatched["generation"] = live["generation"] + 2
        mismatch_path = history / (
            f"channels-generation-{live['generation'] + 1}.json"
        )
        write_canonical_json(mismatch_path, mismatched, refuse_existing=True)
        _expect_governance_error(
            lambda: allocate_next_generation(
                live_index_path=context["live_index_path"],
                snapshot_root=history,
            )
        )
        mismatch_path.unlink()

        live_snapshot = history / f"channels-generation-{live['generation']}.json"
        drifted_live = copy.deepcopy(live)
        stable_version = drifted_live["channels"].get("stable")
        if stable_version is None:
            version = drifted_live["channels"]["alpha"]
        else:
            version = stable_version
        drifted_live["releases"][version]["installer_sha256"] = "f" * 64
        live_snapshot.unlink()
        write_canonical_json(live_snapshot, drifted_live, refuse_existing=True)
        _expect_governance_error(
            lambda: allocate_next_generation(
                live_index_path=context["live_index_path"],
                snapshot_root=history,
            )
        )


def test_exact_artifact_refetch() -> None:
    with StablePrepareFixture() as context:
        qualification_path = (
            context["evidence_root"]
            / context["candidate_path"]
            / "qualification-artifact-index.json"
        )
        qualification = load_json_strict(
            qualification_path,
            require_canonical=True,
        )
        api_root = context["root"] / "fake-api"
        _write_fake_github_api(
            api_root=api_root,
            qualification=qualification,
            artifact_root=context["artifact_root"],
        )
        with fake_gh(api_root):
            output = context["root"] / "refetched"
            fetch_qualification_artifacts(
                qualification_index_path=qualification_path,
                repository="sifr-lang/sifr",
                expected_source_commit=context["source_commit"],
                output_root=output,
            )
            for artifact in qualification["artifacts"]:
                expected = (
                    context["artifact_root"]
                    / artifact["workflow_artifact_name"]
                    / artifact["name"]
                )
                actual = (
                    output
                    / artifact["workflow_artifact_name"]
                    / artifact["name"]
                )
                assert actual.read_bytes() == expected.read_bytes()

            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="example.invalid/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=context["root"] / "rejected-repository",
                )
            )
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit="a" * 40,
                    output_root=context["root"] / "rejected-source",
                )
            )
            existing = context["root"] / "existing-output"
            existing.mkdir()
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=existing,
                )
            )
            output_parent = context["root"] / "artifact-output-parent"
            output_parent.mkdir()
            output_parent_link = context["root"] / "artifact-output-parent-link"
            output_parent_link.symlink_to(output_parent, target_is_directory=True)
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=output_parent_link / "rejected-parent",
                )
            )

            run_path = api_root / "run.json"
            run = json.loads(run_path.read_text(encoding="utf-8"))
            run["conclusion"] = "failure"
            run_path.write_text(json.dumps(run), encoding="utf-8")
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=context["root"] / "rejected-run",
                )
            )
            run["conclusion"] = "success"
            workflow_attempt = qualification["workflow"]["run_attempt"]
            run["run_attempt"] = workflow_attempt + 1
            run_path.write_text(json.dumps(run), encoding="utf-8")
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=context["root"] / "rejected-attempt",
                )
            )
            run["run_attempt"] = workflow_attempt
            run_path.write_text(json.dumps(run), encoding="utf-8")

            first_id = qualification["artifacts"][0]["workflow_artifact_id"]
            metadata_path = api_root / "metadata" / f"{first_id}.json"
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            metadata["expires_at"] = "2099-01-02T00:00:00Z"
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            rejected = context["root"] / "rejected-refetch"
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=rejected,
                )
            )
            assert not rejected.exists()
            metadata["expires_at"] = qualification["artifacts"][0]["expires_at"]
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")

            metadata["expired"] = True
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=context["root"] / "rejected-expired",
                )
            )
            metadata["expired"] = False

            workflow_run = metadata["workflow_run"]
            assert isinstance(workflow_run, dict)
            workflow_run["id"] = qualification["workflow"]["run_id"] + 1
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=context["root"] / "rejected-artifact-run",
                )
            )
            workflow_run["id"] = qualification["workflow"]["run_id"]

            authoritative_size = metadata["size_in_bytes"]
            assert isinstance(authoritative_size, int)
            metadata["size_in_bytes"] = authoritative_size + 1
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            rejected_short = context["root"] / "rejected-short-download"
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=rejected_short,
                )
            )
            assert not rejected_short.exists()

            metadata["size_in_bytes"] = authoritative_size - 1
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            rejected_overrun = context["root"] / "rejected-overrun-download"
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=qualification_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=rejected_overrun,
                )
            )
            assert not rejected_overrun.exists()
            metadata["size_in_bytes"] = authoritative_size
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")

            drifted_index = copy.deepcopy(qualification)
            drifted_index["artifacts"][0]["sha256"] = "f" * 64
            drifted_path = context["root"] / "drifted-qualification.json"
            write_canonical_json(drifted_path, drifted_index, refuse_existing=True)
            drifted_output = context["root"] / "rejected-content"
            _expect_governance_error(
                lambda: fetch_qualification_artifacts(
                    qualification_index_path=drifted_path,
                    repository="sifr-lang/sifr",
                    expected_source_commit=context["source_commit"],
                    output_root=drifted_output,
                )
            )
            assert not drifted_output.exists()


def test_protected_revalidation() -> None:
    with StablePrepareFixture() as context:
        summary_path = context["root"] / "prepare-summary.json"
        write_canonical_json(summary_path, prepare(context), refuse_existing=True)
        history = _write_live_snapshot(context)
        result = revalidate_stable_publication(
            prepare_summary_path=summary_path,
            expected_summary_sha256=sha256_file(summary_path),
            operation=context["operation"],
            mode="initial",
            evidence_root=context["evidence_root"],
            evidence_commit=context["evidence_commit"],
            candidate_path=context["candidate_path"],
            expected_plan_sha256=context["expected_plan_sha256"],
            source_root=context["source_root"],
            live_index_path=context["live_index_path"],
            snapshot_root=history,
            artifact_root=context["artifact_root"],
        )
        assert result["version"] == "0.1.0"
        _expect_governance_error(
            lambda: revalidate_stable_publication(
                prepare_summary_path=summary_path,
                expected_summary_sha256="a" * 64,
                operation=context["operation"],
                mode="initial",
                evidence_root=context["evidence_root"],
                evidence_commit=context["evidence_commit"],
                candidate_path=context["candidate_path"],
                expected_plan_sha256=context["expected_plan_sha256"],
                source_root=context["source_root"],
                live_index_path=context["live_index_path"],
                snapshot_root=history,
                artifact_root=context["artifact_root"],
            )
        )
        _expect_governance_error(
            lambda: revalidate_stable_publication(
                prepare_summary_path=summary_path,
                expected_summary_sha256=sha256_file(summary_path),
                operation=context["operation"],
                mode="resume",
                evidence_root=context["evidence_root"],
                evidence_commit=context["evidence_commit"],
                candidate_path=context["candidate_path"],
                expected_plan_sha256=context["expected_plan_sha256"],
                source_root=context["source_root"],
                live_index_path=context["live_index_path"],
                snapshot_root=history,
                artifact_root=context["artifact_root"],
            )
        )

        changed = prepare(context)
        changed["site"]["base_commit"] = "a" * 40
        changed_path = context["root"] / "changed-summary.json"
        write_canonical_json(changed_path, changed, refuse_existing=True)
        _expect_governance_error(
            lambda: revalidate_stable_publication(
                prepare_summary_path=changed_path,
                expected_summary_sha256=sha256_file(changed_path),
                operation=context["operation"],
                mode="initial",
                evidence_root=context["evidence_root"],
                evidence_commit=context["evidence_commit"],
                candidate_path=context["candidate_path"],
                expected_plan_sha256=context["expected_plan_sha256"],
                source_root=context["source_root"],
                live_index_path=context["live_index_path"],
                snapshot_root=history,
                artifact_root=context["artifact_root"],
            )
        )
        _expect_governance_error(
            lambda: revalidate_stable_publication(
                prepare_summary_path=context["root"] / "missing-summary.json",
                expected_summary_sha256="a" * 64,
                operation=context["operation"],
                mode="initial",
                evidence_root=context["evidence_root"],
                evidence_commit=context["evidence_commit"],
                candidate_path=context["candidate_path"],
                expected_plan_sha256=context["expected_plan_sha256"],
                source_root=context["source_root"],
                live_index_path=context["live_index_path"],
                snapshot_root=history,
                artifact_root=context["artifact_root"],
            )
        )

        burned = load_json_strict(
            context["live_index_path"],
            require_canonical=True,
        )
        burned["generation"] = context["proposed_generation"]
        write_canonical_json(
            history / f"channels-generation-{context['proposed_generation']}.json",
            burned,
            refuse_existing=True,
        )
        _expect_governance_error(
            lambda: revalidate_stable_publication(
                prepare_summary_path=summary_path,
                expected_summary_sha256=sha256_file(summary_path),
                operation=context["operation"],
                mode="initial",
                evidence_root=context["evidence_root"],
                evidence_commit=context["evidence_commit"],
                candidate_path=context["candidate_path"],
                expected_plan_sha256=context["expected_plan_sha256"],
                source_root=context["source_root"],
                live_index_path=context["live_index_path"],
                snapshot_root=history,
                artifact_root=context["artifact_root"],
            )
        )


def _write_fake_github_api(
    *,
    api_root: Path,
    qualification: dict[str, object],
    artifact_root: Path,
) -> None:
    workflow = qualification["workflow"]
    assert isinstance(workflow, dict)
    artifacts = qualification["artifacts"]
    assert isinstance(artifacts, list)
    metadata_root = api_root / "metadata"
    archive_root = api_root / "archives"
    metadata_root.mkdir(parents=True)
    archive_root.mkdir()
    (api_root / "run.json").write_text(
        json.dumps(
            {
                "id": workflow["run_id"],
                "run_attempt": workflow["run_attempt"],
                "head_sha": qualification["source_commit"],
                "conclusion": "success",
            }
        ),
        encoding="utf-8",
    )
    uploads: dict[int, list[dict[str, object]]] = {}
    for value in artifacts:
        assert isinstance(value, dict)
        artifact_id = value["workflow_artifact_id"]
        assert isinstance(artifact_id, int)
        uploads.setdefault(artifact_id, []).append(value)
    for artifact_id, entries in uploads.items():
        name = entries[0]["workflow_artifact_name"]
        expires_at = entries[0]["expires_at"]
        assert isinstance(name, str)
        archive_path = archive_root / f"{artifact_id}.zip"
        with zipfile.ZipFile(archive_path, "w") as archive:
            for artifact in entries:
                artifact_name = artifact["name"]
                assert isinstance(artifact_name, str)
                archive.write(
                    artifact_root / name / artifact_name,
                    arcname=artifact_name,
                )
        (metadata_root / f"{artifact_id}.json").write_text(
            json.dumps(
                {
                    "id": artifact_id,
                    "name": name,
                    "expired": False,
                    "expires_at": expires_at,
                    "size_in_bytes": archive_path.stat().st_size,
                    "workflow_run": {"id": workflow["run_id"]},
                }
            ),
            encoding="utf-8",
        )
    fake_bin = api_root / "bin"
    fake_bin.mkdir()
    fake = fake_bin / "gh"
    fake.write_text(
        """#!/usr/bin/env python3
import os
import pathlib
import re
import sys

root = pathlib.Path(os.environ["FAKE_GH_API_ROOT"])
endpoint = sys.argv[2]
if "/actions/runs/" in endpoint:
    source = root / "run.json"
else:
    match = re.search(r"/actions/artifacts/([1-9][0-9]*)(/zip)?$", endpoint)
    if match is None:
        raise SystemExit(2)
    artifact_id, is_zip = match.groups()
    source = (
        root / "archives" / f"{artifact_id}.zip"
        if is_zip
        else root / "metadata" / f"{artifact_id}.json"
    )
sys.stdout.buffer.write(source.read_bytes())
""",
        encoding="utf-8",
    )
    fake.chmod(0o755)


@contextmanager
def fake_gh(api_root: Path) -> Iterator[None]:
    original_path = os.environ.get("PATH", "")
    original_root = os.environ.get("FAKE_GH_API_ROOT")
    os.environ["PATH"] = f"{api_root / 'bin'}:{original_path}"
    os.environ["FAKE_GH_API_ROOT"] = str(api_root)
    try:
        yield
    finally:
        os.environ["PATH"] = original_path
        if original_root is None:
            os.environ.pop("FAKE_GH_API_ROOT", None)
        else:
            os.environ["FAKE_GH_API_ROOT"] = original_root


def _expect_governance_error(callback: Callable[[], object]) -> None:
    try:
        callback()
    except GovernanceError:
        return
    raise AssertionError("expected GovernanceError")


def _write_live_snapshot(context: dict[str, object]) -> Path:
    history = Path(context["root"]) / "history"
    history.mkdir()
    live_index_path = Path(context["live_index_path"])
    live = load_json_strict(live_index_path, require_canonical=True)
    write_canonical_json(
        history / f"channels-generation-{live['generation']}.json",
        live,
        refuse_existing=True,
    )
    return history


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
