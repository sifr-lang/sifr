"""Repository-surface and checked-in emission gates."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Callable

from check_emitted_rust_audit_inventory import (
    load_inventory as load_audit_inventory,
    run_self_test as run_audit_inventory_self_test,
    validate_inventory as validate_audit_inventory,
)
from quality_policy import (
    compare_exact_debt,
    debt_selection_id,
    load_debt,
    run_debt_self_test,
    scan_files,
    validate_debt_owners,
    validate_debt_selection_ids,
    violation_summary,
)
from surface_inventory import (
    evidence as surface_inventory_evidence,
    load_and_validate as load_surface_inventory,
    run_self_test as run_surface_inventory_self_test,
)

REPO_ROOT = Path(__file__).resolve().parents[3]
GCQ_ROOT = Path(__file__).resolve().parent
SURFACE_INVENTORY = GCQ_ROOT / "data" / "surface_inventory.json"
QUALITY_DEBT = GCQ_ROOT / "data" / "generated_quality_debt.json"
AUDIT_INVENTORY = GCQ_ROOT / "emitted_rust_audit_inventory.json"
CORPUS_MANIFEST = GCQ_ROOT / "data" / "corpus_manifest.json"
SMOKE_ENTRY_IDS = ("demo-001-codegen-output", "demo-002-codegen-structural-passes")


def allowed_debt_selections() -> set[str]:
    payload = json.loads(CORPUS_MANIFEST.read_text(encoding="utf-8"))
    positive_groups = {
        "concurrency-runtime-readiness",
        "demos-required",
        "e2e-pass-representative",
        "multi-module-projects",
        "stdlib-flows",
    }
    ids = [
        entry["id"]
        for entry in payload["entries"]
        if entry.get("group") in positive_groups
    ]
    return {
        debt_selection_id(SMOKE_ENTRY_IDS),
        debt_selection_id(ids[:12]),
        debt_selection_id(ids),
    }


def gate_inventory(
    timed_case: Callable[..., Any],
    record_evidence: Callable[[str, str, list[dict[str, Any]]], Path],
    run_id: Callable[[str], str],
) -> None:
    def check_inventory() -> dict[str, Any]:
        surface = load_surface_inventory(SURFACE_INVENTORY, REPO_ROOT)
        run_surface_inventory_self_test(surface, REPO_ROOT)
        audit = load_audit_inventory(AUDIT_INVENTORY)
        audit_errors = validate_audit_inventory(audit)
        if audit_errors:
            raise RuntimeError("\n".join(audit_errors))
        run_audit_inventory_self_test(audit)
        debt = load_debt(QUALITY_DEBT)
        run_debt_self_test(debt)
        validate_debt_selection_ids(debt, allowed_debt_selections())
        snapshots = sorted((REPO_ROOT / "demos").glob("**/emitted.rs"))
        snapshot_debt = violation_summary(scan_files(snapshots, REPO_ROOT))
        compare_exact_debt(
            category="safety",
            entry_id="checked-demo-snapshots",
            actual=snapshot_debt,
            debt=debt,
        )
        return {
            **surface_inventory_evidence(surface, REPO_ROOT),
            "checked_demo_snapshot_count": len(snapshots),
            "checked_demo_snapshot_safety": snapshot_debt,
        }

    result = timed_case("generated_code_quality", "inventory/full-surface", check_inventory)
    evidence = record_evidence("inventory", run_id("inventory"), [result])
    print(f"generated-code inventory passed; evidence={evidence.relative_to(REPO_ROOT)}")


def gate_freshness(
    timed_case: Callable[..., Any],
    record_evidence: Callable[[str, str, list[dict[str, Any]]], Path],
    run_id: Callable[[str], str],
    run_command: Callable[..., Any],
    sifr_binary: Callable[[], str],
) -> None:
    def check_freshness() -> dict[str, Any] | None:
        debt = load_debt(QUALITY_DEBT)
        validate_debt_owners(debt)
        result = run_command(
            [
                sys.executable,
                str(REPO_ROOT / "scripts" / "check_demo_emitted_freshness.py"),
                "--sifr",
                sifr_binary(),
            ],
            check=False,
        )
        stale = sorted(
            line.removeprefix("stale: ")
            for line in result.stdout.splitlines()
            if line.startswith("stale: ")
        )
        orphans = sorted(
            path.relative_to(REPO_ROOT).as_posix()
            for path in (REPO_ROOT / "demos").glob("**/emitted.rs")
            if not path.with_name("main.sifr").is_file()
        )
        if result.returncode not in {0, 1}:
            raise RuntimeError(
                "demo emitted freshness command failed\n"
                f"stdout={result.stdout}\nstderr={result.stderr}"
            )
        actual = {"stale": stale, "orphans": orphans} if stale or orphans else None
        compare_exact_debt(
            category="freshness",
            entry_id="demo-companions",
            actual=actual,
            debt=debt,
        )
        return actual

    actual = timed_case("generated_code_quality", "freshness/demo-companions", check_freshness)
    evidence = record_evidence(
        "freshness",
        run_id("freshness"),
        [{"debt": actual, "status": "passed"}],
    )
    print(f"generated-code freshness passed; evidence={evidence.relative_to(REPO_ROOT)}")
