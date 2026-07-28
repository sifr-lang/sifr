"""Shared strict primitives for stable release-governance validators."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any, NoReturn

SCHEMA_VERSION = 2
TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
)
PRODUCTION_CREDENTIAL_NAMES = (
    "CLOUDFLARE_API_TOKEN",
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "VSCE_PAT",
    "SIFR_SITE_TOKEN",
    "SIFR_WEBSITE_ACTIONS_TOKEN",
)
BUILDERS = {
    "aarch64-apple-darwin": "macos-15",
    "x86_64-apple-darwin": "macos-15-intel",
    "aarch64-unknown-linux-gnu": "ubuntu-24.04-arm",
    "x86_64-unknown-linux-gnu": "ubuntu-24.04",
}
CHANNELS = ("alpha", "beta", "stable")
PREVIEW_VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta)\.[0-9]+$")
STABLE_VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
PLAN_ID_RE = re.compile(
    r"^stable-(?P<version>[0-9]+\.[0-9]+\.[0-9]+)-[0-9a-f]{12}$"
)
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
INCIDENT_ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]{2,63}$")
ARTIFACT_ID_RE = re.compile(r"^[a-z0-9][a-z0-9_.-]+$")


class GovernanceError(ValueError):
    """A governed artifact violates the canonical stable-release contract."""


def fail(location: str, message: str) -> NoReturn:
    raise GovernanceError(f"{location}: {message}")


def require_object(value: Any, location: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(location, "must be an object")
    return value


def require_array(value: Any, location: str) -> list[Any]:
    if not isinstance(value, list):
        fail(location, "must be an array")
    return value


def require_exact_keys(
    value: dict[str, Any],
    *,
    required: set[str],
    optional: set[str] = frozenset(),
    location: str,
) -> None:
    missing = sorted(required.difference(value))
    if missing:
        fail(location, f"missing required field(s): {', '.join(missing)}")
    unknown = sorted(set(value).difference(required | optional))
    if unknown:
        fail(location, f"unknown field(s): {', '.join(unknown)}")


def require_schema_v2(value: dict[str, Any], location: str = "$") -> None:
    schema_version = value.get("schema_version")
    if type(schema_version) is not int or schema_version != SCHEMA_VERSION:  # noqa: E721
        fail(location, "schema_version must be integer 2")


def require_nonempty_string(value: Any, location: str) -> str:
    if not isinstance(value, str) or not value:
        fail(location, "must be a non-empty string")
    return value


def require_enum(value: Any, allowed: set[str] | frozenset[str], location: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        fail(location, f"must be one of: {', '.join(sorted(allowed))}")
    return value


def require_positive_int(value: Any, location: str) -> int:
    if type(value) is not int or value < 1:  # noqa: E721
        fail(location, "must be a positive integer")
    return value


def require_sha256(value: Any, location: str) -> str:
    if not isinstance(value, str) or SHA256_RE.fullmatch(value) is None:
        fail(location, "must be a lowercase SHA-256 digest")
    if set(value) == {"0"}:
        fail(location, "must not be the zero SHA-256 digest")
    return value


def require_commit(value: Any, location: str) -> str:
    if not isinstance(value, str) or COMMIT_RE.fullmatch(value) is None:
        fail(location, "must be a lowercase 40-character commit SHA")
    if set(value) == {"0"}:
        fail(location, "must not be the zero commit")
    return value


def require_incident_id(value: Any, location: str) -> str:
    if not isinstance(value, str) or INCIDENT_ID_RE.fullmatch(value) is None:
        fail(location, "must be a lowercase incident identifier")
    return value


def require_artifact_id(value: Any, location: str) -> str:
    if not isinstance(value, str) or ARTIFACT_ID_RE.fullmatch(value) is None:
        fail(location, "must be a lowercase artifact identifier")
    return value


def require_plan_id(value: Any, version: str, location: str) -> str:
    if not isinstance(value, str):
        fail(location, "must be a stable release plan identifier")
    match = PLAN_ID_RE.fullmatch(value)
    if match is None:
        fail(location, "must match stable-X.Y.Z-<12 lowercase hex>")
    if match.group("version") != version:
        fail(location, "must name the plan version")
    return value


def version_channel(version: Any, location: str) -> str:
    if not isinstance(version, str):
        fail(location, "must be a version string")
    if STABLE_VERSION_RE.fullmatch(version):
        return "stable"
    match = PREVIEW_VERSION_RE.fullmatch(version)
    if match is not None:
        return match.group(1)
    fail(location, "must be alpha, beta, or stable semver; rc is not supported")


def preview_version_order(version: Any, location: str) -> tuple[int, int, int, int]:
    """Return the numeric ordering key for a validated alpha or beta version."""
    channel = version_channel(version, location)
    if channel == "stable":
        fail(location, "must be an alpha or beta version")
    assert isinstance(version, str)
    core, sequence_text = version.rsplit(".", 1)
    base = core.split("-", 1)[0]
    major, minor, patch = (int(part) for part in base.split("."))
    return major, minor, patch, int(sequence_text)


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_canonical_json(path: Path, value: Any, *, refuse_existing: bool = False) -> None:
    if refuse_existing and path.exists():
        fail(str(path), "refusing to overwrite existing evidence")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_json_bytes(value))


def load_json_bytes_strict(
    raw: bytes,
    *,
    source: str,
    require_canonical: bool = False,
) -> Any:
    def object_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                fail(source, f"duplicate object key: {key}")
            result[key] = value
        return result

    try:
        value = json.loads(raw, object_pairs_hook=object_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise GovernanceError(f"{source}: invalid JSON: {exc}") from exc
    if require_canonical and raw != canonical_json_bytes(value):
        fail(source, "must use canonical JSON bytes")
    return value


def load_json_strict(path: Path, *, require_canonical: bool = False) -> Any:
    try:
        raw = path.read_bytes()
    except OSError as exc:
        raise GovernanceError(f"{path}: invalid JSON: {exc}") from exc
    return load_json_bytes_strict(
        raw,
        source=str(path),
        require_canonical=require_canonical,
    )
