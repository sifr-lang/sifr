#!/usr/bin/env python3
"""Enforce Phase 20 anti-regrowth guardrails for HIR lowering and stdlib registry modules."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import List

BANNED_MONOLITHS = [
    "crates/sifr_hir/src/lower.rs",
    "crates/sifr_hir/src/stdlib.rs",
]

CHECKLIST_DOC = Path("internal_docs/hir_maintainability_guardrails.md")
REQUIRED_CHECKLIST_SNIPPETS = [
    "## Review Checklist",
    "- [ ] Lowering logic is placed in the correct file",
    "- [ ] Shared lowering helper extraction was considered before adding duplicate logic",
    "- [ ] Unified file-size guardrail passes locally (`python3 scripts/check_file_size_guardrails.py`)",
    "- [ ] Guardrail script still passes locally (`python3 scripts/check_hir_maintainability_guardrails.py`)",
]


def resolve_repo_root(script_path: Path) -> Path:
    return script_path.resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce Phase 20 HIR maintainability guardrails."
    )
    return parser.parse_args()


def main() -> int:
    parse_args()
    repo_root = resolve_repo_root(Path(__file__))

    failures: List[str] = []

    for rel in BANNED_MONOLITHS:
        path = repo_root / rel
        if path.exists():
            failures.append(f"banned monolith file still exists: {rel}")

    checklist_path = repo_root / CHECKLIST_DOC
    if not checklist_path.exists():
        failures.append(f"missing required checklist doc: {CHECKLIST_DOC}")
    else:
        checklist_text = checklist_path.read_text(encoding="utf-8")
        for snippet in REQUIRED_CHECKLIST_SNIPPETS:
            if snippet not in checklist_text:
                failures.append(
                    f"checklist doc is missing required item: {snippet}"
                )

    if failures:
        print("HIR maintainability guardrails: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print("HIR maintainability guardrails: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
