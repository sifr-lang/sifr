#!/usr/bin/env python3

from __future__ import annotations

import json
from pathlib import Path

from phase31_leetcode_scorecard import (
    ScorecardPaths,
    build_scorecard,
    render_scorecard_markdown,
)


ROOT = Path(__file__).resolve().parent.parent
SCORECARD_JSON = ROOT / "verification/leetcode/phase31_scorecard.json"
SCORECARD_MD = ROOT / "verification/leetcode/phase31_scorecard.md"


def main() -> None:
    scorecard = build_scorecard(
        ScorecardPaths(
            baseline_results=ROOT / "verification/leetcode/phase31_seed_results.json",
            wave_results=ROOT / "verification/leetcode/phase31_seed_results_wave1.json",
            taxonomy=ROOT / "verification/leetcode/phase31_failure_taxonomy.json",
            backlog=ROOT / "verification/leetcode/phase31_remediation_backlog.json",
        )
    )
    markdown = render_scorecard_markdown(scorecard)

    SCORECARD_JSON.write_text(json.dumps(scorecard, indent=2, sort_keys=True) + "\n")
    SCORECARD_MD.write_text(markdown)
    print(f"wrote {SCORECARD_JSON.relative_to(ROOT)}")
    print(f"wrote {SCORECARD_MD.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
