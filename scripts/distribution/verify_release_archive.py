#!/usr/bin/env python3
"""Verify a Sifr preview release archive contains one complete toolchain."""

from __future__ import annotations

import argparse
import hashlib
import posixpath
import re
import tarfile
from pathlib import PurePosixPath


REQUIRED_FILES = (
    "bin/sifr",
    "Cargo.toml",
    "Cargo.lock",
    "sysroot.toml",
    ".cargo/config.toml",
    "crates/sifr_runtime/Cargo.toml",
    "crates/sifr_stdlib/Cargo.toml",
)

REQUIRED_DIR_PREFIXES = (
    "lib/sifr/stdlib/sifr/",
    "lib/sifr/stdlib/_sifr/",
    "crates/sifr_runtime/",
    "crates/sifr_stdlib/",
    "vendor/",
)

SYSROOT_FIELD_RE = re.compile(r'^"(?P<key>[^"]+)"\s*=\s*(?P<value>"[^"]*"|[0-9]+)\s*$')
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
ZERO_SHA256 = "0" * 64


def normalized_member_name(name: str) -> str:
    normalized = posixpath.normpath(name)
    if normalized == ".":
        return ""
    return normalized


def validate_member(member: tarfile.TarInfo) -> str:
    name = normalized_member_name(member.name)
    path = PurePosixPath(name)
    if not name and member.isdir():
        return ""
    if not name or path.is_absolute() or ".." in path.parts:
        raise SystemExit(f"unsafe archive member path: {member.name!r}")
    if any(part.startswith("._") for part in path.parts):
        raise SystemExit(f"archive member must not contain AppleDouble metadata: {name}")
    if member.issym() or member.islnk():
        raise SystemExit(f"archive member must not be a link: {name}")
    if not (member.isfile() or member.isdir()):
        raise SystemExit(f"archive member must be a regular file or directory: {name}")
    return name.rstrip("/")


def validate_manifest(
    manifest_source: str,
    sysroot_file_digests: dict[str, str],
    version: str,
    target: str,
) -> None:
    manifest = parse_sysroot_manifest(manifest_source)
    if manifest.get("schema-version") != 1:
        raise SystemExit("sysroot.toml schema-version must be 1")
    if manifest.get("sifr-version") != version:
        raise SystemExit(
            f"sysroot.toml sifr-version must be {version}, got {manifest.get('sifr-version')!r}"
        )
    if manifest.get("target-triple") != target:
        raise SystemExit(
            f"sysroot.toml target-triple must be {target}, got {manifest.get('target-triple')!r}"
        )
    content_sha = manifest.get("sysroot-content-sha256")
    if not isinstance(content_sha, str) or SHA256_RE.fullmatch(content_sha) is None:
        raise SystemExit("sysroot.toml sysroot-content-sha256 must be a lowercase sha256 hex string")
    if content_sha == ZERO_SHA256:
        raise SystemExit("sysroot.toml sysroot-content-sha256 must not be the zero placeholder")
    actual_content_sha = sysroot_content_sha256(sysroot_file_digests)
    if content_sha != actual_content_sha:
        raise SystemExit(
            "sysroot.toml sysroot-content-sha256 mismatch: "
            f"expected {content_sha}, got {actual_content_sha}"
        )


def sysroot_content_sha256(sysroot_file_digests: dict[str, str]) -> str:
    digest = hashlib.sha256()
    for name in sorted(sysroot_file_digests):
        digest.update(name.encode("utf-8"))
        digest.update(b"\n")
        digest.update(sysroot_file_digests[name].encode("ascii"))
        digest.update(b"\n")
    return digest.hexdigest()


def is_sysroot_content_path(name: str) -> bool:
    return name in {"Cargo.toml", "Cargo.lock", ".cargo/config.toml"} or name.startswith(
        ("crates/", "lib/", "vendor/")
    )


def parse_sysroot_manifest(source: str) -> dict[str, object]:
    manifest: dict[str, object] = {}
    for raw_line in source.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        match = SYSROOT_FIELD_RE.match(line)
        if match is None:
            raise SystemExit(f"unsupported sysroot.toml line: {raw_line!r}")
        value = match.group("value")
        if value.startswith('"'):
            parsed: object = value[1:-1]
        else:
            parsed = int(value)
        manifest[match.group("key")] = parsed
    return manifest


def verify_archive(path: str, version: str, target: str) -> None:
    with tarfile.open(path, "r:gz") as archive:
        names: set[str] = set()
        name_to_member: dict[str, tarfile.TarInfo] = {}
        sysroot_file_digests: dict[str, str] = {}
        manifest_source: str | None = None
        for member in archive:
            name = validate_member(member)
            if not name:
                continue
            names.add(name)
            name_to_member[name] = member
            if member.isfile() and (name == "sysroot.toml" or is_sysroot_content_path(name)):
                extracted = archive.extractfile(member)
                if extracted is None:
                    raise SystemExit(f"archive member could not be read: {name}")
                content = extracted.read()
                if name == "sysroot.toml":
                    manifest_source = content.decode("utf-8")
                if is_sysroot_content_path(name):
                    sysroot_file_digests[name] = hashlib.sha256(content).hexdigest()

        for required in REQUIRED_FILES:
            member = name_to_member.get(required)
            if member is None or not member.isfile():
                raise SystemExit(f"missing required archive file: {required}")

        for prefix in REQUIRED_DIR_PREFIXES:
            if not any(name == prefix.rstrip("/") or name.startswith(prefix) for name in names):
                raise SystemExit(f"missing required archive directory: {prefix.rstrip('/')}")

        if not any(name.startswith("lib/sifr/stdlib/sifr/") and name.endswith(".sifr") for name in names):
            raise SystemExit("stdlib public root contains no .sifr files")
        if not any(
            name.startswith("lib/sifr/stdlib/_sifr/") and name.endswith(".sifr") for name in names
        ):
            raise SystemExit("stdlib private root contains no .sifr files")

        if manifest_source is None:
            raise SystemExit("sysroot.toml could not be read from archive")
        validate_manifest(manifest_source, sysroot_file_digests, version, target)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive")
    parser.add_argument("--version", required=True)
    parser.add_argument("--target", required=True)
    args = parser.parse_args()
    verify_archive(args.archive, args.version, args.target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
