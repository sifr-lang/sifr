"""Process and digest helpers for stable qualification fixtures."""

from __future__ import annotations

import hashlib
import os
import subprocess
from pathlib import Path


def configure_git(root: Path) -> None:
    git(root, "config", "user.name", "Sifr Fixture")
    git(root, "config", "user.email", "fixture@sifr.invalid")


def git(root: Path, *args: str) -> None:
    env = os.environ.copy()
    if args and args[0] == "commit":
        env.update(
            {
                "GIT_AUTHOR_DATE": "2026-01-01T00:00:00Z",
                "GIT_COMMITTER_DATE": "2026-01-01T00:00:00Z",
            }
        )
    subprocess.run(
        ["git", *args],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )


def git_output(root: Path, *args: str) -> str:
    return command_output(root, "git", *args)


def command_output(root: Path, *args: str) -> str:
    return subprocess.run(
        list(args),
        cwd=root,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout.strip()


def digest_text(value: str) -> str:
    return hashlib.sha256(value.encode()).hexdigest()
