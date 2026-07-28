#!/usr/bin/env python3
"""Allocate the next governed release-index generation."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError,
)
from verification.areas.distribution_release.governance.generation import (  # noqa: E402
    allocate_next_generation,
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--live-index", type=Path, required=True)
    parser.add_argument("--snapshot-root", type=Path, required=True)
    args = parser.parse_args()
    try:
        generation = allocate_next_generation(
            live_index_path=args.live_index,
            snapshot_root=args.snapshot_root,
        )
    except GovernanceError as exc:
        parser.error(str(exc))
    print(generation)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
