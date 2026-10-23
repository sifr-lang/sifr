"""Fresh generated-graph preparation under the profile's subprocess budget."""
from __future__ import annotations

import json
import os
from pathlib import Path
import tempfile
import time

from .batch_execution import execute_batch
from .build_allocation import positive
from .paths import REPO_ROOT


def materialize_entries(quality, entries, run_root: Path) -> tuple[list[Path], Path]:
    entries = list(entries)
    names = [entry.id for entry in entries]
    if not names or len(names) != len(set(names)):
        raise ValueError("generated preparation requires a nonempty unique selection")
    environment = os.environ.copy()
    workers = positive(environment.get("SIFR_VERIFY_PROCESS_SLOTS", "1"),
                       "SIFR_VERIFY_PROCESS_SLOTS")
    if workers > 1:
        environment.update(CARGO_BUILD_JOBS="1", RAYON_NUM_THREADS="1")
    run_root.mkdir(parents=True, exist_ok=True)
    batch_root = Path(tempfile.mkdtemp(prefix="batch-", dir=run_root))
    report_path = batch_root / "materialization.json"
    rows = [{"id": entry.id, "group": entry.group, "status": "pending"} for entry in entries]
    report = {"status": "incomplete", "workers": workers, "selected": len(entries),
              "observed": 0, "rows": rows,
              "inner_cargo_jobs": environment.get("CARGO_BUILD_JOBS"),
              "inner_rayon_workers": environment.get("RAYON_NUM_THREADS")}
    roots, commands, observed = [], [], set()
    started = time.monotonic()

    def save():
        report["observed"] = len(observed)
        report["seconds"] = time.monotonic() - started
        temporary = report_path.with_suffix(".writing")
        temporary.write_text(json.dumps(report, indent=2) + "\n")
        temporary.replace(report_path)

    def completed(index, result):
        if type(index) is not int or not 0 <= index < len(entries):
            raise ValueError("unknown generated materialization outcome")
        if index in observed:
            raise ValueError("duplicate generated materialization outcome")
        observed.add(index)
        row = rows[index]
        stdout = roots[index].parent / "materialization.stdout"
        stderr = roots[index].parent / "materialization.stderr"
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        row.update(cause=result.cause, returncode=result.returncode,
                   truncated=result.truncated, seconds=result.elapsed_seconds,
                   stdout=str(stdout), stderr=str(stderr), status="fail")
        if result.cause != "exit" or result.returncode != 0 or result.truncated:
            row["error"] = "materialization process failed or produced incomplete output"
        else:
            try:
                quality.require_materialized_graph(entries[index], roots[index])
                row["status"] = "pass"
            except RuntimeError as error:
                row["error"] = str(error)
        save()
        print(f"[sifr-materialization-result] entry={row['id']} status={row['status']}", flush=True)

    try:
        save()
        for entry, row in zip(entries, rows, strict=True):
            root, command = quality.materialization_request(entry, batch_root)
            roots.append(root)
            commands.append(command)
            row.update(crate_root=str(root), command=command)
        save()
        # Keep the scalar owner's default per-command deadline/output limit and
        # inherited invocation deadline. Only independent command admission overlaps.
        execute_batch(commands, cwd=REPO_ROOT, env=environment,
                      max_parallel=workers, completed=completed)
        if observed != set(range(len(entries))):
            raise RuntimeError("missing generated materialization outcomes")
        failures = [row["id"] for row in rows if row["status"] != "pass"]
        if failures:
            raise RuntimeError(f"generated materialization failed: {failures}; report: {report_path}")
        report["status"] = "pass"
    except BaseException as error:
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        save()
        print(f"[sifr-materialization-report] {report_path}", flush=True)
    return roots, report_path
