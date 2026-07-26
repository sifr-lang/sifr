#!/usr/bin/env python3
"""Print alpha and beta versions from Sifr channel metadata."""

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
    alpha = channels.get("alpha")
    beta = channels.get("beta")
    if not isinstance(alpha, str) or not isinstance(beta, str):
        raise SystemExit("current channel metadata must contain alpha and beta")
    print(alpha, beta)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
