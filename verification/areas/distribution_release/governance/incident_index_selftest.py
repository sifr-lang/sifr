"""Mutation coverage for the atomic stable incident-index contract."""

from __future__ import annotations

from copy import deepcopy
from typing import Any, Callable

from .common import GovernanceError
from .release_index import validate_incident_index_mutation
from .selftest import active_index, release_record

INCIDENT_ID = "inc-index-001"


def test_incident_index_mutation_contract() -> None:
    previous = active_index()
    previous["releases"]["0.0.8"] = release_record("stable")
    previous["releases"]["0.0.9"] = release_record("stable")

    rollback = deepcopy(previous)
    rollback["generation"] = 9
    rollback["channels"]["stable"] = "0.0.9"
    withdraw(rollback, "0.1.0")
    validate(previous, rollback, operation="rollback", successor="0.0.9")

    invalid_rollbacks = (
        changed(rollback, lambda value: value["releases"].pop("0.1.0")),
        changed(rollback, lambda value: reactivate(value, "0.1.0")),
        changed(
            rollback,
            lambda value: value["releases"]["0.1.0"].update(
                {"installer_sha256": "f" * 64}
            ),
        ),
        changed(rollback, deactivate_successor),
        changed(
            rollback,
            lambda value: value["channels"].update({"stable": "0.0.8"}),
        ),
        changed(
            rollback,
            lambda value: value["releases"].update(
                {"0.0.7": release_record("stable")}
            ),
        ),
        changed(
            rollback,
            lambda value: value["releases"]["0.0.8"].update(
                {"installer_sha256": "f" * 64}
            ),
        ),
        changed(rollback, move_alpha_channel),
    )
    for candidate in invalid_rollbacks:
        assert_rejected(
            lambda candidate=candidate: validate(
                previous,
                candidate,
                operation="rollback",
                successor="0.0.9",
            )
        )

    forward = deepcopy(previous)
    forward["generation"] = 9
    forward["channels"]["stable"] = "0.1.1"
    withdraw(forward, "0.1.0")
    forward["releases"]["0.1.1"] = release_record("stable")
    validate(
        previous,
        forward,
        operation="incident-roll-forward",
        successor="0.1.1",
    )
    assert_rejected(
        lambda: validate(
            previous,
            rollback,
            operation="incident-roll-forward",
            successor="0.0.9",
        )
    )
    assert_rejected(
        lambda: validate(
            previous,
            forward,
            operation="rollback",
            successor="0.1.1",
        )
    )
    extra_forward = changed(
        forward,
        lambda value: value["releases"].update(
            {"0.1.2": release_record("stable")}
        ),
    )
    assert_rejected(
        lambda: validate(
            previous,
            extra_forward,
            operation="incident-roll-forward",
            successor="0.1.1",
        )
    )
    assert_rejected(
        lambda: validate(
            previous,
            rollback,
            operation="rollback",
            affected="0.0.9",
            successor="0.0.8",
        )
    )


def validate(
    previous: dict[str, Any],
    proposed: dict[str, Any],
    *,
    operation: str,
    successor: str,
    affected: str = "0.1.0",
) -> None:
    validate_incident_index_mutation(
        previous,
        proposed,
        operation=operation,
        incident_id=INCIDENT_ID,
        affected_version=affected,
        successor_version=successor,
    )


def withdraw(index: dict[str, Any], version: str) -> None:
    index["releases"][version]["status"] = "withdrawn"
    index["releases"][version]["incident_id"] = INCIDENT_ID


def reactivate(index: dict[str, Any], version: str) -> None:
    index["releases"][version]["status"] = "active"
    index["releases"][version].pop("incident_id")


def deactivate_successor(index: dict[str, Any]) -> None:
    withdraw(index, "0.0.9")
    index["channels"]["stable"] = "0.0.8"


def move_alpha_channel(index: dict[str, Any]) -> None:
    index["releases"]["0.1.0-alpha.3"] = release_record("alpha")
    index["channels"]["alpha"] = "0.1.0-alpha.3"


def changed(
    value: dict[str, Any],
    mutation: Callable[[dict[str, Any]], Any],
) -> dict[str, Any]:
    candidate = deepcopy(value)
    mutation(candidate)
    return candidate


def assert_rejected(callback: Callable[[], None]) -> None:
    try:
        callback()
    except GovernanceError:
        return
    raise AssertionError("invalid incident index mutation unexpectedly passed")
