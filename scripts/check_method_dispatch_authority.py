#!/usr/bin/env python3
"""Ratchet typed method-name dispatch and its scoped codegen authority."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys
import tempfile
from typing import Any

from rust_source_policy import is_test_path, mask_rust_comments, strip_cfg_test_modules


REPO_ROOT = Path(__file__).resolve().parents[1]
INVENTORY_PATH = REPO_ROOT / "verification" / "policy" / "method_dispatch_authority.json"
SCAN_ROOTS = (
    Path("crates/sifr_lowering/src"),
    Path("crates/sifr_codegen/src"),
)
AUTHORITY_PATH = Path("crates/sifr_codegen/src/methods/authority.rs")
VALID_CATEGORIES = {
    "language-semantics",
    "typed-hir",
    "contextual-codegen",
    "rust-ir-consumer",
}
DISPATCH_PATTERNS = (
    re.compile(r'\b(?:method|method_name)\s*==\s*"'),
    re.compile(r'"[^"\\]*(?:\\.[^"\\]*)*"\s*==\s*(?:method|method_name)\b'),
    re.compile(r"\bmatch\s+(?:method|method_name)(?:\.as_str\(\))?\s*\{"),
    re.compile(r"matches!\s*\(\s*(?:method|method_name)(?:\.as_str\(\))?\s*,"),
    re.compile(r"\bmatch\s*\([^)]*\bmethod\b[^)]*\)\s*\{"),
)


def production_text(path: Path) -> str:
    return mask_rust_comments(strip_cfg_test_modules(path.read_text(encoding="utf-8")))


def dispatch_count(text: str) -> int:
    return sum(len(pattern.findall(text)) for pattern in DISPATCH_PATTERNS)


def scan_dispatches(root: Path) -> dict[str, int]:
    found: dict[str, int] = {}
    for relative_root in SCAN_ROOTS:
        source_root = root / relative_root
        for path in sorted(source_root.rglob("*.rs")):
            relative = path.relative_to(root)
            if is_test_path(relative):
                continue
            count = dispatch_count(production_text(path))
            if count:
                found[relative.as_posix()] = count
    return found


def load_inventory(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("inventory root must be an object")
    return payload


def flatten_inventory(payload: dict[str, Any], errors: list[str]) -> dict[str, tuple[str, int]]:
    if payload.get("schema_version") != 1:
        errors.append("method dispatch inventory schema_version must be 1")
    categories = payload.get("categories")
    if not isinstance(categories, dict):
        errors.append("method dispatch inventory categories must be an object")
        return {}
    unknown = sorted(set(categories) - VALID_CATEGORIES)
    if unknown:
        errors.append(f"method dispatch inventory has unknown categories: {unknown}")
    flattened: dict[str, tuple[str, int]] = {}
    for category, rows in categories.items():
        if category not in VALID_CATEGORIES or not isinstance(rows, dict):
            if not isinstance(rows, dict):
                errors.append(f"method dispatch category {category} must be an object")
            continue
        for path, count in rows.items():
            if path in flattened:
                errors.append(f"method dispatch path appears in multiple categories: {path}")
                continue
            if not isinstance(count, int) or isinstance(count, bool) or count <= 0:
                errors.append(f"method dispatch count must be positive for {path}")
                continue
            flattened[path] = (category, count)
    return flattened


def validate(root: Path, payload: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    expected = flatten_inventory(payload, errors)
    actual = scan_dispatches(root)
    authority = AUTHORITY_PATH.as_posix()
    if expected.get(authority, (None, 0))[0] != "language-semantics":
        errors.append(f"{authority} must own the language-semantics category")
    for path, (category, _) in expected.items():
        if category == "language-semantics" and path != authority:
            errors.append(f"language semantics escaped the scoped authority: {path}")
        if category == "typed-hir" and not path.startswith("crates/sifr_lowering/src/"):
            errors.append(f"typed-hir dispatch is outside lowering: {path}")
    for path, count in actual.items():
        row = expected.get(path)
        if row is None:
            errors.append(f"unclassified method-name dispatch: {path} ({count} sites)")
        elif row[1] != count:
            errors.append(
                f"method-name dispatch count changed for {path}: inventory={row[1]} actual={count}"
            )
    for path in sorted(set(expected) - set(actual)):
        errors.append(f"stale method-name dispatch inventory entry: {path}")
    return errors


def run_self_test() -> int:
    with tempfile.TemporaryDirectory(prefix="method-dispatch-", dir=REPO_ROOT / "target") as tmp:
        root = Path(tmp)
        authority = root / AUTHORITY_PATH
        authority.parent.mkdir(parents=True)
        authority.write_text('fn lower(method: &str) { match method { "len" => (), _ => () } }\n')
        baseline = {
            "schema_version": 1,
            "categories": {
                "language-semantics": {AUTHORITY_PATH.as_posix(): 1},
                "typed-hir": {},
                "contextual-codegen": {},
                "rust-ir-consumer": {},
            },
        }
        if validate(root, baseline):
            print("method dispatch self-test baseline failed", file=sys.stderr)
            return 1
        unclassified = root / "crates/sifr_lowering/src/new_dispatch.rs"
        unclassified.parent.mkdir(parents=True, exist_ok=True)
        unclassified.write_text('fn f(method_name: &str) { if method_name == "new" {} }\n')
        if not any("unclassified" in error for error in validate(root, baseline)):
            print("method dispatch self-test missed unclassified site", file=sys.stderr)
            return 1
        unclassified.unlink()
        authority.write_text(
            'fn lower(method: &str) { if method == "len" {} if method == "count" {} }\n'
        )
        if not any("count changed" in error for error in validate(root, baseline)):
            print("method dispatch self-test missed count growth", file=sys.stderr)
            return 1
        misplaced = json.loads(json.dumps(baseline))
        misplaced["categories"]["language-semantics"] = {}
        misplaced["categories"]["contextual-codegen"] = {AUTHORITY_PATH.as_posix(): 2}
        if not any("must own" in error for error in validate(root, misplaced)):
            print("method dispatch self-test missed authority move", file=sys.stderr)
            return 1
    print("method dispatch authority self-test: PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()
    try:
        inventory = load_inventory(INVENTORY_PATH)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"method dispatch authority: {error}", file=sys.stderr)
        return 1
    errors = validate(REPO_ROOT, inventory)
    if errors:
        for error in errors:
            print(f"method dispatch authority error: {error}", file=sys.stderr)
        return 1
    print(f"method dispatch authority: PASS ({len(scan_dispatches(REPO_ROOT))} classified files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
