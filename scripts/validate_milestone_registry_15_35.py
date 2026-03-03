#!/usr/bin/env python3
"""Validate milestone registry coverage for phases 15-35."""

from __future__ import annotations

import re
from pathlib import Path

PHASE_DIR = Path(".cursor/plans/main/phases")
REGISTRY_PATH = Path(".cursor/plans/main/milestone_registry_15_35.md")


def source_milestones() -> set[str]:
    ids: set[str] = set()
    pattern = re.compile(r"^(##|###)\s+(milestone_[^:\n]+):")
    for phase in range(15, 36):
        phase_file = next(PHASE_DIR.glob(f"{phase:02d}_*.md"))
        for line in phase_file.read_text().splitlines():
            m = pattern.match(line)
            if m:
                ids.add(m.group(2))
    return ids


def registry_milestones() -> set[str]:
    ids: set[str] = set()
    pattern = re.compile(r"^\|\s*`(milestone_[^`]+)`\s*\(")
    for line in REGISTRY_PATH.read_text().splitlines():
        m = pattern.match(line)
        if m:
            ids.add(m.group(1))
    return ids


def main() -> None:
    src = source_milestones()
    reg = registry_milestones()

    missing = sorted(src - reg)
    extra = sorted(reg - src)
    if missing or extra:
        if missing:
            print("missing_from_registry:")
            for item in missing:
                print(f"- {item}")
        if extra:
            print("extra_not_in_sources:")
            for item in extra:
                print(f"- {item}")
        raise SystemExit("milestone registry validation failed")

    print(f"milestone registry validation ok: milestones={len(src)}")


if __name__ == "__main__":
    main()
