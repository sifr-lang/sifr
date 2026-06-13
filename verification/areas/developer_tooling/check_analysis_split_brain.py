#!/usr/bin/env python3
"""Reject semantic bypasses in the Phase 36 analysis crate."""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
ANALYSIS_ROOT = REPO_ROOT / "crates" / "sifr_analysis"

FORBIDDEN_PATTERNS = {
    "sifr_python_parser": "analysis must not call the raw parser",
    "ruff_python_parser": "analysis must not call the raw parser",
    "parse_unchecked": "analysis must not call parser internals",
    "sifr_syntax::parse_module": "analysis must route parser work through frontend or formatter handoff",
    "parse_module_raw": "analysis must not call parser internals",
    "lower_module(": "analysis must not lower HIR directly",
    "compile_module_hir": "analysis must not type-check/lower directly",
    "HirModule": "analysis must not traverse HIR for semantic answers",
    "sifr_codegen::": "generated Rust preview must use a reviewed compiler handoff",
    "ty_python_semantic": "Python semantic authority is forbidden",
    "ty_project": "Python project semantics are forbidden",
}

ALLOWED_SNIPPETS = {
    "sifr_format::format_range",
}


def rust_files() -> list[Path]:
    if not ANALYSIS_ROOT.exists():
        return []
    return sorted(path for path in ANALYSIS_ROOT.rglob("*.rs") if "target" not in path.parts)


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8", errors="replace")
        for line_number, line in enumerate(text.splitlines(), 1):
            if any(snippet in line for snippet in ALLOWED_SNIPPETS):
                continue
            for pattern, reason in FORBIDDEN_PATTERNS.items():
                if pattern in line:
                    failures.append(
                        f"{path.relative_to(REPO_ROOT)}:{line_number} contains {pattern!r}: {reason}"
                    )
                    break
    return failures


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        seed = Path(tmp) / "analysis_bypass.rs"
        seed.write_text(
            "use sifr_lowering::HirModule;\nfn bypass() { lower_module(&[]); }\n",
            encoding="utf-8",
        )
        found = violations([seed])
    if not found:
        raise SystemExit("analysis split-brain self-test failed: seeded semantic bypass passed")
    print("analysis split-brain self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    found = violations(rust_files())
    if found:
        print("analysis split-brain: FAIL", file=sys.stderr)
        for failure in found:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("analysis split-brain: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
