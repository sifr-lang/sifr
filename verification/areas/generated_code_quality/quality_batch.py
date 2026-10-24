"""Run compatible quality checks over one invocation's fresh generated corpus."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import sys
import time

from fresh_corpus import FreshCorpus

SHARED_GATES = frozenset({"corpus", "panic-scan", "rustfmt", "clippy", "demos"})


def validate_requests(requests):
    if not isinstance(requests, list) or not requests:
        raise ValueError("quality batch requires a nonempty request list")
    seen = set()
    for request in requests:
        if not isinstance(request, dict) or set(request) != {"mode", "max_entries", "entry_ids"}:
            raise ValueError("invalid quality batch request fields")
        mode = request["mode"]
        if mode not in SHARED_GATES or mode in seen:
            raise ValueError("unknown or duplicate quality batch mode")
        seen.add(mode)
        limit = request["max_entries"]
        if limit is not None and (not isinstance(limit, str) or not limit.isdecimal() or int(limit) < 1):
            raise ValueError("invalid quality batch entry limit")
        ids = request["entry_ids"]
        if ids is not None and (not isinstance(ids, list) or not ids
                                or any(not isinstance(name, str) or not name for name in ids)
                                or len(ids) != len(set(ids))):
            raise ValueError("invalid quality batch explicit selection")
    return requests


@contextmanager
def selection_environment(request):
    values = {"SIFR_GCQ_MAX_ENTRIES": request["max_entries"],
              "SIFR_GCQ_ENTRY_IDS": ",".join(request["entry_ids"]) if request["entry_ids"] else None}
    old = {key: os.environ.get(key) for key in values}
    try:
        for key, value in values.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value
        yield
    finally:
        for key, value in old.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def selected_ids(api, entries, request):
    with selection_environment(request):
        groups = ["demos-required"] if request["mode"] == "demos" else []
        return [entry.id for entry in api.selected_positive_entries(entries, groups)]


def execute(api, requests, workspace: Path, report_path: Path) -> int:
    validate_requests(requests)
    entries = api.load_manifest(api.MANIFEST)
    owner = FreshCorpus(api.materialize_entry, workspace)
    report = {"schema_version": 1, "kind": "fresh-generated-quality-batch", "workspace": str(workspace),
              "requests": requests, "complete": False, "results": []}
    started = time.perf_counter()

    def save():
        report["materialization"] = owner.evidence()
        report["elapsed_ms"] = (time.perf_counter() - started) * 1000
        temporary = report_path.with_suffix(".writing")
        temporary.write_text(json.dumps(report, indent=2) + "\n")
        temporary.replace(report_path)

    passed = False
    keep_success = os.environ.get("SIFR_GCQ_KEEP_SUCCESS") == "1"
    try:
        save()
        for request in requests:
            mode = request["mode"]
            owner.consumer = mode
            errors = []
            selected = []
            tick = time.perf_counter()
            try:
                selected = selected_ids(api, entries, request)
                owner.verify()
                args = argparse.Namespace(group=[], keep_success=keep_success, materialize=owner.get)
                with selection_environment(request):
                    getattr(api, "gate_" + mode.replace("-", "_"))(entries, args)
            except (Exception, SystemExit) as error:
                errors.append(f"{type(error).__name__}: {error}")
            try:
                owner.verify()
            except (OSError, RuntimeError) as error:
                errors.append(str(error))
            status = "fail" if errors else "pass"
            report["results"].append({**request, "status": status, "errors": errors,
                                      "selected_entries": selected,
                                      "gate_result": "raised" if errors else "returned",
                                      "duration_ms": (time.perf_counter() - tick) * 1000})
            save()
            print(f"[sifr-quality-batch] mode={mode} status={status}", flush=True)
        passed = all(row["status"] == "pass" for row in report["results"])
        # Cleanup is part of successful completion, not work after qualification.
        owner.finish(passed=passed, keep_success=keep_success)
        report["complete"] = True
        save()
    finally:
        if not report["complete"]:
            save()
        if not passed:
            print(f"generated quality batch failed; preserved={workspace}", file=sys.stderr)
    return 0 if passed else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--requests", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--workspace", required=True, type=Path)
    args = parser.parse_args()
    import generated_code_quality as api
    return execute(api, json.loads(args.requests.read_text()), args.workspace, args.report)


if __name__ == "__main__":
    raise SystemExit(main())
