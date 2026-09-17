"""Subprocess command primitives for validation profile execution."""

from __future__ import annotations

from .process_execution import execute
import sys
import json

from .paths import REPO_ROOT


class CommandFailed(Exception):
    """A subprocess returned a non-zero exit code."""

    def __init__(self, returncode: int) -> None:
        super().__init__(f"command failed with exit code {returncode}")
        self.returncode = returncode


def run_command(command: list[str], *, env: dict[str, str] | None = None) -> None:
    # Prefix every child line, including runner-looking lines. The authoritative
    # events remain emitted solely by the enclosing profile runner.
    pending = {"stdout": "", "stderr": ""}
    def emit(stream: str, data: bytes) -> None:
        pending[stream] += data.decode("utf-8", errors="replace")
        while "\n" in pending[stream]:
            line, pending[stream] = pending[stream].split("\n", 1)
            sys.stdout.write(f"[child:{stream}] {json.dumps(line, ensure_ascii=True)}\n")
    outcome = execute(command, cwd=REPO_ROOT, env=env, emit=emit,
                      deadline_seconds=float((env or {}).get("SIFR_VERIFY_SAFETY_DEADLINE_SECONDS", "2400")))
    for stream, text in pending.items():
        if text:
            sys.stdout.write(f"[child:{stream}] {json.dumps(text, ensure_ascii=True)}\n")
    if outcome.returncode != 0:
        error = CommandFailed(outcome.returncode)
        error.outcome = outcome
        raise error


def uv_area_command(*args: str) -> list[str]:
    return [
        "uv",
        "run",
        "--project",
        "verification",
        "--locked",
        "python",
        "-m",
        "sifr_verify",
        "areas",
        "run",
        *args,
    ]


def cargo_command(*args: str) -> list[str]:
    command = ["cargo", *args]
    if "--" in command:
        separator = command.index("--")
        return [*command[:separator], "--locked", *command[separator:]]
    return [*command, "--locked"]


def run_python(script: str, *args: str) -> None:
    run_command(["python3", script, *args])
