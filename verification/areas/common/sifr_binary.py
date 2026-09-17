"""Canonical Cargo-selected compiler preparation for verification adapters."""
from __future__ import annotations

import functools
import hashlib
import json
import os
import sys
from pathlib import Path

def resolve_sifr_binary(repo_root: Path, *, explicit_env_var: str | None = None,
                        default_binary: Path | None = None, env: dict[str, str] | None = None) -> Path:
    """Prepare the exact development candidate; never trust an existing pathname."""
    selected_env = dict(os.environ) if env is None else dict(env)
    candidate = _prepare(repo_root.resolve(), tuple(sorted(selected_env.items())))
    override = selected_env.get(explicit_env_var) if explicit_env_var else None
    if override:
        selected = Path(override).resolve()
        if not selected.is_file() or _digest(selected) != _digest(candidate):
            raise RuntimeError(f"compiler override {selected} does not match prepared candidate {candidate}")
        return selected
    return candidate

def _digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()

@functools.lru_cache(maxsize=8)
def _prepare(repo_root: Path, environment: tuple[tuple[str, str], ...]) -> Path:
    # Cargo owns freshness including manifests, build-script inputs and flags.
    sys.path.insert(0, str(repo_root / "verification/runner"))
    from sifr_verify.fixture_execution import run_process
    proc = run_process(["cargo", "build", "--locked", "-q", "-p", "sifr",
                       "--bin", "sifr", "--message-format=json"], cwd=repo_root, env=dict(environment))
    if proc.returncode or proc.cause != "exit" or proc.truncated:
        raise RuntimeError(f"failed to prepare compiler (exit {proc.returncode}):\\n{proc.stderr}")
    executables = []
    for line in proc.stdout.splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact" and message.get("target", {}).get("name") == "sifr" and message.get("executable"):
            executables.append(Path(message["executable"]))
    if len(executables) != 1 or not executables[0].is_file():
        raise RuntimeError("Cargo did not produce exactly one Sifr compiler executable")
    return executables[0]
