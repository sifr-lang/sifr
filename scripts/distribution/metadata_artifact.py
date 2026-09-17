#!/usr/bin/env python3
"""Prepare and bind canonical metadata to an executable without running foreign binaries."""
from __future__ import annotations
import argparse
import hashlib
import json
import platform
import re
import subprocess
from pathlib import Path

METADATA_PATH = "lib/sifr/stdlib.sifrmeta"
DESCRIPTOR_PATH = "lib/sifr/stdlib.metadata.json"
FIELDS = {"schema_version", "compiler_identity", "semantic_target", "semantic_target_id",
          "stdlib_inputs_id", "metadata_id", "compiler_binary_sha256"}


def file_digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def host_target() -> str:
    machine = {"arm64": "aarch64", "aarch64": "aarch64", "x86_64": "x86_64"}.get(platform.machine())
    system = {"Darwin": "apple-darwin", "Linux": "unknown-linux-gnu"}.get(platform.system())
    if machine is None or system is None:
        raise ValueError("metadata preparation requires a supported native qualification host")
    return f"{machine}-{system}"


def semantic_target_id(target: str) -> str:
    digest = hashlib.sha256()
    for name, value in [("domain", b"target-semantic-v1"), ("triple", target.encode()),
                        ("layout", b"pointer64-little-endian-v1")]:
        for part in (name.encode(), value):
            digest.update(len(part).to_bytes(8, "little"))
            digest.update(part)
    return digest.hexdigest()


def require_native_binary(path: Path, allow_fixture_script: bool) -> str | None:
    with path.open("rb") as stream:
        prefix = stream.read(32)
    if prefix.startswith(b"#!") and allow_fixture_script:
        return  # The packaging API explicitly designates --binary as fixture mode.
    actual = None
    if len(prefix)>=20 and prefix[:4] == b"\x7fELF" and prefix[4:6] == b"\x02\x01" and prefix[7] in (0, 3):
        machine = {62: "x86_64", 183: "aarch64"}.get(int.from_bytes(prefix[18:20], "little"))
        if machine:
            actual = f"{machine}-unknown-linux-gnu"
    elif prefix[:4] == bytes.fromhex("cffaedfe"):
        machine = {0x01000007: "x86_64", 0x0100000C: "aarch64"}.get(int.from_bytes(prefix[4:8], "little"))
        if machine:
            actual = f"{machine}-apple-darwin"
    if actual != host_target():
        raise ValueError(f"metadata producer {path} is not native to {host_target()}; prepare this target on its native qualification host")

    return actual


def validate_metadata(metadata: bytes, descriptor: dict, binary_digest: str, target: str) -> None:
    if not isinstance(descriptor,dict) or set(descriptor) != FIELDS or type(descriptor.get("schema_version")) is not int or descriptor["schema_version"] != 1:
        raise ValueError("invalid metadata descriptor fields or schema")
    for field in FIELDS - {"schema_version", "semantic_target"}:
        if not isinstance(descriptor[field], str) or re.fullmatch("[0-9a-f]{64}", descriptor[field]) is None:
            raise ValueError(f"invalid metadata descriptor {field}")
    if descriptor["semantic_target"] != target or descriptor["semantic_target_id"] != semantic_target_id(target):
        raise ValueError("metadata semantic target does not match the package")
    if descriptor["compiler_binary_sha256"] != binary_digest:
        raise ValueError("metadata descriptor does not bind the packaged compiler bytes")
    if not 120 <= len(metadata) <= 256 * 1024 * 1024 or metadata[:8] != b"SIFRMETA":
        raise ValueError("missing or invalid bounded metadata container")
    if int.from_bytes(metadata[8:12], "little") != 1 or int.from_bytes(metadata[112:120], "little") != len(metadata):
        raise ValueError("incompatible or incomplete metadata container")
    count = int.from_bytes(metadata[12:16], "little")
    if count > 200_000 or 120 + count * 92 > len(metadata):
        raise ValueError("metadata directory exceeds its bounded container")
    for field, start in (("compiler_identity", 16), ("semantic_target_id", 48), ("stdlib_inputs_id", 80)):
        if metadata[start:start + 32].hex() != descriptor[field]:
            raise ValueError(f"metadata header mismatch: {field}")
    if hashlib.sha256(metadata).hexdigest() != descriptor["metadata_id"]:
        raise ValueError("metadata content digest mismatch")


def prepare(binary: Path, source_root: Path, package_root: Path, target: str,
            allow_fixture_script: bool = False, runner=subprocess.run) -> dict:
    binary = binary.resolve()
    native = require_native_binary(binary, allow_fixture_script)
    if native is not None and native != target:
        raise ValueError("packaged compiler target must match its native metadata producer")
    def invoke(command):
        try:
            return runner(command,check=True,capture_output=True,text=True)
        except subprocess.CalledProcessError as error:
            raise ValueError(f"canonical producer failed ({error.returncode}): {error.stderr or error.stdout or error}") from error
    identity = invoke([str(binary), "--print", "compiler-identity"]).stdout.strip()
    output = package_root / METADATA_PATH
    output.parent.mkdir(parents=True, exist_ok=True)
    result = invoke([str(binary), "sysroot", "build-metadata", "--source-root", str(source_root.resolve()),
                     "--output", str(output.resolve()), "--target", target])
    report = json.loads(result.stdout)
    metadata = output.read_bytes()
    if not isinstance(report, dict) or report.get("compiler_identity") != identity or report.get("semantic_target") != target:
        raise ValueError("canonical producer reported a different compiler or semantic target")
    descriptor = {"schema_version": 1, "compiler_identity": identity, "semantic_target": target,
                  "semantic_target_id": metadata[48:80].hex(), "stdlib_inputs_id": metadata[80:112].hex(),
                  "metadata_id": report.get("metadata_id"), "compiler_binary_sha256": file_digest(binary)}
    validate_metadata(metadata, descriptor, file_digest(package_root / "bin/sifr"), target)
    (package_root / DESCRIPTOR_PATH).write_text(json.dumps(descriptor, sort_keys=True, indent=2) + "\n")
    return descriptor


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--require-native-target")
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--source-root", type=Path)
    parser.add_argument("--package-root", type=Path)
    parser.add_argument("--target")
    parser.add_argument("--allow-fixture-script", action="store_true")
    args = parser.parse_args()
    if args.require_native_target:
        if args.require_native_target != host_target():
            raise ValueError(f"prepare {args.require_native_target} on its native qualification host; this host is {host_target()}")
        return
    if None in (args.binary, args.source_root, args.package_root, args.target):
        parser.error("--binary, --source-root, --package-root and --target are required")
    prepare(args.binary, args.source_root, args.package_root, args.target, args.allow_fixture_script)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"metadata preparation failed: {error}") from error
