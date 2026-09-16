"""Artifact-bound compiler measurement lanes; application profiles are independent."""

from __future__ import annotations

import hashlib
import json
import os
import shlex
import subprocess
import tomllib
from pathlib import Path
from typing import Any

from benchmark_manifest import BenchmarkError

LANES = {"contributor-dev": "dev", "product-installed-optimized": "release"}
_SELECTION: dict[str, Any] | None = None


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def configure(repo: Path, lane: str, receipt_path: str) -> None:
    """Validate a prepared artifact before selecting any benchmark command.

    Receipts come from prepare_compiler_lane.py, whose Cargo JSON records the
    actual profile. A path or environment label alone cannot select product.
    """
    global _SELECTION
    if lane not in LANES:
        raise BenchmarkError(f"unknown compiler measurement lane: {lane}")
    if not receipt_path:
        if lane != "contributor-dev":
            raise BenchmarkError("product lane requires a prepared artifact receipt")
        _SELECTION = {
            "lane": lane, "compiler_build_profile": "dev",
            "artifact": None, "preparation": "existing source-tree Cargo builder",
        }
        return
    try:
        receipt = json.loads(Path(receipt_path).read_text())
        validate_receipt(repo, lane, receipt)
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise BenchmarkError(f"incomparable compiler artifact: {error}") from error
    _SELECTION = receipt
    _SELECTION["receipt_sha256"] = digest(Path(receipt_path))
    command = [receipt["artifact"]["path"], "lsp", "--stdio"]
    if os.environ.get("SIFR_LSP_COMMAND") not in (None, shlex.join(command)):
        raise BenchmarkError("LSP command differs from selected compiler artifact")
    os.environ["SIFR_LSP_COMMAND"] = shlex.join(command)
    # The installed lane must use its packaged sysroot, never a source override.
    for key in ("SIFR_SYSROOT", "SIFR_SYSROOT_MODE"):
        if os.environ.get(key):
            raise BenchmarkError(f"explicit compiler receipt conflicts with {key}")


def validate_receipt(repo: Path, lane: str, receipt: dict[str, Any]) -> None:
    if receipt["schema_version"] != 1 or receipt["lane"] != lane:
        raise ValueError("receipt schema or lane mismatch")
    if receipt["compiler_build_profile"] != LANES[lane]:
        raise ValueError("compiler profile does not match lane")
    profile = receipt["cargo_artifact_profile"]
    if profile["test"] or profile["opt_level"] != ("3" if lane == "product-installed-optimized" else "1"):
        raise ValueError("Cargo artifact profile does not match lane")
    if lane == "product-installed-optimized" and profile["debug_assertions"]:
        raise ValueError("product compiler has debug assertions")
    artifact = receipt["artifact"]
    path = Path(artifact["path"])
    if not path.is_absolute() or not os.access(path, os.X_OK) or digest(path) != artifact["sha256"]:
        raise ValueError("compiler executable is absent, changed or not executable")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
    if receipt["source_commit"] != head:
        raise ValueError("prepared compiler source does not match candidate")
    if receipt["cargo_lock_sha256"] != digest(repo / "Cargo.lock"):
        raise ValueError("prepared compiler lock differs from candidate")
    if receipt["rustc"] != subprocess.check_output(["rustc", "--version"], text=True).strip():
        raise ValueError("prepared compiler toolchain changed")
    sysroot = receipt["sysroot"]
    if lane == "product-installed-optimized":
        root = Path(sysroot["path"])
        if path != root / "bin" / "sifr":
            raise ValueError("product executable is not in the selected installed toolchain")
        manifest = root / "sysroot.toml"
        if digest(manifest) != sysroot["manifest_sha256"]:
            raise ValueError("installed sysroot manifest changed")
        parsed = tomllib.loads(manifest.read_text())
        if parsed["built-by-compiler-commit"] != head:
            raise ValueError("installed sysroot source differs from compiler")
        if parsed["cargo-lock-sha256"] != receipt["cargo_lock_sha256"]:
            raise ValueError("installed sysroot lock differs from compiler")
        from subprocess import run
        checked = run([str(path), "doctor", "--json"], capture_output=True, text=True)
        if checked.returncode:
            raise ValueError(f"installed sysroot rejected: {checked.stderr[-2000:]}")


def selection() -> dict[str, Any]:
    return _SELECTION or {
        "lane": "contributor-dev", "compiler_build_profile": "dev",
        "artifact": None, "preparation": "existing source-tree Cargo builder",
    }


def selected_binary(default: Path) -> Path:
    artifact = selection().get("artifact")
    return Path(artifact["path"]) if artifact else default


def prepared() -> bool:
    return selection().get("artifact") is not None


def assert_comparable(expected: dict[str, Any], actual: dict[str, Any]) -> None:
    """Comparison contracts exclude changing compiler bytes but bind the lane."""
    for key in ("lane", "compiler_build_profile", "application_profile", "verification_selection"):
        if expected.get(key) is None or expected.get(key) != actual.get(key):
            raise BenchmarkError(f"incomparable measurement: {key}")
