"""Prepare the packaged corpus oracle using the canonical release libraries."""
from __future__ import annotations
import argparse
import os
from pathlib import Path
import subprocess
import sys
from producer_snapshot import prepare_source_snapshot
from corpus_preparation import host_triple, receipt_path, record_prepared, require_prepared

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from verification.areas.common.release_compiler import (
    RELEASE_VERSION, package_build_configuration, prepare_release_compiler, release_target_dir,
)


def corpus_configuration(root: Path, original: dict[str, str]):
    snapshot = root / "target/sysroot_release/corpus-producer-source"
    env = dict(original, CARGO_TARGET_DIR=str(release_target_dir(root)),
               SIFR_RELEASE_VERSION=RELEASE_VERSION, SIFR_SYSROOT=str(snapshot))
    command = [sys.executable, str(root / "verification/areas/sysroot_release/package_build.py"),
               "--run-corpus"]
    return command, env, snapshot


def prepare(root: Path, env: dict[str, str], run=subprocess.run, *, corpus_only=False):
    receipt_path(root).unlink(missing_ok=True)
    if not corpus_only:
        prepare_release_compiler(root, env, run)
    _, corpus_env, snapshot = corpus_configuration(root, env)
    host = host_triple(corpus_env)
    command, build_env = package_build_configuration(
        root, corpus_env, host, root / "target/sysroot_release/preparation", prepare_only=True)
    # The release owner applies the identical version, remaps, target and profile.
    # Both packages are selected so the oracle cannot narrow shared dependency features.
    run([*command, "--prepare-corpus"], cwd=root, env=build_env, check=True)
    prepare_source_snapshot(root, snapshot, RELEASE_VERSION)
    record_prepared(root, corpus_env, snapshot, host)


def run_corpus(root: Path, env: dict[str, str]):
    _, corpus_env, snapshot = corpus_configuration(root, env)
    binary = require_prepared(root, corpus_env, snapshot)
    subprocess.run([str(binary), "--fixtures", str(root / "crates/sifr/tests/e2e/pass")],
                   cwd=root, env=corpus_env, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--corpus-only", action="store_true")
    modes.add_argument("--run-corpus", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    env = dict(os.environ, CARGO_NET_OFFLINE="true")
    if args.run_corpus:
        run_corpus(root, env)
    else:
        prepare(root, env, corpus_only=args.corpus_only)
