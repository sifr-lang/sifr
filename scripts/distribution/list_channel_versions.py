#!/usr/bin/env python3
"""List alpha and beta versions, one per line, from Sifr channel metadata."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

AREA_ROOT = Path(__file__).resolve().parents[2] / "verification" / "areas" / "distribution_release"
sys.path.insert(0, str(AREA_ROOT))

from governance.common import load_json_strict  # noqa: E402
from governance.release_index import validate_release_index  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("channels_json")
    args = parser.parse_args()

    metadata = validate_release_index(load_json_strict(Path(args.channels_json)))
    channels = metadata.get("channels", {})
    for channel in ("alpha", "beta"):
        version = channels.get(channel)
        if not isinstance(version, str):
            raise SystemExit(f"channels.json missing {channel}")
        print(version)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
