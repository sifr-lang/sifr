"""Validate declaration-first Python interop capability and design contracts."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

CAPABILITY_MATRIX = "declaration_capabilities.json"
CAPABILITY_STATES = {
    "declaration-supported",
    "bridge-supported",
    "dynamic-only",
    "unsupported-by-design",
}
IMPLEMENTATION_STATUSES = {"reserved", "active"}
EVIDENCE_KINDS = {"positive", "negative", "cleanup", "cancellation", "live"}
EVIDENCE_STATUSES = {"planned", "passing", "not-applicable"}

REQUIRED_DESIGN_FRAGMENTS = {
    "internal_docs/python_interop_declaration_architecture.md": (
        "is the only conversion type contract.",
        "structured dotted paths, never strings",
        "`python.omit`",
        "typed `*args: T`",
        "typed `**kwargs: T`",
        "explicit `**record`",
        "`SIFR-PYRES-0002`",
    ),
    "internal_docs/python_interop_protocol_architecture.md": (
        "async Python declaration owns exactly one",
        "### Shutdown State Machine",
        "## Buffer Protocol",
        "## Arrow C Data Interface",
        "## DLPack",
    ),
}
DESIGN_SWEEP_PATHS = tuple(REQUIRED_DESIGN_FRAGMENTS)

FORBIDDEN_DESIGN_PATTERNS = (
    (re.compile(r"@python(?:\.[a-z_]+)*\(\s*['\"]"), "string decorator target"),
    (
        re.compile(r"@python\.opaque\([^\n)]*\btype\s*=\s*['\"]"),
        "string opaque type target",
    ),
    (re.compile(r"@python\.opaque\([^\n)]*\bsend\s*="), "configurable opaque send policy"),
    (re.compile(r"@python[^\n]*\bconverter\s*="), "decorator converter type"),
    (
        re.compile(r"@python\.(?:buffer|arrow|dlpack)\([^\n)]*\bcopy\s*="),
        "advanced-data copy policy",
    ),
    (re.compile(r"\b(?:MVP|subset release|reduced release)\b", re.IGNORECASE), "reduced-version term"),
)


def load_and_validate_capabilities(area_root: Path, repo_root: Path) -> dict[str, Any]:
    matrix_path = area_root / CAPABILITY_MATRIX
    if not matrix_path.is_file():
        raise SystemExit(f"missing declaration capability matrix: {matrix_path}")
    try:
        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise SystemExit(f"invalid declaration capability matrix JSON: {error}") from error
    _validate_matrix(matrix)
    _validate_design_contract(repo_root)
    return matrix


def _validate_matrix(matrix: dict[str, Any]) -> None:
    if matrix.get("schema_version") != 1:
        raise SystemExit("declaration capability matrix schema_version must be 1")
    rows = matrix.get("capabilities")
    if not isinstance(rows, list) or not rows:
        raise SystemExit("declaration capability matrix must contain capabilities")

    seen: set[str] = set()
    for row in rows:
        if not isinstance(row, dict):
            raise SystemExit("declaration capability rows must be objects")
        capability_id = row.get("id")
        if not isinstance(capability_id, str) or not capability_id:
            raise SystemExit("declaration capability id is required")
        if capability_id in seen:
            raise SystemExit(f"duplicate declaration capability id: {capability_id}")
        seen.add(capability_id)

        state = row.get("target_state")
        if state not in CAPABILITY_STATES:
            raise SystemExit(f"unknown target state for {capability_id}: {state}")
        implementation = row.get("implementation_status")
        if implementation not in IMPLEMENTATION_STATUSES:
            raise SystemExit(f"unknown implementation status for {capability_id}: {implementation}")
        owner = row.get("activation_owner")
        if not isinstance(owner, str) or not re.fullmatch(r"[a-z][a-z0-9-]+", owner):
            raise SystemExit(f"invalid activation owner for {capability_id}: {owner}")

        evidence = row.get("evidence")
        if not isinstance(evidence, list) or not evidence:
            raise SystemExit(f"missing evidence ownership for {capability_id}")
        kinds: set[str] = set()
        for item in evidence:
            if not isinstance(item, dict):
                raise SystemExit(f"evidence entries for {capability_id} must be objects")
            kind = item.get("kind")
            status = item.get("status")
            evidence_owner = item.get("owner")
            if kind not in EVIDENCE_KINDS:
                raise SystemExit(f"unknown evidence kind for {capability_id}: {kind}")
            if kind in kinds:
                raise SystemExit(f"duplicate {kind} evidence for {capability_id}")
            kinds.add(kind)
            if status not in EVIDENCE_STATUSES:
                raise SystemExit(f"unknown {kind} evidence status for {capability_id}: {status}")
            if not isinstance(evidence_owner, str) or not evidence_owner:
                raise SystemExit(f"missing {kind} evidence owner for {capability_id}")
            if implementation == "reserved" and status == "passing":
                raise SystemExit(f"reserved capability {capability_id} cannot claim passing evidence")

        required_kinds = set(row.get("required_evidence", []))
        if not required_kinds or not required_kinds.issubset(EVIDENCE_KINDS):
            raise SystemExit(f"invalid required evidence set for {capability_id}")
        missing = sorted(required_kinds.difference(kinds))
        if missing:
            raise SystemExit(f"missing required evidence for {capability_id}: {', '.join(missing)}")
        for item in evidence:
            if item["kind"] in required_kinds and item["status"] == "not-applicable":
                raise SystemExit(
                    f"required {item['kind']} evidence for {capability_id} cannot be not-applicable"
                )
            if (
                implementation == "active"
                and item["kind"] in required_kinds
                and item["status"] != "passing"
            ):
                raise SystemExit(
                    f"active capability {capability_id} requires passing {item['kind']} evidence"
                )
            if item["kind"] not in required_kinds and item["status"] != "not-applicable":
                raise SystemExit(
                    f"non-required {item['kind']} evidence for {capability_id} must be not-applicable"
                )


def _validate_design_contract(repo_root: Path) -> None:
    for relative_path, fragments in REQUIRED_DESIGN_FRAGMENTS.items():
        path = repo_root / relative_path
        if not path.is_file():
            raise SystemExit(f"missing declaration-first architecture document: {relative_path}")
        text = path.read_text(encoding="utf-8")
        missing = [fragment for fragment in fragments if fragment not in text]
        if missing:
            raise SystemExit(
                f"declaration-first design contract drift in {relative_path}: missing {missing!r}"
            )
    for relative_path in DESIGN_SWEEP_PATHS:
        path = repo_root / relative_path
        if not path.is_file():
            raise SystemExit(f"missing declaration-first design input: {relative_path}")
        text = path.read_text(encoding="utf-8")
        for pattern, label in FORBIDDEN_DESIGN_PATTERNS:
            match = pattern.search(text)
            if match is not None:
                line = text.count("\n", 0, match.start()) + 1
                raise SystemExit(
                    f"stale declaration-first design in {path.relative_to(repo_root)}:{line}: {label}"
                )


def run_declaration_capability_self_tests(area_root: Path, repo_root: Path) -> None:
    matrix = load_and_validate_capabilities(area_root, repo_root)
    duplicate = json.loads(json.dumps(matrix))
    duplicate["capabilities"].append(duplicate["capabilities"][0])
    _expect_rejection(duplicate, "duplicate declaration capability id")

    unsupported_claim = json.loads(json.dumps(matrix))
    reserved_row = _make_first_row_reserved(unsupported_claim)
    required_item = next(
        item
        for item in reserved_row["evidence"]
        if item["kind"] in reserved_row["required_evidence"]
    )
    required_item["status"] = "passing"
    _expect_rejection(unsupported_claim, "cannot claim passing evidence")

    missing_cleanup = json.loads(json.dumps(matrix))
    row = next(
        candidate
        for candidate in missing_cleanup["capabilities"]
        if "cleanup" in candidate["required_evidence"]
    )
    row["evidence"] = [item for item in row["evidence"] if item["kind"] != "cleanup"]
    _expect_rejection(missing_cleanup, "missing required evidence")

    inapplicable_required = json.loads(json.dumps(matrix))
    reserved_row = _make_first_row_reserved(inapplicable_required)
    required_item = next(
        item
        for item in reserved_row["evidence"]
        if item["kind"] in reserved_row["required_evidence"]
    )
    required_item["status"] = "not-applicable"
    _expect_rejection(inapplicable_required, "cannot be not-applicable")

    incomplete_active = json.loads(json.dumps(matrix))
    active_row = next(
        row
        for row in incomplete_active["capabilities"]
        if row["implementation_status"] == "active"
    )
    required_item = next(
        item
        for item in active_row["evidence"]
        if item["kind"] in active_row["required_evidence"]
    )
    required_item["status"] = "planned"
    _expect_rejection(incomplete_active, "requires passing")


def _make_first_row_reserved(matrix: dict[str, Any]) -> dict[str, Any]:
    row = matrix["capabilities"][0]
    row["implementation_status"] = "reserved"
    required = set(row["required_evidence"])
    for item in row["evidence"]:
        if item["kind"] in required:
            item["status"] = "planned"
    return row


def _expect_rejection(matrix: dict[str, Any], expected: str) -> None:
    try:
        _validate_matrix(matrix)
    except SystemExit as error:
        if expected not in str(error):
            raise SystemExit(f"capability self-test expected {expected!r}, got {error!r}") from error
    else:
        raise SystemExit(f"capability self-test failed: accepted invalid matrix ({expected})")
