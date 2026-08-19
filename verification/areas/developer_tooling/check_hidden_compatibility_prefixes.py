#!/usr/bin/env python3
"""Reject removed hidden compatibility names in production and fixtures."""

from __future__ import annotations

import argparse
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCAN_ROOTS = (Path("crates"), Path("verification"))
EXCLUDED_FILES = {
    Path("verification/compatibility/pre_v1_compatibility_inventory.json"),
    Path("verification/areas/developer_tooling/check_hidden_compatibility_prefixes.py"),
}
REMOVED_PREFIXES = (
    "__compat_" + "sifr_sync_",
    "__compat_" + "sifr_concurrent_",
)


def scan(root: Path) -> list[str]:
    failures: list[str] = []
    for scan_root in SCAN_ROOTS:
        directory = root / scan_root
        if not directory.is_dir():
            continue
        for path in sorted(directory.rglob("*")):
            if not path.is_file():
                continue
            relative = path.relative_to(root)
            if relative in EXCLUDED_FILES or "target" in relative.parts:
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            for line_number, line in enumerate(text.splitlines(), start=1):
                for prefix in REMOVED_PREFIXES:
                    if prefix in line:
                        failures.append(f"{relative}:{line_number}: removed hidden prefix {prefix}")
    return failures


def run_self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        production = root / "crates/example/src/lib.rs"
        fixture = root / "verification/areas/example/fixtures/main.sifr"
        inventory = root / "verification/compatibility/pre_v1_compatibility_inventory.json"
        archive = root / "plans/issues/archive/example.md"
        for path in (production, fixture, inventory, archive):
            path.parent.mkdir(parents=True, exist_ok=True)
        production.write_text(f'const NAME: &str = "{REMOVED_PREFIXES[0]}Lock";\n', encoding="utf-8")
        fixture.write_text(f"class {REMOVED_PREFIXES[1]}Pool:\n    pass\n", encoding="utf-8")
        inventory.write_text("\n".join(REMOVED_PREFIXES), encoding="utf-8")
        archive.write_text("\n".join(REMOVED_PREFIXES), encoding="utf-8")
        failures = scan(root)
    if len(failures) != 2:
        raise SystemExit(f"hidden-prefix guard self-test expected 2 failures, got {failures}")
    if not all(prefix in "\n".join(failures) for prefix in REMOVED_PREFIXES):
        raise SystemExit("hidden-prefix guard self-test did not detect both removed prefixes")
    print("hidden compatibility prefix guard self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = scan(REPO_ROOT)
    if failures:
        for failure in failures:
            print(f"hidden compatibility prefix guard: {failure}")
        return 1
    print("hidden compatibility prefix guard: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
