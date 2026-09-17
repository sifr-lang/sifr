"""Execution adapters for existing fixture manifests; assertions never come from caches."""
from __future__ import annotations

import functools
import json
import sys
from dataclasses import dataclass
from pathlib import Path

from .paths import REPO_ROOT
from .process_execution import execute

@dataclass(frozen=True)
class ProcessResult:
    returncode: int
    stdout: str
    stderr: str
    cause: str
    truncated: bool

def run_process(argv, *, cwd, env=None):
    outcome = execute(argv, cwd=cwd, env=env)
    return ProcessResult(outcome.returncode, outcome.stdout.decode("utf-8", errors="replace"),
                         outcome.stderr.decode("utf-8", errors="replace"),
                         outcome.cause, outcome.truncated)

@functools.cache
def compiler_binary():
    sys.path.insert(0, str(REPO_ROOT / "verification/areas/common"))
    from sifr_binary import resolve_sifr_binary
    return resolve_sifr_binary(REPO_ROOT)

def selected_cases(suites, requested):
    available = {f"{suite['name']}/{case['id']}" for suite in suites for case in suite["cases"]}
    if requested - available:
        raise ValueError(f"unknown case selection: {sorted(requested - available)}")
    if not requested:
        return suites
    return [{**suite, "cases": [case for case in suite["cases"]
             if f"{suite['name']}/{case['id']}" in requested]} for suite in suites
            if any(f"{suite['name']}/{case['id']}" in requested for case in suite["cases"])]

def failed_selection(path):
    payload = json.loads(Path(path).read_text())
    return frozenset(f"{suite['name']}/{case['id']}"
        for suite in payload["suites"] for case in suite["cases"]
        if any(variant["status"] != "pass" for variant in case["variants"]))

def blocked_case(case, reason):
    formats = case.get("diagnostic_formats") or [None]
    variants = [{"label": case["command"] + (f"-{fmt}" if fmt else ""),
                 "diagnostic_format": fmt, "status": "blocked",
                 "mismatches": ["prerequisite-failed"], "reason": reason,
                 "actual_exit_code": None, "expected_exit_code": case["expect_exit_code"],
                 "duration_ms": 0} for fmt in formats]
    return ({"id": case["id"], "entry": case["entry"], "command": case["command"],
             "variants": variants}, True, len(variants))

def assertion_selection(case):
    command = case["command"]
    if command in {"run", "test"}:
        return ["check", "native-compile-link", "runtime", "diagnostic-baseline"]
    if command == "build":
        return ["check", "native-compile-link", "diagnostic-baseline"]
    if command in {"check", "package-check", "lint", "fmt-check"}:
        return [command, "diagnostic-baseline"]
    return [command]

