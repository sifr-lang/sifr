#!/usr/bin/env python3
"""Check diagnostic fixture and baseline files for legacy expectation forms."""

from __future__ import annotations

import pathlib
import re
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[4]
EXPECT_ERROR_RE = re.compile(r"^\s*#\s*expect-error(?:\[col=\d+\])?:\s*SIFR-[A-Z]+-\d{4}\s*$")
LEGACY_PSEUDO_CODE_RE = re.compile(r"\[E\d{4}\]")
CANONICAL_CATCH_ALL = "SIFR-TYPE-" + "0001"
CANONICAL_CATCH_ALL_RE = re.compile(rf"\b{CANONICAL_CATCH_ALL}\b")


def git_ls_files(*patterns: str) -> list[pathlib.Path]:
    result = subprocess.run(
        ["git", "ls-files", *patterns],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return [ROOT / line for line in result.stdout.splitlines() if line]


def main() -> int:
    errors: list[str] = []

    for path in git_ls_files("crates/sifr/tests/e2e/fail/*.sifr"):
        rel = path.relative_to(ROOT)
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if "expect-error" not in line:
                continue
            if not EXPECT_ERROR_RE.fullmatch(line):
                errors.append(
                    f"{rel}:{line_number}: expect-error must assert one canonical SIFR code"
                )

    hygiene_paths = git_ls_files(
        "crates/sifr/tests/e2e/**/*.sifr",
        "crates/sifr/tests/verification/**",
        "verification/**",
        "docs/errors/**",
        "docs/schemas/**",
    )
    for path in hygiene_paths:
        if not path.is_file():
            continue
        rel = path.relative_to(ROOT)
        text = path.read_text(encoding="utf-8", errors="ignore")
        if LEGACY_PSEUDO_CODE_RE.search(text):
            errors.append(f"{rel}: contains legacy [Edddd] pseudo-code text")
        if CANONICAL_CATCH_ALL_RE.search(text):
            errors.append(f"{rel}: contains forbidden {CANONICAL_CATCH_ALL} catch-all code")

    if errors:
        for error in errors:
            print(f"diagnostic baseline hygiene: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
