#!/usr/bin/env python3
"""Prepare and identify the actual compiler artifact before measurement."""

import argparse
import json
import os
import subprocess
import time
from pathlib import Path

from compiler_lanes import LANES, digest, validate_receipt

ROOT = Path(__file__).resolve().parents[3]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lane", choices=LANES, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    profile = LANES[args.lane]
    started = time.monotonic()
    command = ["cargo", "build", "--locked", "-p", "sifr", "--profile", profile,
               "--message-format=json-render-diagnostics"]
    with (output / "cargo.jsonl").open("w") as messages, (output / "cargo.log").open("w") as log:
        subprocess.run(command, cwd=ROOT, stdout=messages, stderr=log, check=True)
    artifacts = [
        row for line in (output / "cargo.jsonl").read_text().splitlines()
        if (row := json.loads(line)).get("reason") == "compiler-artifact"
        and row.get("target", {}).get("name") == "sifr"
        and row.get("executable")
    ]
    if len(artifacts) != 1:
        raise ValueError("Cargo must identify exactly one sifr executable")
    artifact = artifacts[0]
    binary = Path(artifact["executable"]).resolve()
    source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    if subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip():
        raise ValueError("prepare a committed clean candidate before producing measurement receipts")
    sysroot = {"path": str(ROOT), "kind": "source-tree"}
    if args.lane == "product-installed-optimized":
        target = next(
            line.split(": ", 1)[1] for line in
            subprocess.check_output(["rustc", "-vV"], text=True).splitlines()
            if line.startswith("host: ")
        )
        packages = output / "packages"
        package_command = [
            str(ROOT / "scripts/distribution/build_release_artifacts.sh"),
            "--version", "0.0.0", "--output-dir", str(packages),
            "--binary", str(binary), "--sysroot-root", str(ROOT), "--target", target,
        ]
        with (output / "package.log").open("w") as log:
            subprocess.run(package_command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
        archives = list(packages.glob("*.tar.gz"))
        if len(archives) != 1:
            raise ValueError("packager did not produce one toolchain")
        installed = output / "installed"
        installed.mkdir()
        subprocess.run(["tar", "-xzf", str(archives[0]), "-C", str(installed)], check=True)
        selected = installed / "bin/sifr"
        if digest(selected) != digest(binary):
            raise ValueError("packaged compiler differs from Cargo artifact")
        binary = selected
        sysroot = {
            "path": str(installed), "kind": "installed",
            "manifest_sha256": digest(installed / "sysroot.toml"),
            "archive_sha256": digest(archives[0]),
        }
    receipt = {
        "schema_version": 1, "lane": args.lane,
        "compiler_build_profile": profile,
        "cargo_artifact_profile": artifact["profile"],
        "artifact": {"path": str(binary), "sha256": digest(binary)},
        "source_commit": source,
        "embedded_compatibility_identity": {"status": "unavailable-before-DX.2",
                                            "version": "0.0.0"},
        "sysroot": sysroot,
        "cargo_lock_sha256": digest(ROOT / "Cargo.lock"),
        "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
        "preparation": {"command": command, "elapsed_seconds": time.monotonic() - started,
                        "cargo_messages_sha256": digest(output / "cargo.jsonl"),
                        "cargo_jobs": os.environ.get("CARGO_BUILD_JOBS", "cargo-default")},
    }
    validate_receipt(ROOT, args.lane, receipt)
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(output / "receipt.json")


if __name__ == "__main__":
    main()
