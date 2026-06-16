#!/usr/bin/env python3
"""Validate stdlib complexity/resource inventory coverage and waiver discipline."""

from __future__ import annotations

import json
import sys
from pathlib import Path


EXPECTED_MODULES = {
    "env",
    "bytes",
    "base64",
    "hashlib",
    "math",
    "statistics",
    "bisect",
    "heapq",
    "string",
    "textwrap",
    "fnmatch",
    "re",
    "collections",
    "itertools",
    "json",
    "datetime",
    "io",
    "csv",
    "os",
    "pathlib",
    "glob",
    "tempfile",
    "shutil",
    "logging",
    "time",
    "timeit",
    "platform",
    "uuid",
}

REQUIRED_ENTRY_FIELDS = {
    "module",
    "api_class",
    "representative_apis",
    "expected_asymptotic",
    "observed_asymptotic",
    "constant_factor_delta_band",
    "resource_budget",
    "waiver",
}

REQUIRED_WAIVER_FIELDS = {"owner", "rationale", "tracking_issue", "revisit_rule"}
ALLOWED_DELTA_BANDS = {"within_2x", "within_5x", "within_10x", "waived"}


def fail(message: str) -> None:
    print(f"stdlib complexity inventory: FAIL - {message}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    area_root = Path(__file__).resolve().parents[1]
    inventory_path = area_root / "data/stdlib_complexity_resource_inventory.json"
    if not inventory_path.is_file():
        fail(f"missing inventory file: {inventory_path}")

    payload = json.loads(inventory_path.read_text(encoding="utf-8"))
    entries = payload.get("entries")
    if not isinstance(entries, list):
        fail("inventory payload must contain list field 'entries'")

    seen_modules: set[str] = set()
    waived_count = 0
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            fail(f"entry[{index}] is not an object")

        missing = REQUIRED_ENTRY_FIELDS - set(entry.keys())
        if missing:
            fail(f"entry[{index}] missing fields: {sorted(missing)}")

        module = entry["module"]
        if not isinstance(module, str) or not module:
            fail(f"entry[{index}] has invalid module")
        if module not in EXPECTED_MODULES:
            fail(f"entry[{index}] references unknown module '{module}'")
        if module in seen_modules:
            fail(f"duplicate module entry '{module}'")
        seen_modules.add(module)

        representative_apis = entry["representative_apis"]
        if (
            not isinstance(representative_apis, list)
            or not representative_apis
            or not all(isinstance(item, str) and item for item in representative_apis)
        ):
            fail(f"entry[{index}] has invalid representative_apis")

        delta_band = entry["constant_factor_delta_band"]
        if delta_band not in ALLOWED_DELTA_BANDS:
            fail(f"entry[{index}] has invalid constant_factor_delta_band '{delta_band}'")

        waiver = entry["waiver"]
        if delta_band == "waived":
            waived_count += 1
            if not isinstance(waiver, dict):
                fail(f"entry[{index}] must include waiver object when delta band is 'waived'")
            missing_waiver_fields = REQUIRED_WAIVER_FIELDS - set(waiver.keys())
            if missing_waiver_fields:
                fail(
                    f"entry[{index}] waiver missing fields: {sorted(missing_waiver_fields)}"
                )
            for field in REQUIRED_WAIVER_FIELDS:
                value = waiver[field]
                if not isinstance(value, str) or not value.strip():
                    fail(f"entry[{index}] waiver field '{field}' must be non-empty string")
        else:
            if waiver is not None:
                fail(
                    f"entry[{index}] has non-null waiver with non-waived delta band '{delta_band}'"
                )

    missing_modules = sorted(EXPECTED_MODULES - seen_modules)
    if missing_modules:
        fail(f"missing module entries: {missing_modules}")

    print(
        "stdlib complexity inventory: PASS "
        f"(modules={len(entries)}, waived_constant_factor={waived_count})"
    )


if __name__ == "__main__":
    main()
