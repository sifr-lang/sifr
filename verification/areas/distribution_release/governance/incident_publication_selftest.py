"""Self-tests for protected production incident publication primitives."""

from __future__ import annotations

import copy
import json
import os
import shutil
import subprocess
import tempfile
from collections.abc import Callable
from pathlib import Path

from scripts.distribution.verify_retained_stable_release import (
    verify_retained_release,
)

from .common import (
    GovernanceError,
    canonical_json_bytes,
    load_json_strict,
    sha256_bytes,
    sha256_file,
    write_canonical_json,
)
from .incident_prepare import materialize_incident_prepare
from .incident_publish import (
    SMOKE_FILES,
    materialize_incident_signoff,
    stage_incident_publication,
)
from .incident_recovery_selftest import build_rollback_fixture, git
from .schema_contracts import release_signoff
from .stable_prepare_selftest import StablePrepareFixture, prepare
from .stable_publish import PLAN_ASSET_NAME, stage_stable_publication

REPO_ROOT = Path(__file__).resolve().parents[4]


def run_self_tests() -> int:
    tests = (
        test_protected_rollback_prepare_publish_and_resume,
        test_protected_incident_roll_forward_prepare_publish_and_negatives,
        test_retained_release_adapter,
        test_protected_production_adapter_surface,
        test_incident_orchestrator_rejects_unprotected_and_unmerged,
    )
    for test in tests:
        test()
        print(f"incident-publication pass: {test.__name__}")
    print(f"incident publication self-tests ok: tests={len(tests)}")
    return 0


def test_protected_rollback_prepare_publish_and_resume() -> None:
    with tempfile.TemporaryDirectory(
        prefix="sifr-protected-rollback-"
    ) as temporary:
        root = Path(temporary)
        fixture = build_rollback_fixture(root / "fixture")
        dispatchers, dispatcher_digests = _dispatchers(root)
        target = load_json_strict(
            fixture.successor_plan,
            require_canonical=True,
        )
        target["site"]["dispatcher_sha256"] = dispatcher_digests
        _replace_json(fixture.successor_plan, target)
        target_plan_sha256 = sha256_file(fixture.successor_plan)
        affected = load_json_strict(fixture.affected_plan, require_canonical=True)
        affected["site"]["dispatcher_sha256"] = dispatcher_digests
        affected["expected_stable_predecessor"]["plan_sha256"] = (
            target_plan_sha256
        )
        affected["rollback_target"]["plan_sha256"] = target_plan_sha256
        _replace_json(fixture.affected_plan, affected)
        request = load_json_strict(fixture.request, require_canonical=True)
        request["affected_release"]["plan_sha256"] = sha256_file(
            fixture.affected_plan
        )
        request["rollback_target"]["plan_sha256"] = target_plan_sha256
        _replace_json(fixture.request, request)

        governance = root / "governance"
        _initialize_git(governance)
        for version, plan in (
            ("0.1.1", fixture.affected_plan),
            ("0.1.0", fixture.successor_plan),
        ):
            destination = (
                governance
                / "plans/releases/candidates"
                / version
                / "stable-release-plan.json"
            )
            destination.parent.mkdir(parents=True)
            shutil.copyfile(plan, destination)
        git(governance, "add", ".")
        git(governance, "commit", "-qm", "approved release plans")
        (governance / "protected-prepare").mkdir()

        incident = root / "incident"
        _initialize_git(incident)
        incident_directory = (
            incident / "plans/releases/incidents/inc-rollback-001"
        )
        incident_directory.mkdir(parents=True)
        shutil.copyfile(
            fixture.request,
            incident_directory / "stable-incident-request.json",
        )
        shutil.copyfile(
            fixture.request.with_name("withdrawal-evidence.txt"),
            incident_directory / "withdrawal-evidence.txt",
        )
        git(incident, "add", "plans")
        git(incident, "commit", "-qm", "incident evidence")
        incident_path = (
            "plans/releases/incidents/inc-rollback-001/"
            "stable-incident-request.json"
        )
        live = fixture.root / "live/channels.json"
        snapshots = fixture.root / "governance-release"
        arguments = {
            "operation": "rollback",
            "mode": "initial",
            "governance_root": governance,
            "incident_root": incident,
            "incident_commit": git(incident, "rev-parse", "HEAD").strip(),
            "incident_path": incident_path,
            "expected_request_sha256": sha256_file(incident / incident_path),
            "live_index_path": live,
            "snapshot_root": snapshots,
            "proposed_generation": 21,
        }
        tracked_request = incident / incident_path
        tracked_request_bytes = tracked_request.read_bytes()
        tracked_request.write_bytes(tracked_request_bytes + b"drift")
        _expect_rejected(lambda: materialize_incident_prepare(**arguments))
        tracked_request.write_bytes(tracked_request_bytes)
        tracked_plan = _plan(governance, "0.1.1")
        tracked_plan_bytes = tracked_plan.read_bytes()
        tracked_plan.write_bytes(tracked_plan_bytes + b"drift")
        _expect_rejected(lambda: materialize_incident_prepare(**arguments))
        tracked_plan.write_bytes(tracked_plan_bytes)
        untracked_governance = root / "governance-untracked-plan"
        shutil.copytree(governance, untracked_governance)
        untracked_plan = _plan(untracked_governance, "0.1.1")
        untracked_plan_bytes = untracked_plan.read_bytes()
        git(
            untracked_governance,
            "rm",
            "-q",
            str(untracked_plan.relative_to(untracked_governance)),
        )
        git(
            untracked_governance,
            "commit",
            "-qm",
            "remove approved affected plan",
        )
        untracked_plan.parent.mkdir(parents=True, exist_ok=True)
        untracked_plan.write_bytes(untracked_plan_bytes)
        assert (
            git(
                untracked_governance,
                "status",
                "--porcelain",
                "--untracked-files=no",
            )
            == ""
        )
        arguments["governance_root"] = untracked_governance
        _expect_rejected(lambda: materialize_incident_prepare(**arguments))
        arguments["governance_root"] = governance
        summary = materialize_incident_prepare(**arguments)
        assert summary["publication_state"] == "pending"
        assert summary["affected"]["version"] == "0.1.1"
        assert summary["successor"]["version"] == "0.1.0"
        assert summary["release_prepare"] == "none"

        summary_path = root / "prepare.json"
        write_canonical_json(summary_path, summary, refuse_existing=True)
        staged = root / "staged"
        stage_incident_publication(
            prepare_summary_path=summary_path,
            successor_plan_path=_plan(governance, "0.1.0"),
            site_plan_path=_plan(governance, "0.1.1"),
            dispatcher_root=dispatchers,
            output_root=staged,
        )
        mismatched_plan = load_json_strict(
            _plan(governance, "0.1.0"),
            require_canonical=True,
        )
        mismatched_plan["site"]["dispatcher_sha256"]["stable"] = "f" * 64
        mismatched_plan_path = root / "mismatched-target-plan.json"
        write_canonical_json(
            mismatched_plan_path,
            mismatched_plan,
            refuse_existing=True,
        )
        mismatched_summary = copy.deepcopy(summary)
        mismatched_digest = sha256_file(mismatched_plan_path)
        mismatched_summary["successor"]["plan_sha256"] = mismatched_digest
        mismatched_summary["mutation"]["successor_plan_sha256"] = (
            mismatched_digest
        )
        mismatched_summary["mutation"]["plan_sha256"] = mismatched_digest
        mismatched_summary_path = root / "mismatched-prepare.json"
        write_canonical_json(
            mismatched_summary_path,
            mismatched_summary,
            refuse_existing=True,
        )
        mismatched_output = root / "mismatched-staged"
        try:
            stage_incident_publication(
                prepare_summary_path=mismatched_summary_path,
                successor_plan_path=mismatched_plan_path,
                site_plan_path=_plan(governance, "0.1.1"),
                dispatcher_root=dispatchers,
                output_root=mismatched_output,
            )
        except GovernanceError as exc:
            assert "rollback target and affected site dispatcher digests disagree" in str(
                exc
            )
        else:
            raise AssertionError(
                "rollback staging accepted inconsistent dispatcher provenance"
            )
        assert not mismatched_output.exists()
        site_run = root / "site-run.json"
        write_canonical_json(
            site_run,
            {
                "repository": "sifr-lang/sifr-website",
                "workflow": "release-site.yml",
                "run_id": 77,
                "deployed_commit": summary["site"]["base_commit"],
            },
            refuse_existing=True,
        )
        smoke = _incident_smoke(root, summary)
        signoff = materialize_incident_signoff(
            prepare_summary_path=summary_path,
            request_path=incident / incident_path,
            withdrawal_evidence_path=(
                incident_directory / "withdrawal-evidence.txt"
            ),
            site_facts_path=staged / "stable-site-release-facts.json",
            site_run_path=site_run,
            smoke_root=smoke,
            run_id=78,
            approver="release-reviewer",
        )
        assert signoff["release_signoff_sha256"] == "none"
        assert signoff["site_reconciliation"]["run_id"] == 77
        _expect_rejected(
            lambda: materialize_incident_signoff(
                prepare_summary_path=summary_path,
                request_path=incident / incident_path,
                withdrawal_evidence_path=(
                    incident_directory / "withdrawal-evidence.txt"
                ),
                site_facts_path=(
                    staged / "stable-site-release-facts.json"
                ),
                site_run_path=site_run,
                smoke_root=smoke,
                run_id=78,
                approver="release-reviewer",
                release_signoff_path=summary_path,
            )
        )

        _replace_json(live, summary["mutation"]["proposed_index"])
        write_canonical_json(
            snapshots / "channels-generation-21.json",
            summary["mutation"]["proposed_index"],
            refuse_existing=True,
        )
        arguments["mode"] = "resume"
        arguments["proposed_generation"] = 22
        resumed = materialize_incident_prepare(**arguments)
        assert resumed["publication_state"] == "activated"
        assert resumed["mutation"]["proposed_index_sha256"] == resumed[
            "live_index"
        ]["sha256"]


def test_protected_incident_roll_forward_prepare_publish_and_negatives() -> None:
    with StablePrepareFixture() as context:
        affected_plan_path, request_path, withdrawal_path = (
            _configure_roll_forward(context)
        )
        governance = context["root"] / "governance"
        _initialize_git(governance)
        candidate_plan = (
            context["evidence_root"]
            / context["candidate_path"]
            / PLAN_ASSET_NAME
        )
        for version, plan in (
            ("0.0.9", affected_plan_path),
            ("0.1.0", candidate_plan),
        ):
            destination = _plan(governance, version)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(plan, destination)
        git(governance, "add", "plans")
        git(governance, "commit", "-qm", "approved incident plans")

        incident = context["root"] / "incident"
        _initialize_git(incident)
        incident_path = (
            "plans/releases/incidents/inc-forward-qualified/"
            "stable-incident-request.json"
        )
        incident_directory = (incident / incident_path).parent
        incident_directory.mkdir(parents=True)
        shutil.copyfile(request_path, incident / incident_path)
        shutil.copyfile(
            withdrawal_path,
            incident_directory / "withdrawal-evidence.txt",
        )
        git(incident, "add", "plans")
        git(incident, "commit", "-qm", "approved incident request")
        arguments = {
            "operation": "incident-roll-forward",
            "mode": "initial",
            "governance_root": governance,
            "incident_root": incident,
            "incident_commit": git(incident, "rev-parse", "HEAD").strip(),
            "incident_path": incident_path,
            "expected_request_sha256": sha256_file(incident / incident_path),
            "live_index_path": context["live_index_path"],
            "snapshot_root": context["snapshot_root"],
            "proposed_generation": 8,
            "candidate_root": context["evidence_root"],
            "candidate_commit": context["evidence_commit"],
            "candidate_path": context["candidate_path"],
            "expected_plan_sha256": context["expected_plan_sha256"],
            "source_root": context["source_root"],
            "artifact_root": context["artifact_root"],
        }
        summary = materialize_incident_prepare(**arguments)
        assert summary["operation"] == "incident-roll-forward"
        assert summary["release_prepare"]["operation"] == "incident-roll-forward"
        assert summary["successor"]["version"] == "0.1.0"

        summary_path = context["root"] / "incident-prepare.json"
        write_canonical_json(summary_path, summary, refuse_existing=True)
        staged = context["root"] / "incident-staged"
        stage_incident_publication(
            prepare_summary_path=summary_path,
            successor_plan_path=candidate_plan,
            site_plan_path=candidate_plan,
            dispatcher_root=context["dispatcher_root"],
            output_root=staged,
        )
        _expect_rejected(
            lambda: stage_incident_publication(
                prepare_summary_path=summary_path,
                successor_plan_path=candidate_plan,
                site_plan_path=affected_plan_path,
                dispatcher_root=context["dispatcher_root"],
                output_root=context["root"] / "wrong-site-plan",
            )
        )
        dispatcher = context["dispatcher_root"] / "stable"
        dispatcher_bytes = dispatcher.read_bytes()
        dispatcher.write_bytes(dispatcher_bytes + b"drift")
        _expect_rejected(
            lambda: stage_incident_publication(
                prepare_summary_path=summary_path,
                successor_plan_path=candidate_plan,
                site_plan_path=candidate_plan,
                dispatcher_root=context["dispatcher_root"],
                output_root=context["root"] / "dispatcher-drift",
            )
        )
        dispatcher.write_bytes(dispatcher_bytes)
        site_run = context["root"] / "incident-site-run.json"
        write_canonical_json(
            site_run,
            {
                "repository": "sifr-lang/sifr-website",
                "workflow": "release-site.yml",
                "run_id": 91,
                "deployed_commit": summary["site"]["base_commit"],
            },
            refuse_existing=True,
        )
        smoke = _incident_smoke(context["root"], summary)
        release = release_signoff()
        release["version"] = summary["successor"]["version"]
        release["plan_sha256"] = summary["successor"]["plan_sha256"]
        release["channel_generation"] = summary["mutation"]["proposed_index"][
            "generation"
        ]
        release["site_publication"]["deployed_commit"] = summary["site"][
            "base_commit"
        ]
        release_path = context["root"] / "stable-release-signoff.json"
        write_canonical_json(release_path, release, refuse_existing=True)
        signoff_arguments = {
            "prepare_summary_path": summary_path,
            "request_path": incident / incident_path,
            "withdrawal_evidence_path": (
                incident_directory / "withdrawal-evidence.txt"
            ),
            "site_facts_path": staged / "stable-site-release-facts.json",
            "site_run_path": site_run,
            "smoke_root": smoke,
            "run_id": 92,
            "approver": "release-reviewer",
            "release_signoff_path": release_path,
        }
        signoff = materialize_incident_signoff(**signoff_arguments)
        assert signoff["release_signoff_sha256"] == sha256_file(release_path)
        assert signoff["index_mutation"]["successor_version"] == "0.1.0"

        missing_release = dict(signoff_arguments)
        missing_release["release_signoff_path"] = None
        _expect_rejected(lambda: materialize_incident_signoff(**missing_release))
        wrong_site_run = context["root"] / "wrong-site-run.json"
        wrong_site = load_json_strict(site_run, require_canonical=True)
        wrong_site["deployed_commit"] = "f" * 40
        write_canonical_json(wrong_site_run, wrong_site, refuse_existing=True)
        mismatched_site = dict(signoff_arguments)
        mismatched_site["site_run_path"] = wrong_site_run
        _expect_rejected(lambda: materialize_incident_signoff(**mismatched_site))
        incomplete_smoke = context["root"] / "incomplete-smoke"
        shutil.copytree(smoke, incomplete_smoke)
        (incomplete_smoke / "incident-recovery.json").unlink()
        missing_smoke = dict(signoff_arguments)
        missing_smoke["smoke_root"] = incomplete_smoke
        _expect_rejected(lambda: materialize_incident_signoff(**missing_smoke))


def test_retained_release_adapter() -> None:
    with StablePrepareFixture() as context:
        candidate = context["evidence_root"] / context["candidate_path"]
        summary_path = context["root"] / "prepare.json"
        write_canonical_json(summary_path, prepare(context), refuse_existing=True)
        staged = context["root"] / "staged"
        stage_stable_publication(
            prepare_summary_path=summary_path,
            qualification_index_path=(
                candidate / "qualification-artifact-index.json"
            ),
            artifact_root=context["artifact_root"],
            plan_path=candidate / PLAN_ASSET_NAME,
            dispatcher_root=context["dispatcher_root"],
            output_root=staged,
        )
        metadata = context["root"] / "release.json"
        metadata.write_text(
            json.dumps(
                {
                    "tagName": "0.1.0",
                    "targetCommitish": context["source_commit"],
                    "isDraft": False,
                    "isPrerelease": False,
                }
            ),
            encoding="utf-8",
        )
        assets = staged / "release-assets"
        result = verify_retained_release(
            plan_path=candidate / PLAN_ASSET_NAME,
            qualification_path=candidate / "qualification-artifact-index.json",
            assets_root=assets,
            release_metadata_path=metadata,
            tag_commit=context["source_commit"],
        )
        assert result[PLAN_ASSET_NAME] == context["expected_plan_sha256"]
        drifted = assets / PLAN_ASSET_NAME
        drifted.write_bytes(drifted.read_bytes() + b"drift")
        try:
            verify_retained_release(
                plan_path=candidate / PLAN_ASSET_NAME,
                qualification_path=(
                    candidate / "qualification-artifact-index.json"
                ),
                assets_root=assets,
                release_metadata_path=metadata,
                tag_commit=context["source_commit"],
            )
        except GovernanceError:
            pass
        else:
            raise AssertionError("retained release verifier accepted drifted bytes")


def test_protected_production_adapter_surface() -> None:
    workflow = (
        REPO_ROOT / ".github/workflows/release-publication.yml"
    ).read_text(encoding="utf-8")
    dispatch = workflow.split("jobs:", 1)[0]
    assert "\n          - rollback\n" in dispatch
    assert "\n          - incident-roll-forward\n" in dispatch
    assert "- drill-rollback" in dispatch
    drill = (
        REPO_ROOT / ".github/workflows/release-publication-drill.yml"
    ).read_text(encoding="utf-8")
    assert "name: stable-release-drill" in drill
    assert "unshare --net --mount-proc" in drill
    assert "${{ secrets." not in drill
    assert "contents: write" not in drill
    assert "gh release" not in drill
    assert "vsce publish" not in drill
    assert "/dispatches" not in drill
    production = (
        REPO_ROOT / "scripts/distribution/run_incident_publication.sh"
    ).read_text(encoding="utf-8")
    assert "resolve-publication-approvers" in production
    assert "refs/heads/main" in production
    assert production.count("--clobber") == 1
    assert "dispatch_stable_site_publication.sh" in production
    harness = (
        Path(__file__).with_name("incident_fixture.py")
    ).read_text(encoding="utf-8")
    for forbidden in (
        "import socket",
        "import urllib",
        "import requests",
        "import subprocess",
    ):
        assert forbidden not in harness
    fixture_cli = (
        REPO_ROOT / "scripts/distribution/run_incident_fixture.py"
    ).read_text(encoding="utf-8")
    assert "gh release" not in fixture_cli
    assert "vsce publish" not in fixture_cli
    assert "repository_dispatch" not in fixture_cli


def test_incident_orchestrator_rejects_unprotected_and_unmerged() -> None:
    with tempfile.TemporaryDirectory(
        prefix="sifr-incident-orchestrator-"
    ) as temporary:
        root = Path(temporary)
        incident_root = root / "incident"
        incident_root.mkdir()
        prepare = root / "prepare.json"
        prepare.write_text("{}\n", encoding="utf-8")
        workflow_commit = "a" * 40
        incident_commit = "b" * 40
        command = [
            str(REPO_ROOT / "scripts/distribution/run_incident_publication.sh"),
            "--operation",
            "rollback",
            "--mode",
            "initial",
            "--repository",
            "sifr-lang/sifr",
            "--incident-root",
            str(incident_root),
            "--incident-commit",
            incident_commit,
            "--incident-path",
            "plans/releases/incidents/inc-unmerged-001/"
            "stable-incident-request.json",
            "--expected-request-sha256",
            "c" * 64,
            "--prepare-summary",
            str(prepare),
            "--expected-summary-sha256",
            "d" * 64,
            "--workflow-ref",
            "refs/heads/unprotected",
            "--workflow-commit",
            workflow_commit,
            "--run-id",
            "99",
            "--run-attempt",
            "1",
            "--initiator",
            "release-operator",
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
            "e" * 64,
        ]
        environment = os.environ.copy()
        environment["SITE_TOKEN"] = "fixture-site-token"
        rejected_ref = subprocess.run(
            command,
            cwd=REPO_ROOT,
            env=environment,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert rejected_ref.returncode != 0

        fake_bin = root / "fake-bin"
        fake_bin.mkdir()
        git_log = root / "git.log"
        fake_git = fake_bin / "git"
        fake_git.write_text(
            f"""#!/usr/bin/env bash
set -euo pipefail
printf '%s\\n' "$*" >>"${{FAKE_GIT_LOG}}"
case "$*" in
  *"rev-parse HEAD") printf '%s\\n' '{workflow_commit}' ;;
  *"fetch --no-tags origin main:refs/remotes/origin/main") ;;
  *"rev-parse refs/remotes/origin/main") printf '%s\\n' '{workflow_commit}' ;;
  *"merge-base --is-ancestor {incident_commit}"*) exit 1 ;;
  *) echo "unexpected git invocation: $*" >&2; exit 2 ;;
esac
""",
            encoding="utf-8",
        )
        fake_git.chmod(0o755)
        fake_gh = fake_bin / "gh"
        fake_gh.write_text(
            "#!/usr/bin/env bash\n"
            "printf 'unexpected\\n' >\"${FAKE_GH_MARKER}\"\n"
            "exit 99\n",
            encoding="utf-8",
        )
        fake_gh.chmod(0o755)
        gh_marker = root / "gh-called"
        environment.update(
            {
                "PATH": f"{fake_bin}:{environment['PATH']}",
                "FAKE_GIT_LOG": str(git_log),
                "FAKE_GH_MARKER": str(gh_marker),
            }
        )
        main_command = list(command)
        main_command[main_command.index("refs/heads/unprotected")] = (
            "refs/heads/main"
        )
        rejected_ancestry = subprocess.run(
            main_command,
            cwd=REPO_ROOT,
            env=environment,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert rejected_ancestry.returncode != 0
        assert "merge-base --is-ancestor" in git_log.read_text(encoding="utf-8")
        assert not gh_marker.exists()


def _dispatchers(root: Path) -> tuple[Path, dict[str, str]]:
    directory = root / "dispatchers"
    directory.mkdir()
    digests: dict[str, str] = {}
    for name in ("index", "stable", "alpha", "beta"):
        path = directory / name
        path.write_text(f"fixture dispatcher {name}\n", encoding="utf-8")
        digests[name] = sha256_file(path)
    return directory, digests


def _incident_smoke(root: Path, summary: dict[str, object]) -> Path:
    smoke = root / f"{summary['operation']}-smoke"
    smoke.mkdir()
    for filename in SMOKE_FILES.values():
        (smoke / filename).write_text(
            f"fixture evidence {filename}\n",
            encoding="utf-8",
        )
    mutation = summary["mutation"]
    assert isinstance(mutation, dict)
    (smoke / "governed-index.json").write_bytes(
        canonical_json_bytes(mutation["proposed_index"])
    )
    _replace_json(
        smoke / "incident-recovery.json",
        {
            "schema_version": 2,
            "operation": summary["operation"],
            "affected_version": summary["affected"]["version"],
            "successor_version": summary["successor"]["version"],
            "working_client": "pass",
            "out_of_band": "pass",
        },
    )
    return smoke


def _plan(root: Path, version: str) -> Path:
    return (
        root
        / "plans/releases/candidates"
        / version
        / "stable-release-plan.json"
    )


def _replace_json(path: Path, value: dict[str, object]) -> None:
    if path.exists():
        path.unlink()
    write_canonical_json(path, value, refuse_existing=True)


def _initialize_git(root: Path) -> None:
    root.mkdir()
    git(root, "init", "-q")
    git(root, "config", "user.email", "release-fixture@sifr.test")
    git(root, "config", "user.name", "Release Fixture")
    (root / "README.md").write_text("fixture\n", encoding="utf-8")
    git(root, "add", "README.md")
    git(root, "commit", "-qm", "base")


def _configure_roll_forward(
    context: dict[str, object],
) -> tuple[Path, Path, Path]:
    live_path = context["live_index_path"]
    assert isinstance(live_path, Path)
    live = load_json_strict(live_path, require_canonical=True)
    predecessor = copy.deepcopy(live["releases"]["0.1.0-beta.2"])
    predecessor["channel"] = "stable"
    predecessor["source_commit"] = "d" * 40
    live["ga_status"] = "active"
    live["channels"]["stable"] = "0.0.9"
    live["releases"]["0.0.9"] = predecessor
    _replace_json(live_path, live)
    snapshot_root = context["snapshot_root"]
    assert isinstance(snapshot_root, Path)
    _replace_json(snapshot_root / "channels-generation-7.json", live)

    evidence_root = context["evidence_root"]
    candidate_path = context["candidate_path"]
    assert isinstance(evidence_root, Path)
    assert isinstance(candidate_path, str)
    candidate_plan_path = evidence_root / candidate_path / PLAN_ASSET_NAME
    candidate_plan = load_json_strict(
        candidate_plan_path,
        require_canonical=True,
    )
    affected_plan = copy.deepcopy(candidate_plan)
    affected_plan["version"] = "0.0.9"
    affected_plan["source_commit"] = predecessor["source_commit"]
    affected_plan["plan_id"] = (
        f"stable-0.0.9-{predecessor['source_commit'][:12]}"
    )
    affected_plan["desired_release"] = copy.deepcopy(predecessor)
    affected_plan["installer_sha256"] = predecessor["installer_sha256"]
    for target in affected_plan["targets"]:
        record = predecessor["targets"][target["triple"]]
        target["archive_sha256"] = record["artifact_sha256"]
        target["sysroot_sha256"] = record["sysroot_content_sha256"]
        target["sifr_version"] = "0.0.9"
        target["installer_version"] = "0.0.9"
        target["sysroot_version"] = "0.0.9"
    affected_plan_path = context["root"] / "affected-plan.json"
    assert isinstance(affected_plan_path, Path)
    write_canonical_json(
        affected_plan_path,
        affected_plan,
        refuse_existing=True,
    )
    withdrawal = b"qualified roll-forward incident evidence\n"
    request = {
        "schema_version": 2,
        "incident_id": "inc-forward-qualified",
        "operation": "incident-roll-forward",
        "trigger": "post-GA regression",
        "affected_release": {
            "version": "0.0.9",
            "plan_sha256": sha256_file(affected_plan_path),
        },
        "withdrawal": {
            "reason": "qualified successor required",
            "evidence_sha256": sha256_bytes(withdrawal),
        },
    }
    root = context["root"]
    assert isinstance(root, Path)
    request_path = root / "stable-incident-request.json"
    request_path.write_bytes(canonical_json_bytes(request))
    withdrawal_path = root / "withdrawal-evidence.txt"
    withdrawal_path.write_bytes(withdrawal)
    candidate_plan["transition"] = "incident-roll-forward"
    candidate_plan["expected_stable_predecessor"] = {
        "version": "0.0.9",
        "status": "active",
        "plan_sha256": sha256_file(affected_plan_path),
    }
    candidate_plan["rollback_target"] = "none"
    candidate_plan["incident_request_sha256"] = sha256_file(request_path)
    _replace_json(candidate_plan_path, candidate_plan)
    git(evidence_root, "add", ".")
    git(evidence_root, "commit", "--amend", "--no-edit")
    context["evidence_commit"] = git(
        evidence_root,
        "rev-parse",
        "HEAD",
    ).strip()
    context["expected_plan_sha256"] = sha256_file(candidate_plan_path)
    context["operation"] = "incident-roll-forward"
    return affected_plan_path, request_path, withdrawal_path


def _expect_rejected(operation: Callable[[], object]) -> None:
    try:
        operation()
    except GovernanceError:
        return
    raise AssertionError("drifted incident publication unexpectedly passed")


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
