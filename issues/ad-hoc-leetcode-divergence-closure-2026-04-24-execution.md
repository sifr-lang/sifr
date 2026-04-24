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

Status: validated locally
Branch: `ws2-s2-dsu-stdlib`
PR: `https://github.com/yaseralnajjar/sifr/pull/1612`

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
