# Ad-hoc Phase Execution: LeetCode Divergence Closure

Status: in_progress
Started: 2026-04-24
Phase plan: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`

## Wave Checklist

- [x] WS0 corpus normalization and baseline refresh
- [ ] WS1 narrowing design and first compiler slices
- [ ] WS2 heap / DSU / collection stdlib parity
- [ ] WS3 owned-chain helper convention and cursor slices
- [ ] WS4 canonical rewrite debt
- [ ] WS5 architecture boundary documentation
- [ ] WS6 final rerun, scorecard, and closure review

## WS0 Corpus Normalization And Baseline Refresh

Status: validated locally
Branch: `ws0-leetcode-corpus-noise-normalization`
PR: `https://github.com/yaseralnajjar/sifr/pull/1609`

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
| `0516_longest_palindromic_subsequence` | 25 | 13 | 12 | 34/33 |

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
