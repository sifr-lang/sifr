#!/usr/bin/env python3
"""Enforce Phase 37 package-manager maintainability boundaries."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import List


MAX_LINES_BY_GLOB = {
    "crates/sifr_package/src/**/*.rs": 420,
}

REQUIRED_FILES = [
    "crates/sifr_package/DEPENDENCY_AUDIT.md",
    "crates/sifr_package/TRACEABILITY.md",
    "crates/sifr_package/FEATURES.md",
    "crates/sifr_package/src/cargo/metadata.rs",
    "crates/sifr_package/src/manifest/sifr.rs",
    "crates/sifr_package/src/manifest/metadata.rs",
    "crates/sifr_package/src/source/layout.rs",
    "crates/sifr_package/src/ops/plan.rs",
]

CARGO_COMMAND_TERMS = [
    "cargo metadata",
    "cargo fetch",
    "cargo package",
    "cargo publish",
    "cargo vendor",
]

BANNED_PUBLIC_API_TERMS = [
    "cargo_metadata::",
    "cargo::core",
    "cargo::ops",
]


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def count_lines(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce Phase 37 package-manager guardrails."
    )
    parser.add_argument(
        "--max-lines-override",
        type=int,
        default=None,
        help="Override source line limits for negative-path guardrail tests.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = repo_root()
    failures: List[str] = []

    for rel in REQUIRED_FILES:
        if not (root / rel).exists():
            failures.append(f"missing required package-manager file: {rel}")

    package_crate = root / "crates/sifr_package"
    if not package_crate.exists():
        failures.append("missing package-manager crate: crates/sifr_package")
    else:
        for pattern, configured_limit in MAX_LINES_BY_GLOB.items():
            limit = args.max_lines_override or configured_limit
            for path in sorted(root.glob(pattern)):
                lines = count_lines(path)
                if lines > limit:
                    failures.append(
                        f"{path.relative_to(root)} is {lines} lines (limit {limit}); split the module"
                    )

        for path in sorted((package_crate / "src").rglob("*.rs")):
            rel = path.relative_to(root)
            text = path.read_text(encoding="utf-8")
            if "std::process::Command" in text and "/src/cargo/" not in f"/{rel}":
                failures.append(f"{rel} shells out outside crates/sifr_package/src/cargo")
            for term in CARGO_COMMAND_TERMS:
                if term in text and "/src/cargo/" not in f"/{rel}":
                    failures.append(f"{rel} mentions `{term}` outside the cargo adapter")
            if "pub use cargo_metadata" in text or "pub type" in text and "cargo_metadata" in text:
                failures.append(f"{rel} exposes cargo_metadata types in the public facade")

        lib_rs = (package_crate / "src/lib.rs").read_text(encoding="utf-8")
        for term in BANNED_PUBLIC_API_TERMS:
            if term in lib_rs:
                failures.append(f"public sifr_package facade leaks `{term}`")

        cargo_toml = (package_crate / "Cargo.toml").read_text(encoding="utf-8")
        if "cargo_metadata" in cargo_toml or "cargo = " in cargo_toml:
            failures.append(
                "crates/sifr_package/Cargo.toml links Cargo integration crates without an audit entry"
            )

        plan_rs = (package_crate / "src/ops/plan.rs").read_text(encoding="utf-8")
        if "struct OperationPlan" not in plan_rs:
            failures.append("OperationPlan is missing from crates/sifr_package/src/ops/plan.rs")

        marker_rs = (package_crate / "src/source/layout.rs").read_text(encoding="utf-8")
        if "validate_pure_marker_source" not in marker_rs:
            failures.append("pure Sifr marker validation is missing")

        digest_rs = (package_crate / "src/graph/digest.rs").read_text(encoding="utf-8")
        if "CanonicalMetadata" not in digest_rs or "digest_graph_inputs" not in digest_rs:
            failures.append("metadata normalization digest support is missing")

    if failures:
        print("Package-manager guardrails: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print("Package-manager guardrails: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
