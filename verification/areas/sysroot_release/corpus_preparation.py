"""Bind the prepared corpus oracle to current inputs; assertions never rebuild it."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "verification/runner"))
from sifr_verify.fixture_inventory import inventory

BUILD_ENV = frozenset({
    "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET", "CARGO_HOME", "RUSTUP_HOME",
    "CARGO_TARGET_DIR", "CARGO_INCREMENTAL", "SIFR_RELEASE_VERSION",
})


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def host_triple(env: dict[str, str]) -> str:
    version = subprocess.check_output(["rustc", "-vV"], env=env, text=True)
    hosts = [line.removeprefix("host: ") for line in version.splitlines()
             if line.startswith("host: ")]
    if len(hosts) != 1 or not hosts[0]:
        raise RuntimeError("rustc did not report one native target")
    return hosts[0]


def receipt_path(root: Path) -> Path:
    return root / "target/sysroot_release/corpus-oracle-prepared.json"


def oracle_path(root: Path, host: str) -> Path:
    from verification.areas.common.release_compiler import release_target_dir
    return release_target_dir(root) / host / "release/sifr_corpus_oracle"


def binding(root: Path, env: dict[str, str], snapshot: Path, host: str, development: dict[str, str]) -> dict:
    from verification.areas.common.release_compiler import RELEASE_VERSION
    binary = oracle_path(root, host)
    compiler = binary.with_name("sifr")
    if not snapshot.is_dir() or not binary.is_file() or not compiler.is_file():
        raise RuntimeError("corpus oracle, compiler and source snapshot must be explicitly prepared")
    version = subprocess.check_output([str(binary), "--version"], cwd=root,
                                      env=env, text=True).strip()
    if version != f"sifr-corpus-oracle {RELEASE_VERSION}":
        raise RuntimeError(f"corpus oracle version mismatch: {version!r}")
    if set(development) != {"sifr", "sifr_corpus_oracle"}:
        raise RuntimeError("development compiler and oracle must be explicitly prepared")
    development_version = subprocess.check_output([development["sifr"], "--version"],
                                                 cwd=root, env=env, text=True).strip()
    oracle_version = subprocess.check_output([development["sifr_corpus_oracle"], "--version"],
                                            cwd=root, env=env, text=True).strip()
    if not development_version.startswith("sifr ") or oracle_version != (
            "sifr-corpus-oracle " + development_version.removeprefix("sifr ")):
        raise RuntimeError("development compiler and corpus oracle versions differ")
    return {
        "schema_version": 2,
        "development": development,
        "development_sha256": {name: digest(Path(path)) for name, path in development.items()},
        "development_version": development_version,
        "inputs": inventory(root)["input_digest"],
        "host": host,
        "version": RELEASE_VERSION,
        "build_environment": {k: v for k, v in sorted(env.items())
                              if k in BUILD_ENV or k.startswith("CARGO_PROFILE_")},
        "rustc": subprocess.check_output(["rustc", "-vV"], env=env, text=True),
        "cargo": subprocess.check_output(["cargo", "--version"], env=env, text=True),
        "compiler_sha256": digest(compiler),
        "oracle": str(binary),
        "oracle_sha256": digest(binary),
        "snapshot": {str(p.relative_to(snapshot)): digest(p)
                     for p in sorted(snapshot.rglob("*")) if p.is_file()},
    }


def record_prepared(root: Path, env: dict[str, str], snapshot: Path, host: str,
                    development: dict[str, str]) -> None:
    value = binding(root, env, snapshot, host, development)
    destination = receipt_path(root)
    pending = destination.with_suffix(f".{os.getpid()}.tmp")
    try:
        pending.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")
        pending.replace(destination)
    finally:
        pending.unlink(missing_ok=True)


def require_prepared(root: Path, env: dict[str, str], snapshot: Path) -> dict:
    try:
        expected = json.loads(receipt_path(root).read_text())
        actual = binding(root, env, snapshot, host_triple(env), expected["development"])
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        raise RuntimeError("corpus oracle preparation is missing or invalid; run package_build.py") from error
    if actual != expected:
        raise RuntimeError("corpus oracle preparation is stale; run package_build.py")
    return actual
