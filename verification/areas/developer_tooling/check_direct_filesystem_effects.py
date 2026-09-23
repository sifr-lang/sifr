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
READ_FUNCTIONS = ("read_to_string", "read_dir", "read_link", "symlink_metadata",
                  "try_exists", "canonicalize", "metadata", "read")
WRITE_FUNCTIONS = ("write", "copy", "rename", "remove_file", "remove_dir",
                   "remove_dir_all", "create_dir", "create_dir_all", "set_permissions",
                   "hard_link", "soft_link")
FS_FUNCTIONS = READ_FUNCTIONS + WRITE_FUNCTIONS
PATH_METHODS = ("is_file", "is_dir", "exists", "try_exists", "metadata",
                "canonicalize", "read_link")
FILE_FUNCTIONS = ("open", "create", "create_new")
METHOD_WRITES = ("set_len", "set_times", "set_permissions")
WRITE_OPERATIONS = set(WRITE_FUNCTIONS) | set(METHOD_WRITES) | {"create", "create_new", "open-write", "dir-create"}

CLASSIFICATIONS = {"semantic-input", "build-identity", "tooling-input", "output-effect"}
FUNCTION = re.compile(r"\bfn\s+(\w+)")
RAW_START = re.compile(r'r(#{0,16})"')


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
            raw = RAW_START.match(text, i)
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
    file_aliases = {"File", "std::fs::File"}
    options_aliases = {"OpenOptions", "std::fs::OpenOptions"}
    directory_aliases = {"DirBuilder", "std::fs::DirBuilder"}
    for match in re.finditer(r"\buse\s+std::fs::(?:\{([^}]*)\}|(File|OpenOptions|DirBuilder)(?:\s+as\s+(\w+))?)\s*;", code, re.S):
        items = match[1] or (match[2] + (" as " + match[3] if match[3] else ""))
        for item in items.split(","):
            part = re.fullmatch(r"\s*(File|OpenOptions|DirBuilder)(?:\s+as\s+(\w+))?\s*", item)
            if part:
                ({"File": file_aliases, "OpenOptions": options_aliases,
                 "DirBuilder": directory_aliases}[part[1]]).add(part[2] or part[1])
    for alias in aliases:
        file_aliases.add(f"{alias}::File")
        options_aliases.add(f"{alias}::OpenOptions")
        directory_aliases.add(f"{alias}::DirBuilder")
    for alias in file_aliases:
        for operation in FILE_FUNCTIONS:
            results.append((operation, re.compile(rf"(?<![:\w]){re.escape(alias)}\s*::\s*{operation}\s*\(")))
    for alias in file_aliases:
        results.append(("open-options", re.compile(rf"(?<![:\w]){re.escape(alias)}\s*::\s*options\s*\(")))
    for alias in options_aliases:
        results.append(("open-options", re.compile(rf"(?<![:\w]){re.escape(alias)}\s*::\s*new\s*\(")))
    for alias in directory_aliases:
        results.append(("dir-builder", re.compile(rf"(?<![:\w]){re.escape(alias)}\s*::\s*new\s*\(")))
    for operation in METHOD_WRITES:
        results.append((operation, re.compile(rf"\.\s*{operation}\s*\(")))
    return results


def classify(path: str, operation: str) -> str:
    if operation in WRITE_OPERATIONS:
        return "output-effect"
    if "/build/" in path or "/cargo/" in path or "cache" in path or "identity" in path or "manifest" in path or "projection" in path:
        return "build-identity"
    if any(part in path for part in ("/bin/", "/tests/", "_tests.rs", "/format/", "/lint/", "/lsp/", "host_tool", "cli.rs")):
        return "tooling-input"
    return "semantic-input"


def sites(root: Path) -> list[dict[str, str]]:
    found = []
    for path in source_paths(root):
        rel = path.relative_to(root).as_posix()
        raw = path.read_text(encoding="utf-8")
        code = rust_code(raw)
        original_lines = raw.splitlines()
        rules = patterns(code)
        current_symbol = "<module>"
        # A builder's final open can be on another line. Track its chain, not
        # just the line containing OpenOptions::new().
        builder = None
        builder_writes = False
        for number, line in enumerate(code.splitlines(), 1):
            function = FUNCTION.search(line)
            if function:
                current_symbol = function[1]
                builder = None
                builder_writes = False
            if any(operation == "open-options" and regex.search(line)
                   for operation, regex in rules):
                builder = "options"
                builder_writes = False
            if any(operation == "dir-builder" and regex.search(line)
                   for operation, regex in rules):
                builder = "directory"
            if builder and re.search(r"\.(?:write|append|create|create_new|truncate)\s*\(\s*true\s*\)", line):
                builder_writes = True
            hits = []
            for operation, regex in rules:
                if operation not in {"open-options", "dir-builder"}:
                    hits.extend((match.start(), operation) for match in regex.finditer(line))
            if builder == "options":
                hits.extend((match.start(), "open-write" if builder_writes else "open")
                            for match in re.finditer(r"\.\s*open\s*\(", line))
                if re.search(r"\.\s*open\s*\(", line):
                    builder = None
            elif builder == "directory":
                hits.extend((match.start(), "dir-create")
                            for match in re.finditer(r"\.\s*create\s*\(", line))
                if re.search(r"\.\s*create\s*\(", line):
                    builder = None
            for _, operation in sorted(set(hits)):
                site = " ".join(original_lines[number - 1].strip().split())
                found.append({"path": rel, "symbol": current_symbol, "operation": operation,
                              "site": site, "classification": classify(rel, operation), "line": number})
    return found


def compare(found: list[dict[str, str]], expected: list[dict[str, str]]) -> list[str]:
    def key(item: dict[str, str]) -> tuple[str, ...]:
        return tuple(item[field] for field in ("path", "symbol", "operation", "site"))
    actual = Counter(map(key, found))
    baseline = Counter(map(key, expected))
    failures = [f"invalid effect classification: {item.get('path')}" for item in expected
                if item.get("classification") not in CLASSIFICATIONS]
    failures.extend(f"effect classification drift: {item.get('path')}::{item.get('symbol')} {item.get('operation')}"
                    for item in expected if item.get("classification") !=
                    classify(item.get("path", ""), item.get("operation", "")))
    for item, count in sorted((actual - baseline).items()):
        failures.append(f"unclassified filesystem effect ({count}): {item[0]}::{item[1]} {item[2]} {item[3]}")
    for item, count in sorted((baseline - actual).items()):
        failures.append(f"stale filesystem effect ({count}): {item[0]}::{item[1]} {item[2]} {item[3]}")
    return failures


def self_test() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        crate = root / "crates/demo"
        (crate / "src/bin").mkdir(parents=True)
        (root / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/demo"]\n')
        (crate / "Cargo.toml").write_text('[package]\nname = "demo"\nversion = "0.1.0"\n')
        source = crate / "src/lib.rs"
        seed = 'use std::fs;\nfn old() { let _ = fs::read_to_string("x"); }\n'
        source.write_text(seed)
        baseline = sites(root)
        assert len(baseline) == 1
        mutations = (
            'fn new() { let _ = fs::read_to_string("y"); }\n',
            'fn new() { let _ = fs::read("x"); }\n',
            'fn new() { let _ = std::fs::write("x", b"y"); }\n',
            'fn new() { let _ = std::fs::remove_dir_all("x"); }\n',
            'fn new() { let _ = std::fs::File::create("x"); }\n',
            'fn new() { let _ = std::fs::OpenOptions::new().read(true).open("x"); }\n',
            'fn new() { let _ = std::fs::OpenOptions::new().write(true).open("x"); }\n',
            'fn new() { let _ = std::fs::File::options().write(true).open("x"); }\n',
            'fn new() { let _ = std::fs::DirBuilder::new().create("x"); }\n',
            'fn new(file: std::fs::File) { let _ = file.set_len(0); }\n',
            'fn new(file: std::fs::File) { let _ = file.set_times(std::fs::FileTimes::new()); }\n',
            'fn new(path: std::path::PathBuf, permissions: std::fs::Permissions) { let _ = path.set_permissions(permissions); }\n',
        )
        for addition in mutations:
            source.write_text(seed + addition)
            assert compare(sites(root), baseline), addition
        alias_only = 'use std::fs as disk;\nfn alias_only() { let _ = disk::read("x"); }\n'
        source.write_text(alias_only)
        assert len(sites(root)) == 1, "alias-only source escaped"
        assert compare(sites(root), baseline)
        source.write_text('use std::fs::read as load;\nfn alias_only() { let _ = load("x"); }\n')
        assert len(sites(root)) == 1, "imported function alias escaped"
        source.write_text(seed)
        (crate / "src/bin/new.rs").write_text('fn main() { let _ = std::fs::read("x"); }\n')
        assert compare(sites(root), baseline)
        (root / "crates/added/src").mkdir(parents=True)
        (root / "crates/added/Cargo.toml").write_text('[package]\nname="added"\nversion="0.1.0"\n')
        (root / "crates/added/src/lib.rs").write_text('fn f() { let _ = std::fs::read("x"); }\n')
        (root / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/demo", "crates/added"]\n')
        assert compare(sites(root), baseline)
        assert any(item["path"] == "crates/added/src/lib.rs" for item in sites(root))
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
