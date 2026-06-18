from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class PackageEntry:
    name: str
    tier: str
    groups: tuple[str, ...]
    gate: str | None = None
    native: bool = False
    host_dependent: bool = False


def load_matrix(path: Path) -> list[PackageEntry]:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    entries = data.get("packages")
    if not isinstance(entries, list):
        raise ValueError(f"{path} must contain a [[packages]] array")
    return [parse_entry(path, index, entry) for index, entry in enumerate(entries)]


def parse_entry(path: Path, index: int, entry: object) -> PackageEntry:
    if not isinstance(entry, dict):
        raise ValueError(f"{path} package entry {index} must be a table")
    name = required_string(path, index, entry, "name")
    tier = required_string(path, index, entry, "tier")
    groups = required_string_list(path, index, entry, "groups")
    gate = optional_string(path, index, entry, "gate")
    native = optional_bool(path, index, entry, "native")
    host_dependent = optional_bool(path, index, entry, "host-dependent")
    return PackageEntry(name, tier, tuple(groups), gate, native, host_dependent)


def required_string(path: Path, index: int, entry: dict[str, object], key: str) -> str:
    value = entry.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{path} package entry {index} field {key} must be a non-empty string")
    return value


def required_string_list(path: Path, index: int, entry: dict[str, object], key: str) -> list[str]:
    value = entry.get(key)
    if not isinstance(value, list) or not value:
        raise ValueError(f"{path} package entry {index} field {key} must be a non-empty list")
    if not all(isinstance(item, str) and item for item in value):
        raise ValueError(f"{path} package entry {index} field {key} must contain strings")
    return value


def optional_bool(path: Path, index: int, entry: dict[str, object], key: str) -> bool:
    value = entry.get(key, False)
    if not isinstance(value, bool):
        raise ValueError(f"{path} package entry {index} field {key} must be boolean")
    return value


def optional_string(path: Path, index: int, entry: dict[str, object], key: str) -> str | None:
    value = entry.get(key)
    if value is None:
        return None
    if not isinstance(value, str) or not value:
        raise ValueError(f"{path} package entry {index} field {key} must be a non-empty string")
    return value
