"""Focused mutation tests for protected stable release sign-off."""

from __future__ import annotations

import copy
from collections.abc import Callable
from typing import Any

from .common import GovernanceError
from .release_plan import validate_release_signoff

SHA_A = "a" * 64
SHA_B = "b" * 64
SHA_C = "c" * 64
SHA_D = "d" * 64
COMMIT = "e" * 40


def test_release_signoff_mutations() -> None:
    signoff = valid_release_signoff()
    validate_release_signoff(signoff)
    waived = mutate(
        signoff,
        lambda item: (
            item.update(
                {
                    "approval_policy": {
                        "mode": "single-maintainer-waiver",
                        "waiver_sha256": SHA_A,
                    }
                }
            ),
            item["attempts"][0].update({"approver": "release-initiator"}),
        ),
    )
    validate_release_signoff(waived)
    expect_rejected(
        mutate(
            signoff,
            lambda item: item["attempts"][0].update(
                {"approver": "release-initiator"}
            ),
        )
    )
    expect_rejected(
        mutate(
            waived,
            lambda item: item["attempts"][0].update(
                {"approver": "release-reviewer"}
            ),
        )
    )
    expect_rejected(
        mutate(
            signoff,
            lambda item: item.update(
                {
                    "approval_policy": {
                        "mode": "distinct-reviewer",
                        "waiver_sha256": SHA_A,
                    }
                }
            ),
        )
    )
    expect_rejected(
        mutate(
            signoff,
            lambda item: item.update(
                {
                    "approval_policy": {
                        "mode": "single-maintainer-waiver",
                        "waiver_sha256": "none",
                    }
                }
            ),
        )
    )
    expect_rejected(
        mutate(signoff, lambda item: item.update({"version": "0.1.0-alpha.1"}))
    )
    for mutation in (
        lambda item: item["attempts"][0].update({"mutations": []}),
        lambda item: item["attempts"][0].update({"mode": []}),
        lambda item: item["attempts"][0].update({"status": []}),
    ):
        expect_rejected(mutate(signoff, mutation))


def valid_release_signoff() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "version": "0.1.0",
        "plan_sha256": SHA_A,
        "initiator": "release-initiator",
        "approval_policy": {
            "mode": "distinct-reviewer",
            "waiver_sha256": "none",
        },
        "attempts": [valid_attempt()],
        "published_assets": {"sifr-installer-0.1.0": SHA_A},
        "marketplace": {
            "publisher": "sifr",
            "extension": "sifr",
            "version": "0.1.0",
            "vsix_sha256": SHA_B,
        },
        "channel_generation": 8,
        "site_publication": {
            "repository": "sifr-lang/sifr-website",
            "workflow": "release-site.yml",
            "run_id": 11,
            "deployed_commit": COMMIT,
        },
        "site_facts_sha256": SHA_C,
        "post_publication_smoke": [
            {"id": f"smoke-{index}", "status": "pass", "sha256": SHA_D}
            for index in range(4)
        ],
    }


def valid_attempt() -> dict[str, Any]:
    return {
        "run_id": 10,
        "mode": "initial",
        "approver": "release-reviewer",
        "status": "completed",
        "mutations": [
            {
                "kind": "release-index",
                "identity": "generation-8",
                "sha256": SHA_A,
            }
        ],
    }


def mutate(
    payload: dict[str, Any],
    callback: Callable[[dict[str, Any]], object],
) -> dict[str, Any]:
    changed = copy.deepcopy(payload)
    callback(changed)
    return changed


def expect_rejected(payload: dict[str, Any]) -> None:
    try:
        validate_release_signoff(payload)
    except GovernanceError:
        return
    raise AssertionError("invalid release sign-off mutation unexpectedly passed")
