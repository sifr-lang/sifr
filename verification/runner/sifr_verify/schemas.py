"""Small, explicit schema subset for committed verification data."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .errors import SchemaError
from .paths import REPO_ROOT, SCHEMAS_DIR

SUPPORTED_SCHEMA_KEYS = {
    "$schema",
    "title",
    "description",
    "type",
    "required",
    "properties",
    "items",
    "enum",
    "additionalProperties",
    "format",
}
SUPPORTED_TYPES = {"object", "array", "string", "integer", "number", "boolean"}
SUPPORTED_FORMATS = {"repo-relative-path"}


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def load_schema(name: str) -> dict[str, Any]:
    path = SCHEMAS_DIR / name
    payload = load_json(path)
    if not isinstance(payload, dict):
        raise SchemaError(f"schema must be a JSON object: {path}")
    validate_schema_contract(payload, path)
    return payload


def validate_all_committed_schemas(schema_dir: Path = SCHEMAS_DIR) -> list[str]:
    checked: list[str] = []
    for path in sorted(schema_dir.glob("*.schema.json")):
        payload = load_json(path)
        if not isinstance(payload, dict):
            raise SchemaError(f"schema must be a JSON object: {path}")
        validate_schema_contract(payload, path)
        checked.append(str(path.relative_to(REPO_ROOT)))
    return checked


def validate_schema_contract(schema: dict[str, Any], path: Path) -> None:
    _validate_schema_node(schema, path, "$")


def _validate_schema_node(node: Any, path: Path, location: str) -> None:
    if not isinstance(node, dict):
        raise SchemaError(f"{path}:{location}: schema nodes must be objects")
    for key, value in node.items():
        if key not in SUPPORTED_SCHEMA_KEYS:
            raise SchemaError(f"{path}:{location}: unsupported schema keyword '{key}'")
        if key == "type":
            if value not in SUPPORTED_TYPES:
                raise SchemaError(f"{path}:{location}: unsupported type {value!r}")
        elif key == "required":
            if not _is_string_list(value):
                raise SchemaError(f"{path}:{location}: required must be a string array")
        elif key == "properties":
            if not isinstance(value, dict):
                raise SchemaError(f"{path}:{location}: properties must be an object")
            for prop, child in value.items():
                if not isinstance(prop, str) or not prop:
                    raise SchemaError(f"{path}:{location}: property names must be non-empty strings")
                _validate_schema_node(child, path, f"{location}.properties.{prop}")
        elif key == "items":
            _validate_schema_node(value, path, f"{location}.items")
        elif key == "enum":
            if not isinstance(value, list) or not value:
                raise SchemaError(f"{path}:{location}: enum must be a non-empty array")
            if not all(isinstance(item, str | int | bool) for item in value):
                raise SchemaError(f"{path}:{location}: enum values must be primitive scalars")
        elif key == "additionalProperties":
            if not isinstance(value, bool):
                raise SchemaError(f"{path}:{location}: additionalProperties must be boolean")
        elif key == "format":
            if value not in SUPPORTED_FORMATS:
                raise SchemaError(f"{path}:{location}: unsupported format {value!r}")
        elif key in {"$schema", "title", "description"} and not isinstance(value, str):
            raise SchemaError(f"{path}:{location}: {key} must be a string")


def validate_data(data: Any, schema: dict[str, Any], *, source: str) -> None:
    _validate_data_node(data, schema, source, "$")


def _validate_data_node(data: Any, schema: dict[str, Any], source: str, location: str) -> None:
    expected_type = schema.get("type")
    if expected_type == "object":
        if not isinstance(data, dict):
            raise SchemaError(f"{source}:{location}: expected object")
        required = schema.get("required", [])
        assert isinstance(required, list)
        for key in required:
            if key not in data:
                raise SchemaError(f"{source}:{location}: missing required key '{key}'")
        properties = schema.get("properties", {})
        assert isinstance(properties, dict)
        if schema.get("additionalProperties") is False:
            extra = sorted(set(data) - set(properties))
            if extra:
                raise SchemaError(f"{source}:{location}: unsupported keys: {', '.join(extra)}")
        for key, child_schema in properties.items():
            if key in data:
                _validate_data_node(data[key], child_schema, source, f"{location}.{key}")
    elif expected_type == "array":
        if not isinstance(data, list):
            raise SchemaError(f"{source}:{location}: expected array")
        item_schema = schema.get("items")
        if isinstance(item_schema, dict):
            for index, item in enumerate(data):
                _validate_data_node(item, item_schema, source, f"{location}[{index}]")
    elif expected_type == "string":
        if not isinstance(data, str):
            raise SchemaError(f"{source}:{location}: expected string")
        if schema.get("format") == "repo-relative-path":
            _validate_repo_relative_path(data, source, location)
    elif expected_type == "integer":
        if not isinstance(data, int) or isinstance(data, bool):
            raise SchemaError(f"{source}:{location}: expected integer")
    elif expected_type == "number":
        if not isinstance(data, int | float) or isinstance(data, bool):
            raise SchemaError(f"{source}:{location}: expected number")
    elif expected_type == "boolean":
        if not isinstance(data, bool):
            raise SchemaError(f"{source}:{location}: expected boolean")

    enum_values = schema.get("enum")
    if isinstance(enum_values, list) and data not in enum_values:
        allowed = ", ".join(str(value) for value in enum_values)
        raise SchemaError(f"{source}:{location}: expected one of: {allowed}")


def _validate_repo_relative_path(value: str, source: str, location: str) -> None:
    path = Path(value)
    if not value or path.is_absolute() or ".." in path.parts:
        raise SchemaError(f"{source}:{location}: expected repo-relative path, got {value!r}")


def _is_string_list(value: Any) -> bool:
    return isinstance(value, list) and all(isinstance(item, str) and item for item in value)
