#!/usr/bin/env python3
"""Validate module-owned stdlib parity evidence."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
INVENTORY_PATH = REPO_ROOT / "verification/areas/stdlib_parity/data/module_parity_inventory.json"
DEFAULT_TIMEOUT_SECONDS = 15
DEFAULT_SIFR_BIN = REPO_ROOT / "target" / "debug" / "sifr"
SUPPORTED_STATUSES = {"supported", "known_gap", "zero_example_inventory"}
SUPPORTED_PROFILES = {"merge", "full", "inventory-only"}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--scope",
        choices=("inventory", "merge", "full"),
        default="inventory",
        help="Validation scope to execute.",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Exercise negative inventory mutations to prove fail-closed behavior.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    failures = run_self_test() if args.self_test else run_scope(args.scope)
    if failures:
        for failure in failures:
            print(f"stdlib module parity failure: {failure}", file=sys.stderr)
        return 1
    print(f"stdlib module parity ok: scope={args.scope}")
    return 0


def run_scope(scope: str) -> list[str]:
    failures: list[str] = []
    inventory = load_inventory(INVENTORY_PATH, failures)
    if inventory is None:
        return failures
    failures.extend(validate_inventory(inventory))
    if failures or scope == "inventory":
        return failures
    entries = selected_entries(inventory, scope)
    failures.extend(run_entries(entries))
    return failures


def run_self_test() -> list[str]:
    failures: list[str] = []
    inventory = load_inventory(INVENTORY_PATH, failures)
    if inventory is None:
        return ["self-test baseline inventory could not load"]
    if validate_inventory(inventory):
        return ["self-test baseline inventory is invalid"]

    missing_token_inventory = clone_json(inventory)
    first_api = first_supported_api(missing_token_inventory)
    first_api["tokens"] = ["__missing_stdlib_token__"]
    if not validate_inventory(missing_token_inventory):
        failures.append("self-test expected missing API token coverage to fail")

    blank_token_inventory = clone_json(inventory)
    first_api = first_supported_api(blank_token_inventory)
    first_api["tokens"] = [""]
    if not validate_inventory(blank_token_inventory):
        failures.append("self-test expected blank API token coverage to fail")

    missing_gap_inventory = clone_json(inventory)
    first_gap = first_entry_with_status(missing_gap_inventory, "known_gap")
    first_gap.pop("known_gap", None)
    if not validate_inventory(missing_gap_inventory):
        failures.append("self-test expected known-gap row without reason to fail")
    return failures


def load_inventory(path: Path, failures: list[str]) -> dict[str, Any] | None:
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        failures.append(f"{relative(path)} could not be loaded: {error}")
        return None
    if not isinstance(raw, dict):
        failures.append(f"{relative(path)} must contain a JSON object")
        return None
    return raw


def validate_inventory(inventory: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if inventory.get("schema_version") != 1:
        failures.append("module parity inventory must use schema_version 1")
    if inventory.get("area") != "stdlib_parity":
        failures.append("module parity inventory area must be stdlib_parity")
    entries = inventory.get("entries")
    if not isinstance(entries, list) or not entries:
        failures.append("module parity inventory entries must be a non-empty list")
        return failures

    ids: list[str] = []
    example_rows = 0
    zero_example_rows = 0
    for entry in entries:
        if not isinstance(entry, dict):
            failures.append("module parity entries must be objects")
            continue
        entry_id = string_field(entry, "id", failures)
        ids.append(entry_id or "")
        module = string_field(entry, "module", failures)
        string_field(entry, "owner", failures)
        status = string_field(entry, "support_status", failures)
        profile = string_field(entry, "profile", failures)
        fixture = string_field(entry, "fixture", failures)
        command = string_field(entry, "command", failures)
        if status and status not in SUPPORTED_STATUSES:
            failures.append(f"{entry_id}: unsupported support_status {status}")
        if profile and profile not in SUPPORTED_PROFILES:
            failures.append(f"{entry_id}: unsupported profile {profile}")
        if command != "check":
            failures.append(f"{entry_id}: command must be check")
        fixture_path = resolve_repo_path(fixture) if fixture else None
        if fixture_path is not None and not fixture_path.is_file():
            failures.append(f"{entry_id}: fixture does not exist: {fixture}")
        if status == "known_gap" and not isinstance(entry.get("known_gap"), str):
            failures.append(f"{entry_id}: known_gap rows must include a reason")
        apis = entry.get("supported_apis")
        if status == "zero_example_inventory":
            zero_example_rows += 1
            if apis not in ([], None):
                failures.append(f"{entry_id}: zero-example rows must not list APIs")
            continue
        if not isinstance(apis, list) or not apis:
            failures.append(f"{entry_id}: supported_apis must be a non-empty list")
            continue
        if fixture_path is None or not fixture_path.is_file():
            continue
        example_rows += 1
        source = fixture_path.read_text(encoding="utf-8")
        failures.extend(validate_api_coverage(entry_id or module or "<unknown>", apis, source))

    if ids != sorted(ids):
        failures.append("module parity inventory entries must be sorted by id")
    if len(ids) != len(set(ids)):
        failures.append("module parity inventory ids must be unique")
    if example_rows == 0 and zero_example_rows == 0:
        failures.append("module parity inventory must record examples or an explicit zero-example row")
    return failures


def validate_api_coverage(entry_id: str, apis: list[object], source: str) -> list[str]:
    failures: list[str] = []
    api_names: list[str] = []
    for raw_api in apis:
        if not isinstance(raw_api, dict):
            failures.append(f"{entry_id}: supported_apis entries must be objects")
            continue
        name = raw_api.get("name")
        tokens = raw_api.get("tokens")
        if not isinstance(name, str) or not name:
            failures.append(f"{entry_id}: API name must be a non-empty string")
            continue
        api_names.append(name)
        if not isinstance(tokens, list) or not tokens:
            failures.append(f"{entry_id}: API {name} must declare coverage tokens")
            continue
        if not all(isinstance(token, str) and token for token in tokens):
            failures.append(f"{entry_id}: API {name} coverage tokens must be non-empty strings")
            continue
        if not any(isinstance(token, str) and token in source for token in tokens):
            failures.append(f"{entry_id}: API {name} is not covered by fixture tokens")
    if len(api_names) != len(set(api_names)):
        failures.append(f"{entry_id}: API names must be unique")
    return failures


def selected_entries(inventory: dict[str, Any], scope: str) -> list[dict[str, Any]]:
    entries = [entry for entry in inventory["entries"] if entry["support_status"] == "supported"]
    if scope == "merge":
        return [entry for entry in entries if entry["profile"] == "merge"]
    return [entry for entry in entries if entry["profile"] in {"merge", "full"}]


def run_entries(entries: list[dict[str, Any]]) -> list[str]:
    failures: list[str] = []
    if not entries:
        return ["selected stdlib module parity scope must execute at least one supported row"]
    command_prefix = stdlib_module_command_prefix()
    for entry in entries:
        fixture = resolve_repo_path(entry["fixture"])
        started = time.perf_counter()
        try:
            proc = subprocess.run(
                [*command_prefix, "check", str(fixture)],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                timeout=DEFAULT_TIMEOUT_SECONDS,
                check=False,
            )
            actual_exit = proc.returncode
        except subprocess.TimeoutExpired:
            actual_exit = 124
            proc = None
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        status = "pass" if actual_exit == 0 else "fail"
        print(
            f"[sifr-case-timing] bucket=stdlib_parity case=module-parity/{entry['id']} "
            f"elapsed_ms={int(elapsed_ms)} status={status}",
            flush=True,
        )
        if actual_exit != 0:
            output = "" if proc is None else (proc.stdout + proc.stderr)[-1200:]
            failures.append(f"{entry['id']}: exit={actual_exit} fixture={entry['fixture']}\n{output}")
    return failures


def stdlib_module_command_prefix() -> list[str]:
    configured_bin = os.environ.get("SIFR_STDLIB_MODULE_BIN")
    if configured_bin:
        return [configured_bin]
    if DEFAULT_SIFR_BIN.is_file():
        return [str(DEFAULT_SIFR_BIN)]
    return ["cargo", "run", "--locked", "-q", "-p", "sifr", "--"]


def string_field(entry: dict[str, Any], field: str, failures: list[str]) -> str | None:
    value = entry.get(field)
    if isinstance(value, str) and value:
        return value
    failures.append(f"{entry.get('id', '<unknown>')}: {field} must be a non-empty string")
    return None


def first_supported_api(inventory: dict[str, Any]) -> dict[str, Any]:
    for entry in inventory["entries"]:
        if entry.get("support_status") == "supported":
            return entry["supported_apis"][0]
    raise AssertionError("inventory must contain a supported API")


def first_entry_with_status(inventory: dict[str, Any], status: str) -> dict[str, Any]:
    for entry in inventory["entries"]:
        if entry.get("support_status") == status:
            return entry
    raise AssertionError(f"inventory must contain a {status} row")


def clone_json(value: object) -> Any:
    return json.loads(json.dumps(value))


def resolve_repo_path(path: str) -> Path:
    candidate = Path(path)
    resolved = candidate if candidate.is_absolute() else REPO_ROOT / candidate
    resolved = resolved.resolve()
    try:
        resolved.relative_to(REPO_ROOT)
    except ValueError as error:
        raise SystemExit(f"stdlib module parity path must stay under repo root: {path}") from error
    return resolved


def relative(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return str(path)


if __name__ == "__main__":
    raise SystemExit(main())
