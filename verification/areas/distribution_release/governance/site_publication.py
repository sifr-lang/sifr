"""Schema-v2 site publication binding validation."""

from __future__ import annotations

import re
from typing import Any

from .common import (
    fail,
    require_commit,
    require_enum,
    require_exact_keys,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
)

CONTRACT = "sifr-site-publication-binding-v2"
DISPATCHERS = ("index", "stable", "alpha", "beta")
PUBLICATION_ATTEMPT_RE = re.compile(r"^[A-Za-z0-9._-]{1,128}$")


def validate_site_publication_facts(value: Any) -> dict[str, Any]:
    facts = require_object(value, "$")
    require_exact_keys(
        facts,
        required={
            "schema_version",
            "contract",
            "publication_attempt",
            "source_commit",
            "site_base_commit",
            "release_plan_sha256",
            "release_index",
            "dispatcher_default_channel",
            "dispatchers",
        },
        location="$",
    )
    require_schema_v2(facts)
    if facts["contract"] != CONTRACT:
        fail("$.contract", f"must equal {CONTRACT}")
    publication_attempt = facts["publication_attempt"]
    if (
        not isinstance(publication_attempt, str)
        or PUBLICATION_ATTEMPT_RE.fullmatch(publication_attempt) is None
    ):
        fail(
            "$.publication_attempt",
            "must be 1-128 letters, digits, dot, underscore, or dash",
        )
    require_commit(facts["source_commit"], "$.source_commit")
    require_commit(facts["site_base_commit"], "$.site_base_commit")
    require_sha256(facts["release_plan_sha256"], "$.release_plan_sha256")

    release_index = require_object(facts["release_index"], "$.release_index")
    require_exact_keys(
        release_index,
        required={"generation", "sha256"},
        location="$.release_index",
    )
    require_positive_int(release_index["generation"], "$.release_index.generation")
    require_sha256(release_index["sha256"], "$.release_index.sha256")
    require_enum(
        facts["dispatcher_default_channel"],
        {"beta", "stable"},
        "$.dispatcher_default_channel",
    )

    dispatchers = require_object(facts["dispatchers"], "$.dispatchers")
    require_exact_keys(
        dispatchers,
        required=set(DISPATCHERS),
        location="$.dispatchers",
    )
    for dispatcher in DISPATCHERS:
        require_sha256(dispatchers[dispatcher], f"$.dispatchers.{dispatcher}")
    return facts
