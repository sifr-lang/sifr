"""Focused negative JSON Schema contracts for incident publication evidence."""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

from verification.json_schema_202012 import JsonSchemaError, validate_instance


def validate_incident_schema_negatives(
    fixtures: dict[str, Any],
    schema_root: Path,
) -> None:
    """Reject invalid incident prepare conditionals and mutation generations."""
    prepare_schema = schema_root / "incident_publication_prepare.schema.json"
    prepare = fixtures["incident_publication_prepare.schema.json"]
    activated_initial = copy.deepcopy(prepare)
    activated_initial["publication_state"] = "activated"
    rollback_with_release_prepare = copy.deepcopy(prepare)
    rollback_with_release_prepare["operation"] = "rollback"
    rollback_with_release_prepare["mutation"]["operation"] = "rollback"
    roll_forward_without_release_prepare = copy.deepcopy(prepare)
    roll_forward_without_release_prepare["release_prepare"] = "none"
    for invalid in (
        activated_initial,
        rollback_with_release_prepare,
        roll_forward_without_release_prepare,
    ):
        try:
            validate_instance(invalid, prepare_schema)
        except JsonSchemaError:
            pass
        else:
            raise ValueError(
                "incident prepare schema accepted an invalid conditional shape"
            )

    mutation_schema = schema_root / "incident_index_mutation_evidence.schema.json"
    invalid_generation = copy.deepcopy(
        fixtures["incident_index_mutation_evidence.schema.json"]
    )
    invalid_generation["previous_index"]["generation"] = 0
    try:
        validate_instance(invalid_generation, mutation_schema)
    except JsonSchemaError:
        pass
    else:
        raise ValueError(
            "incident mutation schema accepted a zero predecessor generation"
        )

    stable_prepare_schema = schema_root / "stable_publication_prepare.schema.json"
    roll_forward_prepare = prepare["release_prepare"]
    missing_incident = copy.deepcopy(roll_forward_prepare)
    del missing_incident["incident"]
    unexpected_incident = copy.deepcopy(
        fixtures["stable_publication_prepare.schema.json"]
    )
    unexpected_incident["incident"] = copy.deepcopy(
        roll_forward_prepare["incident"]
    )
    for invalid in (missing_incident, unexpected_incident):
        try:
            validate_instance(invalid, stable_prepare_schema)
        except JsonSchemaError:
            pass
        else:
            raise ValueError(
                "stable prepare schema accepted an invalid incident binding"
            )
