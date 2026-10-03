"""Conservative correctness checkpoint consumption with immutable attempts.

Every reusable recipe supplies an independently recomputed, complete dependency
key. Unknown recipes execute fresh. This never consumes performance receipts.
"""
from __future__ import annotations

import dataclasses
from datetime import UTC, datetime
import json
import os
from pathlib import Path
import stat
import shutil
import uuid
from typing import Callable

from .execution_evidence import build_evidence, validate_evidence, validate_records, write_evidence
from .execution_identity import EvidenceError, artifact_identity, validate_key
from .errors import VerificationError


@dataclasses.dataclass(frozen=True)
class Completed:
    records: list[dict]
    artifacts: list[Path]


@dataclasses.dataclass(frozen=True)
class Consumption:
    mode: str
    path: Path | None
    evidence: dict | None


class UnknownClosure(EvidenceError):
    """No checkpoint claim is possible; execute the assertion fresh."""


def safe_store(root: Path) -> Path:
    """Only this UID's private store may act as a local trusted producer."""
    root = root.absolute()
    for path in (root, *root.parents):
        if path.is_symlink():
            raise EvidenceError("checkpoint store has a symlink ancestor")
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    information = root.stat()
    if (information.st_uid != os.getuid() or information.st_mode & 0o022 or
            not stat.S_ISDIR(information.st_mode)):
        raise EvidenceError("checkpoint store is not privately owned")
    return root


class Checkpoints:
    def __init__(self, root: Path, *, producer: dict, max_age_seconds: int = 86400, max_retained_bytes: int = 16 * 1024**2):
        if not producer:
            raise EvidenceError("checkpoint consumer must independently identify a trusted producer")
        if type(max_age_seconds) is not int or max_age_seconds <= 0:
            raise EvidenceError("checkpoint lifetime must be a positive integer")
        if type(max_retained_bytes) is not int or max_retained_bytes <= 0:
            raise EvidenceError("checkpoint retained byte budget must be positive")
        self.max_retained_bytes = max_retained_bytes
        self.root = safe_store(root)
        self.producer = producer
        self.max_age_seconds = max_age_seconds

    def run(self, *, factory: Callable[[], dict] | None,
            callback: Callable[[], Completed], observe: Callable[[dict], None]) -> Consumption:
        if factory is None:
            observe({"state": "fresh", "reason": "dependency closure unknown; reuse disabled"})
            callback()
            return Consumption("fresh-uncheckpointed", None, None)
        try:
            expected = factory()
        except UnknownClosure as error:
            observe({"state": "fresh", "reason": str(error), "reuse": "disabled"})
            callback()
            return Consumption("fresh-uncheckpointed", None, None)
        validate_key(expected)
        if expected["inputs"]["producer"] != self.producer:
            raise EvidenceError("recipe identifies an untrusted checkpoint producer")
        kinds = expected["inputs"]["selection"]["required_kinds"]
        selected = sorted(kinds)
        bucket = safe_store(self.root / expected["input_digest"])
        for attempt in sorted(bucket.glob("*/evidence.json"), reverse=True):
            try:
                if attempt.is_symlink() or attempt.parent.is_symlink():
                    raise EvidenceError("checkpoint attempt is a symlink")
                for path in (attempt.parent, attempt):
                    info = path.stat()
                    if info.st_uid != os.getuid() or info.st_mode & 0o022:
                        raise EvidenceError("checkpoint attempt is not privately owned")
                payload = json.loads(attempt.read_text())
                validate_evidence(payload, expected_key=expected, require_complete=True,
                    expected_producer=self.producer, max_age_seconds=self.max_age_seconds)
                # Recompute independently again at consumption; no producer key
                # is accepted as an assertion of equivalent current inputs.
                if factory() != expected:
                    raise EvidenceError("inputs drifted during checkpoint consumption")
            except (VerificationError, OSError, ValueError, TypeError, KeyError) as error:
                observe({"state": "invalidated", "path": str(attempt), "reason": str(error)})
                continue
            observe({"state": "reused", "path": str(attempt),
                     "evidence_digest": payload["evidence_digest"],
                     "observed_commit": payload["key"]["observed_commit"],
                     "selected": selected, "executed_count": sum(row["executed_count"] for row in payload["records"])})
            return Consumption("reused", attempt, payload)
        directory = bucket / str(uuid.uuid4())
        directory.mkdir(mode=0o700)
        started = datetime.now(UTC).isoformat()
        observe({"state": "fresh", "selected": selected, "attempt": str(directory)})
        try:
            completed = callback()
            if factory() != expected:
                raise EvidenceError("inputs drifted during fresh checkpoint execution")
            originals = [(path, artifact_identity(path)) for path in completed.artifacts]
            if sum(info["size_bytes"] for _, info in originals) > self.max_retained_bytes:
                raise EvidenceError("checkpoint retained artifacts exceed declared byte budget")
            retained = []
            for index, (original, info) in enumerate(originals):
                copied = directory / f"artifact-{index:04d}-{info['sha256']}"
                with original.open("rb") as source, copied.open("xb") as destination:
                    shutil.copyfileobj(source, destination)
                    destination.flush()
                    os.fsync(destination.fileno())
                copied.chmod(0o400)
                current = artifact_identity(copied)
                if (artifact_identity(original) != info or current["sha256"] != info["sha256"]
                        or current["size_bytes"] != info["size_bytes"]):
                    raise EvidenceError("checkpoint output changed during retention")
                retained.append(copied)
            payload = build_evidence(key=expected, records=completed.records,
                selected=selected, complete=True, started_at=started,
                finished_at=datetime.now(UTC).isoformat(), retained_artifacts=retained)
        except BaseException as error:
            # Record the failed attempt, never overwrite it or fabricate an
            # executed count from a failed callback's missing observations.
            write_evidence(directory / "failure.json", {
                "schema_version": 1, "claim": "checkpoint-failure",
                "started_at": started, "finished_at": datetime.now(UTC).isoformat(),
                "expected_key": expected, "selected": selected, "complete": False,
                "detail": str(error), "exception_type": type(error).__name__})
            observe({"state": "failed", "attempt": str(directory), "reason": str(error)})
            raise
        path = directory / "evidence.json"
        write_evidence(path, payload)
        observe({"state": "checkpointed", "path": str(path),
                 "evidence_digest": payload["evidence_digest"]})
        return Consumption("executed", path, payload)


def reconcile(*, required: dict[str, str], receipts: list[tuple[dict, dict]],
              expected_producer: dict, max_age_seconds: int = 86400) -> list[dict]:
    """Require complete disjoint case coverage, using consumer-computed keys.

    Each receipt pairs the current independently computed key with evidence.
    No performance or source-equivalence claim is inferred from these records.
    """
    validate_records([], sorted(required), complete=False, required_kinds=required)
    records = {}
    for expected, payload in receipts:
        validate_evidence(payload, expected_key=expected, require_complete=True,
            expected_producer=expected_producer, max_age_seconds=max_age_seconds)
        for row in payload["records"]:
            if row["id"] not in required or row["id"] in records:
                raise EvidenceError("checkpoint aggregate has unselected or duplicate cases")
            if row["execution_kind"] != required[row["id"]]:
                raise EvidenceError("checkpoint aggregate execution kind differs")
            records[row["id"]] = row
    if set(records) != set(required):
        raise EvidenceError("checkpoint aggregate omits required inventory")
    return [records[identifier] for identifier in sorted(records)]
