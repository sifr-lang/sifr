#!/usr/bin/env python3
"""Reject new parser/lowering/type-check entrypoints outside approved crates."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

APPROVED_PREFIXES = {
    Path("crates/sifr_syntax/src"),
    Path("crates/sifr_frontend/src"),
    Path("crates/sifr_lowering/src"),
}

FORBIDDEN_PATTERNS = (
    "sifr_python_parser",
    "ruff_python_parser",
    "parse_unchecked",
    "parse_module_with_diagnostics",
    "lower_module_with_externals",
    "lower_module(",
    "lower_frontend_module",
)


def is_approved(path: Path) -> bool:
    rel = path.relative_to(REPO_ROOT)
    if "tests" in rel.parts or rel.name.endswith("_tests.rs") or "_tests_" in rel.name:
        return True
    return any(rel.is_relative_to(prefix) for prefix in APPROVED_PREFIXES)


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        if is_approved(path):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for pattern in FORBIDDEN_PATTERNS:
            if pattern in text:
                failures.append(f"{path.relative_to(REPO_ROOT)} contains forbidden frontend pattern {pattern!r}")
                break
    return failures


def rust_files() -> list[Path]:
    return [
        path
        for path in (REPO_ROOT / "crates").rglob("*.rs")
        if "target" not in path.parts
    ]


def run_self_test() -> None:
    seeded = REPO_ROOT / "target" / "performance" / "split_brain_seed.rs"
    seeded.parent.mkdir(parents=True, exist_ok=True)
    seeded.write_text("use sifr_python_parser::parse_module;\n", encoding="utf-8")
    found = violations([seeded])
    seeded.unlink()
    if not found:
        raise SystemExit("split-brain guardrail self-test failed: seeded parser entrypoint passed")
    print("split-brain guardrail self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    found = violations(rust_files())
    if found:
        print("split-brain guardrail: FAIL", file=sys.stderr)
        for failure in found:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("split-brain guardrail: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
