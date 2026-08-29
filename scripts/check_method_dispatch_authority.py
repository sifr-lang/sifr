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

from rust_source_policy import (
    is_test_path,
    mask_rust_comments,
    mask_rust_non_code,
    strip_cfg_test_modules,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
INVENTORY_PATH = REPO_ROOT / "verification" / "policy" / "method_dispatch_authority.json"
SCAN_ROOTS = (
    Path("crates/sifr_frontend/src"),
    Path("crates/sifr_lowering/src"),
    Path("crates/sifr_codegen/src"),
)
AUTHORITY_PATH = Path("crates/sifr_codegen/src/methods/authority.rs")
VALID_CATEGORIES = {
    "compile-time-semantics",
    "language-semantics",
    "typed-hir",
    "contextual-codegen",
    "rust-ir-consumer",
}
STRING_COMPARISON = re.compile(
    r'(?:(?P<left>\b[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*(?:\.as_str\(\))?)\s*==\s*"(?P<right_literal>[A-Za-z_]\w*)"|'
    r'"(?P<left_literal>[A-Za-z_]\w*)"\s*==\s*(?P<right>\b[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*(?:\.as_str\(\))?))'
)
SIMPLE_MATCH = re.compile(
    r"\b(?P<kind>match|matches!)\s*\(?\s*(?P<binding>[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*(?:\.as_str\(\))?)"
)
TUPLE_METHOD_MATCH = re.compile(r"\bmatch\s*\([^)]*\b[A-Za-z_]\w*method\w*\b[^)]*\)\s*\{")
FUNCTION = re.compile(r"\bfn\s+([A-Za-z_]\w*)\s*(?:<[^>{}]*>)?\s*\(")
STRING_LITERAL = re.compile(r'"([A-Za-z_]\w*)"')


def production_text(path: Path) -> str:
    return mask_rust_comments(strip_cfg_test_modules(path.read_text(encoding="utf-8")))


def source_method_names(root: Path) -> set[str]:
    authority = production_text(root / AUTHORITY_PATH)
    return set(STRING_LITERAL.findall(authority))


def enclosing_function(text: str, offset: int) -> str:
    functions = list(FUNCTION.finditer(text, 0, offset))
    return functions[-1].group(1) if functions else "<module>"


def matching_delimiter(code: str, opening: int, left: str, right: str) -> int | None:
    depth = 0
    for index in range(opening, len(code)):
        if code[index] == left:
            depth += 1
        elif code[index] == right:
            depth -= 1
            if depth == 0:
                return index
    return None


def method_branch_shape(text: str, code: str, match: re.Match[str], method_names: set[str]) -> str:
    if match.group("kind") == "matches!":
        opening = code.find("(", match.start(), match.end())
        closing = matching_delimiter(code, opening, "(", ")") if opening >= 0 else None
    else:
        opening = code.find("{", match.end(), min(len(code), match.end() + 256))
        closing = matching_delimiter(code, opening, "{", "}") if opening >= 0 else None
    if opening < 0 or closing is None:
        return "$method"
    literals = [
        literal
        for literal in STRING_LITERAL.findall(text[opening : closing + 1])
        if literal in method_names
    ]
    return "|".join(dict.fromkeys(literals)) or "$method"


def dispatch_sites(text: str, method_names: set[str]) -> list[str]:
    raw_sites: list[tuple[int, str]] = []
    code = mask_rust_non_code(text)
    for match in STRING_COMPARISON.finditer(text):
        literal = match.group("right_literal") or match.group("left_literal")
        if literal not in method_names:
            continue
        raw_sites.append(
            (match.start(), f"{enclosing_function(text, match.start())}:compare:{literal}")
        )
    for match in SIMPLE_MATCH.finditer(code):
        binding = match.group("binding")
        branch_shape = method_branch_shape(text, code, match, method_names)
        if (
            "method" not in binding
            and not binding.endswith(".name")
            and branch_shape == "$method"
        ):
            continue
        raw_sites.append(
            (
                match.start(),
                f"{enclosing_function(code, match.start())}:{match.group('kind')}:{branch_shape}",
            )
        )
    for match in TUPLE_METHOD_MATCH.finditer(code):
        opening = code.find("{", match.start(), match.end())
        closing = matching_delimiter(code, opening, "{", "}") if opening >= 0 else None
        span = text[opening : closing + 1] if opening >= 0 and closing is not None else ""
        literals = [literal for literal in STRING_LITERAL.findall(span) if literal in method_names]
        shape = "|".join(dict.fromkeys(literals)) or "$method"
        raw_sites.append(
            (match.start(), f"{enclosing_function(code, match.start())}:match-tuple:{shape}")
        )
    occurrences: dict[str, int] = {}
    fingerprints = []
    for _, base in sorted(raw_sites):
        occurrences[base] = occurrences.get(base, 0) + 1
        fingerprints.append(f"{base}#{occurrences[base]}")
    return fingerprints


def scan_dispatches(root: Path) -> dict[str, list[str]]:
    found: dict[str, list[str]] = {}
    method_names = source_method_names(root)
    for relative_root in SCAN_ROOTS:
        source_root = root / relative_root
        for path in sorted(source_root.rglob("*.rs")):
            relative = path.relative_to(root)
            if is_test_path(relative):
                continue
            sites = dispatch_sites(production_text(path), method_names)
            if sites:
                found[relative.as_posix()] = sites
    return found


def load_inventory(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("inventory root must be an object")
    return payload


def flatten_inventory(payload: dict[str, Any], errors: list[str]) -> dict[str, tuple[str, list[str]]]:
    if payload.get("schema_version") != 2:
        errors.append("method dispatch inventory schema_version must be 2")
    categories = payload.get("categories")
    if not isinstance(categories, dict):
        errors.append("method dispatch inventory categories must be an object")
        return {}
    unknown = sorted(set(categories) - VALID_CATEGORIES)
    if unknown:
        errors.append(f"method dispatch inventory has unknown categories: {unknown}")
    flattened: dict[str, tuple[str, list[str]]] = {}
    for category, rows in categories.items():
        if category not in VALID_CATEGORIES or not isinstance(rows, dict):
            if not isinstance(rows, dict):
                errors.append(f"method dispatch category {category} must be an object")
            continue
        for path, sites in rows.items():
            if path in flattened:
                errors.append(f"method dispatch path appears in multiple categories: {path}")
                continue
            if (
                not isinstance(sites, list)
                or not sites
                or not all(isinstance(site, str) and site for site in sites)
                or len(sites) != len(set(sites))
            ):
                errors.append(f"method dispatch sites must be unique non-empty strings for {path}")
                continue
            flattened[path] = (category, sites)
    return flattened


def validate(root: Path, payload: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    expected = flatten_inventory(payload, errors)
    actual = scan_dispatches(root)
    authority = AUTHORITY_PATH.as_posix()
    if expected.get(authority, (None, []))[0] != "language-semantics":
        errors.append(f"{authority} must own the language-semantics category")
    for path, (category, _) in expected.items():
        if category == "language-semantics" and path != authority:
            errors.append(f"language semantics escaped the scoped authority: {path}")
        if category == "typed-hir" and not path.startswith("crates/sifr_lowering/src/"):
            errors.append(f"typed-hir dispatch is outside lowering: {path}")
        if category == "compile-time-semantics" and not path.startswith(
            "crates/sifr_frontend/src/const_evaluator"
        ):
            errors.append(f"compile-time method semantics escaped const evaluation: {path}")
    for path, sites in actual.items():
        row = expected.get(path)
        if row is None:
            errors.append(f"unclassified method-name dispatch: {path} ({len(sites)} sites)")
        elif row[1] != sites:
            errors.append(
                f"method-name dispatch sites changed for {path}: inventory={row[1]} actual={sites}"
            )
    for path in sorted(set(expected) - set(actual)):
        errors.append(f"stale method-name dispatch inventory entry: {path}")
    return errors


def run_self_test() -> int:
    with tempfile.TemporaryDirectory(prefix="method-dispatch-") as tmp:
        root = Path(tmp)
        authority = root / AUTHORITY_PATH
        authority.parent.mkdir(parents=True)
        authority.write_text(
            "const QUOTE: char = '\"';\nconst BYTE_QUOTE: u8 = b'\"';\n"
            'fn lower(method: &str) { match method { "len" => (), _ => () } }\n'
        )
        baseline_sites = scan_dispatches(root)[AUTHORITY_PATH.as_posix()]
        baseline = {
            "schema_version": 2,
            "categories": {
                "language-semantics": {AUTHORITY_PATH.as_posix(): baseline_sites},
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
        unclassified.write_text('fn f(candidate: &str) { if candidate == "len" {} }\n')
        if not any("unclassified" in error for error in validate(root, baseline)):
            print("method dispatch self-test missed unclassified site", file=sys.stderr)
            return 1
        unclassified.unlink()
        authority.write_text(
            'fn lower(method: &str) { if method == "len" {} if method == "count" {} }\n'
        )
        if not any("sites changed" in error for error in validate(root, baseline)):
            print("method dispatch self-test missed site growth", file=sys.stderr)
            return 1
        authority.write_text(
            "const QUOTE: char = '\"';\nconst BYTE_QUOTE: u8 = b'\"';\n"
            'fn lower(candidate: &str) { match candidate { "len" => (), _ => () } }\n'
        )
        if validate(root, baseline):
            print("method dispatch self-test fingerprint depends on binding name", file=sys.stderr)
            return 1
        misplaced = json.loads(json.dumps(baseline))
        misplaced["categories"]["language-semantics"] = {}
        misplaced["categories"]["contextual-codegen"] = {
            AUTHORITY_PATH.as_posix(): baseline_sites
        }
        if not any("must own" in error for error in validate(root, misplaced)):
            print("method dispatch self-test missed authority move", file=sys.stderr)
            return 1
    print("method dispatch authority self-test: PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()
    try:
        inventory = load_inventory(INVENTORY_PATH)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"method dispatch authority: {error}", file=sys.stderr)
        return 1
    if args.write:
        previous_categories = {
            path: category
            for category, rows in inventory.get("categories", {}).items()
            if category in VALID_CATEGORIES and isinstance(rows, dict)
            for path in rows
        }
        actual = scan_dispatches(REPO_ROOT)
        categories: dict[str, dict[str, list[str]]] = {
            category: {} for category in sorted(VALID_CATEGORIES)
        }
        for path, sites in sorted(actual.items()):
            category = previous_categories.get(path)
            if category is None:
                if path.startswith("crates/sifr_frontend/src/const_evaluator"):
                    category = "compile-time-semantics"
                elif path.startswith("crates/sifr_lowering/src/"):
                    category = "typed-hir"
                else:
                    category = "contextual-codegen"
            if path == AUTHORITY_PATH.as_posix():
                category = "language-semantics"
            categories[category][path] = sites
        INVENTORY_PATH.write_text(
            json.dumps({"schema_version": 2, "categories": categories}, indent=2, sort_keys=True)
            + "\n",
            encoding="utf-8",
        )
        print(f"updated {INVENTORY_PATH.relative_to(REPO_ROOT)}")
        return 0
    errors = validate(REPO_ROOT, inventory)
    if errors:
        for error in errors:
            print(f"method dispatch authority error: {error}", file=sys.stderr)
        return 1
    print(f"method dispatch authority: PASS ({len(scan_dispatches(REPO_ROOT))} classified files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
