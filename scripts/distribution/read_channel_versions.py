#!/usr/bin/env python3
"""Print alpha and beta versions from Sifr channel metadata."""

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
    alpha = channels.get("alpha")
    beta = channels.get("beta")
    if not isinstance(alpha, str) or not isinstance(beta, str):
        raise SystemExit("current channel metadata must contain alpha and beta")
    print(alpha, beta)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
