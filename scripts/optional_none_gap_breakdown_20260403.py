#!/usr/bin/env python3

from __future__ import annotations

import csv
import json
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
TAXONOMY_PATH = (
    ROOT
    / "verification/leetcode/full_corpus_failure_taxonomy_20260402_live_after_ownership_boundary_closure_reclass.json"
)
DIAGNOSTICS_PATH = (
    ROOT / "verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_rerun.json"
)
OUT_CSV = ROOT / "verification/leetcode/optional_none_gap_38_root_cause_inventory_20260403.csv"
OUT_MD = ROOT / "issues/optional-none-gap-38-root-cause-breakdown-2026-04-03.md"


def load_optional_snapshot() -> dict[str, str]:
    payload = json.loads(TAXONOMY_PATH.read_text())
    return {
        row["fixture_slug"]: row["first_diagnostic"]
        for row in payload["failures"]
        if row["category"] == "optional_none_flow_and_narrowing_gap"
    }


def load_current_diagnostics() -> dict[str, str]:
    payload = json.loads(DIAGNOSTICS_PATH.read_text())
    out: dict[str, str] = {}
    for case in payload["cases"]:
        blob = (case["stderr"] or case["stdout"]).strip()
        out[case["fixture_slug"]] = blob.splitlines()[0] if blob else ""
    return out


def load_current_status() -> dict[str, tuple[int | None, str]]:
    payload = json.loads(DIAGNOSTICS_PATH.read_text())
    out: dict[str, tuple[int | None, str]] = {}
    for case in payload["cases"]:
        out[case["fixture_slug"]] = (
            case.get("exit_code"),
            "pass" if case.get("exit_code") == 0 else "fail",
        )
    return out


def decision_map() -> dict[str, tuple[str, str, str]]:
    # (cluster, owner, rationale)
    return {
        "0002_add_two_numbers": (
            "cluster_recursive_nullable_return_contract",
            "compiler_fix",
            "return ListNode vs None|ListNode should follow declared nullable linked-list contract",
        ),
        "0046_permutations": (
            "cluster_container_element_optional_contamination",
            "compiler_fix",
            "list element type poisoned by optional index read despite dominated loop bounds",
        ),
        "0047_permutations_ii": (
            "cluster_stdlib_compat_counter_surface",
            "compiler_fix",
            "Counter compatibility signature/inference mismatch cascades into optional/Any contamination",
        ),
        "0057_insert_interval": (
            "cluster_cfg_narrowing_for_builtin_min_max",
            "compiler_fix",
            "min() sees Optional operands that should be narrowed to concrete interval endpoints",
        ),
        "0064_minimum_path_sum": (
            "cluster_cfg_narrowing_for_builtin_min_max",
            "both",
            "optional DP cell leakage plus numeric/int-vs-float contract cleanup needed",
        ),
        "0088_merge_sorted_array": (
            "cluster_container_element_optional_contamination",
            "compiler_fix",
            "subscript assignment propagates int|None into list[int] under bounded indices",
        ),
        "0103_binary_tree_zigzag_level_order_traversal": (
            "cluster_recursive_container_boundary",
            "compiler_fix",
            "tree queue/list operations leak None|TreeNode despite control-flow guards",
        ),
        "0105_construct_binary_tree_from_preorder_and_inorder_traversal": (
            "cluster_index_based_optional_leakage",
            "compiler_fix",
            "indexed tree-split arithmetic leaks int|None where index is dominated as present",
        ),
        "0106_construct_binary_tree_from_inorder_and_postorder_traversal": (
            "cluster_recursive_constructor_nullable_arg",
            "compiler_fix",
            "TreeNode constructor receives int|None because recursive split index not stabilized",
        ),
        "0108_convert_sorted_array_to_binary_search_tree": (
            "cluster_recursive_constructor_nullable_arg",
            "compiler_fix",
            "TreeNode constructor argument should be concrete under midpoint bounds",
        ),
        "0139_word_break": (
            "cluster_container_element_optional_contamination",
            "compiler_fix",
            "memo table updates retain bool|None instead of refining to bool",
        ),
        "0150_evaluate_reverse_polish_notation": (
            "cluster_parse_result_and_optional_stack",
            "both",
            "parse Result plus optional stack pop semantics require explicit adaptation and better typing flow",
        ),
        "0261_graph_valid_tree": (
            "cluster_index_based_optional_leakage",
            "compiler_fix",
            "union/find arguments leak int|None from list access despite valid edge iteration shape",
        ),
        "0287_find_the_duplicate_number": (
            "cluster_index_based_optional_leakage",
            "compiler_fix",
            "cycle-detection indices remain Optional at dominated indexing sites",
        ),
        "0304_range_sum_query_2d_immutable": (
            "cluster_none_comparison_and_matrix_indexing",
            "both",
            "None/int comparison and nested indexing indicate mixed fixture-shape adaptation plus compiler narrowing gaps",
        ),
        "0329_longest_increasing_path_in_a_matrix": (
            "cluster_recursive_argument_optional_leakage",
            "compiler_fix",
            "recursive dfs args leak int|None despite checked coordinate bounds",
        ),
        "0394_decode_string": (
            "cluster_parse_result_and_optional_stack",
            "both",
            "isdigit and arithmetic receive None/Result from stack+parse operations without explicit adaptation",
        ),
        "0417_pacific_atlantic_water_flow": (
            "cluster_recursive_argument_optional_leakage",
            "compiler_fix",
            "dfs call arguments leak int|None from bounded grid traversal indices",
        ),
        "0438_find_all_anagrams_in_a_string": (
            "cluster_string_index_optional_leakage",
            "compiler_fix",
            "char extraction under bounded window leaks str|None into dict/in checks",
        ),
        "0452_minimum_number_of_arrows_to_burst_balloons": (
            "cluster_cfg_narrowing_for_builtin_min_max",
            "compiler_fix",
            "min() optional operands should narrow from sorted interval access",
        ),
        "0567_permutation_in_string": (
            "cluster_parse_result_and_optional_stack",
            "both",
            "ord receives str|None and parse/result pollution requires explicit adaptation + better narrowing",
        ),
        "0778_swim_in_rising_water": (
            "cluster_heap_pop_optional_unpack",
            "both",
            "heap pop/unpack Optional tuple/list semantics need explicit non-empty adaptation and container typing improvements",
        ),
        "0802_find_eventual_safe_states": (
            "cluster_index_based_optional_leakage",
            "compiler_fix",
            "iteration over graph[i] leaks list[int]|None despite bounded i loop",
        ),
        "0875_koko_eating_bananas": (
            "cluster_binary_search_index_optional_arithmetic",
            "both",
            "binary-search math uses int|None intermediates; needs narrowing + canonical adaptation cleanup",
        ),
        "0881_boats_to_save_people": (
            "cluster_two_pointer_index_optional_arithmetic",
            "compiler_fix",
            "two-pointer bounded indices still typed as Optional in arithmetic",
        ),
        "0904_fruit_into_baskets": (
            "cluster_container_element_optional_contamination",
            "compiler_fix",
            "dict key receives int|None from bounded array index access",
        ),
        "0948_bag_of_tokens": (
            "cluster_two_pointer_index_optional_arithmetic",
            "compiler_fix",
            "two-pointer score math polluted by Optional index value under guarded bounds",
        ),
        "0977_squares_of_a_sorted_array": (
            "cluster_two_pointer_index_optional_arithmetic",
            "both",
            "abs receives int|None; plus fixture local-name hygiene needed in canonical form",
        ),
        "1203_sort_items_by_groups_respecting_dependencies": (
            "cluster_stdlib_compat_deque_surface",
            "both",
            "deque compat expects list but gets iterator and optional/index contamination follows",
        ),
        "1397_find_all_good_strings": (
            "cluster_string_index_optional_leakage",
            "both",
            "string indexing with int|None plus helper-shape adaptation required for deterministic typing",
        ),
        "1423_maximum_points_you_can_obtain_from_cards": (
            "cluster_index_based_optional_leakage",
            "compiler_fix",
            "window arithmetic leaks Optional from bounded card index access",
        ),
        "1498_number_of_subsequences_that_satisfy_the_given_sum_condition": (
            "cluster_two_pointer_index_optional_arithmetic",
            "compiler_fix",
            "two-pointer arithmetic sees Optional where bounds imply concrete ints",
        ),
        "1584_min_cost_to_connect_all_points": (
            "cluster_heap_pop_optional_unpack",
            "both",
            "heap tuple unpack receives Optional; requires explicit non-empty adaptation and comparable typing closure",
        ),
        "1631_path_with_minimum_effort": (
            "cluster_heap_pop_optional_unpack",
            "both",
            "priority-queue pop/unpack Optional tuple leakage plus return-type stabilization needed",
        ),
        "1700_number_of_students_unable_to_eat_lunch": (
            "cluster_container_element_optional_contamination",
            "compiler_fix",
            "queue/list append path leaks int|None into list[int]",
        ),
        "1838_frequency_of_the_most_frequent_element": (
            "cluster_binary_search_index_optional_arithmetic",
            "compiler_fix",
            "window arithmetic leaks Optional from bounded sorted array indices",
        ),
        "1980_find_unique_binary_string": (
            "cluster_optional_sentinel_return_shape",
            "sifr_adaptation",
            "Python-style None sentinel branch in str-return helper should be rewritten to explicit Option flow",
        ),
        "2300_successful_pairs_of_spells_and_potions": (
            "cluster_binary_search_index_optional_arithmetic",
            "compiler_fix",
            "binary-search multiplication uses int|None from bounded potions index",
        ),
    }


def main() -> None:
    snapshot = load_optional_snapshot()
    current = load_current_diagnostics()
    current_status = load_current_status()
    mapping = decision_map()

    missing = sorted(set(snapshot) - set(mapping))
    extra = sorted(set(mapping) - set(snapshot))
    if missing or extra:
        raise SystemExit(f"mapping mismatch missing={missing} extra={extra}")

    rows = []
    for slug in sorted(snapshot):
        cluster, owner, rationale = mapping[slug]
        current_line = current.get(slug, "")
        rows.append(
            {
                "fixture_slug": slug,
                "snapshot_first_diagnostic": snapshot[slug],
                "current_check_first_line_20260403": current_line,
                "current_check_exit_code_20260403_rerun": current_status[slug][0],
                "current_check_status_20260403_rerun": current_status[slug][1],
                "still_failing_on_current_check": "yes"
                if current_status[slug][1] == "fail"
                else "no",
                "root_cause_cluster": cluster,
                "resolution_owner": owner,
                "rationale": rationale,
            }
        )

    OUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    with OUT_CSV.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)

    owner_counts = Counter(row["resolution_owner"] for row in rows)
    cluster_counts = Counter(row["root_cause_cluster"] for row in rows)
    still_counts = Counter(row["still_failing_on_current_check"] for row in rows)
    failing_rows = [row for row in rows if row["current_check_status_20260403_rerun"] == "fail"]
    failing_cluster_counts = Counter(row["root_cause_cluster"] for row in failing_rows)
    owner_cluster: dict[str, Counter[str]] = defaultdict(Counter)
    for row in rows:
        owner_cluster[row["resolution_owner"]][row["root_cause_cluster"]] += 1

    lines: list[str] = []
    lines.append("# Optional/None Flow and Narrowing Gap (38) Root Cause Breakdown")
    lines.append("")
    lines.append(
        "- Snapshot analyzed: "
        "`verification/leetcode/full_corpus_current_results_20260402_live_after_ownership_boundary_closure.json`"
    )
    lines.append(
        "- Category source: "
        "`verification/leetcode/full_corpus_failure_taxonomy_20260402_live_after_ownership_boundary_closure_reclass.json`"
    )
    lines.append(
        "- Deep diagnostics rerun: "
        "`verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_rerun.json`"
    )
    lines.append(
        "- Inventory: "
        "`verification/leetcode/optional_none_gap_38_root_cause_inventory_20260403.csv`"
    )
    lines.append("")
    lines.append("## Decision Summary")
    for key in ("compiler_fix", "both", "sifr_adaptation"):
        lines.append(f"- `{key}`: `{owner_counts[key]}`")
    lines.append("")
    lines.append("## Current Drift Check (2026-04-03 check-only)")
    lines.append(f"- Still failing: `{still_counts['yes']}`")
    lines.append(f"- Now passing at check stage: `{still_counts['no']}`")
    warning_passes = [
        row
        for row in rows
        if row["current_check_status_20260403_rerun"] == "pass"
        and row["current_check_first_line_20260403"] != "no errors found"
    ]
    lines.append(f"- Passes with warnings: `{len(warning_passes)}`")
    lines.append("")
    lines.append("## Largest Unresolved Subcategories (Current 27 Failures)")
    for cluster, count in sorted(
        failing_cluster_counts.items(), key=lambda item: (-item[1], item[0])
    ):
        fixtures = ", ".join(
            row["fixture_slug"]
            for row in rows
            if row["current_check_status_20260403_rerun"] == "fail"
            and row["root_cause_cluster"] == cluster
        )
        lines.append(f"- `{cluster}`: `{count}` -> {fixtures}")
    lines.append("")
    lines.append("## Language Adjustment Decision")
    lines.append("- Keep core language principles unchanged (static safety, explicit Option/Result, no panic paths).")
    lines.append("- `compiler_fix` items should be solved by stronger control-flow narrowing and dominated-index/container element refinement, not by weakening typing rules.")
    lines.append("- `sifr_adaptation` item (`1980`) should be rewritten to explicit Option flow; this is canonical Sifr style, not a compiler loophole request.")
    lines.append("- `both` items require both compiler precision and canonical fixture adaptation (stdlib surface/undefined-name cleanup) while keeping strict typing intact.")
    lines.append("")
    lines.append("## Root Cause Clusters")
    for cluster, count in sorted(cluster_counts.items(), key=lambda item: (-item[1], item[0])):
        lines.append(f"- `{cluster}`: `{count}`")
    lines.append("")
    lines.append("## Ownership x Cluster")
    for owner in ("compiler_fix", "both", "sifr_adaptation"):
        lines.append(f"- `{owner}`")
        for cluster, count in sorted(
            owner_cluster[owner].items(), key=lambda item: (-item[1], item[0])
        ):
            lines.append(f"`{cluster}`: `{count}`")
    lines.append("")
    lines.append("## Reviewer Validation")
    lines.append("- Claude review pass 1: `reviews/optional-none-gap-38-root-cause-breakdown-review-pass1.md`")
    lines.append("- Claude review pass 2: `reviews/optional-none-gap-38-root-cause-breakdown-review-pass2.md` (contained a disputed owner-count claim).")
    lines.append("- Claude review pass 3 reconciliation: `reviews/optional-none-gap-38-root-cause-breakdown-review-pass3.md`.")
    lines.append("- Final reviewer-backed verdict: owner split is `compiler_fix=25`, `both=12`, `sifr_adaptation=1`.")
    lines.append("")
    lines.append("## Per-Case Decisions")
    for row in rows:
        lines.append(
            f"- `{row['fixture_slug']}` | owner=`{row['resolution_owner']}` | "
            f"cluster=`{row['root_cause_cluster']}`"
        )
        lines.append(
            f"current status: {row['current_check_status_20260403_rerun']} "
            f"(exit={row['current_check_exit_code_20260403_rerun']})"
        )
        lines.append(f"snapshot: {row['snapshot_first_diagnostic']}")
        lines.append(f"current check: {row['current_check_first_line_20260403']}")
        lines.append(f"rationale: {row['rationale']}")

    OUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUT_MD.write_text("\n".join(lines) + "\n")

    print(f"wrote {OUT_CSV.relative_to(ROOT)}")
    print(f"wrote {OUT_MD.relative_to(ROOT)}")
    print(f"owner_counts {dict(owner_counts)}")
    print(f"still_counts {dict(still_counts)}")


if __name__ == "__main__":
    main()
