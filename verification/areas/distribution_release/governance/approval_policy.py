"""The sole live stable-release policy; retained waiver history is separate."""

from __future__ import annotations

from typing import Any

from .common import fail, require_array, require_object, require_positive_int, require_sha256

MAINTAINER = "yaseralnajjar"
REPOSITORY = "sifr-lang/sifr"
ENVIRONMENT = "stable-release"
SOLO_MAINTAINER = "solo-maintainer"
OPERATIONS = {
    "bootstrap-alpha", "bootstrap-index", "ga-activation", "normal",
    "rollback", "incident-roll-forward", "bootstrap-recovery",
}


def validate_environment(payload: Any) -> None:
    environment = require_object(payload, "environment configuration")
    if environment.get("name") != ENVIRONMENT:
        fail("environment configuration", "must name stable-release")
    if environment.get("can_admins_bypass") is not False:
        fail("environment configuration", "admin bypass must be disabled")
    rules = require_array(environment.get("protection_rules"), "protection rules")
    reviewers = [rule for rule in rules if isinstance(rule, dict)
                 and rule.get("type") == "required_reviewers"]
    if len(reviewers) != 1 or reviewers[0].get("prevent_self_review") is not False:
        fail("protection rules", "must allow designated maintainer self-review")
    users = require_array(reviewers[0].get("reviewers"), "required reviewers")
    if len(users) != 1:
        fail("required reviewers", "must designate only yaseralnajjar")
    user = require_object(users[0], "required reviewer")
    identity = require_object(user.get("reviewer"), "required reviewer identity")
    if (user.get("type") != "User" or identity.get("login") != MAINTAINER
            or identity.get("id") != 10493809):
        fail("required reviewers", "must designate only yaseralnajjar")


def resolve_live_approval(
    approvals: Any, *, configuration: Any, run: Any, repository: str,
    operation: str, initiator: str, run_id: int, run_attempt: int,
    evidence_sha256: str,
) -> dict[str, Any]:
    """Validate responses fetched from the exact current protected run.

    The protected job is admitted by GitHub for each attempt. Its immutable
    prepare summary binds the evidence shown before that approval. The CLI
    fetches approval history itself; callers cannot supply another run's file.
    """
    if repository != REPOSITORY or operation not in OPERATIONS:
        fail("publication approval", "unsupported repository or operation")
    require_positive_int(run_id, "run ID")
    require_positive_int(run_attempt, "run attempt")
    require_sha256(evidence_sha256, "release evidence SHA-256")
    validate_environment(configuration)
    metadata = require_object(run, "GitHub run")
    if metadata.get("id") != run_id or metadata.get("run_attempt") != run_attempt:
        fail("GitHub run", "run ID or attempt does not match publication")
    repo = require_object(metadata.get("repository"), "GitHub run repository")
    actor = require_object(metadata.get("triggering_actor"), "GitHub run initiator")
    if repo.get("full_name") != repository or actor.get("login") != initiator:
        fail("GitHub run", "repository or initiator does not match publication")
    if metadata.get("status") != "in_progress":
        fail("GitHub run", "new publication requires the active run")
    values = require_array(approvals, "GitHub approval history")
    for raw in values:
        review = require_object(raw, "GitHub approval")
        if review.get("state") != "approved":
            continue
        environments = require_array(review.get("environments"), "approval environments")
        if not any(isinstance(value, dict) and value.get("name") == ENVIRONMENT
                   for value in environments):
            continue
        user = require_object(review.get("user"), "GitHub approver")
        if user.get("login") == MAINTAINER and user.get("id") == 10493809:
            return {
                "approvers": [MAINTAINER],
                "approval_policy": {"mode": SOLO_MAINTAINER, "waiver_sha256": "none"},
            }
    fail("GitHub approval history", "requires explicit approval by yaseralnajjar")
