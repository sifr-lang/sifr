#!/usr/bin/env python3
"""Verify that a GitHub release view JSON contains an installer asset."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("release_view_json")
    parser.add_argument("expected_asset")
    args = parser.parse_args()

    release = json.loads(Path(args.release_view_json).read_text(encoding="utf-8"))
    assets = release.get("assets", [])
    for asset in assets:
        if asset.get("name") == args.expected_asset:
            size = asset.get("size")
            if not isinstance(size, int) or size < 1024:
                raise SystemExit(f"{args.expected_asset} has invalid size {size!r}")
            return 0
    raise SystemExit(f"missing release asset {args.expected_asset}")


if __name__ == "__main__":
    raise SystemExit(main())
