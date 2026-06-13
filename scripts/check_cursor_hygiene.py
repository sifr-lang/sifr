#!/usr/bin/env python3
"""Enforce portable Cursor workflow assets."""

from __future__ import annotations

import argparse
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


FORBIDDEN_PATH_PARTS = {
    ".cursor/.rules",
    ".cursor/plans/.obsidian",
}
FORBIDDEN_FILE_NAMES = {
    ".DS_Store",
}
PERSONAL_PATH = "/Users/" + "yaseralnajjar" + "/"
FORBIDDEN_TEXT = (
    PERSONAL_PATH,
    "talk-to-claude-default",
    "talk-to-claude-gui-review",
    "@.cursor/.rules/",
)
REMOVED_CLAUDE_SKILLS = (
    ".cursor/skills/talk-to-claude-default/SKILL.md",
    ".cursor/skills/talk-to-claude-gui-review/SKILL.md",
)


@dataclass(frozen=True)
class Violation:
    path: str
    message: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def repo_root_from_script(script_path: Path) -> Path:
    return script_path.resolve().parent.parent


def tracked_cursor_files(repo_root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", ".cursor"],
        cwd=repo_root,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return [repo_root / line for line in result.stdout.splitlines() if line]


def rel(path: Path, repo_root: Path) -> str:
    return path.relative_to(repo_root).as_posix()


def collect_violations(repo_root: Path, files: Iterable[Path]) -> list[Violation]:
    violations: list[Violation] = []
    rel_files = {rel(path, repo_root) for path in files}

    for removed_skill in REMOVED_CLAUDE_SKILLS:
        if removed_skill in rel_files:
            violations.append(
                Violation(removed_skill, "removed Claude review skill variant is tracked")
            )

    for path in files:
        rel_path = rel(path, repo_root)
        if path.name in FORBIDDEN_FILE_NAMES:
            violations.append(Violation(rel_path, "tracked local editor artifact"))
        for forbidden_part in FORBIDDEN_PATH_PARTS:
            if rel_path == forbidden_part or rel_path.startswith(f"{forbidden_part}/"):
                violations.append(
                    Violation(rel_path, f"tracked forbidden Cursor path {forbidden_part}")
                )
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for forbidden in FORBIDDEN_TEXT:
            if forbidden in text:
                violations.append(
                    Violation(rel_path, f"forbidden Cursor workflow text: {forbidden}")
                )

    return violations


def format_violations(violations: list[Violation]) -> str:
    return "\n".join(
        f"- {violation.path}: {violation.message}" for violation in violations
    )


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-cursor-hygiene-") as temp_dir:
        repo_root = Path(temp_dir)
        cursor = repo_root / ".cursor"
        cursor.mkdir()
        good = cursor / "skills" / "talk-to-claude-opus" / "SKILL.md"
        bad = cursor / ".rules" / "architecture-overview.mdc"
        personal = cursor / "skills" / "phase-closure-loop" / "SKILL.md"
        good.parent.mkdir(parents=True)
        bad.parent.mkdir(parents=True)
        personal.parent.mkdir(parents=True, exist_ok=True)
        good.write_text("TALK_TO_CLAUDE_PROJECT\n", encoding="utf-8")
        bad.write_text("obsolete\n", encoding="utf-8")
        personal.write_text(PERSONAL_PATH + "work/tool\n", encoding="utf-8")

        violations = collect_violations(repo_root, [good, bad, personal])
        messages = {violation.message for violation in violations}
        if not any(".cursor/.rules" in message for message in messages):
            raise AssertionError("expected .cursor/.rules violation")
        if not any(PERSONAL_PATH in message for message in messages):
            raise AssertionError("expected personal path violation")


def main() -> None:
    args = parse_args()
    if args.self_test:
        run_self_test()
        print("Cursor hygiene self-test: PASS")
        return

    repo_root = repo_root_from_script(Path(__file__))
    violations = collect_violations(repo_root, tracked_cursor_files(repo_root))
    if violations:
        raise SystemExit(
            "Cursor hygiene guardrail failed:\n" + format_violations(violations)
        )
    print("Cursor hygiene guardrail: PASS")


if __name__ == "__main__":
    main()
