"""Helpers for verification tools that need a Sifr CLI binary."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

FRESHNESS_INPUTS = (
    "Cargo.lock",
    "Cargo.toml",
    "crates",
    "stdlib",
    "third_party/ruff/crates",
)


def resolve_sifr_binary(
    repo_root: Path,
    *,
    explicit_env_var: str | None = None,
    default_binary: Path | None = None,
) -> Path:
    """Return a Sifr binary path, building the configured target if needed."""
    if explicit_env_var:
        configured_bin = os.environ.get(explicit_env_var)
        if configured_bin:
            return Path(configured_bin)

    target_bin = _configured_target_binary(repo_root)
    if target_bin is not None:
        if not target_bin.is_file() or _binary_is_stale(repo_root, target_bin):
            _build_sifr_binary(repo_root, target_bin)
        return target_bin

    fallback = default_binary or repo_root / "target" / "debug" / "sifr"
    if not fallback.is_file() or _binary_is_stale(repo_root, fallback):
        _build_sifr_binary(repo_root, fallback)
    return fallback


def _configured_target_binary(repo_root: Path) -> Path | None:
    configured_target_dir = os.environ.get("CARGO_TARGET_DIR")
    if not configured_target_dir:
        return None
    target_dir = Path(configured_target_dir)
    if not target_dir.is_absolute():
        target_dir = repo_root / target_dir
    return target_dir / "debug" / "sifr"


def _binary_is_stale(repo_root: Path, binary: Path) -> bool:
    binary_mtime = binary.stat().st_mtime
    for relative in FRESHNESS_INPUTS:
        path = repo_root / relative
        if path.is_file() and path.stat().st_mtime > binary_mtime:
            return True
        if path.is_dir():
            for child in path.rglob("*"):
                if child.is_file() and child.stat().st_mtime > binary_mtime:
                    return True
    return False


def _build_sifr_binary(repo_root: Path, expected_binary: Path) -> None:
    proc = subprocess.run(
        ["cargo", "build", "--locked", "-q", "-p", "sifr"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        print("failed to build Sifr verification binary", file=sys.stderr)
        if proc.stdout:
            print(proc.stdout, file=sys.stderr)
        if proc.stderr:
            print(proc.stderr, file=sys.stderr)
        raise SystemExit(proc.returncode)
    if not expected_binary.is_file():
        print(f"Sifr verification binary was not produced: {expected_binary}", file=sys.stderr)
        raise SystemExit(1)
