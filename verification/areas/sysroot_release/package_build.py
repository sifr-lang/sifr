"""Prepare corpus verification from the existing driver libtest and release CLI graphs."""
from __future__ import annotations
import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from producer_snapshot import prepare_source_snapshot
from corpus_preparation import host_triple, receipt_path, record_prepared, require_prepared
from corpus_profiles import prepare_development, run_development, compare_profiles

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from verification.areas.common.release_compiler import (
    RELEASE_VERSION, package_build_configuration, prepare_release_compiler, release_target_dir,
)


def corpus_configuration(root: Path, original: dict[str, str]):
    snapshot = root / "target/sysroot_release/corpus-producer-source"
    command = [sys.executable, str(root / "verification/areas/sysroot_release/package_build.py"),
               "--run-corpus"]
    return command, dict(original), snapshot


def corpus_environments(root: Path, original: dict[str, str], snapshot: Path):
    development = dict(original, SIFR_SYSROOT=str(root))
    release = dict(original, CARGO_TARGET_DIR=str(release_target_dir(root)),
                   SIFR_RELEASE_VERSION=RELEASE_VERSION, SIFR_SYSROOT=str(snapshot))
    return development, release


def prepare(root: Path, env: dict[str, str], run=subprocess.run, *, corpus_only=False):
    receipt_path(root).unlink(missing_ok=True)
    if not corpus_only:
        prepare_release_compiler(root, env, run)
    _, _, snapshot = corpus_configuration(root, env)
    development_env, release_env = corpus_environments(root, env, snapshot)
    host = host_triple(release_env)
    command, build_env = package_build_configuration(
        root, release_env, host, root / "target/sysroot_release/preparation", prepare_only=True)
    # The release owner applies the identical version, remaps, target and profile.
    # Both packages are selected so the oracle cannot narrow shared dependency features.
    run([*command, "--prepare-corpus"], cwd=root, env=build_env, check=True)
    development = prepare_development(root, development_env, run)
    prepare_source_snapshot(root, snapshot, RELEASE_VERSION)
    record_prepared(root, env, snapshot, host, development)


def run_corpus(root: Path, env: dict[str, str]):
    _, _, snapshot = corpus_configuration(root, env)
    artifacts = require_prepared(root, env, snapshot)
    development_env, release_env = corpus_environments(root, env, snapshot)
    fixtures = str(root / "crates/sifr/tests/e2e/pass")
    # Own both new reports so a failed run cannot consume a previous passing one.
    parent = root / "target/sysroot_release"
    output = Path(env["SIFR_DX8_CORPUS_OUTPUT"]) if env.get("SIFR_DX8_CORPUS_OUTPUT") else None
    base = Path(tempfile.mkdtemp(prefix="corpus-profiles-", dir=parent))
    print(f"corpus-profile evidence: {base}", flush=True)
    development_output = base / "development"
    release_output = base / "release"
    for environment, destination in ((development_env, development_output),
                                     (release_env, release_output)):
        environment["SIFR_DX8_CORPUS_OUTPUT"] = str(destination)
        environment.pop("SIFR_DX8_CORPUS_CASES", None)
        environment.pop("SIFR_DX8_SEMANTIC_TARGET", None)
    # This is the existing ignored test, executed once under its selected area owner.
    # A standalone native selection without that owner continues to execute it itself.
    run_development(artifacts["development"]["driver_test"], root, development_env)
    subprocess.run([artifacts["oracle"], "--fixtures", fixtures],
                   cwd=root, env=release_env, check=True)
    compare_profiles(root, development_output, release_output)
    if output is not None:
        import shutil
        # Reference publication follows both successful current executions.
        output.mkdir(parents=True, exist_ok=True)
        for path in release_output.iterdir():
            shutil.copy2(path, output / path.name)


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
