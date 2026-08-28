"""Write a deterministic source minimization candidate for a fuzz finding."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
OUTPUT_ROOT = REPO_ROOT / "target" / "verification" / "actual" / "fuzz_property" / "minimized"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, help="Hardening target id from mutation_smoke_manifest.json.")
    parser.add_argument("failing_source", help="Path to the failing generated .sifr source.")
    parser.add_argument("--output", help="Optional output path for the minimized candidate.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    source_path = Path(args.failing_source)
    if not source_path.is_file():
        raise SystemExit(f"failing source not found: {source_path}")
    source = source_path.read_text(encoding="utf-8")
    minimized = normalize_source(source)
    digest = hashlib.sha256(minimized.encode("utf-8")).hexdigest()[:16]
    output = Path(args.output) if args.output else OUTPUT_ROOT / args.target / f"{digest}.sifr"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(minimized, encoding="utf-8")
    print(output)
    return 0


def normalize_source(source: str) -> str:
    lines = [line.rstrip() for line in source.replace("\r\n", "\n").split("\n")]
    while lines and not lines[0].strip():
        lines.pop(0)
    while lines and not lines[-1].strip():
        lines.pop()
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    raise SystemExit(main())
