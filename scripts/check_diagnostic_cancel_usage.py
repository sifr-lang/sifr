#!/usr/bin/env python3
"""Restrict diagnostic cancellation to the diagnostic model implementation."""

from __future__ import annotations

import pathlib
import re
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
MODEL_FILE = pathlib.Path("crates/sifr_diagnostics/src/model/mod.rs")
CANCEL_RE = re.compile(r"(?:\.\s*)?cancel\s*\(")


def git_ls_files(*patterns: str) -> list[pathlib.Path]:
    result = subprocess.run(
        ["git", "ls-files", *patterns],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return [ROOT / line for line in result.stdout.splitlines() if line]


def strip_cfg_test_blocks(text: str) -> str:
    kept: list[str] = []
    pending_cfg_test = False
    skipping = False
    depth = 0

    for line in text.splitlines():
        stripped = line.strip()
        if skipping:
            depth += line.count("{") - line.count("}")
            if depth <= 0:
                skipping = False
            continue

        if stripped == "#[cfg(test)]":
            pending_cfg_test = True
            continue

        if pending_cfg_test:
            pending_cfg_test = False
            if "{" in line:
                skipping = True
                depth = line.count("{") - line.count("}")
                if depth <= 0:
                    skipping = False
                continue
            continue

        kept.append(line)

    return "\n".join(kept)


def main() -> int:
    errors: list[str] = []
    for path in git_ls_files("crates/**/*.rs"):
        if not path.exists():
            continue
        rel = path.relative_to(ROOT)
        text = strip_cfg_test_blocks(path.read_text(encoding="utf-8", errors="ignore"))
        for line_number, line in enumerate(text.splitlines(), 1):
            if not CANCEL_RE.search(line):
                continue
            if rel == MODEL_FILE:
                continue
            errors.append(f"{rel}:{line_number}: diagnostic cancel usage is not allowed")

    if errors:
        for error in errors:
            print(f"diagnostic cancel usage: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
