"""Normalize a failing structured diagnostic JSON payload."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
OUTPUT_ROOT = REPO_ROOT / "target" / "verification" / "actual" / "fuzz_property" / "minimized"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, help="Hardening target id from fuzz_smoke_manifest.json.")
    parser.add_argument("failing_rendered_diagnostic_json", help="Path to the failing rendered diagnostic JSON.")
    parser.add_argument("--output", help="Optional output path for the minimized candidate.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    payload_path = Path(args.failing_rendered_diagnostic_json)
    if not payload_path.is_file():
        raise SystemExit(f"diagnostic JSON not found: {payload_path}")
    payload = json.loads(payload_path.read_text(encoding="utf-8"))
    normalized = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    digest = hashlib.sha256(normalized.encode("utf-8")).hexdigest()[:16]
    output = Path(args.output) if args.output else OUTPUT_ROOT / args.target / f"{digest}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(normalized, encoding="utf-8")
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
