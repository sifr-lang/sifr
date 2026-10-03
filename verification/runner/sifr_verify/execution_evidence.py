"""Strict correctness evidence; a checkpoint never manufactures a final pass.

Paired performance receipts remain owned by the performance protocol. This
module cannot merge their interrupted captures or grant performance acceptance.
"""
from __future__ import annotations

import json
import math
import os
import tempfile
from datetime import UTC, datetime
from pathlib import Path

from .execution_identity import EvidenceError, artifact_identity, digest, validate_key
from .schemas import load_schema, validate_data

STATES = {"selected", "validated", "compiled", "executed", "passed", "failed",
          "skipped", "blocked", "infrastructure-failure"}
INFRASTRUCTURE = {"none", "timeout", "cancelled", "oom", "enospc", "admission", "unavailable"}


def validate_records(records: list[dict], selected: list[str], *, complete: bool,
                     required_kinds: dict[str, str]) -> None:
    if not isinstance(selected, list) or not selected or any(not isinstance(x, str) or not x for x in selected):
        raise EvidenceError("required selection must have nonempty IDs")
    if len(selected) != len(set(selected)):
        raise EvidenceError("duplicate required selection IDs")
    if not isinstance(records, list):
        raise EvidenceError("execution records must be an array")
    if set(required_kinds) != set(selected) or any(
            kind not in {"runtime", "compile", "validation"} for kind in required_kinds.values()):
        raise EvidenceError("every selected claim requires an explicit execution kind")
    seen = set()
    for record in records:
        if not isinstance(record, dict):
            raise EvidenceError("execution record must be an object")
        identifier = record.get("id")
        if not isinstance(identifier, str) or identifier not in selected or identifier in seen:
            raise EvidenceError("duplicate or unselected execution evidence")
        seen.add(identifier)
        state = record.get("state")
        phases = record.get("phases")
        count = record.get("executed_count")
        duration = record.get("elapsed_seconds")
        infrastructure = record.get("infrastructure")
        if (state not in STATES or not isinstance(phases, list) or not phases or
                any(phase not in STATES for phase in phases) or phases[0] != "selected" or
                phases[-1] != state or len(phases) != len(set(phases))):
            raise EvidenceError("invalid execution state transition evidence")
        terminal = STATES - {"selected", "validated", "compiled", "executed"}
        rank = {"selected": 0, "validated": 1, "compiled": 2, "executed": 3}
        active = [rank[phase] for phase in phases if phase in rank]
        if any(phase in terminal for phase in phases[:-1]) or active != sorted(active):
            raise EvidenceError("failed or unordered phases cannot be reinterpreted as passing")
        if record.get("execution_kind") != required_kinds[identifier]:
            raise EvidenceError("compilation/manifest checks cannot satisfy required runtime execution")
        if infrastructure not in INFRASTRUCTURE:
            raise EvidenceError("unknown infrastructure classification")
        if not isinstance(count, int) or isinstance(count, bool) or count < 0:
            raise EvidenceError("execution count must be an observed nonnegative integer")
        if (not isinstance(duration, (int, float)) or isinstance(duration, bool) or
                not math.isfinite(duration) or duration < 0):
            raise EvidenceError("elapsed time must be finite and nonnegative")
        if state == "passed":
            if "executed" not in phases or count <= 0 or infrastructure != "none":
                raise EvidenceError("passing evidence requires actual execution without infrastructure failure")
        if complete and state != "passed":
            raise EvidenceError(f"required work did not pass: {identifier}: {state}")
    if complete and seen != set(selected):
        raise EvidenceError("complete evidence omits required work")


def build_evidence(*, key: dict, records: list[dict], selected: list[str],
                   complete: bool, started_at: str, finished_at: str,
                   retained_artifacts: list[Path]) -> dict:
    validate_key(key)
    validate_records(records, selected, complete=complete,
                     required_kinds=key["inputs"]["selection"]["required_kinds"])
    artifacts = [artifact_identity(path) for path in retained_artifacts]
    payload = {"schema_version": 1, "claim": "correctness", "key": key,
               "selected": selected, "records": records, "complete": complete,
               "started_at": started_at, "finished_at": finished_at,
               "retained_artifacts": artifacts}
    payload["evidence_digest"] = digest(payload)
    validate_evidence(payload, expected_key=key, require_complete=complete,
                      expected_producer=key["inputs"]["producer"])
    return payload


def validate_evidence(payload: dict, *, expected_key: dict, require_complete: bool,
                      expected_producer: dict, now: datetime | None = None,
                      max_age_seconds: int | None = None) -> None:
    validate_data(payload, load_schema("execution_evidence.schema.json"), source="execution evidence")
    validate_key(payload["key"])
    validate_key(expected_key)
    if not expected_producer or payload["key"]["inputs"].get("producer") != expected_producer:
        raise EvidenceError("untrusted evidence producer")
    if payload["claim"] != "correctness":
        raise EvidenceError("correctness checkpoints cannot qualify performance")
    if expected_key["inputs"].get("producer") != expected_producer:
        raise EvidenceError("expected key has an untrusted producer")
    if payload["key"] != expected_key:
        raise EvidenceError("source, selector, command, runtime, artifact, or resource inputs changed")
    content = {k: v for k, v in payload.items() if k != "evidence_digest"}
    if payload["evidence_digest"] != digest(content):
        raise EvidenceError("evidence digest differs")
    if require_complete and payload["complete"] is not True:
        raise EvidenceError("incomplete checkpoint cannot satisfy final acceptance")
    validate_records(payload["records"], payload["selected"], complete=payload["complete"],
                     required_kinds=expected_key["inputs"]["selection"]["required_kinds"])
    try:
        start = datetime.fromisoformat(payload["started_at"])
        end = datetime.fromisoformat(payload["finished_at"])
    except ValueError as error:
        raise EvidenceError("invalid evidence timestamps") from error
    if start.utcoffset() is None or end.utcoffset() is None or end < start:
        raise EvidenceError("evidence requires ordered timezone-aware timestamps")
    current = now or datetime.now(UTC)
    if end > current:
        raise EvidenceError("evidence completion is in the future")
    if max_age_seconds is not None:
        if isinstance(max_age_seconds, bool) or not isinstance(max_age_seconds, int) or max_age_seconds <= 0:
            raise EvidenceError("freshness duration must be a positive integer")
        if (current - end).total_seconds() > max_age_seconds:
            raise EvidenceError("evidence expired after completion")
    for recorded in [*payload["key"]["inputs"]["artifacts"], *payload["retained_artifacts"]]:
        try:
            current_artifact = artifact_identity(Path(recorded["requested_path"]))
        except (OSError, EvidenceError) as error:
            raise EvidenceError("retained evidence artifact is unavailable") from error
        if current_artifact != recorded:
            raise EvidenceError("retained evidence artifact drifted")


def write_evidence(path: Path, payload: dict) -> None:
    """Exclusive immutable publication; previous failures are never overwritten."""
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary_name = tempfile.mkstemp(prefix=".evidence-", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(fd, "w") as stream:
            json.dump(payload, stream, sort_keys=True, indent=2, allow_nan=False)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        # Link an immutable JSON file only, never mutable Cargo target artifacts.
        os.link(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)
