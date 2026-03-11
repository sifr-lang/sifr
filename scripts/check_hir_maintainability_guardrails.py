#!/usr/bin/env python3
"""Enforce Phase 20 anti-regrowth guardrails for HIR lowering and stdlib registry modules."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
from typing import Dict, List


MAX_LINES_BY_FILE: Dict[str, int] = {
    "crates/sifr_hir/src/lower/mod.rs": 1200,
    "crates/sifr_hir/src/lower/imports.rs": 300,
    "crates/sifr_hir/src/lower/diagnostics.rs": 600,
    "crates/sifr_hir/src/lower/classes.rs": 1400,
    "crates/sifr_hir/src/lower/typing_and_functions.rs": 1400,
    "crates/sifr_hir/src/lower/statements.rs": 2200,
    "crates/sifr_hir/src/lower/expressions.rs": 3800,
    "crates/sifr_hir/src/stdlib/mod.rs": 200,
    "crates/sifr_hir/src/stdlib/io_json.rs": 250,
    "crates/sifr_hir/src/stdlib/math_test.rs": 900,
    "crates/sifr_hir/src/stdlib/collections_bytes_time.rs": 600,
    "crates/sifr_hir/src/stdlib/sys_fs.rs": 700,
    "crates/sifr_hir/src/stdlib/crypto_regex_uuid.rs": 700,
    "crates/sifr_hir/src/stdlib/platform_misc.rs": 450,
}

BANNED_MONOLITHS = [
    "crates/sifr_hir/src/lower.rs",
    "crates/sifr_hir/src/stdlib.rs",
]

CHECKLIST_DOC = Path("internal_docs/hir_maintainability_guardrails.md")
REQUIRED_CHECKLIST_SNIPPETS = [
    "## Review Checklist",
    "- [ ] Lowering logic is placed in the correct file",
    "- [ ] Shared lowering helper extraction was considered before adding duplicate logic",
    "- [ ] File-size guardrails stay within limits",
    "- [ ] Guardrail script still passes locally (`python3 scripts/check_hir_maintainability_guardrails.py`)",
]


def count_lines(path: Path) -> int:
    with path.open("r", encoding="utf-8") as handle:
        return sum(1 for _ in handle)


def resolve_repo_root(script_path: Path) -> Path:
    return script_path.resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce Phase 20 HIR maintainability guardrails."
    )
    parser.add_argument(
        "--max-lines-override",
        type=int,
        default=None,
        help="Override all max line limits (testing/negative-path validation helper).",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = resolve_repo_root(Path(__file__))

    override = args.max_lines_override
    if override is None:
        env_override = os.getenv("SIFR_HIR_GUARD_MAX_OVERRIDE")
        if env_override is not None:
            override = int(env_override)

    failures: List[str] = []

    for rel in BANNED_MONOLITHS:
        path = repo_root / rel
        if path.exists():
            failures.append(f"banned monolith file still exists: {rel}")

    for rel, configured_limit in MAX_LINES_BY_FILE.items():
        limit = override if override is not None else configured_limit
        path = repo_root / rel
        if not path.exists():
            failures.append(f"required guardrail file missing: {rel}")
            continue
        lines = count_lines(path)
        if lines > limit:
            failures.append(
                f"{rel} is {lines} lines (limit {limit}); split the module instead of growing it"
            )

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
