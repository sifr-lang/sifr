#!/usr/bin/env python3
"""Extract one GitHub Actions artifact ZIP without path or link traversal."""

from __future__ import annotations

import argparse
import shutil
import stat
import zipfile
from pathlib import Path, PurePosixPath


def extract_artifact(
    archive_path: Path,
    destination: Path,
    *,
    expected_uncompressed_bytes: int,
) -> None:
    if expected_uncompressed_bytes < 1:
        raise ValueError("expected uncompressed size must be positive")
    if destination.is_symlink() or not destination.is_dir():
        raise ValueError("destination must be an existing non-symlink directory")
    if any(destination.iterdir()):
        raise ValueError("destination must be empty")
    destination = destination.resolve()
    seen: set[str] = set()
    try:
        with zipfile.ZipFile(archive_path) as archive:
            planned: list[tuple[zipfile.ZipInfo, PurePosixPath, Path]] = []
            for member in archive.infolist():
                relative = _safe_member_path(member)
                normalized = relative.as_posix()
                if normalized in seen:
                    raise ValueError(f"duplicate artifact member: {normalized}")
                seen.add(normalized)
                target = destination.joinpath(*relative.parts)
                if not target.resolve().is_relative_to(destination):
                    raise ValueError(f"artifact member escapes destination: {normalized}")
                planned.append((member, relative, target))
            files = [member for member, _, _ in planned if not member.is_dir()]
            if not files:
                raise ValueError("artifact ZIP contains no files")
            if sum(member.file_size for member in files) != expected_uncompressed_bytes:
                raise ValueError("artifact ZIP uncompressed size does not match evidence")
            for member, relative, target in planned:
                if member.is_dir():
                    target.mkdir(parents=True, exist_ok=True)
                    continue
                target.parent.mkdir(parents=True, exist_ok=True)
                if target.exists() or target.is_symlink():
                    raise ValueError(
                        f"artifact member would overwrite: {relative.as_posix()}"
                    )
                with archive.open(member) as source, target.open("xb") as output:
                    shutil.copyfileobj(source, output)
    except (OSError, zipfile.BadZipFile) as exc:
        raise ValueError(f"invalid artifact ZIP: {exc}") from exc


def _safe_member_path(member: zipfile.ZipInfo) -> PurePosixPath:
    name = member.filename
    relative = PurePosixPath(name)
    if (
        not name
        or "\\" in name
        or relative.is_absolute()
        or any(part in {"", ".", ".."} for part in relative.parts)
    ):
        raise ValueError(f"unsafe artifact member path: {name!r}")
    mode = member.external_attr >> 16
    file_type = stat.S_IFMT(mode)
    if file_type not in {0, stat.S_IFREG, stat.S_IFDIR}:
        raise ValueError(f"artifact member is not a regular file/directory: {name}")
    if member.is_dir() != (file_type == stat.S_IFDIR) and file_type != 0:
        raise ValueError(f"artifact member type is inconsistent: {name}")
    return relative


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--expected-uncompressed-bytes", required=True, type=int)
    args = parser.parse_args()
    try:
        extract_artifact(
            args.archive,
            args.destination,
            expected_uncompressed_bytes=args.expected_uncompressed_bytes,
        )
    except ValueError as exc:
        parser.error(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
