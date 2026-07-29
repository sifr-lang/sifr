#!/usr/bin/env python3
"""Fail-closed self-tests for the generated-code release divergence."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path
from types import SimpleNamespace
from typing import Any, Callable

import release_clippy
import runner as gcq_runner
from generated_code_quality import load_manifest

AREA_ROOT = Path(__file__).resolve().parent
CANONICAL_PATH = AREA_ROOT / "data" / "release_divergences.json"
CORPUS_MANIFEST = AREA_ROOT / "data" / "corpus_manifest.json"


def canonical_payload() -> dict[str, Any]:
    return json.loads(CANONICAL_PATH.read_text(encoding="utf-8"))


def expect_failure(
    name: str, mutate: Callable[[dict[str, Any]], None], diagnostic: str
) -> None:
    payload = canonical_payload()
    mutate(payload)
    with tempfile.TemporaryDirectory(prefix="sifr-gcq-divergence-self-test-") as temp:
        path = Path(temp) / "release_divergences.json"
        path.write_text(json.dumps(payload), encoding="utf-8")
        original = release_clippy.RELEASE_DIVERGENCES
        release_clippy.RELEASE_DIVERGENCES = path
        try:
            try:
                release_clippy.load_divergences(load_manifest(CORPUS_MANIFEST))
            except RuntimeError as error:
                if diagnostic not in str(error):
                    raise AssertionError(
                        f"{name}: expected diagnostic {diagnostic!r}, got {str(error)!r}"
                    ) from error
            else:
                raise AssertionError(f"{name}: mutation unexpectedly passed")
        finally:
            release_clippy.RELEASE_DIVERGENCES = original


def set_record(payload: dict[str, Any], key: str, value: Any) -> None:
    payload["records"][0][key] = value


def expect_check_entry_failure(
    name: str,
    entry: Any,
    divergence: release_clippy.ReleaseClippyDivergence,
    result: SimpleNamespace,
    diagnostic: str,
) -> None:
    original_materialize = release_clippy.gcq.materialize_entry
    original_run = release_clippy.gcq.run_command
    release_clippy.gcq.materialize_entry = lambda *_args: Path("/tmp/fake-sifr-output")

    def fake_run(args: list[str], **_kwargs: Any) -> SimpleNamespace:
        if args[1] == "fmt":
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        return result

    release_clippy.gcq.run_command = fake_run
    try:
        try:
            release_clippy.check_entry(entry, divergence, Path("/tmp/fake-run"))
        except RuntimeError as error:
            if diagnostic not in str(error):
                raise AssertionError(
                    f"{name}: expected diagnostic {diagnostic!r}, got {str(error)!r}"
                ) from error
        else:
            raise AssertionError(f"{name}: mutation unexpectedly passed")
    finally:
        release_clippy.gcq.materialize_entry = original_materialize
        release_clippy.gcq.run_command = original_run


def main() -> int:
    entries = load_manifest(CORPUS_MANIFEST)
    divergences = release_clippy.load_divergences(entries)
    assert [
        (
            divergence.record_id,
            divergence.entry_ids,
            divergence.expires_on.isoformat(),
            divergence.expected_lints,
        )
        for divergence in divergences
    ] == [
        (
            "GENC-NAN",
            (
                "e2e-018-cpython-math-semantic-corrections",
                "e2e-027-error-mixed-builtin-stdlib",
                "stdlib-007-math",
            ),
            "2026-10-31",
            ("clippy::zero_divided_by_zero",),
        )
    ]
    assert gcq_runner.PROFILE_SUITES["release-full"] == [
        ("clippy-release", max_entries, entry_ids)
        if gate == "clippy"
        else (gate, max_entries, entry_ids)
        for gate, max_entries, entry_ids in gcq_runner.PROFILE_SUITES["full"]
    ]
    assert release_clippy.extract_clippy_lints(
        "-D clippy::zero-divided-by-zero\n" "#[allow(clippy::zero_divided_by_zero)]\n"
    ) == ("clippy::zero_divided_by_zero",)

    cases: list[tuple[str, Callable[[dict[str, Any]], None], str]] = [
        (
            "expired",
            lambda payload: set_record(payload, "expires_on", "2000-01-01"),
            "expired",
        ),
        (
            "unknown entry",
            lambda payload: set_record(payload, "entry_ids", ["missing"]),
            "unknown entry_ids",
        ),
        (
            "wrong gate",
            lambda payload: set_record(payload, "gate", "rustfmt"),
            "only the clippy gate",
        ),
        (
            "empty lints",
            lambda payload: set_record(payload, "expected_lints", []),
            "invalid expected_lints",
        ),
        (
            "unbound record",
            lambda payload: set_record(payload, "record_id", "UNBOUND"),
            "matrix mismatch",
        ),
        (
            "entry binding mismatch",
            lambda payload: set_record(
                payload,
                "entry_ids",
                [
                    "e2e-018-cpython-math-semantic-corrections",
                    "e2e-027-error-mixed-builtin-stdlib",
                    "stdlib-008-env",
                ],
            ),
            "matrix mismatch",
        ),
        (
            "malformed header",
            lambda payload: payload.update(schema_version=2),
            "invalid generated-code release divergence document header",
        ),
        (
            "malformed record",
            lambda payload: payload["records"][0].pop("gate"),
            "invalid fields",
        ),
        (
            "duplicate record",
            lambda payload: payload["records"].append(dict(payload["records"][0])),
            "duplicate record_id",
        ),
        (
            "duplicate entries",
            lambda payload: set_record(
                payload,
                "entry_ids",
                [
                    "e2e-018-cpython-math-semantic-corrections",
                    "e2e-018-cpython-math-semantic-corrections",
                ],
            ),
            "invalid entry_ids",
        ),
        (
            "unsorted entries",
            lambda payload: set_record(
                payload,
                "entry_ids",
                ["stdlib-007-math", "e2e-018-cpython-math-semantic-corrections"],
            ),
            "invalid entry_ids",
        ),
        (
            "negative entry",
            lambda payload: set_record(
                payload, "entry_ids", ["negative-003-clippy-warning"]
            ),
            "divergence entries must be positive",
        ),
    ]
    for name, mutate, diagnostic in cases:
        expect_failure(name, mutate, diagnostic)

    entries_by_id = {entry.id: entry for entry in entries}
    divergence = divergences[0]
    governed_entry = entries_by_id[divergence.entry_ids[0]]
    expect_check_entry_failure(
        "unexpected pass",
        governed_entry,
        divergence,
        SimpleNamespace(returncode=0, stdout="", stderr=""),
        "unexpectedly passed",
    )
    expect_check_entry_failure(
        "additional lint",
        governed_entry,
        divergence,
        SimpleNamespace(
            returncode=1,
            stdout="",
            stderr=(
                "#[allow(clippy::zero_divided_by_zero)]\n"
                "#[allow(clippy::unwrap_used)]\n"
            ),
        ),
        "observed ['clippy::unwrap_used', 'clippy::zero_divided_by_zero']",
    )
    try:
        release_clippy.require_all_divergences_exercised(
            {entry_id: divergence for entry_id in divergence.entry_ids},
            {divergence.entry_ids[0]},
        )
    except RuntimeError as error:
        assert "did not exercise governed entries" in str(error)
    else:
        raise AssertionError("missing governed entries unexpectedly passed")

    total_cases = len(cases) + 3
    print(
        f"generated-code release divergence self-test: PASS ({total_cases} mutations)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
