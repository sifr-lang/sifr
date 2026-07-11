#!/usr/bin/env python3
"""Require every public sifr_stdlib callable adapter to have a live consumer."""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[1]
STDLIB_RS_ROOT = REPO_ROOT / "crates" / "sifr_stdlib" / "src"
STDLIB_SIFR_ROOT = REPO_ROOT / "stdlib"
INVENTORY_PATH = REPO_ROOT / "internal_docs" / "stdlib_native_adapter_reachability.toml"

PUBLIC_FN_RE = re.compile(
    r"(?m)^pub\s+(?:const\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"
)
RUST_TARGET_RE = re.compile(
    r"@rust(?:\.[A-Za-z_]+)?\(\s*sifr_stdlib\.([A-Za-z0-9_.]+)"
)
ALLOWED_TOP_LEVEL_FIELDS = {"schema_version", "substrate"}
ALLOWED_SUBSTRATE_FIELDS = {"adapter", "reason", "consumer_files"}


def main() -> int:
    public_adapters = _public_adapters(STDLIB_RS_ROOT)
    rust_targets = _rust_targets(STDLIB_SIFR_ROOT)
    inventory = tomllib.loads(INVENTORY_PATH.read_text(encoding="utf-8"))
    failures = _validate(
        public_adapters,
        rust_targets,
        inventory,
        lambda relative: (REPO_ROOT / relative).read_text(encoding="utf-8"),
    )
    if failures:
        print("stdlib native adapter reachability: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    substrates = inventory.get("substrate", [])
    active_targets = public_adapters & rust_targets
    print(
        "stdlib native adapter reachability: PASS "
        f"(public_adapters={len(public_adapters)}, "
        f"active_rust_targets={len(active_targets)}, "
        f"cross_module_substrates={len(substrates)})"
    )
    return 0


def _public_adapters(root: Path) -> set[str]:
    adapters: set[str] = set()
    for path in root.rglob("*.rs"):
        # Crate identity and feature-contract metadata are package API, not
        # callable generated-program adapters.
        if path.name in {"feature_contract.rs", "lib.rs"}:
            continue
        module = path.relative_to(root).parts[0].removesuffix(".rs")
        text = path.read_text(encoding="utf-8")
        adapters.update(f"{module}.{name}" for name in PUBLIC_FN_RE.findall(text))
    return adapters


def _rust_targets(root: Path) -> set[str]:
    targets: set[str] = set()
    for path in root.rglob("*.sifr"):
        targets.update(RUST_TARGET_RE.findall(path.read_text(encoding="utf-8")))
    return targets


def _validate(
    public_adapters: set[str],
    rust_targets: set[str],
    inventory: dict[str, Any],
    read_consumer: Callable[[str], str],
) -> list[str]:
    failures: list[str] = []
    unknown_top_level = sorted(set(inventory) - ALLOWED_TOP_LEVEL_FIELDS)
    if unknown_top_level:
        failures.append(f"unknown top-level fields: {', '.join(unknown_top_level)}")
    if inventory.get("schema_version") != 1:
        failures.append("schema_version must be 1")

    rows = inventory.get("substrate")
    if not isinstance(rows, list):
        return failures + ["substrate must be a list of tables"]

    substrates: set[str] = set()
    for index, row in enumerate(rows):
        context = f"substrate entry {index}"
        if not isinstance(row, dict):
            failures.append(f"{context}: must be a table")
            continue
        unknown_fields = sorted(set(row) - ALLOWED_SUBSTRATE_FIELDS)
        if unknown_fields:
            failures.append(f"{context}: unknown fields: {', '.join(unknown_fields)}")
        adapter = _required_text(failures, row, "adapter", context)
        reason = _required_text(failures, row, "reason", context)
        consumer_files = _required_string_list(
            failures, row, "consumer_files", context
        )
        if not adapter:
            continue
        if adapter in substrates:
            failures.append(f"{adapter}: duplicate substrate entry")
        substrates.add(adapter)
        if adapter in rust_targets:
            failures.append(f"{adapter}: substrate is already an active @rust target")
        reference = f"sifr_stdlib::{adapter.replace('.', '::')}"
        if reason:
            for consumer_file in consumer_files:
                try:
                    consumer_text = read_consumer(consumer_file)
                except (FileNotFoundError, IsADirectoryError):
                    failures.append(f"{adapter}: missing consumer file {consumer_file}")
                    continue
                structured_segments = all(
                    f'"{segment}".to_string()' in consumer_text
                    for segment in ("sifr_stdlib", *adapter.split("."))
                )
                if reference not in consumer_text and not structured_segments:
                    failures.append(
                        f"{adapter}: {consumer_file} does not contain {reference}"
                    )

    unreachable = sorted(public_adapters - rust_targets - substrates)
    if unreachable:
        failures.append("public adapters without a live owner: " + ", ".join(unreachable))
    stale = sorted(substrates - public_adapters)
    if stale:
        failures.append("documented substrates are not public adapters: " + ", ".join(stale))
    return failures


def _required_text(
    failures: list[str], table: dict[str, Any], key: str, context: str
) -> str:
    value = table.get(key)
    if not isinstance(value, str) or not value.strip():
        failures.append(f"{context}: {key} must be a non-empty string")
        return ""
    return value.strip()


def _required_string_list(
    failures: list[str], table: dict[str, Any], key: str, context: str
) -> list[str]:
    value = table.get(key)
    if not isinstance(value, list) or not value:
        failures.append(f"{context}: {key} must be a non-empty list")
        return []
    parsed = []
    for item in value:
        if not isinstance(item, str) or not item.strip():
            failures.append(f"{context}: {key} entries must be non-empty strings")
            continue
        parsed.append(item.strip())
    return parsed


def _self_test() -> int:
    inventory = {
        "schema_version": 1,
        "substrate": [
            {
                "adapter": "alpha.substrate",
                "reason": "compiler generated fixture",
                "consumer_files": ["consumer.rs"],
            }
        ],
    }
    consumer = lambda relative: (
        "sifr_stdlib::alpha::substrate()" if relative == "consumer.rs" else ""
    )
    public = {"alpha.active", "alpha.substrate"}
    targets = {"alpha.active"}
    if _validate(public, targets, inventory, consumer):
        print("self-test valid fixture should pass", file=sys.stderr)
        return 1

    orphan_failures = _validate(public | {"alpha.orphan"}, targets, inventory, consumer)
    if not any("alpha.orphan" in failure for failure in orphan_failures):
        print("self-test orphan public adapter was not rejected", file=sys.stderr)
        return 1

    stale_failures = _validate({"alpha.active"}, targets, inventory, consumer)
    if not any("alpha.substrate" in failure for failure in stale_failures):
        print("self-test stale substrate was not rejected", file=sys.stderr)
        return 1

    missing_consumer = lambda _relative: ""
    consumer_failures = _validate(public, targets, inventory, missing_consumer)
    if not any("does not contain" in failure for failure in consumer_failures):
        print("self-test missing substrate consumer was not rejected", file=sys.stderr)
        return 1

    print("stdlib native adapter reachability self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
