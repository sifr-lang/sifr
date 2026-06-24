"""Audit fixture manifest validation and bounded smoke execution."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from .paths import REPO_ROOT

DEFAULT_TIMEOUT_SECONDS = 10
DEFAULT_SIFR_BIN = REPO_ROOT / "target" / "debug" / "sifr"


def run_audit_fixture_manifest(manifest_path: Path, *, area: str) -> int:
    manifest = load_manifest(manifest_path)
    failures = validate_manifest(manifest_path, manifest, area=area)
    if failures:
        emit_failures(area, failures)
        return 1

    smoke_entries = [entry for entry in manifest["entries"] if entry.get("smoke") is True]
    run_failures = run_smoke_entries(area, smoke_entries)
    if run_failures:
        emit_failures(area, run_failures)
        return 1

    print(
        f"{area} audit fixtures ok: fixtures={len(manifest['entries'])}, "
        f"smoke={len(smoke_entries)}"
    )
    return 0


def load_manifest(manifest_path: Path) -> dict[str, Any]:
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def validate_manifest(manifest_path: Path, manifest: dict[str, Any], *, area: str) -> list[str]:
    failures: list[str] = []
    if manifest.get("schema_version") != 1:
        failures.append(f"{manifest_path}: schema_version must be 1")
    if manifest.get("area") != area:
        failures.append(f"{manifest_path}: area must be {area!r}")

    fixture_root_raw = manifest.get("fixture_root")
    if not isinstance(fixture_root_raw, str) or not fixture_root_raw:
        failures.append(f"{manifest_path}: fixture_root must be a non-empty string")
        return failures
    fixture_root = resolve_repo_path(fixture_root_raw)
    if not fixture_root.is_dir():
        failures.append(f"{fixture_root_raw}: fixture root does not exist")
        return failures

    entries = manifest.get("entries")
    if not isinstance(entries, list) or not entries:
        failures.append(f"{manifest_path}: entries must be a non-empty list")
        return failures

    manifest_paths: set[str] = set()
    ids: list[str] = []
    smoke_count = 0
    for raw_entry in entries:
        if not isinstance(raw_entry, dict):
            failures.append(f"{manifest_path}: entry must be an object")
            continue
        entry_id = raw_entry.get("id")
        entry_path = raw_entry.get("path")
        category = raw_entry.get("category")
        command = raw_entry.get("command")
        smoke = raw_entry.get("smoke")
        if not isinstance(entry_id, str) or not entry_id:
            failures.append(f"{manifest_path}: entry has invalid id: {raw_entry!r}")
            continue
        if not isinstance(entry_path, str) or not entry_path.endswith(".sifr"):
            failures.append(f"{entry_id}: path must be a .sifr repo-relative path")
            continue
        if not isinstance(category, str) or not category:
            failures.append(f"{entry_id}: category must be a non-empty string")
        if command != "check":
            failures.append(f"{entry_id}: audit fixture command must be check")
        if not isinstance(smoke, bool):
            failures.append(f"{entry_id}: smoke must be a boolean")
        if smoke:
            smoke_count += 1
            if not isinstance(raw_entry.get("expect_exit_code"), int):
                failures.append(f"{entry_id}: smoke entries must declare expect_exit_code")
        elif "expect_exit_code" in raw_entry:
            failures.append(f"{entry_id}: non-smoke entries must not declare expect_exit_code")

        path = resolve_repo_path(entry_path)
        if not path.is_file():
            failures.append(f"{entry_id}: fixture path does not exist: {entry_path}")
            continue
        formatted_path = format_repo_relative_path(path)
        if formatted_path in manifest_paths:
            failures.append(f"{entry_id}: duplicate fixture path in manifest: {formatted_path}")
        try:
            path.relative_to(fixture_root)
        except ValueError:
            failures.append(f"{entry_id}: fixture path is outside fixture_root: {entry_path}")
        manifest_paths.add(formatted_path)
        ids.append(entry_id)

    if ids != sorted(ids):
        failures.append(f"{manifest_path}: entries must be sorted by id")
    if len(ids) != len(set(ids)):
        failures.append(f"{manifest_path}: entry ids must be unique")
    if smoke_count == 0:
        failures.append(f"{manifest_path}: at least one smoke fixture is required")

    actual_paths = {
        format_repo_relative_path(path)
        for path in fixture_root.rglob("*.sifr")
        if path.is_file()
    }
    for path in sorted(actual_paths - manifest_paths):
        failures.append(f"fixture missing from manifest: {path}")
    for path in sorted(manifest_paths - actual_paths):
        failures.append(f"manifest references missing fixture: {path}")

    stray_files = [
        format_repo_relative_path(path)
        for path in fixture_root.rglob("*")
        if path.is_file() and path.suffix != ".sifr"
    ]
    for path in sorted(stray_files):
        failures.append(f"non-fixture file under audit fixture root: {path}")
    return failures


def run_smoke_entries(area: str, entries: list[dict[str, Any]]) -> list[str]:
    failures: list[str] = []
    command_prefix = audit_fixture_command_prefix()
    for entry in entries:
        entry_id = str(entry["id"])
        entry_path = resolve_repo_path(str(entry["path"]))
        expected_exit = int(entry["expect_exit_code"])
        started = time.perf_counter()
        try:
            proc = subprocess.run(
                [*command_prefix, "check", str(entry_path)],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                timeout=DEFAULT_TIMEOUT_SECONDS,
                check=False,
            )
            actual_exit = proc.returncode
        except subprocess.TimeoutExpired:
            actual_exit = 124
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        status = "pass" if actual_exit == expected_exit else "fail"
        print(
            f"[sifr-case-timing] bucket={area} case=audit-fixtures/{entry_id} "
            f"elapsed_ms={int(elapsed_ms)} status={status}"
        )
        if actual_exit != expected_exit:
            failures.append(
                f"{entry_id}: exit={actual_exit} expected={expected_exit} path={entry['path']}"
            )
    return failures


def audit_fixture_command_prefix() -> list[str]:
    configured_bin = os.environ.get("SIFR_AUDIT_FIXTURE_BIN")
    if configured_bin:
        return [configured_bin]
    configured_target_dir = os.environ.get("CARGO_TARGET_DIR")
    if configured_target_dir:
        target_bin = Path(configured_target_dir) / "debug" / "sifr"
        if target_bin.is_file():
            return [str(target_bin)]
    if DEFAULT_SIFR_BIN.is_file():
        return [str(DEFAULT_SIFR_BIN)]
    return ["cargo", "run", "--locked", "-q", "-p", "sifr", "--"]


def emit_failures(area: str, failures: list[str]) -> None:
    print(f"{area} audit fixtures: FAIL", file=sys.stderr)
    for failure in failures:
        print(f"  - {failure}", file=sys.stderr)


def resolve_repo_path(path: str) -> Path:
    candidate = Path(path)
    resolved = candidate if candidate.is_absolute() else REPO_ROOT / candidate
    resolved = resolved.resolve()
    try:
        resolved.relative_to(REPO_ROOT)
    except ValueError as error:
        raise SystemExit(f"audit fixture path must stay under repo root: {path}") from error
    return resolved


def format_repo_relative_path(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return path.as_posix()
