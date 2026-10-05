"""Exact packaged compiler preparation, separate from timed installed assertions."""
from __future__ import annotations
import os
from pathlib import Path
import subprocess
from producer_snapshot import prepare_source_snapshot

RELEASE_VERSION = "0.1.0-beta.1300"


def host_target(environment: dict[str, str]) -> str:
    identity = subprocess.check_output(["rustc", "-vV"], text=True, env=environment)
    return next(line.removeprefix("host: ") for line in identity.splitlines() if line.startswith("host: "))


def package_compiler_path(root: Path, environment: dict[str, str], host: str) -> Path:
    command, env = package_build_configuration(root, environment, host,
                                               root / "target/sysroot_release/preparation", prepare_only=True)
    target = command[command.index("--target") + 1]
    return Path(env["CARGO_TARGET_DIR"]) / target / "release/sifr"


def package_build_configuration(root: Path, original: dict[str, str], host: str,
                                artifact_dir: Path, *, prepare_only: bool = False):
    env = dict(original, CARGO_TARGET_DIR=str((root / "target/sysroot_release/cargo-target").resolve()),
               SIFR_RELEASE_VERSION=RELEASE_VERSION)
    command = ["scripts/distribution/build_release_artifacts.sh", "--version", RELEASE_VERSION,
               "--output-dir", str(artifact_dir), "--target", host, "--cargo-build"]
    if prepare_only:
        command.append("--prepare-only")
    return command, env



def corpus_configuration(root: Path, original: dict[str, str]):
    snapshot = root / "target/sysroot_release/corpus-producer-source"
    env = dict(original, SIFR_RELEASE_VERSION=RELEASE_VERSION, SIFR_SYSROOT=str(snapshot))
    command = ["cargo", "test", "--locked", "--offline", "-p", "sifr_driver", "--lib",
               "full_corpus_exact_emission", "--", "--ignored", "--nocapture"]
    return command, env, snapshot


def prepare(root: Path, env: dict[str, str], run=subprocess.run):
    host = host_target(env)
    command, build_env = package_build_configuration(
        root, env, host, root / "target/sysroot_release/preparation", prepare_only=True)
    run(command, cwd=root, env=build_env, check=True)
    corpus, corpus_env, snapshot = corpus_configuration(root, env)
    prepare_source_snapshot(root, snapshot, RELEASE_VERSION)
    # Preserve the selected library graph; filters execute only after preparation.
    run([*corpus[:7], "--no-run"], cwd=root, env=corpus_env, check=True)
    # metadata-structural is a distinct source-version test configuration.
    # Prepare it explicitly rather than cold-build it during assertions.
    run(["cargo", "test", "--locked", "--offline", "-p", "sifr_driver", "--no-run",
         "metadata_structural_"], cwd=root, env=env, check=True)


if __name__ == "__main__":
    prepare(Path(__file__).resolve().parents[3], dict(os.environ, CARGO_NET_OFFLINE="true"))
