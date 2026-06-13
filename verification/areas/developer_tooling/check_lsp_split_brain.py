#!/usr/bin/env python3
"""Reject semantics-bearing implementation paths inside sifr_lsp."""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
LSP_ROOT = REPO_ROOT / "crates" / "sifr_lsp"

FORBIDDEN_PATTERNS = {
    "sifr_python_parser": "LSP must not call the raw parser",
    "ruff_python_parser": "LSP must not call the raw parser",
    "parse_unchecked": "LSP must not call parser internals",
    "parse_module_with_diagnostics": "LSP must route parse diagnostics through analysis/frontend",
    "lower_module(": "LSP must not lower HIR directly",
    "lower_frontend_module": "LSP must not lower HIR directly",
    "type_check": "LSP must not type-check directly",
    "HirModule": "LSP must not traverse HIR for semantic answers",
    "sifr_codegen::": "LSP must not call codegen directly",
}

ALLOWED_SNIPPETS = {
    "AnalysisHost::generated_rust_preview",
}


def rust_files() -> list[Path]:
    if not LSP_ROOT.exists():
        return []
    return sorted(path for path in LSP_ROOT.rglob("*.rs") if "target" not in path.parts)


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        for line_number, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
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
        seed = Path(tmp) / "handler.rs"
        seed.write_text("use sifr_lowering::HirModule;\nfn handler() { lower_module(&[]); }\n", encoding="utf-8")
        found = violations([seed])
    if not found:
        raise SystemExit("LSP split-brain self-test failed: seeded direct HIR path passed")
    print("LSP split-brain self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    found = violations(rust_files())
    if found:
        print("LSP split-brain: FAIL", file=sys.stderr)
        for failure in found:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("LSP split-brain: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
