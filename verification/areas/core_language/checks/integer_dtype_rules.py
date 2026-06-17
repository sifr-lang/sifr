#!/usr/bin/env python3
"""Validate the INT-6A integer dtype rules sentinels."""

from __future__ import annotations

import pathlib
import sys


ROOT = pathlib.Path(__file__).resolve().parents[4]
RULES_DOC = ROOT / "verification" / "areas" / "core_language" / "data" / "integer_dtype_rules.md"

REQUIRED_TEXT = [
    "array[int32] + array[int32] -> Result[array[int32], OverflowError]",
    "array[int32] + array[int32]` cannot silently wrap",
    "array[int32] + array[int32]` cannot accidentally widen to `array[int]`",
    "Constructing compact column, tensor, or array storage from `list[int]` requires",
    "SIFR-INT-0008",
    "Arrow and Parquet integer columns map to matching fixed-width Sifr dtypes",
    "must not silently widen external integer columns to source-level",
]


def main() -> int:
    if not RULES_DOC.is_file():
        print(f"integer dtype rules missing: {RULES_DOC}", file=sys.stderr)
        return 1

    text = RULES_DOC.read_text(encoding="utf-8")
    missing = [required for required in REQUIRED_TEXT if required not in text]
    if missing:
        for required in missing:
            print(
                f"integer dtype rules missing required sentinel: {required}",
                file=sys.stderr,
            )
        return 1

    print("integer dtype rules sentinels ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
