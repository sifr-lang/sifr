#!/usr/bin/env python3
"""Enforce package-management contract package-manager maintainability boundaries."""

from __future__ import annotations

import argparse
import configparser
import json
import tomllib
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
    "verification/areas/package_management/data/package_e2e_fixture_matrix.json",
    "verification/areas/package_management/data/package_demo_repositories.json",
    "verification/areas/package_management/data/cargo_cli_alignment_matrix.json",
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

REQUIRED_DEMO_REPOS = {
    "sifr-demo-json",
    "sifr-demo-http",
    "sifr-demo-test-support",
    "sifr-demo-app",
    "sifr-demo-workspace",
}

REQUIRED_DEMO_VALIDATIONS = {
    "git_dependency_fetch",
    "lockfile_pins_git_revisions",
    "locked_build_after_fetch",
    "offline_requires_fetch",
    "archive_missing_sifr_source_rejection",
    "pure_marker_rejects_rust_implementation",
    "rust_backed_trust_requires_reqwest",
    "multiple_version_alias_identity",
    "workspace_filter_dependency_closure",
    "workspace_default_members_and_exclude",
    "workspace_dependencies_inheritance",
    "changed_file_filters",
}


def repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def count_lines(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce package-management contract package-manager guardrails."
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


def load_toml(path: Path, failures: List[str]) -> dict[str, Any]:
    try:
        loaded = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        failures.append(f"{path} is not valid TOML: {error}")
        return {}
    if not isinstance(loaded, dict):
        failures.append(f"{path} must contain a TOML table")
        return {}
    return loaded


def load_gitmodules(root: Path, failures: List[str]) -> dict[str, dict[str, str]]:
    gitmodules_path = root / ".gitmodules"
    if not gitmodules_path.exists():
        failures.append("missing .gitmodules for package-management contract demo subrepos")
        return {}
    parser = configparser.ConfigParser()
    try:
        parser.read(gitmodules_path, encoding="utf-8")
    except configparser.Error as error:
        failures.append(f".gitmodules is not valid config: {error}")
        return {}

    submodules: dict[str, dict[str, str]] = {}
    for section in parser.sections():
        if not section.startswith("submodule "):
            continue
        path = parser.get(section, "path", fallback="")
        url = parser.get(section, "url", fallback="")
        branch = parser.get(section, "branch", fallback="")
        if path:
            submodules[path] = {"url": url, "branch": branch}
    return submodules


def check_fixture_matrix(root: Path, failures: List[str]) -> None:
    matrix_path = root / "verification/areas/package_management/data/package_e2e_fixture_matrix.json"
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


def check_demo_repositories(root: Path, failures: List[str]) -> None:
    manifest_path = root / "verification/areas/package_management/data/package_demo_repositories.json"
    if not manifest_path.exists():
        return

    manifest = load_json(manifest_path, failures)
    if manifest.get("checkout_model") != "git_submodule":
        failures.append("package-management contract demo manifest checkout_model must be git_submodule")
    repository_root_value = manifest.get("repository_root")
    if not isinstance(repository_root_value, str) or not repository_root_value:
        failures.append("package-management contract demo manifest is missing repository_root")
        return
    repository_root = root / repository_root_value
    if not repository_root.exists():
        failures.append(f"package-management contract demo repository root does not exist: {repository_root_value}")
        return
    submodules = load_gitmodules(root, failures)

    validations = manifest.get("required_validations")
    if not isinstance(validations, list):
        failures.append("package-management contract demo manifest required_validations must be a list")
    else:
        missing_validations = REQUIRED_DEMO_VALIDATIONS - set(validations)
        if missing_validations:
            failures.append(
                "package-management contract demo manifest is missing validations: "
                + ", ".join(sorted(missing_validations))
            )

    repositories = manifest.get("repositories", [])
    if not isinstance(repositories, list):
        failures.append("package-management contract demo manifest repositories must be a list")
        return

    seen_ids = set()
    for repo in repositories:
        if not isinstance(repo, dict):
            failures.append("package-management contract demo repository entries must be JSON objects")
            continue
        repo_id = repo.get("id")
        repo_root = repo.get("root")
        required_paths = repo.get("required_paths")
        repo_validations = repo.get("validations")
        if not isinstance(repo_id, str) or not repo_id:
            failures.append("package-management contract demo repository entry is missing id")
            continue
        seen_ids.add(repo_id)
        if not isinstance(repo_root, str) or not repo_root:
            failures.append(f"package-management contract demo repository `{repo_id}` is missing root")
            continue
        repo_path = repository_root / repo_root
        if not repo_path.exists():
            failures.append(f"package-management contract demo repository `{repo_id}` root is missing")
            continue
        repo_rel_path = f"{repository_root_value}/{repo_root}"
        check_demo_submodule(repo_id, repo_rel_path, repo.get("url"), submodules, failures)
        if not isinstance(required_paths, list):
            failures.append(f"package-management contract demo repository `{repo_id}` required_paths must be a list")
            continue
        for rel_path in required_paths:
            if not isinstance(rel_path, str) or not rel_path:
                failures.append(f"package-management contract demo repository `{repo_id}` has invalid required path")
                continue
            if not (repo_path / rel_path).exists():
                failures.append(
                    f"package-management contract demo repository `{repo_id}` is missing required path {rel_path}"
                )
        if not isinstance(repo_validations, list) or not repo_validations:
            failures.append(f"package-management contract demo repository `{repo_id}` has no validations")

        check_demo_repository_shape(repo_id, repo_path, failures)

    missing_repos = REQUIRED_DEMO_REPOS - seen_ids
    if missing_repos:
        failures.append(
            "package-management contract demo manifest is missing repositories: "
            + ", ".join(sorted(missing_repos))
        )


def check_demo_submodule(
    repo_id: str,
    repo_rel_path: str,
    repo_url: object,
    submodules: dict[str, dict[str, str]],
    failures: List[str],
) -> None:
    entry = submodules.get(repo_rel_path)
    if entry is None:
        failures.append(f"package-management contract demo repository `{repo_id}` is missing from .gitmodules")
        return
    if not isinstance(repo_url, str) or not repo_url:
        failures.append(f"package-management contract demo repository `{repo_id}` is missing url")
        return
    expected_urls = {repo_url, f"{repo_url}.git"}
    if entry.get("url") not in expected_urls:
        failures.append(
            f"package-management contract demo repository `{repo_id}` has unexpected submodule URL {entry.get('url')}"
        )
    if entry.get("branch") != "main":
        failures.append(f"package-management contract demo repository `{repo_id}` submodule must track main")


def check_demo_repository_shape(repo_id: str, repo_path: Path, failures: List[str]) -> None:
    cargo_toml = load_toml(repo_path / "Cargo.toml", failures)
    sifr_toml_path = repo_path / "sifr.toml"

    if repo_id != "sifr-demo-workspace":
        manifest = (
            cargo_toml.get("package", {})
            .get("metadata", {})
            .get("sifr", {})
            .get("manifest")
        )
        if manifest != "sifr.toml":
            failures.append(f"package-management contract demo repository `{repo_id}` must link sifr.toml")
        if not sifr_toml_path.exists():
            failures.append(f"package-management contract demo repository `{repo_id}` is missing sifr.toml")
        else:
            check_production_sifr_manifest(repo_id, sifr_toml_path, failures)
        check_cargo_projection_markers(repo_id, repo_path / "Cargo.toml", failures)
        check_src_layout(repo_id, repo_path, failures)

    if repo_id == "sifr-demo-json":
        check_pure_marker(repo_id, repo_path / "src/lib.rs", failures)
        text = (repo_path / "Cargo.toml").read_text(encoding="utf-8")
        if "src/**/*.sifr" not in text:
            failures.append("sifr-demo-json must include src Sifr sources for archive validation")
    elif repo_id == "sifr-demo-http":
        check_rust_backed_http_template(repo_path, failures)
    elif repo_id == "sifr-demo-app":
        check_consumer_app_template(repo_path, failures)
    elif repo_id == "sifr-demo-workspace":
        check_workspace_template(repo_path, failures)
    elif repo_id == "sifr-demo-test-support":
        check_pure_marker(repo_id, repo_path / "src/lib.rs", failures)


def check_production_sifr_manifest(repo_id: str, manifest_path: Path, failures: List[str]) -> None:
    manifest = load_toml(manifest_path, failures)
    if "exports" in manifest:
        failures.append(f"{repo_id} must not use production-forbidden [exports]")
    if "bin" in manifest:
        failures.append(f"{repo_id} must not use Sifr manifest [[bin]] tables")
    source = manifest.get("source", {})
    if isinstance(source, dict) and "roots" in source:
        failures.append(f"{repo_id} must use canonical [source].root/default src layout")
    scripts = manifest.get("scripts", {})
    if scripts and not isinstance(scripts, dict):
        failures.append(f"{repo_id} [scripts] must be a table of command plans")
    if isinstance(scripts, dict):
        for name, script in scripts.items():
            if not isinstance(script, dict) or not isinstance(script.get("command"), str):
                failures.append(f"{repo_id} script `{name}` must be a structured command plan")


def check_cargo_projection_markers(repo_id: str, cargo_toml_path: Path, failures: List[str]) -> None:
    text = cargo_toml_path.read_text(encoding="utf-8")
    if "[package.metadata.sifr]" in text and "# sifr-managed" not in text:
        failures.append(f"{repo_id} Cargo projection metadata must use # sifr-managed markers")
    if "sifr/**/*.sifr" in text:
        failures.append(f"{repo_id} Cargo projection must not include legacy sifr/**/*.sifr")


def check_src_layout(repo_id: str, repo_path: Path, failures: List[str]) -> None:
    if (repo_path / "sifr").exists():
        failures.append(f"{repo_id} must not keep legacy sifr/ package sources")
    if not (repo_path / "src").exists():
        failures.append(f"{repo_id} must use canonical src/ package sources")


def check_pure_marker(repo_id: str, marker_path: Path, failures: List[str]) -> None:
    marker = marker_path.read_text(encoding="utf-8")
    if "Pure Sifr package marker" not in marker:
        failures.append(f"package-management contract demo repository `{repo_id}` is missing the pure marker")
    for forbidden in ["pub fn", "pub mod", "use ", "macro_rules!"]:
        if forbidden in marker:
            failures.append(f"package-management contract demo repository `{repo_id}` marker contains Rust code")


def check_rust_backed_http_template(repo_path: Path, failures: List[str]) -> None:
    cargo_toml = (repo_path / "Cargo.toml").read_text(encoding="utf-8")
    sifr_toml = (repo_path / "sifr.toml").read_text(encoding="utf-8")
    rust_source = (repo_path / "src/lib.rs").read_text(encoding="utf-8")
    if "sifr-demo-json" not in cargo_toml or "path = \"../sifr-demo-json\"" not in cargo_toml:
        failures.append("sifr-demo-http must depend on local sifr-demo-json")
    if "reqwest" not in cargo_toml or "reqwest" not in sifr_toml:
        failures.append("sifr-demo-http must trust and depend on reqwest")
    if "reqwest::" not in rust_source:
        failures.append("sifr-demo-http Rust shim must exercise reqwest")


def check_consumer_app_template(repo_path: Path, failures: List[str]) -> None:
    cargo_toml = (repo_path / "Cargo.toml").read_text(encoding="utf-8")
    sifr_toml = load_toml(repo_path / "sifr.toml", failures)
    lockfile = (repo_path / "Cargo.lock").read_text(encoding="utf-8")
    migrate = (repo_path / "src/migrate.sifr").read_text(encoding="utf-8")
    for required in [
        "sifr-demo-json",
        "sifr-demo-http",
        "sifr-demo-test-support",
        "demo_json_v1",
        "demo_json_v2",
        "path = \"../sifr-demo-json\"",
        "path = \"../sifr-demo-json-v2\"",
    ]:
        if required not in cargo_toml:
            failures.append(f"sifr-demo-app Cargo.toml is missing `{required}`")
    if 'name = "sifr-demo-json"\nversion = "0.1.0"' not in lockfile:
        failures.append("sifr-demo-app lockfile must include the v0.1.0 alias")
    if 'name = "sifr-demo-json"\nversion = "0.2.0"' not in lockfile:
        failures.append("sifr-demo-app lockfile must include the v0.2.0 alias")
    if "demo_json_v1" not in migrate or "demo_json_v2" not in migrate:
        failures.append("sifr-demo-app migrate.sifr must import both alias roots")
    dependencies = sifr_toml.get("dependencies", {})
    if "demo_json_v1" not in dependencies or "demo_json_v2" not in dependencies:
        failures.append("sifr-demo-app sifr.toml must declare both Sifr-facing aliases")
    scripts = sifr_toml.get("scripts", {})
    for script in ["dev", "check-offline", "publish-dry-run"]:
        if script not in scripts:
            failures.append(f"sifr-demo-app missing script command plan `{script}`")


def check_workspace_template(repo_path: Path, failures: List[str]) -> None:
    workspace = load_toml(repo_path / "Cargo.toml", failures).get("workspace", {})
    if workspace.get("default-members") != ["packages/app", "packages/core"]:
        failures.append("sifr-demo-workspace must set default-members")
    if workspace.get("exclude") != ["packages/experimental-*"]:
        failures.append("sifr-demo-workspace must set exclude")
    dependencies = load_toml(repo_path / "Cargo.toml", failures).get("workspace", {}).get(
        "dependencies", {}
    )
    if "sifr-demo-core" not in dependencies or "sifr-demo-utils" not in dependencies:
        failures.append("sifr-demo-workspace must define workspace Sifr dependencies")
    app_toml = (repo_path / "packages/app/Cargo.toml").read_text(encoding="utf-8")
    if "workspace = true" not in app_toml or "backend-utils" not in app_toml:
        failures.append("sifr-demo-workspace app must inherit workspace deps and reach backend")
    app_manifest = load_toml(repo_path / "packages/app/sifr.toml", failures)
    scripts = app_manifest.get("scripts", {})
    if "status-smoke" not in scripts:
        failures.append("sifr-demo-workspace app must define status-smoke run script")
    if not (repo_path / "packages/app/src/bin/status.sifr").exists():
        failures.append("sifr-demo-workspace app must include src/bin/status.sifr")
    for member in ["core", "utils", "app"]:
        member_root = repo_path / f"packages/{member}"
        check_production_sifr_manifest(
            f"sifr-demo-workspace/packages/{member}",
            member_root / "sifr.toml",
            failures,
        )
        check_cargo_projection_markers(
            f"sifr-demo-workspace/packages/{member}",
            member_root / "Cargo.toml",
            failures,
        )
        check_src_layout(f"sifr-demo-workspace/packages/{member}", member_root, failures)
        check_pure_marker(
            f"sifr-demo-workspace/packages/{member}",
            repo_path / f"packages/{member}/src/lib.rs",
            failures,
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
    check_demo_repositories(root, failures)

    if failures:
        print("Package-manager guardrails: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print("Package-manager guardrails: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
