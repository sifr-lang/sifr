#!/usr/bin/env python3
"""Validate entry/exit phase gate status from plans/roadmap.md."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

STATUS_SET = {"planned", "completed", "draft", "superseded"}


def parse_phase_statuses(roadmap_path: Path) -> dict[int, str]:
    statuses: dict[int, str] = {}
    for raw_line in roadmap_path.read_text().splitlines():
        line = raw_line.strip()
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) < 2:
            continue
        if not re.fullmatch(r"\d+", cells[0]):
            continue
        phase = int(cells[0])
        status_cell = cells[1].lower()
        if status_cell in STATUS_SET:
            statuses[phase] = status_cell
            continue
        if len(cells) >= 3 and cells[2].lower() in STATUS_SET:
            statuses[phase] = cells[2].lower()
    return statuses


def require_phase_file_exists(phase: int) -> None:
    matches = list(Path("plans/phases").glob(f"{phase:02d}_*.md"))
    if not matches:
        raise SystemExit(
            f"gate-check failed: no phase file found for phase {phase} in plans/phases"
        )


def require_completed(statuses: dict[int, str], phase: int) -> None:
    status = statuses.get(phase)
    if status != "completed":
        raise SystemExit(
            f"gate-check failed: phase {phase} status is '{status or 'missing'}', expected 'completed'"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--phase", type=int, required=True, help="Phase number to validate.")
    parser.add_argument(
        "--check",
        choices=("entry", "exit"),
        required=True,
        help="Gate check mode.",
    )
    parser.add_argument(
        "--roadmap",
        default="plans/roadmap.md",
        help="Path to roadmap file.",
    )
    args = parser.parse_args()

    require_phase_file_exists(args.phase)
    statuses = parse_phase_statuses(Path(args.roadmap))
    if args.check == "entry":
        prior_phase = 14 if args.phase == 15 else args.phase - 1
        require_completed(statuses, prior_phase)
        print(
            f"entry gate ok: phase {args.phase} prior dependency phase {prior_phase} is completed"
        )
        return

    require_completed(statuses, args.phase)
    print(f"exit gate ok: phase {args.phase} is completed")


if __name__ == "__main__":
    main()
