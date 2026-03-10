#!/usr/bin/env python3
"""Enforce anti-regrowth guardrails for the sifr_driver decomposition."""

from __future__ import annotations

import argparse
import os
import re
from pathlib import Path
from typing import Iterable, List


CHECKLIST_DOC = Path("docs/sifr_driver_maintainability_guardrails.md")
TESTS_DIR = Path("crates/sifr_driver/src/tests")
DRIVER_SRC = Path("crates/sifr_driver/src")

LIB_RS_MAX_LINES = 250
MOD_RS_MAX_LINES = 250
IMPL_RS_MAX_LINES = 900
TEST_RS_MAX_LINES = 700

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
    "- [ ] File-size guardrails stay within limits",
    "- [ ] Guardrail script still passes locally (`python3 scripts/check_sifr_driver_maintainability_guardrails.py`)",
]


def resolve_repo_root(script_path: Path) -> Path:
    return script_path.resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce sifr_driver maintainability guardrails."
    )
    parser.add_argument(
        "--max-lines-override",
        type=int,
        default=None,
        help="Override all max line limits (testing/negative-path validation helper).",
    )
    return parser.parse_args()


def count_lines(path: Path) -> int:
    with path.open("r", encoding="utf-8") as handle:
        return sum(1 for _ in handle)


def implementation_line_limit(rel: Path, limit_override: int | None) -> int:
    if limit_override is not None:
        return limit_override
    if rel == DRIVER_SRC / "lib.rs":
        return LIB_RS_MAX_LINES
    if rel.parts[-1] == "mod.rs":
        return MOD_RS_MAX_LINES
    if rel.is_relative_to(TESTS_DIR):
        return TEST_RS_MAX_LINES
    return IMPL_RS_MAX_LINES


def check_line_limits(repo_root: Path, limit_override: int | None) -> List[str]:
    failures: List[str] = []
    for path in sorted((repo_root / DRIVER_SRC).rglob("*.rs")):
        rel = path.relative_to(repo_root)
        lines = count_lines(path)
        limit = implementation_line_limit(rel, limit_override)
        if lines > limit:
            failures.append(
                f"{rel} is {lines} lines (limit {limit}); split the module instead of growing it"
            )
    return failures


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


def collect_failures(repo_root: Path, limit_override: int | None) -> List[str]:
    failures: List[str] = []
    failures.extend(check_banned_monoliths(repo_root))
    failures.extend(check_line_limits(repo_root, limit_override))
    failures.extend(check_lib_rs_shape(repo_root))
    failures.extend(check_checklist_doc(repo_root))
    return failures


def emit_failures(label: str, failures: Iterable[str]) -> None:
    print(label)
    for failure in failures:
        print(f"- {failure}")


def main() -> int:
    args = parse_args()
    repo_root = resolve_repo_root(Path(__file__))

    override = args.max_lines_override
    if override is None:
        env_override = os.getenv("SIFR_DRIVER_GUARD_MAX_OVERRIDE")
        if env_override is not None:
            override = int(env_override)

    expect_failure = os.getenv("SIFR_DRIVER_GUARDRAIL_EXPECT_FAILURE") == "1"
    if expect_failure and override is None:
        override = 10

    failures = collect_failures(repo_root, override)

    if expect_failure:
        if failures:
            emit_failures(
                "sifr_driver maintainability guardrails: PASS (expected failure mode)",
                failures,
            )
            return 0
        print(
            "sifr_driver maintainability guardrails: FAIL (expected the override to trigger a failure)"
        )
        return 1

    if failures:
        emit_failures("sifr_driver maintainability guardrails: FAIL", failures)
        return 1

    print("sifr_driver maintainability guardrails: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
