"""Validate the exact WASI-Virt source used by SQL compiler components."""

from __future__ import annotations

import hashlib
import subprocess
import tomllib
from pathlib import Path

WASI_VIRT_VERSION = "0.2.0"
WASI_VIRT_COMMIT = "448f6df8f688cee5d6995e96b1ffc31f9bf00742"
WASI_VIRT_SOURCE_SHA256 = "47c1ca1cc80df330c93c4797f6748d5330c2804001bdcff0342c4001920d1d2e"


def tracked_content_sha256(repository: Path) -> str:
    digest = hashlib.sha256()
    paths = subprocess.check_output(
        ["git", "-C", str(repository), "ls-files", "-z"]
    ).split(b"\0")
    for raw_path in paths:
        if raw_path:
            digest.update(raw_path)
            digest.update(b"\0")
            digest.update(hashlib.sha256((repository / raw_path.decode()).read_bytes()).digest())
    return digest.hexdigest()


def validate_wasi_virt(repository: Path) -> None:
    manifest_path = repository / "Cargo.toml"
    if not manifest_path.is_file():
        raise SystemExit("initialize the pinned third_party/wasi-virt submodule")
    package = tomllib.loads(manifest_path.read_text(encoding="utf-8"))["package"]
    if package.get("version") != WASI_VIRT_VERSION:
        raise SystemExit(f"SQL components require WASI-Virt {WASI_VIRT_VERSION}")
    commit = subprocess.check_output(
        ["git", "-C", str(repository), "rev-parse", "HEAD"], text=True
    ).strip()
    if commit != WASI_VIRT_COMMIT:
        raise SystemExit("third_party/wasi-virt commit has drifted")
    if tracked_content_sha256(repository) != WASI_VIRT_SOURCE_SHA256:
        raise SystemExit("third_party/wasi-virt content has drifted")
