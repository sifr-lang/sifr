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
REPO_ROOT = AREA_ROOT.parents[2]
DEFAULT_INVENTORY = AREA_ROOT / "emitted_rust_audit_inventory.json"
DISPOSITIONS = {"confirmed", "partially_confirmed", "rejected"}
ACTIONABLE_DISPOSITIONS = {"confirmed", "partially_confirmed"}
SEVERITIES = {"blocking", "informational"}
FINDING_ID_RE = re.compile(r"ERQ-[0-9]{3}\Z")
SHA_RE = re.compile(r"[0-9a-f]{40}\Z")
GLOB_CHARS = frozenset("*?[")


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
    elif not (REPO_ROOT / phase_file).is_file():
        errors.append("phase_file must exist")

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
    baseline_context = payload.get("baseline_context")
    if not isinstance(baseline_context, dict) or not baseline_context or any(
        not isinstance(value, str) or not value.strip()
        for value in baseline_context.values()
    ):
        errors.append("baseline_context must contain non-empty text values")

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
        else:
            for evidence_path in evidence:
                if not evidence_exists(evidence_path):
                    errors.append(
                        f"{finding_label}.evidence path does not exist: {evidence_path}"
                    )
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
            if (
                not isinstance(owner_item, int)
                or isinstance(owner_item, bool)
                or owner_item not in items
            ):
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


def evidence_exists(value: str) -> bool:
    path = Path(value)
    if path.is_absolute() or ".." in path.parts:
        return False
    if any(char in value for char in GLOB_CHARS):
        return any(REPO_ROOT.glob(value))
    return (REPO_ROOT / path).exists()


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

    invalid_schema = copy.deepcopy(payload)
    invalid_schema["schema_version"] = 2
    expect_invalid(invalid_schema, "schema_version must equal 1")

    invalid_sha = copy.deepcopy(payload)
    invalid_sha["baseline_commit"] = "not-a-sha"
    expect_invalid(invalid_sha, "baseline_commit must be a lowercase 40-character SHA")

    invalid_phase = copy.deepcopy(payload)
    invalid_phase["phase_file"] = "plans/issues/archive/closed.md"
    expect_invalid(invalid_phase, "phase_file must name an active issue record")

    missing_phase = copy.deepcopy(payload)
    missing_phase["phase_file"] = "plans/issues/active/missing.md"
    expect_invalid(missing_phase, "phase_file must exist")

    invalid_item_type = copy.deepcopy(payload)
    invalid_item_type["implementation_items"][0] = True
    expect_invalid(invalid_item_type, "implementation_items must contain integers only")

    duplicate_item = copy.deepcopy(payload)
    duplicate_item["implementation_items"][1] = 1
    expect_invalid(duplicate_item, "implementation_items must be unique")

    invalid_item_range = copy.deepcopy(payload)
    invalid_item_range["implementation_items"][-1] = 12
    expect_invalid(invalid_item_range, "implementation_items must be exactly Items 1 through 11")

    invalid_baseline = copy.deepcopy(payload)
    invalid_baseline["baseline"]["demo_emitted_rs_files"] = -1
    expect_invalid(invalid_baseline, "baseline values must be non-negative integers")

    invalid_baseline_context = copy.deepcopy(payload)
    invalid_baseline_context["baseline_context"]["rustfmt_command"] = ""
    expect_invalid(invalid_baseline_context, "baseline_context must contain non-empty text values")

    missing_findings = copy.deepcopy(payload)
    missing_findings["findings"] = []
    expect_invalid(missing_findings, "findings must be a non-empty list")

    invalid_finding_shape = copy.deepcopy(payload)
    invalid_finding_shape["findings"][0] = "not-an-object"
    expect_invalid(invalid_finding_shape, "findings[0] must be an object")

    invalid_id = copy.deepcopy(payload)
    invalid_id["findings"][0]["id"] = "bad-id"
    expect_invalid(invalid_id, "findings[0].id must match ERQ-NNN")

    missing_title = copy.deepcopy(payload)
    missing_title["findings"][0]["title"] = ""
    expect_invalid(missing_title, ".title must be non-empty text")

    invalid_evidence_shape = copy.deepcopy(payload)
    invalid_evidence_shape["findings"][0]["evidence"] = []
    expect_invalid(invalid_evidence_shape, ".evidence must contain non-empty paths")

    nonexistent_evidence = copy.deepcopy(payload)
    nonexistent_evidence["findings"][0]["evidence"] = ["missing/evidence.rs"]
    expect_invalid(nonexistent_evidence, ".evidence path does not exist")

    invalid_sources = copy.deepcopy(payload)
    invalid_sources["findings"][0]["sources"] = ["unknown"]
    expect_invalid(invalid_sources, ".sources must contain internal and/or external")

    missing_owner = copy.deepcopy(payload)
    missing_owner["findings"][0]["owner_item"] = None
    expect_invalid(missing_owner, "must name one valid implementation owner")

    invalid_owner = copy.deepcopy(payload)
    invalid_owner["findings"][0]["owner_item"] = 12
    expect_invalid(invalid_owner, "must name one valid implementation owner")

    boolean_owner = copy.deepcopy(payload)
    boolean_owner["findings"][0]["owner_item"] = True
    expect_invalid(boolean_owner, "must name one valid implementation owner")

    invalid_disposition = copy.deepcopy(payload)
    invalid_disposition["findings"][0]["disposition"] = "unknown"
    expect_invalid(invalid_disposition, "disposition is invalid")

    invalid_severity = copy.deepcopy(payload)
    invalid_severity["findings"][0]["severity"] = "unknown"
    expect_invalid(invalid_severity, "severity is invalid")

    actionable_informational = copy.deepcopy(payload)
    actionable_informational["findings"][0]["severity"] = "informational"
    expect_invalid(actionable_informational, "actionable findings must be blocking")

    unsupported_rejection = copy.deepcopy(payload)
    rejected = find_disposition(unsupported_rejection, "rejected")
    rejected.pop("rejection_reason")
    expect_invalid(unsupported_rejection, "rejected findings need a rejection_reason")

    rejected_with_owner = copy.deepcopy(payload)
    rejected = find_disposition(rejected_with_owner, "rejected")
    rejected["owner_item"] = 1
    expect_invalid(rejected_with_owner, "rejected findings cannot have an owner")

    rejected_blocking = copy.deepcopy(payload)
    rejected = find_disposition(rejected_blocking, "rejected")
    rejected["severity"] = "blocking"
    expect_invalid(rejected_blocking, "rejected findings must be informational")

    partial_without_qualification = copy.deepcopy(payload)
    partial = find_disposition(partial_without_qualification, "partially_confirmed")
    partial.pop("qualification")
    expect_invalid(partial_without_qualification, "need qualification")

    missing_item_coverage = copy.deepcopy(payload)
    missing_item_coverage["findings"] = [
        finding for finding in missing_item_coverage["findings"]
        if finding.get("owner_item") != 10
    ]
    expect_invalid(missing_item_coverage, "implementation items without an actionable finding: 10")

    print("emitted Rust audit inventory self-test: PASS")


def find_disposition(payload: dict[str, Any], disposition: str) -> dict[str, Any]:
    matches = [
        finding for finding in payload["findings"]
        if isinstance(finding, dict) and finding.get("disposition") == disposition
    ]
    if not matches:
        raise AssertionError(f"self-test fixture needs a {disposition} finding")
    return matches[0]


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
