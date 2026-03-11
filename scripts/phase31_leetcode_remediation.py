#!/usr/bin/env python3

from __future__ import annotations

from typing import Any

from phase31_leetcode_lib import REPO_ROOT, load_json


PRIORITY_ORDER = {"P1": 0, "P2": 1, "P3": 2}
EFFORT_ORDER = {"small": 0, "medium": 1, "large": 2}
VALID_ITEM_TYPES = {"bug", "spec_gap", "intentional_divergence"}

BACKLOG_METADATA = {
    "type_system.optional_narrowing_and_union_ops": {
        "item_id": "phase31-remediation-001",
        "item_type": "spec_gap",
        "priority": "P1",
        "effort": "large",
        "owner": "compiler/type_system",
        "dependencies": [],
        "wave_candidate": True,
        "proposed_target_phase": "phase31",
        "selection_rationale": "Largest bucket in the seed baseline with multi-topic impact and direct unblock potential.",
        "acceptance_criteria": [
            "Optional/union arithmetic, indexing, and return-value paths used by the seed corpus type-check successfully.",
            "New regression coverage locks the repaired operator and narrowing behaviors.",
        ],
    },
    "lowering.destructuring_target_support": {
        "item_id": "phase31-remediation-002",
        "item_type": "spec_gap",
        "priority": "P1",
        "effort": "medium",
        "owner": "compiler/lowering",
        "dependencies": [],
        "wave_candidate": True,
        "proposed_target_phase": "phase31",
        "selection_rationale": "High-frequency blocker across graph, heap, and string problems with low conceptual dependency depth.",
        "acceptance_criteria": [
            "Tuple/loop destructuring targets used by the corpus lower deterministically.",
            "Graph and heap corpus repros for the bucket move past the prior lowering failure.",
        ],
    },
    "frontend.nested_function_annotation_support": {
        "item_id": "phase31-remediation-003",
        "item_type": "spec_gap",
        "priority": "P1",
        "effort": "medium",
        "owner": "compiler/frontend",
        "dependencies": ["lowering.unsupported_ast_shape"],
        "wave_candidate": False,
        "proposed_target_phase": "phase32",
        "selection_rationale": "Nested helper definitions block several backtracking and divide-and-conquer fixtures but should follow AST-shape enablement.",
        "acceptance_criteria": [
            "Nested helper parameters and returns no longer require manual annotations in the covered corpus patterns.",
            "Classifier bucket count drops for nested helper-driven failures without regressing diagnostics.",
        ],
    },
    "stdlib.python_module_surface": {
        "item_id": "phase31-remediation-004",
        "item_type": "spec_gap",
        "priority": "P1",
        "effort": "medium",
        "owner": "compiler/stdlib",
        "dependencies": [],
        "wave_candidate": True,
        "proposed_target_phase": "phase31",
        "selection_rationale": "Set/defaultdict/deque/heapq parity gaps block multiple unrelated topics and are straightforward to measure.",
        "acceptance_criteria": [
            "Corpus cases using `set`, `defaultdict`, `deque`, `heapq`, and equivalent module aliases no longer fail at symbol resolution.",
            "Stdlib parity coverage is added for every newly exposed API surface.",
        ],
    },
    "type_system.recursive_node_forward_reference": {
        "item_id": "phase31-remediation-005",
        "item_type": "spec_gap",
        "priority": "P1",
        "effort": "medium",
        "owner": "compiler/type_system",
        "dependencies": [],
        "wave_candidate": False,
        "proposed_target_phase": "phase32",
        "selection_rationale": "Tree problems remain structurally blocked until recursive node references resolve predictably in signatures.",
        "acceptance_criteria": [
            "TreeNode/ListNode forward references resolve without manual reordering in covered corpus patterns.",
            "Tree-focused corpus cases advance past unknown-type failures.",
        ],
    },
    "frontend.generic_check_failure": {
        "item_id": "phase31-remediation-006",
        "item_type": "bug",
        "priority": "P2",
        "effort": "medium",
        "owner": "compiler/frontend",
        "dependencies": [],
        "wave_candidate": False,
        "proposed_target_phase": "phase32",
        "selection_rationale": "Residual generic failures need concrete decomposition after the dominant bucket fixes land.",
        "acceptance_criteria": [
            "Each case in the generic bucket is either reclassified into a specific bucket or fixed.",
            "The generic bucket reaches zero before phase closeout.",
        ],
    },
    "codegen.generic_run_failure": {
        "item_id": "phase31-remediation-007",
        "item_type": "bug",
        "priority": "P2",
        "effort": "medium",
        "owner": "compiler/codegen",
        "dependencies": [],
        "wave_candidate": True,
        "proposed_target_phase": "phase31",
        "selection_rationale": "Run-stage Rust build failures should be decomposed quickly because they already pass frontend validation.",
        "acceptance_criteria": [
            "The seed run no longer contains generic run failures; remaining run failures map to specific buckets.",
            "Rust build diagnostics for the repro case are covered by a regression test.",
        ],
    },
    "codegen.mutable_binding_emission": {
        "item_id": "phase31-remediation-008",
        "item_type": "bug",
        "priority": "P1",
        "effort": "small",
        "owner": "compiler/codegen",
        "dependencies": [],
        "wave_candidate": True,
        "proposed_target_phase": "phase31",
        "selection_rationale": "Small, isolated codegen bug with immediate end-to-end payoff and low regression radius.",
        "acceptance_criteria": [
            "Tuple-destructured mutable locals emit `mut` where reassignment is required.",
            "The `0069_sqrtx` repro runs end-to-end without Rust mutability errors.",
        ],
    },
    "lowering.attribute_expression_support": {
        "item_id": "phase31-remediation-009",
        "item_type": "spec_gap",
        "priority": "P2",
        "effort": "medium",
        "owner": "compiler/lowering",
        "dependencies": ["type_system.recursive_node_forward_reference"],
        "wave_candidate": False,
        "proposed_target_phase": "phase32",
        "selection_rationale": "Tree field access remains blocked after type resolution and should be sequenced behind recursive node support.",
        "acceptance_criteria": [
            "Attribute reads on supported class/recursive-node values lower without expression-shape rejection.",
            "Tree corpus cases advance beyond attribute-expression diagnostics.",
        ],
    },
    "lowering.unsupported_ast_shape": {
        "item_id": "phase31-remediation-010",
        "item_type": "spec_gap",
        "priority": "P2",
        "effort": "medium",
        "owner": "compiler/lowering",
        "dependencies": [],
        "wave_candidate": False,
        "proposed_target_phase": "phase32",
        "selection_rationale": "Nested function definitions and related AST shapes should be enabled before deeper nested-helper inference work.",
        "acceptance_criteria": [
            "Previously unsupported nested function statement shapes lower into HIR with stable diagnostics.",
            "The `0052_n_queens_ii` repro no longer fails with `unsupported statement type`.",
        ],
    },
    "ownership.borrowed_return_surface": {
        "item_id": "phase31-remediation-011",
        "item_type": "intentional_divergence",
        "priority": "P3",
        "effort": "small",
        "owner": "language/ownership",
        "dependencies": [],
        "wave_candidate": False,
        "proposed_target_phase": "deferred",
        "selection_rationale": "Borrow-by-default return semantics may remain intentionally explicit rather than silently cloning or changing ownership rules.",
        "acceptance_criteria": [
            "Phase documentation explicitly records the divergence and the required `own`/`.clone()` escape hatch.",
            "No remediation work starts unless product direction changes the ownership contract.",
        ],
    },
    "stdlib.python_builtin_signature_surface": {
        "item_id": "phase31-remediation-012",
        "item_type": "spec_gap",
        "priority": "P2",
        "effort": "small",
        "owner": "compiler/stdlib",
        "dependencies": [],
        "wave_candidate": False,
        "proposed_target_phase": "phase32",
        "selection_rationale": "Builtin signature parity issues are low-volume but easy to close after the higher-leverage buckets land.",
        "acceptance_criteria": [
            "Builtin signatures used by the corpus match documented Sifr semantics or are explicitly documented as divergences.",
            "The `2235_add_two_integers` repro leaves the builtin-signature bucket.",
        ],
    },
}

APPROVAL_PROCESS = {
    "owner_signoff": "yaseralnajjar",
    "reviewer_signoff": "external_phase_reviewer",
    "required_before_status": "approved",
    "policy": "Every backlog item must carry an owner, dependencies, acceptance criteria, and a target phase before it is considered approved for scheduling.",
}

STALE_BLOCKER_POLICY = {
    "threshold_days": 14,
    "applies_to_priority": "P1",
    "required_action": "reassign owner or record an explicit defer decision with rationale and target phase",
}


def load_taxonomy_buckets() -> list[dict[str, Any]]:
    payload = load_json(REPO_ROOT / "verification" / "leetcode" / "phase31_failure_taxonomy.json")
    return payload["buckets"]


def build_backlog() -> dict[str, Any]:
    buckets = load_taxonomy_buckets()
    bucket_ids = [bucket["bucket_id"] for bucket in buckets]
    missing = sorted(set(bucket_ids) - set(BACKLOG_METADATA))
    if missing:
        raise ValueError(f"missing backlog metadata for buckets: {missing}")

    entries = []
    for bucket in buckets:
        metadata = BACKLOG_METADATA[bucket["bucket_id"]]
        entries.append(
            {
                **metadata,
                "bucket_id": bucket["bucket_id"],
                "bucket_case_count": bucket["case_count"],
                "bucket_layer": bucket["layer"],
                "bucket_title": bucket["title"],
                "opened_on": "2026-03-11",
                "status": "proposed",
            }
        )

    entries.sort(
        key=lambda entry: (
            PRIORITY_ORDER[entry["priority"]],
            EFFORT_ORDER[entry["effort"]],
            -entry["bucket_case_count"],
            entry["item_id"],
        )
    )
    return {
        "phase": 31,
        "name": "phase31_remediation_backlog",
        "approval_process": APPROVAL_PROCESS,
        "stale_blocker_policy": STALE_BLOCKER_POLICY,
        "entries": entries,
    }


def build_backlog_markdown(backlog: dict[str, Any]) -> str:
    lines = [
        "# Phase 31 Remediation Backlog",
        "",
        "## Approval Process",
        "",
        f"- Owner sign-off: `{backlog['approval_process']['owner_signoff']}`",
        f"- Reviewer sign-off: `{backlog['approval_process']['reviewer_signoff']}`",
        f"- Policy: {backlog['approval_process']['policy']}",
        "",
        "## Stale Blocker Policy",
        "",
        f"- Threshold: `{backlog['stale_blocker_policy']['threshold_days']}` days",
        f"- Applies to: `{backlog['stale_blocker_policy']['applies_to_priority']}`",
        f"- Required action: {backlog['stale_blocker_policy']['required_action']}",
        "",
        "## Ranked Items",
        "",
    ]
    for entry in backlog["entries"]:
        lines.extend(
            [
                f"### {entry['item_id']}: {entry['bucket_id']}",
                f"- Type: `{entry['item_type']}`",
                f"- Priority: `{entry['priority']}`",
                f"- Effort: `{entry['effort']}`",
                f"- Owner: `{entry['owner']}`",
                f"- Bucket case count: `{entry['bucket_case_count']}`",
                f"- Dependencies: `{entry['dependencies']}`",
                f"- Wave candidate: `{entry['wave_candidate']}`",
                f"- Proposed target phase: `{entry['proposed_target_phase']}`",
                f"- Rationale: {entry['selection_rationale']}",
                "- Acceptance criteria:",
            ]
        )
        for criterion in entry["acceptance_criteria"]:
            lines.append(f"  - {criterion}")
        lines.append("")
    return "\n".join(lines)
