"""Reconcile actual in-process gate outcomes from a fresh-corpus batch child."""
from __future__ import annotations

import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

from quality_batch import selected_ids, validate_requests


def reconcile(payload, requests, selections, returncode, workspace):
    validate_requests(requests)
    if (not isinstance(payload, dict) or payload.get("schema_version") != 1
            or payload.get("kind") != "fresh-generated-quality-batch"
            or payload.get("complete") is not True or payload.get("requests") != requests
            or payload.get("workspace") != str(workspace)):
        raise ValueError("missing, incomplete or mismatched quality batch report")
    rows = payload.get("results")
    if not isinstance(rows, list) or len(rows) != len(requests):
        raise ValueError("missing or duplicate quality batch outcomes")
    for row, request in zip(rows, requests, strict=True):
        if not isinstance(row, dict) or any(row.get(k) != v for k, v in request.items()):
            raise ValueError("quality batch outcome selection mismatch")
        if row.get("selected_entries") != selections[request["mode"]]:
            raise ValueError("quality batch selected corpus mismatch")
        seconds = row.get("duration_ms")
        if type(seconds) not in (int, float) or not math.isfinite(seconds) or seconds < 0:
            raise ValueError("invalid quality batch duration")
        errors = row.get("errors")
        if not isinstance(errors, list) or any(not isinstance(error, str) or not error for error in errors):
            raise ValueError("invalid quality batch failure evidence")
        expected = "fail" if errors else "pass"
        if row.get("status") != expected or row.get("gate_result") != ("raised" if errors else "returned"):
            raise ValueError("contradictory quality batch gate outcome")
    if returncode != (1 if any(row["status"] == "fail" for row in rows) else 0):
        raise ValueError("quality batch process exit contradicts gate evidence")
    corpus = payload.get("materialization")
    if not isinstance(corpus, dict) or corpus.get("ownership") != "fresh-current-batch":
        raise ValueError("quality batch has no fresh corpus owner evidence")
    programs = corpus.get("programs")
    if not isinstance(programs, list) or any(not isinstance(p, dict) for p in programs):
        raise ValueError("invalid generated corpus evidence")
    names = [program.get("id") for program in programs]
    if any(not isinstance(name, str) or not name for name in names) or len(set(names)) != len(names):
        raise ValueError("missing or duplicate generated program identity")
    by_id = dict(zip(names, programs, strict=True))
    selected_names = {name for names in selections.values() for name in names}
    for name, program in by_id.items():
        root = program.get("crate_root")
        consumers = program.get("consumers")
        files = program.get("generated_files")
        if (name not in selected_names or not isinstance(root, str)
                or not Path(root).resolve().is_relative_to(workspace.resolve())
                or not isinstance(consumers, list)
                or any(consumer not in selections for consumer in consumers)
                or not isinstance(files, dict) or not files
                or any(not isinstance(path, str) or Path(path).is_absolute() or ".." in Path(path).parts
                       or not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest)
                       for path, digest in files.items())):
            raise ValueError("generated corpus identity or file evidence is invalid")
    for row in rows:
        if row["status"] != "pass":
            continue
        if corpus.get("poisoned"):
            raise ValueError("passing gate on a poisoned generated corpus")
        for name in row["selected_entries"]:
            program = by_id.get(name)
            if (program is None or row["mode"] not in program.get("consumers", [])
                    or not isinstance(program.get("generated_files"), dict)
                    or not program["generated_files"]):
                raise ValueError("passing gate lacks fresh program consumption evidence")
    return rows


def run_shared_gates(suite, plan, *, repo_root: Path, area_root: Path):
    import generated_code_quality as api
    requests = [{"mode": mode, "max_entries": limit,
                 "entry_ids": list(ids) if ids is not None else None}
                for mode, limit, ids in plan]
    validate_requests(requests)
    entries = api.load_manifest(api.MANIFEST)
    selections = {r["mode"]: selected_ids(api, entries, r) for r in requests}
    parent = repo_root / "target/sifr_generated_code_quality/batches"
    parent.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix="batch-", dir=parent))
    request_path, report = root / "requests.json", root / "results.json"
    request_path.write_text(json.dumps(requests, indent=2) + "\n")
    argv = [sys.executable, str(area_root / "quality_batch.py"),
            "--requests", str(request_path), "--report", str(report),
            "--workspace", str(root / "programs")]
    start = time.perf_counter()
    returncode = None
    failure = None
    rows = []
    try:
        proc = subprocess.run(argv, cwd=repo_root, env=os.environ.copy(), check=False)
        returncode = proc.returncode
        rows = reconcile(json.loads(report.read_text()), requests, selections, returncode, root / "programs")
    except (OSError, ValueError) as error:
        failure = str(error)
    elapsed = (time.perf_counter() - start) * 1000
    process = {"suite": suite, "argv": argv, "returncode": returncode, "duration_ms": elapsed,
               "protocol_failure": failure, "report": str(report)}
    (root / "process.json").write_text(json.dumps(process, indent=2) + "\n")
    by_mode = {row["mode"]: row for row in rows}
    variants = {}
    for request in requests:
        row = by_mode.get(request["mode"]) if failure is None else None
        status = row["status"] if row else "fail"
        variants[request["mode"]] = {
            "label": request["mode"], "max_entries": request["max_entries"],
            "entry_ids": request["entry_ids"], "status": status,
            "mismatches": row["errors"] if row else ["invalid-batch-evidence: " + str(failure)],
            "duration_ms": round(row["duration_ms"], 3) if row else None,
            # A gate function is not an OS process. Keep its result distinct
            # from the shared child's real argv/exit; never invent child runs.
            "execution_kind": "in-process-gate", "batch_process": process,
            "gate_result": row["gate_result"] if row else "unqualified",
            "case_evidence": {"selected_entries": selections[request["mode"]],
                              "fresh_corpus_report": str(report)},
        }
    return variants
