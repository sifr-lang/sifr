# Ad Hoc Phase: LeetCode Incomplete And Failed Benchmark Fixes

Status: complete on 2026-05-30; merged `sifr-lang/leetcode#8` through `sifr-lang/leetcode#14`, `sifr-lang/leetcode#19`, `sifr-lang/leetcode#20`, `sifr-lang/leetcode#25`, `sifr-lang/leetcode#26`, `sifr-lang/leetcode#27`, `sifr-lang/leetcode#28`, `sifr-lang/leetcode#29`, `sifr-lang/leetcode#30`, and `sifr-lang/leetcode#31`, plus compiler support in `sifr-lang/sifr#2215`, `sifr-lang/sifr#2218`, and `sifr-lang/sifr#2220`
Context: follow-up phase for the `Incomplete And Failed Problem Appendix` in `issues/ad-hoc-leetcode-benchmark-slowness-root-cause-analysis.md`.

## Purpose

Turn every currently incomplete or failed LeetCode benchmark into a complete, correctness-passing Python/Sifr benchmark case, without hiding failures behind benchmark skips or weakening Sifr's safety model.

This phase is intentionally separate from the slowness phase. A problem that does not build, times out, or fails correctness is not performance evidence yet. The goal here is to make those problems benchmarkable first, then hand newly slower cases back to the slowness-analysis workflow.

## Source Inputs

- Failure appendix: `issues/ad-hoc-leetcode-benchmark-slowness-root-cause-analysis.md`
- Raw run logs: `audits/leetcode/benchmarks/results/.raw/*.run.log`
- Sifr sources: `audits/leetcode/src/*.sifr`
- Python oracle sources: `audits/leetcode/src/*.py`
- Benchmark registry: `audits/leetcode/benchmarks/problems/*.json`
- Generic benchmark harness: `audits/leetcode/benchmarks/harnesses/generic.py`
- Helpers: `audits/leetcode/src/helpers/list_node.sifr`, `audits/leetcode/src/helpers/tree_node.sifr`
- Proposed safe-math helper: `audits/leetcode/src/helpers/safe_math.sifr`

Inventory from the source appendix:

- **53 incomplete/failed entries**
- **52 no-complete-pair failures**
- **1 partial benchmark**: `0234_palindrome_linked_list`

The table in this phase is the authoritative working copy for all 53 rows copied from the slowness phase failure appendix. All 52 original no-complete-pair failures and the `0234_palindrome_linked_list` partial row are now complete, correctness-passing benchmark cases.

## Executive Summary

The failures split into three implementation tracks.

### Track H: Benchmark Harness / Helper Fixes

Most linked-list and tree failures are benchmark infrastructure problems, not problem-solution failures.

Primary causes:

1. **Owned list/tree results are consumed while validating expected output.**
   The generated runner calls `listNodeToString(result)` or `treeToString(result)` and then also uses `result` in the wrong-result print path or checksum path. Because the helper signatures consume owned values, the runner triggers `use of moved value`.

2. **Owned tree inputs are built once and reused across validation and benchmark loops.**
   Runners for tree problems bind `root` once, call an `own root` solution, then call the solution again inside loops with the moved value.

3. **Tree result wrong-result printing uses `str(result)` instead of `treeToString(result)`.**
   Several generated runners compile the expected check with `treeToString(result)`, but the failure print still emits `str(result)`. Generated Rust then tries to format `TreeNode` with `Display` and fails to compile.

These are generated-runner fixes in `audits/leetcode/benchmarks/harnesses/generic.py`. They should be fixed before touching the compiler or rewriting many LeetCode solutions.

### Track L: LeetCode Sifr Code Fixes

Several Sifr ports are not valid Sifr under current language rules.

Primary causes:

1. **Division and modulo return `Result[int, DivisionError]`.**
   Many ports use Python-style `/`, `//`, or `%` directly in comparisons, indexing, assignments, or returns. Sifr is correct to require explicit handling when the divisor may be zero or overflow-sensitive.

2. **Nullable APIs are too narrow.**
   Some LeetCode functions accept `TreeNode` or `ListNode` when the benchmark harness correctly models the input as `TreeNode | None` or `ListNode | None`.

3. **Empty stack/list literals need explicit element types.**
   Stack problems that start with `stack = []` and later store tuples currently infer `Any | None` on `pop()` or indexing paths.

4. **A few ports have actual correctness or parity issues.**
   `0212_word_search_ii` returns duplicates for duplicate input words. `0269_alien_dictionary` returns a valid but non-canonical topological order that does not match the Python-generated expected fixture. `0707_design_linked_list` uses recursive singly-linked-list operations while Python uses a sentinel doubly-linked list.

### Track C: Compiler / Runtime Follow-Ups

Compiler work should not be used to bypass safe error handling, but the repeated failure patterns point to useful ergonomics and precision improvements:

- better diagnostics and refinement around non-zero divisors,
- bidirectional inference for empty list literals populated by tuple appends,
- clearer ownership diagnostics for generated harness code,
- borrowed string operations so `len(s)` / `for c in s` patterns do not force unnecessary ownership in ordinary code.

These compiler improvements should come after the immediate harness and Sifr-code unblocking work, except when a compiler bug is proven by a minimal reproduction outside the benchmark harness.

## Fix Tracks

### H1: Owned Result Rendering In Generated Runners

Problems:

- `0206_reverse_linked_list`
- `0021_merge_two_sorted_lists`
- `0203_remove_linked_list_elements`
- `0083_remove_duplicates_from_sorted_list`
- `0876_middle_of_the_linked_list`
- `0019_remove_nth_node_from_end_of_list`
- `1721_swapping_nodes_in_a_linked_list`
- `0002_add_two_numbers`
- `0024_swap_nodes_in_pairs`
- `0148_sort_list`
- `0086_partition_list`
- `0061_rotate_list`
- `0147_insertion_sort_list`
- `0025_reverse_nodes_in_k_group`

Evidence:

```sifr
result: ListNode | None = reverseList(_build_list_node(_bench_tokens, 0, len(_bench_tokens)))
if listNodeToString(result) != expected_text.strip():
    print("wrong result: " + str(result))
```

Best fix:

- Change `single_sifr_runner_body` and related generated-runner paths in `audits/leetcode/benchmarks/harnesses/generic.py` so structured outputs are rendered into a local string exactly once before comparison.
- Use that same local rendered string for wrong-result printing.
- Keep checksum generation consistent with the same formatter.
- Do not fix H1 by changing helper signatures or adding clones to `ListNode` / `TreeNode`; those would hide ownership problems and affect solution semantics.

Do not fix this by rewriting all linked-list problem solutions. The failure is at the runner/helper boundary.

### H2: Owned Tree Inputs Reused Across Validation And Loops

Problems:

- `0144_binary_tree_preorder_traversal`
- `0145_binary_tree_postorder_traversal`
- `0617_merge_two_binary_trees`
- `0701_insert_into_a_binary_search_tree`
- `0450_delete_node_in_a_bst`
- `0103_binary_tree_zigzag_level_order_traversal`
- `0662_maximum_width_of_binary_tree`
- `0513_find_bottom_left_tree_value`
- `0669_trim_a_binary_search_tree`

Evidence:

```sifr
root: TreeNode | None = _build_balanced_tree(root_values, 0, len(root_values) - 1)
result: list[int] = preorderTraversal(root)
...
loop_result: list[int] = preorderTraversal(root)
```

Best fix:

- Teach the Sifr runner generator to rebuild owned `tree_node[int]` inputs for every validation and loop call, matching how list-node inputs are rebuilt from tokens.
- Alternatively, generate a fresh tree binding expression instead of a reusable `root` variable for owned tree arguments.
- Keep borrowed/non-mutating tree calls efficient only after the correctness path is fixed.

### H3: Tree Wrong-Result Formatting Uses `str(TreeNode)`

Problems:

- `0226_invert_binary_tree`
- `0108_convert_sorted_array_to_binary_search_tree`
- `0106_construct_binary_tree_from_inorder_and_postorder_traversal`
- `0105_construct_binary_tree_from_preorder_and_inorder_traversal`

Evidence:

```text
error[E0277]: `TreeNode` doesn't implement `std::fmt::Display`
println!("{}", format!("{}{}", "wrong result: ", ... format!("{}", __v)))
```

Best fix:

- Update `sifr_expected_check` / runner generation so `tree_node_int` wrong-result paths use `treeToString(result)` instead of `str(result)`.
- Apply the same rule for `list_node_int`.
- Add a generated-runner test that compiles a failing tree result path without requiring `Display` on `TreeNode`.

### L1: Explicit Division / Modulo Result Handling

Problems:

- `0853_car_fleet`
- `1209_remove_all_adjacent_duplicates_in_string_ii`
- `0441_arranging_coins`
- `0875_koko_eating_bananas`
- `0622_design_circular_queue`
- `1383_maximum_performance_of_a_team`
- `0502_ipo`
- `0698_partition_to_k_equal_sum_subsets`
- `0909_snakes_and_ladders`
- `0743_network_delay_time`
- `0062_unique_paths`
- `1220_count_vowels_permutation`
- `0846_hand_of_straights`
- `0263_ugly_number`
- `1260_shift_2d_grid`
- `0007_reverse_integer`

Representative failures:

```text
cannot compare 'Result[int, DivisionError]' and 'int' with !=
unsupported operand type(s) for +: 'int' and 'Result[int, DivisionError]'
return type mismatch: expected 'int', got 'Result[int, DivisionError]'
exact integer to float conversion requires handling possible overflow or precision loss
```

Best fix:

- Add a small LeetCode-audit helper surface for safe integer math in `audits/leetcode/src/helpers/safe_math.sifr`. This helper is for audit solutions only; it is not a compiler prelude and should not change language semantics.
- Initial helper API:
  - `def div_or_zero(a: int, b: int) -> int`
  - `def mod_or_zero(a: int, b: int) -> int`
  - `def ceil_div_positive_or_zero(a: int, b: int) -> int`
  - `def trunc_div_toward_zero_or_zero(a: int, b: int) -> int`
  - `def ratio_or_zero(a: int, b: int) -> float`
- Helper behavior:
  - if `b == 0`, return `0` as a safe fallback;
  - otherwise use Sifr `try` / `except DivisionError` locally and return `0` on the impossible error path;
  - callers must still guard invalid problem inputs when LeetCode semantics require another result such as `False`, `-1`, or early return.
- Update each Sifr solution to call these helpers only after documenting or guarding the divisor constraints.
- Preserve the compiler guarantee: do not add a compiler fallback that silently unwraps division or modulo.
- Add comments only where the divisor safety comes from LeetCode constraints and is not obvious locally.

Compiler follow-up:

- Improve non-zero divisor refinement for constants, loop counters with positive ranges, and branches like `if k <= 0: return ...`.
- Improve diagnostics to point at the divisor proof obligation and suggested local handling pattern.

### L2: Nullable Function Signatures

Problems:

- `0234_palindrome_linked_list`
- `0141_linked_list_cycle`
- `1448_count_good_nodes_in_binary_tree`
- `0230_kth_smallest_element_in_a_bst`

Representative failures:

```text
argument 1 ('head') of function 'hasCycle': expected 'ListNode', got 'None | ListNode'
argument 1 ('root') of function 'goodNodes': expected 'TreeNode', got 'None | TreeNode'
```

Best fix:

- Match LeetCode API shapes in Sifr: public linked-list/tree entrypoints should accept `ListNode | None` or `TreeNode | None` unless the registry marks the input as non-nullable.
- For problems where the fixture generator guarantees non-empty input, encode that in the registry with `nullable: false` and keep the function signature non-null only if the LeetCode API also allows that assumption.
- For `0234_palindrome_linked_list`, prefer changing `isPalindrome(own head: ListNode)` to accept `ListNode | None`; empty input should return `True` or follow the generated oracle.

### L3: Typed Stack / Tuple Collections

Problems:

- `0739_daily_temperatures`
- `0084_largest_rectangle_in_histogram`

Representative failures:

```text
cannot index type 'Any | None' with 'int'
'>' not supported between instances of 'int' and 'Any'
```

Best fix:

- Add explicit stack element types in the Sifr ports, e.g. `stack: list[tuple[int, int]] = []`.
- Handle `pop()` as optional if Sifr returns `tuple[int, int] | None`, even when guarded by `while stack`.

Compiler follow-up:

- Improve empty-list inference from later `append((...))`.
- Narrow `pop()` return type under a preceding non-empty guard when the guard and pop are in the same control-flow region.

### L4: Correctness And Fixture Semantics

Problems:

- `0212_word_search_ii`
- `0269_alien_dictionary`

Best fixes:

- `0212_word_search_ii`: return unique found words. The current Sifr code records `found[word] = True`, then appends once per input word, so duplicate words in the fixture produce duplicate output. Iterate over `found` keys or track an emitted set.
- `0269_alien_dictionary`: this is primarily a benchmark expected-shape problem. The current fixture expects the Python DFS order `cba` for repeated `abc`, while Sifr's Kahn-style implementation returns `abc`, which is also valid when there are no ordering edges. Preferred fix: add a problem-specific expected shape that validates topological-order correctness instead of exact string equality. Fallback only if problem-specific validators are rejected: intentionally port Sifr to match the Python DFS order and mark the registry as exact-order parity.

Reclassification note for `0269`: the slowness phase listed it as correctness because the generated fixture rejected Sifr's output. This phase assigns the primary fix to the benchmark harness/expected-shape layer because the fixture encodes an arbitrary valid topological order as the only accepted answer.

### L5: Stateful Object Implementation Parity / Timeout

Problem:

- `0707_design_linked_list`

Evidence:

- Raw log shows two fixture sizes pass, then the full benchmark is terminated.
- Python uses sentinel doubly-linked-list pointer updates.
- Sifr uses recursive singly-linked-list rebuilds for add/delete operations.

Best fix:

- Port the Python data-structure shape more directly, or use a vector-backed representation with equivalent semantics and predictable update cost.
- Avoid recursive whole-list reconstruction on every operation.
- After correctness passes all sizes, re-run as a performance case because this may become a slowness problem rather than merely an incomplete benchmark.

### L6: String Ownership In `0006_zigzag_conversion`

Problem:

- `0006_zigzag_conversion`

Failure:

```text
use of moved value: 's'
```

Best fix:

- Remove unnecessary ownership from the public signature: `def convert(s: str, numRows: int) -> str`.
- Keep the early return path and loop over `s` borrow-like.

Compiler follow-up:

- Verify whether `len(s)` or `for c in s` is forcing a move where a borrow should be enough. If so, reduce to a compiler test outside the benchmark harness.

## Every Failed / Incomplete Problem

| Problem | Current failure | Primary track | Best first fix |
| --- | --- | --- | --- |
| `0739_daily_temperatures` | tuple stack inferred as `Any | None` | LeetCode Sifr code | Add `list[tuple[int, int]]` stack annotation and optional-safe `pop()` handling. |
| `0853_car_fleet` | int division to float requires explicit handling | LeetCode Sifr code | Use explicit checked conversion/division helper for travel time. |
| `1209_remove_all_adjacent_duplicates_in_string_ii` | `%` returns `Result[int, DivisionError]` | LeetCode Sifr code | Handle modulo result explicitly after proving `k > 0`. |
| `0084_largest_rectangle_in_histogram` | tuple stack inferred as `Any | None` | LeetCode Sifr code | Add typed stack annotation and optional-safe pop handling. |
| `0441_arranging_coins` | `/` exact integer-to-float conversion rejected | LeetCode Sifr code | Rewrite as integer arithmetic with checked `//`, avoiding float division. |
| `0875_koko_eating_bananas` | `//` result used as int | LeetCode Sifr code | Guard `k > 0`, unwrap checked ceil-division helper. |
| `0206_reverse_linked_list` | moved `result` in runner validation | Benchmark harness | Render list-node result into one local string in the generated runner. |
| `0021_merge_two_sorted_lists` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0234_palindrome_linked_list` | partial; nullable input mismatch | LeetCode Sifr code | Accept `ListNode | None` or mark registry non-null only if fixtures guarantee it. |
| `0203_remove_linked_list_elements` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0083_remove_duplicates_from_sorted_list` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0876_middle_of_the_linked_list` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0019_remove_nth_node_from_end_of_list` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `1721_swapping_nodes_in_a_linked_list` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0002_add_two_numbers` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0141_linked_list_cycle` | public API rejects nullable head | LeetCode Sifr code | Accept `ListNode | None` and preserve no-cycle fixture semantics. |
| `0024_swap_nodes_in_pairs` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0148_sort_list` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0086_partition_list` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0061_rotate_list` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0147_insertion_sort_list` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0025_reverse_nodes_in_k_group` | moved `result` in runner validation | Benchmark harness | Same list-node result-rendering fix as H1. |
| `0707_design_linked_list` | terminated after partial success | LeetCode Sifr code | Replace recursive singly-linked-list rebuilds with pointer-parity or vector-backed object state. |
| `0622_design_circular_queue` | modulo result used as list index | LeetCode Sifr code | Explicitly handle modulo result after proving capacity is positive. |
| `0144_binary_tree_preorder_traversal` | moved `root` reused by runner | Benchmark harness | Rebuild owned tree input for each validation/loop call. |
| `0145_binary_tree_postorder_traversal` | moved `root` reused by runner | Benchmark harness | Same owned-tree rebuild fix as H2. |
| `0226_invert_binary_tree` | generated Rust tries `Display` on `TreeNode` | Benchmark harness | Use `treeToString` in wrong-result rendering. |
| `0108_convert_sorted_array_to_binary_search_tree` | generated Rust tries `Display` on `TreeNode` | Benchmark harness | Use `treeToString` in wrong-result rendering. |
| `0617_merge_two_binary_trees` | moved tree inputs reused by runner | Benchmark harness | Rebuild both owned tree inputs per call. |
| `0701_insert_into_a_binary_search_tree` | moved `root` reused by runner | Benchmark harness | Same owned-tree rebuild fix as H2. |
| `0450_delete_node_in_a_bst` | moved `root` reused by runner | Benchmark harness | Same owned-tree rebuild fix as H2. |
| `0103_binary_tree_zigzag_level_order_traversal` | moved `root` reused by runner | Benchmark harness | Same owned-tree rebuild fix as H2. |
| `0106_construct_binary_tree_from_inorder_and_postorder_traversal` | generated Rust tries `Display` on `TreeNode` | Benchmark harness | Use `treeToString` in wrong-result rendering. |
| `0662_maximum_width_of_binary_tree` | moved `root` reused by runner | Benchmark harness | Same owned-tree rebuild fix as H2. |
| `1448_count_good_nodes_in_binary_tree` | public API rejects nullable root | LeetCode Sifr code | Accept `TreeNode | None` and return `0` for empty root. |
| `0230_kth_smallest_element_in_a_bst` | public API rejects nullable root | LeetCode Sifr code | Either mark fixture non-null in registry or accept `TreeNode | None` with explicit empty handling. |
| `0105_construct_binary_tree_from_preorder_and_inorder_traversal` | generated Rust tries `Display` on `TreeNode` | Benchmark harness | Use `treeToString` in wrong-result rendering. |
| `0513_find_bottom_left_tree_value` | moved `root` reused by runner | Benchmark harness | Same owned-tree rebuild fix as H2. |
| `0669_trim_a_binary_search_tree` | moved `root` reused by runner | Benchmark harness | Same owned-tree rebuild fix as H2. |
| `0212_word_search_ii` | duplicate words in Sifr output | LeetCode Sifr code | Emit unique found words, not one output per duplicate input word. |
| `1383_maximum_performance_of_a_team` | modulo result returned as int | LeetCode Sifr code | Explicitly handle `% mod`; `mod` is positive constant. |
| `0502_ipo` | encoded heap division/mod results used as ints | LeetCode Sifr code | Use checked `//` and `%` helpers for positive base. |
| `0698_partition_to_k_equal_sum_subsets` | modulo/division results used in comparisons | LeetCode Sifr code | Guard `k > 0`, handle `%` and `//` explicitly. |
| `0909_snakes_and_ladders` | division/mod results flow into list indexes | LeetCode Sifr code | Use checked board-coordinate helper returning plain ints after bounds proof. |
| `0743_network_delay_time` | division/mod results and moved heap item | LeetCode Sifr code | Decode heap entries through checked helper and avoid reusing moved encoded values. |
| `0269_alien_dictionary` | exact expected string rejects valid alternate order | Benchmark harness | Add topological-order validity expected shape; fallback is matching Python DFS order intentionally. |
| `0062_unique_paths` | checked division result assigned to int | LeetCode Sifr code | Use exact combinatorics helper with explicit checked division. |
| `1220_count_vowels_permutation` | `% MOD` result assigned to int variables | LeetCode Sifr code | Handle modulo result for positive constant `MOD`. |
| `0846_hand_of_straights` | modulo result compared to int | LeetCode Sifr code | Handle `len(hand) % groupSize` after guarding `groupSize > 0`. |
| `0263_ugly_number` | modulo/division results used directly | LeetCode Sifr code | Handle `% p` and `// p` explicitly for constant positive divisors. |
| `1260_shift_2d_grid` | division/mod results returned as list ints | LeetCode Sifr code | Use checked index conversion helper after proving matrix dimensions positive. |
| `0006_zigzag_conversion` | moved string parameter | LeetCode Sifr code | Remove unnecessary `own` from string parameter; reduce compiler repro if still failing. |
| `0007_reverse_integer` | exact int-to-float conversion rejected | LeetCode Sifr code | Avoid float division; use integer truncation helper that models Python behavior explicitly. |

## Analyzer Schema

The machine-readable analyzer output is a superset of the slowness phase M0 output. It should keep the slowness phase fields unchanged and add failure-specific fields under a stable schema version.

Required top-level metadata:

- `schema_version`: `leetcode_failed_benchmark_inventory_v1`
- `source_raw_dir`: path to the raw result directory analyzed
- `generated_at`: ISO timestamp
- `problem_count`: total rows emitted

Required row fields:

- `problem_id`: registry problem id, for example `0206_reverse_linked_list`
- `benchmark_status`: `partial`, `failed_build`, `failed_typecheck`, `failed_correctness`, or `failed_timeout`
- `primary_track`: `benchmark_harness`, `leetcode_sifr_code`, `mixed_harness_and_code`, or `compiler_followup`
- `failure_mode`: one stable tag from the vocabulary below
- `failure_excerpt`: short excerpt from the raw log or generated runner evidence
- `first_fix`: short machine-readable summary matching the table's best first fix
- `related_slowness_phase`: boolean indicating whether this problem also has complete timing rows in the slowness phase

The table above is a human-readable rendering. The analyzer should emit snake_case identifiers, not the prose labels shown in the table.

Failure mode vocabulary:

- `moved_result_rendering`
- `moved_owned_tree_input`
- `structured_result_display`
- `division_result_unhandled`
- `float_conversion_unhandled`
- `nullable_signature_mismatch`
- `typed_stack_inference`
- `correctness_duplicate_output`
- `correctness_expected_shape`
- `timeout_stateful_object`
- `moved_string_parameter`

## Milestones

Dependency order: **M0 -> M1 -> M2 -> M3 -> M4**.

### M0: Failure Inventory Lock

- Extend `audits/leetcode/benchmarks/analyze_slowness.py`, introduced by the slowness phase M0, so it emits these 53 incomplete/failed rows deterministically as JSON.
- Validate analyzer output against the slowness phase M0 registry metadata.
- Use `schema_version: "leetcode_failed_benchmark_inventory_v1"` and the `Analyzer Schema` section above.
- Include `problem_id`, `benchmark_status`, `primary_track`, `failure_mode`, `failure_excerpt`, and `first_fix` for each row.
- Record the primary track for each row: `benchmark_harness`, `leetcode_sifr_code`, `mixed_harness_and_code`, or `compiler_followup`.
- Preserve `0234_palindrome_linked_list` as `benchmark_status: "partial"` until every configured fixture passes.

### M1: Benchmark Harness Unblockers

- Fix list/tree structured-result rendering without consuming result values.
- Rebuild owned tree inputs for each validation and benchmark loop call.
- Fix `tree_node_int` and `list_node_int` wrong-result formatting.
- Re-run the H1/H2/H3 problem subset and verify those failures disappear without changing problem solutions.

### M2: LeetCode Sifr Code Unblockers

- Add explicit safe math helper usage for division/modulo failures.
- Fix nullable public signatures for linked-list/tree APIs.
- Add typed stack annotations for tuple stack problems.
- Fix `0212`, `0269`, `0707`, and `0006` as problem-specific ports.
- Re-run the full incomplete subset after each small batch.

Completed waves:

- M0 failure inventory lock: `sifr-lang/leetcode#8` added deterministic failed/incomplete JSON inventory output.
- M1 harness unblockers: `sifr-lang/leetcode#9` fixed structured result rendering, owned tree input rebuilding, and tree/list wrong-result formatting in generated runners.
- M2a Sifr-code unblockers: `sifr-lang/leetcode#10` fixed the first Sifr-code failure batch.
- M2b safe math unblockers: `sifr-lang/leetcode#11` added explicit safe-math handling for division/modulo failure cases.
- M2c nullable tree inputs: `sifr-lang/leetcode#12` fixed nullable tree API failures.
- M2d correctness blockers: `sifr-lang/leetcode#13` fixed `0212_word_search_ii` duplicate semantics and `0269_alien_dictionary` fixture order parity.
- M2e linked-list object timeout: `sifr-lang/leetcode#14` replaced `0707_design_linked_list` recursive node rebuilding with vector-backed state and marked it complete/equivalent.
- M4a 0212 reintegration: `sifr-lang/leetcode#19` reran `0212_word_search_ii` as a complete benchmark, moved it out of failed inventory, and handed its residual `mixed` + `equivalent` trie slowness back to the slowness phase.
- M4b stateful parity reintegration: `sifr-lang/leetcode#20` reran the remaining stateful M1 rows and kept them in the slowness phase as complete `mixed` + `equivalent` rows; it did not add new failed or partial cases.
- M4c safe-math reintegration: `sifr-lang/leetcode#25` reran the safe-math formerly-failed family. Fifteen rows moved from failed-build metadata to complete/equivalent faster-than-Python benchmark rows, while `1209_remove_all_adjacent_duplicates_in_string_ii` was rewritten to stack parity and handed to the slowness phase as a complete/equivalent residual compiler row (`string_allocation`, `stack_clone`). This reduced no-pair failures from 50 to 34 and raised fully complete problems from 274 to 290.
- M4d typed-stack/string-move reintegration: `sifr-lang/leetcode#26` reran `0739_daily_temperatures`, `0084_largest_rectangle_in_histogram`, and `0006_zigzag_conversion`. All three moved from failed-build metadata to complete/equivalent faster-than-Python benchmark rows. This reduced no-pair failures from 34 to 31 and raised fully complete problems from 290 to 293.
- M4e reverse-linked-list reintegration: `sifr-lang/sifr#2215` fixed owned recursive optional field move lowering and `sifr-lang/leetcode#27` reran `0206_reverse_linked_list`. The row moved from failed-build/no-pair metadata to complete/equivalent faster-than-Python benchmark data at all sizes, reducing no-pair failures from 31 to 30 and raising fully complete problems from 293 to 294.
- M4f linked-list measured-slower reintegration: `sifr-lang/leetcode#28` reran `0002_add_two_numbers` and `0019_remove_nth_node_from_end_of_list`. Both rows moved from failed-build/no-pair metadata to complete/equivalent benchmark data and were handed to the slowness phase as residual compiler-owned list-node/optional clone cases, reducing no-pair failures from 30 to 28 and raising fully complete problems from 294 to 296.
- M4g linked-list moved-result reintegration: `sifr-lang/sifr#2218` fixed the recursive optional field partial-move follow-up by lowering moved child reads through `.take().map(...)`, and `sifr-lang/leetcode#29` reran the remaining 11 linked-list moved-result rows. `0024_swap_nodes_in_pairs` and `0147_insertion_sort_list` moved to complete/equivalent faster-than-Python benchmark data. The nine residual slower rows were handed to the slowness phase as compiler-owned list-node/optional clone cases, reducing no-pair failures from 28 to 17 and raising fully complete problems from 296 to 307.
- M4h final residual reintegration: `sifr-lang/sifr#2220` fixed recursive-node tree codegen residuals and `sifr-lang/leetcode#30` reran the final 17 no-pair residual rows. Sixteen rows moved to complete/equivalent faster-than-Python benchmark data, while `0269_alien_dictionary` became a complete/equivalent small residual noise row, reducing no-pair failures from 17 to 0 and raising fully complete problems from 307 to 324.
- M4i partial cleanup: `sifr-lang/leetcode#31` reran `0234_palindrome_linked_list` at the missing size 100 and refreshed all configured sizes. The row moved from partial to complete/equivalent measured-slower metadata, reducing partial benchmark problems from 1 to 0 and raising fully complete problems from 324 to 325.

Post-closure validation:

- The full formerly incomplete subset of 53 problem IDs passed targeted correctness after `sifr-lang/leetcode#14` was merged.
- After `sifr-lang/leetcode#31`, `python3 benchmarks/analyze_slowness.py --check-metadata` reports 0 no-pair failures, 0 partial benchmark problems, 325 fully complete problems, and 971 complete fixture pairs. No incomplete or failed benchmark rows remain.

### M3: Compiler Ergonomics Follow-Ups

- Only after M1/M2 are complete, reduce recurring patterns into compiler tests.
- Candidate compiler work:
  - non-zero divisor refinement,
  - empty-list tuple inference,
  - `pop()` optional narrowing under non-empty guards,
  - borrowed string iteration/length behavior,
  - clearer ownership diagnostics for generated runner shapes.

### M4: Reintegrate With Performance Analysis

- Use the post-fix re-benchmark protocol from `issues/ad-hoc-leetcode-benchmark-slowness-root-cause-analysis.md` as the authoritative procedure.
- Every newly complete problem must be benchmarked for runtime and memory.
- The engineer completing an M1/M2 fix owns the subset re-run and metadata update for affected problems.
- Newly slower problems should be added to the slowness phase metadata and reviewed there instead of being treated as unrelated regressions.
- Report UI should show formerly failed problems as complete only after all fixture sizes have Python/Sifr timing and memory rows.

## Acceptance Criteria

- All 53 appendix entries are listed with a primary fix track and first fix.
- Harness fixes unblock harness-owned failures without requiring LeetCode solution rewrites.
- LeetCode Sifr code fixes preserve Sifr safety semantics and do not rely on implicit division/modulo unwraps.
- Correctness fixes for `0212` and `0269` are validated against fixture semantics, not just one expected string.
- `0707_design_linked_list` completes all configured sizes and is classified with concrete residual metadata.
- Claude review approved each milestone and the phase has no unresolved incomplete/failed benchmark rows.
