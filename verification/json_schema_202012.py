"""Dependency-free JSON Schema 2020-12 subset used by governed artifacts."""

from __future__ import annotations

import json
import re
from datetime import datetime
from pathlib import Path
from typing import Any

SCHEMA_KEYS = {
    "$schema",
    "$id",
    "$defs",
    "$ref",
    "title",
    "description",
    "type",
    "const",
    "enum",
    "required",
    "properties",
    "propertyNames",
    "additionalProperties",
    "items",
    "minItems",
    "maxItems",
    "uniqueItems",
    "minProperties",
    "minimum",
    "minLength",
    "pattern",
    "format",
    "oneOf",
    "allOf",
    "if",
    "then",
    "else",
    "not",
}


class JsonSchemaError(ValueError):
    """A schema or instance violates the supported governed subset."""


def lint_schema(path: Path) -> dict[str, Any]:
    try:
        schema = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise JsonSchemaError(f"{path}: invalid JSON Schema: {exc}") from exc
    _lint_node(schema, path, "$")
    _validate_refs(schema, path, path)
    return schema


def validate_instance(instance: Any, schema_path: Path) -> None:
    schema = lint_schema(schema_path)
    _validate(instance, schema, schema_path, schema_path, "$")


def _lint_node(node: Any, path: Path, location: str) -> None:
    if isinstance(node, bool):
        return
    if not isinstance(node, dict):
        raise JsonSchemaError(f"{path}:{location}: schema node must be an object or boolean")
    unknown = sorted(set(node).difference(SCHEMA_KEYS))
    if unknown:
        raise JsonSchemaError(f"{path}:{location}: unsupported keyword(s): {', '.join(unknown)}")
    for keyword in ("properties", "$defs"):
        children = node.get(keyword, {})
        if not isinstance(children, dict):
            raise JsonSchemaError(f"{path}:{location}.{keyword}: must be an object")
        for name, child in children.items():
            _lint_node(child, path, f"{location}.{keyword}.{name}")
    for keyword in ("items", "propertyNames", "additionalProperties", "if", "then", "else", "not"):
        child = node.get(keyword)
        if child is not None and isinstance(child, (dict, bool)):
            _lint_node(child, path, f"{location}.{keyword}")
        elif child is not None and keyword not in {"additionalProperties"}:
            raise JsonSchemaError(f"{path}:{location}.{keyword}: must be a schema")
    for keyword in ("oneOf", "allOf"):
        if keyword not in node:
            continue
        children = node.get(keyword, [])
        if not isinstance(children, list) or (keyword == "oneOf" and not children):
            raise JsonSchemaError(f"{path}:{location}.{keyword}: must be a non-empty array")
        for index, child in enumerate(children):
            _lint_node(child, path, f"{location}.{keyword}[{index}]")
    if "format" in node and node["format"] != "date-time":
        raise JsonSchemaError(f"{path}:{location}.format: unsupported format")


def _validate_refs(node: Any, root_path: Path, current_path: Path) -> None:
    if isinstance(node, dict):
        ref = node.get("$ref")
        if isinstance(ref, str):
            _resolve_ref(ref, root_path, current_path)
        for value in node.values():
            _validate_refs(value, root_path, current_path)
    elif isinstance(node, list):
        for value in node:
            _validate_refs(value, root_path, current_path)


def _validate(
    value: Any,
    schema: Any,
    root_path: Path,
    current_path: Path,
    location: str,
) -> None:
    if schema is True:
        return
    if schema is False:
        raise JsonSchemaError(f"{location}: rejected by false schema")
    if "$ref" in schema:
        resolved, resolved_path = _resolve_ref(schema["$ref"], root_path, current_path)
        _validate(value, resolved, resolved_path, resolved_path, location)
        return
    for child in schema.get("allOf", []):
        _validate(value, child, root_path, current_path, location)
    if "if" in schema:
        matched = _matches(value, schema["if"], root_path, current_path, location)
        branch = schema.get("then") if matched else schema.get("else")
        if branch is not None:
            _validate(value, branch, root_path, current_path, location)
    if "oneOf" in schema:
        matches = sum(
            _matches(value, child, root_path, current_path, location)
            for child in schema["oneOf"]
        )
        if matches != 1:
            raise JsonSchemaError(f"{location}: must match exactly one oneOf branch")
    if "not" in schema and _matches(value, schema["not"], root_path, current_path, location):
        raise JsonSchemaError(f"{location}: matches forbidden schema")
    if "const" in schema and value != schema["const"]:
        raise JsonSchemaError(f"{location}: must equal {schema['const']!r}")
    if "enum" in schema and value not in schema["enum"]:
        raise JsonSchemaError(f"{location}: is not an allowed value")
    _validate_type(value, schema, location)
    if isinstance(value, dict):
        _validate_object(value, schema, root_path, current_path, location)
    elif isinstance(value, list):
        _validate_array(value, schema, root_path, current_path, location)
    elif isinstance(value, str):
        _validate_string(value, schema, location)
    elif isinstance(value, (int, float)) and not isinstance(value, bool):
        minimum = schema.get("minimum")
        if minimum is not None and value < minimum:
            raise JsonSchemaError(f"{location}: must be at least {minimum}")


def _validate_object(
    value: dict[str, Any],
    schema: dict[str, Any],
    root_path: Path,
    current_path: Path,
    location: str,
) -> None:
    for key in schema.get("required", []):
        if key not in value:
            raise JsonSchemaError(f"{location}: missing required field {key}")
    minimum = schema.get("minProperties")
    if minimum is not None and len(value) < minimum:
        raise JsonSchemaError(f"{location}: has too few properties")
    properties = schema.get("properties", {})
    property_names = schema.get("propertyNames")
    for key, child_value in value.items():
        if property_names is not None:
            _validate(key, property_names, root_path, current_path, f"{location} key")
        child_schema = properties.get(key)
        if child_schema is not None:
            _validate(child_value, child_schema, root_path, current_path, f"{location}.{key}")
            continue
        additional = schema.get("additionalProperties", True)
        if additional is False:
            raise JsonSchemaError(f"{location}: unknown field {key}")
        if isinstance(additional, (dict, bool)):
            _validate(child_value, additional, root_path, current_path, f"{location}.{key}")


def _validate_array(
    value: list[Any],
    schema: dict[str, Any],
    root_path: Path,
    current_path: Path,
    location: str,
) -> None:
    if len(value) < schema.get("minItems", 0):
        raise JsonSchemaError(f"{location}: has too few items")
    if "maxItems" in schema and len(value) > schema["maxItems"]:
        raise JsonSchemaError(f"{location}: has too many items")
    if schema.get("uniqueItems") and len({canonical(item) for item in value}) != len(value):
        raise JsonSchemaError(f"{location}: items must be unique")
    if "items" in schema:
        for index, item in enumerate(value):
            _validate(item, schema["items"], root_path, current_path, f"{location}[{index}]")


def _validate_string(value: str, schema: dict[str, Any], location: str) -> None:
    if len(value) < schema.get("minLength", 0):
        raise JsonSchemaError(f"{location}: is too short")
    pattern = schema.get("pattern")
    if pattern is not None and re.search(pattern, value) is None:
        raise JsonSchemaError(f"{location}: does not match {pattern}")
    if schema.get("format") == "date-time":
        try:
            parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
        except ValueError as exc:
            raise JsonSchemaError(f"{location}: must be an ISO-8601 date-time") from exc
        if parsed.tzinfo is None:
            raise JsonSchemaError(f"{location}: date-time must include a timezone")


def _validate_type(value: Any, schema: dict[str, Any], location: str) -> None:
    expected = schema.get("type")
    if expected is None:
        return
    allowed = expected if isinstance(expected, list) else [expected]
    checks = {
        "object": lambda item: isinstance(item, dict),
        "array": lambda item: isinstance(item, list),
        "string": lambda item: isinstance(item, str),
        "integer": lambda item: isinstance(item, int) and not isinstance(item, bool),
        "number": lambda item: isinstance(item, (int, float)) and not isinstance(item, bool),
        "boolean": lambda item: isinstance(item, bool),
        "null": lambda item: item is None,
    }
    if not any(name in checks and checks[name](value) for name in allowed):
        raise JsonSchemaError(f"{location}: expected type {expected}")


def _matches(
    value: Any,
    schema: Any,
    root_path: Path,
    current_path: Path,
    location: str,
) -> bool:
    try:
        _validate(value, schema, root_path, current_path, location)
    except JsonSchemaError:
        return False
    return True


def _resolve_ref(ref: str, root_path: Path, current_path: Path) -> tuple[Any, Path]:
    file_text, separator, fragment = ref.partition("#")
    path = (current_path.parent / file_text).resolve() if file_text else root_path.resolve()
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise JsonSchemaError(f"{current_path}: cannot resolve schema reference {ref}: {exc}") from exc
    node: Any = payload
    if separator and fragment:
        if not fragment.startswith("/"):
            raise JsonSchemaError(f"{current_path}: unsupported schema fragment {fragment}")
        for token in fragment[1:].split("/"):
            token = token.replace("~1", "/").replace("~0", "~")
            if not isinstance(node, dict) or token not in node:
                raise JsonSchemaError(f"{current_path}: unresolved schema reference {ref}")
            node = node[token]
    return node, path


def canonical(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
