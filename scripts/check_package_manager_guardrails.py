#!/usr/bin/env python3
"""Enforce Phase 37 package-manager maintainability boundaries."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, List


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
    "verification/package_management/phase37_e2e_fixture_matrix.json",
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

REQUIRED_FIXTURE_CATEGORIES = {
    "pure_sifr_cargo_package",
    "rust_backed_sifr_package",
    "workspace_selection",
    "path_dependency",
    "git_dependency",
    "registry_dependency",
    "multiple_version_graph",
    "alias_imports",
    "publishing",
}


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


def load_json(path: Path, failures: List[str]) -> dict[str, Any]:
    try:
        loaded = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        failures.append(f"{path} is not valid JSON: {error}")
        return {}
    if not isinstance(loaded, dict):
        failures.append(f"{path} must contain a JSON object")
        return {}
    return loaded


def check_fixture_matrix(root: Path, failures: List[str]) -> None:
    matrix_path = root / "verification/package_management/phase37_e2e_fixture_matrix.json"
    if not matrix_path.exists():
        return

    matrix = load_json(matrix_path, failures)
    fixtures = matrix.get("fixtures", [])
    if not isinstance(fixtures, list):
        failures.append("package-management fixture matrix `fixtures` must be a list")
        return

    categories = set()
    for fixture in fixtures:
        if not isinstance(fixture, dict):
            failures.append("package-management fixture entries must be JSON objects")
            continue
        category = fixture.get("category")
        coverage = fixture.get("coverage")
        status = fixture.get("status")
        if not isinstance(category, str) or not category:
            failures.append("package-management fixture entry is missing a category")
            continue
        categories.add(category)
        if status not in {"ported", "adapted", "non-port"}:
            failures.append(f"package-management fixture `{category}` has invalid status")
        if not isinstance(coverage, list) or not all(
            isinstance(item, str) and item for item in coverage
        ):
            failures.append(f"package-management fixture `{category}` has no coverage")

    missing = sorted(REQUIRED_FIXTURE_CATEGORIES - categories)
    if missing:
        failures.append(
            "package-management fixture matrix is missing required categories: "
            + ", ".join(missing)
        )


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

    check_fixture_matrix(root, failures)

    if failures:
        print("Package-manager guardrails: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print("Package-manager guardrails: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
