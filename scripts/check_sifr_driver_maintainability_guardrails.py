#!/usr/bin/env python3
"""Enforce anti-regrowth guardrails for the sifr_driver decomposition."""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from typing import Iterable, List


CHECKLIST_DOC = Path("internal_docs/sifr_driver_maintainability_guardrails.md")
DRIVER_SRC = Path("crates/sifr_driver/src")

BANNED_MONOLITHS = [
    DRIVER_SRC / "stdlib.rs",
    DRIVER_SRC / "frontend.rs",
    DRIVER_SRC / "project.rs",
    DRIVER_SRC / "build.rs",
    DRIVER_SRC / "test_runner.rs",
]

REQUIRED_CHECKLIST_SNIPPETS = [
    "## Review Checklist",
    "- [ ] New driver logic is placed in the correct module subtree",
    "- [ ] `crates/sifr_driver/src/lib.rs` stays crate wiring plus re-exports only",
    "- [ ] Test coverage lives in focused `crates/sifr_driver/src/tests/` modules or beside the extracted concern",
    "- [ ] Unified file-size guardrail passes locally (`python3 scripts/check_file_size_guardrails.py`)",
    "- [ ] Guardrail script still passes locally (`python3 scripts/check_sifr_driver_maintainability_guardrails.py`)",
]


def resolve_repo_root(script_path: Path) -> Path:
    return script_path.resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce sifr_driver maintainability guardrails."
    )
    return parser.parse_args()


def check_banned_monoliths(repo_root: Path) -> List[str]:
    failures: List[str] = []
    for rel in BANNED_MONOLITHS:
        if (repo_root / rel).exists():
            failures.append(f"banned monolith file still exists: {rel}")
    return failures


def check_lib_rs_shape(repo_root: Path) -> List[str]:
    lib_rs = repo_root / DRIVER_SRC / "lib.rs"
    text = lib_rs.read_text(encoding="utf-8")
    failures: List[str] = []
    if re.search(r"^\s*#\[test\]", text, flags=re.MULTILINE):
        failures.append("crates/sifr_driver/src/lib.rs must not contain inline tests")
    if re.search(
        r"^\s*(pub(?:\(crate\))?\s+)?(fn|struct|enum|impl|type)\b",
        text,
        flags=re.MULTILINE,
    ):
        failures.append(
            "crates/sifr_driver/src/lib.rs must stay crate wiring plus re-exports only"
        )
    return failures


def check_checklist_doc(repo_root: Path) -> List[str]:
    checklist_path = repo_root / CHECKLIST_DOC
    if not checklist_path.exists():
        return [f"missing required checklist doc: {CHECKLIST_DOC}"]

    failures: List[str] = []
    text = checklist_path.read_text(encoding="utf-8")
    for snippet in REQUIRED_CHECKLIST_SNIPPETS:
        if snippet not in text:
            failures.append(f"checklist doc is missing required item: {snippet}")
    return failures


def collect_failures(repo_root: Path) -> List[str]:
    failures: List[str] = []
    failures.extend(check_banned_monoliths(repo_root))
    failures.extend(check_lib_rs_shape(repo_root))
    failures.extend(check_checklist_doc(repo_root))
    return failures


def emit_failures(label: str, failures: Iterable[str]) -> None:
    print(label)
    for failure in failures:
        print(f"- {failure}")


def main() -> int:
    parse_args()
    repo_root = resolve_repo_root(Path(__file__))
    failures = collect_failures(repo_root)

    if failures:
        emit_failures("sifr_driver maintainability guardrails: FAIL", failures)
        return 1

    print("sifr_driver maintainability guardrails: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
