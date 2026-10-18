"""Runner-owned command receipts; child progress remains an observation."""
from __future__ import annotations
import json
import math
import time
import uuid


def emit(row):
    print("[sifr-command] " + json.dumps(row, sort_keys=True), flush=True)


def decode_child(line):
    # Nested runners quote children again. Decode observations, never authority.
    for _ in range(8):
        if not line.startswith(("[child:stdout] ", "[child:stderr] ")):
            return line
        try:
            line = json.loads(line.split("] ", 1)[1])
        except (ValueError, IndexError):
            return ""
        if not isinstance(line, str):
            return ""
        line = line.strip()
    return ""


class CommandProgress:
    def __init__(self, command, cwd, env, deadline_seconds=2400):
        self.id = uuid.uuid4().hex
        self.start = time.monotonic()
        self.last_heartbeat = self.start
        self.observation = None
        self.observed_at = None
        self.common = {"schema": 1, "id": self.id, "scope": "command",
                       "command": command, "cwd": str(cwd)}
        names = ("CARGO_BUILD_JOBS", "SIFR_E2E_SIFR_JOBS", "SIFR_E2E_RUST_JOBS",
                 "SIFR_E2E_CARGO_BUILD_JOBS", "SIFR_E2E_RUN_JOBS",
                 "SIFR_VERIFY_SAFETY_DEADLINE_SECONDS", "SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC")
        try:
            remaining = float(deadline_seconds)
            if "SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC" in env:
                remaining = min(remaining, float(env["SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC"]) - self.start)
            remaining = max(0, remaining) if math.isfinite(remaining) else None
        except (TypeError, ValueError):
            remaining = None
        emit(self.common | {"event": "start", "unix_ms": time.time_ns() // 1000000,
                            "environment_budgets": {name: env.get(name) for name in names},
                            "safety_seconds_at_start": remaining})

    def observe(self, line):
        line = decode_child(line.strip())
        if line.startswith(("Compiling ", "Checking ", "Fresh ", "Linking ",
                            "Blocking waiting for file lock", "[sifr-e2e-event] ")):
            self.observation = line[:1200]
            self.observed_at = time.monotonic()

    def heartbeat(self, elapsed):
        now = time.monotonic()
        if now - self.last_heartbeat < 30:
            return
        self.last_heartbeat = now
        emit(self.common | {"event": "heartbeat", "elapsed_ms": int(elapsed * 1000),
             "last_observed_log": self.observation,
             "observation_age_seconds": None if self.observed_at is None else now - self.observed_at,
             "note": "Last observed output is not proof of current compiler activity; silence is not a hang."})

    def finish(self, outcome):
        emit(self.common | {"event": "end", "unix_ms": time.time_ns() // 1000000,
             "elapsed_ms": int((time.monotonic() - self.start) * 1000),
             "process_elapsed_ms": int(outcome.elapsed_seconds * 1000),
             "status": "pass" if outcome.returncode == 0 else "fail",
             "cause": outcome.cause, "exit_code": outcome.returncode,
             "output_truncated": outcome.truncated})

    def error(self, error):
        emit(self.common | {"event": "end", "elapsed_ms": int((time.monotonic() - self.start) * 1000),
             "status": "incomplete", "cause": "infrastructure" if isinstance(error, (OSError, ValueError)) else "exception",
             "error_type": type(error).__name__})
