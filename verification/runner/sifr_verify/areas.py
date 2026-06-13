"""Verification area discovery."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .errors import DiscoveryError
from .paths import AREAS_DIR, REPO_ROOT
from .schemas import load_json, load_schema, validate_data


@dataclass(frozen=True)
class Area:
    name: str
    owner: str
    manifest_path: Path
    parallel_safe: bool
    resource_classes: tuple[str, ...]


def discover_areas(areas_dir: Path = AREAS_DIR) -> list[Area]:
    if not areas_dir.exists():
        return []
    if not areas_dir.is_dir():
        raise DiscoveryError(f"areas path is not a directory: {areas_dir}")

    schema = load_schema("area.schema.json")
    areas: list[Area] = []
    seen: set[str] = set()
    for manifest_path in sorted(areas_dir.glob("*/manifest.json")):
        payload = load_json(manifest_path)
        source = _display_path(manifest_path)
        validate_data(payload, schema, source=source)
        name = payload["name"]
        if name in seen:
            raise DiscoveryError(f"duplicate verification area name: {name}")
        if manifest_path.parent.name != name:
            raise DiscoveryError(
                f"area manifest name '{name}' must match directory '{manifest_path.parent.name}'"
            )
        seen.add(name)
        areas.append(
            Area(
                name=name,
                owner=payload["owner"],
                manifest_path=manifest_path,
                parallel_safe=payload["parallel_safe"],
                resource_classes=tuple(payload.get("resource_classes", [])),
            )
        )
    return areas


def _display_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)
