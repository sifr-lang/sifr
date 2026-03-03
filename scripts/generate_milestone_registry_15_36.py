#!/usr/bin/env python3
"""Generate milestone coverage registry for phases 15-36."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

PHASE_DIR = Path(".cursor/plans/main/phases")
OUTPUT_PATH = Path(".cursor/plans/main/milestone_registry_15_36.md")


@dataclass
class Milestone:
    milestone_id: str
    title: str
    scope: str
    dod: str


def clean_inline(text: str) -> str:
    text = text.strip()
    text = re.sub(r"`", "", text)
    return re.sub(r"\s+", " ", text)


def parse_bullets(lines: list[str], start_idx: int) -> tuple[list[str], int]:
    bullets: list[str] = []
    idx = start_idx
    while idx < len(lines):
        line = lines[idx]
        stripped = line.strip()
        if stripped.startswith("- ") and not line.startswith("  "):
            break
        if stripped.startswith("##"):
            break
        if stripped.startswith("- ") or stripped.startswith("* "):
            bullets.append(clean_inline(stripped[2:]))
        elif line.startswith("  - ") or line.startswith("    - "):
            bullets.append(clean_inline(stripped[2:]))
        idx += 1
    return bullets, idx


def extract_scope(block_lines: list[str]) -> str:
    for idx, line in enumerate(block_lines):
        if line.strip() == "- Scope:":
            bullets, _ = parse_bullets(block_lines, idx + 1)
            if bullets:
                return "; ".join(bullets)
    for line in block_lines:
        if line.strip().startswith("**Goal:**"):
            return clean_inline(line.split("**Goal:**", 1)[1])
    return "See source phase file section."


def extract_dod(block_lines: list[str]) -> str:
    for idx, line in enumerate(block_lines):
        if line.strip().lower() == "- definition of done:":
            bullets, _ = parse_bullets(block_lines, idx + 1)
            if bullets:
                return "; ".join(bullets)

    for idx, line in enumerate(block_lines):
        if line.strip().lower().startswith("### definition of done"):
            bullets: list[str] = []
            j = idx + 1
            while j < len(block_lines):
                stripped = block_lines[j].strip()
                if stripped.startswith("##"):
                    break
                if stripped.startswith("- "):
                    bullets.append(clean_inline(stripped[2:]))
                j += 1
            if bullets:
                return "; ".join(bullets)

    return "See source phase file section."


def parse_phase_file(path: Path) -> tuple[str, list[Milestone]]:
    text = path.read_text()
    lines = text.splitlines()
    phase_title_match = re.search(r"^#\s+(.+)$", text, flags=re.M)
    phase_title = phase_title_match.group(1) if phase_title_match else path.stem

    header_pattern = re.compile(r"^(##|###)\s+(milestone_[^:\n]+):\s*(.+)$")
    headers: list[tuple[int, str, str]] = []
    for idx, line in enumerate(lines):
        m = header_pattern.match(line)
        if m:
            headers.append((idx, m.group(2).strip(), m.group(3).strip()))

    milestones: list[Milestone] = []
    for i, (start_idx, milestone_id, title) in enumerate(headers):
        end_idx = headers[i + 1][0] if i + 1 < len(headers) else len(lines)
        block = lines[start_idx:end_idx]
        scope = extract_scope(block)
        dod = extract_dod(block)
        milestones.append(Milestone(milestone_id, title, scope, dod))

    return phase_title, milestones


def main() -> None:
    out: list[str] = []
    out.append("# Milestone Registry (Phases 15-36)")
    out.append("")
    out.append("Last updated: 2026-03-03")
    out.append("Owner: Phase 15 (`milestone_15_2` detail closure)")
    out.append("Status: active")
    out.append("")
    out.append("## Purpose")
    out.append(
        "This registry guarantees that every milestone across phases 15-36 is explicitly tracked with scope and definition-of-done snapshots."
    )
    out.append("")
    out.append("Mandatory local validation contract per milestone:")
    out.append(
        "- `python scripts/phase_contract_gate_check.py --phase <N> --check entry`"
    )
    out.append("- `python scripts/validate_milestone_registry_15_36.py`")
    out.append("- `./scripts/run_all_tests.sh`")
    out.append("")

    total = 0
    for phase in range(15, 37):
        phase_file = next(PHASE_DIR.glob(f"{phase:02d}_*.md"))
        phase_title, milestones = parse_phase_file(phase_file)
        total += len(milestones)

        out.append(f"## Phase {phase}: {phase_title.replace(f'Phase {phase}: ', '')}")
        out.append(f"Source: `{phase_file}`")
        out.append("")
        out.append("| Milestone | Scope Snapshot | Definition-of-Done Snapshot |")
        out.append("|---|---|---|")
        for m in milestones:
            out.append(
                f"| `{m.milestone_id}` ({m.title}) | {m.scope} | {m.dod} |"
            )
        out.append("")

    out.append("## Coverage Summary")
    out.append(f"- Total milestones tracked: `{total}`")
    out.append("- Validation command: `python scripts/validate_milestone_registry_15_36.py`")
    out.append("")

    OUTPUT_PATH.write_text("\n".join(out) + "\n")
    print(f"wrote {OUTPUT_PATH} with {total} milestones")


if __name__ == "__main__":
    main()
