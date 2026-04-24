# Ad-hoc Phase Execution: LeetCode Divergence Closure

Status: in_progress
Started: 2026-04-24
Phase plan: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`

## Wave Checklist

- [x] WS0 corpus normalization and baseline refresh
- [x] WS1 narrowing design and first compiler slices
- [ ] WS2 heap / DSU / collection stdlib parity
- [ ] WS3 owned-chain helper convention and cursor slices
- [ ] WS4 canonical rewrite debt
- [ ] WS5 architecture boundary documentation
- [ ] WS6 final rerun, scorecard, and closure review

## WS0 Corpus Normalization And Baseline Refresh

Status: merged
Branch: `ws0-leetcode-corpus-noise-normalization`
PR: `https://github.com/yaseralnajjar/sifr/pull/1609`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1609`

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

## WS1 D0 Narrowing Invalidation Design

Status: merged
Branch: `ws1-narrowing-invalidation-design`
PR: `https://github.com/yaseralnajjar/sifr/pull/1610`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1610`

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
PR: `https://github.com/yaseralnajjar/sifr/pull/1611`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1611`

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
PR: `https://github.com/yaseralnajjar/sifr/pull/1612`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1612`

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
PR: `https://github.com/yaseralnajjar/sifr/pull/1613`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1613`

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
PR: `https://github.com/yaseralnajjar/sifr/pull/1614`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1614`

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
PR: `https://github.com/yaseralnajjar/sifr/pull/1615`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1615`

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

## WS2 S6 Trie Decision And API

Status: merged
Branch: `ws2-s6-trie-decision`
PR: `https://github.com/yaseralnajjar/sifr/pull/1616`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1616`

### Scope

Add an explicit trie stdlib surface and consume it in the first trie-dependent fixture:

- `internal_docs/trie_stdlib_design.md`
- `lib/sifr/trie.sifr`
- `audits/leetcode/0208_implement_trie_prefix_tree.sifr`

### Decision

`sifr.trie.Trie` uses owned node indices backed by owned edge lists plus terminal markers. `insert` is the only node-creating operation; traversal APIs return `int | None` for missing edges or invalid node indices. This rejects auto-insert-on-read while leaving enough API surface for later `0211` wildcard DFS and `0212` board-prefix pruning rewrites.

### Changes

- Added `sifr.trie.Trie` with whole-word APIs (`insert`, `contains`, `search`, `starts_with`, `startsWith`) and node traversal APIs (`find_node`, `child`, `children`, `is_terminal`, `node_count`).
- Registered `sifr.trie` in the embedded stdlib registry and added an export regression for the class.
- Added `stdlib_trie` e2e coverage for word lookup, prefix lookup, terminal markers, child traversal, and invalid node handling.
- Rewrote `0208_implement_trie_prefix_tree.sifr` to import `sifr.trie.Trie` directly instead of scanning a word list.

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
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/stdlib_trie.sifr` PASS
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_trie.sifr` PASS
- `cargo run -q -p sifr -- check audits/leetcode/0208_implement_trie_prefix_tree.sifr` PASS
- `cargo run -q -p sifr -- run audits/leetcode/0208_implement_trie_prefix_tree.sifr` PASS
- `cargo test -p sifr_driver stdlib_trie_exports_trie_class -- --nocapture` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS3 B1 Fixture Helper Convention

Status: merged
Branch: `ws3-b1-fixture-helper-convention`
PR: `https://github.com/yaseralnajjar/sifr/pull/1617`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1617`

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
PR: `https://github.com/yaseralnajjar/sifr/pull/1618`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1618`

### Scope

Use the WS2 S6 trie API to rewrite the wildcard word dictionary fixture:

- `audits/leetcode/0211_design_add_and_search_words_data_structure.sifr`

### Changes

- Replaced the fixture-local `list[str]` storage and per-word linear wildcard scan with a `sifr.trie.Trie`.
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
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_trie.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0212 Trie Board-Search Rewrite

Status: merged
Branch: `ws4-0212-trie-board-search`
PR: `https://github.com/yaseralnajjar/sifr/pull/1619`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1619`

### Scope

Use the WS2 S6 trie API to replace per-word board searches with prefix-pruned trie traversal:

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
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_trie.sifr` PASS
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260424.json --top 80` PASS

Local gate:

- `scripts/run_all_tests.sh --profile quick` PASS

## WS4 0146 Recency Structure Design

Status: merged
Branch: `ws4-0146-recency-design`
PR: `https://github.com/yaseralnajjar/sifr/pull/1620`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1620`

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
PR: `https://github.com/yaseralnajjar/sifr/pull/1621`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1621`

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
PR: `https://github.com/yaseralnajjar/sifr/pull/1622`
Merged: `https://github.com/yaseralnajjar/sifr/pull/1622`

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

Status: validated locally
Branch: `ws4-0146-lru-rewrite`

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
