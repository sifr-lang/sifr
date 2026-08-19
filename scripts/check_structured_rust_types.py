#!/usr/bin/env python3
"""Reject Sifr type-to-Rust string rendering outside the Rust IR renderer."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE_ROOTS = (
    ROOT / "crates" / "sifr_codegen" / "src",
    ROOT / "crates" / "sifr_lowering" / "src",
    ROOT / "crates" / "sifr_type_system" / "src",
)

DIRECT_PATTERNS = {
    "Type method call": re.compile(r"\.rust_type\s*\("),
    "Type method reference": re.compile(r"\bType::rust_type\b"),
    "field renderer": re.compile(r"rust_type_for_struct_field"),
    "Type renderer definition": re.compile(r"\bfn\s+rust_type\s*\("),
}

CONVERTER_PATTERNS = {
    "renderer call in conversion": re.compile(r"\b(?:render_type|render_type_string)\s*\("),
    "display fallback in conversion": re.compile(r"\.display_name\s*\("),
    "wildcard conversion fallback": re.compile(
        r"\b_\s*=>\s*RustType::Named", re.MULTILINE
    ),
    "rendered Named fallback": re.compile(
        r"RustType::Named\s*\([^\n)]*(?:render_|display_name)"
    ),
}


def rust_sources() -> dict[str, str]:
    return {
        str(path.relative_to(ROOT)): path.read_text(encoding="utf-8")
        for source_root in SOURCE_ROOTS
        for path in source_root.rglob("*.rs")
    }


def violations(sources: dict[str, str]) -> list[str]:
    problems: list[str] = []
    for path, source in sources.items():
        for label, pattern in DIRECT_PATTERNS.items():
            if pattern.search(source):
                problems.append(f"{path}: {label}")

    converter_path = "crates/sifr_codegen/src/preamble/types_and_errors.rs"
    converter_source = sources.get(converter_path, "")
    if not re.search(r"\bpub\s+fn\s+sifr_type_to_rust_type\s*\(", converter_source):
        problems.append(f"{converter_path}: canonical converter is missing")
    else:
        for label, pattern in CONVERTER_PATTERNS.items():
            if pattern.search(converter_source):
                problems.append(f"{converter_path}: {label}")
    return problems


def run_self_test() -> None:
    converter_path = "crates/sifr_codegen/src/preamble/types_and_errors.rs"
    seed = {
        converter_path: """
pub fn sifr_type_to_rust_type(ty: &Type) -> RustType {
    match ty { Type::Int => RustType::I64 }
}
""",
        "crates/sifr_type_system/src/types/type_queries.rs": "impl Type {}",
    }
    if violations(seed):
        raise SystemExit("structured Rust type guard self-test rejected the valid seed")

    mutations = {
        "direct call": "fn bad(ty: &Type) { ty.rust_type(); }",
        "indirect display fallback": """
pub fn sifr_type_to_rust_type(ty: &Type) -> RustType {
    RustType::Named(ty.display_name())
}
""",
        "wildcard fallback": """
pub fn sifr_type_to_rust_type(ty: &Type) -> RustType {
    match ty { _ => RustType::Named(String::new()) }
}
""",
    }
    for label, mutation in mutations.items():
        candidate = dict(seed)
        if label == "direct call":
            candidate["crates/sifr_codegen/src/bad.rs"] = mutation
        else:
            candidate[converter_path] = mutation
        if not violations(candidate):
            raise SystemExit(f"structured Rust type guard missed {label}")
    print("structured Rust type guard self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0
    problems = violations(rust_sources())
    if problems:
        for problem in problems:
            print(problem)
        return 1
    print("structured Rust type guard: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
