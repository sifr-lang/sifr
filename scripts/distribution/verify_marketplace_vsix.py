#!/usr/bin/env python3
"""Verify exact Marketplace VSIX bytes and extension identity."""

from __future__ import annotations

import argparse
import json
import re
import sys
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError,
    fail,
    require_nonempty_string,
    sha256_file,
)

IDENTIFIER_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")
PACKAGE_JSON = "extension/package.json"


def verify_marketplace_vsix(
    *,
    vsix_path: Path,
    expected_sha256: str,
    publisher: str,
    extension: str,
    version: str,
) -> dict[str, str]:
    """Verify bytes, safe archive structure, and exact package metadata."""
    if not vsix_path.is_file() or vsix_path.is_symlink():
        fail("vsix_path", "must be a regular non-symlink file")
    if sha256_file(vsix_path) != expected_sha256:
        fail("expected_sha256", "does not match the VSIX bytes")
    for label, value in (
        ("publisher", publisher),
        ("extension", extension),
        ("version", version),
    ):
        require_nonempty_string(value, label)
    if IDENTIFIER_RE.fullmatch(publisher) is None:
        fail("publisher", "contains unsupported characters")
    if IDENTIFIER_RE.fullmatch(extension) is None:
        fail("extension", "contains unsupported characters")

    try:
        with zipfile.ZipFile(vsix_path) as archive:
            package_entries = []
            names: set[str] = set()
            for info in archive.infolist():
                path = PurePosixPath(info.filename)
                if (
                    path.is_absolute()
                    or ".." in path.parts
                    or "\\" in info.filename
                    or info.filename in names
                ):
                    fail("vsix_path", "contains an unsafe or duplicate archive entry")
                names.add(info.filename)
                if info.is_dir():
                    continue
                unix_mode = info.external_attr >> 16
                if unix_mode & 0o170000 == 0o120000:
                    fail("vsix_path", "contains a symbolic link")
                if info.filename == PACKAGE_JSON:
                    package_entries.append(info)
            if len(package_entries) != 1:
                fail("vsix_path", f"must contain exactly one {PACKAGE_JSON}")
            package = _load_package(archive.read(package_entries[0]))
    except zipfile.BadZipFile:
        fail("vsix_path", "is not a valid VSIX ZIP archive")

    for field, expected in (
        ("publisher", publisher),
        ("name", extension),
        ("version", version),
    ):
        if package.get(field) != expected:
            fail(f"{PACKAGE_JSON}.{field}", f"must equal {expected}")
    return {
        "publisher": publisher,
        "extension": extension,
        "version": version,
        "vsix_sha256": expected_sha256,
    }


def _load_package(raw: bytes) -> dict[str, Any]:
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError):
        fail(PACKAGE_JSON, "must contain valid UTF-8 JSON")
    if not isinstance(value, dict):
        fail(PACKAGE_JSON, "must contain a JSON object")
    return value


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vsix", type=Path, required=True)
    parser.add_argument("--expected-sha256", required=True)
    parser.add_argument("--publisher", required=True)
    parser.add_argument("--extension", required=True)
    parser.add_argument("--version", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        verify_marketplace_vsix(
            vsix_path=args.vsix,
            expected_sha256=args.expected_sha256,
            publisher=args.publisher,
            extension=args.extension,
            version=args.version,
        )
    except GovernanceError as exc:
        print(f"Marketplace VSIX verification failed: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
