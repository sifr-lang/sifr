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
    Path("crates/sifr_format/src/lib.rs"): (
        "formatter round-trip validation; syntax reparsing verifies that formatted "
        "text and range edits remain parseable"
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
SYNTAX_PARSE_IMPORT_PATTERN = re.compile(
    r"\buse\s+sifr_syntax(?:::\{[^;]*\bparse_module\b(?:\s+as\s+\w+)?[^;]*\}|::parse_module\b)",
    re.DOTALL,
)
SYNTAX_PARSE_IMPORT_LABEL = "sifr_syntax parse_module import"
SYNTAX_WILDCARD_IMPORT_PATTERN = re.compile(r"\buse\s+sifr_syntax::\*\s*;")
SYNTAX_WILDCARD_IMPORT_LABEL = "sifr_syntax wildcard import"
SYNTAX_CRATE_ALIAS_PATTERN = re.compile(r"\buse\s+sifr_syntax\s+as\s+(\w+)\s*;")


def is_approved(path: Path) -> bool:
    rel = path.relative_to(REPO_ROOT)
    if "tests" in rel.parts or rel.name.endswith("_tests.rs") or "_tests_" in rel.name:
        return True
    return any(rel.is_relative_to(prefix) for prefix in APPROVED_PREFIXES)


def is_classified_syntax_only(path: Path, pattern: str) -> bool:
    rel = path.relative_to(REPO_ROOT)
    return pattern in {
        "sifr_syntax::parse_module(",
        SYNTAX_PARSE_IMPORT_LABEL,
    } and rel in SYNTAX_ONLY_CONSUMERS


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        if is_approved(path):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        test_ranges = inline_test_module_ranges(text)
        rules = [
            (pattern, re.compile(re.escape(pattern))) for pattern in FORBIDDEN_PATTERNS
        ]
        rules.append((SYNTAX_PARSE_IMPORT_LABEL, SYNTAX_PARSE_IMPORT_PATTERN))
        rules.append((SYNTAX_WILDCARD_IMPORT_LABEL, SYNTAX_WILDCARD_IMPORT_PATTERN))
        for alias in SYNTAX_CRATE_ALIAS_PATTERN.findall(text):
            rules.append(
                (
                    f"sifr_syntax alias-qualified parse_module ({alias})",
                    re.compile(rf"\b{re.escape(alias)}::parse_module\s*\("),
                )
            )
        for pattern, regex in rules:
            if is_classified_syntax_only(path, pattern):
                continue
            matched = False
            for match in regex.finditer(text):
                if any(start <= match.start() <= end for start, end in test_ranges):
                    continue
                failures.append(
                    f"{path.relative_to(REPO_ROOT)} contains forbidden frontend pattern {pattern!r}"
                )
                matched = True
                break
            if matched:
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
        "use sifr_syntax::{SourceText, parse_module};\n"
        "fn bypass() { let _ = parse_module(source, None); }\n",
        encoding="utf-8",
    )
    if not violations([seeded]):
        seeded.unlink()
        raise SystemExit(
            "split-brain guardrail self-test failed: seeded syntax import passed"
        )
    seeded.write_text(
        "use sifr_syntax as syntax;\n"
        "fn bypass() { let _ = syntax::parse_module(source, None); }\n",
        encoding="utf-8",
    )
    if not violations([seeded]):
        seeded.unlink()
        raise SystemExit(
            "split-brain guardrail self-test failed: seeded syntax crate alias passed"
        )
    seeded.write_text(
        "use sifr_syntax::*;\nfn bypass() { let _ = parse_module(source, None); }\n",
        encoding="utf-8",
    )
    if not violations([seeded]):
        seeded.unlink()
        raise SystemExit(
            "split-brain guardrail self-test failed: seeded syntax wildcard passed"
        )
    seeded.write_text(
        "use sifr_syntax::{parse_module as parse};\n"
        "fn bypass() { let _ = parse(source, None); }\n",
        encoding="utf-8",
    )
    if not violations([seeded]):
        seeded.unlink()
        raise SystemExit(
            "split-brain guardrail self-test failed: seeded syntax item alias passed"
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
    formatter_path = REPO_ROOT / "crates" / "sifr_format" / "src" / "lib.rs"
    if violations([cli_path, lint_path, formatter_path]):
        raise SystemExit(
            "split-brain guardrail self-test failed: classified syntax-only consumer rejected"
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
