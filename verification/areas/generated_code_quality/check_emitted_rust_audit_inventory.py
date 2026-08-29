#!/usr/bin/env python3
"""Validate ownership and disposition of the emitted-Rust audit inventory."""

from __future__ import annotations

import argparse
import copy
import json
import re
import sys
from pathlib import Path
from typing import Any

AREA_ROOT = Path(__file__).resolve().parent
DEFAULT_INVENTORY = AREA_ROOT / "emitted_rust_audit_inventory.json"
DISPOSITIONS = {"confirmed", "partially_confirmed", "rejected"}
ACTIONABLE_DISPOSITIONS = {"confirmed", "partially_confirmed"}
SEVERITIES = {"blocking", "informational"}
FINDING_ID_RE = re.compile(r"ERQ-[0-9]{3}\Z")
SHA_RE = re.compile(r"[0-9a-f]{40}\Z")


def load_inventory(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read inventory {path}: {error}") from error
    if not isinstance(payload, dict):
        raise ValueError("inventory root must be an object")
    return payload


def validate_inventory(payload: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if payload.get("schema_version") != 1:
        errors.append("schema_version must equal 1")
    baseline_commit = payload.get("baseline_commit")
    if not isinstance(baseline_commit, str) or SHA_RE.fullmatch(baseline_commit) is None:
        errors.append("baseline_commit must be a lowercase 40-character SHA")
    phase_file = payload.get("phase_file")
    if not isinstance(phase_file, str) or not phase_file.startswith("plans/issues/active/"):
        errors.append("phase_file must name an active issue record")

    raw_items = payload.get("implementation_items")
    if not isinstance(raw_items, list) or not raw_items:
        errors.append("implementation_items must be a non-empty list")
        items: set[int] = set()
    else:
        if any(not isinstance(item, int) or isinstance(item, bool) for item in raw_items):
            errors.append("implementation_items must contain integers only")
        items = {item for item in raw_items if isinstance(item, int) and not isinstance(item, bool)}
        if len(items) != len(raw_items):
            errors.append("implementation_items must be unique")
        if sorted(items) != list(range(1, 12)):
            errors.append("implementation_items must be exactly Items 1 through 11")

    baseline = payload.get("baseline")
    if not isinstance(baseline, dict) or not baseline:
        errors.append("baseline must be a non-empty object")
    elif any(
        not isinstance(value, int) or isinstance(value, bool) or value < 0
        for value in baseline.values()
    ):
        errors.append("baseline values must be non-negative integers")

    raw_findings = payload.get("findings")
    if not isinstance(raw_findings, list) or not raw_findings:
        errors.append("findings must be a non-empty list")
        return errors

    seen_ids: set[str] = set()
    covered_items: set[int] = set()
    for index, raw_finding in enumerate(raw_findings):
        label = f"findings[{index}]"
        if not isinstance(raw_finding, dict):
            errors.append(f"{label} must be an object")
            continue
        finding_id = raw_finding.get("id")
        if not isinstance(finding_id, str) or FINDING_ID_RE.fullmatch(finding_id) is None:
            errors.append(f"{label}.id must match ERQ-NNN")
        elif finding_id in seen_ids:
            errors.append(f"duplicate finding id: {finding_id}")
        else:
            seen_ids.add(finding_id)
        finding_label = finding_id if isinstance(finding_id, str) else label

        for field in ("title", "mechanism"):
            value = raw_finding.get(field)
            if not isinstance(value, str) or not value.strip():
                errors.append(f"{finding_label}.{field} must be non-empty text")
        evidence = raw_finding.get("evidence")
        if not isinstance(evidence, list) or not evidence or any(
            not isinstance(value, str) or not value.strip() for value in evidence
        ):
            errors.append(f"{finding_label}.evidence must contain non-empty paths")
        sources = raw_finding.get("sources")
        if not isinstance(sources, list) or not sources or any(
            value not in {"internal", "external"} for value in sources
        ):
            errors.append(f"{finding_label}.sources must contain internal and/or external")

        disposition = raw_finding.get("disposition")
        if disposition not in DISPOSITIONS:
            errors.append(f"{finding_label}.disposition is invalid")
        severity = raw_finding.get("severity")
        if severity not in SEVERITIES:
            errors.append(f"{finding_label}.severity is invalid")
        owner_item = raw_finding.get("owner_item")
        if disposition in ACTIONABLE_DISPOSITIONS:
            if owner_item not in items:
                errors.append(f"{finding_label} must name one valid implementation owner")
            else:
                covered_items.add(owner_item)
            if severity != "blocking":
                errors.append(f"{finding_label} actionable findings must be blocking")
            if disposition == "partially_confirmed":
                qualification = raw_finding.get("qualification")
                if not isinstance(qualification, str) or not qualification.strip():
                    errors.append(f"{finding_label} partially confirmed findings need qualification")
        elif disposition == "rejected":
            if owner_item is not None:
                errors.append(f"{finding_label} rejected findings cannot have an owner")
            if severity != "informational":
                errors.append(f"{finding_label} rejected findings must be informational")
            rejection_reason = raw_finding.get("rejection_reason")
            if not isinstance(rejection_reason, str) or not rejection_reason.strip():
                errors.append(f"{finding_label} rejected findings need a rejection_reason")

    missing_owners = sorted(items - covered_items)
    if missing_owners:
        errors.append(
            "implementation items without an actionable finding: "
            + ", ".join(str(item) for item in missing_owners)
        )
    return errors


def expect_invalid(payload: dict[str, Any], expected: str) -> None:
    errors = validate_inventory(payload)
    if not any(expected in error for error in errors):
        joined = "\n".join(errors) or "<no errors>"
        raise AssertionError(f"expected validation error containing {expected!r}, got:\n{joined}")


def run_self_test(payload: dict[str, Any]) -> None:
    valid_errors = validate_inventory(payload)
    if valid_errors:
        raise AssertionError("valid inventory rejected:\n" + "\n".join(valid_errors))

    duplicate = copy.deepcopy(payload)
    duplicate["findings"][1]["id"] = duplicate["findings"][0]["id"]
    expect_invalid(duplicate, "duplicate finding id")

    missing_owner = copy.deepcopy(payload)
    missing_owner["findings"][0]["owner_item"] = None
    expect_invalid(missing_owner, "must name one valid implementation owner")

    invalid_owner = copy.deepcopy(payload)
    invalid_owner["findings"][0]["owner_item"] = 12
    expect_invalid(invalid_owner, "must name one valid implementation owner")

    invalid_disposition = copy.deepcopy(payload)
    invalid_disposition["findings"][0]["disposition"] = "unknown"
    expect_invalid(invalid_disposition, "disposition is invalid")

    unsupported_rejection = copy.deepcopy(payload)
    rejected = next(
        finding for finding in unsupported_rejection["findings"]
        if finding["disposition"] == "rejected"
    )
    rejected.pop("rejection_reason")
    expect_invalid(unsupported_rejection, "rejected findings need a rejection_reason")

    rejected_with_owner = copy.deepcopy(payload)
    rejected = next(
        finding for finding in rejected_with_owner["findings"]
        if finding["disposition"] == "rejected"
    )
    rejected["owner_item"] = 1
    expect_invalid(rejected_with_owner, "rejected findings cannot have an owner")

    partial_without_qualification = copy.deepcopy(payload)
    partial = next(
        finding for finding in partial_without_qualification["findings"]
        if finding["disposition"] == "partially_confirmed"
    )
    partial.pop("qualification")
    expect_invalid(partial_without_qualification, "need qualification")

    missing_item_coverage = copy.deepcopy(payload)
    missing_item_coverage["findings"] = [
        finding for finding in missing_item_coverage["findings"]
        if finding.get("owner_item") != 10
    ]
    expect_invalid(missing_item_coverage, "implementation items without an actionable finding: 10")

    print("emitted Rust audit inventory self-test: PASS")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, default=DEFAULT_INVENTORY)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    payload = load_inventory(args.inventory)
    if args.self_test:
        run_self_test(payload)
        return 0
    errors = validate_inventory(payload)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    actionable = sum(
        finding["disposition"] in ACTIONABLE_DISPOSITIONS
        for finding in payload["findings"]
    )
    rejected = sum(finding["disposition"] == "rejected" for finding in payload["findings"])
    print(
        "emitted Rust audit inventory: PASS "
        f"findings={len(payload['findings'])} actionable={actionable} rejected={rejected}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
