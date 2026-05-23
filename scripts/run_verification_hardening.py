#!/usr/bin/env python3
"""Verification hardening runner entrypoint."""

from pathlib import Path

_PARTS = (
    "core.py",
    "self_tests_and_baselines.py",
    "fixedbugs_and_crashes.py",
    "property_and_fuzz.py",
    "oss_and_determinism.py",
    "main_flow.py",
)

_PARTS_DIR = Path(__file__).with_name("run_verification_hardening")
for _part in _PARTS:
    _part_path = _PARTS_DIR / _part
    exec(compile(_part_path.read_text(encoding="utf-8"), str(_part_path), "exec"), globals())

if __name__ == "__main__":
    raise SystemExit(main())
