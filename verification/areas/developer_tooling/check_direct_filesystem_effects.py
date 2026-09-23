#!/usr/bin/env python3
"""Pin each first-party Rust filesystem effect to an explicit ownership class."""

from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
import re
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[3]
INVENTORY = Path(__file__).with_name("data") / "direct_filesystem_effects.json"
OPERATIONS = ("read_to_string", "read_dir", "read_link", "symlink_metadata", "try_exists", "canonicalize", "metadata", "exists", "is_file", "is_dir", "read", "open")
FS_FUNCTIONS = OPERATIONS[:-3] + ("read",)
PATH_METHODS = ("is_file", "is_dir", "exists", "try_exists", "metadata", "canonicalize", "read_link")
CLASSIFICATIONS = {"semantic-input", "build-identity", "tooling-input", "output-effect"}
FUNCTION = re.compile(r"\bfn\s+(\w+)")


def rust_code(text: str) -> str:
    """Mask comments and literals without changing offsets or line boundaries."""
    out = list(text)
    i = 0
    state = "code"
    depth = 0
    hashes = 0
    while i < len(text):
        pair = text[i:i + 2]
        char = text[i]
        if state == "code":
            raw = re.match(r'r(#{0,16})"', text[i:])
            if pair == "//":
                state = "line"
                out[i:i + 2] = "  "
                i += 2
                continue
            if pair == "/*":
                state = "block"
                depth = 1
                out[i:i + 2] = "  "
                i += 2
                continue
            if raw:
                state = "raw"
                hashes = len(raw[1])
                end = i + len(raw[0])
                out[i:end] = " " * (end - i)
                i = end
                continue
            if char == '"':
                state = "string"
                out[i] = " "
            elif char == "'" and i + 2 < len(text) and text[i + 2] == "'":
                state = "char"
                out[i] = " "
        elif state == "line":
            if char == "\n":
                state = "code"
            else:
                out[i] = " "
        elif state == "block":
            if pair == "/*":
                depth += 1
                out[i:i + 2] = "  "
                i += 2
                continue
            if pair == "*/":
                depth -= 1
                out[i:i + 2] = "  "
                i += 2
                if depth == 0:
                    state = "code"
                continue
            if char != "\n":
                out[i] = " "
        elif state == "string":
            if char == "\\":
                out[i] = " "
                if i + 1 < len(text):
                    i += 1
                    if text[i] != "\n":
                        out[i] = " "
            elif char == '"':
                out[i] = " "
                state = "code"
            elif char != "\n":
                out[i] = " "
        elif state == "raw":
            terminator = '"' + '#' * hashes
            if text.startswith(terminator, i):
                out[i:i + len(terminator)] = " " * len(terminator)
                i += len(terminator)
                state = "code"
                continue
            if char != "\n":
                out[i] = " "
        elif state == "char":
            if char == "'":
                state = "code"
            if char != "\n":
                out[i] = " "
        i += 1
    return "".join(out)


def source_paths(root: Path) -> list[Path]:
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    members = manifest["workspace"]["members"]
    paths: set[Path] = set()
    for member in members:
        if "*" in member:
            raise ValueError(f"unresolved workspace member glob: {member}")
        crate = root / member
        if not (crate / "Cargo.toml").is_file():
            raise ValueError(f"missing workspace member: {member}")
        for path in crate.rglob("*.rs"):
            if not any(part in {"target", "third_party", "vendor"} for part in path.relative_to(crate).parts):
                paths.add(path)
        cargo = tomllib.loads((crate / "Cargo.toml").read_text(encoding="utf-8"))
        for section in ("lib", "bin", "example", "test", "bench"):
            targets = cargo.get(section, [])
            for target in ([targets] if isinstance(targets, dict) else targets):
                if "path" in target:
                    named = crate / target["path"]
                    if not named.is_file():
                        raise ValueError(f"missing declared target: {named}")
                    paths.add(named)
    return sorted(paths)


def patterns(code: str) -> list[tuple[str, re.Pattern[str]]]:
    aliases = {"fs", "std::fs"}
    for match in re.finditer(r"\buse\s+std::fs\s+as\s+(\w+)\s*;", code):
        aliases.add(match[1])
    for match in re.finditer(r"\buse\s+std::fs::\{[^}]*\bself\s+as\s+(\w+)[^}]*\}", code, re.S):
        aliases.add(match[1])
    results = []
    for alias in sorted(aliases, key=len, reverse=True):
        for operation in FS_FUNCTIONS:
            results.append((operation, re.compile(rf"(?<![:\w]){re.escape(alias)}\s*::\s*{operation}\s*\(")))
    for match in re.finditer(r"\buse\s+std::fs::(?:\{([^}]*)\}|(\w+)(?:\s+as\s+(\w+))?)\s*;", code, re.S):
        items = match[1] or (match[2] + (" as " + match[3] if match[3] else ""))
        for item in items.split(","):
            part = re.fullmatch(r"\s*(\w+)(?:\s+as\s+(\w+))?\s*", item)
            if part and part[1] in FS_FUNCTIONS:
                results.append((part[1], re.compile(rf"(?<![:\w]){re.escape(part[2] or part[1])}\s*\(")))
    for operation in PATH_METHODS:
        results.append((operation, re.compile(rf"\.\s*{operation}\s*\(")))
    results.append(("open", re.compile(r"\b(?:std::fs::)?File\s*::\s*open\s*\(")))
    return results


def classify(path: str, symbol: str) -> str:
    if "/build/" in path or "/cargo/" in path or "cache" in path or "identity" in path or "manifest" in path or "projection" in path:
        return "build-identity"
    if any(part in path for part in ("/bin/", "/tests/", "_tests.rs", "/format/", "/lint/", "/lsp/", "host_tool", "cli.rs")):
        return "tooling-input"
    if any(word in symbol for word in ("write", "publish", "copy", "storage", "prune", "remove", "output")):
        return "output-effect"
    return "semantic-input"


def sites(root: Path) -> list[dict[str, str]]:
    found = []
    for path in source_paths(root):
        rel = path.relative_to(root).as_posix()
        raw = path.read_text(encoding="utf-8")
        if not any(token in raw for token in ("fs::", "File::open", ".is_file(", ".is_dir(", ".exists(", ".try_exists(", ".metadata(", ".canonicalize(", ".read_link(")):
            continue
        code = rust_code(raw)
        original_lines = raw.splitlines()
        rules = patterns(code)
        current_symbol = "<module>"
        for number, line in enumerate(code.splitlines(), 1):
            function = FUNCTION.search(line)
            if function:
                current_symbol = function[1]
            hits = []
            for operation, regex in rules:
                hits.extend((match.start(), operation) for match in regex.finditer(line))
            for _, operation in sorted(set(hits)):
                site = " ".join(original_lines[number - 1].strip().split())
                found.append({"path": rel, "symbol": current_symbol, "operation": operation,
                              "site": site, "classification": classify(rel, current_symbol), "line": number})
    return found


def compare(found: list[dict[str, str]], expected: list[dict[str, str]]) -> list[str]:
    def key(item: dict[str, str]) -> tuple[str, ...]:
        return tuple(item[field] for field in ("path", "symbol", "operation", "site", "classification"))
    actual = Counter(map(key, found))
    baseline = Counter(map(key, expected))
    failures = []
    for item, count in sorted((actual - baseline).items()):
        failures.append(f"unclassified filesystem effect ({count}): {item[0]}::{item[1]} {item[2]} {item[3]}")
    for item, count in sorted((baseline - actual).items()):
        failures.append(f"stale filesystem effect ({count}): {item[0]}::{item[1]} {item[2]} {item[3]}")
    return failures


def self_test() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        (root / "crates/demo/src/bin").mkdir(parents=True)
        (root / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/demo"]\n')
        (root / "crates/demo/Cargo.toml").write_text('[package]\nname = "demo"\nversion = "0.1.0"\n')
        source = root / "crates/demo/src/lib.rs"
        source.write_text('use std::fs;\nfn old() { let _ = fs::read_to_string("x"); }\n')
        baseline = sites(root)
        assert len(baseline) == 1
        mutations = (
            'fn new() { let _ = fs::read_to_string("y"); }\n',
            'use std::fs as disk;\nfn new() { let _ = disk::read("x"); }\n',
            'fn new() { let _ = fs::read("x"); }\n',
            'use std::fs::read as load;\nfn new() { let _ = load("x"); }\n',
        )
        for addition in mutations:
            source.write_text('use std::fs;\nfn old() { let _ = fs::read_to_string("x"); }\n' + addition)
            assert compare(sites(root), baseline), addition
        source.write_text('use std::fs;\nfn old() { let _ = fs::read_to_string("x"); }\n')
        (root / "crates/demo/src/bin/new.rs").write_text('fn main() { let _ = std::fs::read("x"); }\n')
        assert compare(sites(root), baseline)
        (root / "crates/added/src").mkdir(parents=True)
        (root / "crates/added/Cargo.toml").write_text('[package]\nname="added"\nversion="0.1.0"\n')
        (root / "crates/added/src/lib.rs").write_text('fn f() { let _ = std::fs::read("x"); }\n')
        (root / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/demo", "crates/added"]\n')
        assert compare(sites(root), baseline)
    print("direct filesystem effects self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--write-inventory", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    found = sites(ROOT)
    if args.write_inventory:
        INVENTORY.parent.mkdir(exist_ok=True)
        INVENTORY.write_text(json.dumps({"schema_version": 1, "sites": [
            {k: v for k, v in site.items() if k != "line"} for site in found
        ]}, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {len(found)} filesystem effects")
        return 0
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    if inventory.get("schema_version") != 1:
        raise ValueError("unsupported filesystem inventory schema")
    failures = compare(found, inventory["sites"])
    if failures:
        print("direct filesystem effects: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print(f"direct filesystem effects: PASS ({len(found)} sites)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
