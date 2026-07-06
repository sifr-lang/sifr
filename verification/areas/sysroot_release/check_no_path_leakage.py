#!/usr/bin/env python3
"""Scan release and generated artifacts for forbidden source/CI path leakage."""

from __future__ import annotations

import argparse
import os
import posixpath
import sys
import tarfile
import tempfile
from pathlib import Path, PurePosixPath

REPO_ROOT = Path(__file__).resolve().parents[3]
DEFAULT_FORBIDDEN_PATHS = (
    REPO_ROOT,
    Path("/home/runner/work/sifr/sifr"),
    Path("/Users/runner/work/sifr/sifr"),
    Path("/workspace/sifr"),
)
SKIPPED_DIR_NAMES = frozenset({".git", "target", "__pycache__"})


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", help="Files, directories, or .tar.gz archives to scan.")
    parser.add_argument(
        "--forbidden-path",
        action="append",
        default=[],
        help="Forbidden absolute path prefix or exact path string; can repeat.",
    )
    parser.add_argument("--self-test", action="store_true", help="Run scanner self-tests.")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.self_test:
        run_self_test()
        print("sysroot release path leakage self-test: PASS")
        return 0

    if not args.paths:
        raise SystemExit("at least one path or --self-test is required")

    forbidden = forbidden_needles(args.forbidden_path)
    failures: list[str] = []
    for raw_path in args.paths:
        scan_path(Path(raw_path), forbidden, failures)

    if failures:
        print("sysroot release path leakage: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print(f"sysroot release path leakage: PASS (paths={len(args.paths)})")
    return 0


def forbidden_needles(extra_paths: list[str]) -> tuple[bytes, ...]:
    values: list[str] = []
    for path in DEFAULT_FORBIDDEN_PATHS:
        values.extend(path_spellings(path))
    for raw in extra_paths:
        values.extend(path_spellings(Path(raw)))
        values.append(raw)
    return tuple(sorted({value.encode("utf-8") for value in values if value}))


def path_spellings(path: Path) -> list[str]:
    raw = str(path)
    resolved = str(path.resolve()) if path.exists() else raw
    return [raw, resolved, raw.replace(os.sep, "/"), resolved.replace(os.sep, "/")]


def scan_path(path: Path, forbidden: tuple[bytes, ...], failures: list[str]) -> None:
    if not path.exists():
        failures.append(f"{display_path(path)} does not exist")
        return
    if path.is_dir():
        for child in sorted(path.rglob("*")):
            if any(part in SKIPPED_DIR_NAMES for part in child.parts):
                continue
            if child.is_file():
                scan_file(child, forbidden, failures)
        return
    if path.name.endswith(".tar.gz") or path.suffix == ".tgz":
        scan_tar_archive(path, forbidden, failures)
        return
    scan_file(path, forbidden, failures)


def scan_file(path: Path, forbidden: tuple[bytes, ...], failures: list[str]) -> None:
    try:
        content = path.read_bytes()
    except OSError as error:
        failures.append(f"{display_path(path)} could not be read: {error}")
        return
    scan_bytes(display_path(path), content, forbidden, failures)


def scan_tar_archive(path: Path, forbidden: tuple[bytes, ...], failures: list[str]) -> None:
    try:
        archive = tarfile.open(path, "r:gz")
    except tarfile.TarError as error:
        failures.append(f"{display_path(path)} could not be opened as tar.gz: {error}")
        return
    with archive:
        for member in archive.getmembers():
            member_name = validate_member_name(path, member.name, failures)
            if member_name is None:
                continue
            scan_bytes(f"{display_path(path)}:{member_name}", member_name.encode("utf-8"), forbidden, failures)
            if not member.isfile():
                continue
            extracted = archive.extractfile(member)
            if extracted is None:
                failures.append(f"{display_path(path)}:{member_name} could not be read")
                continue
            scan_bytes(f"{display_path(path)}:{member_name}", extracted.read(), forbidden, failures)


def validate_member_name(path: Path, name: str, failures: list[str]) -> str | None:
    normalized = posixpath.normpath(name)
    if normalized == ".":
        return None
    pure = PurePosixPath(normalized)
    if pure.is_absolute() or ".." in pure.parts:
        failures.append(f"{display_path(path)} contains unsafe archive member {name!r}")
        return None
    return normalized


def scan_bytes(label: str, content: bytes, forbidden: tuple[bytes, ...], failures: list[str]) -> None:
    for needle in forbidden:
        if needle in content:
            failures.append(f"{label} contains forbidden path {needle.decode('utf-8', errors='replace')}")


def display_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-path-leakage-self-test.") as temp:
        root = Path(temp)
        clean = root / "clean.txt"
        clean.write_text("sysroot=/opt/sifr\n", encoding="utf-8")
        failures: list[str] = []
        scan_path(clean, (str(REPO_ROOT).encode("utf-8"),), failures)
        if failures:
            raise SystemExit(f"clean file was rejected: {failures}")

        leaking = root / "leaking.txt"
        leaking.write_text(f"source={REPO_ROOT}\n", encoding="utf-8")
        failures = []
        scan_path(leaking, (str(REPO_ROOT).encode("utf-8"),), failures)
        if not failures:
            raise SystemExit("leaking file was not rejected")

        archive_path = root / "leaking.tar.gz"
        member = root / "member.txt"
        member.write_text(f"path={REPO_ROOT}\n", encoding="utf-8")
        with tarfile.open(archive_path, "w:gz") as archive:
            archive.add(member, arcname="member.txt")
        failures = []
        scan_path(archive_path, (str(REPO_ROOT).encode("utf-8"),), failures)
        if not failures:
            raise SystemExit("leaking archive was not rejected")


if __name__ == "__main__":
    raise SystemExit(main())
