"""Lockfile validation helpers for Rust-interop scenario packages."""

from __future__ import annotations

import tomllib
from functools import lru_cache
from pathlib import Path
from typing import Any


# SQLx 0.9 query macros lock weak MySQL and SQLite edges. The exact backend
# fixture owns these entries because SQLite cannot share the Rusqlite lock.
SQLX_QUERY_MACRO_FIXTURE_ONLY_LOCK_PACKAGES = {
    (
        "chacha20",
        "0.10.1",
        "registry+https://github.com/rust-lang/crates.io-index",
        "d524456ba66e72eb8b115ff89e01e497f8e6d11d78b70b1aa13c0fbd97540a81",
    ),
    (
        "getrandom",
        "0.4.3",
        "registry+https://github.com/rust-lang/crates.io-index",
        "300e883d756b2e4ec94e02791f39b04b522276138852cfc41d9fb7e904106099",
    ),
    (
        "flume",
        "0.12.0",
        "registry+https://github.com/rust-lang/crates.io-index",
        "5e139bc46ca777eb5efaf62df0ab8cc5fd400866427e56c68b22e414e53bd3be",
    ),
    (
        "libsqlite3-sys",
        "0.37.0",
        "registry+https://github.com/rust-lang/crates.io-index",
        "b1f111c8c41e7c61a49cd34e44c7619462967221a6443b0ec299e0ac30cfb9b1",
    ),
    (
        "spin",
        "0.9.9",
        "registry+https://github.com/rust-lang/crates.io-index",
        "3763264f6b73151db08c50ff20d7d8a0b8796e021cdea7ceedad07b80155fa0e",
    ),
    (
        "sqlx-mysql",
        "0.9.0",
        "registry+https://github.com/rust-lang/crates.io-index",
        "90b8020fe17c5f2c245bfa2505d7ef59c5604839527c740266ad2214acebea27",
    ),
    (
        "sqlx-sqlite",
        "0.9.0",
        "registry+https://github.com/rust-lang/crates.io-index",
        "488e99c397a62007e4229aec669a179816339afc6d2620ca6fa420dbee2e982c",
    ),
    (
        "whoami",
        "2.1.3",
        "registry+https://github.com/rust-lang/crates.io-index",
        "626c4bac6755d76ffc12cb01b2eac751db1996b9e0041de9aa02c8c211ddc82c",
    ),
}


@lru_cache(maxsize=1)
def _load_root_lock(path: Path) -> tuple[dict[str, Any] | None, str | None]:
    if not path.is_file():
        return None, None
    try:
        return tomllib.loads(path.read_text(encoding="utf-8")), None
    except tomllib.TOMLDecodeError as error:
        return None, str(error)


def read_root_lock(
    failures: list[str],
    fixture_id: str,
    path: Path,
) -> dict[str, Any] | None:
    """Read the immutable repository lock once per checker process."""
    lock, error = _load_root_lock(path)
    if error is not None:
        failures.append(
            f"{fixture_id}: repository root/{path.name} is not valid TOML: {error}"
        )
    return lock


def require_root_lock_subset(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    scenario_lock: dict[str, Any],
    root_lock: dict[str, Any],
) -> None:
    """Require every external scenario package identity in the root lock."""
    root_packages = {
        (
            str(package.get("name")),
            str(package.get("version")),
            str(package.get("source")),
            str(package.get("checksum")),
        )
        for package in root_lock.get("package", [])
        if isinstance(package, dict) and package.get("source")
    }
    allowed_fixture_only = (
        SQLX_QUERY_MACRO_FIXTURE_ONLY_LOCK_PACKAGES
        if fixture_id == "ecosystem_backend_certification"
        and raw_path == "examples/backend_feature_package"
        else set()
    )
    for package in scenario_lock.get("package", []):
        if not isinstance(package, dict) or not package.get("source"):
            continue
        identity = (
            str(package.get("name")),
            str(package.get("version")),
            str(package.get("source")),
            str(package.get("checksum")),
        )
        if identity not in root_packages and identity not in allowed_fixture_only:
            failures.append(
                f"{fixture_id}: {raw_path}/Cargo.lock package "
                f"{identity[0]} {identity[1]} is not present in root Cargo.lock "
                "with the same source/checksum identity"
            )
