#!/usr/bin/env python3
"""Reject new parser/lowering/type-check entrypoints outside approved crates."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
COMMON_ROOT = REPO_ROOT / "verification" / "areas" / "common"
sys.path.insert(0, str(COMMON_ROOT))

from rust_source_ranges import inline_test_module_ranges  # noqa: E402

APPROVED_PREFIXES = {
    Path("crates/sifr_syntax/src"),
    Path("crates/sifr_frontend/src"),
    Path("crates/sifr_lowering/src"),
}

SYNTAX_ONLY_CONSUMERS = {
    Path("crates/sifr/src/check_and_package_commands.rs"): (
        "declaration Python requirement discovery; this inspects syntax metadata "
        "and does not produce compiler diagnostics or semantic types"
    ),
    Path("crates/sifr_lint/src/engine.rs"): (
        "lint token and syntax-node rules; compiler semantic and HIR answers still "
        "route through sifr_frontend"
    ),
}
FORBIDDEN_PATTERNS = (
    "sifr_python_parser",
    "ruff_python_parser",
    "parse_unchecked",
    "sifr_syntax::parse_module(",
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


def is_classified_syntax_only(path: Path, pattern: str) -> bool:
    rel = path.relative_to(REPO_ROOT)
    return pattern == "sifr_syntax::parse_module(" and rel in SYNTAX_ONLY_CONSUMERS


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        if is_approved(path):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        test_ranges = inline_test_module_ranges(text)
        for pattern in FORBIDDEN_PATTERNS:
            if is_classified_syntax_only(path, pattern):
                continue
            for match in re.finditer(re.escape(pattern), text):
                if any(start <= match.start() <= end for start, end in test_ranges):
                    continue
                failures.append(
                    f"{path.relative_to(REPO_ROOT)} contains forbidden frontend pattern {pattern!r}"
                )
                break
            if failures and failures[-1].startswith(str(path.relative_to(REPO_ROOT))):
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
    seeded.write_text(
        "let parsed = sifr_syntax::parse_module(source, None);\n", encoding="utf-8"
    )
    found = violations([seeded])
    seeded.unlink()
    if not found:
        raise SystemExit(
            "split-brain guardrail self-test failed: seeded syntax entrypoint passed"
        )
    seeded.write_text(
        "#[cfg(test)]\nmod tests {\n"
        "    fn parses_fixture() { let _ = sifr_syntax::parse_module(source, None); }\n"
        "}\n",
        encoding="utf-8",
    )
    if violations([seeded]):
        seeded.unlink()
        raise SystemExit(
            "split-brain guardrail self-test failed: inline test syntax use rejected"
        )
    seeded.unlink()
    cli_path = REPO_ROOT / "crates" / "sifr" / "src" / "check_and_package_commands.rs"
    if "sifr_syntax::parse_module(" not in cli_path.read_text(encoding="utf-8"):
        raise SystemExit(
            "split-brain guardrail self-test failed: classified CLI syntax use missing"
        )
    lint_path = REPO_ROOT / "crates" / "sifr_lint" / "src" / "engine.rs"
    if violations([cli_path, lint_path]):
        raise SystemExit(
            "split-brain guardrail self-test failed: classified syntax-only CLI use rejected"
        )
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
