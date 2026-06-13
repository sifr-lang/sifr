#!/usr/bin/env python3
"""Validate that legacy audits are normalized under verification areas."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]

AUDIT_MANIFESTS = [
    REPO_ROOT / "verification/areas/core_language/data/audit_fixtures.json",
    REPO_ROOT / "verification/areas/project_workspace/data/audit_fixtures.json",
    REPO_ROOT / "verification/areas/stdlib_parity/data/audit_fixtures.json",
]

ACTIVE_REFERENCE_ROOTS = [
    REPO_ROOT / ".github",
    REPO_ROOT / "AGENTS.md",
    REPO_ROOT / "README.md",
    REPO_ROOT / "scripts",
    REPO_ROOT / "verification",
]
REFERENCE_SCAN_EXCLUSIONS = {
    "scripts/check_audits_normalization.py",
    "scripts/check_submodule_ownership.py",
}
TOP_LEVEL_AUDITS_REF_RE = re.compile(r"(?<![A-Za-z0-9_./-])audits/")


def validate(root: Path) -> list[str]:
    failures: list[str] = []
    audits_root = root / "audits"
    if audits_root.exists():
        failures.append("top-level audits/ must not exist")

    manifest_path_counts: dict[str, int] = {}
    for manifest_path in AUDIT_MANIFESTS:
        failures.extend(validate_manifest(root, manifest_path, manifest_path_counts))

    for path in sorted(path for path, count in manifest_path_counts.items() if count > 1):
        failures.append(f"audit fixture path is owned by multiple manifests: {path}")

    failures.extend(validate_stale_references(root))
    return failures


def validate_manifest(root: Path, manifest_path: Path, path_counts: dict[str, int]) -> list[str]:
    failures: list[str] = []
    if not manifest_path.is_file():
        return [f"missing audit fixture manifest: {repo_path(root, manifest_path)}"]
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    entries = manifest.get("entries")
    fixture_root_raw = manifest.get("fixture_root")
    if not isinstance(entries, list) or not entries:
        failures.append(f"{repo_path(root, manifest_path)} has no audit fixture entries")
        return failures
    if not isinstance(fixture_root_raw, str) or not fixture_root_raw:
        failures.append(f"{repo_path(root, manifest_path)} has invalid fixture_root")
        return failures

    fixture_root = root / fixture_root_raw
    actual_paths = {
        repo_path(root, path)
        for path in fixture_root.rglob("*.sifr")
        if path.is_file()
    }
    entry_paths: set[str] = set()
    smoke_count = 0
    for raw_entry in entries:
        if not isinstance(raw_entry, dict):
            failures.append(f"{repo_path(root, manifest_path)} contains non-object entry")
            continue
        entry_path = raw_entry.get("path")
        if not isinstance(entry_path, str) or not entry_path.endswith(".sifr"):
            failures.append(f"{repo_path(root, manifest_path)} contains invalid path entry")
            continue
        if entry_path in entry_paths:
            failures.append(f"{repo_path(root, manifest_path)} lists duplicate fixture path: {entry_path}")
        entry_paths.add(entry_path)
        path_counts[entry_path] = path_counts.get(entry_path, 0) + 1
        if raw_entry.get("smoke") is True:
            smoke_count += 1
    for path in sorted(actual_paths - entry_paths):
        failures.append(f"promoted audit fixture missing from manifest: {path}")
    for path in sorted(entry_paths - actual_paths):
        failures.append(f"audit manifest references missing fixture: {path}")
    if smoke_count == 0:
        failures.append(f"{repo_path(root, manifest_path)} must contain at least one smoke fixture")

    stray_reports = [
        repo_path(root, path)
        for path in fixture_root.rglob("*.md")
        if path.is_file()
    ]
    for path in sorted(stray_reports):
        failures.append(f"historical audit report remains under fixture root: {path}")
    return failures


def validate_stale_references(root: Path) -> list[str]:
    failures: list[str] = []
    for file_path in iter_reference_files(root):
        try:
            text = file_path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        if TOP_LEVEL_AUDITS_REF_RE.search(text):
            failures.append(f"{repo_path(root, file_path)} references top-level audits/")
    return failures


def iter_reference_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for path in ACTIVE_REFERENCE_ROOTS:
        if path.is_file():
            files.append(path)
        elif path.is_dir():
            files.extend(
                child
                for child in path.rglob("*")
                if child.is_file()
                and not is_under_submodule(root, child)
                and child.relative_to(root).as_posix() not in REFERENCE_SCAN_EXCLUSIONS
                and ".git" not in child.parts
                and "__pycache__" not in child.parts
                and child.suffix not in {".pyc", ".lock", ".png", ".webp"}
            )
    return sorted(set(files))


def is_under_submodule(root: Path, path: Path) -> bool:
    for parent in path.parents:
        if parent == root:
            return False
        if (parent / ".git").exists():
            return True
    return False


def repo_path(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def run_self_test() -> None:
    valid_manifest = {
        "entries": [
            {"path": "verification/areas/core_language/fixtures/audits/x.sifr", "smoke": True}
        ],
        "fixture_root": "verification/areas/core_language/fixtures/audits",
    }
    if not isinstance(valid_manifest["entries"], list):
        raise SystemExit("audits normalization self-test failed: valid manifest rejected")
    if not TOP_LEVEL_AUDITS_REF_RE.search("see audits/old") or TOP_LEVEL_AUDITS_REF_RE.search(
        "verification/areas/core_language/fixtures/audits/x.sifr"
    ):
        raise SystemExit("audits normalization self-test failed: audits/ pattern missing")
    print("audits normalization self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = validate(REPO_ROOT)
    if failures:
        print("audits normalization guardrail: FAIL")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("audits normalization guardrail: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
