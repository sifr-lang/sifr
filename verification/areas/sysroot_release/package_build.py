"""Exact packaged compiler preparation, separate from timed installed assertions."""
from __future__ import annotations
import os
from pathlib import Path
import subprocess
from graph_paths import graph_path
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
    env = dict(original, CARGO_TARGET_DIR=str(graph_path(root,original,"cargo-target")),
               SIFR_RELEASE_VERSION=RELEASE_VERSION)
    command = ["scripts/distribution/build_release_artifacts.sh", "--version", RELEASE_VERSION,
               "--output-dir", str(artifact_dir), "--target", host, "--cargo-build"]
    if prepare_only:
        command.append("--prepare-only")
    return command, env



def metadata_environment(root: Path, original: dict[str, str], suite: str):
    if suite not in {"metadata-corpus", "metadata-structural"}:
        raise ValueError("unknown metadata preparation suite")
    env = dict(original)
    if suite == "metadata-corpus":
        # Cargo build-script environment changes invalidate a shared graph even
        # when both configurations were prepared. Keep the release-versioned
        # corpus graph inside the caller's owned target, apart from development.
        target = Path(original.get("CARGO_TARGET_DIR", root / "target"))
        if not target.is_absolute():
            target = root / target
        env["CARGO_TARGET_DIR"] = str(target / "sysroot-metadata-corpus")
        env["SIFR_RELEASE_VERSION"] = RELEASE_VERSION
        env["SIFR_SYSROOT"] = str(root / "target/sysroot_release/corpus-producer-source")
    return env


def corpus_configuration(root: Path, original: dict[str, str]):
    snapshot = root / "target/sysroot_release/corpus-producer-source"
    env = metadata_environment(root, original, "metadata-corpus")
    command = ["cargo", "test", "--locked", "--offline", "-p", "sifr_driver", "--lib",
               "full_corpus_exact_emission", "--", "--ignored", "--nocapture"]
    return command, env, snapshot


def prepare_metadata(root: Path, env: dict[str, str], run=subprocess.run, *, suite=None):
    """Prepare the two library configurations after private graph retirement."""
    if suite not in (None,"metadata-corpus","metadata-structural"):
        raise ValueError("unknown metadata preparation suite")
    if suite in (None,"metadata-corpus"):
        corpus, corpus_env, snapshot = corpus_configuration(root, env)
        prepare_source_snapshot(root, snapshot, RELEASE_VERSION)
        run([*corpus[:7], "--no-run"], cwd=root, env=corpus_env, check=True)
    if suite in (None,"metadata-structural"):
        run(["cargo", "test", "--locked", "--offline", "-p", "sifr_driver", "--no-run",
             "metadata_structural_"], cwd=root, env=env, check=True)


def prepare_package(root: Path, env: dict[str, str], run=subprocess.run):
    host = host_target(env)
    command, build_env = package_build_configuration(
        root, env, host, root / "target/sysroot_release/preparation", prepare_only=True)
    run(command, cwd=root, env=build_env, check=True)


def prepare(root: Path, env: dict[str, str], run=subprocess.run):
    prepare_package(root, env, run)
    # Preserve the selected library graph; filters execute only after preparation.
    # metadata-structural is a distinct source-version test configuration.
    # Prepare it explicitly rather than cold-build it during assertions.
    prepare_metadata(root, env, run)


if __name__ == "__main__":
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package-only',action='store_true')
    parser.add_argument('--metadata-only',action='store_true')
    parser.add_argument('--metadata-suite',choices=('metadata-corpus','metadata-structural'))
    args=parser.parse_args()
    if args.package_only and (args.metadata_only or args.metadata_suite):
        parser.error('--package-only cannot be combined with metadata preparation')
    if args.package_only:
        prepare_package(Path(__file__).resolve().parents[3],dict(os.environ,CARGO_NET_OFFLINE='true'))
    elif args.metadata_suite:
        prepare_metadata(Path(__file__).resolve().parents[3],dict(os.environ,CARGO_NET_OFFLINE='true'),suite=args.metadata_suite)
    else:
        callback=prepare_metadata if args.metadata_only else prepare
        callback(Path(__file__).resolve().parents[3],dict(os.environ,CARGO_NET_OFFLINE='true'))
