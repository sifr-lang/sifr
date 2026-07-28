"""Installed self-update certification with isolated schema-v2 metadata."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Callable

FIXTURE_TARGET_VERSION = "0.1.0-beta.1301"


class CertificationError(Exception):
    """Raised when an installed release surface violates its contract."""


def write_install_receipt(
    install_root: Path,
    host: str,
    sysroot_payload: dict[str, Any],
    *,
    release_version: str,
) -> None:
    receipt = {
        "schema_version": 2,
        "name": "sifr",
        "version": release_version,
        "channel": "beta",
        "target": host,
        "install_dir": str(install_root / "bin"),
        "binary_path": str(install_root / "bin" / "sifr"),
        "sysroot_path": str(install_root),
        "sysroot_schema_version": 1,
        "sysroot_sifr_version": release_version,
        "sysroot_target_triple": host,
        "sysroot_content_sha256": str(sysroot_payload.get("sysroot_content_sha256")),
        "artifact": f"sifr-{release_version}-{host}.tar.gz",
        "modify_path": False,
    }
    (install_root / "install.json").write_text(
        json.dumps(receipt, indent=2, sort_keys=True),
        encoding="utf-8",
    )


def run_self_update_snapshots(
    installed_sifr: Path,
    install_root: Path,
    work_root: Path,
    env: dict[str, str],
    self_version_path: Path,
    self_update_dry_run_path: Path,
    metadata_path: Path,
    *,
    release_version: str,
    run_checked: Callable[..., Any],
) -> None:
    version = run_checked(
        [str(installed_sifr), "self", "version", "--format", "json"],
        cwd=work_root,
        env=env,
        label="self version json",
        stdout_path=self_version_path,
        echo_output=False,
    )
    validate_self_version_json(version.stdout, install_root, release_version)

    dry_run_env = env.copy()
    dry_run_env["SIFR_TEST_CHANNEL_METADATA_PATH"] = str(metadata_path)
    dry_run = run_checked(
        [
            str(installed_sifr),
            "self",
            "update",
            "--dry-run",
            "--version",
            FIXTURE_TARGET_VERSION,
            "--format",
            "json",
        ],
        cwd=work_root,
        env=dry_run_env,
        label="self update dry-run json",
        stdout_path=self_update_dry_run_path,
        echo_output=False,
    )
    validate_self_update_dry_run_json(
        dry_run.stdout,
        install_root,
        release_version,
    )


def write_self_update_metadata_fixture(temp_root: Path) -> Path:
    targets = {
        target: {
            "artifact_sha256": "a" * 64,
            "sysroot_content_sha256": "b" * 64,
        }
        for target in (
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "x86_64-unknown-linux-gnu",
        )
    }

    def release(channel: str) -> dict[str, Any]:
        return {
            "channel": channel,
            "status": "active",
            "source_commit": "c" * 40,
            "installer_sha256": "d" * 64,
            "targets": targets,
        }

    payload = {
        "schema_version": 2,
        "generation": 1,
        "ga_status": "preview",
        "channels": {
            "alpha": "0.1.0-alpha.1",
            "beta": FIXTURE_TARGET_VERSION,
        },
        "releases": {
            "0.1.0-alpha.1": release("alpha"),
            FIXTURE_TARGET_VERSION: release("beta"),
        },
    }
    path = temp_root / "self-update-channels.json"
    path.write_text(
        json.dumps(payload, separators=(",", ":"), sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return path


def validate_self_version_json(
    raw: str,
    install_root: Path,
    release_version: str,
) -> None:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise CertificationError(f"self version json did not parse: {error}") from error
    if (
        payload.get("current_version") != release_version
        or payload.get("receipt_version") != release_version
    ):
        raise CertificationError(
            "self version json did not preserve the installed receipt version"
        )
    require_installed_paths(payload, install_root, "self version json")
    if payload.get("matches_receipt") is not True:
        raise CertificationError("self version json did not report a matching receipt")


def validate_self_update_dry_run_json(
    raw: str,
    install_root: Path,
    release_version: str,
) -> None:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise CertificationError(
            f"self update dry-run json did not parse: {error}"
        ) from error
    if (
        payload.get("current_version") != release_version
        or payload.get("target_version") != FIXTURE_TARGET_VERSION
    ):
        raise CertificationError(
            "self update dry-run json did not preserve requested version transition"
        )
    if payload.get("action") != "update" or payload.get("would_run_installer") is not True:
        raise CertificationError("self update dry-run json did not plan an update action")
    require_installed_paths(payload, install_root, "self update dry-run json")


def require_installed_paths(
    payload: dict[str, Any],
    install_root: Path,
    label: str,
) -> None:
    if Path(str(payload.get("sysroot_path"))).resolve() != install_root.resolve():
        raise CertificationError(f"{label} did not preserve the installed sysroot path")
    expected_binary = (install_root / "bin" / "sifr").resolve()
    if Path(str(payload.get("binary_path"))).resolve() != expected_binary:
        raise CertificationError(f"{label} did not preserve the installed binary path")
