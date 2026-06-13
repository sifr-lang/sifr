#!/usr/bin/env python3
"""Validate Phase 15 canonical backlog integrity constraints."""

from __future__ import annotations

import re
from pathlib import Path

BACKLOG_PATH = Path("plans/phases/15_baseline_reconciliation.md")
ISSUES_PATH = Path("plans/issues/archive/phase15-canonical-backlog-issues.md")
ROADMAP_PATH = Path("plans/roadmap.md")

ALLOWED_SEVERITIES = {"P0", "P1", "P2", "P3"}


def parse_roadmap_phase_ids(path: Path) -> set[int]:
    phase_ids: set[int] = set()
    for raw_line in path.read_text().splitlines():
        line = raw_line.strip()
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if not cells:
            continue
        if re.fullmatch(r"\d+", cells[0]):
            phase_ids.add(int(cells[0]))
    return phase_ids


def parse_issue_headings(path: Path) -> set[str]:
    headings: set[str] = set()
    for raw_line in path.read_text().splitlines():
        if raw_line.startswith("## "):
            headings.add(raw_line[3:].strip())
    return headings


def parse_backlog_rows(path: Path) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for raw_line in path.read_text().splitlines():
        if not raw_line.startswith("| `BL-15-"):
            continue
        cells = [cell.strip() for cell in raw_line.strip("|").split("|")]
        if len(cells) != 7:
            raise SystemExit(
                f"invalid backlog row format (expected 7 columns): {raw_line}"
            )
        match_id = re.fullmatch(r"`(BL-15-\d{3})`", cells[0])
        match_severity = re.fullmatch(r"`(P[0-3])`", cells[2])
        match_phase = re.fullmatch(r"Phase (\d+)", cells[3])
        if not (match_id and match_severity and match_phase):
            raise SystemExit(f"invalid backlog row values: {raw_line}")
        rows.append(
            {
                "id": match_id.group(1),
                "severity": match_severity.group(1),
                "owning_phase": match_phase.group(1),
                "issue_link_cell": cells[5],
            }
        )
    return rows


def main() -> None:
    phase_ids = parse_roadmap_phase_ids(ROADMAP_PATH)
    issue_headings = parse_issue_headings(ISSUES_PATH)
    rows = parse_backlog_rows(BACKLOG_PATH)

    if not rows:
        raise SystemExit("no canonical backlog rows found")

    ids = [row["id"] for row in rows]
    if len(ids) != len(set(ids)):
        raise SystemExit("duplicate BL-15 IDs found in canonical backlog table")

    for row in rows:
        if row["severity"] not in ALLOWED_SEVERITIES:
            raise SystemExit(
                f"invalid severity '{row['severity']}' for backlog item {row['id']}"
            )
        phase_id = int(row["owning_phase"])
        if phase_id not in phase_ids:
            raise SystemExit(
                f"owning phase {phase_id} for {row['id']} not found in roadmap"
            )

        expected_anchor = f"phase15-{row['id'].lower()}"
        if f"#{expected_anchor}" not in row["issue_link_cell"]:
            raise SystemExit(
                f"backlog issue link for {row['id']} does not reference #{expected_anchor}"
            )

        expected_heading = f"phase15-{row['id']}"
        if expected_heading not in issue_headings:
            raise SystemExit(
                f"backlog issue heading '{expected_heading}' missing from {ISSUES_PATH}"
            )

    print(
        f"phase15 backlog validation ok: rows={len(rows)} unique_ids={len(set(ids))} severities=P0-P3"
    )


if __name__ == "__main__":
    main()
