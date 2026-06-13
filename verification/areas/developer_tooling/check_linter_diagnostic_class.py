#!/usr/bin/env python3
"""Validate typed diagnostic-class gating for lint code actions."""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCAN_ROOTS = [
    REPO_ROOT / "crates" / "sifr_analysis" / "src",
    REPO_ROOT / "crates" / "sifr_lsp" / "src",
]
FORBIDDEN = [
    'starts_with("SIFR-LINT-")',
    "starts_with(\"SIFR-LINT-\")",
    "starts_with('SIFR-LINT-')",
]
REQUIRED = [
    "DiagnosticClass::Policy",
    '"diagnosticClass"',
    '"policy"',
]


def rust_files() -> list[Path]:
    files: list[Path] = []
    for root in SCAN_ROOTS:
        if root.exists():
            files.extend(sorted(root.rglob("*.rs")))
    return files


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8", errors="replace")
        for line_number, line in enumerate(text.splitlines(), 1):
            for pattern in FORBIDDEN:
                if pattern in line:
                    failures.append(
                        f"{path.relative_to(REPO_ROOT)}:{line_number} uses prefix diagnostic gating {pattern!r}"
                    )
        if path == REPO_ROOT / "crates" / "sifr_analysis" / "src" / "host" / "implementation.rs":
            if "DiagnosticClass::Policy" not in text:
                failures.append(f"{path.relative_to(REPO_ROOT)} does not gate actions on DiagnosticClass::Policy")
        if path == REPO_ROOT / "crates" / "sifr_lsp" / "src" / "conversion.rs":
            missing = [snippet for snippet in REQUIRED[1:] if snippet not in text]
            if missing:
                failures.append(f"{path.relative_to(REPO_ROOT)} missing diagnostic class payload snippets: {missing}")
    return failures


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        seed = Path(tmp) / "code_action.rs"
        seed.write_text(
            'fn gate(code: &str) -> bool { code.starts_with("SIFR-LINT-") }\n',
            encoding="utf-8",
        )
        found = violations([seed])
    if not found:
        raise SystemExit("linter diagnostic-class self-test failed: seeded prefix gate passed")
    print("linter diagnostic-class self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0
    found = violations(rust_files())
    if found:
        print("linter diagnostic-class: FAIL", file=sys.stderr)
        for failure in found:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("linter diagnostic-class: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
