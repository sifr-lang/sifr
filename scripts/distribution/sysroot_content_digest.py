#!/usr/bin/env python3
"""Compute the archive verifier's content identity from a staged sysroot."""
from __future__ import annotations

import argparse
import hashlib
import stat
from pathlib import Path

from verify_release_archive import sysroot_content_sha256

CONTENT_ROOTS = ("Cargo.toml", "Cargo.lock", ".cargo/config.toml", "crates", "lib", "vendor")


def content_file_digests(root: Path) -> dict[str, str]:
    """Read every governed regular file; never traverse links or special files."""
    digests: dict[str, str] = {}
    pending = [root / name for name in CONTENT_ROOTS]
    while pending:
        path = pending.pop()
        mode = path.lstat().st_mode
        if stat.S_ISDIR(mode):
            pending.extend(path.iterdir())
        elif stat.S_ISREG(mode):
            with path.open("rb") as source:
                digests[path.relative_to(root).as_posix()] = hashlib.file_digest(
                    source, "sha256"
                ).hexdigest()
        else:
            raise ValueError(f"sysroot content must be a regular file or directory: {path}")
    return digests


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    try:
        print(sysroot_content_sha256(content_file_digests(args.root)))
    except (OSError, ValueError) as error:
        raise SystemExit(f"cannot hash staged sysroot: {error}") from error


if __name__ == "__main__":
    main()
