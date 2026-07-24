#!/usr/bin/env python3
"""Ensure Sifr diagnostic code references in docs link to error pages."""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DOCS_ROOT = REPO_ROOT / "docs"
FULL_CODE = re.compile(r"SIFR-[A-Z]+(?:-[A-Z]+)*-\d{4}")
LINKED = re.compile(
    r"\[`?(SIFR-[A-Z]+(?:-[A-Z]+)*-\d{4})`?\]\(/errors/[^)]+\)"
)

SKIP_PATH_PREFIXES = (
    "docs/errors/SIFR-",
    "docs/errors/diagnostic-codes.",
)
SKIP_PATHS = {
    "docs/docs.json",
}


def active_error_pages() -> set[str]:
    pages: set[str] = set()
    errors_dir = DOCS_ROOT / "errors"
    for path in errors_dir.glob("SIFR-*.md"):
        pages.add(path.stem)
    return pages


def mask_code_fences(text: str) -> str:
    lines = text.splitlines(keepends=True)
    masked: list[str] = []
    in_fence = False
    for line in lines:
        stripped = line.lstrip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            masked.append("\0" * len(line))
        elif in_fence:
            masked.append("\0" * len(line))
        else:
            masked.append(line)
    return "".join(masked)


def linked_code_spans(text: str) -> set[tuple[int, int]]:
    return {(match.start(1), match.end(1)) for match in LINKED.finditer(text)}


def is_table_link_row(line: str) -> bool:
    return "| [`" in line and "](/errors/" in line


def inside_inline_code(text: str, index: int) -> bool:
    tick_count = 0
    idx = 0
    while idx < index:
        if text.startswith("```", idx):
            idx += 3
            continue
        if text[idx] == "`":
            tick_count += 1
        idx += 1
    return tick_count % 2 == 1


def find_unlinked_references(text: str) -> list[tuple[int, str]]:
    masked = mask_code_fences(text)
    linked = linked_code_spans(text)
    issues: list[tuple[int, str]] = []

    for match in FULL_CODE.finditer(masked):
        if "\0" in match.group():
            continue
        start, end = match.start(), match.end()
        if inside_inline_code(text, start):
            continue
        if (start, end) in linked:
            continue

        before = text[:start]
        if before.rfind("](") > before.rfind(")"):
            continue

        line_no = text.count("\n", 0, start) + 1
        line = text.splitlines()[line_no - 1]
        if is_table_link_row(line):
            continue

        issues.append((line_no, match.group()))

    return issues


def iter_doc_files() -> list[Path]:
    files: list[Path] = []
    for path in sorted(DOCS_ROOT.rglob("*")):
        if path.suffix not in {".md", ".mdx"}:
            continue
        rel = path.relative_to(REPO_ROOT).as_posix()
        if rel in SKIP_PATHS:
            continue
        if any(rel.startswith(prefix) for prefix in SKIP_PATH_PREFIXES):
            continue
        files.append(path)
    return files


def main() -> int:
    failures: list[str] = []
    for path in iter_doc_files():
        rel = path.relative_to(REPO_ROOT).as_posix()
        text = path.read_text()
        for line_no, code in find_unlinked_references(text):
            failures.append(f"{rel}:{line_no}: unlinked diagnostic reference `{code}`")

    if failures:
        print("Docs error-code link guardrail failed:", file=sys.stderr)
        for failure in failures:
            print(failure, file=sys.stderr)
        print(
            "\nLink active codes as [`SIFR-FAMILY-0001`](/errors/SIFR-FAMILY-0001). "
            "Link retired or reserved codes to [`SIFR-FAMILY-0001`](/errors/diagnostic-codes). "
            "Do not link codes inside fenced code blocks.",
            file=sys.stderr,
        )
        return 1

    print("Docs error-code link guardrail passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
