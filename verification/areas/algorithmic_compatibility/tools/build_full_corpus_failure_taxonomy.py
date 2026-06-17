#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from datetime import date
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[4]


HEURISTIC_RULES: list[tuple[str, list[str]]] = [
    (
        "class_field_state_and_object_layout",
        ["has no field", "attribute assignment target must be a simple name"],
    ),
    (
        "destructuring_and_assignment_target_surface_gap",
        [
            "tuple unpacking target must be a simple name",
            "for loop tuple target expects iterable elements of tuple type",
            "assignment target must be a simple name",
            "augmented subscript assignment target must be a simple name",
            "cannot unpack non-tuple type 'list[",
            "chained assignment targets must be simple names",
        ],
    ),
    (
        "return_path_and_function_rules_gap",
        ["must return on all control-flow paths", "undefined variable"],
    ),
    (
        "any_unknown_typing_and_container_specialization_gap",
        [
            " type 'Any' ",
            "type 'Any'",
            "type 'Unknown'",
            " got 'Any'",
            " got 'Unknown'",
            "cannot index type 'Any'",
            "cannot index type 'Unknown'",
            "cannot iterate over type 'Any'",
            "unsupported operand type(s) for +: 'Any'",
            "'Any | None'",
            "'Unknown | None'",
            "cannot compare 'list[",
        ],
    ),
    (
        "optional_none_flow_and_narrowing_gap",
        [
            "cannot iterate over type 'list[int] | None'",
            "cannot iterate over type 'list[str] | None'",
        ],
    ),
    (
        "operator_and_truthiness_typing_gap",
        [
            "condition must be bool or collection/string truthiness",
            "bad operand type for unary not: 'int'",
            "cannot compare 'float' and 'int'",
            "cannot compare 'Never' and 'str' with ==",
        ],
    ),
    (
        "python_stdlib_and_builtin_parity_gap",
        [
            "undefined function: 'Counter'",
            "undefined function: 'defaultdict'",
            "undefined function: 'deque'",
            "undefined function: 'set'",
            "undefined variable: 'collections'",
            "undefined variable: 'heapq'",
            "undefined variable: 'math'",
            "takes exactly",
            "takes 1 or 2 arguments",
            "cannot iterate over type 'range'",
            "undefined variable: 'Iterator'",
            "'in' operator not supported for type 'range'",
            "'not in' operator not supported for type 'range'",
        ],
    ),
    (
        "nonlocal_mutable_capture_not_supported",
        ["nonlocal", "capture", "rebind", "mutable capture"],
    ),
    (
        "recursive_node_and_field_expression_surface",
        [
            "unknown type: 'TreeNode'",
            "unknown type: 'LinkedNode'",
            "attribute access '.right' is not supported as an expression",
            "attribute access '.left' is not supported as an expression",
        ],
    ),
    (
        "signature_invalid_fixture_surface",
        ["is missing a type annotation", "missing type annotation"],
    ),
    (
        "callable_argument_rules_mismatch",
        ["argument 2 of callable 'Path': expected 'int', got 'float'"],
    ),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Build full-corpus failure taxonomy artifacts from a full-corpus results JSON."
        )
    )
    parser.add_argument("--results", required=True, help="Results JSON path, repo-relative.")
    parser.add_argument(
        "--output-json", required=True, help="Output taxonomy JSON path, repo-relative."
    )
    parser.add_argument(
        "--output-md",
        default=None,
        help="Output taxonomy markdown path, repo-relative. Defaults to output-json stem + .md",
    )
    parser.add_argument(
        "--name",
        default=None,
        help="Taxonomy artifact name. Defaults to output-json filename stem.",
    )
    parser.add_argument(
        "--generated-on",
        default=date.today().isoformat(),
        help="Summary generated_on value (YYYY-MM-DD). Defaults to today.",
    )
    parser.add_argument(
        "--baseline-taxonomy",
        default=None,
        help=(
            "Prior taxonomy JSON used for fixture-level category seeding and optional delta "
            "report generation."
        ),
    )
    parser.add_argument(
        "--baseline-results",
        default=None,
        help=(
            "Prior results JSON used for status deltas. Defaults to baseline taxonomy "
            "summary.source_results."
        ),
    )
    parser.add_argument(
        "--delta-md",
        default=None,
        help="Optional markdown path for category/status deltas vs baseline taxonomy.",
    )
    return parser.parse_args()


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def normalize_repo_path(path_str: str) -> Path:
    path = Path(path_str)
    return path if path.is_absolute() else REPO_ROOT / path


def first_diagnostic(result: dict[str, Any]) -> str:
    failure_stage = result.get("failure_stage")
    stages = result.get("stages", [])
    stage = next((item for item in stages if item.get("stage") == failure_stage), None)
    if stage is None and stages:
        stage = stages[-1]
    stderr = (stage or {}).get("stderr", "")
    for line in stderr.splitlines():
        if line.strip():
            return line.strip()
    return result.get("status", "unknown failure")


def classify_category(
    fixture_slug: str,
    diagnostic: str,
    seeded_category_by_diagnostic: dict[str, tuple[str, str]],
    seeded_category_by_fixture: dict[str, tuple[str, str]],
) -> tuple[str, str]:
    seeded_diagnostic = seeded_category_by_diagnostic.get(diagnostic)
    if seeded_diagnostic is not None:
        return seeded_diagnostic

    seeded_fixture = seeded_category_by_fixture.get(fixture_slug)
    if seeded_fixture is not None:
        return seeded_fixture

    lowered = diagnostic.lower()
    for category, needles in HEURISTIC_RULES:
        if any(needle.lower() in lowered for needle in needles):
            return category, "generic"

    return "other_type_surface_and_api_mismatch", "generic"


def sort_category_counts(counter: Counter[str]) -> dict[str, int]:
    return {
        key: counter[key]
        for key in sorted(counter.keys(), key=lambda item: (-counter[item], item))
    }


def build_taxonomy(
    results_payload: dict[str, Any],
    source_results_path: str,
    artifact_name: str,
    generated_on: str,
    seeded_category_by_diagnostic: dict[str, tuple[str, str]],
    seeded_category_by_fixture: dict[str, tuple[str, str]],
) -> dict[str, Any]:
    results = results_payload["results"]
    failures: list[dict[str, str]] = []
    category_counts: Counter[str] = Counter()
    category_subcategory_counts: defaultdict[str, Counter[str]] = defaultdict(Counter)

    for result in results:
        status = result.get("status")
        if status not in {"CHECK_ERROR", "RUN_ERROR"}:
            continue
        fixture_slug = result["fixture_slug"]
        diagnostic = first_diagnostic(result)
        category, subcategory = classify_category(
            fixture_slug,
            diagnostic,
            seeded_category_by_diagnostic,
            seeded_category_by_fixture,
        )
        failures.append(
            {
                "category": category,
                "subcategory": subcategory,
                "fixture_slug": fixture_slug,
                "status": status,
                "failure_stage": result.get("failure_stage", "check"),
                "first_diagnostic": diagnostic,
            }
        )
        category_counts[category] += 1
        category_subcategory_counts[category][subcategory] += 1

    sorted_failures = sorted(failures, key=lambda item: (item["category"], item["fixture_slug"]))
    sorted_category_counts = sort_category_counts(category_counts)
    sorted_subcategory_counts = {
        category: {subcategory: count for subcategory, count in sorted(counts.items())}
        for category, counts in sorted(
            category_subcategory_counts.items(),
            key=lambda item: (-sum(item[1].values()), item[0]),
        )
    }

    return {
        "name": artifact_name,
        "summary": {
            "generated_on": generated_on,
            "source_results": source_results_path,
            "total_cases": results_payload["summary"]["case_count"],
            "failing_cases": len(sorted_failures),
            "category_count": len(sorted_category_counts),
            "category_counts": sorted_category_counts,
        },
        "category_subcategory_counts": sorted_subcategory_counts,
        "failures": sorted_failures,
    }


def taxonomy_markdown(taxonomy: dict[str, Any], title_suffix: str) -> str:
    summary = taxonomy["summary"]
    lines = [
        f"# Full Corpus Failure Taxonomy ({title_suffix})",
        "",
        f"- Source: `{summary['source_results']}`",
        f"- Total cases: `{summary['total_cases']}`",
        f"- Failing cases: `{summary['failing_cases']}`",
        "",
        "## Categories",
    ]
    for category, count in summary["category_counts"].items():
        lines.append(f"- `{category}`: `{count}`")
    lines.append("")
    return "\n".join(lines)


def read_baseline_seed_map(
    baseline_taxonomy_payload: dict[str, Any] | None,
) -> tuple[dict[str, tuple[str, str]], dict[str, tuple[str, str]]]:
    if baseline_taxonomy_payload is None:
        return {}, {}
    fixture_seed_map: dict[str, tuple[str, str]] = {}
    diagnostic_seed_map: dict[str, tuple[str, str]] = {}
    for failure in baseline_taxonomy_payload.get("failures", []):
        category_pair = (
            failure["category"],
            failure.get("subcategory", "generic"),
        )
        fixture_seed_map[failure["fixture_slug"]] = category_pair
        diagnostic_seed_map[failure["first_diagnostic"]] = category_pair
    return fixture_seed_map, diagnostic_seed_map


def status_counts(results_payload: dict[str, Any]) -> dict[str, int]:
    counts = results_payload["summary"]["status_counts"]
    return {key: counts.get(key, 0) for key in ["PASS", "CHECK_ERROR", "RUN_ERROR", "NO_ORACLE"]}


def render_delta_markdown(
    baseline_taxonomy_payload: dict[str, Any],
    current_taxonomy_payload: dict[str, Any],
    baseline_results_payload: dict[str, Any],
    current_results_payload: dict[str, Any],
    baseline_label: str,
    current_label: str,
) -> str:
    baseline_categories = baseline_taxonomy_payload["summary"]["category_counts"]
    current_categories = current_taxonomy_payload["summary"]["category_counts"]
    category_keys = sorted(set(baseline_categories) | set(current_categories))

    baseline_status = status_counts(baseline_results_payload)
    current_status = status_counts(current_results_payload)
    status_keys = ["PASS", "CHECK_ERROR", "RUN_ERROR", "NO_ORACLE"]

    lines = [
        "# Full Corpus Taxonomy Delta Report",
        "",
        f"- Baseline taxonomy: `{baseline_label}`",
        f"- Current taxonomy: `{current_label}`",
        "",
        "## Status Delta",
    ]
    for key in status_keys:
        prev = baseline_status.get(key, 0)
        curr = current_status.get(key, 0)
        lines.append(f"- `{key}`: `{prev} -> {curr}` (`{curr - prev:+d}`)")

    lines.extend(["", "## Category Delta"])
    for key in category_keys:
        prev = baseline_categories.get(key, 0)
        curr = current_categories.get(key, 0)
        lines.append(f"- `{key}`: `{prev} -> {curr}` (`{curr - prev:+d}`)")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    args = parse_args()
    results_path = normalize_repo_path(args.results)
    output_json_path = normalize_repo_path(args.output_json)
    output_md_path = (
        normalize_repo_path(args.output_md)
        if args.output_md
        else output_json_path.with_suffix(".md")
    )

    baseline_taxonomy_payload: dict[str, Any] | None = None
    baseline_results_payload: dict[str, Any] | None = None
    if args.baseline_taxonomy:
        baseline_taxonomy_path = normalize_repo_path(args.baseline_taxonomy)
        baseline_taxonomy_payload = load_json(baseline_taxonomy_path)
        baseline_results_rel = args.baseline_results or baseline_taxonomy_payload["summary"].get(
            "source_results"
        )
        if baseline_results_rel:
            baseline_results_payload = load_json(normalize_repo_path(baseline_results_rel))

    results_payload = load_json(results_path)
    fixture_seed_map, diagnostic_seed_map = read_baseline_seed_map(
        baseline_taxonomy_payload
    )
    artifact_name = args.name or output_json_path.stem
    taxonomy = build_taxonomy(
        results_payload=results_payload,
        source_results_path=args.results,
        artifact_name=artifact_name,
        generated_on=args.generated_on,
        seeded_category_by_diagnostic=diagnostic_seed_map,
        seeded_category_by_fixture=fixture_seed_map,
    )

    write_json(output_json_path, taxonomy)
    write_text(output_md_path, taxonomy_markdown(taxonomy, args.generated_on))
    print(f"wrote {output_json_path.relative_to(REPO_ROOT)}")
    print(f"wrote {output_md_path.relative_to(REPO_ROOT)}")

    if args.delta_md:
        if baseline_taxonomy_payload is None or baseline_results_payload is None:
            raise SystemExit(
                "--delta-md requires --baseline-taxonomy and baseline results resolution"
            )
        delta_md_path = normalize_repo_path(args.delta_md)
        delta_md = render_delta_markdown(
            baseline_taxonomy_payload=baseline_taxonomy_payload,
            current_taxonomy_payload=taxonomy,
            baseline_results_payload=baseline_results_payload,
            current_results_payload=results_payload,
            baseline_label=args.baseline_taxonomy,
            current_label=args.output_json,
        )
        write_text(delta_md_path, delta_md)
        print(f"wrote {delta_md_path.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
