"""Versioned, provider-independent release evidence archive contracts.

An archive authenticates custody, never live qualification or publication approval.
The manifest digest must be pinned outside the archive before readback.
"""

from __future__ import annotations

import copy
import os
import stat
import unicodedata
from pathlib import Path
from typing import Any, Callable

from verification.json_schema_202012 import JsonSchemaError, validate_instance

from .common import (
    GovernanceError, canonical_json_bytes, load_json_bytes_strict,
    require_commit, require_exact_keys, require_object, require_sha256, sha256_bytes,
)

SCHEMA = Path(__file__).resolve().parents[1] / "schemas/release_evidence_archive.schema.json"
WORKFLOW = ".github/workflows/release-qualification.yml"
SKIP_TARGETS = ("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu")


def safe_path(value: str) -> str:
    """Use one portable spelling; reject aliases before touching payload bytes."""
    if (not isinstance(value, str) or not value or value.startswith("/")
            or "\\" in value or ":" in value
            or any(ord(char) < 32 for char in value)
            or unicodedata.normalize("NFC", value) != value
            or any(part in {"", ".", ".."} or part.endswith((".", " "))
                   for part in value.split("/"))):
        raise GovernanceError(f"unsafe archive path: {value!r}")
    return value


def check_paths(paths: list[str]) -> None:
    seen: set[str] = set()
    for path in paths:
        normalized = safe_path(path).casefold()
        if normalized in seen:
            raise GovernanceError(f"conflicting normalized path: {path}")
        seen.add(normalized)
    for path in seen:
        parts = path.split("/")
        if any("/".join(parts[:i]) in seen for i in range(1, len(parts))):
            raise GovernanceError(f"file/directory path conflict: {path}")


def read_local(root: Path, relative: str) -> bytes:
    """Descriptor-relative no-follow traversal, including all root ancestors."""
    safe_path(relative)
    root = Path(os.path.abspath(root))
    descriptors: list[int] = []
    try:
        descriptor = os.open(root.anchor, os.O_RDONLY | os.O_DIRECTORY)
        descriptors.append(descriptor)
        for part in (*root.parts[1:], *relative.split("/")[:-1]):
            descriptor = os.open(
                part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=descriptor
            )
            descriptors.append(descriptor)
        descriptor = os.open(
            relative.split("/")[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
            dir_fd=descriptor,
        )
        descriptors.append(descriptor)
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            raise GovernanceError(f"not a regular archive file: {relative}")
        with os.fdopen(os.dup(descriptor), "rb") as stream:
            return stream.read()
    except OSError as exc:
        raise GovernanceError(f"cannot safely read {relative}: {exc}") from exc
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)


def archive_root(identity: dict[str, Any]) -> str:
    return (f"sifr/{identity['source_commit']}/{identity['run_id']}/"
            f"{identity['run_attempt']}/{identity['index_sha256']}/")


def inventory_of(manifest: dict[str, Any]) -> dict[str, Any]:
    inventory = copy.deepcopy({key: manifest[key] for key in ("schema_version", "identity", "matrix", "artifacts")})
    for artifact in inventory["artifacts"]:
        artifact.pop("key")
    return inventory


def construct_manifest(inventory: Any) -> dict[str, Any]:
    inventory = require_object(inventory, "inventory")
    require_exact_keys(inventory, required={"schema_version", "identity", "matrix", "artifacts"},
                       location="inventory")
    manifest = copy.deepcopy(inventory)
    if not isinstance(manifest["artifacts"], list) or not isinstance(manifest["matrix"], list):
        raise GovernanceError("inventory artifacts and matrix must be arrays")
    for record in manifest["artifacts"]:
        require_exact_keys(require_object(record, "artifact"),
                           required={"role", "path", "size_bytes", "sha256", "producer"},
                           location="artifact")
    identity = require_object(manifest["identity"], "identity")
    if not {"source_commit", "run_id", "run_attempt", "index_sha256"} <= identity.keys():
        raise GovernanceError("incomplete archive identity")
    manifest["archive_root"] = archive_root(identity)
    for record in manifest["artifacts"]:
        record["key"] = manifest["archive_root"] + f"objects/sha256/{record['sha256']}"
    try:
        manifest["artifacts"].sort(key=lambda record: record["role"])
        manifest["matrix"].sort(key=lambda row: row["target"])
    except (KeyError, TypeError) as exc:
        raise GovernanceError("invalid logical artifact or matrix identity") from exc
    manifest["inventory_sha256"] = sha256_bytes(canonical_json_bytes(inventory_of(manifest)))
    return validate_manifest(manifest)


def validate_manifest(manifest: Any) -> dict[str, Any]:
    try:
        validate_instance(manifest, SCHEMA)
    except JsonSchemaError as exc:
        raise GovernanceError(str(exc)) from exc
    identity = manifest["identity"]
    require_commit(identity["source_commit"], "archive source")
    for commit in identity["submodules"].values():
        require_commit(commit, "archive submodule")
    for field in ("index_sha256", "lock_sha256", "profile_sha256"):
        require_sha256(identity[field], field)
    check_paths(list(identity["submodules"]))
    root = archive_root(identity)
    if manifest["archive_root"] != root:
        raise GovernanceError("archive root identity mismatch")
    roles = [record["role"] for record in manifest["artifacts"]]
    if roles != sorted(set(roles)):
        raise GovernanceError("duplicate or unordered logical artifact records")
    check_paths([record["path"] for record in manifest["artifacts"]])
    for record in manifest["artifacts"]:
        require_sha256(record["sha256"], "artifact digest")
        if record["key"] != root + f"objects/sha256/{record['sha256']}":
            raise GovernanceError("content-addressed key mismatch")
        for field in ("repository", "source_commit", "workflow_path", "run_id", "run_attempt"):
            if record["producer"][field] != identity[field]:
                raise GovernanceError(f"{record['role']}: wrong producer {field}")
    if manifest["inventory_sha256"] != sha256_bytes(canonical_json_bytes(inventory_of(manifest))):
        raise GovernanceError("exact inventory digest mismatch")
    from .archive_bindings import validate_required_roles
    validate_required_roles(manifest)
    return manifest


def parse_manifest(raw: bytes, *, manifest_sha256: str, expected_source: str,
                   expected_run: int, expected_attempt: int) -> dict[str, Any]:
    require_sha256(manifest_sha256, "external manifest digest")
    if sha256_bytes(raw) != manifest_sha256:
        raise GovernanceError("untrusted manifest digest")
    manifest = validate_manifest(load_json_bytes_strict(raw, source="archive manifest", require_canonical=True))
    identity = manifest["identity"]
    if (type(expected_run) is not int or type(expected_attempt) is not int
            or (identity["source_commit"], identity["run_id"], identity["run_attempt"])
            != (expected_source, expected_run, expected_attempt)):
        raise GovernanceError("wrong expected source/run/attempt")
    return manifest


def verify_bytes(record: dict[str, Any], raw: bytes) -> bytes:
    if not isinstance(raw, bytes) or len(raw) != record["size_bytes"] or sha256_bytes(raw) != record["sha256"]:
        raise GovernanceError(f"{record['role']}: missing, mutated or truncated bytes")
    return raw


def verify_payloads(manifest: dict[str, Any], read: Callable[[dict[str, Any]], bytes]) -> None:
    validate_manifest(manifest)
    for record in manifest["artifacts"]:
        verify_bytes(record, read(record))
    from .archive_bindings import verify_bindings
    try:
        verify_bindings(manifest, lambda record: verify_bytes(record, read(record)))
    except (KeyError, TypeError, AttributeError) as exc:
        raise GovernanceError(f"malformed bound archive document: {exc}") from exc


def plan(inventory: Any, payload_root: Path) -> dict[str, Any]:
    manifest = construct_manifest(inventory)
    verify_payloads(manifest, lambda record: read_local(payload_root, record["path"]))
    return manifest


def verify_local(raw: bytes, *, payload_root: Path, **expectations: Any) -> dict[str, Any]:
    manifest = parse_manifest(raw, **expectations)
    verify_payloads(manifest, lambda record: read_local(payload_root, record["path"]))
    return manifest
