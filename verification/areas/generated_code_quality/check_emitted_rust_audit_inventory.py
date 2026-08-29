#!/usr/bin/env python3
"""Validate ownership and disposition of the emitted-Rust audit inventory."""

from __future__ import annotations

import argparse
import copy
import functools
import json
import re
import subprocess
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
REQUIRED_BASELINE_CONTEXT_KEYS = {"rustfmt_command", "rustfmt_version", "note"}


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
    elif set(baseline_context) != REQUIRED_BASELINE_CONTEXT_KEYS:
        errors.append("baseline_context must contain the exact required keys")

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
                if not evidence_exists(evidence_path, baseline_commit):
                    errors.append(
                        f"{finding_label}.evidence path does not exist: {evidence_path}"
                    )
        anchor = raw_finding.get("semantic_anchor")
        if isinstance(evidence, list):
            errors.extend(
                validate_semantic_anchor(
                    finding_label,
                    anchor,
                    evidence,
                    baseline if isinstance(baseline, dict) else {},
                    baseline_commit if isinstance(baseline_commit, str) else "",
                )
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


@functools.cache
def baseline_paths(commit: str) -> tuple[str, ...]:
    if SHA_RE.fullmatch(commit) is None:
        return ()
    result = subprocess.run(
        ["git", "ls-tree", "-r", "--name-only", commit],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    return tuple(result.stdout.splitlines()) if result.returncode == 0 else ()


@functools.cache
def evidence_exists(value: str, baseline_commit: str = "") -> bool:
    path = Path(value)
    if path.is_absolute() or ".." in path.parts or "?" in value or "[" in value:
        return False
    matches = list(REPO_ROOT.glob(value)) if "*" in value else [REPO_ROOT / path]
    repo = REPO_ROOT.resolve()
    current_exists = bool(matches) and all(
        match.exists() and match.resolve().is_relative_to(repo)
        for match in matches
    )
    if current_exists:
        return True
    historical = baseline_paths(baseline_commit)
    if "*" in value:
        return any(Path(candidate).match(value) for candidate in historical)
    return value in historical or any(candidate.startswith(f"{value}/") for candidate in historical)


def evidence_covers(anchor_path: str, evidence_paths: list[str]) -> bool:
    candidate = REPO_ROOT / anchor_path
    for evidence in evidence_paths:
        if "*" in evidence and candidate.match(evidence):
            return True
        evidence_path = REPO_ROOT / evidence
        if evidence_path.is_dir() and candidate.is_relative_to(evidence_path):
            return True
        if evidence == anchor_path:
            return True
    return False


@functools.cache
def read_anchor_text(path: str, revision: str, baseline_commit: str) -> str:
    if revision == "current":
        return (REPO_ROOT / path).read_text(encoding="utf-8")
    result = subprocess.run(
        ["git", "show", f"{baseline_commit}:{path}"],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise ValueError(f"cannot read baseline anchor {path}: {result.stderr.strip()}")
    return result.stdout


def validate_semantic_anchor(
    finding_label: str,
    anchor: object,
    evidence_paths: list[str],
    baseline: dict[str, Any],
    baseline_commit: str,
) -> list[str]:
    prefix = f"{finding_label}.semantic_anchor"
    if not isinstance(anchor, dict):
        return [f"{prefix} must be an object"]
    kind = anchor.get("kind")
    if kind == "metric":
        if set(anchor) != {"kind", "key"}:
            return [f"{prefix} metric must contain exactly kind and key"]
        key = anchor.get("key")
        if not isinstance(key, str) or key not in baseline:
            return [f"{prefix} metric must name an existing baseline key"]
        return []
    if kind == "text":
        if set(anchor) != {"kind", "revision", "path", "contains"}:
            return [f"{prefix} text must contain exactly kind, revision, path, and contains"]
        revision = anchor.get("revision")
        path = anchor.get("path")
        contains = anchor.get("contains")
        if revision not in {"baseline", "current"}:
            return [f"{prefix}.revision must be baseline or current"]
        if not isinstance(path, str) or not evidence_exists(path, baseline_commit):
            return [f"{prefix}.path must be an exact repository path"]
        if not evidence_covers(path, evidence_paths):
            return [f"{prefix}.path must be covered by finding evidence"]
        if not isinstance(contains, str) or not contains.strip():
            return [f"{prefix}.contains must be non-empty text"]
        try:
            source = read_anchor_text(path, revision, baseline_commit)
        except (OSError, UnicodeDecodeError, ValueError) as error:
            return [f"{prefix}: {error}"]
        if contains not in source:
            return [f"{prefix} text is not present in {revision} {path}"]
        return []
    if kind == "search_count":
        if set(anchor) != {"kind", "roots", "patterns", "contains", "expected_count"}:
            return [f"{prefix} search_count has invalid fields"]
        roots = anchor.get("roots")
        patterns = anchor.get("patterns")
        contains = anchor.get("contains")
        expected_count = anchor.get("expected_count")
        if (
            not isinstance(roots, list)
            or not roots
            or any(not isinstance(root, str) or not root.strip() for root in roots)
            or not isinstance(patterns, list)
            or not patterns
            or any(pattern not in {"**/*.rs", "**/*.sifr"} for pattern in patterns)
            or not isinstance(contains, str)
            or not contains.strip()
            or not isinstance(expected_count, int)
            or isinstance(expected_count, bool)
            or expected_count < 0
        ):
            return [f"{prefix} search_count fields are invalid"]
        files: set[Path] = set()
        for root in roots:
            root_path = (REPO_ROOT / root).resolve()
            if (
                Path(root).is_absolute()
                or ".." in Path(root).parts
                or not root_path.is_relative_to(REPO_ROOT.resolve())
                or not root_path.is_dir()
                or not evidence_covers(root, evidence_paths)
            ):
                return [f"{prefix} search root must be an evidence-covered repository directory"]
            for pattern in patterns:
                for path in root_path.glob(pattern):
                    if path.is_file() and path.resolve().is_relative_to(REPO_ROOT.resolve()):
                        files.add(path)
        actual_count = sum(
            contains in path.read_text(encoding="utf-8")
            for path in files
        )
        if actual_count != expected_count:
            return [
                f"{prefix} search count mismatch expected={expected_count} actual={actual_count}"
            ]
        return []
    return [f"{prefix}.kind must be metric, text, or search_count"]


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

    empty_items = copy.deepcopy(payload)
    empty_items["implementation_items"] = []
    expect_invalid(empty_items, "implementation_items must be a non-empty list")

    duplicate_item = copy.deepcopy(payload)
    duplicate_item["implementation_items"][1] = 1
    expect_invalid(duplicate_item, "implementation_items must be unique")

    invalid_item_range = copy.deepcopy(payload)
    invalid_item_range["implementation_items"][-1] = 12
    expect_invalid(invalid_item_range, "implementation_items must be exactly Items 1 through 11")

    invalid_baseline = copy.deepcopy(payload)
    invalid_baseline["baseline"]["demo_emitted_rs_files"] = -1
    expect_invalid(invalid_baseline, "baseline values must be non-negative integers")

    empty_baseline = copy.deepcopy(payload)
    empty_baseline["baseline"] = {}
    expect_invalid(empty_baseline, "baseline must be a non-empty object")

    invalid_baseline_context = copy.deepcopy(payload)
    invalid_baseline_context["baseline_context"]["rustfmt_command"] = ""
    expect_invalid(invalid_baseline_context, "baseline_context must contain non-empty text values")

    missing_baseline_context_key = copy.deepcopy(payload)
    missing_baseline_context_key["baseline_context"].pop("rustfmt_version")
    expect_invalid(missing_baseline_context_key, "baseline_context must contain the exact required keys")

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

    missing_mechanism = copy.deepcopy(payload)
    missing_mechanism["findings"][0]["mechanism"] = ""
    expect_invalid(missing_mechanism, ".mechanism must be non-empty text")

    invalid_evidence_shape = copy.deepcopy(payload)
    invalid_evidence_shape["findings"][0]["evidence"] = []
    expect_invalid(invalid_evidence_shape, ".evidence must contain non-empty paths")

    nonexistent_evidence = copy.deepcopy(payload)
    nonexistent_evidence["findings"][0]["evidence"] = ["missing/evidence.rs"]
    expect_invalid(nonexistent_evidence, ".evidence path does not exist")

    ambiguous_glob = copy.deepcopy(payload)
    ambiguous_glob["findings"][0]["evidence"] = ["demos/[abc]*/emitted.rs"]
    expect_invalid(ambiguous_glob, ".evidence path does not exist")

    missing_anchor = copy.deepcopy(payload)
    missing_anchor["findings"][0].pop("semantic_anchor")
    expect_invalid(missing_anchor, ".semantic_anchor must be an object")

    metric_extra_field = copy.deepcopy(payload)
    find_anchor_kind(metric_extra_field, "metric")["extra"] = True
    expect_invalid(metric_extra_field, "metric must contain exactly kind and key")

    invalid_metric_key = copy.deepcopy(payload)
    find_anchor_kind(invalid_metric_key, "metric")["key"] = "missing-baseline-key"
    expect_invalid(invalid_metric_key, "metric must name an existing baseline key")

    text_extra_field = copy.deepcopy(payload)
    find_anchor_kind(text_extra_field, "text")["extra"] = True
    expect_invalid(
        text_extra_field,
        "text must contain exactly kind, revision, path, and contains",
    )

    invalid_text_revision = copy.deepcopy(payload)
    find_anchor_kind(invalid_text_revision, "text")["revision"] = "unknown"
    expect_invalid(invalid_text_revision, ".semantic_anchor.revision must be baseline or current")

    stale_anchor = copy.deepcopy(payload)
    stale_anchor["findings"][0]["semantic_anchor"] = {
        "kind": "text",
        "revision": "current",
        "path": "verification/areas/generated_code_quality/runner.py",
        "contains": "definitely-not-present-anchor-text",
    }
    expect_invalid(stale_anchor, ".semantic_anchor text is not present")

    escaped_anchor = copy.deepcopy(payload)
    escaped_anchor["findings"][0]["semantic_anchor"] = {
        "kind": "text",
        "revision": "current",
        "path": "../outside",
        "contains": "outside",
    }
    expect_invalid(escaped_anchor, ".semantic_anchor.path must be an exact repository path")

    uncovered_anchor = copy.deepcopy(payload)
    uncovered_anchor["findings"][0]["semantic_anchor"] = {
        "kind": "text",
        "revision": "current",
        "path": "Cargo.toml",
        "contains": "[workspace]",
    }
    expect_invalid(uncovered_anchor, ".semantic_anchor.path must be covered by finding evidence")

    empty_text_anchor = copy.deepcopy(payload)
    find_anchor_kind(empty_text_anchor, "text")["contains"] = ""
    expect_invalid(empty_text_anchor, ".semantic_anchor.contains must be non-empty text")

    unreadable_baseline_anchor = copy.deepcopy(payload)
    unreadable_baseline_anchor["findings"][0]["evidence"] = [
        "verification/areas/generated_code_quality/inventory_gates.py"
    ]
    unreadable_baseline_anchor["findings"][0]["semantic_anchor"] = {
        "kind": "text",
        "revision": "baseline",
        "path": "verification/areas/generated_code_quality/inventory_gates.py",
        "contains": "Repository-surface and checked-in emission gates",
    }
    expect_invalid(unreadable_baseline_anchor, "cannot read baseline anchor")

    search_extra_field = copy.deepcopy(payload)
    find_anchor_kind(search_extra_field, "search_count")["extra"] = True
    expect_invalid(search_extra_field, ".semantic_anchor search_count has invalid fields")

    invalid_search_fields = copy.deepcopy(payload)
    invalid_search_fields["findings"][0]["semantic_anchor"] = {
        "kind": "search_count",
        "roots": ["verification/areas/generated_code_quality"],
        "patterns": ["**/*.py"],
        "contains": "generated",
        "expected_count": 0,
    }
    expect_invalid(invalid_search_fields, ".semantic_anchor search_count fields are invalid")

    uncovered_search_root = copy.deepcopy(payload)
    find_anchor_kind(uncovered_search_root, "search_count")["roots"] = ["stdlib/sifr"]
    expect_invalid(
        uncovered_search_root,
        ".semantic_anchor search root must be an evidence-covered repository directory",
    )

    stale_search_count = copy.deepcopy(payload)
    search_anchor = find_anchor_kind(stale_search_count, "search_count")
    search_anchor["expected_count"] += 1
    expect_invalid(stale_search_count, ".semantic_anchor search count mismatch")

    unknown_anchor_kind = copy.deepcopy(payload)
    unknown_anchor_kind["findings"][0]["semantic_anchor"] = {"kind": "unknown"}
    expect_invalid(
        unknown_anchor_kind,
        ".semantic_anchor.kind must be metric, text, or search_count",
    )

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


def find_anchor_kind(payload: dict[str, Any], kind: str) -> dict[str, Any]:
    matches = [
        finding["semantic_anchor"]
        for finding in payload["findings"]
        if isinstance(finding, dict)
        and isinstance(finding.get("semantic_anchor"), dict)
        and finding["semantic_anchor"].get("kind") == kind
    ]
    if not matches:
        raise AssertionError(f"self-test fixture needs a {kind} semantic anchor")
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
