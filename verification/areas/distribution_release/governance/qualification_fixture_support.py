"""Process and digest helpers for stable qualification fixtures."""

from __future__ import annotations

import hashlib
import importlib.util
import json
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


def metadata_fixture_files(binary: bytes, target: str) -> dict[str, bytes]:
    """Build an explicitly synthetic empty container with canonical envelope checks."""
    script = Path(__file__).resolve().parents[4] / "scripts/distribution/metadata_artifact.py"
    spec = importlib.util.spec_from_file_location("_fixture_metadata_envelope", script)
    if spec is None or spec.loader is None:
        raise AssertionError(f"cannot load {script}")
    metadata = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(metadata)
    compiler = hashlib.sha256(binary).hexdigest()
    semantic = metadata.semantic_target_id(target)
    inputs = hashlib.sha256(b"synthetic-governance-fixture-inputs").hexdigest()
    logical = (b"SIFRMETA" + (4).to_bytes(4, "little") + bytes(4)
               + bytes.fromhex(compiler + semantic + inputs) + (120).to_bytes(8, "little"))
    frames = (bytes.fromhex("28b52ffd2078c10300") + logical
              + bytes.fromhex("28b52ffd2000010000") * 2)
    payload = logical[:112] + (128 + len(frames)).to_bytes(8, "little") + (120).to_bytes(8, "little") + frames
    descriptor = {"schema_version": 1, "compiler_identity": compiler, "semantic_target": target,
                  "semantic_target_id": semantic, "stdlib_inputs_id": inputs,
                  "metadata_id": hashlib.sha256(payload).hexdigest(),
                  "compiler_binary_sha256": compiler}
    metadata.validate_metadata(payload, descriptor, compiler, target)
    return {metadata.METADATA_PATH: payload,
            metadata.DESCRIPTOR_PATH: (json.dumps(descriptor, sort_keys=True) + "\n").encode()}
