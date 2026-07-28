"""Mutation tests for stable GA-activation and normal index planning."""

from __future__ import annotations

import copy
import subprocess
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import Any

from .common import (
    GovernanceError,
    canonical_json_bytes,
    load_json_strict,
    sha256_bytes,
    sha256_file,
    write_canonical_json,
)
from .release_index import propose_stable_release
from .selftest import (
    SHA_A,
    active_index,
    preview_index,
    release_record,
    valid_plan,
)
from .stable_planner import (
    materialize_stable_mutation,
    validate_stable_mutation_evidence,
)

REPO_ROOT = Path(__file__).resolve().parents[4]
CLI = REPO_ROOT / "scripts" / "distribution" / "release_governance.py"


def run_self_tests() -> int:
    tests = (
        test_ga_activation,
        test_normal_successor,
        test_fail_closed_identity_and_transition,
        test_direct_transition_defenses,
        test_cli_producer,
    )
    for test in tests:
        test()
        print(f"stable-planner-self-test pass: {test.__name__}")
    print(f"stable planner self-tests ok: tests={len(tests)}")
    return 0


def test_ga_activation() -> None:
    with fixture_files(preview_index(), valid_plan()) as fixture:
        mutation = plan_fixture(fixture, proposed_generation=8)
        proposed = mutation.proposed_index
        assert mutation.transition == "ga-activation"
        assert mutation.version == "0.1.0"
        assert proposed["generation"] == 8
        assert proposed["ga_status"] == "active"
        assert proposed["channels"] == {
            "alpha": "0.1.0-alpha.2",
            "beta": "0.1.0-beta.2",
            "stable": "0.1.0",
        }
        assert proposed["releases"]["0.1.0"] == release_record("stable")
        assert_retained_bytes(mutation.previous_index, proposed)
        evidence = mutation.evidence()
        validate_stable_mutation_evidence(evidence)
        stale = copy.deepcopy(evidence)
        stale["proposed_index"]["generation"] = mutation.previous_index["generation"]
        stale["proposed_index_sha256"] = sha256_bytes(
            canonical_json_bytes(stale["proposed_index"])
        )
        expect_rejected(
            lambda: validate_stable_mutation_evidence(stale),
            "must follow the previous index",
        )


def test_normal_successor() -> None:
    live = active_index()
    live["channels"]["stable"] = "0.0.9"
    live["releases"].pop("0.1.0")
    live["releases"]["0.0.9"] = release_record("stable")
    with fixture_files(live, valid_plan(transition="normal")) as fixture:
        mutation = plan_fixture(fixture, proposed_generation=12)
        proposed = mutation.proposed_index
        assert mutation.transition == "normal"
        assert proposed["generation"] == 12
        assert proposed["channels"]["stable"] == "0.1.0"
        assert proposed["releases"]["0.0.9"] == live["releases"]["0.0.9"]
        assert proposed["releases"]["0.1.0"] == release_record("stable")
        assert_retained_bytes(live, proposed)
        validate_stable_mutation_evidence(mutation.evidence())


def test_fail_closed_identity_and_transition() -> None:
    with fixture_files(preview_index(), valid_plan()) as fixture:
        expect_rejected(
            lambda: materialize_stable_mutation(
                plan_path=fixture.plan,
                live_index_path=fixture.index,
                expected_generation=6,
                expected_sha256=sha256_file(fixture.index),
                proposed_generation=8,
            ),
            "expected_generation",
        )
        expect_rejected(
            lambda: materialize_stable_mutation(
                plan_path=fixture.plan,
                live_index_path=fixture.index,
                expected_generation=7,
                expected_sha256=SHA_A,
                proposed_generation=8,
            ),
            "expected_sha256",
        )
        expect_rejected(
            lambda: plan_fixture(fixture, proposed_generation=7),
            "proposed_generation",
        )

    with fixture_files(active_index(), valid_plan()) as fixture:
        expect_rejected(
            lambda: plan_fixture(fixture, proposed_generation=9),
            "ga-activation",
        )

    with fixture_files(
        preview_index(),
        valid_plan(transition="normal"),
    ) as fixture:
        expect_rejected(
            lambda: plan_fixture(fixture, proposed_generation=8),
            "active live stable predecessor",
        )

    incident = valid_plan(transition="incident-roll-forward")
    incident["expected_stable_predecessor"] = {
        "version": "0.1.0",
        "status": "active",
        "plan_sha256": "d" * 64,
    }
    with fixture_files(active_index(), incident) as fixture:
        expect_rejected(
            lambda: plan_fixture(fixture, proposed_generation=9),
            "stable publication accepts ga-activation or normal",
        )


def test_direct_transition_defenses() -> None:
    candidate = release_record("stable")
    expect_rejected(
        lambda: propose_stable_release(
            preview_index(),
            transition="incident-roll-forward",
            version="0.1.0",
            release_value=candidate,
            expected_predecessor=None,
            proposed_generation=8,
        ),
        "transition",
    )
    expect_rejected(
        lambda: propose_stable_release(
            preview_index(),
            transition="ga-activation",
            version="0.1.0",
            release_value=candidate,
            expected_predecessor="0.0.9",
            proposed_generation=8,
        ),
        "cannot name a predecessor",
    )
    expect_rejected(
        lambda: propose_stable_release(
            preview_index(),
            transition="ga-activation",
            version="0.1.0",
            release_value=candidate,
            expected_predecessor=None,
            proposed_generation=7,
        ),
        "proposed_generation",
    )
    expect_rejected(
        lambda: propose_stable_release(
            active_index(),
            transition="normal",
            version="0.2.0",
            release_value=candidate,
            expected_predecessor="0.0.9",
            proposed_generation=9,
        ),
        "does not equal the live stable version",
    )
    expect_rejected(
        lambda: propose_stable_release(
            active_index(),
            transition="normal",
            version="0.0.9",
            release_value=candidate,
            expected_predecessor="0.1.0",
            proposed_generation=9,
        ),
        "must move forward",
    )
    expect_rejected(
        lambda: propose_stable_release(
            active_index(),
            transition="normal",
            version="0.1.0",
            release_value=candidate,
            expected_predecessor="0.1.0",
            proposed_generation=9,
        ),
        "already exists",
    )


def test_cli_producer() -> None:
    with fixture_files(preview_index(), valid_plan()) as fixture:
        output = fixture.root / "proposed.json"
        command = [
            sys.executable,
            str(CLI),
            "plan-stable-index",
            "--plan",
            str(fixture.plan),
            "--live-index",
            str(fixture.index),
            "--expected-generation",
            "7",
            "--expected-sha256",
            sha256_file(fixture.index),
            "--proposed-generation",
            "8",
            "--out",
            str(output),
        ]
        subprocess.run(command, cwd=REPO_ROOT, check=True)
        evidence = validate_stable_mutation_evidence(
            load_json_strict(output, require_canonical=True),
        )
        proposed = evidence["proposed_index"]
        assert evidence["transition"] == "ga-activation"
        assert evidence["version"] == "0.1.0"
        assert evidence["plan_sha256"] == sha256_file(fixture.plan)
        assert evidence["previous_index"] == {
            "generation": 7,
            "sha256": sha256_file(fixture.index),
        }
        assert evidence["proposed_index_sha256"] == sha256_bytes(
            canonical_json_bytes(proposed)
        )
        assert proposed["channels"]["stable"] == "0.1.0"
        repeated = subprocess.run(
            command,
            cwd=REPO_ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        assert repeated.returncode == 2
        assert "refusing to overwrite" in repeated.stderr


class FixtureFiles:
    def __init__(self, root: Path, index: Path, plan: Path) -> None:
        self.root = root
        self.index = index
        self.plan = plan


class fixture_files:
    def __init__(self, index: dict[str, Any], plan: dict[str, Any]) -> None:
        self.index_value = index
        self.plan_value = plan
        self.temporary: tempfile.TemporaryDirectory[str] | None = None

    def __enter__(self) -> FixtureFiles:
        self.temporary = tempfile.TemporaryDirectory(prefix="sifr-stable-planner-")
        root = Path(self.temporary.name)
        index = root / "channels.json"
        plan = root / "stable-release-plan.json"
        write_canonical_json(index, self.index_value)
        write_canonical_json(plan, self.plan_value)
        return FixtureFiles(root, index, plan)

    def __exit__(self, *_: object) -> None:
        assert self.temporary is not None
        self.temporary.cleanup()


def plan_fixture(
    fixture: FixtureFiles,
    *,
    proposed_generation: int,
):
    current = load_json_strict(fixture.index, require_canonical=True)
    return materialize_stable_mutation(
        plan_path=fixture.plan,
        live_index_path=fixture.index,
        expected_generation=current["generation"],
        expected_sha256=sha256_file(fixture.index),
        proposed_generation=proposed_generation,
    )


def assert_retained_bytes(
    previous: dict[str, Any],
    proposed: dict[str, Any],
) -> None:
    for channel in ("alpha", "beta"):
        assert proposed["channels"][channel] == previous["channels"][channel]
    for version, release in previous["releases"].items():
        assert proposed["releases"][version] == release


def expect_rejected(action: Callable[[], Any], message: str) -> None:
    try:
        action()
    except GovernanceError as exc:
        assert message in str(exc), exc
    else:
        raise AssertionError(f"expected rejection containing {message!r}")


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
