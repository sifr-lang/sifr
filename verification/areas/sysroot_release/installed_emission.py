"""Fresh public CLI emission with bounded execution and exact corpus assertions."""
from __future__ import annotations

import hashlib
from pathlib import Path
import shutil
import time

from sifr_verify.batch_execution import execute_batch
from sifr_verify.build_allocation import positive


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def assert_emission(row, actual: bytes, expected: bytes) -> None:
    name = row["fixture"]
    assert actual == expected, ("public emit bytes differ", name)
    assert actual.endswith(b"\n") == row["final_newline"], name
    start = actual.find(b"// --- stdlib:")
    end = actual.find(b"\n// --- end stdlib ---")
    assert (None if start < 0 else start) == row["preamble_start"], name
    assert (None if end < 0 else end) == row["preamble_end"], name
    for mapping in row["generated_source_maps"]:
        mapped = actual
        if mapping["path"].endswith("#stdlib-preamble"):
            assert start >= 0 and end >= start, name
            mapped = actual[start:end].decode().rstrip().encode() + b"\n"
        assert len(mapped) == mapping["source_bytes"], name
        assert hashlib.sha256(mapped).hexdigest() == mapping["source_sha256"], name


def verify_public_emission(qualification, binary: Path, rows: list[dict]) -> None:
    names = [row["fixture"] for row in rows]
    assert names and len(names) == len(set(names)), "public emit selection is empty or duplicated"
    workers = positive(qualification.env.get("SIFR_VERIFY_PROCESS_SLOTS", "1"),
                       "SIFR_VERIFY_PROCESS_SLOTS")
    environment = dict(qualification.env)
    # Independent outer commands share the profile budget. Nested compiler pools
    # or probes cannot each multiply that allocation.
    if workers > 1:
        environment.update(RAYON_NUM_THREADS="1", CARGO_BUILD_JOBS="1")
    commands = []
    for row in rows:
        source = qualification.workspace / (row["fixture"] + ".sifr")
        shutil.copy2(qualification.corpus / source.name, source)
        assert digest(source) == row["source_sha256"], ("corpus source bytes differ", row["fixture"])
        commands.append([str(binary), "emit", str(source)])
    first_row = len(qualification.rows)
    observed = set()
    failures = []
    started = time.monotonic()

    def completed(index, result):
        assert type(index) is int and 0 <= index < len(rows), "unknown public emit outcome"
        assert index not in observed, "duplicate public emit outcome"
        observed.add(index)
        row = rows[index]
        name = row["fixture"]
        qualification.record("emit-" + name, commands[index], result)
        try:
            assert not result.truncated and result.cause == "exit" and result.returncode == 0, (
                name, result.cause, result.returncode, result.stderr.decode(errors="replace")[-3000:])
            expected = (qualification.corpus / (name + ".source.rs")).read_bytes()
            assert_emission(row, result.stdout, expected)
        except AssertionError as error:
            failures.append((name, str(error)))

    try:
        execute_batch(commands, cwd=qualification.workspace, env=environment,
                      max_parallel=workers, completed=completed,
                      deadline_seconds=900, limit_bytes=16 * 1024 * 1024)
        assert observed == set(range(len(rows))), "missing public emit outcomes"
        assert not failures, ("public emit qualification failed", failures)
    finally:
        # Completion order is an execution detail, not a report-order change.
        order = {"emit-" + name: index for index, name in enumerate(names)}
        qualification.rows[first_row:] = sorted(qualification.rows[first_row:],
                                                key=lambda row: order[row["id"]])
        qualification.report["public_emit_execution"] = {
            "workers": workers, "inner_rayon": environment.get("RAYON_NUM_THREADS"),
            "seconds": time.monotonic() - started, "selected": len(rows),
            "observed": len(observed), "failures": failures,
        }
        qualification.save()
