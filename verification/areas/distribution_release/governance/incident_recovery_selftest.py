"""End-to-end mutation tests for local-only stable incident recovery."""

from __future__ import annotations

import copy
import os
import subprocess
import tempfile
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

from .common import (
    GovernanceError,
    TARGETS,
    canonical_json_bytes,
    load_json_strict,
    sha256_bytes,
    sha256_file,
    write_canonical_json,
)
from .evidence_custody import validate_changed_path_set, validate_incident_directory
from .incident import validate_incident_signoff
from .incident_evidence import validate_incident_evidence_commit
from .incident_fixture import (
    FORBIDDEN_CREDENTIALS,
    check_release_submission_allowed,
    plan_fixture_recovery,
    run_incident_fixture,
)
from .incident_planner import materialize_incident_mutation
from .release_index import (
    propose_incident_roll_forward,
    propose_rollback,
    validate_release_index,
)
from .release_plan import validate_release_plan, validate_site_release_facts
from .selftest import release_record as base_release_record
from .selftest import valid_plan as base_plan

REPO_ROOT = Path(__file__).resolve().parents[4]


def run_self_tests() -> int:
    tests = (
        test_rollback_burns_generation_and_resumes,
        test_site_timeout_resumes_without_second_index_mutation,
        test_first_ga_incident_roll_forward,
        test_fail_closed_preconditions,
        test_concurrency_and_credential_boundaries,
        test_evidence_only_commit_validator,
        test_cli_surfaces,
        test_no_production_adapter_surface,
    )
    for test in tests:
        test()
        print(f"incident-recovery-self-test pass: {test.__name__}")
    print(f"incident recovery self-tests ok: tests={len(tests)}")
    return 0


def test_rollback_burns_generation_and_resumes() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-incident-rollback-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        assets_before = tree_digest(fixture.root / "release-assets")
        with scrub_credentials():
            failed = run_fixture(fixture, mode="initial", fail_at="after-reservation")
            assert failed == {"status": "failed", "failure": "after-reservation", "run_id": 1}
            assert read_index(fixture.root)["generation"] == 20
            assert (fixture.root / "governance-release" / "channels-generation-21.json").is_file()
            completed = run_fixture(fixture, mode="resume")
        assert completed["status"] == "completed"
        index = read_index(fixture.root)
        assert index["generation"] == 22
        assert index["channels"]["stable"] == "0.1.0"
        assert index["releases"]["0.1.1"]["status"] == "withdrawn"
        assert index["releases"]["0.1.1"]["incident_id"] == "inc-rollback-001"
        assert (fixture.root / "governance-release" / "channels-generation-21.json").is_file()
        assert (fixture.root / "governance-release" / "channels-generation-22.json").is_file()
        assert tree_digest(fixture.root / "release-assets") == assets_before
        assert_signoff_and_site(fixture, completed)
        with scrub_credentials():
            expect_rejected(
                lambda: run_fixture(fixture, mode="resume"),
                "already signed off",
            )
        fresh = plan_fixture_recovery(
            fixture_root=fixture.root,
            current_version=None,
            entrypoint="fresh-install",
            force=False,
        )
        assert fresh["target_version"] == "0.1.0"
        expect_rejected(
            lambda: plan_fixture_recovery(
                fixture_root=fixture.root,
                current_version="0.1.1",
                entrypoint="self-update",
                force=False,
            ),
            "sifr self update --channel stable --force",
        )
        working = plan_fixture_recovery(
            fixture_root=fixture.root,
            current_version="0.1.1",
            entrypoint="self-update",
            force=True,
        )
        expect_rejected(
            lambda: plan_fixture_recovery(
                fixture_root=fixture.root,
                current_version="0.1.1",
                entrypoint="out-of-band",
                force=False,
            ),
            "https://sifr.sh/install/stable",
        )
        broken = plan_fixture_recovery(
            fixture_root=fixture.root,
            current_version="0.1.1",
            entrypoint="out-of-band",
            force=True,
        )
        assert working["action"] == broken["action"] == "delegate-to-immutable-installer"
        for name, recovery in (("fresh", fresh), ("working", working), ("broken", broken)):
            state = fixture.root / "installations" / name / "version"
            execute_fixture_installer(recovery, state)
            assert state.read_text(encoding="utf-8") == "0.1.0\n"


def test_site_timeout_resumes_without_second_index_mutation() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-incident-timeout-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        with scrub_credentials():
            failed = run_fixture(fixture, mode="initial", fail_at="site-timeout")
            assert failed["status"] == "failed"
            timeout_evidence = next((fixture.root / "site" / "attempts").glob("*.txt"))
            timeout_text = timeout_evidence.read_text(encoding="utf-8")
            assert "deadline_minutes=20" in timeout_text
            assert "status=terminal-timeout" in timeout_text
            assert "cancellation=requested" in timeout_text
            realized_bytes = (fixture.root / "live" / "channels.json").read_bytes()
            realized = read_index(fixture.root)
            assert realized["generation"] == 21
            completed = run_fixture(fixture, mode="resume")
        assert (fixture.root / "live" / "channels.json").read_bytes() == realized_bytes
        assert not (fixture.root / "governance-release" / "channels-generation-22.json").exists()
        assert_signoff_and_site(fixture, completed)
        attempts = sorted((fixture.root / "state").glob("*/attempt-*.json"))
        assert [load_json_strict(path)["status"] for path in attempts] == ["failed", "completed"]
        assert len(list((fixture.root / "site" / "attempts").glob("*.txt"))) == 2


def test_first_ga_incident_roll_forward() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-incident-forward-") as temporary:
        fixture = build_roll_forward_fixture(Path(temporary))
        sole_stable = read_index(fixture.root)
        expect_rejected(
            lambda: propose_rollback(
                sole_stable,
                incident_id="inc-first-ga-rollback",
                affected_version="0.1.0",
                target_version="0.1.0",
                proposed_generation=21,
            ),
            "distinct retained active stable release",
        )
        write_range_fixture(
            fixture.root / "extension-metadata.json",
            ">=9.0.0,<10.0.0",
            ["0.1.0"],
        )
        with scrub_credentials():
            completed = run_fixture(fixture, mode="initial")
        index = read_index(fixture.root)
        assert index["generation"] == 21
        assert index["channels"]["stable"] == "0.1.1"
        assert index["releases"]["0.1.0"]["status"] == "withdrawn"
        assert index["releases"]["0.1.1"]["status"] == "active"
        assert_signoff_and_site(fixture, completed)


def test_fail_closed_preconditions() -> None:
    current = active_fixture_index(
        generation=20,
        stable_version="0.1.0",
        stable_records={"0.1.0": base_release_record("stable")},
    )
    expect_rejected(
        lambda: propose_incident_roll_forward(
            current,
            incident_id="inc-forward-old",
            affected_version="0.1.0",
            successor_version="0.0.9",
            successor_release=base_release_record("stable"),
            proposed_generation=21,
        ),
        "newer stable",
    )
    with tempfile.TemporaryDirectory(prefix="sifr-incident-negative-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        live = fixture.root / "live" / "channels.json"
        expect_rejected(
            lambda: materialize_incident_mutation(
                request_path=fixture.request,
                live_index_path=live,
                affected_plan_path=fixture.affected_plan,
                successor_plan_path=fixture.successor_plan,
                expected_generation=19,
                expected_sha256=sha256_file(live),
                proposed_generation=21,
            ),
            "expected_generation",
        )
        write_range_fixture(
            fixture.root / "extension-metadata.json",
            ">=0.1.1,<0.2.0",
            ["0.1.0", "0.1.1"],
        )
        with scrub_credentials():
            expect_rejected(lambda: run_fixture(fixture, mode="initial"), "excludes")
        assert read_index(fixture.root)["generation"] == 20
        assert not (fixture.root / "governance-release" / "channels-generation-21.json").exists()

    with tempfile.TemporaryDirectory(prefix="sifr-incident-ga-rollback-") as temporary:
        fixture = build_rollback_fixture(Path(temporary), affected_transition="ga-activation")
        with scrub_credentials():
            expect_rejected(lambda: run_fixture(fixture, mode="initial"), "normal release plan")
        assert read_index(fixture.root)["generation"] == 20

    with tempfile.TemporaryDirectory(prefix="sifr-incident-marketplace-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        write_range_fixture(
            fixture.root / "marketplace.json",
            ">=0.1.0,<0.2.0",
            ["0.1.1"],
        )
        with scrub_credentials():
            expect_rejected(lambda: run_fixture(fixture, mode="initial"), "successor version")
        assert read_index(fixture.root)["generation"] == 20

    with tempfile.TemporaryDirectory(prefix="sifr-incident-assets-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        installer = fixture.root / "release-assets" / "0.1.0" / "sifr-installer-0.1.0"
        installer.write_bytes(installer.read_bytes() + b"drift\n")
        with scrub_credentials():
            expect_rejected(lambda: run_fixture(fixture, mode="initial"), "digest")
        assert read_index(fixture.root)["generation"] == 20

    with tempfile.TemporaryDirectory(prefix="sifr-incident-site-marker-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        (fixture.root / "site" / ".non-deploying-fixture").unlink()
        with scrub_credentials():
            expect_rejected(lambda: run_fixture(fixture, mode="initial"), "non-deploying")
        assert read_index(fixture.root)["generation"] == 20

    with tempfile.TemporaryDirectory(prefix="sifr-incident-race-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        with scrub_credentials():
            failed = run_fixture(
                fixture,
                mode="initial",
                fail_at="race-before-index",
            )
        assert failed["failure"] == "stale-generation"
        raced = read_index(fixture.root)
        assert raced["generation"] == 22
        assert raced["channels"]["stable"] == "0.1.1"
        assert raced["releases"]["0.1.1"]["status"] == "active"
        assert (fixture.root / "governance-release" / "channels-generation-21.json").is_file()
        assert (fixture.root / "governance-release" / "channels-generation-22.json").is_file()

    with tempfile.TemporaryDirectory(prefix="sifr-incident-symlink-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        (fixture.root / "site" / "escape").symlink_to(Path(temporary).parent)
        with scrub_credentials():
            expect_rejected(lambda: run_fixture(fixture, mode="initial"), "symbolic links")
        assert read_index(fixture.root)["generation"] == 20


def test_concurrency_and_credential_boundaries() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-incident-boundary-") as temporary:
        fixture = build_rollback_fixture(Path(temporary))
        lock = fixture.root / "state" / "metadata-concurrency.lock"
        lock.parent.mkdir(parents=True, exist_ok=True)
        lock.write_text("rollback\n", encoding="utf-8")
        expect_rejected(
            lambda: check_release_submission_allowed(fixture.root, "stable"),
            "blocked",
        )
        lock.unlink()
        check_release_submission_allowed(fixture.root, "preview")

        original = os.environ.get("GH_TOKEN")
        os.environ["GH_TOKEN"] = "fixture-must-refuse"
        try:
            expect_rejected(
                lambda: run_fixture(fixture, mode="initial"),
                "production credential",
            )
        finally:
            if original is None:
                os.environ.pop("GH_TOKEN", None)
            else:
                os.environ["GH_TOKEN"] = original
        assert not list((fixture.root / "governance-release").glob("stable-incident-request-*"))


def test_evidence_only_commit_validator() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-incident-evidence-") as temporary:
        repository = Path(temporary)
        git(repository, "init", "-q")
        git(repository, "config", "user.email", "release-fixture@sifr.test")
        git(repository, "config", "user.name", "Release Fixture")
        (repository / "README.md").write_text("fixture\n", encoding="utf-8")
        git(repository, "add", "README.md")
        git(repository, "commit", "-qm", "base")
        base = git(repository, "rev-parse", "HEAD").strip()

        evidence = b"stable smoke regression evidence\n"
        request = {
            "schema_version": 2,
            "incident_id": "inc-evidence-001",
            "operation": "incident-roll-forward",
            "trigger": "stable smoke regression",
            "affected_release": {
                "version": "0.1.0",
                "plan_sha256": "a" * 64,
            },
            "withdrawal": {
                "reason": "confirmed regression",
                "evidence_sha256": sha256_bytes(evidence),
            },
        }
        incident_root = (
            repository / "plans" / "releases" / "incidents" / request["incident_id"]
        )
        incident_root.mkdir(parents=True)
        (incident_root / "stable-incident-request.json").write_bytes(canonical_json_bytes(request))
        (incident_root / "withdrawal-evidence.txt").write_bytes(evidence)
        git(repository, "add", "plans")
        git(repository, "commit", "-qm", "incident evidence")
        head = git(repository, "rev-parse", "HEAD").strip()
        request_relative = (
            "plans/releases/incidents/"
            f"{request['incident_id']}/stable-incident-request.json"
        )
        evidence_relative = (
            "plans/releases/incidents/"
            f"{request['incident_id']}/withdrawal-evidence.txt"
        )
        validate_changed_path_set({request_relative, evidence_relative})
        validate_incident_directory(incident_root)
        expect_rejected(
            lambda: validate_changed_path_set(
                {request_relative, evidence_relative, "crates/sifr/src/main.rs"}
            ),
            "cannot mix with source changes",
        )
        validated = validate_incident_evidence_commit(
            repository=repository,
            base=base,
            head=head,
            request_path=request_relative,
            evidence_path=evidence_relative,
        )
        assert validated == request

        (repository / "source.py").write_text("unexpected = True\n", encoding="utf-8")
        git(repository, "add", "source.py")
        git(repository, "commit", "-qm", "unrelated source")
        bad_head = git(repository, "rev-parse", "HEAD").strip()
        expect_rejected(
            lambda: validate_incident_evidence_commit(
                repository=repository,
                base=base,
                head=bad_head,
                request_path=request_relative,
                evidence_path=evidence_relative,
            ),
            "exactly request",
        )


def test_no_production_adapter_surface() -> None:
    root = REPO_ROOT
    harness = (Path(__file__).with_name("incident_fixture.py")).read_text(encoding="utf-8")
    for forbidden in ("import socket", "import urllib", "import requests", "import subprocess"):
        assert forbidden not in harness
    workflow = (root / ".github" / "workflows" / "release-publication.yml").read_text(
        encoding="utf-8"
    )
    dispatch = workflow.split("jobs:", 1)[0]
    assert "rollback" not in dispatch
    assert "incident-roll-forward" not in dispatch
    assert "stable-release-drill" not in workflow
    script = (root / "scripts" / "distribution" / "run_incident_fixture.py").read_text(
        encoding="utf-8"
    )
    assert "gh release" not in script
    assert "vsce publish" not in script
    assert "repository_dispatch" not in script


def test_cli_surfaces() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-incident-cli-") as temporary:
        fixture = build_rollback_fixture(Path(temporary) / "fixture")
        live = fixture.root / "live" / "channels.json"
        planned = fixture.root / "planned-index.json"
        run_cli(
            REPO_ROOT / "scripts" / "distribution" / "release_governance.py",
            "plan-incident-index",
            "--request",
            str(fixture.request),
            "--live-index",
            str(live),
            "--affected-plan",
            str(fixture.affected_plan),
            "--successor-plan",
            str(fixture.successor_plan),
            "--expected-generation",
            "20",
            "--expected-sha256",
            sha256_file(live),
            "--proposed-generation",
            "21",
            "--out",
            str(planned),
        )
        proposed = validate_release_index(
            load_json_strict(planned, require_canonical=True)
        )
        assert proposed["generation"] == 21
        assert proposed["channels"]["stable"] == "0.1.0"

        work = Path(temporary) / "request-work"
        work.mkdir()
        spec = work / "request-spec.json"
        evidence = work / "withdrawal-evidence.txt"
        spec.write_bytes(fixture.request.read_bytes())
        evidence.write_bytes(
            (fixture.root / "evidence" / "withdrawal-evidence.txt").read_bytes()
        )
        generated = work / "stable-incident-request.json"
        run_cli(
            REPO_ROOT / "scripts" / "distribution" / "release_governance.py",
            "generate-incident-request",
            "--spec",
            str(spec),
            "--out",
            str(generated),
            "--live-index",
            str(live),
            "--withdrawal-evidence",
            str(evidence),
            "--affected-plan",
            str(fixture.affected_plan),
            "--rollback-plan",
            str(fixture.successor_plan),
        )
        assert generated.read_bytes() == fixture.request.read_bytes()

        with scrub_credentials():
            result = run_cli(
                REPO_ROOT / "scripts" / "distribution" / "run_incident_fixture.py",
                "run",
                "--fixture-root",
                str(fixture.root),
                "--live-index",
                str(fixture.root / "live" / "channels.json"),
                "--governance-release",
                str(fixture.root / "governance-release"),
                "--release-assets",
                str(fixture.root / "release-assets"),
                "--marketplace-stub",
                str(fixture.root / "marketplace.json"),
                "--extension-metadata",
                str(fixture.root / "extension-metadata.json"),
                "--site-repo",
                str(fixture.root / "site"),
                "--request",
                str(fixture.request),
                "--affected-plan",
                str(fixture.affected_plan),
                "--successor-plan",
                str(fixture.successor_plan),
                "--mode",
                "initial",
                "--approver",
                "fixture-cli-reviewer",
            )
        assert '"status":"completed"' in result


class Fixture:
    def __init__(
        self,
        root: Path,
        request: Path,
        affected_plan: Path,
        successor_plan: Path,
    ) -> None:
        self.root = root
        self.request = request
        self.affected_plan = affected_plan
        self.successor_plan = successor_plan


def build_rollback_fixture(
    root: Path,
    *,
    affected_transition: str = "normal",
) -> Fixture:
    root.mkdir(parents=True, exist_ok=True)
    target_record = make_release(root, "0.1.0", "d")
    affected_record = make_release(root, "0.1.1", "e")
    target_plan = make_plan("0.1.0", target_record, "ga-activation")
    target_path = root / "evidence" / "target-plan.json"
    write_json(target_path, target_plan)
    target_digest = sha256_file(target_path)
    affected_plan = make_plan(
        "0.1.1",
        affected_record,
        affected_transition,
        predecessor=(
            ("0.1.0", target_digest)
            if affected_transition == "normal"
            else None
        ),
    )
    affected_path = root / "evidence" / "affected-plan.json"
    write_json(affected_path, affected_plan)
    affected_digest = sha256_file(affected_path)
    withdrawal = b"rollback regression evidence\n"
    request = {
        "schema_version": 2,
        "incident_id": "inc-rollback-001",
        "operation": "rollback",
        "trigger": "stable target smoke regression",
        "affected_release": {
            "version": "0.1.1",
            "plan_sha256": affected_digest,
        },
        "withdrawal": {
            "reason": "confirmed stable regression",
            "evidence_sha256": sha256_bytes(withdrawal),
        },
        "rollback_target": {
            "version": "0.1.0",
            "plan_sha256": target_digest,
        },
    }
    request_path = root / "evidence" / "stable-incident-request.json"
    write_json(request_path, request)
    (root / "evidence" / "withdrawal-evidence.txt").write_bytes(withdrawal)
    index = active_fixture_index(
        generation=20,
        stable_version="0.1.1",
        stable_records={"0.1.0": target_record, "0.1.1": affected_record},
    )
    initialize_fixture_surface(root, index, published=["0.1.0", "0.1.1"])
    return Fixture(root, request_path, affected_path, target_path)


def build_roll_forward_fixture(root: Path) -> Fixture:
    root.mkdir(parents=True, exist_ok=True)
    affected_record = make_release(root, "0.1.0", "d")
    successor_record = make_release(root, "0.1.1", "e")
    affected_plan = make_plan("0.1.0", affected_record, "ga-activation")
    affected_path = root / "evidence" / "affected-plan.json"
    write_json(affected_path, affected_plan)
    affected_digest = sha256_file(affected_path)
    withdrawal = b"first GA incident evidence\n"
    request = {
        "schema_version": 2,
        "incident_id": "inc-forward-001",
        "operation": "incident-roll-forward",
        "trigger": "first GA stable regression",
        "affected_release": {
            "version": "0.1.0",
            "plan_sha256": affected_digest,
        },
        "withdrawal": {
            "reason": "first GA must roll forward",
            "evidence_sha256": sha256_bytes(withdrawal),
        },
    }
    request_path = root / "evidence" / "stable-incident-request.json"
    write_json(request_path, request)
    request_digest = sha256_file(request_path)
    successor_plan = make_plan(
        "0.1.1",
        successor_record,
        "incident-roll-forward",
        predecessor=("0.1.0", affected_digest),
        incident_request_sha256=request_digest,
    )
    successor_path = root / "evidence" / "successor-plan.json"
    write_json(successor_path, successor_plan)
    (root / "evidence" / "withdrawal-evidence.txt").write_bytes(withdrawal)
    index = active_fixture_index(
        generation=20,
        stable_version="0.1.0",
        stable_records={"0.1.0": affected_record},
    )
    initialize_fixture_surface(root, index, published=["0.1.0", "0.1.1"])
    return Fixture(root, request_path, affected_path, successor_path)


def make_release(root: Path, version: str, commit_character: str) -> dict[str, Any]:
    installer = (
        b"#!/bin/sh\nset -eu\n"
        + b': "${SIFR_FIXTURE_INSTALL_STATE:?missing fixture state path}"\n'
        + f"mkdir -p \"$(dirname \"${{SIFR_FIXTURE_INSTALL_STATE}}\")\"\n".encode()
        + f"printf '%s\\n' '{version}' >\"${{SIFR_FIXTURE_INSTALL_STATE}}\"\n".encode()
        + f"# immutable fixture installer for {version}\n".encode()
        + b"# padding-000000000000000000000000000000000000000000000000000000000000\n"
    )
    installer_path = root / "release-assets" / version / f"sifr-installer-{version}"
    installer_path.parent.mkdir(parents=True, exist_ok=True)
    installer_path.write_bytes(installer)
    installer_path.chmod(0o755)
    record = base_release_record("stable")
    record["source_commit"] = commit_character * 40
    record["installer_sha256"] = sha256_bytes(installer)
    return record


def make_plan(
    version: str,
    release: dict[str, Any],
    transition: str,
    *,
    predecessor: tuple[str, str] | None = None,
    incident_request_sha256: str | None = None,
) -> dict[str, Any]:
    plan = copy.deepcopy(base_plan())
    plan["version"] = version
    plan["transition"] = transition
    plan["source_commit"] = release["source_commit"]
    plan["plan_id"] = f"stable-{version}-{release['source_commit'][:12]}"
    plan["desired_release"] = copy.deepcopy(release)
    plan["installer_sha256"] = release["installer_sha256"]
    plan["vscode"]["version"] = version
    for row in plan["targets"]:
        row["sifr_version"] = version
        row["installer_version"] = version
        row["sysroot_version"] = version
    if predecessor is None:
        plan["expected_stable_predecessor"] = "none"
        plan["rollback_target"] = "none"
    else:
        predecessor_value = {
            "version": predecessor[0],
            "status": "active",
            "plan_sha256": predecessor[1],
        }
        plan["expected_stable_predecessor"] = predecessor_value
        plan["rollback_target"] = {
            "version": predecessor[0],
            "plan_sha256": predecessor[1],
        }
    if transition == "incident-roll-forward":
        assert incident_request_sha256 is not None
        plan["rollback_target"] = "none"
        plan["incident_request_sha256"] = incident_request_sha256
    validate_release_plan(plan)
    return plan


def active_fixture_index(
    *,
    generation: int,
    stable_version: str,
    stable_records: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    alpha = base_release_record("alpha")
    beta = base_release_record("beta")
    index = {
        "schema_version": 2,
        "generation": generation,
        "ga_status": "active",
        "channels": {
            "alpha": "0.1.0-alpha.2",
            "beta": "0.1.0-beta.2",
            "stable": stable_version,
        },
        "releases": {
            "0.1.0-alpha.2": alpha,
            "0.1.0-beta.2": beta,
            **stable_records,
        },
    }
    validate_release_index(index)
    return index


def initialize_fixture_surface(
    root: Path,
    index: dict[str, Any],
    *,
    published: list[str],
) -> None:
    write_json(root / "live" / "channels.json", index)
    write_json(
        root / "governance-release" / f"channels-generation-{index['generation']}.json",
        index,
    )
    write_range_fixture(
        root / "extension-metadata.json",
        ">=0.1.0,<0.2.0",
        published,
    )
    write_range_fixture(root / "marketplace.json", ">=0.1.0,<0.2.0", published)
    install_root = root / "site" / "install"
    install_root.mkdir(parents=True, exist_ok=True)
    (root / "site" / ".non-deploying-fixture").write_text(
        "local-only\n",
        encoding="utf-8",
    )
    for name in ("index", "stable", "alpha", "beta"):
        (install_root / name).write_text(
            f"#!/bin/sh\n# local non-deploying {name} dispatcher\n",
            encoding="utf-8",
        )


def write_range_fixture(path: Path, expression: str, versions: list[str]) -> None:
    write_json(
        path,
        {
            "schema_version": 2,
            "compiler_compatibility": expression,
            "published_versions": versions,
        },
        replace=True,
    )


def run_fixture(
    fixture: Fixture,
    *,
    mode: str,
    fail_at: str = "none",
) -> dict[str, Any]:
    return run_incident_fixture(
        fixture_root=fixture.root,
        live_index_path=fixture.root / "live" / "channels.json",
        governance_root=fixture.root / "governance-release",
        release_assets_root=fixture.root / "release-assets",
        marketplace_path=fixture.root / "marketplace.json",
        extension_metadata_path=fixture.root / "extension-metadata.json",
        site_root=fixture.root / "site",
        request_path=fixture.request,
        affected_plan_path=fixture.affected_plan,
        successor_plan_path=fixture.successor_plan,
        mode=mode,
        approver="fixture-release-reviewer",
        fail_at=fail_at,
    )


def assert_signoff_and_site(fixture: Fixture, result: dict[str, Any]) -> None:
    signoff = validate_incident_signoff(
        load_json_strict(Path(result["signoff"]), require_canonical=True),
        incident_request=load_json_strict(
            fixture.request,
            require_canonical=True,
        ),
    )
    index = read_index(fixture.root)
    facts_paths = sorted(
        (fixture.root / "governance-release").glob("site-release-facts-generation-*.json")
    )
    assert len(facts_paths) == 1
    facts = validate_site_release_facts(
        load_json_strict(facts_paths[0], require_canonical=True),
        governed_index=index,
    )
    assert facts["stable_version"] == index["channels"]["stable"]
    assert facts["withdrawals"]
    assert signoff["index_mutation"]["realized_generation"] == index["generation"]


def read_index(root: Path) -> dict[str, Any]:
    return validate_release_index(
        load_json_strict(root / "live" / "channels.json", require_canonical=True)
    )


def write_json(path: Path, value: dict[str, Any], *, replace: bool = False) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if replace:
        path.unlink(missing_ok=True)
    write_canonical_json(path, value, refuse_existing=True)


def tree_digest(root: Path) -> str:
    payload = b"".join(
        path.relative_to(root).as_posix().encode()
        + b"\0"
        + path.read_bytes()
        + b"\0"
        for path in sorted(item for item in root.rglob("*") if item.is_file())
    )
    return sha256_bytes(payload)


def execute_fixture_installer(recovery: dict[str, Any], state: Path) -> None:
    environment = os.environ.copy()
    environment["SIFR_FIXTURE_INSTALL_STATE"] = str(state)
    subprocess.run(
        [recovery["installer"]],
        check=True,
        env=environment,
        stdout=subprocess.DEVNULL,
    )


def git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *args],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return result.stdout


def run_cli(script: Path, *args: str) -> str:
    result = subprocess.run(
        [str(script), *args],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return result.stdout


def expect_rejected(callback: Any, contains: str) -> None:
    try:
        callback()
    except GovernanceError as exc:
        if contains not in str(exc):
            raise AssertionError(f"expected {contains!r} in {exc!r}") from exc
        return
    raise AssertionError("invalid incident operation unexpectedly passed")


@contextmanager
def scrub_credentials() -> Iterator[None]:
    saved = {name: os.environ.pop(name) for name in FORBIDDEN_CREDENTIALS if name in os.environ}
    try:
        yield
    finally:
        os.environ.update(saved)


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
