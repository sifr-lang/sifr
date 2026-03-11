#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path
from typing import Any

from phase31_leetcode_lib import REPO_ROOT, load_json, write_json


TAXONOMY_RULES = [
    {
        "bucket_id": "codegen.mutable_binding_emission",
        "layer": "codegen",
        "title": "Codegen mutable binding emission gap",
        "match_any": ["cannot assign twice to immutable variable"],
    },
    {
        "bucket_id": "type_system.recursive_node_forward_reference",
        "layer": "type_system",
        "title": "Recursive node forward-reference resolution gap",
        "match_any": ["unknown type: 'TreeNode'", "unknown type: 'ListNode'"],
    },
    {
        "bucket_id": "lowering.attribute_expression_support",
        "layer": "lowering",
        "title": "Attribute-expression lowering gap",
        "match_any": ["attribute access '."],
    },
    {
        "bucket_id": "lowering.destructuring_target_support",
        "layer": "lowering",
        "title": "Destructuring target lowering gap",
        "match_any": [
            "tuple unpacking target must be a simple name",
            "for loop tuple target expects iterable elements of tuple type",
            "assignment target must be a simple name",
        ],
    },
    {
        "bucket_id": "lowering.unsupported_ast_shape",
        "layer": "lowering",
        "title": "Unsupported AST lowering shape",
        "match_any": ["unsupported statement type", "unsupported expression type"],
    },
    {
        "bucket_id": "stdlib.python_module_surface",
        "layer": "stdlib_runtime",
        "title": "Python stdlib/module surface gap",
        "match_any": [
            "undefined function: 'set'",
            "undefined function: 'defaultdict'",
            "undefined function: 'deque'",
            "undefined function: 'Counter'",
            "undefined variable: 'collections'",
            "undefined variable: 'heapq'",
            "undefined variable: 'math'",
        ],
    },
    {
        "bucket_id": "stdlib.python_builtin_signature_surface",
        "layer": "stdlib_runtime",
        "title": "Python builtin signature/parity gap",
        "match_any": [
            "sum() takes exactly 1 argument",
            "range() takes 1 or 2 arguments, got 3",
            "cannot iterate over type 'range'",
        ],
    },
    {
        "bucket_id": "ownership.borrowed_return_surface",
        "layer": "ownership",
        "title": "Borrowed return ownership gap",
        "match_any": ["cannot return borrowed parameter"],
    },
    {
        "bucket_id": "type_system.optional_narrowing_and_union_ops",
        "layer": "type_system",
        "title": "Optional narrowing and union-operator gap",
        "match_any": [
            "int | None",
            "str | None",
            "Any | None",
            "got 'list[int] | None'",
            "got 'list[str] | None'",
        ],
    },
    {
        "bucket_id": "frontend.nested_function_annotation_support",
        "layer": "frontend",
        "title": "Nested function annotation/inference gap",
        "match_any": ["is missing a type annotation"],
    },
    {
        "bucket_id": "frontend.untyped_any_propagation",
        "layer": "frontend",
        "title": "Untyped Any propagation gap",
        "match_any": [
            "got 'Any'",
            "type 'Any' has no method",
            "cannot index type 'Any'",
            "cannot iterate over type 'Any'",
            "unsupported operand type(s) for +: 'Any'",
            "undefined variable: 'digit'",
            "undefined variable: 'str'",
        ],
    },
]

GENERIC_CHECK_BUCKET = {
    "bucket_id": "frontend.generic_check_failure",
    "layer": "frontend",
    "title": "Generic frontend check failure",
}
GENERIC_RUN_BUCKET = {
    "bucket_id": "codegen.generic_run_failure",
    "layer": "codegen",
    "title": "Generic codegen/runtime build failure",
}

SPOT_AUDIT_CASES = [
    {"id": "0003", "expected_bucket_id": "stdlib.python_module_surface"},
    {"id": "0017", "expected_bucket_id": "frontend.nested_function_annotation_support"},
    {"id": "0052", "expected_bucket_id": "lowering.unsupported_ast_shape"},
    {"id": "0069", "expected_bucket_id": "codegen.mutable_binding_emission"},
    {"id": "0100", "expected_bucket_id": "type_system.recursive_node_forward_reference"},
    {"id": "0207", "expected_bucket_id": "lowering.destructuring_target_support"},
    {"id": "0238", "expected_bucket_id": "type_system.optional_narrowing_and_union_ops"},
    {"id": "0295", "expected_bucket_id": "lowering.destructuring_target_support"},
    {"id": "0502", "expected_bucket_id": "stdlib.python_module_surface"},
    {"id": "0746", "expected_bucket_id": "type_system.optional_narrowing_and_union_ops"},
    {"id": "0912", "expected_bucket_id": "frontend.nested_function_annotation_support"},
    {"id": "1299", "expected_bucket_id": "ownership.borrowed_return_surface"},
    {"id": "1456", "expected_bucket_id": "type_system.optional_narrowing_and_union_ops"},
    {"id": "2235", "expected_bucket_id": "stdlib.python_builtin_signature_surface"},
]


def classify_failure(result: dict[str, Any]) -> dict[str, str]:
    status = result["status"]
    stderr = "\n".join(stage["stderr"] for stage in result["stages"]).strip()
    for rule in TAXONOMY_RULES:
        if any(needle in stderr for needle in rule["match_any"]):
            return {
                "bucket_id": rule["bucket_id"],
                "layer": rule["layer"],
                "title": rule["title"],
                "matched_excerpt": next(
                    needle for needle in rule["match_any"] if needle in stderr
                ),
            }

    generic_bucket = GENERIC_RUN_BUCKET if status == "RUN_ERROR" else GENERIC_CHECK_BUCKET
    if status == "RUN_ERROR":
        run_stage_stderr = result["stages"][-1]["stderr"].strip()
        matched_excerpt = run_stage_stderr.split("\n")[0] if run_stage_stderr else status
    else:
        matched_excerpt = stderr.split("\n")[0] if stderr else status
    return {
        "bucket_id": generic_bucket["bucket_id"],
        "layer": generic_bucket["layer"],
        "title": generic_bucket["title"],
        "matched_excerpt": matched_excerpt,
    }


def load_results(results_path: Path) -> dict[str, Any]:
    return load_json(results_path)


def load_manifest_cases(manifest_path: Path) -> dict[str, dict[str, Any]]:
    payload = load_json(manifest_path)
    return {case["id"]: case for case in payload["cases"]}


def line_count(relative_path: str) -> int:
    return len((REPO_ROOT / relative_path).read_text().splitlines())


def build_taxonomy(
    results_payload: dict[str, Any], manifest_cases: dict[str, dict[str, Any]]
) -> dict[str, Any]:
    classified_results = []
    buckets: dict[str, dict[str, Any]] = {}

    for result in results_payload["results"]:
        if result["status"] not in {"CHECK_ERROR", "RUN_ERROR"}:
            continue
        classification = classify_failure(result)
        manifest_case = manifest_cases[result["id"]]
        classified_result = {
            "id": result["id"],
            "fixture_slug": result["fixture_slug"],
            "status": result["status"],
            "primary_topic": result["primary_topic"],
            "difficulty": result["difficulty"],
            "sifr_path": manifest_case["sifr_path"],
            "classification": classification,
            "stderr_excerpt": classification["matched_excerpt"],
        }
        classified_results.append(classified_result)
        bucket = buckets.setdefault(
            classification["bucket_id"],
            {
                "bucket_id": classification["bucket_id"],
                "layer": classification["layer"],
                "title": classification["title"],
                "case_count": 0,
                "statuses": {},
                "case_ids": [],
                "primary_topics": {},
            },
        )
        bucket["case_count"] += 1
        bucket["statuses"][result["status"]] = bucket["statuses"].get(result["status"], 0) + 1
        bucket["case_ids"].append(result["id"])
        bucket["primary_topics"][result["primary_topic"]] = (
            bucket["primary_topics"].get(result["primary_topic"], 0) + 1
        )

    sorted_buckets = sorted(
        buckets.values(), key=lambda bucket: (-bucket["case_count"], bucket["bucket_id"])
    )
    return {
        "summary": {
            "classified_case_count": len(classified_results),
            "bucket_count": len(sorted_buckets),
        },
        "buckets": sorted_buckets,
        "classified_results": classified_results,
    }


def build_repro_inventory(classified_payload: dict[str, Any]) -> list[dict[str, Any]]:
    by_bucket: dict[str, list[dict[str, Any]]] = {}
    for item in classified_payload["classified_results"]:
        by_bucket.setdefault(item["classification"]["bucket_id"], []).append(item)

    repros = []
    for bucket_id, items in sorted(by_bucket.items()):
        sorted_items = sorted(
            items,
            key=lambda item: (line_count(item["sifr_path"]), item["id"]),
        )
        selected = sorted_items[0]
        repros.append(
            {
                "bucket_id": bucket_id,
                "title": selected["classification"]["title"],
                "layer": selected["classification"]["layer"],
                "smallest_known_repro_case_id": selected["id"],
                "smallest_known_repro_path": selected["sifr_path"],
                "line_count": line_count(selected["sifr_path"]),
                "stderr_excerpt": selected["stderr_excerpt"],
            }
        )
    return repros


def build_spot_audit(classified_payload: dict[str, Any]) -> dict[str, Any]:
    classified_by_id = {
        item["id"]: item["classification"]["bucket_id"]
        for item in classified_payload["classified_results"]
    }
    evaluated = []
    correct = 0
    for case in SPOT_AUDIT_CASES:
        actual = classified_by_id.get(case["id"])
        matched = actual == case["expected_bucket_id"]
        if matched:
            correct += 1
        evaluated.append(
            {
                "id": case["id"],
                "expected_bucket_id": case["expected_bucket_id"],
                "actual_bucket_id": actual,
                "matched": matched,
            }
        )
    accuracy = correct / len(SPOT_AUDIT_CASES)
    return {
        "accuracy": accuracy,
        "threshold": 0.9,
        "passed": accuracy >= 0.9,
        "cases": evaluated,
    }


def build_markdown_report(
    classified_payload: dict[str, Any],
    repro_inventory: list[dict[str, Any]],
    spot_audit: dict[str, Any],
) -> str:
    lines = [
        "# Phase 31 Failure Taxonomy Report",
        "",
        f"- Classified failing seed cases: `{classified_payload['summary']['classified_case_count']}`",
        f"- Buckets: `{classified_payload['summary']['bucket_count']}`",
        f"- Spot-audit accuracy: `{spot_audit['accuracy']:.0%}`",
        "",
        "## Buckets",
        "",
    ]
    repros_by_bucket = {item["bucket_id"]: item for item in repro_inventory}
    for bucket in classified_payload["buckets"]:
        repro = repros_by_bucket[bucket["bucket_id"]]
        lines.extend(
            [
                f"### {bucket['bucket_id']}",
                f"- Layer: `{bucket['layer']}`",
                f"- Title: {bucket['title']}",
                f"- Case count: `{bucket['case_count']}`",
                f"- Statuses: `{bucket['statuses']}`",
                f"- Topics: `{bucket['primary_topics']}`",
                f"- Smallest known repro: `{repro['smallest_known_repro_case_id']}` -> `{repro['smallest_known_repro_path']}` ({repro['line_count']} lines)",
                f"- Repro stderr excerpt: `{repro['stderr_excerpt']}`",
                "",
            ]
        )

    lines.extend(
        [
            "## Spot Audit",
            "",
            f"- Passed: `{spot_audit['passed']}`",
            f"- Accuracy: `{spot_audit['accuracy']:.0%}`",
            f"- Threshold: `{spot_audit['threshold']:.0%}`",
            "",
        ]
    )
    for case in spot_audit["cases"]:
        lines.append(
            f"- `{case['id']}` expected `{case['expected_bucket_id']}` got `{case['actual_bucket_id']}` matched=`{case['matched']}`"
        )
    lines.append("")
    return "\n".join(lines)


def write_markdown(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
