#!/usr/bin/env python3
"""Validate governed GA release claims and their mutation harness."""

from __future__ import annotations

import argparse
import tempfile
from pathlib import Path

REQUIRED_FACTS = (
    "https://sifr.sh/install",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "macOS 15.0",
    "glibc 2.39",
)
FORBIDDEN_CLAIMS = (
    "cryptographically signed",
    "notarized",
    "Windows installer",
    "all Rust crates are supported",
    "-rc.",
)
MUTATION_CASES = (
    "missing-stable-entrypoint",
    "unsupported-target-claim",
    "unsupported-rust-claim",
    "signing-claim",
    "notarization-claim",
)


def validate_text(text: str) -> None:
    for fact in REQUIRED_FACTS:
        if fact not in text:
            raise ValueError(f"missing governed GA fact: {fact}")
    for claim in FORBIDDEN_CLAIMS:
        if claim in text:
            raise ValueError(f"forbidden or unsupported GA claim: {claim}")


def run_self_test() -> None:
    valid = "\n".join(REQUIRED_FACTS)
    validate_text(valid)
    mutations = {
        "missing-stable-entrypoint": valid.replace("https://sifr.sh/install", ""),
        "unsupported-target-claim": valid.replace(
            "aarch64-unknown-linux-gnu",
            "aarch64-pc-windows-msvc",
        ),
        "unsupported-rust-claim": valid + "\nall Rust crates are supported",
        "signing-claim": valid + "\ncryptographically signed",
        "notarization-claim": valid + "\nnotarized",
    }
    if tuple(mutations) != MUTATION_CASES:
        raise ValueError("GA documentation mutation registration drifted")
    for case_id, text in mutations.items():
        try:
            validate_text(text)
        except ValueError:
            continue
        raise ValueError(f"GA documentation mutation unexpectedly passed: {case_id}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument(
        "--document",
        action="append",
        default=[],
        help="GA document to validate; repeat for every canonical document.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.self_test:
        with tempfile.TemporaryDirectory():
            run_self_test()
        print("GA documentation mutation harness ok")
        return 0
    if not args.document:
        raise SystemExit("GA documentation check requires at least one document")
    text = "\n".join(Path(path).read_text(encoding="utf-8") for path in args.document)
    validate_text(text)
    print("GA documentation claims ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
