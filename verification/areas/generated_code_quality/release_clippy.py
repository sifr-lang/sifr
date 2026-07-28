#!/usr/bin/env python3
"""Release-only generated-code Clippy gate with exact, expiry-bound divergences."""

from __future__ import annotations

import argparse
import dataclasses
import json
import os
import re
import shutil
import sys
from datetime import date
from pathlib import Path
from typing import Any

import generated_code_quality as gcq

RELEASE_DIVERGENCES = gcq.GCQ_ROOT / "data" / "release_divergences.json"
SURFACE_MATRIX = (
    gcq.REPO_ROOT
    / "verification"
    / "areas"
    / "coverage_matrix"
    / "compiler_surface_matrix.json"
)
RELEASE_SUITE = "generated_code_quality:release-full"


@dataclasses.dataclass(frozen=True)
class ReleaseClippyDivergence:
    record_id: str
    entry_ids: tuple[str, ...]
    expires_on: date
    expected_lints: tuple[str, ...]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(gcq.MANIFEST))
    parser.add_argument("--group", action="append", default=[])
    parser.add_argument(
        "--keep-success",
        action="store_true",
        default=os.environ.get("SIFR_GCQ_KEEP_SUCCESS") == "1",
    )
    return parser.parse_args()


def load_divergences(entries: list[gcq.Entry]) -> list[ReleaseClippyDivergence]:
    payload = json.loads(RELEASE_DIVERGENCES.read_text(encoding="utf-8"))
    if (
        set(payload) != {"schema_version", "records"}
        or payload.get("schema_version") != 1
    ):
        raise RuntimeError("invalid generated-code release divergence document header")
    raw_records = payload.get("records")
    if not isinstance(raw_records, list) or not raw_records:
        raise RuntimeError(
            "generated-code release divergences must define non-empty records"
        )

    entries_by_id = {entry.id: entry for entry in entries}
    divergences: list[ReleaseClippyDivergence] = []
    seen_records: set[str] = set()
    seen_entries: set[str] = set()
    for index, raw in enumerate(raw_records):
        location = f"release divergences records[{index}]"
        if not isinstance(raw, dict) or set(raw) != {
            "record_id",
            "gate",
            "entry_ids",
            "expires_on",
            "expected_lints",
        }:
            raise RuntimeError(f"{location}: invalid fields")
        record_id = raw.get("record_id")
        gate = raw.get("gate")
        entry_ids_raw = raw.get("entry_ids")
        expiry_raw = raw.get("expires_on")
        expected_lints_raw = raw.get("expected_lints")
        if not isinstance(record_id, str) or not record_id:
            raise RuntimeError(f"{location}: invalid record_id")
        if record_id in seen_records:
            raise RuntimeError(f"{location}: duplicate record_id {record_id}")
        if gate != "clippy":
            raise RuntimeError(
                f"{location}: only the clippy gate supports release divergence"
            )
        if (
            not isinstance(entry_ids_raw, list)
            or not entry_ids_raw
            or not all(isinstance(entry_id, str) for entry_id in entry_ids_raw)
            or entry_ids_raw != sorted(entry_ids_raw)
            or len(entry_ids_raw) != len(set(entry_ids_raw))
        ):
            raise RuntimeError(f"{location}: invalid entry_ids")
        unknown = [
            entry_id for entry_id in entry_ids_raw if entry_id not in entries_by_id
        ]
        if unknown:
            raise RuntimeError(f"{location}: unknown entry_ids {unknown}")
        duplicates = sorted(set(entry_ids_raw).intersection(seen_entries))
        if duplicates:
            raise RuntimeError(f"{location}: duplicate entry_ids {duplicates}")
        non_positive = [
            entry_id
            for entry_id in entry_ids_raw
            if entries_by_id[entry_id].group not in gcq.POSITIVE_GROUPS
        ]
        if non_positive:
            raise RuntimeError(
                f"{location}: divergence entries must be positive: {non_positive}"
            )
        expires_on = parse_expiry(expiry_raw, location)
        expected_lints = parse_expected_lints(expected_lints_raw, location)
        divergences.append(
            ReleaseClippyDivergence(
                record_id=record_id,
                entry_ids=tuple(entry_ids_raw),
                expires_on=expires_on,
                expected_lints=expected_lints,
            )
        )
        seen_records.add(record_id)
        seen_entries.update(entry_ids_raw)
    validate_matrix_binding(divergences)
    return divergences


def validate_matrix_binding(divergences: list[ReleaseClippyDivergence]) -> None:
    payload = json.loads(SURFACE_MATRIX.read_text(encoding="utf-8"))
    rows = payload.get("rows")
    if not isinstance(rows, list):
        raise RuntimeError("compiler surface matrix is missing rows")
    bound = [
        row
        for row in rows
        if isinstance(row, dict)
        and RELEASE_SUITE
        in {
            token.strip()
            for token in str(row.get("release_suite", "")).split(",")
            if token.strip()
        }
    ]
    if not bound:
        raise RuntimeError(f"compiler surface matrix does not bind {RELEASE_SUITE}")
    expected = {
        (
            divergence.record_id,
            divergence.expires_on.isoformat(),
            divergence.entry_ids,
        )
        for divergence in divergences
    }
    observed = {
        (
            row.get("release_divergence_record"),
            row.get("release_divergence_expiry"),
            tuple(row.get("release_divergence_entries", [])),
        )
        for row in bound
    }
    if observed != expected:
        raise RuntimeError(
            f"generated-code release divergence matrix mismatch: "
            f"expected={sorted(expected)}, observed={sorted(observed)}"
        )


def parse_expiry(raw: Any, location: str) -> date:
    if not isinstance(raw, str):
        raise RuntimeError(f"{location}: invalid expires_on")
    try:
        expiry = date.fromisoformat(raw)
    except ValueError as error:
        raise RuntimeError(f"{location}: expires_on must be YYYY-MM-DD") from error
    if expiry < date.today():
        raise RuntimeError(
            f"{location}: release divergence expired on {expiry.isoformat()}"
        )
    return expiry


def parse_expected_lints(raw: Any, location: str) -> tuple[str, ...]:
    if (
        not isinstance(raw, list)
        or not raw
        or not all(
            isinstance(lint, str) and re.fullmatch(r"clippy::[a-z0-9_]+", lint)
            for lint in raw
        )
        or len(raw) != len(set(raw))
    ):
        raise RuntimeError(f"{location}: invalid expected_lints")
    return tuple(raw)


def clippy_command(crate_root: Path) -> list[str]:
    return [
        "cargo",
        "clippy",
        "--manifest-path",
        str(crate_root / "Cargo.toml"),
        "--",
        *gcq.GENERATED_CLIPPY_ARGS,
    ]


def extract_clippy_lints(output: str) -> tuple[str, ...]:
    return tuple(
        sorted(
            set(
                re.findall(
                    r"clippy::[a-z0-9_]+(?![a-z0-9_-])",
                    output,
                )
            )
        )
    )


def check_entry(
    entry: gcq.Entry,
    divergence: ReleaseClippyDivergence | None,
    run_root: Path,
) -> tuple[Path, tuple[str, ...] | None]:
    crate_root = gcq.materialize_entry(entry, run_root)
    gcq.run_command(["cargo", "fmt", "--manifest-path", str(crate_root / "Cargo.toml")])
    if divergence is None:
        gcq.run_command(clippy_command(crate_root))
        return crate_root, None
    result = gcq.run_command(clippy_command(crate_root), check=False)
    if result.returncode == 0:
        raise RuntimeError(
            f"{entry.id}: stale {divergence.record_id} release divergence unexpectedly passed"
        )
    observed_lints = extract_clippy_lints(result.stdout + result.stderr)
    if observed_lints != tuple(sorted(divergence.expected_lints)):
        raise RuntimeError(
            f"{entry.id}: {divergence.record_id} expected lints "
            f"{sorted(divergence.expected_lints)}, observed {list(observed_lints)}"
        )
    return crate_root, observed_lints


def require_all_divergences_exercised(
    divergences_by_entry: dict[str, ReleaseClippyDivergence],
    exercised_ids: set[str],
) -> None:
    missing = sorted(set(divergences_by_entry).difference(exercised_ids))
    if missing:
        raise RuntimeError(
            f"release Clippy did not exercise governed entries: {missing}"
        )


def run(entries: list[gcq.Entry], args: argparse.Namespace) -> None:
    run_id = gcq.run_id("clippy-release")
    run_root = gcq.TARGET_ROOT / run_id
    records: list[dict[str, Any]] = []
    divergences = load_divergences(entries)
    divergences_by_entry = {
        entry_id: divergence
        for divergence in divergences
        for entry_id in divergence.entry_ids
    }
    exercised_ids: set[str] = set()
    failures: list[str] = []
    try:
        gcq.timed_case(
            "generated_code_quality",
            "clippy-release/negative-clippy-warning",
            lambda: gcq.assert_negative_clippy(
                gcq.GCQ_ROOT / "negative_seeds" / "clippy_warning.rs", run_root
            ),
        )
        for entry in gcq.selected_positive_entries(entries, args.group):
            divergence = divergences_by_entry.get(entry.id)
            exercised_ids.add(entry.id)
            try:
                crate_root, observed = gcq.timed_case(
                    "generated_code_quality",
                    f"clippy-release/{entry.id}",
                    lambda: check_entry(entry, divergence, run_root),
                )
            except Exception as error:
                failures.append(f"{entry.id}: {error}")
                continue
            record = gcq.record_for_entry(
                entry,
                crate_root,
                "expected-failure" if divergence is not None else "passed",
            )
            if divergence is not None:
                record["release_divergence"] = {
                    "record_id": divergence.record_id,
                    "expires_on": divergence.expires_on.isoformat(),
                    "observed_lints": list(observed or ()),
                }
            records.append(record)
        try:
            require_all_divergences_exercised(divergences_by_entry, exercised_ids)
        except RuntimeError as error:
            failures.append(str(error))
        if failures:
            raise RuntimeError("\n".join(["release Clippy failures:", *failures]))
        evidence = gcq.record_evidence("clippy-release", run_id, records)
        print(
            f"generated-code release clippy passed; evidence={evidence.relative_to(gcq.REPO_ROOT)}"
        )
    except Exception:
        print(
            f"generated-code release clippy failed; preserved={run_root}",
            file=sys.stderr,
        )
        raise
    else:
        if not args.keep_success:
            shutil.rmtree(run_root, ignore_errors=True)


def main() -> None:
    args = parse_args()
    try:
        run(gcq.load_manifest(Path(args.manifest)), args)
    except Exception as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1) from error


if __name__ == "__main__":
    main()
