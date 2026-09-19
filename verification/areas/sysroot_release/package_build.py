"""Exact packaged compiler preparation, separate from timed installed assertions."""
from __future__ import annotations
import os
from pathlib import Path
import subprocess
import sys

RELEASE_VERSION = "0.1.0-beta.1300"


def package_build_configuration(root: Path, original: dict[str, str], host: str,
                                artifact_dir: Path, *, prepare_only: bool = False):
    env = dict(original, CARGO_TARGET_DIR=str((root / "target/sysroot_release/cargo-target").resolve()),
               SIFR_RELEASE_VERSION=RELEASE_VERSION)
    command = ["scripts/distribution/build_release_artifacts.sh", "--version", RELEASE_VERSION,
               "--output-dir", str(artifact_dir), "--target", host, "--cargo-build"]
    if prepare_only:
        command.append("--prepare-only")
    return command, env


def prepare(root: Path, env: dict[str, str], run=subprocess.run):
    host = subprocess.check_output(["rustc", "-vV"], text=True, env=env)
    host = next(line.removeprefix("host: ") for line in host.splitlines() if line.startswith("host: "))
    command, build_env = package_build_configuration(
        root, env, host, root / "target/sysroot_release/preparation", prepare_only=True)
    run(command, cwd=root, env=build_env, check=True)


if __name__ == "__main__":
    prepare(Path(__file__).resolve().parents[3], dict(os.environ, CARGO_NET_OFFLINE="true"))
