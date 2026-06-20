"""Rust interop verification area adapter."""

from __future__ import annotations

import argparse
from pathlib import Path

from sifr_verify.area_adapter import AreaAdapterConfig, AreaRunOptions, run_area

REPO_ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = Path(__file__).resolve().with_name("manifest.json")
ACTUAL_ROOT = REPO_ROOT / "target" / "verification" / "actual" / "rust_interop"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "rust-interop-results.json"
CONFIG = AreaAdapterConfig(
    area="rust_interop",
    owner="compiler/package-management",
    runner_name="rust-interop-area",
    manifest_path=MANIFEST_PATH,
    actual_root=ACTUAL_ROOT,
    status_label="rust interop",
)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Update checked-in baselines.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable rust interop area result summary.",
    )
    parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit the legacy hardening summary line consumed by validation reports.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    return run_area(
        CONFIG,
        AreaRunOptions(
            suite_filters=set(args.suite),
            bless=args.bless,
            result_json=Path(args.result_json),
            hardening_summary=args.hardening_summary,
        ),
    )


if __name__ == "__main__":
    raise SystemExit(main())
