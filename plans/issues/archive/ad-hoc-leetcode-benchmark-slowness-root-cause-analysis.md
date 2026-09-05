# Ad Hoc Phase: Fix LeetCode Benchmark Slowness Root Causes

Status: complete on 2026-05-30; M1 heap/trie/direct/stateful parity waves merged through `sifr-lang/leetcode#20`; M2 stateful/list-key/trie-direct-state/TinyURL waves merged through `sifr-lang/sifr#2208` and `sifr-lang/leetcode#24`; final reintegration merged through `sifr-lang/sifr#2220`, `sifr-lang/leetcode#30`, and `sifr-lang/leetcode#31`; 2026-05-31 canonical-Python audit and fresh full Python/Sifr benchmark closure completed locally with 0 measured-slower problems; 2026-06-01 394-problem Python/Sifr runtime and memory closure completed locally with 0 regressions
Context: corrective implementation phase for `audits/leetcode` after the completed benchmark analyzer/report phase identified every measured Sifr-slower, partial, and failed LeetCode benchmark case.

## 2026-06-01 394-Problem Closure Addendum

The LeetCode benchmark registry now has 394 benchmarkable problems with both canonical Python sources and matching Sifr sources. Python sources were not changed for this closure; Sifr fixtures and benchmark harness code were adjusted where emitted Rust or runner behavior made the comparison non-representative.

Fresh local raw-result audit after the split full run and affected-harness reruns:

| Metric | Count |
| --- | ---: |
| Registry problems | 394 |
| Benchmarkable problems | 394 |
| Fully complete problems | 394 |
| Complete fixture pairs | 1178 |
| Measured Sifr runtime regressions vs Python | 0 |
| Measured Sifr peak-RSS regressions vs Python | 0 |
| Partial benchmark problems | 0 |
| No-pair failed problems | 0 |

Merged PRs: `sifr-lang/leetcode#36` and `sifr-lang/sifr#2225`.

Closure fixes included:

- `0076_minimum_window_substring`: explicit `dict[str, int]` annotations avoid mixed string/character map inference in generated Rust.
- `0662_maximum_width_of_binary_tree`: direct optional index unwrapping avoids nested optional codegen in the runner build.
- `0981_time_based_key_value_store`: direct map membership plus indexed bucket reads avoid cloning the full timestamp list on every binary-search lookup.
- `0332_reconstruct_itinerary`: sorted adjacency traversal now uses map bucket append and per-source cursors instead of cloned adjacency slices.
- `1462_course_schedule_iv`: Sifr now follows the Python DFS/memo prerequisite-set algorithm, including self-prerequisite membership.
- Generic Sifr benchmark runners now avoid retaining large validation list results into benchmark loops, use structural checksums for `list[list[int]]` loop results, and copy parsed base inputs for mutating list runners instead of repeatedly tokenizing large fixtures.
- `0047_permutations_ii` fixture generation now uses four repeated copies per value (`index // 4`) instead of three. This keeps the duplicate-permutation algorithm covered while preventing benchmark memory from being dominated by an enormous expected/result artifact.

agent review round `reviews/complete-sifr-leetcode-benchmarks-review-2.md` found no blockers.

## 2026-05-31 Closure Addendum

The Python LeetCode sources are treated as the canonical oracle. Changes introduced after `091aade2a6c76637f9d2c50ccf00d5e8d972dd7d` that altered Python problem algorithms were reverted to that canonical state. Matching Sifr fixtures were then updated toward the Python algorithms unless current Sifr semantics require an explicit, documented deviation.

Important fixture decisions:

- `0929_unique_email_addresses`: Python remains canonical. Sifr uses the same split/replace normalization shape, but consumes the owned benchmark input with `pop()` because the result is a set and input order is irrelevant; this avoids generated per-element string clones while preserving problem semantics.
- `0205_isomorphic_strings` and `0567_permutation_in_string`: Sifr keeps fixed-array/explicit-index helpers because canonical Python relies on cheap dynamic character keys/`ord()`, while Sifr's dynamic string `ord()` is fallible and current string-key dictionary lowering is not yet comparable.
- `0149_max_points_on_a_line` and `2001_number_of_pairs_of_interchangeable_rectangles`: Sifr keeps exact reduced tuple keys because Rust `f64` is not a `HashMap` key and exact rational keys avoid precision drift.
- `0049_group_anagrams`: Sifr uses the canonical count-key algorithm; the key is chunked because Rust tuple `Hash`/`Eq` support is bounded.
- `2002_maximum_product_of_the_length_of_two_palindromic_subsequences`: Python was restored to the canonical `lru_cache` recurrence from `091aade2a6c76637f9d2c50ccf00d5e8d972dd7d`; Sifr now uses the same memoized recurrence with an explicit list cache because decorators are not Sifr syntax.

Fresh local closure run:

```text
SIFR_BIN=target/release/sifr python3 benchmarks/bench.py run --language python --language sifr --runs 2 --warmup 1
python3 benchmarks/analyze_slowness.py --check-metadata
```

Analyzer snapshot after the run:

| Metric | Count |
| --- | ---: |
| Registry problems | 394 |
| Benchmarkable problems | 325 |
| Fully complete problems | 325 |
| Complete fixture pairs | 971 |
| Measured-slower problems | 0 |
| Partial benchmark problems | 0 |
| No-pair failed problems | 0 |

Compiler/root-cause fixes added in this closure wave include string split/replace literal-pattern lowering, dead string character-cache suppression for mutated strings that are never indexed/sliced/`len`-queried, nested list/dict mutation lowering, borrowed string comparisons, cached string-index comparison without `String` allocation, efficient single-element list repeat lowering, self-field clone suppression, and direct mutable state updates for list-indexed dictionaries.

## Purpose

Fix the actual root causes behind the LeetCode benchmark slowness, not just classify them. This phase turns the prior diagnostic inventory into an implementation plan that restores apples-to-apples benchmark parity, removes generated-code performance pathologies, and prevents the report from presenting known-divergent implementations as language-performance evidence.

The benchmark only becomes a language-performance signal when:

- the Python and Sifr problem implementations use equivalent algorithms and data structures,
- generated Rust preserves the intended complexity instead of inserting hidden full-container copies,
- benchmark harness overhead is symmetric enough to not dominate the result,
- failed correctness cases are not treated as performance data.

This phase is successful only when the known root-cause families below have either been fixed or explicitly reclassified with fresh emitted-code evidence and benchmark results.

## Source Inputs

- HTML report: `audits/leetcode/benchmarks/results/report.html`
- Raw hyperfine and memory output: `audits/leetcode/benchmarks/results/.raw`
- Problem registry: `audits/leetcode/benchmarks/problems`
- Fixture cases: `audits/leetcode/benchmarks/cases`
- Python implementations: `audits/leetcode/src/*.py`
- Sifr implementations: `audits/leetcode/src/*.sifr`
- Emitted Rust sampled with `target/release/sifr emit audits/leetcode/benchmarks/generated/sifr/<problem>_runner.sifr`

Benchmark profile used by this analysis:

- `python3 benchmarks/bench.py run --runs 2 --warmup 1 --memory-runs 1`
- 272 fully complete problems, 814 complete fixture-size pairs
- 273 problems have at least one complete Python/Sifr pair
- 75 complete problems have at least one fixture where Sifr is slower than Python
- 52 problems have no complete Python/Sifr pair at any fixture size
- 53 problems are incomplete if partial problems are included; `0234_palindrome_linked_list` has two complete fixture pairs but one missing fixture pair

### Count Reconciliation

agent review pass 1 challenged the count as 78 slower and 52 failed. Re-reading the raw data locally gives a stricter, reproducible definition:

- **75 slower problems**: problems with at least one `.hyperfine.json` containing both `python` and `sifr` rows where `python.mean / sifr.mean < 1`.
- **52 no-pair failures**: problems with no fixture where both implementations produced hyperfine rows.
- **53 incomplete problems**: the 52 no-pair failures plus `0234_palindrome_linked_list`, which has partial benchmark data but is not fully complete.

This phase uses the 75 measured-slower problem definition for the slowness table and separately tracks all 53 incomplete/failed problem entries in the failure appendix.

## Executive Summary

There are two distinct classes of slowness, and they must be fixed in this order:

1. **Restore benchmark parity first.**
   If the Python and Sifr solutions use different algorithms or data structures, port the Sifr LeetCode solution to the Python algorithm before treating the result as a compiler-performance problem.

2. **Fix compiler/runtime lowering where parity already exists.**
   Equivalent Sifr solutions should not emit repeated string scans, full container clones, cloned optional trees/lists, or row-copying matrix updates in hot loops.

3. **Re-run and reclassify after each fix.**
   A problem only leaves this phase when correctness passes, runtime and memory are refreshed from raw benchmark output, and the registry metadata is updated from the analyzer.

### Fix in the Sifr compiler/runtime

These are cases where the Sifr code is broadly similar to the Python solution, but emitted Rust changes the practical cost model.

Primary compiler/runtime causes:

1. **String indexing and length lowering is too expensive.**
   Emitted Rust commonly uses `s.chars().nth(i)`, `s.chars().count()`, and per-character `to_string()` in loops. That turns index-heavy string algorithms into repeated scans and allocations.

2. **Container access and mutation clone too much.**
   Emitted Rust often uses `.clone()` around list/dict/set/class-field access. Some helpers that look like local aliases in Sifr become full vector/hashmap clones in Rust.

3. **Class field mutation and object-state methods clone state in hot paths.**
   Stateful LeetCode classes such as maps, caches, tries, browser history, and streams expose full-field clones around `self.field` access and update.

4. **Matrix/list cell helpers hide row/container copies.**
   Several graph and DP Sifr implementations use safe helper wrappers like `getIntCell` / `setIntCell`. Those are semantically safe, but generated code often clones rows or containers during nested loops.

5. **Generated benchmark runners add parsing/allocation overhead.**
   Object-operation runners split the full fixture into `Vec<String>` and split each line on every loop. Python does similar parsing, so this is not the primary cause, but it amplifies clone-heavy Sifr object workloads.

### Fix in the LeetCode Sifr code

These are cases where Python and Sifr are not apples-to-apples.

Primary LeetCode-code causes:

1. **Algorithmic divergence.**
   Python uses `heapq`, set grouping, pruning, or amortized O(1) structures while Sifr often uses repeated scans, insertion sort, vector-backed lookups, or O(V^2) graph loops.

2. **Shared Sifr helper is not equivalent to the Python local implementation.**
   The Tries category is the clearest case: Python defines problem-specific trie nodes, while Sifr imports `helpers.trie.Trie`. That helper is arena-based, vector-of-edges based, and currently emits full-structure clones.

3. **Correctness divergence in failed cases.**
   `0212_word_search_ii` previously returned duplicate found words on fixtures with duplicate input words; PR `sifr-lang/leetcode#19` moved it to a complete, equivalent trie-parity row with residual mixed slowness.

4. **Sifr workaround code is more defensive than the Python source.**
   Many Sifr solutions include `None` guards, wrapper helpers, copied rows, or fallback default values because current language/library support makes direct Python-style code hard. Some of that is necessary today, but it means the benchmark is not comparing equivalent code.

## Implementation Decisions

These decisions are locked for this phase so implementation work can proceed without re-litigating benchmark semantics.

### D1: Apples-To-Apples Means Same Algorithm And Comparable Data Structure

For benchmark purposes, "same problem" is not enough. A Sifr solution is equivalent only when it uses the same algorithmic complexity class and a comparable data structure to the Python implementation selected by the runner.

Implications:

- `heapq` Python solutions must be matched with a Sifr heap/priority-queue implementation, not repeated scans or sorted-vector insertion.
- Python deque/monotonic-queue solutions must be matched with a Sifr deque/monotonic-queue implementation, not window rescans.
- Python trie-node solutions must be matched by equivalent Sifr trie-node algorithms for the audited problems, even if a shared Sifr helper also exists.
- Stateful design problems must compare against the final Python class definition that the runner imports, not an earlier alternate class in the file.

### D2: Fix LeetCode Sifr Code Before Compiler Attribution For Known-Divergent Rows

Rows marked `leetcode_sifr_code` or known-divergent `mixed` are not compiler regressions yet. Their first implementation ticket is a Sifr-code parity repair. Only after the Sifr code uses the equivalent algorithm should remaining slowness be attributed to compiler/runtime lowering.

Primary parity repairs:

- Heap/priority queue: `1985`, `0973`, `0703`, `1046`, `1834`, `0295`, `1631`, `0778`.
- Trie/correctness: `0208`, `0211`, `0212`.
- Direct algorithm divergence: `0015`, `0239`, `0496`, `2306`.
- Stateful parity review before compiler-only work: `0146`, `0355`, `0380`, `1396`, `1472`.

### D3: Add Reusable Audit Helpers Only When They Preserve Parity

Reusable helpers are allowed, but they must not change the benchmarked algorithm.

Decisions:

- Use the existing `lib/sifr/heapq.sifr` API for heap parity unless a specific problem proves a missing operation:
  - `heapify(heap)`,
  - `heappush(heap, item)`,
  - `heappop(heap) -> T | None`,
  - `heappushpop(heap, item) -> T | None`,
  - `heapreplace(heap, item) -> T | None`,
  - `nsmallest(n, items)`,
  - `nlargest(n, items)`.
- Use the existing `sifr.collections.deque` API for monotonic queue parity:
  - `append`,
  - `appendleft`,
  - `pop`,
  - `popleft`,
  - `len`,
  - `clear`.
- Do not use the current shared `helpers.trie.Trie` as the parity implementation for `0208`, `0211`, or `0212` unless the Python benchmark side is changed to the same representation. The default fix is direct Sifr ports that mirror the Python trie algorithms.
- Helper APIs must have focused fixture tests before broad benchmark use, because a helper bug can contaminate many problem results.

Trie port structure decision:

- `0208`, `0211`, and `0212` should each carry the trie structure needed by the Python source in the Sifr problem file or a problem-local helper under `audits/leetcode/benchmarks/cases/<category>` only if multiple fixtures need it.
- Do not introduce a new shared benchmark trie helper during M1. Shared trie optimization belongs to M2 after parity is restored.
- If a shared helper is later used, both Python and Sifr benchmark sides must intentionally use comparable helper semantics.

### D4: Compiler Fixes Must Preserve Safety Semantics

Compiler/runtime work must remove accidental work without weakening Sifr's safety guarantees.

Decisions:

- String indexing fixes use cached `Vec<char>` or equivalent code-point storage for immutable/index-heavy strings. They must not silently switch Python/Sifr character indexing to byte indexing.
- Repeated `len(s)` in a loop is cached only when the string is loop-invariant.
- Container membership, length, and index reads lower through borrowed access whenever the source expression is a read, not a move.
- Mutable list/matrix cell updates lower as place mutations instead of clone-modify-reassign when the ownership model proves the container is uniquely mutable.
- Optional tree/list traversal uses borrowed child accessors where possible; cloning boxed nodes/subtrees in traversal is a bug unless ownership requires it.
- No fix may introduce user-triggerable panics or data-dependent unwraps in generated runtime code.

### D5: Every Fix Needs A Generated-Code Regression Test

Benchmark speedups are necessary but not sufficient. Each compiler/root-cause fix needs a generated-code regression that checks the emitted Rust shape.

Test location and runner:

- Compiler emitted-code assertions live in `crates/sifr_codegen/src/lib_codegen_tests/`, following the existing `generate_rust_from_source` / `generate_rust_with_metadata` pattern.
- Add focused tests near the affected lowering module, or create a dedicated `leetcode_performance_codegen_tests.rs` module if the fixture spans multiple lowering surfaces.
- Run the focused test with `cargo test -p sifr_codegen -- <test_name>`.
- Closure for compiler fixes also runs `scripts/run_all_tests.sh --profile quick`.
- LeetCode parity fixes use benchmark fixtures plus existing stdlib fixtures where relevant:
  - `crates/sifr/tests/e2e/pass/cpython_heapq*.sifr` for heap behavior,
  - `crates/sifr/tests/e2e/pass/*deque*.sifr` for deque behavior,
  - the affected `audits/leetcode` problem fixtures for benchmark parity.

Required negative assertions:

- String hot-loop fixtures should not contain repeated `chars().nth(...)` or loop-invariant `chars().count()`.
- Field reads such as `self.map.len()` and `self.map.contains_key(...)` should not emit `self.map.clone()`.
- Trie/table reads should not clone full `_children`, `_terminal`, maps, or rows for lookup-only paths.
- Tree/list traversal fixtures should not emit subtree/node clones in simple traversal paths.
- Matrix cell update fixtures should not clone a whole row for each cell mutation.

Representative emitted-code contracts:

| Track | Before | After |
| --- | --- | --- |
| C1 string indexing | `s.chars().nth(i as usize)` inside the loop | one loop-scope cached character storage such as `let __s_chars: Vec<char> = s.chars().collect();` followed by indexed reads from `__s_chars` |
| C1 string length | `s.chars().count()` repeated in loop conditions | one loop-invariant cached length derived from the same cached character storage |
| C2 field reads | `(self.map.clone()).contains_key(...)` / `(self.map.clone()).len()` | `self.map.contains_key(...)` / `self.map.len()` borrowed reads |
| C2 container index reads | `self.rows.clone()[i as usize].clone()` for lookup-only paths | borrowed row/item access where the result is not moved |
| C3 optional tree/list traversal | `root.as_deref().cloned()` in traversal/comparison paths | borrowed `root.as_deref()` access with cloning only at ownership boundaries |
| C4 matrix mutation | clone row, mutate clone, assign whole row back per cell | direct mutable place update such as `grid[r as usize][c as usize] = value` when the matrix is uniquely mutable |

### D6: Benchmark Reclassification Is Data-Driven

Manual intuition does not close a fix.

Decisions:

- Re-run the affected subset after each fix, then the full category if the subset passes.
- Run the analyzer after every benchmark run and update metadata from the analyzer output.
- A runtime fix with a Peak RSS regression greater than 10% at the same fixture size remains open with a memory-specific tag unless the PR documents why the memory tradeoff is intentional and bounded.
- A fixed failed problem that becomes benchmarkable may enter the slower inventory and must be handled by this phase.

Benchmark commands:

- Subset: `python3 benchmarks/bench.py run --runs 2 --warmup 1 --memory-runs 1 <problem_id> ...`
- Report: `python3 benchmarks/bench.py report-html`
- Analyzer: `python3 benchmarks/analyze_slowness.py --check-metadata`
- Full closure benchmark uses the same command shape without explicit problem ids.

## Classification Rule

Each slower problem below is assigned a primary owner:

- **Compiler**: same or close algorithm, but emitted Rust/runtime behavior is the main slowness source.
- **LeetCode Sifr code**: algorithm/data structure differs materially from Python or is known-correctness divergent.
- **Mixed**: both are material; fixing only one side may not recover parity.
- **Low-priority/noise**: Sifr is only slightly slower on one fixture, usually near the benchmark noise boundary or only at the largest size after being faster elsewhere.

Ratio convention: `0.25x` means Python/Sifr ratio is 0.25, so Sifr is roughly 4x slower for that fixture.

Partial benchmarks are allowed in this diagnostic inventory only when at least one fixture has complete Python/Sifr timing rows. They must be marked as partial and excluded from apples-to-apples report summaries until every fixture for that problem builds, passes correctness, and produces comparable runtime and memory rows. `0234_palindrome_linked_list` was the only partial measured-slower case and was completed in `sifr-lang/leetcode#31`.

## Every Sifr-Slower Benchmark Result

| Problem | Category | Worst Py/Sifr | Slower sizes | Primary owner | Root cause |
| --- | --- | ---: | --- | --- | --- |
| `1985_find_the_kth_largest_integer_in_the_array` | Heap / Priority Queue | 0.003x | 1k, 5k, 10k | LeetCode Sifr code | Python uses `heapq`; Sifr uses insertion into a sorted list with numeric string comparisons, so this is not apples-to-apples. String length/comparison lowering adds compiler overhead but the dominant issue is algorithmic. |
| `0211_design_add_and_search_words_data_structure` | Tries | 0.003x | 1k, 5k, 10k | Mixed | Python uses dict-backed trie nodes. Sifr uses shared `helpers.trie.Trie`; emitted Rust clones `self.trie`, `_children`, rows, and terminal vectors in add/search paths. |
| `0973_k_closest_points_to_origin` | Heap / Priority Queue | 0.015x | 1k, 5k, 10k | LeetCode Sifr code | Python heapifies once and pops `k`; Sifr repeatedly scans all unused points for each result. |
| `0535_encode_and_decode_tinyurl` | Arrays & Hashing | 0.011x | 1k, 5k, 10k | Compiler | Algorithm is close, but emitted Rust clones hash maps for `contains_key`/`len`, repeatedly formats strings, and object-op runner replays state setup. |
| `1472_design_browser_history` | Linked List | 0.017x | 1k, 5k, 10k | Mixed | Python file contains two class definitions; the benchmark resolves the final list-with-index class. Sifr uses that broad approach, but truncates forward history with a pop loop and emitted Rust clones `history` for length checks. |
| `0208_implement_trie_prefix_tree` | Tries | 0.004x | 1k, 5k, 10k | Mixed | Python uses fixed 26-child nodes. Sifr imports shared vector-of-edge trie; emitted Rust clones the whole `_children`/`_terminal` storage in insert and lookup. |
| `0567_permutation_in_string` | Sliding Window | 0.043x | 1k, 10k, 100k | Mixed | Same sliding-window idea, but Sifr uses `ALPHA` linear search for char-to-index and wrapper functions for list updates; compiler emits expensive string indexing/counting. |
| `2130_maximum_twin_sum_of_a_linked_list` | Linked List | 0.004x | 100, 1k, 5k | Compiler | Same broad algorithm, but list-node helper and generated optional-node access clone linked-list nodes heavily in traversal. |
| `0049_group_anagrams` | Arrays & Hashing | 0.146x | 1k, 5k, 20k | Compiler | Dict/list grouping is close, but emitted Rust clones map/list values around updates and string/list keys. |
| `2306_naming_a_company` | Arrays & Hashing | 0.014x | 100, 250, 500 | LeetCode Sifr code | Python groups suffixes by first letter and intersects sets. Sifr checks every pair and calls `_contains` over the full ideas list, an algorithmic mismatch. |
| `0003_longest_substring_without_repeating_characters` | Sliding Window | 0.017x | 1k, 10k, 100k | Compiler | Same sliding-window family; emitted Rust pays repeated `chars().nth`/`chars().count` and set/string allocation overhead. |
| `0706_design_hashmap` | Arrays & Hashing | 0.109x | 1k, 5k, 10k | Mixed | Sifr class implementation is structurally different from Python and emitted Rust clones backing maps/vectors around method calls. |
| `1888_minimum_number_of_flips_to_make_the_binary_string_alternating` | Sliding Window | 0.016x | 1k, 10k, 100k | Compiler | Python and Sifr are close; slowness comes from string concatenation, repeated indexing, `chars().nth`, and building alternate strings character by character. |
| `0036_valid_sudoku` | Arrays & Hashing | 0.255x | 9 | Compiler | Small fixed input; overhead is dominated by hash/set and safe-index codegen, not asymptotic work. |
| `0014_longest_common_prefix` | Arrays & Hashing | 0.199x | 1k, 10k, 100k | Compiler | Repeated string indexing and length checks lower to character scans and allocations. |
| `1768_merge_strings_alternately` | Two Pointers | 0.016x | 1k, 10k, 100k | Compiler | Same algorithm; emitted Rust performs repeated string indexing and string append/format allocation. |
| `1930_unique_length_3_palindromic_subsequences` | Arrays & Hashing | 0.255x | 1k, 10k, 100k | Compiler | Same high-level work, but string indexing and set/list operations allocate more in emitted Rust. |
| `0187_repeated_dna_sequences` | Arrays & Hashing | 0.024x | 1k, 10k, 100k | Compiler | Substring/set-heavy workload; emitted Rust allocates/clones strings for windows and set operations. |
| `0402_remove_k_digits` | Stack | 0.134x | 1k, 10k, 100k | Compiler | Stack algorithm is close, but Sifr string/list mutation emits extra char/string allocation and clone-heavy list operations. |
| `1631_path_with_minimum_effort` | Advanced Graphs | 0.020x | 30, 60 | LeetCode Sifr code | Python uses Dijkstra with `heapq`; Sifr uses O(V^2) repeated full-grid selection and matrix-cell helpers. |
| `0763_partition_labels` | Greedy | 0.032x | 1k, 10k, 100k | Compiler | Same greedy concept; emitted string indexing and dict access overhead dominate large strings. |
| `0680_valid_palindrome_ii` | Two Pointers | 0.021x | 10k, 100k | Compiler | Two-pointer algorithm is close; string indexing lowers to repeated character scans. |
| `0221_maximal_square` | 2-D Dynamic Programming | 0.176x | 50, 150, 300 | Compiler | Same DP shape; nested matrix indexing/update helpers clone rows/containers and add optional checks. |
| `1461_check_if_a_string_contains_all_binary_codes_of_size_k` | Arrays & Hashing | 0.020x | 10k, 100k | Compiler | Window/string/set workload; emitted Rust allocates substring/string values repeatedly. |
| `0355_design_twitter` | Heap / Priority Queue | 0.255x | 1k, 5k, 10k | Mixed | Python and Sifr object models differ; emitted Rust clones maps/lists around stateful methods. |
| `0424_longest_repeating_character_replacement` | Sliding Window | 0.058x | 1k, 10k, 100k | Compiler | Same sliding-window family; dict/string indexing lowering dominates. |
| `0139_word_break` | 1-D Dynamic Programming | 0.039x | 10k, 50k | Compiler | Same DP class, but substring and word-membership checks allocate/scan aggressively in emitted Rust. |
| `1456_maximum_number_of_vowels_in_a_substring_of_given_length` | Sliding Window | 0.039x | 10k, 100k | Compiler | Same sliding-window idea; repeated char extraction and vowel checks allocate per character. |
| `0721_accounts_merge` | Graphs | 0.300x | 100, 300, 700 | Mixed | Algorithm appears related but map/list/set-heavy union/grouping emits many clones and differs from Python helper style. |
| `0703_kth_largest_element_in_a_stream` | Heap / Priority Queue | 0.142x | 5k, 10k | LeetCode Sifr code | Python commonly uses heap discipline; Sifr stream implementation uses list manipulation and pays extra object-state clones. |
| `0200_number_of_islands` | Graphs | 0.249x | 20, 40, 80 | Compiler | Same grid traversal class; emitted matrix access and set/visited operations clone rows and values. |
| `0149_max_points_on_a_line` | Math & Geometry | 0.453x | 100, 300, 600 | Compiler | Same O(n^2) slope-count idea; dict key/value cloning and tuple/string formatting overhead dominate. |
| `0146_lru_cache` | Linked List | 0.092x | 5k, 10k | Mixed | Python uses O(1) doubly-linked-list pointer surgery. Sifr uses integer-node dictionaries and emitted Rust clones full dict state during detach/insert operations. |
| `2001_number_of_pairs_of_interchangeable_rectangles` | Arrays & Hashing | 0.293x | 100, 1k, 5k | Compiler | Ratio counting is close; emitted dict access/update clones map entries/keys. |
| `0130_surrounded_regions` | Graphs | 0.342x | 20, 40, 80 | Compiler | Same flood-fill class; nested matrix mutation lowers through row/container clone paths. |
| `0058_length_of_last_word` | Arrays & Hashing | 0.033x | 10k, 100k | Compiler | Simple string scan; emitted char indexing/counting path makes the large input slow. |
| `0895_maximum_frequency_stack` | Stack | 0.202x | 5k, 10k | Mixed | Stateful map-of-stacks workload; Sifr object state and dict/list operations clone aggressively. |
| `0392_is_subsequence` | Arrays & Hashing | 0.039x | 10k, 100k | Compiler | Same two-pointer string scan, but emitted string indexing uses repeated character scans. |
| `2013_detect_squares` | Math & Geometry | 0.468x | 100, 500, 1k | Mixed | Stateful map/point counting; emitted HashMap cloning and object-op replay overhead are material. |
| `0205_isomorphic_strings` | Arrays & Hashing | 0.552x | 1k, 10k, 100k | Compiler | Same map-check algorithm; emitted string iteration and dict operations clone. |
| `0778_swim_in_rising_water` | Advanced Graphs | 0.025x | 30, 60 | LeetCode Sifr code | Python uses heap-based best-first search; Sifr uses repeated O(n^4)-ish grid selection over all cells. |
| `0015_3sum` | Two Pointers | 0.074x | 300, 800 | LeetCode Sifr code | Python uses sorted O(n^2) two-pointer search. Sifr uses brute-force triple loops plus deduplication, so the primary issue is algorithmic non-parity. |
| `0067_add_binary` | Bit Manipulation | 0.365x | 10k, 100k | Compiler | Same digit-carry idea; string indexing and result string construction allocate heavily. |
| `1189_maximum_number_of_balloons` | Arrays & Hashing | 0.543x | 1k, 10k, 100k | Compiler | Same count-map idea; emitted map operations clone and string iteration allocates. |
| `0125_valid_palindrome` | Two Pointers | 0.057x | 10k, 100k | Compiler | Same two-pointer scan; repeated string indexing is the root cause. |
| `0929_unique_email_addresses` | Arrays & Hashing | 0.709x | 1k, 10k, 100k | Compiler | Same normalization/set idea; string building and set operations allocate/clone more. |
| `0344_reverse_string` | Two Pointers | 0.459x | 10k, 100k | Compiler | Same in-place reversal; list/string element access and mutation clone more than Python list operations. |
| `0102_binary_tree_level_order_traversal` | Trees | 0.467x | 1k, 5k | Compiler | Same traversal class; generated tree helper clones optional nodes (`as_deref().cloned`) and queue/list values. |
| `0647_palindromic_substrings` | 1-D Dynamic Programming | 0.564x | 300, 800 | Compiler | Same expand-around-center idea; repeated string indexing lowers poorly. |
| `0005_longest_palindromic_substring` | 1-D Dynamic Programming | 0.517x | 300, 800 | Compiler | Same palindrome expansion class; string slicing/indexing and allocation dominate. |
| `0981_time_based_key_value_store` | Binary Search | 0.255x | 5k, 10k | Mixed | Stateful dict/list binary-search workload; emitted HashMap/list clones around object state. |
| `0572_subtree_of_another_tree` | Trees | 0.332x | 1k, 5k | Compiler | Recursive tree comparison/search clones optional tree nodes and subtrees during traversal. |
| `0234_palindrome_linked_list` | Linked List | 0.005x | 100, 1k, 5k | Compiler | Completed in `sifr-lang/leetcode#31`; residual slowness remains linked-list helper/codegen cloning while traversing optional nodes. |
| `2405_optimal_partition_of_string` | Arrays & Hashing | 0.710x | 10k, 100k | Compiler | Same set-partition greedy; string/set operations allocate more. |
| `0100_same_tree` | Trees | 0.351x | 1k, 5k | Compiler | Recursive optional tree handling clones nodes/subtrees. |
| `0072_edit_distance` | 2-D Dynamic Programming | 0.928x | 100, 200 | Low-priority/noise | Near parity; string indexing/counting overhead keeps Sifr slightly behind. |
| `0054_spiral_matrix` | Math & Geometry | 0.604x | 300, 700 | Compiler | Same matrix traversal; nested list indexing clones rows/values. |
| `0020_valid_parentheses` | Stack | 0.981x | 10k, 100k | Low-priority/noise | Essentially parity; small overhead from dict/list/loop lowering. |
| `1834_single_threaded_cpu` | Heap / Priority Queue | 0.520x | 300, 700 | LeetCode Sifr code | Python uses heap-based scheduling, while Sifr scans for the minimum available task. Treat as an O(n log n) versus O(n^2) parity gap. |
| `0189_rotate_array` | Two Pointers | 0.548x | 5k, 10k | Compiler | Same rotation class; list slicing/index mutation emits extra vector copies. |
| `0682_baseball_game` | Stack | 0.915x | 100k | Low-priority/noise | Slightly behind at largest size; stack/string parse overhead. |
| `0606_construct_string_from_binary_tree` | Trees | 0.801x | 1k, 5k | Compiler | Tree recursion plus string construction clones nodes and allocates strings. |
| `0064_minimum_path_sum` | 2-D Dynamic Programming | 0.982x | 300 | Low-priority/noise | Near parity; matrix indexing overhead only. |
| `0752_open_the_lock` | Graphs | 0.434x | 100 | Mixed | BFS/string-neighbor generation uses many string/index operations and set checks; emitted code shows high clone/to_string counts. |
| `0179_largest_number` | Arrays & Hashing | 0.182x | 500, 1k | Mixed | Comparator/string-concat workload; check algorithm parity and then fix string comparison/concat lowering. |
| `0295_find_median_from_data_stream` | Heap / Priority Queue | 0.510x | 10k | Mixed | Stateful stream workload; likely data-structure parity plus object-state clone overhead. |
| `0094_binary_tree_inorder_traversal` | Trees | 0.885x | 5k | Compiler | Tree helper clones optional nodes; mild regression. |
| `0380_insert_delete_getrandom_o1` | Arrays & Hashing | 0.087x | 10k, 100k | Mixed | Stateful dict/list/randomized-set workload; emitted HashMap/list clones and possible implementation parity gaps. |
| `1396_design_underground_system` | Arrays & Hashing | 0.319x | 100k | Mixed | Stateful maps and string keys; emitted HashMap clones are material, and fixture expected checksums were regenerated during benchmark setup. |
| `0104_maximum_depth_of_binary_tree` | Trees | 0.902x | 5k | Compiler | Recursive optional tree traversal clones nodes but remains close. |
| `0199_binary_tree_right_side_view` | Trees | 0.920x | 5k | Compiler | Tree/list traversal overhead; close to parity. |
| `0013_roman_to_integer` | Math & Geometry | 0.724x | 5k | Compiler | Same map/string scan class; string indexing and map lookup overhead. |
| `0496_next_greater_element_i` | Arrays & Hashing | 0.029x | 10k, 100k | LeetCode Sifr code | Python uses a stack plus index map for O(n + m). Sifr scans `nums2` for each `nums1` target and then scans forward again, so the largest fixture is algorithmically non-parity. |
| `0239_sliding_window_maximum` | Sliding Window | 0.404x | 50k | LeetCode Sifr code | Python uses a deque/monotonic queue O(n) approach. Sifr is brute-force O(n*k) over each window. |
| `1046_last_stone_weight` | Heap / Priority Queue | 0.784x | 10k | LeetCode Sifr code | Python uses `heapq`; Sifr repeatedly sorts or linearly selects stones. This is data-structure non-parity even though the measured gap is modest. |

## Former Failed Tries Case Now Counted as Slower

`0212_word_search_ii` did not originally produce benchmark rows because Sifr failed correctness at the largest fixture. PR `sifr-lang/leetcode#19` replaced the shared trie helper with a problem-local dict-backed trie arena, added `refs`/`removeWord` pruning parity, and refreshed benchmark rows for all three sizes.

Observed failure:

```text
wrong result: ["ab", "bc", ..., "de"]
```

Root cause:

- Python uses a set of results and trie reference pruning.
- Sifr writes into `found: dict[str, bool]`, but then appends once for every input word that is present.
- The generated large fixture repeats two-letter words after the 26-letter cycle, so Sifr returns duplicates while expected output counts unique found words.

Current owner: **Mixed**. The LeetCode parity/correctness issue is fixed; remaining slowness is attributed to trie/dict/field clone and recursive-search lowering, with Sifr using substantially less memory in the refreshed subset run.

## Compiler Work Track

The compiler-track claims below are grounded in emitted Rust samples from the generated benchmark runners. Representative examples:

### Representative Emitted Rust Evidence

String indexing/counting evidence from `1985_find_the_kth_largest_integer_in_the_array`:

```rust
if ((a.chars().count() as i64) != (b.chars().count() as i64)) {
    return ((a.chars().count() as i64) < (b.chars().count() as i64));
}
let current: String = nums_sorted[pos as usize].clone();
```

HashMap/class-field clone evidence from `0535_encode_and_decode_tinyurl`:

```rust
if !((self.encodeMap.clone()).contains_key((longUrl).as_str())) {
    let short_for_encode: String = format!(
        "{}{}",
        self.base.clone(),
        format!("{}", (self.encodeMap.clone().len() as i64) + (1_i64)),
    );
    self.encodeMap.insert(longUrl.clone(), short_for_encode);
}
```

Trie full-structure clone evidence from `0208_implement_trie_prefix_tree` / `0211_design_add_and_search_words_data_structure`:

```rust
let mut children: Vec<Vec<(String, i64)>> = self._children.clone();
let mut terminal: Vec<bool> = self._terminal.clone();
if (node < 0_i64) || (node >= (self._children.clone().len() as i64)) {
    return None;
}
let row: Option<Vec<(String, i64)>> = Some(self._children.clone()[node as usize].clone());
```

Object-operation runner allocation evidence:

```rust
let lines: Vec<String> = fixture_text
    .split(&"\n".to_string())
    .map(|s| s.to_string())
    .collect::<Vec<String>>();
let parts: Vec<String> = line
    .split_whitespace()
    .map(|s| s.to_string())
    .collect::<Vec<String>>();
```

### C1: String Indexing And Character Iteration

Problem families:

- `0003`, `0014`, `0058`, `0067`, `0125`, `0187`, `0205`, `0392`, `0402`, `0424`, `0567`, `0647`, `0680`, `0763`, `0929`, `1189`, `1456`, `1461`, `1768`, `1888`, `1930`, `2405`

Current emitted-code symptoms:

- `s.chars().nth(i)` in loops
- `s.chars().count()` used as `len(s)` in loops
- `ch.to_string()` allocation per character
- repeated string `+` / `format!` allocation in result-building loops

Important distinction:

- Some problems are inherently string-index-heavy (`0392`, `0058`, `1768`, `1888`), so compiler string lowering is the primary fix.
- Some problems choose a more string-heavy Sifr approach than the Python implementation. These need a LeetCode-code parity check before evaluating compiler fixes, especially `0402`, `0567`, `0179`, `0187`, `0929`, and `1888`.

Required compiler/runtime direction:

- Introduce an indexed string view or cached character vector lowering for index-heavy strings.
- Lower repeated `len(s)` to a cached length when the string is loop-invariant.
- Avoid per-character `String` allocation when a `char` or borrowed slice is sufficient.
- Add generated-code regression tests for representative LeetCode strings: `0392`, `0058`, `1768`, `1888`.

### C2: Collection And Class Field Clone Elision

Problem families:

- dict/set/map: `0049`, `0205`, `0535`, `0706`, `0895`, `0981`, `1189`, `1396`, `2001`, `2013`
- result/dedup dictionaries and tuple-key maps: `0015`, `0149`
- object state: `0146`, `0355`, `0380`, `1472`
- trie helper: `0208`, `0211`

Current emitted-code symptoms:

- `self.field.clone().len()`
- `self.map.clone().contains_key(...)`
- `self._children.clone()[node].clone()`
- local aliasing in Sifr turning into full Rust clones

Required compiler/runtime direction:

- Distinguish borrowed field reads from owned field moves in HIR/codegen.
- Lower map/set membership and length queries through borrowed access.
- Avoid cloning full containers for index reads when the result can be borrowed or copied.
- Add generated-code assertions that hot methods do not contain full-field clones.

### C3: Tree/List Optional Node Traversal

Problem families:

- linked list: `0234`, `2130`
- tree: `0094`, `0100`, `0102`, `0104`, `0199`, `0572`, `0606`

Current emitted-code symptoms:

- optional helper access uses node clones during traversal,
- recursive calls pass cloned subtrees/nodes,
- traversal queues/lists clone node payloads.

Required compiler/runtime direction:

- Improve recursive object/boxed-node borrowing.
- Add borrow-preserving accessors for `ListNode` and `TreeNode` helper surfaces.
- Lock emitted-code tests for traversal that should borrow child links instead of cloning subtrees.

### C4: Matrix/List Cell Mutation

Problem families:

- graph/DP matrix cases: `0130`, `0200`, `0221`, `0054`, `0064`

Current emitted-code symptoms:

- helper reads pull optional rows by value,
- `setCell` patterns rewrite whole rows,
- nested loops multiply row clone overhead.

Required compiler/runtime direction:

- Support mutable indexed place updates without extracting/cloning the row value.
- Add codegen tests for `matrix[r][c] = value` and row borrowing.

## LeetCode Sifr Code Work Track

### L1: Heap/Priority Queue Parity

Problem families:

- `1985`, `0973`, `0703`, `1046`, `1834`, `0295`, `1631`, `0778`

Current issue:

- Python implementations often use `heapq`.
- Sifr versions frequently use repeated scans, insertion into sorted lists, or O(V^2) Dijkstra variants.

Complexity examples:

- `1985`: Python uses heapify/pop (`O(n + k log n)` after integer conversion); Sifr inserts strings into a sorted vector (`O(n^2)` comparisons plus string-length/string comparison overhead).
- `0973`: Python heapifies points and pops `k` (`O(n + k log n)`); Sifr repeatedly scans all unused points (`O(n*k)`).
- `1046`: Python uses repeated heap pop/push (`O(n log n)`); Sifr sorts or selects repeatedly, making the hot path at least quadratic over the number of stones.
- `1631`: Python uses heap Dijkstra (`O(mn log mn)`); Sifr scans the whole grid for the next best cell on every step (`O((mn)^2)`).
- `0778`: Python uses heap best-first search (`O(n^2 log n)`); Sifr performs repeated all-cell selection (`O(n^4)` shape).
- `1834`: Python uses heap scheduling (`O(n log n)`); Sifr scans remaining tasks for the next executable task (`O(n^2)`).

Required direction:

- Add or use a Sifr heap/priority-queue helper for LeetCode audits.
- Port Python heap algorithms directly once a helper exists.
- Re-run those problems before attributing residual slowness to the compiler.

### L2: Trie Parity

Problem families:

- `0208`, `0211`, `0212`

Current issue:

- Python uses problem-specific trie nodes.
- Sifr uses a shared helper with a different representation.
- `0212` was originally correctness-divergent on duplicate words; `sifr-lang/leetcode#19` restored correctness and left only residual mixed trie slowness.

Required direction:

- Port the Python trie node algorithms directly to Sifr for `0208`, `0211`, and `0212` unless the benchmark intentionally changes both languages to the same shared-helper representation.
- Fix `0212` uniqueness semantics.
- After parity is restored, address compiler clone-elision in `helpers.trie`.

### L3: Algorithmic Divergence In Hand Ports

Problem families:

- `2306`: Sifr pairwise/full-list contains versus Python grouped suffix sets.
- `1472`: Sifr pop-loop history truncation versus Python logical-length overwrite.
- `0239`: verify Sifr uses monotonic queue like Python/deque.
- `0380`, `1396`, `0355`, `0146`: verify stateful data-structure parity before compiler-only work.

Required direction:

- For every benchmarked problem, add a parity note in the problem registry or case file:
  - `same_algorithm: true`
  - `known_divergence: heap_missing`, `trie_helper`, `stateful_helper`, etc.
- Report UI should distinguish "Sifr slower on equivalent implementation" from "Sifr slower on known-divergent implementation."
- Code parity changes must land before compiler attribution changes for the same problem unless emitted Rust already proves a compiler-only issue.

## Benchmark/Report Contract Updates

The report should not flatten all slower examples into one claim.

Required report metadata:

- Complete/runnable count.
- Failed correctness/build count.
- A flag for known implementation divergence.
- A flag for known compiler/codegen slowness class.
- Runtime comparisons only for complete, correctness-passing fixtures.

Required raw-analysis command:

```bash
python3 benchmarks/bench.py report-html
python3 benchmarks/analyze_slowness.py --output issues/ad-hoc-leetcode-benchmark-slowness-root-cause-analysis.md
```

The analyzer was added by the predecessor reporting phase and is now the authoritative way to refresh this phase's generated snapshot. If analyzer output and hand-written phase text disagree, treat the analyzer output as the benchmark-data source and update the hand-written implementation plan accordingly.

Concrete metadata path:

- Add optional fields to each entry in `audits/leetcode/benchmarks/problems/*.json`:
  - `parity_status`: `equivalent`, `known_divergent`, `unknown`, or `failed_correctness`
  - `primary_slowness_owner`: `compiler`, `leetcode_sifr_code`, `mixed`, or `noise`
  - `slowness_tags`: list of stable tags such as `string_indexing`, `heap_missing`, `trie_helper`, `field_clone`, `matrix_clone`, `tree_clone`, `stateful_object`
  - `benchmark_status`: `complete`, `partial`, `failed_build`, `failed_correctness`, or `failed_timeout`
- Report generation should read these fields instead of hard-coding classifications.
- Missing metadata should render as `unknown`, never as equivalent.
- M0 must seed these fields for all 75 measured-slower problems and all 53 incomplete/failed problems before M4 report UI work starts.

Initial seeding rules:

- Measured-slower table rows map directly to `primary_slowness_owner` and `slowness_tags`.
- Rows marked **Compiler** should start with `parity_status: "equivalent"` only when the Sifr and Python algorithms are close enough to defend that claim; otherwise use `unknown`.
- Rows marked **LeetCode Sifr code** should start with `parity_status: "known_divergent"`.
- Rows marked **Mixed** should start with `parity_status: "unknown"` unless the divergence is already known, such as trie helper or heap-helper mismatch.
- Low-priority/noise rows should still get metadata so the report can avoid silently reclassifying them.
- `0234_palindrome_linked_list` used `benchmark_status: "partial"` until all fixtures passed; `sifr-lang/leetcode#31` marks it complete after rerunning the missing size-100 pair.

## Post-Fix Re-Benchmark Protocol

Every implementation ticket cut from this phase needs a before/after benchmark record.

Required flow after a fix:

1. Re-run only the affected problem subset first.
2. Regenerate raw results and the HTML report.
3. Run `benchmarks/analyze_slowness.py` to refresh measured-slower, partial, and failed inventories.
4. Update registry metadata only from the refreshed analyzer output plus a human parity review.
5. Re-run the full category after a subset looks fixed, because helper/compiler changes often affect neighboring problems.

Reclassification thresholds:

- `python.mean / sifr.mean >= 1.0`: Sifr is faster or equal; remove the problem from measured-slower summaries unless memory regresses.
- `0.8 <= python.mean / sifr.mean < 1.0`: near parity; classify as `noise` unless the regression is consistent across all sizes and has a clear root cause.
- `python.mean / sifr.mean < 0.8`: still materially slower; keep the current owner or reclassify based on the new emitted code and parity review.
- Any correctness/build failure after a fix overrides runtime data and sets `benchmark_status` to the corresponding failed state.

Memory must be rechecked with the same status discipline. A runtime win with a Peak RSS regression greater than 10% at the same fixture size is not considered fully fixed unless the PR explicitly documents an intentional and bounded memory tradeoff; otherwise it remains a benchmark concern with a memory-specific tag.

## Failed-To-Benchmarkable Conversion

The incomplete appendix is not a dead-end list. Fixing compiler type errors, moved-value failures, build errors, or correctness mismatches will move problems into the measured benchmark pool.

When a failed problem becomes correctness-passing:

- mark `benchmark_status: "complete"` only after every configured fixture has Python and Sifr runtime plus memory rows,
- run the analyzer to determine whether it joins the measured-slower table,
- seed `parity_status`, `primary_slowness_owner`, and `slowness_tags` before showing it as apples-to-apples in the report,
- treat newly slower cases as follow-up scope for this phase, not as unrelated regressions.

## Incomplete And Failed Problem Appendix

These rows are the historical baseline failure appendix that drove the fix phase. The generated analyzer snapshot below is authoritative for current state after `sifr-lang/leetcode#31`: 0 partial benchmarks and 0 no-pair failed problems.

| Problem | Failure mode | Representative excerpt |
| --- | --- | --- |
| `0739_daily_temperatures` | type error | `cannot index type 'Any | None' with 'int'` |
| `0853_car_fleet` | type error | `exact integer to float conversion requires handling possible overflow or precision loss` |
| `1209_remove_all_adjacent_duplicates_in_string_ii` | type error | `cannot compare 'Result[int, DivisionError]' and 'int' with !=` |
| `0084_largest_rectangle_in_histogram` | type error | `cannot index type 'Any | None' with 'int'` |
| `0441_arranging_coins` | type error | `exact integer to float conversion requires handling possible overflow or precision loss` |
| `0875_koko_eating_bananas` | type error | `unsupported operand type(s) for +: 'int' and 'Result[int, DivisionError]'` |
| `0206_reverse_linked_list` | type error | `use of moved value: 'result'` |
| `0021_merge_two_sorted_lists` | type error | `use of moved value: 'result'` |
| `0234_palindrome_linked_list` | partial/type error | complete pairs exist for some sizes; missing size fails with `expected 'ListNode', got 'None | ListNode'` |
| `0203_remove_linked_list_elements` | type error | `use of moved value: 'result'` |
| `0083_remove_duplicates_from_sorted_list` | type error | `use of moved value: 'result'` |
| `0876_middle_of_the_linked_list` | type error | `use of moved value: 'result'` |
| `0019_remove_nth_node_from_end_of_list` | type error | `use of moved value: 'result'` |
| `1721_swapping_nodes_in_a_linked_list` | type error | `use of moved value: 'result'` |
| `0002_add_two_numbers` | type error | `use of moved value: 'result'` |
| `0141_linked_list_cycle` | type error | `expected 'ListNode', got 'None | ListNode'` |
| `0024_swap_nodes_in_pairs` | type error | `use of moved value: 'result'` |
| `0148_sort_list` | type error | `use of moved value: 'result'` |
| `0086_partition_list` | type error | `use of moved value: 'result'` |
| `0061_rotate_list` | type error | `use of moved value: 'result'` |
| `0147_insertion_sort_list` | type error | `use of moved value: 'result'` |
| `0025_reverse_nodes_in_k_group` | type error | `use of moved value: 'result'` |
| `0707_design_linked_list` | timeout/terminated | Sifr process was terminated after a long correctness/measurement run |
| `0622_design_circular_queue` | type error | `cannot index type 'list[int]' with 'Result[int, DivisionError]'` |
| `0144_binary_tree_preorder_traversal` | type error | `use of moved value: 'root'` |
| `0145_binary_tree_postorder_traversal` | type error | `use of moved value: 'root'` |
| `0226_invert_binary_tree` | build/runtime error | `build error: cargo build failed` |
| `0108_convert_sorted_array_to_binary_search_tree` | build/runtime error | `build error: cargo build failed` |
| `0617_merge_two_binary_trees` | type error | `use of moved value: 'p'` |
| `0701_insert_into_a_binary_search_tree` | type error | `use of moved value: 'root'` |
| `0450_delete_node_in_a_bst` | type error | `use of moved value: 'root'` |
| `0103_binary_tree_zigzag_level_order_traversal` | type error | `use of moved value: 'root'` |
| `0106_construct_binary_tree_from_inorder_and_postorder_traversal` | build/runtime error | `build error: cargo build failed` |
| `0662_maximum_width_of_binary_tree` | type error | `use of moved value: 'root'` |
| `1448_count_good_nodes_in_binary_tree` | type error | `expected 'TreeNode', got 'None | TreeNode'` |
| `0230_kth_smallest_element_in_a_bst` | type error | `expected 'TreeNode', got 'None | TreeNode'` |
| `0105_construct_binary_tree_from_preorder_and_inorder_traversal` | build/runtime error | `build error: cargo build failed` |
| `0513_find_bottom_left_tree_value` | type error | `use of moved value: 'root'` |
| `0669_trim_a_binary_search_tree` | type error | `use of moved value: 'root'` |
| `0212_word_search_ii` | correctness | duplicate words in Sifr result for large fixture |
| `1383_maximum_performance_of_a_team` | type error | `return type mismatch: expected 'int', got 'Result[int, DivisionError]'` |
| `0502_ipo` | type error | `'<=' not supported between instances of 'Result[int, DivisionError]' and 'int'` |
| `0698_partition_to_k_equal_sum_subsets` | type error | `cannot compare 'Result[int, DivisionError]' and 'int' with !=` |
| `0909_snakes_and_ladders` | type error | `return type mismatch: expected 'list[int]', got 'list[Result[int, DivisionError]]'` |
| `0743_network_delay_time` | type error | `use of moved value: 'w1'` |
| `0269_alien_dictionary` | correctness | Sifr returned `abc` for a fixture whose expected output differs |
| `0062_unique_paths` | type error | `cannot assign 'Result[int, DivisionError]' to variable 'result' of type 'int'` |
| `1220_count_vowels_permutation` | type error | `cannot assign 'Result[int, DivisionError]' to variable 'a' of type 'int'` |
| `0846_hand_of_straights` | type error | `cannot compare 'Result[int, DivisionError]' and 'int' with !=` |
| `0263_ugly_number` | type error | `cannot compare 'Result[int, DivisionError]' and 'int' with ==` |
| `1260_shift_2d_grid` | type error | `return type mismatch: expected 'list[int]', got 'list[Result[int, DivisionError]]'` |
| `0006_zigzag_conversion` | type error | `use of moved value: 's'` |
| `0007_reverse_integer` | type error | `exact integer to float conversion requires handling possible overflow or precision loss` |

## Milestones

Dependency order: **M0 -> M1 -> M2/M3 -> M4 -> M5**. M2 and M3 can proceed in parallel only for disjoint problem families. Any problem marked `known_divergent` must go through M1 before compiler performance work is credited as a fix.

Track ownership matrix:

| Track | Primary scope | Must wait for |
| --- | --- | --- |
| M1 heap/deque/trie/direct parity | `1985`, `0973`, `0703`, `1046`, `1834`, `0295`, `1631`, `0778`, `0208`, `0211`, `0212`, `0015`, `0239`, `0496`, `2306`, `0146`, `0355`, `0380`, `1396`, `1472` | baseline metadata only |
| M2 string/container/stateful codegen | C1/C2 families after any known-divergent row is ported | M1 for overlapping known-divergent or mixed rows |
| M3 recursive/matrix codegen | C3/C4 tree/list/grid families | M1 only for overlapping mixed rows |
| M4 report/analyzer gates | all benchmark metadata and report summaries | M1/M2/M3 metadata updates |
| M5 closure | full suite and residual tickets | M1-M4 |

### M0: Baseline Lock And Ticket Slicing

- Confirm the generated analyzer snapshot still matches the raw benchmark results before implementation starts.
- Cut implementation tickets from the decisions in this phase:
  - LeetCode Sifr parity repairs,
  - compiler string lowering,
  - compiler container/field clone elision,
  - tree/list optional traversal borrowing,
  - matrix/list cell mutation lowering,
  - report/analyzer reclassification gates.
- Each ticket must name the affected problem set, expected emitted-code change, benchmark subset, and acceptance gate.
- Do not start broad compiler work from a known-divergent benchmark row until the parity ticket for that row is complete.

### M1: LeetCode Sifr Parity Repairs

- Add Sifr heap/priority-queue support or use an existing equivalent helper.
- Port heap/priority-queue problems to match Python algorithms: `1985`, `0973`, `0703`, `1046`, `1834`, `0295`, `1631`, `0778`.
- Port trie problems to parity with Python problem-local trie algorithms: `0208`, `0211`, `0212`.
- Fix `0212_word_search_ii` duplicate-result semantics.
- Fix direct hand-port divergences: `0015`, `0239`, `0496`, `2306`.
- Review and repair stateful problem parity before compiler attribution: `0146`, `0355`, `0380`, `1396`, `1472`.
- Re-run each repaired subset and update registry metadata from the analyzer.

Completed M1 waves:

- `sifr-lang/leetcode#15`: ported `1985`, `0973`, `0703`, `1046`, `1834`, `1631`, and `0778` to heap-backed Sifr implementations, refreshed registry metadata to `parity_status: "equivalent"`, and reduced the measured-slower inventory from 75 to 68. Local gates: targeted correctness and benchmark subset, `python3 benchmarks/analyze_slowness.py --check-metadata`, `git diff --check`, file-size guardrail, agent review `reviews/leetcode-heap-parity-m1a-review-pass-1.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/skew advisories only).
- `sifr-lang/leetcode#16`: completed residual heap/stateful parity classification for `0355_design_twitter` and `0295_find_median_from_data_stream`, updated `benchmarks/slowness_seed.py` so M1 heap rows remain `heap_parity`, and left `0355`/`0295` visible as measured-slower `mixed` + `equivalent` rows for later stateful/codegen work. Local gates: `0355` direct run, targeted correctness for `0355` and `0295`, targeted benchmark subset, analyzer metadata check, seed Python compile, `git diff --check`, file-size guardrail, agent reviews `reviews/leetcode-heap-stateful-parity-m1b-review-pass-1.md` and `reviews/leetcode-heap-stateful-parity-m1b-review-pass-2.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/skew advisories only).
- `sifr-lang/leetcode#17`: replaced shared `helpers.trie.Trie` usage in `0208_implement_trie_prefix_tree` and `0211_design_add_and_search_words_data_structure` with problem-local dict-backed trie arenas, regenerated object fixtures with explicit `__init__` operations so Python and Sifr runners consume matching streams, and reclassified both rows as `mixed` + `equivalent` residual trie/dict/field-clone cases. Local gates: direct Sifr runs for both problems, targeted correctness for all trie fixture sizes, targeted benchmark subset, analyzer metadata check, seed and generator Python compile, `git diff --check`, file-size guardrail, agent review `reviews/leetcode-trie-parity-m1c-review-pass-1.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/skew advisories only).
- `sifr-lang/leetcode#18`: ported the direct hand-divergence rows `0015_3sum`, `0239_sliding_window_maximum`, `0496_next_greater_element_i`, and `2306_naming_a_company` to the Python benchmark algorithms/data structures, updated registry metadata to `parity_status: "equivalent"`, and removed those rows from the measured-slower analyzer output in the refreshed subset run. Local gates: direct Sifr runs for all four problems, targeted fixture correctness, targeted benchmark subset showing Sifr faster with lower memory for all four, analyzer metadata check, seed Python compile, `git diff --check`, file-size and HIR guardrails, agent review `reviews/leetcode-direct-parity-m1d-review-pass-1.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/skew advisories only).
- `sifr-lang/leetcode#19`: completed the remaining trie/failure crossover by moving `0212_word_search_ii` from failed correctness to a complete `mixed` + `equivalent` measured-slower row, using a problem-local dict-backed trie with terminal/ref-count pruning parity. Local gates: direct Sifr run, targeted fixture correctness, targeted benchmark subset, analyzer metadata check, failed-inventory consistency check, seed Python compile, `git diff --check`, file-size guardrail, agent review `reviews/leetcode-word-search-trie-m1e-review-pass-1.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; cache/wall-time/skew advisories only).
- `sifr-lang/leetcode#20`: completed the remaining M1 stateful parity review for `1472_design_browser_history`, `0380_insert_delete_getrandom_o1`, `1396_design_underground_system`, and `0146_lru_cache`; fixed the `1472` forward-history overwrite and `0380` indexed helper paths, replaced `1396` hardcoded station codes with generic length-prefixed route keys, and marked all four rows `mixed` + `equivalent` for later stateful/codegen work. Local gates: direct Sifr runs for all four problems, regenerated fixtures, targeted correctness and benchmark subset, post-key-change `1396` rerun, analyzer metadata check, seed Python compile, JSON validation, `git diff --check`, file-size and HIR guardrails, agent review `reviews/leetcode-stateful-parity-m1f-review-pass-1.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/cache/skew advisories only).

### M2: High-Impact Compiler Codegen Repairs

- Fix string index/iteration lowering for equivalent string-heavy problems.
- Fix collection and class-field clone elision for dictionary, set, list, trie, and object-state reads.
- Add generated-code regression tests for every fixed lowering pattern.
- Re-run affected string/container/stateful benchmark subsets and update metadata.

Completed M2 waves:

- `sifr-lang/sifr#2208` and `sifr-lang/leetcode#21`: completed attribute-list mutation and stateful list clone removal for `1472_design_browser_history`. Codegen now handles `self.field[index] = value` in structured statement bodies by lowering attribute-list subscript assignment to bounded `get_mut`, and direct `self.field` read receivers no longer clone for read-only method calls or borrowed helper arguments. The Sifr source now uses direct `self.history[self.i + 1] = str(url)` instead of copying `history` and assigning it back. Targeted benchmark with the rebuilt compiler shows Sifr faster than Python at every `1472` size (`~4.8x` to `~5.2x`), removing `1472` from the measured-slower table and reducing measured-slower problems from 65 to 64. Local gates: focused `sifr_codegen` regression tests, `cargo build -p sifr`, generated-runner emit check, direct Sifr run, targeted `1472` benchmark with `SIFR_BIN=target/debug/sifr`, analyzer refresh, metadata Python compile, JSON validation, `cargo fmt --check`, `git diff --check`, HIR guardrail, file-size guardrail, agent reviews `reviews/leetcode-attribute-list-mutation-m2-review-pass-1.md` and `reviews/leetcode-attribute-list-mutation-m2-review-pass-2.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/cache/skew advisories only).
- `sifr-lang/leetcode#22`: completed the tuple route-key parity cleanup for `1396_design_underground_system`. The Sifr implementation now mirrors the Python tuple-key route representation with `dict[tuple[str, str], list[int]]` instead of constructing length-prefixed string route keys in the hot path. Targeted benchmark with the rebuilt compiler shows Sifr faster than Python at every `1396` size (`~3.6x` to `~4.5x`), removing `1396` from the measured-slower table and reducing measured-slower problems from 64 to 63. Local gates: direct Sifr run, generated-code emit check showing `HashMap<(String, String), Vec<i64>>`, targeted `1396` benchmark with `SIFR_BIN=target/debug/sifr`, analyzer refresh, metadata Python compile, JSON validation, `git diff --check`, and agent review `reviews/leetcode-underground-tuple-route-m2-review-pass-1.md`.
- `sifr-lang/leetcode#23`: completed trie direct-state cleanup for `0208_implement_trie_prefix_tree` and `0211_design_add_and_search_words_data_structure`. Insert paths now mutate `self.edges` and `self.end` directly instead of cloning full trie state into local aliases; `0211` wildcard search also iterates `row.values()` directly instead of allocating a child list per wildcard. This does not reduce the measured-slower count, but it removes the stale `field_clone`/`stateful_object` attribution: `0208` is now a small residual/noise row, while `0211` remains a `mixed` + `equivalent` recursive-search/dict-iteration row. Local gates: direct Sifr runs, generated-code emit checks showing direct `self.edges.get_mut` / `self.end.get_mut`, targeted trie benchmark subset with `SIFR_BIN=target/debug/sifr`, analyzer refresh, metadata Python compile, JSON validation, `git diff --check`, and agent review `reviews/leetcode-trie-direct-state-m2-review-pass-1.md`.
- `sifr-lang/leetcode#24`: completed TinyURL encode-map cleanup for `0535_encode_and_decode_tinyurl`. The Sifr implementation now uses a `get`/default early return instead of membership/indexing that generated whole-map `contains_key` clones, computes the short URL once, and drops the unused tree import. Targeted benchmark with the rebuilt compiler shows Sifr faster than Python at every `0535` size (`~4.5x` to `~5.3x`), removing `0535` from the measured-slower table and reducing measured-slower problems from 63 to 62. Local gates: direct Sifr run, generated-code emit check confirming the map-clone membership path is gone, targeted `0535` benchmark with `SIFR_BIN=target/debug/sifr`, analyzer refresh, metadata Python compile, JSON validation, `git diff --check`, and agent review `reviews/leetcode-tinyurl-encode-map-m2-review-pass-1.md`.

### M3: Recursive And Matrix Compiler Repairs

- Fix tree/list optional traversal borrow preservation.
- Fix moved-value and optional-node failure classes that currently block linked-list/tree benchmarks where feasible in this phase.
- Fix matrix/list cell mutation lowering.
- Add generated-code regression tests for traversal and matrix mutation.
- Re-run affected tree/list/graph/DP benchmark subsets and update metadata.

### M4: Benchmark Report And Analyzer Enforcement

- Ensure report summaries include only `complete` + `equivalent` comparisons by default.
- Ensure known-divergent, unknown, partial, and failed cases remain visible as coverage/work items.
- Add or maintain UI badges and filters for:
  - equivalent implementation,
  - known divergent Sifr code,
  - suspected compiler/codegen bottleneck,
  - failed correctness/build,
  - partial benchmark.
- Make analyzer checks fail when a problem is missing required metadata after it becomes benchmarkable.
- Add category summaries that ignore known-divergent solutions unless explicitly requested.

Completed M4 waves:

- `sifr-lang/leetcode#25`: reintegrated the safe-math formerly-failed benchmark family by rerunning 16 rows through correctness, runtime, and memory measurement. Fifteen rows now benchmark as complete/equivalent and faster than Python, so they were removed from the failed inventory: `0853`, `0441`, `0875`, `0622`, `1383`, `0502`, `0698`, `0909`, `0743`, `0062`, `1220`, `0846`, `0263`, `1260`, and `0007`. `1209_remove_all_adjacent_duplicates_in_string_ii` was rewritten to stack-parity Sifr and moved from failed-build metadata into the complete/equivalent measured-slower table with residual `compiler` tags `string_allocation` and `stack_clone`; it is faster at `1k`/`10k` but remains slower at `100k`. The refreshed analyzer state is 290 fully complete problems, 868 complete fixture pairs, 63 measured-slower problems, and 34 no-pair failures. Local gates: targeted safe-math batch benchmark with `SIFR_BIN=target/debug/sifr`, focused `1209` direct run, generated-code emit check, focused `1209` benchmark rerun, analyzer metadata check, metadata Python compile, full registry JSON parse, `git diff --check`, HIR guardrail, file-size guardrail, and agent review `reviews/leetcode-safe-math-reintegration-m4-review-pass-1.md`.
- `sifr-lang/leetcode#26`: reintegrated the typed-stack/string-move rows `0739_daily_temperatures`, `0084_largest_rectangle_in_histogram`, and `0006_zigzag_conversion`. Existing Sifr source fixes already produced complete/equivalent benchmarks, and targeted runs show Sifr faster than Python at all configured sizes, so the rows were removed from failed inventory without adding slowness metadata. The refreshed analyzer state is 293 fully complete problems, 877 complete fixture pairs, 63 measured-slower problems, and 31 no-pair failures. Local gates: targeted benchmark for all three rows with `SIFR_BIN=target/debug/sifr`, analyzer metadata check, metadata Python compile, registry JSON validation, `git diff --check`, and agent review `reviews/leetcode-typed-stack-string-reintegration-m4-review-pass-1.md`.
- `sifr-lang/sifr#2215` and `sifr-lang/leetcode#27`: fixed owned recursive optional field lowering so linked-list child reads move boxed children instead of cloning tails, while borrowed helper reads still clone. `0206_reverse_linked_list` now emits `cur.next.map(|...| *...)` and `Some(cur)`, and `nodeNext` still emits `as_deref().cloned()`. The refreshed `0206` benchmark is complete/equivalent and faster than Python at every configured size (`~5.19x`, `~3.68x`, and `~3.95x`), reducing no-pair failures from 31 to 30 and raising fully complete problems from 293 to 294. Local gates: focused codegen tests, direct Sifr run, targeted `0206` benchmark with `SIFR_BIN=target/debug/sifr`, analyzer metadata check, metadata Python compile, `cargo fmt --check`, `git diff --check`, HIR and file-size guardrails, agent review `reviews/leetcode-recursive-option-move-m4-review-pass-1.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/cache/skew advisories only).
- `sifr-lang/leetcode#28`: reran `0002_add_two_numbers` and `0019_remove_nth_node_from_end_of_list` after the recursive-option lowering fix made their runners benchmarkable. Both rows moved from failed-build/no-pair metadata to complete/equivalent measured-slower rows with residual `compiler` tags `list_node_clone` and `optional_clone`. `0002` is faster at size `100` but Python is faster at `1000`/`5000`; `0019` remains Python-faster at all sizes. This reduces no-pair failures from 30 to 28, raises fully complete problems from 294 to 296, and raises complete fixture pairs from 880 to 886. Local gates: targeted two-problem benchmark with `SIFR_BIN=target/debug/sifr`, analyzer metadata check, metadata Python compile, full registry JSON parse, `git diff --check`, and agent review `reviews/leetcode-linked-list-measured-slower-m5-review-pass-1.md`.
- `sifr-lang/sifr#2218` and `sifr-lang/leetcode#29`: fixed the partial-move follow-up for owned recursive optional field reads by lowering moved child reads through `.take().map(...)`, then reran the remaining 11 linked-list moved-result rows. `0024_swap_nodes_in_pairs` and `0147_insertion_sort_list` are now complete/equivalent and faster than Python at every configured size. The other nine rows are complete/equivalent measured-slower rows with residual `compiler` tags `list_node_clone` and `optional_clone`: `0021`, `0025`, `0061`, `0083`, `0086`, `0148`, `0203`, `0876`, and `1721`. This reduces no-pair failures from 28 to 17, raises fully complete problems from 296 to 307, and raises complete fixture pairs from 886 to 919. Local gates: focused recursive-option codegen tests, direct `0024`/`0206` Sifr runs, targeted 11-problem benchmark with `SIFR_BIN=target/debug/sifr`, analyzer metadata check, metadata Python compile, full registry JSON parse, `cargo fmt --check`, `git diff --check`, HIR and file-size guardrails, agent reviews `reviews/leetcode-recursive-option-take-m6-review-pass-1.md` and `reviews/leetcode-linked-list-moved-result-m6-review-pass-1.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/cache/skew advisories only).
- `sifr-lang/sifr#2220` and `sifr-lang/leetcode#30`: fixed recursive-node tree codegen residuals by making locals with optional recursive fields mutable and by cloning non-copy name arguments for borrowed optional parameters, then reran the final 17 no-pair residual rows. Sixteen rows are complete/equivalent and faster than Python at every configured size; `0269_alien_dictionary` is complete/equivalent with a small residual `noise` row at size `5000`. This reduces no-pair failures from 17 to 0, raises fully complete problems from 307 to 324, and raises complete fixture pairs from 919 to 970. Local gates: focused recursive-node codegen tests, targeted residual benchmark batches with `SIFR_BIN=target/debug/sifr`, analyzer metadata check, metadata Python compile, full registry JSON parse, `cargo fmt --check`, `git diff --check`, HIR and file-size guardrails, agent reviews `reviews/leetcode-recursive-node-tree-m7-review-pass-1.md` and `reviews/leetcode-final-residual-metadata-m7-review-pass-1.md`, and `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/cache/skew advisories only).
- `sifr-lang/leetcode#31`: reran `0234_palindrome_linked_list` for the missing size-100 pair and refreshed all three configured sizes. Correctness passes for sizes `100`, `1000`, and `5000`; the row is now complete/equivalent and remains in the measured-slower table as a compiler-owned linked-list clone case. This reduces partial benchmark problems from 1 to 0, raises fully complete problems from 324 to 325, and raises complete fixture pairs from 970 to 971. Local gates: targeted `0234` benchmark with `SIFR_BIN=target/debug/sifr`, analyzer metadata check, metadata Python compile, full registry JSON parse, `git diff --check`, and agent review `reviews/leetcode-palindrome-partial-m8-review-pass-1.md`.

### M5: Full Re-Benchmark And Closure

- Run the full LeetCode benchmark suite with the production benchmark command.
- Regenerate the HTML report and analyzer snapshot.
- Confirm every formerly known-divergent row is either:
  - fixed and reclassified,
  - still divergent with a follow-up ticket,
  - or moved to failed/partial with a concrete blocker.
- Confirm compiler-fixed families no longer emit the targeted pathological Rust in regression fixtures.
- Confirm runtime and memory outcomes are both acceptable; runtime fixes with material Peak RSS regressions stay open.
- Record final PR links, validation commands, and residual follow-up tickets in this document.

## Acceptance Criteria

- Known-divergent LeetCode Sifr implementations named in M1 are either ported to equivalent algorithms/data structures or explicitly deferred with linked follow-up tickets and report metadata that keeps them out of apples-to-apples summaries.
- Compiler fixes in M2/M3 include generated-code regression tests that prove the targeted pathological lowering is gone.
- Each fixed problem follows the post-fix re-benchmark protocol before metadata is reclassified.
- Runtime and Peak RSS both remain visible in the report; a runtime improvement with a Peak RSS regression greater than 10% at the same fixture size does not close the ticket unless the PR documents an intentional bounded tradeoff.
- `0212_word_search_ii` passes correctness before any Tries runtime comparison is treated as benchmark evidence.
- No partial measured benchmarks remain; `0234_palindrome_linked_list` now builds, passes correctness, and produces runtime plus memory rows for every configured fixture.
- The analyzer snapshot and report agree on complete, partial, failed, known-divergent, unknown, and equivalent counts.
- The final phase closure records implementation PRs, validation commands, benchmark command, refreshed analyzer counts, and any residual follow-up tickets.
- agent review has approved this fix-oriented phase as implementation-ready after iterative review.

## Predecessor Baseline Work

Completed on 2026-05-30 by the prior benchmark-analysis/report phase:

- Added `audits/leetcode/benchmarks/analyze_slowness.py` for deterministic `.raw` result analysis.
- Added `audits/leetcode/benchmarks/slowness_seed.py` and seeded registry metadata for the 75 measured-slower problems plus the 53 incomplete/failed entries.
- Extended benchmark specs with `benchmark_status`, `parity_status`, `primary_slowness_owner`, and `slowness_tags`.
- Updated the HTML report so summary metrics only include `complete` + `equivalent` comparisons, while divergent, unknown, partial, and failed cases remain visible through metadata badges and the coverage inventory.
- Generated analyzer snapshot below confirms the phase inventory counts.

Merged PRs:

- `sifr-lang/leetcode#7`: benchmark analyzer, metadata seeding, and report filtering.
- `sifr-lang/sifr#2201`: phase closure docs, review artifacts, validation note, and initial subrepo pointer update.
- `sifr-lang/sifr#2202`: normalized `audits/leetcode` to the squash-merged `leetcode` main commit.
- `sifr-lang/sifr#2204`: restored the phase review artifacts referenced by this document.

Validation run:

- `python3 -m py_compile benchmarks/analyze_slowness.py benchmarks/report_metadata.py benchmarks/report.py benchmarks/specs.py benchmarks/slowness_seed.py`
- `python3 benchmarks/analyze_slowness.py --check-metadata`
- `python3 benchmarks/bench.py report-html`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 verification/tooling/check_editor_assets.py`
- `scripts/run_all_tests.sh --profile quick` (exit 0; wall-time/skew advisories only)
- touched source files checked under the 900-line guardrail

Prior agent review rounds for the predecessor analysis/report phase:

- `reviews/leetcode-slowness-phase-review-pass-1.md`: satisfied, no blocking issues.
- `reviews/leetcode-slowness-phase-review-pass-2.md`: satisfied, no blocking or important issues remain.

Fix-phase agent review rounds:

- `reviews/leetcode-slowness-fix-phase-review-pass-1.md`: confirmed the phase was fix-oriented; requested concrete helper APIs, trie structure, emitted-code contracts, regression-test location, parallelism grounding, and memory threshold.
- `reviews/leetcode-slowness-fix-phase-review-pass-2.md`: confirmed those gaps were resolved and no blocking issues remain.

<!-- analyze_slowness:start -->
## Generated Analyzer Snapshot

<!-- This section is generated by audits/leetcode/benchmarks/analyze_slowness.py. -->

### Summary

| Metric | Count |
| --- | --- |
| Registry problems | 394 |
| Benchmarkable problems | 394 |
| Source-only/unbenchmarked problems | 0 |
| Fully complete problems | 394 |
| Complete fixture pairs | 1178 |
| Measured-slower problems | 0 |
| Partial benchmark problems | 0 |
| No-pair failed problems | 0 |

### Measured-Slower Problems

| Problem | Category | Worst Py/Sifr | Slower sizes | Owner | Parity | Tags |
| --- | --- | --- | --- | --- | --- | --- |

### Partial Benchmarks

| Problem | Complete pairs | Missing sizes | Status |
| --- | --- | --- | --- |

### No-Pair Failures

| Problem | Status | Failure excerpt |
| --- | --- | --- |
<!-- analyze_slowness:end -->
