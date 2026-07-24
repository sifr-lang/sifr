#!/usr/bin/env python3
"""Link bare Sifr diagnostic code references in hand-maintained docs."""

from __future__ import annotations

import re
from pathlib import Path

from check_docs_error_code_links import (
    DOCS_ROOT,
    FULL_CODE,
    LINKED,
    SKIP_PATH_PREFIXES,
    active_error_pages,
    mask_code_fences,
)

REPO_ROOT = Path(__file__).resolve().parents[1]


def link_markdown(code: str, active_pages: set[str]) -> str:
    href = (
        f"/errors/{code}"
        if code in active_pages
        else "/errors/diagnostic-codes"
    )
    return f"[`{code}`]({href})"


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


def linkify_text(text: str, active_pages: set[str]) -> str:
    masked = mask_code_fences(text)
    replacements: list[tuple[int, int, str]] = []

    for match in FULL_CODE.finditer(masked):
        if "\0" in match.group():
            continue

        start, end = match.start(), match.end()
        if inside_inline_code(text, start):
            continue
        code = match.group()
        window_start = max(0, start - 2)
        window = text[window_start:end + 30]
        if re.search(r"\[`" + re.escape(code) + r"`\]\(/errors/", window):
            continue

        before = text[:start]
        if before.rfind("](") > before.rfind(")"):
            continue

        if text[start - 1 : start] == "`" and text[end : end + 1] == "`":
            replacements.append((start - 1, end + 1, link_markdown(code, active_pages)))
        else:
            replacements.append((start, end, link_markdown(code, active_pages)))

    if not replacements:
        return text

    updated = text
    for start, end, replacement in sorted(replacements, key=lambda item: item[0], reverse=True):
        updated = updated[:start] + replacement + updated[end:]
    return updated


def main() -> None:
    active_pages = active_error_pages()
    changed_files = 0

    for path in sorted(DOCS_ROOT.rglob("*")):
        if path.suffix not in {".md", ".mdx"}:
            continue
        rel = path.relative_to(REPO_ROOT).as_posix()
        if any(rel.startswith(prefix) for prefix in SKIP_PATH_PREFIXES):
            continue

        original = path.read_text()
        updated = linkify_text(original, active_pages)
        if updated != original:
            path.write_text(updated)
            changed_files += 1
            print(f"updated {rel}")

    print(f"linked error references in {changed_files} file(s)")


if __name__ == "__main__":
    main()
