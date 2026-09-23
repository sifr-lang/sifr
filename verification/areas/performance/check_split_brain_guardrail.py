#!/usr/bin/env python3
"""Guard the production owner of Sifr parsing and lowering.

This is a source admission check, not a Rust name resolver. It follows imports and
module gates so test code cannot exempt production code in the same file.
"""

from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import re
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
APPROVED_PREFIXES = (
    Path("crates/sifr_syntax/src"),
    Path("crates/sifr_frontend/src"),
    Path("crates/sifr_lowering/src"),
)
PARSERS = ("sifr_syntax", "sifr_python_parser", "ruff_python_parser")
ENTRYPOINTS = ("parse_module", "parse_module_raw", "parse_module_suite")
OTHER_FORBIDDEN = (
    "parse_unchecked", "parse_module_with_diagnostics", "lower_module_with_externals",
    "lower_module", "lower_frontend_module",
)
# Each line is an individually adjudicated production site. Source inspection here
# does not establish an independent Sifr semantic result. Bootstrap is the one
# stdlib compilation owner: it parses before its own lower_stdlib_source call.
ALLOWED_SITES: dict[Path, dict[str, str]] = {
    Path("crates/sifr/src/check_and_package_commands.rs"): {
        "let Ok(parsed) = sifr_syntax::parse_module(source.as_str(), Some(&context)) else {": "CLI dependency declaration inspection",
        "if let Ok(parsed) = sifr_syntax::parse_module(source.as_str(), Some(&context)) {": "CLI entry dependency inspection",
    },
    Path("crates/sifr_lint/src/engine.rs"): {
        "sifr_syntax::parse_module(source, parse_context.as_deref()).ok()": "lint token and syntax rules",
    },
    Path("crates/sifr_format/src/lib.rs"): {
        "parse_module_raw(": "formatter output syntax validation",
    },
    Path("crates/sifr_package/src/python/bridge_inventory/filesystem.rs"): {
        'sifr_syntax::parse_module_suite(&format!("import {module}\\n"), None)': "Python import-name syntax validation",
    },
    Path("crates/sifr_package/src/python/bridge_inventory/mod.rs"): {
        "let suite = sifr_syntax::parse_module_suite(&source, Some(&source_path.display().to_string()))": "Python bridge import inventory",
    },
    Path("crates/sifr_driver/src/metadata_producer/locations.rs"): {
        "let parsed = sifr_syntax::parse_module_raw(source, None).map_err(|_| {": "checked declaration source ranges",
    },
    Path("crates/sifr_driver/src/stdlib/bootstrap.rs"): {
        "let parsed = match parse_module_raw(stdlib_source.source.as_str(), Some(&source_name)) {": "canonical stdlib compile input",
    },
}
# Two formatter calls share a line shape; the count still fixes their number.
ALLOWED_COUNTS = {Path("crates/sifr_format/src/lib.rs"): {"parse_module_raw(": 2}}


RAW_LITERAL = re.compile(r'(?<![A-Za-z0-9_])(?:br|r)(#{0,16})"')
CHAR_LITERAL = re.compile(r"'([^'\\]|\\.)'")


def scrub(text: str) -> str:
    """Replace comments and literals with spaces, preserving byte offsets/newlines."""
    out = list(text)
    i = 0
    while i < len(text):
        start = i
        if text.startswith("//", i):
            end = text.find("\n", i)
            i = len(text) if end < 0 else end
        elif text.startswith("/*", i):
            depth = 1
            i += 2
            while i < len(text) and depth:
                if text.startswith("/*", i):
                    depth += 1; i += 2
                elif text.startswith("*/", i):
                    depth -= 1; i += 2
                else:
                    i += 1
        else:
            raw = RAW_LITERAL.match(text, i)
            if raw:
                delimiter = '"' + raw.group(1)
                i += len(raw.group(0))
                end = text.find(delimiter, i)
                i = len(text) if end < 0 else end + len(delimiter)
            elif text[i] == '"' or text.startswith('b"', i):
                i += 2 if text.startswith('b"', i) else 1
                while i < len(text):
                    if text[i] == "\\": i += 2
                    elif text[i] == '"':
                        i += 1; break
                    else: i += 1
            elif text[i] == "'" and (char := CHAR_LITERAL.match(text, i)):
                i = char.end()
            else:
                i += 1
                continue
        for j in range(start, min(i, len(text))):
            if out[j] != "\n": out[j] = " "
    return "".join(out)


def closing_brace(code: str, opening: int) -> int | None:
    depth = 0
    for i in range(opening, len(code)):
        if code[i] == "{": depth += 1
        elif code[i] == "}":
            depth -= 1
            if depth == 0: return i
    return None


# Attribute text is read from the original source for #[path = "..."]; the
# positions are checked against scrubbed code before a module is accepted.
MOD_PATTERN = re.compile(
    r'(?m)^[ \t]*(?P<attrs>(?:#\[[^\]\n]+\][ \t]*\n[ \t]*)*)'
    r'(?:pub(?:\([^)]*\))?[ \t]+)?mod[ \t]+(?P<name>\w+)[ \t]*(?P<kind>[;{])'
)


def module_declarations(text: str, code: str):
    for match in MOD_PATTERN.finditer(text):
        if not re.match(r'\bmod\b', code[match.start("name") - 4:match.start("name")]):
            continue
        attrs = match.group("attrs")
        gate = bool(re.search(r'#\[cfg\s*\(\s*test\s*\)\s*\]', attrs))
        path = re.search(r'#\[path\s*=\s*"([^"]+)"\s*\]', attrs)
        yield match, gate, path.group(1) if path else None


def inline_test_ranges(text: str, code: str) -> list[tuple[int, int]]:
    ranges = []
    for match, gated, _ in module_declarations(text, code):
        if gated and match.group("kind") == "{":
            end = closing_brace(code, match.end() - 1)
            if end is not None: ranges.append((match.start(), end + 1))
    return ranges


def within(offset: int, ranges: list[tuple[int, int]]) -> bool:
    return any(start <= offset < end for start, end in ranges)


def module_target(owner: Path, name: str, explicit: str | None) -> Path | None:
    if explicit:
        candidate = owner.parent / explicit
        return candidate.resolve() if candidate.is_file() else None
    base = owner.parent if owner.stem in {"lib", "main", "mod"} else owner.parent / owner.stem
    for candidate in (base / f"{name}.rs", base / name / "mod.rs"):
        if candidate.is_file(): return candidate.resolve()
    return None


def test_only_files(paths: list[Path]) -> set[Path]:
    path_set = {p.resolve() for p in paths}
    incoming: dict[Path, list[tuple[Path, bool]]] = defaultdict(list)
    for owner in paths:
        text = owner.read_text(encoding="utf-8", errors="replace")
        code = scrub(text)
        ranges = inline_test_ranges(text, code)
        for match, gated, explicit in module_declarations(text, code):
            if match.group("kind") != ";": continue
            target = module_target(owner, match.group("name"), explicit)
            if target in path_set:
                incoming[target].append((owner.resolve(), gated or within(match.start(), ranges)))
    test_only: set[Path] = set()
    changed = True
    while changed:
        changed = False
        for path, refs in incoming.items():
            if path not in test_only and refs and all(gated or owner in test_only for owner, gated in refs):
                test_only.add(path); changed = True
    return test_only


def import_bindings(code: str, test_ranges: list[tuple[int, int]]) -> tuple[set[str], set[str], bool]:
    crates = set(PARSERS)
    calls: set[str] = set()
    wildcard = False
    for match in re.finditer(r'\buse\s+([^;]+);', code, re.DOTALL):
        if within(match.start(), test_ranges): continue
        statement = match.group(1)
        root = re.match(r'\s*(sifr_syntax|sifr_python_parser|ruff_python_parser)\b', statement)
        if root is None: continue
        rest = statement[root.end():]
        for alias in re.findall(r'\bself\s+as\s+(\w+)', rest): crates.add(alias)
        alias = re.match(r'\s+as\s+(\w+)\s*$', rest)
        if alias: crates.add(alias.group(1))
        if re.search(r'::\s*\*', rest): wildcard = True
        for entry, renamed in re.findall(
            r'\b(parse_module(?:_raw|_suite)?)\b(?:\s+as\s+(\w+))?', rest
        ):
            calls.add(renamed or entry)
    return crates, calls, wildcard


def violations_text(path: Path, text: str, *, test_only: bool = False) -> list[str]:
    if test_only: return []
    try: rel = path.relative_to(REPO_ROOT)
    except ValueError: rel = path
    if any(rel.is_relative_to(prefix) for prefix in APPROVED_PREFIXES): return []
    code = scrub(text)
    ranges = inline_test_ranges(text, code)
    crates, imported, wildcard = import_bindings(code, ranges)
    allowed = ALLOWED_SITES.get(rel, {})
    remaining = Counter({line: ALLOWED_COUNTS.get(rel, {}).get(line, 1) for line in allowed})
    failures = []
    names = "|".join(re.escape(name) for name in sorted(set(ENTRYPOINTS + OTHER_FORBIDDEN) | imported, key=len, reverse=True))
    candidates = re.finditer(rf"\b(?:(\w+)\s*::\s*)?({names})\s*\(", code)
    for match in candidates:
        if within(match.start(), ranges): continue
        qualifier, name = match.groups()
        if qualifier:
            if name in ENTRYPOINTS and qualifier not in crates: continue
            if name in OTHER_FORBIDDEN and qualifier not in {*crates, "sifr_lowering", "sifr_frontend"}: continue
        elif name not in OTHER_FORBIDDEN and name not in imported and not (wildcard and name in ENTRYPOINTS):
            continue
        elif name in OTHER_FORBIDDEN and name not in {"lower_module", "lower_frontend_module"}:
            continue
        start = text.rfind("\n", 0, match.start()) + 1
        end = text.find("\n", match.start())
        line = text[start:end if end >= 0 else len(text)].strip()
        # A multiline formatter call is anchored by its opening line.
        if line in allowed and remaining[line] > 0:
            remaining[line] -= 1
            continue
        failures.append(f"{rel}:{text.count(chr(10), 0, match.start()) + 1} unadjudicated production {qualifier + '::' if qualifier else ''}{name} call")
    return failures


def rust_files() -> list[Path]:
    return sorted(p for p in (REPO_ROOT / "crates").rglob("*.rs") if "target" not in p.parts)


def violations(paths: list[Path]) -> list[str]:
    test_only = test_only_files(paths)
    return [finding for p in paths for finding in violations_text(
        p, p.read_text(encoding="utf-8", errors="replace"), test_only=p.resolve() in test_only
    )]


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-v03b-f25-") as tmp:
        root = Path(tmp)
        owner = root / "lib.rs"
        child = root / "fixture.rs"
        for name in ENTRYPOINTS:
            for snippet in (
                f"use sifr_syntax::{{{name} as parse}}; fn prod() {{ parse(src, None); }}",
                f"use sifr_syntax as syntax; fn prod() {{ syntax::{name}(src, None); }}",
                f"use sifr_syntax::{{self as syntax}}; fn prod() {{ syntax::{name}(src, None); }}",
                f"use sifr_syntax::*; fn prod() {{ {name}(src, None); }}",
            ):
                assert violations_text(owner, snippet), snippet
        for name in ENTRYPOINTS:
            mixed = f'''#[cfg(test)]\nmod tests {{ use sifr_syntax::{name} as parse; fn fixture() {{ parse(src, None); }} }}\nuse sifr_syntax::{name} as semantic_parse;\nfn prod() {{ semantic_parse(src, None); }}\n'''
            assert violations_text(owner, mixed), f"mixed production {name} alias escaped"
        assert violations_text(owner, "use sifr_python_parser::parse_module as parse; use sifr_lowering::lower_module; fn prod() { let parsed = parse(src); lower_module(parsed.suite()); }")
        assert not violations_text(owner, '#[cfg(test)]\nmod tests { use sifr_syntax::parse_module_raw as parse; fn fixture() { parse(src, None); } }')
        assert not violations_text(owner, 'let s = "sifr_syntax::parse_module(x, None)"; // parse_module_raw(x, None)')
        owner.write_text('#[cfg(test)]\n#[path = "fixture.rs"]\nmod fixture;\n')
        child.write_text('use sifr_syntax::parse_module_raw as parse; fn fixture() { parse(src, None); }')
        assert not violations([owner, child]), "test-only cross-file module rejected"
        owner.write_text('mod fixture;\n')
        assert violations([owner, child]), "production cross-file module escaped"
        owner.write_text('#[cfg(test)]\nmod fixture;\nmod fixture_tests;\n')
        (root / 'fixture_tests.rs').write_text('use sifr_syntax::parse_module_suite as parse; fn prod() { parse(src, None); }')
        assert violations([owner, child, root / 'fixture_tests.rs']), "production test-named sibling escaped"
        cli = REPO_ROOT / 'crates/sifr/src/check_and_package_commands.rs'
        assert not violations([cli])
        extra = cli.read_text() + '\nfn duplicate() { sifr_syntax::parse_module(src, None); }\n'
        assert violations_text(cli, extra), "same-file duplicate escaped"
    print('split-brain guardrail self-test: PASS')


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0
    found = violations(rust_files())
    if found:
        print('split-brain guardrail: FAIL', file=sys.stderr)
        for failure in found: print(f'  - {failure}', file=sys.stderr)
        return 1
    print('split-brain guardrail: PASS')
    return 0


if __name__ == '__main__':
    sys.exit(main())
