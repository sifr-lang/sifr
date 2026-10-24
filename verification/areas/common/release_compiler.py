"""One real release compiler for installed and distribution verification."""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys

RELEASE_VERSION = "0.1.0-beta.1300"
INSTALLED_PACKAGE_SUITES = frozenset({
    "boundary-equivalence", "host-installed-smoke",
    "host-installed-stdlib-heavy", "metadata-corpus",
})


def release_compiler_owners(profile):
    required = {"sysroot_release": INSTALLED_PACKAGE_SUITES,
                "distribution_release": {"representative", "full"}}
    return tuple(dict.fromkeys(
        "area_" + selection["area"]
        for selection in profile.get("selected_areas", [])
        if required.get(selection["area"], set()).intersection(selection["suites"])
    ))


def release_target_dir(root: Path) -> Path:
    """Keep packaged-version graphs separate from normal development builds."""
    return (root / "target/sysroot_release/cargo-target").resolve()


def package_build_configuration(root: Path, original: dict[str, str], host: str,
                                artifact_dir: Path, *, prepare_only: bool = False):
    env = dict(original, CARGO_TARGET_DIR=str(release_target_dir(root)),
               SIFR_RELEASE_VERSION=RELEASE_VERSION, CARGO_NET_OFFLINE="true")
    command = ["scripts/distribution/build_release_artifacts.sh", "--version", RELEASE_VERSION,
               "--output-dir", str(artifact_dir), "--target", host, "--cargo-build"]
    if prepare_only:
        command.append("--prepare-only")
    return command, env


def prepare_release_compiler(root: Path, env: dict[str, str], run=subprocess.run) -> Path:
    host_output = subprocess.check_output(["rustc", "-vV"], text=True, env=env)
    hosts = [line.removeprefix("host: ") for line in host_output.splitlines()
             if line.startswith("host: ")]
    if len(hosts) != 1 or not hosts[0]:
        raise RuntimeError("rustc did not report one host target")
    host = hosts[0]
    command, build_env = package_build_configuration(
        root, env, host, root / "target/sysroot_release/preparation", prepare_only=True)
    # Cargo remains the freshness authority on every request. Route build output
    # to stderr so --print-binary has exactly one machine-readable stdout line.
    run(command, cwd=root, env=build_env, check=True, stdout=sys.stderr)
    binary = Path(build_env["CARGO_TARGET_DIR"]) / host / "release/sifr"
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise RuntimeError(f"prepared release compiler is not executable: {binary}")
    version = subprocess.check_output([str(binary), "--version"], cwd=root,
                                      env=build_env, text=True).strip()
    if version != f"sifr {RELEASE_VERSION}":
        raise RuntimeError(f"prepared release compiler version mismatch: {version!r}")
    return binary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--print-binary", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    binary = prepare_release_compiler(root, dict(os.environ))
    if args.print_binary:
        print(binary)


if __name__ == "__main__":
    main()
