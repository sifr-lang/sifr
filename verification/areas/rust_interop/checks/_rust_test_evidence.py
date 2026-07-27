"""Parse the Rust test declarations used by interop evidence provenance."""

from __future__ import annotations

import re
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

RUST_TEST_ATTRIBUTE = re.compile(
    r"^\s*#\[\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*test(?:\s*\([^]]*\))?\s*\]\s*$"
)
RUST_IGNORE_ATTRIBUTE = re.compile(r"^\s*#\[\s*ignore(?:\s*=|\s*\]|\s*\()")
RUST_FUNCTION = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?"
    r"fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("
)
RUST_INLINE_MODULE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{"
)
RUST_EXTERNAL_MODULE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;"
)
RUST_CFG_FEATURE = re.compile(r'feature\s*=\s*"([^"]+)"')
RUST_PATH_ATTRIBUTE = re.compile(r'#\[\s*path\s*=\s*"([^"]+)"\s*\]')
RUST_PATH_HINT = re.compile(r"#\s*\[\s*path\b")
EXECUTES_CARGO_PROBE_MARKER = "sifr-evidence: executes-cargo-probe"
EXECUTES_RUNTIME_OBSERVED_MARKER = "sifr-evidence: executes-runtime-observed"


@dataclass(frozen=True)
class RustTestDefinition:
    """Properties that determine whether a suite genuinely executes a Rust test."""

    ignored: bool
    required_features: frozenset[str]
    executes_cargo_probe: bool
    executes_runtime_observed: bool
    inline_modules: tuple[str, ...]


@dataclass(frozen=True)
class _ModuleScope:
    depth: int
    name: str
    features: frozenset[str]


def rust_test_definitions(
    source_path: Path,
    test_name: str,
) -> list[RustTestDefinition]:
    """Return exact uncommented Rust test declarations and their enclosing gates."""
    matches: list[RustTestDefinition] = []
    attributes: list[str] = []
    file_features: set[str] = set()
    brace_depth = 0
    module_scopes: list[_ModuleScope] = []
    for line, raw_attribute in _logical_rust_lines(source_path):
        stripped = line.strip()
        if raw_attribute is not None:
            if stripped.startswith("#!["):
                file_features.update(_features_from_attributes([raw_attribute]))
            else:
                attributes.append(raw_attribute)
            continue
        if not stripped:
            continue
        inline_module = RUST_INLINE_MODULE.match(line)
        if inline_module is not None:
            next_depth = brace_depth + line.count("{") - line.count("}")
            if next_depth > brace_depth:
                module_scopes.append(
                    _ModuleScope(
                        next_depth,
                        inline_module.group(1),
                        _features_from_attributes(attributes),
                    )
                )
            attributes.clear()
            brace_depth = next_depth
            continue
        function = RUST_FUNCTION.match(line)
        if function is not None:
            if (
                function.group(1) == test_name
                and any(RUST_TEST_ATTRIBUTE.match(attribute) for attribute in attributes)
            ):
                matches.append(
                    RustTestDefinition(
                        ignored=any(
                            RUST_IGNORE_ATTRIBUTE.match(attribute)
                            for attribute in attributes
                        ),
                        required_features=frozenset(file_features)
                        | _features_from_attributes(attributes)
                        | frozenset(
                            feature
                            for scope in module_scopes
                            for feature in scope.features
                        ),
                        executes_cargo_probe=any(
                            EXECUTES_CARGO_PROBE_MARKER in attribute
                            for attribute in attributes
                        ),
                        executes_runtime_observed=any(
                            EXECUTES_RUNTIME_OBSERVED_MARKER in attribute
                            for attribute in attributes
                        ),
                        inline_modules=tuple(scope.name for scope in module_scopes),
                    )
                )
            attributes.clear()
        else:
            attributes.clear()
        brace_depth += line.count("{") - line.count("}")
        module_scopes = [
            scope for scope in module_scopes if brace_depth >= scope.depth
        ]
    return matches


def external_module_features(
    source_path: Path,
    package_root: Path,
) -> frozenset[str]:
    """Collect cfg(feature) gates from each external module declaration."""
    source_root = (package_root / "src").resolve()
    return _external_module_features(source_path.resolve(), source_root, frozenset())


def _external_module_features(
    source_path: Path,
    source_root: Path,
    seen: frozenset[Path],
) -> frozenset[str]:
    if source_path in seen:
        return frozenset()
    next_seen = seen | {source_path}
    for parent, _module_name, target, features in _path_module_declarations(
        source_root
    ):
        if target == source_path:
            return features | _external_module_features(
                parent.resolve(),
                source_root,
                next_seen,
            )
    try:
        relative = source_path.relative_to(source_root)
    except ValueError:
        return frozenset()
    parts = list(relative.with_suffix("").parts)
    if parts and parts[-1] in {"lib", "main"}:
        parts.clear()
    elif parts and parts[-1] == "mod":
        parts.pop()
    features: set[str] = set()
    for index, module_name in enumerate(parts):
        if index == 0:
            parents = (source_root / "lib.rs", source_root / "main.rs")
        else:
            prefix = Path(*parts[:index])
            parents = (
                source_root / prefix.with_suffix(".rs"),
                source_root / prefix / "mod.rs",
            )
        for parent in parents:
            if parent.is_file():
                features.update(_module_declaration_features(parent, module_name))
    return frozenset(features)


def rust_test_path(
    source_path: Path,
    package_root: Path,
    definition: RustTestDefinition,
    test_name: str,
) -> str:
    """Derive the full cargo-test path used by substring filters."""
    source_root = (package_root / "src").resolve()
    parts = list(_source_module_parts(source_path, source_root, frozenset()))
    return "::".join((*parts, *definition.inline_modules, test_name))


def _source_module_parts(
    source_path: Path,
    source_root: Path,
    seen: frozenset[Path],
) -> tuple[str, ...]:
    """Resolve a source file's declared module chain, including #[path] aliases."""
    resolved_source = source_path.resolve()
    if resolved_source in seen:
        return _natural_module_parts(source_path, source_root)
    next_seen = seen | {resolved_source}
    for parent, module_name, target, _features in _path_module_declarations(
        source_root
    ):
        if target == resolved_source:
            return (
                *_source_module_parts(parent, source_root, next_seen),
                module_name,
            )
    return _natural_module_parts(source_path, source_root)


def _natural_module_parts(source_path: Path, source_root: Path) -> tuple[str, ...]:
    try:
        parts = list(source_path.relative_to(source_root).with_suffix("").parts)
    except ValueError:
        return ()
    if parts and parts[0] in {"lib", "main"}:
        parts.pop(0)
    if parts and parts[-1] == "mod":
        parts.pop()
    return tuple(parts)


@lru_cache(maxsize=None)
def _path_module_declarations(
    source_root: Path,
) -> tuple[tuple[Path, str, Path, frozenset[str]], ...]:
    declarations: list[tuple[Path, str, Path, frozenset[str]]] = []
    for parent in sorted(source_root.rglob("*.rs")):
        if RUST_PATH_HINT.search(parent.read_text(encoding="utf-8")) is None:
            continue
        attributes: list[str] = []
        file_features: set[str] = set()
        for line, raw_attribute in _logical_rust_lines(parent):
            if raw_attribute is not None:
                if line.strip().startswith("#!["):
                    file_features.update(_features_from_attributes([raw_attribute]))
                else:
                    attributes.append(raw_attribute)
                continue
            if not line.strip():
                continue
            declaration = RUST_EXTERNAL_MODULE.match(line)
            if declaration is not None:
                raw_path = next(
                    (
                        match.group(1)
                        for attribute in attributes
                        if (match := RUST_PATH_ATTRIBUTE.search(attribute)) is not None
                    ),
                    None,
                )
                if raw_path is not None:
                    declarations.append(
                        (
                            parent,
                            declaration.group(1),
                            (parent.parent / raw_path).resolve(),
                            frozenset(file_features)
                            | _features_from_attributes(attributes),
                        )
                    )
            attributes.clear()
    return tuple(declarations)


def clear_module_declaration_cache() -> None:
    """Discard cached module topology after a mutation self-test changes sources."""
    _path_module_declarations.cache_clear()


def _module_declaration_features(parent: Path, module_name: str) -> frozenset[str]:
    attributes: list[str] = []
    file_features: set[str] = set()
    for line, raw_attribute in _logical_rust_lines(parent):
        stripped = line.strip()
        if raw_attribute is not None:
            if stripped.startswith("#!["):
                file_features.update(_features_from_attributes([raw_attribute]))
            else:
                attributes.append(raw_attribute)
            continue
        if not stripped:
            continue
        declaration = RUST_EXTERNAL_MODULE.match(line)
        if declaration is not None and declaration.group(1) == module_name:
            return frozenset(file_features) | _features_from_attributes(attributes)
        attributes.clear()
    return frozenset()


def _features_from_attributes(attributes: list[str]) -> frozenset[str]:
    return frozenset(
        feature
        for attribute in attributes
        for feature in RUST_CFG_FEATURE.findall(attribute)
    )


def _logical_rust_lines(source_path: Path) -> list[tuple[str, str | None]]:
    """Return code lines, joining outer and inner attributes across lines."""
    result: list[tuple[str, str | None]] = []
    lexical_state: tuple[int, int | None, bool] = (0, None, False)
    attribute_code: list[str] = []
    attribute_raw: list[str] = []
    bracket_depth = 0
    for raw_line in source_path.read_text(encoding="utf-8").splitlines():
        line, lexical_state = _rust_code_line(raw_line, lexical_state)
        stripped = line.strip()
        if attribute_code:
            attribute_code.append(line)
            attribute_raw.append(raw_line)
            bracket_depth += line.count("[") - line.count("]")
            if bracket_depth <= 0:
                result.append(
                    (" ".join(attribute_code), "\n".join(attribute_raw))
                )
                attribute_code.clear()
                attribute_raw.clear()
            continue
        if stripped.startswith(("#[", "#![")):
            attribute_code.append(line)
            attribute_raw.append(raw_line)
            bracket_depth = line.count("[") - line.count("]")
            if bracket_depth <= 0:
                result.append((line, raw_line))
                attribute_code.clear()
                attribute_raw.clear()
            continue
        result.append((line, None))
    return result


def _rust_code_line(
    line: str,
    state: tuple[int, int | None, bool],
) -> tuple[str, tuple[int, int | None, bool]]:
    """Blank Rust comments and strings, including nested and multiline forms."""
    block_depth, raw_hashes, in_string = state
    visible: list[str] = []
    index = 0
    escaped = False
    while index < len(line):
        if block_depth:
            if line.startswith("/*", index):
                block_depth += 1
                index += 2
            elif line.startswith("*/", index):
                block_depth -= 1
                index += 2
            else:
                index += 1
            visible.append(" ")
            continue
        if raw_hashes is not None:
            terminator = '"' + ("#" * raw_hashes)
            if line.startswith(terminator, index):
                raw_hashes = None
                index += len(terminator)
            else:
                index += 1
            visible.append(" ")
            continue
        if in_string:
            character = line[index]
            index += 1
            visible.append(" ")
            if character == '"' and not escaped:
                in_string = False
            escaped = character == "\\" and not escaped
            if character != "\\":
                escaped = False
            continue
        if line.startswith("//", index):
            break
        if line.startswith("/*", index):
            block_depth = 1
            index += 2
            visible.append(" ")
            continue
        raw_start = re.match(r'(?:b)?r(#+)?"', line[index:])
        if raw_start is not None:
            raw_hashes = len(raw_start.group(1) or "")
            index += len(raw_start.group(0))
            visible.append(" ")
            continue
        character = line[index]
        if character == '"':
            in_string = True
            escaped = False
            visible.append(" ")
        else:
            visible.append(character)
        index += 1
    return "".join(visible), (block_depth, raw_hashes, in_string)
