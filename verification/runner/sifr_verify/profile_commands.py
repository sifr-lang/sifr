"""Subprocess command primitives for validation profile execution."""

from __future__ import annotations

import subprocess
import sys

from .paths import REPO_ROOT


class CommandFailed(Exception):
    """A subprocess returned a non-zero exit code."""

    def __init__(self, returncode: int) -> None:
        super().__init__(f"command failed with exit code {returncode}")
        self.returncode = returncode


def run_command(command: list[str], *, env: dict[str, str] | None = None) -> None:
    proc = subprocess.Popen(
        command,
        cwd=REPO_ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    assert proc.stdout is not None
    for line in proc.stdout:
        sys.stdout.write(line)
    returncode = proc.wait()
    if returncode != 0:
        raise CommandFailed(returncode)


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
