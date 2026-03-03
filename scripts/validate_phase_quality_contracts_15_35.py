#!/usr/bin/env python3
"""Validate embedded phase quality contracts for phases 15-35."""

from __future__ import annotations

import re
from pathlib import Path

PHASE_DIR = Path(".cursor/plans/main/phases")


def has_milestones(text: str) -> bool:
    return bool(re.search(r"^(##|###)\s+milestone_[^:\n]+:", text, re.M))


def milestone_blocks(text: str) -> list[str]:
    matches = list(re.finditer(r"^(##|###)\s+(milestone_[^:\n]+):", text, re.M))
    blocks: list[str] = []
    for i, match in enumerate(matches):
        start = match.start()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        blocks.append(text[start:end])
    return blocks


def milestone_has_scope_and_dod(block: str) -> bool:
    has_scope = "- Scope:" in block or "**Goal:**" in block
    has_dod = "- Definition of done:" in block or "### Definition of Done" in block
    return has_scope and has_dod


def validate_phase_file(phase: int, path: Path) -> list[str]:
    text = path.read_text()
    errs: list[str] = []

    if "## Quality Contract" not in text:
        errs.append(f"phase {phase}: missing '## Quality Contract' section")
        return errs

    contract = text.split("## Quality Contract", 1)[1]
    if "## Exit Gate" in contract:
        contract = contract.split("## Exit Gate", 1)[0]

    required_lines = [
        "- Entry criteria:",
        "- Exit criteria:",
        "- Milestone quality checks:",
        "- Mandatory local validation commands:",
        f"`python scripts/phase_contract_gate_check.py --phase {phase} --check entry`",
        f"`python scripts/phase_contract_gate_check.py --phase {phase} --check exit`",
        "`python scripts/validate_phase_quality_contracts_15_35.py`",
        "`./scripts/run_all_tests.sh`",
    ]
    for line in required_lines:
        if line not in contract:
            errs.append(f"phase {phase}: missing quality-contract line: {line}")

    if not has_milestones(text):
        errs.append(f"phase {phase}: no milestone sections found")
        return errs

    for block in milestone_blocks(text):
        header = re.search(r"^(##|###)\s+(milestone_[^:\n]+):", block, re.M)
        milestone_id = header.group(2) if header else "unknown"
        if not milestone_has_scope_and_dod(block):
            errs.append(
                f"phase {phase}: milestone {milestone_id} missing scope or definition-of-done markers"
            )

    return errs


def main() -> None:
    all_errs: list[str] = []
    for phase in range(15, 36):
        phase_file = next(PHASE_DIR.glob(f"{phase:02d}_*.md"))
        all_errs.extend(validate_phase_file(phase, phase_file))

    if all_errs:
        print("phase quality contract validation failed:")
        for err in all_errs:
            print(f"- {err}")
        raise SystemExit(1)

    print("phase quality contract validation ok: phases=21 milestones=embedded")


if __name__ == "__main__":
    main()
