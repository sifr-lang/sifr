#!/usr/bin/env python3
"""Validate production formatter rules manifests and Ruff fork baseline."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
RULES_DOC = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "formatter_rules.md"
MANIFEST_DIR = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "formatter_manifests"
GITMODULES = REPO_ROOT / ".gitmodules"

ALLOWED_CLASSIFICATIONS = {"supported", "adapted", "not-applicable", "not-exposed"}
REQUIRED_AST_ROWS = {
    "param_default_borrow",
    "param_mut",
    "param_own",
    "param_own_mut",
    "param_mut_own_tolerant",
    "sifr_type_annotations",
    "sifr_generics",
    "match_case",
    "ownership_aware_collections",
    "formatter_pragmas",
    "docstring_code_snippets",
}
RUFF_FORMAT_COMMAND_MARKERS = {
    "`--check`": "pub check: bool",
    "`--diff`": "pub diff: bool",
    "`--no-cache`": "pub no_cache: bool",
    "`--cache-dir <path>`": "pub cache_dir: Option<PathBuf>",
    "`--respect-gitignore`": "respect_gitignore: bool",
    "`--no-respect-gitignore`": "no_respect_gitignore: bool",
    "`--exclude <pattern[,pattern...]>`": "pub exclude: Option<Vec<FilePattern>>",
    "`--force-exclude`": "force_exclude: bool",
    "`--no-force-exclude`": "no_force_exclude: bool",
    "`--line-length <n>`": "pub line_length: Option<LineLength>",
    "`--stdin-filename <path>`": "pub stdin_filename: Option<PathBuf>",
    "`--extension <ext:language>`": "pub extension: Option<Vec<ExtensionPair>>",
    "`--target-version <version>`": "pub target_version: Option<PythonVersion>",
    "`--preview`": "preview: bool",
    "`--no-preview`": "no_preview: bool",
    "`--range <range>`": "pub range: Option<FormatRange>",
}


def load_json(name: str) -> object:
    with (MANIFEST_DIR / name).open(encoding="utf-8") as handle:
        return json.load(handle)


def run(command: list[str], *, cwd: Path = REPO_ROOT) -> str:
    completed = subprocess.run(command, cwd=cwd, text=True, capture_output=True)
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed: {' '.join(command)}\n{completed.stdout}\n{completed.stderr}"
        )
    return completed.stdout.strip()


def require(condition: bool, message: str, failures: list[str]) -> None:
    if not condition:
        failures.append(message)


def check_capability_manifest(failures: list[str], rules_text: str) -> None:
    manifest = load_json("capability_matrix.json")
    require(isinstance(manifest, dict), "capability manifest is not an object", failures)
    rows = manifest.get("rows", []) if isinstance(manifest, dict) else []
    require(len(rows) == 35, f"capability manifest row count changed: {len(rows)}", failures)
    seen = set()
    for index, row in enumerate(rows):
        capability = row.get("capability")
        classification = row.get("classification")
        requirement = row.get("implementation_requirement")
        require(bool(capability), f"capability row {index} missing capability", failures)
        require(capability not in seen, f"duplicate capability row: {capability}", failures)
        seen.add(capability)
        require(
            classification in ALLOWED_CLASSIFICATIONS,
            f"capability {capability!r} has invalid classification {classification!r}",
            failures,
        )
        require(
            isinstance(requirement, str) and bool(requirement.strip()),
            f"capability {capability!r} missing implementation requirement",
            failures,
        )
        require(
            isinstance(capability, str) and capability in rules_text,
            f"capability {capability!r} missing from formatter rules reference",
            failures,
        )
        require(
            isinstance(requirement, str) and requirement in rules_text,
            f"capability {capability!r} requirement drifts from formatter rules reference",
            failures,
        )


def check_cli_manifest(failures: list[str], rules_text: str) -> None:
    manifest = load_json("cli_parity.json")
    require(isinstance(manifest, dict), "CLI manifest is not an object", failures)
    rows = manifest.get("rows", []) if isinstance(manifest, dict) else []
    require(len(rows) == 22, f"CLI manifest row count changed: {len(rows)}", failures)
    args_rs = (REPO_ROOT / "third_party" / "ruff" / "crates" / "ruff" / "src" / "args.rs").read_text(
        encoding="utf-8"
    )
    for index, row in enumerate(rows):
        ruff_surface = row.get("ruff_surface")
        classification = row.get("classification")
        fixture = row.get("required_fixture")
        require(bool(fixture), f"CLI row {index} missing required fixture", failures)
        require(
            classification in ALLOWED_CLASSIFICATIONS,
            f"CLI row {ruff_surface!r} has invalid classification {classification!r}",
            failures,
        )
        require(
            isinstance(ruff_surface, str) and ruff_surface in rules_text,
            f"CLI surface {ruff_surface!r} missing from formatter rules reference",
            failures,
        )
        require(
            isinstance(fixture, str) and fixture in rules_text,
            f"CLI fixture {fixture!r} missing from formatter rules reference",
            failures,
        )
        marker = RUFF_FORMAT_COMMAND_MARKERS.get(ruff_surface)
        if marker is not None:
            require(
                marker in args_rs,
                f"Ruff FormatCommand marker for {ruff_surface} is missing: {marker}",
                failures,
            )


def check_ast_manifest(failures: list[str], rules_text: str) -> None:
    manifest = load_json("ast_coverage.json")
    rows = manifest.get("rows", []) if isinstance(manifest, dict) else []
    row_ids = {row.get("id") for row in rows}
    missing = REQUIRED_AST_ROWS - row_ids
    extra = row_ids - REQUIRED_AST_ROWS
    require(not missing, f"AST coverage manifest missing rows: {sorted(missing)}", failures)
    require(not extra, f"AST coverage manifest has unreviewed rows: {sorted(extra)}", failures)
    for row in rows:
        syntax = row.get("syntax")
        behavior = row.get("required_formatter_behavior")
        row_id = row.get("id")
        require(isinstance(row_id, str) and row_id in rules_text, f"AST row {row_id!r} missing from formatter rules reference", failures)
        require(bool(syntax), f"AST row {row.get('id')!r} missing syntax", failures)
        require(bool(behavior), f"AST row {row.get('id')!r} missing behavior", failures)
        require(
            isinstance(syntax, str) and syntax in rules_text,
            f"AST row {row_id!r} syntax missing from formatter rules reference",
            failures,
        )
        require(
            isinstance(behavior, str) and behavior in rules_text,
            f"AST row {row_id!r} behavior drifts from formatter rules reference",
            failures,
        )


def check_baseline(failures: list[str]) -> None:
    manifest = load_json("ruff_baseline.json")
    require(isinstance(manifest, dict), "baseline manifest is not an object", failures)
    path = manifest.get("submodule_path")
    required_commit = manifest.get("required_seed_commit")
    required_branch = manifest.get("required_branch")
    repository = manifest.get("repository")
    submodule = REPO_ROOT / str(path)
    gitmodules = GITMODULES.read_text(encoding="utf-8")
    require(f"url = {repository}" in gitmodules, "third_party/ruff .gitmodules URL drifted", failures)
    require(
        f"branch = {required_branch}" in gitmodules,
        "third_party/ruff .gitmodules branch drifted",
        failures,
    )
    try:
        head = run(["git", "-C", str(submodule), "rev-parse", "HEAD"])
        run(["git", "-C", str(submodule), "merge-base", "--is-ancestor", str(required_commit), head])
        contains = run(["git", "-C", str(submodule), "branch", "-a", "--contains", head])
        seed_subject = run(["git", "-C", str(submodule), "show", "-s", "--format=%s", str(required_commit)])
        seed_paths = run(["git", "-C", str(submodule), "show", "--format=", "--name-only", str(required_commit)])
    except RuntimeError as error:
        failures.append(str(error))
        return
    require(
        str(required_branch) in contains or f"remotes/origin/{required_branch}" in contains,
        f"third_party/ruff HEAD {head} is not contained in {required_branch}",
        failures,
    )
    require(
        seed_subject == manifest.get("required_seed_subject"),
        "required Ruff seed commit subject drifted",
        failures,
    )
    for required_path in manifest.get("required_seed_paths", []):
        require(required_path in seed_paths, f"required seed path missing from seed commit: {required_path}", failures)


def run_positive() -> None:
    rules_text = RULES_DOC.read_text(encoding="utf-8")
    failures: list[str] = []
    check_capability_manifest(failures, rules_text)
    check_cli_manifest(failures, rules_text)
    check_ast_manifest(failures, rules_text)
    check_baseline(failures)
    if failures:
        print("formatter rules manifests: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        raise SystemExit(1)
    print("formatter rules manifests: PASS")


def run_self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        bad_manifest = Path(tmp) / "capability_matrix.json"
        bad_manifest.write_text(
            json.dumps(
                {
                    "schema": 1,
                    "allowed_classifications": sorted(ALLOWED_CLASSIFICATIONS),
                    "rows": [
                        {
                            "capability": "Whole-file formatting",
                            "classification": "deferred",
                            "implementation_requirement": "bad",
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )
        failures: list[str] = []
        original_manifest_dir = globals()["MANIFEST_DIR"]
        try:
            globals()["MANIFEST_DIR"] = Path(tmp)
            check_capability_manifest(failures, "Whole-file formatting")
        finally:
            globals()["MANIFEST_DIR"] = original_manifest_dir
        if not any("invalid classification" in failure for failure in failures):
            raise SystemExit("formatter rules manifest self-test failed: invalid classification passed")
    print("formatter rules manifests self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
    else:
        run_positive()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
