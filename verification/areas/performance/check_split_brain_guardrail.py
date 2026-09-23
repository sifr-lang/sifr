#!/usr/bin/env python3
"""Reject new parser/lowering/type-check entrypoints outside approved crates."""

from __future__ import annotations

from collections import Counter
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

# These exact production calls inspect syntax only. New calls in the same file
# require review; a file-wide exemption would hide a second semantic parser.
SYNTAX_ONLY_SITES = {
    Path("crates/sifr/src/check_and_package_commands.rs"): {
        "let Ok(parsed) = sifr_syntax::parse_module(source.as_str(), Some(&context)) else {",
        "if let Ok(parsed) = sifr_syntax::parse_module(source.as_str(), Some(&context)) {",
    },
    Path("crates/sifr_lint/src/engine.rs"): {
        "sifr_syntax::parse_module(source, parse_context.as_deref()).ok()",
    },
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
SYNTAX_CRATE_ALIAS_PATTERN = re.compile(
    r"\buse\s+sifr_syntax(?:\s+as\s+(\w+)\s*;|::\{\s*self\s+as\s+(\w+)[^}]*\}\s*;)"
)


def is_approved(path: Path) -> bool:
    rel = path.relative_to(REPO_ROOT)
    if "tests" in rel.parts or rel.name.endswith("_tests.rs") or "_tests_" in rel.name:
        return True
    return any(rel.is_relative_to(prefix) for prefix in APPROVED_PREFIXES)


def is_classified_syntax_only(path: Path, pattern: str, text: str, offset: int, remaining: Counter[str]) -> bool:
    if pattern != "sifr_syntax::parse_module(":
        return False
    rel = path.relative_to(REPO_ROOT)
    start = text.rfind("\n", 0, offset) + 1
    end = text.find("\n", offset)
    line = text[start:end if end >= 0 else len(text)].strip()
    if line not in SYNTAX_ONLY_SITES.get(rel, set()) or remaining[line] <= 0:
        return False
    remaining[line] -= 1
    return True


def violations_text(path: Path, text: str) -> list[str]:
    failures: list[str] = []
    if is_approved(path):
        return failures
    remaining = Counter(SYNTAX_ONLY_SITES.get(path.relative_to(REPO_ROOT), set()))
    test_ranges = inline_test_module_ranges(text)
    rules = [
        (pattern, re.compile(re.escape(pattern))) for pattern in FORBIDDEN_PATTERNS
    ]
    rules.append((SYNTAX_PARSE_IMPORT_LABEL, SYNTAX_PARSE_IMPORT_PATTERN))
    rules.append((SYNTAX_WILDCARD_IMPORT_LABEL, SYNTAX_WILDCARD_IMPORT_PATTERN))
    for plain_alias, braced_alias in SYNTAX_CRATE_ALIAS_PATTERN.findall(text):
        alias = plain_alias or braced_alias
        rules.append(
            (
                f"sifr_syntax alias-qualified parse_module ({alias})",
                re.compile(rf"\b{re.escape(alias)}::parse_module\s*\("),
            )
        )
    for pattern, regex in rules:
        matched = False
        for match in regex.finditer(text):
            if any(start <= match.start() <= end for start, end in test_ranges):
                continue
            if is_classified_syntax_only(path, pattern, text, match.start(), remaining):
                continue
            failures.append(
                f"{path.relative_to(REPO_ROOT)} contains forbidden frontend pattern {pattern!r}"
            )
            matched = True
            break
        if matched:
            break
    return failures


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        failures.extend(violations_text(path, path.read_text(encoding="utf-8", errors="replace")))
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
    lint_path = REPO_ROOT / "crates" / "sifr_lint" / "src" / "engine.rs"
    formatter_path = REPO_ROOT / "crates" / "sifr_format" / "src" / "lib.rs"
    if violations([cli_path, lint_path, formatter_path]):
        raise SystemExit("split-brain guardrail self-test failed: syntax-only consumer rejected")
    original = cli_path.read_text(encoding="utf-8")
    added = original + "\nfn duplicate_semantics() { let _ = sifr_syntax::parse_module(source, None); }\n"
    if not violations_text(cli_path, added):
        raise SystemExit("split-brain guardrail self-test failed: same-file parser escaped")
    seeded.write_text(
        "#[cfg(test)]\nmod tests {\n  use sifr_syntax::parse_module as parse;\n"
        "  fn fixture() { let _ = parse(source, None); }\n}\n"
        "use sifr_syntax::{parse_module as semantic_parse};\n"
        "fn production() { let _ = semantic_parse(source, None); }\n",
        encoding="utf-8",
    )
    if not violations([seeded]):
        seeded.unlink()
        raise SystemExit("split-brain guardrail self-test failed: mixed-source alias escaped")
    seeded.write_text(
        "#[cfg(test)]\nmod tests { fn fixture() { let _ = sifr_syntax::parse_module(source, None); } }\n"
        "use sifr_syntax::{self as syntax};\n"
        "fn production() { let _ = syntax::parse_module(source, None); }\n",
        encoding="utf-8",
    )
    if not violations([seeded]):
        seeded.unlink()
        raise SystemExit("split-brain guardrail self-test failed: braced crate alias escaped")
    seeded.unlink()
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
