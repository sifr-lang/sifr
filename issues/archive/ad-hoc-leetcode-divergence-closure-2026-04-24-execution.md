# Ad-hoc Phase Execution: LeetCode Divergence Closure

Status: closed
Started: 2026-04-24
Phase plan: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`

## Wave Checklist

- [x] WS0 corpus normalization and baseline refresh
- [x] WS1 narrowing design and first compiler slices
- [x] WS2 heap / DSU / collection stdlib parity
- [x] WS3 owned-chain helper convention and cursor slices
- [x] WS4 canonical rewrite debt
- [x] WS5 architecture boundary documentation
- [x] WS6 final rerun, scorecard, and closure review

## WS0 Corpus Normalization And Baseline Refresh

Status: merged
Branch: `ws0-leetcode-corpus-noise-normalization`
PR: `https://github.com/sifr-lang/sifr/pull/1609`
Merged: `https://github.com/sifr-lang/sifr/pull/1609`

### Scope

Normalized Python-side corpus noise for:

- `audits/leetcode/0104_maximum_depth_of_binary_tree.py`
- `audits/leetcode/0130_surrounded_regions.py`
- `audits/leetcode/0200_number_of_islands.py`
- `audits/leetcode/0516_longest_palindromic_subsequence.py`

### Changes

- `0104`: removed unused generic `Node` helper, unused string helper, unused `deque` import, and duplicate iterative/BFS definitions; kept the recursive canonical implementation matching the Sifr fixture.
- `0130`: removed the quoted alternate implementation block; kept the existing set-backed border DFS implementation.
- `0200`: removed unused `deque` import and duplicate mutating DFS/BFS definitions; kept the visited-set DFS implementation matching the Sifr fixture.
- `0516`: removed unreachable and duplicate DP/LCS implementations; kept one memoized LCS-over-reversed-string implementation aligned with the Sifr fixture and preserving the sentinel `.get((i, j), -1)` pressure tracked by WS1.

### Pair Scan Movement

Previous stats from the pre-edit scan:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0104_maximum_depth_of_binary_tree` | 82 | 74 | 8 | 93/27 |
| `0130_surrounded_regions` | 81 | 55 | 26 | 71/42 |
| `0200_number_of_islands` | 103 | 82 | 21 | 95/34 |
| `0516_longest_palindromic_subsequence` | 83 | 61 | 22 | 72/33 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0104_maximum_depth_of_binary_tree` | 13 | 7 | 6 | 28/27 |
| `0130_surrounded_regions` | 46 | 20 | 26 | 36/42 |
| `0200_number_of_islands` | 46 | 25 | 21 | 38/34 |
| `0516_longest_palindromic_subsequence` | 20 | 10 | 10 | 33/33 |

### Validation

Targeted Python fixture checks:

- `python3 audits/leetcode/0104_maximum_depth_of_binary_tree.py`
- `python3 audits/leetcode/0130_surrounded_regions.py`
- `python3 audits/leetcode/0200_number_of_islands.py`
- `python3 audits/leetcode/0516_longest_palindromic_subsequence.py`

Targeted Sifr fixture checks:

- `cargo run -q -p sifr -- run audits/leetcode/0104_maximum_depth_of_binary_tree.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0130_surrounded_regions.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0200_number_of_islands.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0516_longest_palindromic_subsequence.sifr`

Scan regeneration:

- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80`

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS5 Architecture Boundary Classification

Status: merged
Branch: `ws5-architecture-boundary-classification`
PR: `https://github.com/sifr-lang/sifr/pull/1631`
Merged: `https://github.com/sifr-lang/sifr/pull/1631`

### Scope

Record explicit Category 4 architecture-boundary classifications:

- `verification/leetcode/leetcode_architecture_boundary_classification_20260424.md`

### Changes

- Documented mutable `nonlocal` capture boundary for `0673` and below-cutoff continuity fixtures.
- Documented object-identity/shared-ownership boundary for `0133`, `0138`, `0141`, `0160`, and `0894`.
- Marked vacuous/acyclic `0141` tests as boundary-limited rather than canonical cycle-input evidence.
- Stated the closure rule for future safe arena/handle or nonlocal-capture designs.

### Validation

Docs/tracking validation:

- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS6 Final Rerun, Scorecard, And Closure Review

Status: merged
Branch: `ws6-final-leetcode-closure`
PR: `https://github.com/sifr-lang/sifr/pull/1632`
Merged: `https://github.com/sifr-lang/sifr/pull/1632`

### Scope

Close the phase with final corpus artifacts, taxonomy/scorecard evidence, closure review, and remediation for failures exposed by the final full corpus rerun.

Artifacts:

- `verification/leetcode/full_corpus_current_results_20260424_leetcode_divergence_closure.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure.md`
- `verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure_delta_vs_20260409.md`
- `verification/leetcode/leetcode_pair_diff_scan_20260424.json`
- `verification/leetcode/leetcode_divergence_closure_scorecard_20260424.md`
- `reviews/ad-hoc-leetcode-divergence-closure-2026-04-24-review-pass3.md`

### Closure Remediation

The first WS6 full corpus run exposed 12 legacy fixture failures around optional `pop`, subscript, tuple-index, and node-field access under the stricter proof surface. WS6 made explicit fixture-local handling in:

- `0084_largest_rectangle_in_histogram`
- `0103_binary_tree_zigzag_level_order_traversal`
- `0232_implement_queue_using_stacks`
- `0332_reconstruct_itinerary`
- `0513_find_bottom_left_tree_value`
- `0735_asteroid_collision`
- `0739_daily_temperatures`
- `0838_push_dominoes`
- `0895_maximum_frequency_stack`
- `1046_last_stone_weight`
- `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero`
- `1609_even_odd_tree`

Targeted `check` and `run` passed for all 12 before the final full corpus rerun.

### Final Corpus Result

Command:

- `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/full_corpus_manifest_20260402_live.json --output verification/leetcode/full_corpus_current_results_20260424_leetcode_divergence_closure.json --sifr-bin ./target/release/sifr --no-build-release-if-missing`

Summary:

- Cases: `411`
- `PASS`: `208`
- `NO_ORACLE`: `203`
- `CHECK_ERROR`: `0`
- `RUN_ERROR`: `0`
- `TIMEOUT`: `0`

### Final Pair Scan

Command:

- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80`

Summary:

- Paired cases: `395`
- Python-only cases: `1`
- Sifr-only cases: `16`

### Closure Review

Review file:

- `reviews/ad-hoc-leetcode-divergence-closure-2026-04-24-review-pass3.md`

Verdict:

- PASS with `0148_sort_list` explicitly tracked as a follow-up blocker in `issues/leetcode-0148-owned-merge-sort-blocker-2026-04-24.md`.

### Validation

Targeted remediation:

- Targeted `check` and `run` for the 12 WS6-remediated fixtures PASS

Artifact generation:

- `cargo build --release -p sifr` PASS
- `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/full_corpus_manifest_20260402_live.json --output verification/leetcode/full_corpus_current_results_20260424_leetcode_divergence_closure.json --sifr-bin ./target/release/sifr --no-build-release-if-missing` PASS
- `python3 scripts/build_full_corpus_failure_taxonomy.py --results verification/leetcode/full_corpus_current_results_20260424_leetcode_divergence_closure.json --output-json verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure.json --output-md verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure.md --baseline-taxonomy verification/leetcode/full_corpus_failure_taxonomy_20260409_live_rerun1.json --generated-on 2026-04-24` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Required gates:

- `cargo fmt --check` PASS
- `git diff --check` PASS
- `scripts/run_all_tests.sh --profile quick` PASS
- `scripts/run_all_tests.sh` PASS

### Post-Closure Review Follow-Up

Reviewer feedback after closure flagged the WS6 optional-remediation helpers as a silent-fallback anti-pattern. The phase remains closed because the full corpus result is clean and the residual issue is now explicitly tracked rather than hidden in the backlog.

Follow-up issue:

- `issues/leetcode-ws6-silent-fallback-remediation-2026-04-25.md`

## WS1 D0 Narrowing Invalidation Design

Status: merged
Branch: `ws1-narrowing-invalidation-design`
PR: `https://github.com/sifr-lang/sifr/pull/1610`
Merged: `https://github.com/sifr-lang/sifr/pull/1610`

### Scope

Added the D0 design note and initial safety guardrails for existing narrowing facts.

Design note:

- `internal_docs/narrowing_flow_facts_design.md`

Compiler changes:

- Added shared sequence/dict guard invalidation when a dependent binding is rebound.
- Cleared optional binding narrowing on rebinding.
- Cleared collection flow facts after collection methods that can remove entries.

Regression coverage:

- Optional rebinding invalidates prior `is not None` narrowing.
- Sequence rebinding invalidates prior index guards.
- Index rebinding invalidates prior index guards.
- Shrinking collection mutation invalidates prior index guards.
- Shrinking field-collection mutation invalidates prior field index guards.

### Validation

Targeted validation:

- `cargo test -p sifr_hir invalidates -- --nocapture` PASS (`5` tests)
- CLI repro checks for optional rebinding, sequence rebinding, and shrinking collection mutation now reject unsafe access with type errors.

Required Rust/HIR validation:

- `cargo fmt --check` PASS
- `python3 scripts/check_hir_maintainability_guardrails.py` PASS
- `cargo clippy --workspace -- -D warnings` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS2 S1 Heap Stdlib Consumption

Status: merged
Branch: `ws2-s1-heap-stdlib`
PR: `https://github.com/sifr-lang/sifr/pull/1611`
Merged: `https://github.com/sifr-lang/sifr/pull/1611`

### Scope

The repo already had a pure Sifr `sifr.heapq` implementation with min-heap, max-heap helpers, CPython heapq compatibility imports, and e2e coverage. This wave consumes that existing stdlib surface in a representative rewrite:

- `audits/leetcode/0295_find_median_from_data_stream.sifr`

### Changes

- Rewrote `0295` from sorted-array insertion to the canonical two-heap median finder shape.
- Uses `sifr.heapq.heappush` / `heappop`.
- Keeps the max side as negated integers, matching the canonical Python fixture's max-heap-over-min-heap encoding.
- Uses local heap copies before assigning fields back, because direct mutating calls on field access do not currently mutate the stored field in place.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0295_find_median_from_data_stream` | 56 | 26 | 30 | 39/43 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0295_find_median_from_data_stream` | 66 | 24 | 42 | 39/57 |

The raw diff grows because the Sifr fixture now preserves the canonical two-heap public model instead of using a shorter sorted-array workaround.

### Validation

Targeted validation:

- `python3 audits/leetcode/0295_find_median_from_data_stream.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0295_find_median_from_data_stream.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0295_find_median_from_data_stream.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS2 S2 DSU Stdlib And First Fixture Consumption

Status: merged
Branch: `ws2-s2-dsu-stdlib`
PR: `https://github.com/sifr-lang/sifr/pull/1612`
Merged: `https://github.com/sifr-lang/sifr/pull/1612`

### Scope

Added a pure Sifr integer union-find helper and consumed it in one representative DSU fixture:

- `lib/sifr/dsu.sifr`
- `audits/leetcode/0261_graph_valid_tree.sifr`

Registry and regression coverage:

- `crates/sifr_driver/src/stdlib/registry.rs`
- `crates/sifr_driver/src/tests/stdlib_exports.rs`
- `crates/sifr/tests/e2e/pass/stdlib_dsu.sifr`

### Changes

- Added `sifr.dsu.UnionFind` with `find`, `union`, `connected`, and `component_count`.
- Uses union-by-size and path compression.
- Treats negative, out-of-range, and negative-size inputs as safe no-op/empty cases rather than relying on list subscript behavior.
- Keeps list-field mutation explicit by copying list fields into locals and assigning fields back after mutation, matching the field-mutation constraint observed in `S1`.
- Rewrote `0261_graph_valid_tree` to use `UnionFind` instead of fixture-local parent/rank helpers.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0261_graph_valid_tree` | 117 | 67 | 50 | 82/65 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0261_graph_valid_tree` | 95 | 72 | 23 | 82/33 |

### Validation

Targeted validation:

- `python3 audits/leetcode/0261_graph_valid_tree.py` PASS
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/stdlib_dsu.sifr` PASS
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_dsu.sifr` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0261_graph_valid_tree.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0261_graph_valid_tree.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Targeted Rust export tests:

- `cargo test -p sifr_driver stdlib_dsu_exports_union_find_class -- --nocapture` PASS
- `cargo test -p sifr_driver stdlib_heapq_exports_allowlisted_private_max_heap_helpers -- --nocapture` PASS

Known unrelated validation note:

- `cargo test -p sifr_driver stdlib -- --nocapture` FAILS in existing `tests::project_build_check::test_build_project_includes_reachable_support_module_stdlib_crates_in_manifest` with `[helper] type mismatch: expected 'str', got 'Result[TomlValue, TOMLDecodeError]`; this broad filter failure is unrelated to the DSU module and is not introduced by this slice.

## WS2 S3 Deque Consumption And Nonempty Popleft Codegen

Status: merged
Branch: `ws2-s3-deque-consumption`
PR: `https://github.com/sifr-lang/sifr/pull/1613`
Merged: `https://github.com/sifr-lang/sifr/pull/1613`

### Scope

The repo already had `sifr.collections.deque` and e2e coverage. This wave consumes that existing stdlib surface in one representative BFS fixture and fixes the imported-deque lowering path needed for the canonical queue shape:

- `audits/leetcode/0752_open_the_lock.sifr`
- `crates/sifr/tests/e2e/pass/deque_nonempty_popleft_narrowing.sifr`

Compiler support:

- `crates/sifr_hir/src/lower/nonempty_method_narrowing.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- `crates/sifr_codegen/src/stmt_support_emitter.rs`

### Changes

- Rewrote `0752_open_the_lock` from `list + head` queue emulation to `sifr.collections.deque`.
- Preserved explicit digit stepping because whole-token integer parsing belongs to `S5`.
- Taught non-empty pop narrowing/codegen to recognize imported `sifr.collections.deque` class names, so `while q.len() > 0: item = q.popleft()` lowers with the existing compiler-verified unwrap instead of producing mismatched Rust.
- Added a focused e2e regression for imported deque `popleft()` under a non-empty guard.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0752_open_the_lock` | 96 | 26 | 70 | 40/84 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0752_open_the_lock` | 93 | 26 | 67 | 40/81 |

### Validation

Targeted validation:

- `python3 audits/leetcode/0752_open_the_lock.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0752_open_the_lock.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0752_open_the_lock.sifr` PASS
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/deque_nonempty_popleft_narrowing.sifr` PASS
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/deque_nonempty_popleft_narrowing.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS2 S4 Character Predicate Consumption

Status: merged
Branch: `ws2-s4-char-predicate-consumption`
PR: `https://github.com/sifr-lang/sifr/pull/1614`
Merged: `https://github.com/sifr-lang/sifr/pull/1614`

### Scope

The compiler already supports string predicate methods including `.isdigit()`, with e2e coverage in `string_case_predicates`. This wave consumes that surface in a representative decode-stack fixture:

- `audits/leetcode/0394_decode_string.sifr`

### Changes

- Removed the fixture-local `isDigit` digit ladder.
- Replaced the multiplier scan guard with `peekStr(stack).isdigit()`.
- Kept `digitValue` / manual integer accumulation in place because whole-token integer parsing belongs to `S5`.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0394_decode_string` | 119 | 22 | 97 | 32/107 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0394_decode_string` | 96 | 22 | 74 | 32/84 |

### Validation

Targeted validation:

- `python3 audits/leetcode/0394_decode_string.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0394_decode_string.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0394_decode_string.sifr` PASS
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/string_case_predicates.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS2 S5 Integer Parse Consumption

Status: merged
Branch: `ws2-s5-int-parse-consumption`
PR: `https://github.com/sifr-lang/sifr/pull/1615`
Merged: `https://github.com/sifr-lang/sifr/pull/1615`

### Scope

The compiler already supports fallible `int(str)` parsing through `Result[int, ParseError]` / `try` handling. This wave consumes that surface in a representative RPN fixture:

- `audits/leetcode/0150_evaluate_reverse_polish_notation.sifr`

### Changes

- Removed fixture-local digit-value and signed-token parsing boilerplate.
- Replaced it with `int(token)` inside an explicit `try` / `except ParseError` block.
- Preserved the fixture's existing invalid-token fallback behavior by returning `0` on parse failure.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0150_evaluate_reverse_polish_notation` | 90 | 16 | 74 | 28/86 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0150_evaluate_reverse_polish_notation` | 54 | 16 | 38 | 28/50 |

### Validation

Targeted validation:

- `python3 audits/leetcode/0150_evaluate_reverse_polish_notation.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0150_evaluate_reverse_polish_notation.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0150_evaluate_reverse_polish_notation.sifr` PASS
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/result_basic.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS2 S6 Trie Helper Decision

Status: merged; corrected by `move-trie-to-leetcode-helper`
Branch: `ws2-s6-trie-decision`
PR: `https://github.com/sifr-lang/sifr/pull/1616`
Merged: `https://github.com/sifr-lang/sifr/pull/1616`

### Scope

Add an explicit LeetCode-local trie helper shape and consume it in the trie-dependent fixtures:

- `internal_docs/leetcode_trie_helper_design.md`
- `audits/leetcode/0208_implement_trie_prefix_tree.sifr`

### Decision

The LeetCode trie helper uses owned node indices backed by owned edge lists plus terminal markers. `insert` is the only node-creating operation; traversal APIs return `int | None` for missing edges or invalid node indices. This rejects auto-insert-on-read while leaving enough API surface for `0211` wildcard DFS and `0212` board-prefix pruning rewrites.

Correction: the helper does not belong in the public Sifr stdlib. It is kept inline in trie-dependent LeetCode fixtures because non-`main.sifr` LeetCode roots still compile in single-file mode and cannot import sibling helper modules through the CLI.

### Changes

- Added a LeetCode-local `Trie` helper shape with whole-word APIs (`insert`, `contains`, `search`, `starts_with`, `startsWith`) and node traversal APIs (`find_node`, `child`, `children`, `is_terminal`, `node_count`).
- Rewrote `0208_implement_trie_prefix_tree.sifr` to use the helper directly instead of scanning a word list.
- Removed the public `sifr.trie` stdlib module, registry entry, stdlib export regression, and `stdlib_trie` e2e fixture in the follow-up correction.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0208_implement_trie_prefix_tree` | 79 | 63 | 16 | 78/31 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0208_implement_trie_prefix_tree` | 70 | 68 | 2 | 78/12 |

### Validation

Targeted validation:

- `python3 audits/leetcode/0208_implement_trie_prefix_tree.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0208_implement_trie_prefix_tree.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0208_implement_trie_prefix_tree.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS3 B1 Fixture Helper Convention

Status: merged
Branch: `ws3-b1-fixture-helper-convention`
PR: `https://github.com/sifr-lang/sifr/pull/1617`
Merged: `https://github.com/sifr-lang/sifr/pull/1617`

### Scope

Choose the linked-list/tree fixture helper strategy and pilot it without introducing new cursor semantics:

- `internal_docs/leetcode_fixture_helper_convention.md`
- `audits/leetcode/0021_merge_two_sorted_lists.sifr`

### Decision

Use self-contained fixture boilerplate with a strict inline helper template. Import-based helper modules are blocked for current LeetCode root fixtures because non-`main.sifr` entries intentionally compile in single-file mode, so sibling imports are not resolved by the CLI. The accepted convention keeps only used structural helpers inline and removes unrelated catch-all node scaffolding.

### Changes

- Documented the WS3 B1 helper convention and the current CLI constraint that prevents shared sibling helper imports for non-`main.sifr` fixture entries.
- Removed unused catch-all `Node` scaffolding from the `0021` pilot.
- Kept the existing drain/sort/rebuild algorithm unchanged; owned-chain cursor behavior belongs to later WS3 slices.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0021_merge_two_sorted_lists` | 121 | 45 | 76 | 74/105 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0021_merge_two_sorted_lists` | 123 | 60 | 63 | 74/77 |

### Validation

Targeted validation:

- `python3 audits/leetcode/0021_merge_two_sorted_lists.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0021_merge_two_sorted_lists.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0021_merge_two_sorted_lists.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0211 Trie Wildcard Rewrite

Status: merged
Branch: `ws4-0211-trie-wildcard-rewrite`
PR: `https://github.com/sifr-lang/sifr/pull/1618`
Merged: `https://github.com/sifr-lang/sifr/pull/1618`

### Scope

Use the WS2 S6 LeetCode trie helper to rewrite the wildcard word dictionary fixture:

- `audits/leetcode/0211_design_add_and_search_words_data_structure.sifr`

### Changes

- Replaced the fixture-local `list[str]` storage and per-word linear wildcard scan with an inline LeetCode trie helper.
- Added explicit wildcard DFS over trie node indices using `children`, `child`, and `is_terminal`.
- Preserved explicit field write-back after insertion because mutating a copied class field value does not update the stored field.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0211_design_add_and_search_words_data_structure` | 71 | 52 | 19 | 66/33 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0211_design_add_and_search_words_data_structure` | 82 | 51 | 31 | 66/46 |

The raw line diff increases because the previous Sifr fixture used a short word-list scan. This wave closes the structural rewrite criterion: the fixture now uses trie traversal with wildcard DFS and no per-word full scan.

### Validation

Targeted validation:

- `python3 audits/leetcode/0211_design_add_and_search_words_data_structure.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0211_design_add_and_search_words_data_structure.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0211_design_add_and_search_words_data_structure.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0212 Trie Board-Search Rewrite

Status: merged
Branch: `ws4-0212-trie-board-search`
PR: `https://github.com/sifr-lang/sifr/pull/1619`
Merged: `https://github.com/sifr-lang/sifr/pull/1619`

### Scope

Use the WS2 S6 LeetCode trie helper to replace per-word board searches with prefix-pruned trie traversal:

- `audits/leetcode/0212_word_search_ii.sifr`

### Changes

- Replaced `_word_exists` per-word full-board search with one trie build and board DFS from each cell.
- Added prefix-pruned traversal through `Trie.child` and terminal detection through `Trie.is_terminal`.
- Used an owned `found` accumulator map through recursive DFS to avoid mutable nonlocal capture.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0212_word_search_ii` | 127 | 83 | 44 | 90/51 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0212_word_search_ii` | 149 | 81 | 68 | 90/77 |

The raw line diff increases because the previous Sifr fixture was a shorter per-word search. This wave closes the structural rewrite criterion: the fixture now builds one trie and prunes board DFS by prefix instead of running a full-board search for each word.

### Validation

Targeted validation:

- `python3 audits/leetcode/0212_word_search_ii.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0212_word_search_ii.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0212_word_search_ii.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0146 Recency Structure Design

Status: merged
Branch: `ws4-0146-recency-design`
PR: `https://github.com/sifr-lang/sifr/pull/1620`
Merged: `https://github.com/sifr-lang/sifr/pull/1620`

### Scope

Choose the O(1) recency representation before rewriting the LRU cache fixture:

- `internal_docs/leetcode_0146_lru_recency_design.md`

### Decision

Use integer node handles plus maps for the recency list: `key_to_node`, `node_key`, `node_value`, `prev`, and `next`, with fixed `head` / `tail` sentinel node ids and monotonically allocated entry node ids. Absence is determined through `key_to_node`, not a stored sentinel value, so real cached value `-1` remains representable.

### Validation

Docs/design validation:

- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0706 HashMap Storage Design

Status: merged
Branch: `ws4-0706-hashmap-storage-design`
PR: `https://github.com/sifr-lang/sifr/pull/1621`
Merged: `https://github.com/sifr-lang/sifr/pull/1621`

### Scope

Choose the storage representation before rewriting the design-hashmap fixture:

- `internal_docs/leetcode_0706_hashmap_storage_design.md`

### Decision

Use separate chaining with explicit `list[list[tuple[int, int]]]` buckets and a fixed prime bucket count. `get`, `put`, and `remove` scan only one bucket; `remove` rebuilds the bucket without the key instead of writing a sentinel value. Real stored value `-1` remains representable because absence is determined by key lookup.

### Validation

Docs/design validation:

- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0706 HashMap Rewrite

Status: merged
Branch: `ws4-0706-hashmap-rewrite`
PR: `https://github.com/sifr-lang/sifr/pull/1622`
Merged: `https://github.com/sifr-lang/sifr/pull/1622`

### Scope

Rewrite the design-hashmap fixture to stop delegating storage to built-in `dict`:

- `audits/leetcode/0706_design_hashmap.sifr`

### Changes

- Replaced `dict[int, int]` storage with explicit `list[list[tuple[int, int]]]` buckets.
- Implemented bucket-local `put`, `get`, and `remove` operations.
- Made `remove` rebuild the target bucket without the removed key instead of writing `-1`.
- Added assertions for storing and updating a real `-1` value.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0706_design_hashmap` | 62 | 51 | 11 | 68/28 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0706_design_hashmap` | 118 | 48 | 70 | 68/90 |

The raw line diff increases because the previous Sifr fixture delegated to a built-in dictionary. This wave closes the structural rewrite criterion: storage is explicit bucket chaining, `get` / `put` / `remove` scan only one bucket, and removal deletes entries rather than writing sentinel values.

### Validation

Targeted validation:

- `python3 audits/leetcode/0706_design_hashmap.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0706_design_hashmap.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0706_design_hashmap.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0146 LRU Cache Rewrite

Status: merged
Branch: `ws4-0146-lru-rewrite`
PR: `https://github.com/sifr-lang/sifr/pull/1623`
Merged: `https://github.com/sifr-lang/sifr/pull/1623`

### Scope

Rewrite the LRU cache fixture to use the accepted O(1) recency representation:

- `audits/leetcode/0146_lru_cache.sifr`

### Changes

- Replaced parallel `keys` / `values` arrays and shifting/popping recency operations with map-backed integer node handles.
- Added `detach`, `insertAfter`, `moveToFront`, and `evictLru` helper methods over `prev` / `next` maps.
- Preserved explicit field write-back for mutated maps to avoid copied-field mutation loss.
- Added assertions for storing, reading, updating, and evicting a real `-1` value.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0146_lru_cache` | 79 | 33 | 46 | 51/64 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0146_lru_cache` | 147 | 33 | 114 | 51/132 |

The raw line diff increases because the previous Sifr fixture used compact array shifts. This wave closes the structural rewrite criterion: `get`, `put`, update, and eviction use map lookups plus recency-list rewiring instead of linear scans or array shifts.

### Validation

Targeted validation:

- `python3 audits/leetcode/0146_lru_cache.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0146_lru_cache.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0146_lru_cache.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0004 Binary Median Rewrite

Status: merged
Branch: `ws4-0004-binary-median`
PR: `https://github.com/sifr-lang/sifr/pull/1624`
Merged: `https://github.com/sifr-lang/sifr/pull/1624`

### Scope

Rewrite the median-of-two-sorted-arrays fixture to use binary partitioning:

- `audits/leetcode/0004_median_of_two_sorted_arrays.py`
- `audits/leetcode/0004_median_of_two_sorted_arrays.sifr`

### Changes

- Replaced the Sifr full merge with a count-based binary partition over the shorter input.
- Used explicit numeric sentinels for empty partition sides instead of merged-array indexing.
- Added odd, even, empty-side, all-zero, and negative-value assertions to the paired fixtures.
- Avoided storing borrowed parameter lists into local list variables by selecting the shorter input with lengths and a boolean selector.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0004_median_of_two_sorted_arrays` | 63 | 32 | 31 | 42/41 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0004_median_of_two_sorted_arrays` | 105 | 33 | 72 | 47/86 |

The raw line diff increases because the canonical binary-partition algorithm needs explicit optional indexing and boundary sentinels in Sifr. This wave closes the structural rewrite criterion: the Sifr fixture no longer performs a full merge and covers odd/even plus empty-side cases.

### Validation

Targeted validation:

- `python3 audits/leetcode/0004_median_of_two_sorted_arrays.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0004_median_of_two_sorted_arrays.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0004_median_of_two_sorted_arrays.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS
- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0206 Reverse Linked List Rewrite

Status: merged
Branch: `ws4-0206-reverse-linked-list`
PR: `https://github.com/sifr-lang/sifr/pull/1625`
Merged: `https://github.com/sifr-lang/sifr/pull/1625`

### Scope

Rewrite the reverse-linked-list fixture to use the canonical `ListNode` public model and owned node rewiring:

- `audits/leetcode/0206_reverse_linked_list.py`
- `audits/leetcode/0206_reverse_linked_list.sifr`

### Changes

- Replaced the Sifr `list[int]` public model with the shared `ListNode` fixture helper shape.
- Implemented recursive owned-node reversal with `reverseInto(own mut cur, own prev)` and direct `.next` rewiring.
- Removed the unused catch-all `Node` helper from the Python pair and made the Python signature explicitly nullable.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0206_reverse_linked_list` | 71 | 54 | 17 | 57/20 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0206_reverse_linked_list` | 62 | 23 | 39 | 38/54 |

This wave closes the structural rewrite criterion: the Sifr fixture no longer exposes `list[int]` and reverses the owned node chain by rewiring links rather than copying values through an array.

### Validation

Targeted validation:

- `python3 audits/leetcode/0206_reverse_linked_list.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0206_reverse_linked_list.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0206_reverse_linked_list.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS
- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0024 Swap Nodes In Pairs Rewrite

Status: merged
Branch: `ws4-0024-swap-pairs`
PR: `https://github.com/sifr-lang/sifr/pull/1626`
Merged: `https://github.com/sifr-lang/sifr/pull/1626`

### Scope

Rewrite the swap-pairs fixture to use the canonical `ListNode` public model and owned node rewiring:

- `audits/leetcode/0024_swap_nodes_in_pairs.py`
- `audits/leetcode/0024_swap_nodes_in_pairs.sifr`

### Changes

- Replaced the Sifr `list[int]` public model with the shared `ListNode` fixture helper shape.
- Implemented recursive pair swapping by rewiring `head.next` and `second.next`.
- Removed the unused catch-all `Node` helper from the Python pair and made the Python signature explicitly nullable.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0024_swap_nodes_in_pairs` | 79 | 63 | 16 | 67/20 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0024_swap_nodes_in_pairs` | 66 | 30 | 36 | 48/54 |

This wave closes the structural rewrite criterion: the Sifr fixture no longer exposes `list[int]` and swaps adjacent owned nodes by rewiring links.

### Validation

Targeted validation:

- `python3 audits/leetcode/0024_swap_nodes_in_pairs.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0024_swap_nodes_in_pairs.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0024_swap_nodes_in_pairs.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS
- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0147 Insertion Sort List Rewrite

Status: merged
Branch: `ws4-0147-insertion-sort-list`
PR: `https://github.com/sifr-lang/sifr/pull/1627`
Merged: `https://github.com/sifr-lang/sifr/pull/1627`

### Scope

Rewrite the insertion-sort-list fixture to sort the owned node chain directly:

- `audits/leetcode/0147_insertion_sort_list.py`
- `audits/leetcode/0147_insertion_sort_list.sifr`

### Changes

- Replaced the Sifr drain/sort/rebuild implementation with recursive owned-node insertion sort.
- Added `insertSorted` and `sortInto` helpers that move nodes by rewiring `.next`.
- Removed unused `Node` / `unwrapInt` helper baggage from the paired fixtures.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0147_insertion_sort_list` | 102 | 36 | 66 | 61/91 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0147_insertion_sort_list` | 79 | 29 | 50 | 42/63 |

This wave closes the structural rewrite criterion: the Sifr fixture no longer drains values into a list, calls `sorted`, or rebuilds a new result chain.

### Validation

Targeted validation:

- `python3 audits/leetcode/0147_insertion_sort_list.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0147_insertion_sort_list.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0147_insertion_sort_list.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS
- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0707 Linked List Design Rewrite

Status: merged
Branch: `ws4-0707-linked-list-design`
PR: `https://github.com/sifr-lang/sifr/pull/1628`
Merged: `https://github.com/sifr-lang/sifr/pull/1628`

### Scope

Rewrite the design-linked-list fixture to use linked-list storage rather than an array-backed public model:

- `audits/leetcode/0707_design_linked_list.py`
- `audits/leetcode/0707_design_linked_list.sifr`

### Changes

- Replaced the Sifr `list[int]` storage with a singly owned `ListNode` chain plus explicit `size`.
- Implemented `get`, `addAtHead`, `addAtTail`, `addAtIndex`, and `deleteAtIndex` through recursive chain helpers.
- Aligned the Python fixture with LeetCode index semantics by tracking `size`, rejecting `index > size`, and treating negative insert indexes as head insertion.
- Added assertions for out-of-range insert, negative-index head insertion, and delete-at-head behavior.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0707_design_linked_list` | 108 | 64 | 44 | 82/62 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0707_design_linked_list` | 113 | 50 | 63 | 86/99 |

The raw line diff increases slightly because the Sifr fixture now carries explicit node helpers and size bookkeeping. This wave closes the structural rewrite criterion: operations use a linked node chain instead of array slicing, concatenation, or filtering.

### Validation

Targeted validation:

- `python3 audits/leetcode/0707_design_linked_list.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0707_design_linked_list.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0707_design_linked_list.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS
- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0023 Merge K Sorted Lists Rewrite

Status: merged
Branch: `ws4-0023-merge-k-lists-heap`
PR: `https://github.com/sifr-lang/sifr/pull/1629`
Merged: `https://github.com/sifr-lang/sifr/pull/1629`

### Scope

Rewrite the merge-k-sorted-lists fixture to use the `ListNode` public model and heap-backed ordering:

- `audits/leetcode/0023_merge_k_sorted_lists.py`
- `audits/leetcode/0023_merge_k_sorted_lists.sifr`

### Changes

- Replaced the Sifr `list[list[int]]` public model with `ListNode` helpers and `list[ListNode | None]` input.
- Used `sifr.heapq.heappush` / `heappop` to collect node values in sorted order instead of calling `merged.sort()`.
- Rebuilt the result as a `ListNode` chain and removed unused Python catch-all `Node` helper baggage.

### Pair Scan Movement

Previous stats from `origin/main` scan artifact:

| Fixture | Previous changed_total | Previous changed_py | Previous changed_sifr | Previous lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0023_merge_k_sorted_lists` | 101 | 88 | 13 | 92/17 |

Regenerated artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`

| Fixture | Current changed_total | Current changed_py | Current changed_sifr | Current lines py/sifr |
| --- | ---: | ---: | ---: | --- |
| `0023_merge_k_sorted_lists` | 117 | 56 | 61 | 73/78 |

The raw line diff increases because the Sifr fixture now includes the linked-list helper surface. This wave closes the public-model criterion and removes the full-array `sort()` workaround by using heap ordering.

### Validation

Targeted validation:

- `python3 audits/leetcode/0023_merge_k_sorted_lists.py` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0023_merge_k_sorted_lists.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0023_merge_k_sorted_lists.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS
- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0148 Merge Sort Blocker Tracking

Status: opened PR
Branch: `ws4-0148-blocker-note`
PR: `https://github.com/sifr-lang/sifr/pull/1630`

### Scope

Track the remaining canonical linked-list merge sort blocker:

- `issues/leetcode-0148-owned-merge-sort-blocker-2026-04-24.md`

### Decision

`0148_sort_list` remains the only WS4 rewrite item not safely closed in this phase. Direct owned-node insertion sort is expressible (`0147`), and single-chain rewiring is expressible (`0206`, `0024`, `0707`), but the canonical `0148` merge requires moving one of two owned list heads across sibling branches. The checker currently reports moved-value errors for both optional and non-optional two-list merge helper shapes.

The fixture must not be replaced by another drain/sort/rebuild workaround. It is now separately tracked as an owned two-list merge/cursor capability gap.

### Validation

Docs/tracking validation:

- `cargo fmt --check` PASS
- `git diff --check` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS
