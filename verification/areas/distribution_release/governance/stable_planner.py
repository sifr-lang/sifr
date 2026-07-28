"""Plan deterministic GA-activation and normal stable index mutations."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .common import (
    canonical_json_bytes,
    fail,
    load_json_bytes_strict,
    require_enum,
    require_exact_keys,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    sha256_bytes,
    version_channel,
)
from .release_index import (
    propose_stable_release,
    validate_release_index,
)
from .release_plan import validate_release_plan


@dataclass(frozen=True)
class StableMutation:
    """One approved stable release-index proposal."""

    transition: str
    version: str
    plan_sha256: str
    previous_index_sha256: str
    previous_index: dict[str, Any]
    proposed_index: dict[str, Any]

    def evidence(self) -> dict[str, Any]:
        """Return a canonicalizable plan-to-index binding."""
        return {
            "schema_version": 2,
            "transition": self.transition,
            "version": self.version,
            "plan_sha256": self.plan_sha256,
            "previous_index": {
                "generation": self.previous_index["generation"],
                "sha256": self.previous_index_sha256,
            },
            "proposed_index": self.proposed_index,
            "proposed_index_sha256": sha256_bytes(
                canonical_json_bytes(self.proposed_index)
            ),
        }


def materialize_stable_mutation(
    *,
    plan_path: Path,
    live_index_path: Path,
    expected_generation: int,
    expected_sha256: str,
    proposed_generation: int,
) -> StableMutation:
    """Validate exact inputs and return a stable index mutation without writing."""
    require_sha256(expected_sha256, "expected_sha256")
    try:
        live_bytes = live_index_path.read_bytes()
        plan_bytes = plan_path.read_bytes()
    except OSError as exc:
        fail("stable publication input", str(exc))
    current = validate_release_index(
        load_json_bytes_strict(
            live_bytes,
            source=str(live_index_path),
            require_canonical=True,
        )
    )
    if current["generation"] != expected_generation:
        fail("expected_generation", "does not equal the live release index")
    live_sha256 = sha256_bytes(live_bytes)
    if live_sha256 != expected_sha256:
        fail("expected_sha256", "does not equal the live release index")

    plan = validate_release_plan(
        load_json_bytes_strict(
            plan_bytes,
            source=str(plan_path),
            require_canonical=True,
        ),
        active_index=current,
    )
    transition = plan["transition"]
    if transition not in {"ga-activation", "normal"}:
        fail("$.transition", "stable publication accepts ga-activation or normal")
    predecessor = plan["expected_stable_predecessor"]
    expected_predecessor = (
        None if predecessor == "none" else predecessor["version"]
    )
    proposed = propose_stable_release(
        current,
        transition=transition,
        version=plan["version"],
        release_value=plan["desired_release"],
        expected_predecessor=expected_predecessor,
        proposed_generation=proposed_generation,
    )
    return StableMutation(
        transition=transition,
        version=plan["version"],
        plan_sha256=sha256_bytes(plan_bytes),
        previous_index_sha256=live_sha256,
        previous_index=current,
        proposed_index=proposed,
    )


def validate_stable_mutation_evidence(payload: object) -> dict[str, Any]:
    """Validate an emitted stable plan-to-index binding."""
    evidence = require_object(payload, "$")
    require_exact_keys(
        evidence,
        required={
            "schema_version",
            "transition",
            "version",
            "plan_sha256",
            "previous_index",
            "proposed_index",
            "proposed_index_sha256",
        },
        location="$",
    )
    require_schema_v2(evidence)
    require_enum(
        evidence["transition"],
        {"ga-activation", "normal"},
        "$.transition",
    )
    version = evidence["version"]
    if version_channel(version, "$.version") != "stable":
        fail("$.version", "must be an exact stable version")
    require_sha256(evidence["plan_sha256"], "$.plan_sha256")
    previous = require_object(evidence["previous_index"], "$.previous_index")
    require_exact_keys(
        previous,
        required={"generation", "sha256"},
        location="$.previous_index",
    )
    previous_generation = require_positive_int(
        previous["generation"],
        "$.previous_index.generation",
    )
    require_sha256(previous["sha256"], "$.previous_index.sha256")
    proposed = validate_release_index(evidence["proposed_index"])
    if proposed["generation"] <= previous_generation:
        fail("$.proposed_index.generation", "must follow the previous index")
    if proposed["ga_status"] != "active" or proposed["channels"].get("stable") != version:
        fail("$.proposed_index", "must activate the bound stable version")
    proposed_sha256 = require_sha256(
        evidence["proposed_index_sha256"],
        "$.proposed_index_sha256",
    )
    if proposed_sha256 != sha256_bytes(canonical_json_bytes(proposed)):
        fail("$.proposed_index_sha256", "does not match the proposed index bytes")
    return evidence
