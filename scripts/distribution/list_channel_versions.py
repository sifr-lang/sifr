#!/usr/bin/env python3
"""List alpha and beta versions, one per line, from Sifr channel metadata."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("channels_json")
    args = parser.parse_args()

    metadata = json.loads(Path(args.channels_json).read_text(encoding="utf-8"))
    channels = metadata.get("channels", {})
    for channel in ("alpha", "beta"):
        version = channels.get(channel)
        if not isinstance(version, str):
            raise SystemExit(f"channels.json missing {channel}")
        print(version)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
