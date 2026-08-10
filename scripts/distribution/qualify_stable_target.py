#!/usr/bin/env python3
"""Install and smoke one stable target archive, then emit canonical qualification evidence."""

from __future__ import annotations

import argparse
import json
import os
import platform
import shlex
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "distribution_release"
sys.path.insert(0, str(AREA_ROOT))

from governance import GovernanceError, validate_self_version  # noqa: E402
from governance.common import (  # noqa: E402
    BUILDERS,
    TARGETS,
    canonical_json_bytes,
    require_commit,
    sha256_file,
    version_channel,
    write_canonical_json,
)

sys.path.insert(0, str(REPO_ROOT / "scripts" / "distribution"))
from verify_release_archive import parse_sysroot_manifest, verify_archive  # noqa: E402


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--builder", required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--out-dir", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        report = qualify_target(
            archive=Path(args.archive),
            version=args.version,
            target=args.target,
            builder=args.builder,
            source_commit=args.source_commit,
            out_dir=Path(args.out_dir),
        )
    except (GovernanceError, OSError, subprocess.SubprocessError) as exc:
        print(f"stable-target-qualification: {exc}", file=sys.stderr)
        return 2
    print(
        "stable target qualification ok: "
        f"version={report['candidate_version']} target={report['target']}"
    )
    return 0


def qualify_target(
    *,
    archive: Path,
    version: str,
    target: str,
    builder: str,
    source_commit: str,
    out_dir: Path,
) -> dict[str, Any]:
    if version_channel(version, "--version") != "stable":
        raise GovernanceError("--version must be exact stable SemVer")
    if target not in TARGETS or BUILDERS[target] != builder:
        raise GovernanceError("--builder does not match the governed target matrix")
    host_target = current_host_target()
    if target != host_target:
        raise GovernanceError(
            f"--target {target} does not match the current host {host_target}"
        )
    require_commit(source_commit, "--source-commit")
    if not archive.is_file():
        raise GovernanceError(f"archive does not exist: {archive}")
    checksum_path = Path(f"{archive}.sha256")
    if not checksum_path.is_file():
        raise GovernanceError(f"checksum does not exist: {checksum_path}")
    expected_archive_sha = read_utf8(
        checksum_path,
        location="archive checksum",
    ).strip()
    archive_sha = sha256_file(archive)
    if expected_archive_sha != archive_sha:
        raise GovernanceError(f"{archive}: checksum mismatch")
    try:
        verify_archive(str(archive), version, target)
    except SystemExit as exc:
        raise GovernanceError(f"{archive}: archive verification failed: {exc}") from exc

    out_dir.mkdir(parents=True, exist_ok=True)
    report_path = out_dir / f"qualification-{target}.json"
    sysroot_bundle = out_dir / f"sifr-{version}-{target}-sysroot.tar.gz"
    if report_path.exists() or sysroot_bundle.exists():
        raise GovernanceError("refusing to overwrite stable qualification evidence")

    with tempfile.TemporaryDirectory(prefix="sifr-stable-target-") as directory:
        install_root = Path(directory) / "install"
        install_root.mkdir()
        with tarfile.open(archive, "r:gz") as source:
            source.extractall(install_root, filter="data")
        binary = install_root / "bin" / "sifr"
        sysroot_manifest = parse_sysroot_manifest(
            read_utf8(
                install_root / "sysroot.toml",
                location="installed sysroot manifest",
            )
        )
        version_output = run_checked(
            [str(binary), "--version"],
            env={"SIFR_SYSROOT": str(install_root)},
        ).strip()
        if version_output != f"sifr {version}":
            raise GovernanceError(
                f"{binary}: version mismatch (expected sifr {version}, got {version_output})"
            )
        smoke_source = Path(directory) / "qualification_smoke.sifr"
        smoke_source.write_text(
            'def main() -> None:\n    print("stable qualification")\n',
            encoding="utf-8",
        )
        generated_rust_env = {
            "SIFR_SYSROOT": str(install_root),
            "SIFR_RUST_BRIDGE_PROBE_CACHE_DIR": str(
                Path(directory) / "generated-rust-probe-cache"
            ),
            "CARGO_TARGET_DIR": str(Path(directory) / "generated-rust-cargo-target"),
        }
        run_checked(
            [str(binary), "check", str(smoke_source)],
            env={"SIFR_SYSROOT": str(install_root)},
            cwd=smoke_source.parent,
        )
        for temperature in ("cold", "warm"):
            emitted = run_checked(
                [str(binary), "emit", str(smoke_source)],
                env=generated_rust_env,
                cwd=smoke_source.parent,
                timeout_seconds=90,
            )
            if "fn main" not in emitted:
                raise GovernanceError(
                    f"installed {temperature} generated Rust smoke returned no main function"
                )
        lsp_env = {
            "SIFR_SYSROOT": str(install_root),
            "SIFR_LSP_COMMAND": shlex.join([str(binary), "lsp", "--stdio"]),
            "SIFR_RUST_BRIDGE_PROBE_CACHE_DIR": str(
                Path(directory) / "lsp-generated-rust-probe-cache"
            ),
            "CARGO_TARGET_DIR": str(Path(directory) / "lsp-generated-rust-cargo-target"),
        }
        run_checked(
            [
                sys.executable,
                str(
                    REPO_ROOT
                    / "verification"
                    / "areas"
                    / "developer_tooling"
                    / "lsp_protocol_smoke.py"
                ),
                "--candidate-smoke",
            ],
            env=lsp_env,
            timeout_seconds=120,
        )
        receipt_dir = Path(directory) / "receipt"
        receipt_dir.mkdir()
        receipt = {
            "schema_version": 2,
            "name": "sifr",
            "version": version,
            "channel": "stable",
            "target": target,
            "install_dir": str(binary.parent),
            "binary_path": str(binary),
            "sysroot_path": str(install_root),
            "sysroot_schema_version": sysroot_manifest["schema-version"],
            "sysroot_sifr_version": sysroot_manifest["sifr-version"],
            "sysroot_target_triple": sysroot_manifest["target-triple"],
            "sysroot_content_sha256": sysroot_manifest["sysroot-content-sha256"],
            "artifact": archive.name,
            "modify_path": False,
        }
        (receipt_dir / "install.json").write_bytes(canonical_json_bytes(receipt))
        self_version_bytes = run_checked(
            [str(binary), "self", "version", "--format", "json"],
            env={
                "SIFR_SYSROOT": str(install_root),
                "SIFR_INSTALL_MANIFEST_DIR": str(receipt_dir),
            },
        ).encode()
        self_version = json.loads(self_version_bytes)
        validate_self_version(self_version)
        if (
            self_version["current_version"] != version
            or self_version["receipt_version"] != version
            or self_version["channel"] != "stable"
            or self_version["target"] != target
        ):
            raise GovernanceError("self version evidence disagrees with the candidate")
        with tarfile.open(sysroot_bundle, "w:gz") as destination:
            for path in sorted(install_root.rglob("*")):
                relative = path.relative_to(install_root)
                if relative.parts[0] == "bin":
                    continue
                destination.add(path, arcname=relative, recursive=False)

        report = {
            "schema_version": 2,
            "kind": "stable-target-qualification",
            "candidate_version": version,
            "source_commit": source_commit,
            "target": target,
            "builder": builder,
            "binary_sha256": sha256_file(binary),
            "sysroot_sha256": sysroot_manifest["sysroot-content-sha256"],
            "archive_sha256": archive_sha,
            "checksum_sha256": sha256_file(checksum_path),
            "sysroot_bundle_sha256": sha256_file(sysroot_bundle),
            "sifr_version": version,
            "installer_version": version,
            "receipt_channel": "stable",
            "sysroot_version": sysroot_manifest["sifr-version"],
            "sysroot_target": sysroot_manifest["target-triple"],
            "smoke_status": "pass",
            "self_version_sha256": sha256_bytes(self_version_bytes),
        }
        write_canonical_json(report_path, report, refuse_existing=True)
    return report


def run_checked(
    command: list[str],
    *,
    env: dict[str, str],
    cwd: Path = REPO_ROOT,
    timeout_seconds: float = 90,
) -> str:
    merged_env = os.environ.copy()
    merged_env.update(env)
    try:
        result = subprocess.run(
            command,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=merged_env,
            cwd=cwd,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as exc:
        raise GovernanceError(
            f"{' '.join(command)} exceeded the governed {timeout_seconds:g}s timeout"
        ) from exc
    if result.returncode != 0:
        raise GovernanceError(
            f"{' '.join(command)} failed with exit {result.returncode}: {result.stderr.strip()}"
        )
    return result.stdout


def sha256_bytes(value: bytes) -> str:
    import hashlib

    return hashlib.sha256(value).hexdigest()


def read_utf8(path: Path, *, location: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise GovernanceError(f"{location} is not readable UTF-8: {exc}") from exc


def current_host_target() -> str:
    system = platform.system()
    machine = platform.machine().lower()
    if system == "Darwin" and machine in {"arm64", "aarch64"}:
        return "aarch64-apple-darwin"
    if system == "Darwin" and machine == "x86_64":
        return "x86_64-apple-darwin"
    if system == "Linux" and machine == "x86_64":
        return "x86_64-unknown-linux-gnu"
    if system == "Linux" and machine in {"arm64", "aarch64"}:
        return "aarch64-unknown-linux-gnu"
    raise GovernanceError(f"unsupported stable qualification host: {system}/{machine}")


if __name__ == "__main__":
    raise SystemExit(main())
