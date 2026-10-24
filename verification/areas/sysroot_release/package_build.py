"""Exact packaged compiler preparation, separate from timed installed assertions."""
from __future__ import annotations
import argparse
import os
from pathlib import Path
import subprocess
import sys
from producer_snapshot import prepare_source_snapshot

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from verification.areas.common.release_compiler import (
    RELEASE_VERSION, package_build_configuration, prepare_release_compiler, release_target_dir,
)


def corpus_configuration(root: Path, original: dict[str, str]):
    snapshot = root / "target/sysroot_release/corpus-producer-source"
    # This test embeds the packaged version. Sharing the development target
    # would invalidate its normal-version compiler and reverse on the next check.
    env = dict(original, CARGO_TARGET_DIR=str(release_target_dir(root)),
               SIFR_RELEASE_VERSION=RELEASE_VERSION, SIFR_SYSROOT=str(snapshot))
    command = ["cargo", "test", "--locked", "--offline", "-p", "sifr_driver", "--lib",
               "full_corpus_exact_emission", "--", "--ignored", "--nocapture"]
    return command, env, snapshot


def prepare(root: Path, env: dict[str, str], run=subprocess.run, *, corpus_only=False):
    if not corpus_only:
        prepare_release_compiler(root, env, run)
    corpus, corpus_env, snapshot = corpus_configuration(root, env)
    prepare_source_snapshot(root, snapshot, RELEASE_VERSION)
    # Preserve the selected library graph; filters execute only after preparation.
    run([*corpus[:7], "--no-run"], cwd=root, env=corpus_env, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus-only", action="store_true")
    args = parser.parse_args()
    prepare(Path(__file__).resolve().parents[3], dict(os.environ, CARGO_NET_OFFLINE="true"),
            corpus_only=args.corpus_only)
