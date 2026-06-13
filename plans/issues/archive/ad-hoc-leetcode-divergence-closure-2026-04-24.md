# Ad-hoc Phase: LeetCode Divergence Closure And Canonical Fixture Plan (2026-04-24)

Status: closed
Owner: ad_hoc_leetcode_divergence_closure
Source analysis: `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`
Source scan: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Source corpus: `audits/leetcode/`
Review status: pass-2 ready verdict from `reviews/ad-hoc-leetcode-divergence-closure-2026-04-24-review-pass2.md`; follow-up clarifications incorporated

## Purpose

Convert the LeetCode divergence analysis into an executable implementation phase that closes canonical algorithm drift, reduces language/stdlib friction, and preserves Sifr's core guarantees:

- static typing and explicit `Option` / `Result` safety
- no Python truthiness coercions
- no implicit nullable access
- no mutable `nonlocal` capture
- no ownership weakening, shared mutable node aliases, or interior-mutability escape hatches
- no user-triggerable runtime panics in generated user paths

This phase is about paired-fixture divergence. It is not the authoritative run-failure tracker; fresh compile/run failures must be collected from a new full LeetCode run and cross-linked only when their root cause maps to a workstream here.

## Baseline Snapshot

Primary category counts from the reviewed analysis:

- Category 1 rewrite debt: `13`
- Category 2a recursive node / cursor ergonomics: `19`
- Category 2b collection / index / stdlib ergonomics: `21`
- Category 3 okay as-is / corpus noise: `4`
- Category 4 intentional architecture boundary: `6`
- Primary total: `63`
- Duplicate primary placements: `0`
- Missing paired fixtures with `changed_total_lines >= 80`: `0`

Below-cutoff fixtures are intentionally included only when they show explicit parity debt or an architecture-boundary issue:

- `0004_median_of_two_sorted_arrays`
- `0024_swap_nodes_in_pairs`
- `0138_copy_list_with_random_pointer`
- `0141_linked_list_cycle`
- `0146_lru_cache`
- `0206_reverse_linked_list`
- `0208_implement_trie_prefix_tree`
- `0211_design_add_and_search_words_data_structure`
- `0295_find_median_from_data_stream`
- `0706_design_hashmap`

Layered-pressure note:

- `0516_longest_palindromic_subsequence` remains Category 3/5 as corpus noise, but WS1 tracks its `memo.get((i, j), -1)` sentinel pattern under `I2`.
- `0673_number_of_longest_increasing_subsequence` remains Category 4a for mutable `nonlocal`, but WS1 tracks its linear `valueAt` workaround under `I1`.

## Non-goals

- Do not add mutable `nonlocal` capture.
- Do not add Python-style truthiness or `a or b` value fallback semantics.
- Do not change the abstract return type of `list` / `dict` subscripts; narrowing must remain local and proof-based.
- Do not introduce `Rc<RefCell<...>>`, `Cell`, or similar interior mutability into cursor or collection ergonomics.
- Do not add `defaultdict`-style auto-insert-on-read semantics.
- Do not attempt canonical object-identity rewrites for Category 4b without a separately approved safe arena / handle design.
- Do not triage the 16 `sifr_only` `_v2` fixtures in this phase; they need a separate decision on whether they are deliberate Sifr-native alternates, orphaned fixtures needing Python pairs, or deletion candidates.

## Workstreams

### WS0: Corpus Normalization And Baseline Hygiene

Fixtures:

- `0104_maximum_depth_of_binary_tree`
- `0130_surrounded_regions`
- `0200_number_of_islands`
- `0516_longest_palindromic_subsequence`

Scope:

- Remove or isolate Python-side stacked implementations and unreachable helper baggage that inflate raw diff size.
- Normalize mirrored helper boilerplate where it hides true Sifr-side divergence.
- Re-run the pair scan and record the new baseline.

Implementation steps:

1. Normalize the four Category 3/5 Python fixtures in one PR.
2. Keep one canonical implementation per fixture unless alternatives are explicitly split into named fixtures.
3. Preserve existing assertions and add oracle coverage if normalization changes fixture structure.
4. Regenerate the pair scan with `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_<YYYYMMDD>.json`.

Exit criteria:

- The four fixtures no longer appear as high-priority divergence outliers for corpus-noise reasons.
- Pair scan shows expected `changed_py_lines` reduction.
- Sifr-side helper boilerplate cleanup is out of scope for this WS0 slice unless it is in one of the four fixtures; broader helper cleanup belongs to `B1` or a follow-up helper-boilerplate sweep.
- No new compile/run failures are introduced by fixture cleanup.

### WS1: Narrowing Rule Design, Diagnostics, And Guardrails

Feature IDs:

- `D0`: shared narrowing/invalidation design plus diagnostics
- `N1`: local `is not None` narrowing for bindings
- `N2`: recursive-node field projection narrowing
- `N3`: narrowing across rebinding when RHS is provably non-`None`
- `N4`: repeated-check narrowing without copy-to-local ceremony
- `I1`: list-index narrowing after proven in-bounds access
- `I2`: dict-key narrowing after insert / contains-key proof / sentinel `.get(k, s)` pattern

Scope:

- Implement proof-gated, local Optional-flow improvements without changing the base type of indexing operations.
- Make invalidation rules explicit before implementation: intervening writes, collection mutation, calls that can alias-mutate, and function-boundary escape must invalidate local facts.
- Improve diagnostics so failed narrowing explains the invalidating write, call, or proof gap.

Representative fixtures:

- 2a: `0002`, `0019`, `0021`, `0450`, `0669`, `0876`
- 2b: `0150`, `0261`, `0297`, `0394`, `0516`, `0567`, `0673`, `0721`, `1203`, `2092`, `2709`

Implementation steps:

1. Add a short design note for `D0` before code changes.
2. Implement `N1` and `I1` first with targeted unit and e2e tests.
3. Implement `I2` separately from `I1`; dict proofs and invalidation differ from list-index proofs.
4. Implement `N2` through `N4` in separate PRs with small linked-list/tree fixtures as acceptance tests.
5. Add at least one non-LeetCode regression per narrowing rule to prevent corpus-specific pattern matching.

Exit criteria:

- Targeted fixtures remove dead Optional guard boilerplate without implicit unwraps.
- Existing failure tests still reject unsafe nullable access.
- Diagnostics identify why narrowing failed when a proof cannot be maintained.
- Each narrowing rule lands with at least one non-LeetCode unit or e2e regression in the same PR.
- `scripts/run_all_tests.sh --profile quick` passes after each PR.

### WS2: Stdlib And Owned Collection Parity

Feature IDs:

- `O1`: owned collection helpers with explicit ownership signatures (`drain`, `take_at`, `split_first`, `iter_mut_indexed`)
- `S1`: heap / priority queue
- `S2`: DSU / union-find
- `S3`: deque
- `S4`: character predicates
- `S5`: whole-token integer parsing returning `Result`
- `S6`: explicit LeetCode trie helper decision

Scope:

- Add stdlib and helper surface that materially unlocks canonical algorithms.
- Keep all APIs explicit and compatible with Sifr's safety model.

Implementation order:

1. `S1` heap and `S2` DSU first; they are independent and can land in either order.
2. `S3` deque, `S4` character predicates, and `S5` parse helpers next.
3. `S6` trie decision last because it determines the canonical rewrite shape for three fixtures.
4. `O1` helpers only when a target fixture needs the helper and the Rust lowering can be expressed without interior mutability.

`O1` minimum deliverables:

- `drain`: move all elements out of an owned collection while leaving a valid empty collection behind.
- `take_at`: remove and return one element at an index with explicit `Option` / `Result` behavior, no panic path.
- `split_first`: split the first element from the remaining collection without cloning.
- `iter_mut_indexed`: expose index-aware mutable iteration where Rust lowering can preserve uniqueness.

Representative fixtures:

- `S1`: `0355`, `0743`, `0778`, `1584`, `1631`, `0295`
- `S2`: `0261`, `0721`, `1489`, `2092`, `2709`
- `S3`: `0286`, `0513`, `0752`
- `S4`: `0394`, `1397`
- `S5`: `0150`, `0297`
- `S6`: `0208`, `0211`, `0212`, `1397`

Exit criteria:

- Canonical heap/DSU/deque/trie algorithms can be expressed without fixture-local encoded queues/heaps or linear scans.
- `int` parsing remains `Result`-returning; fixtures handle parse failure explicitly.
- Trie-dependent LeetCode fixtures use an explicit local `Trie` helper or explicit nested-dict helpers. Auto-insert-on-read is rejected, and no public stdlib trie surface is introduced for this phase.

### WS3: Owned Chain Cursor Ergonomics And Fixture Helper Convention

Feature IDs:

- `C1`: dummy-head cursor over owned linked-list chains
- `C2`: in-place `.next` skip under double narrowing
- `C3`: sub-range rewire/reverse over owned chains
- `R1`: structural recursion over owned chains/trees with read-only reborrows
- `B1`: shared fixture-helper convention

Scope:

- Make canonical linked-list and tree traversal/rewiring patterns practical without weakening ownership.
- Decide fixture helper strategy before rewriting multiple list/tree fixtures.

Helper convention decision:

- Choose exactly one before WS4 starts:
  - fixture prelude/import for shared `ListNode` / `TreeNode` helpers
  - generated fixture helper module
  - self-contained duplication accepted with a strict boilerplate template

Representative fixtures:

- `C1`: `0021`, `0023`, `0024`, `0206`
- `C2`: `0019`, `0203`, `0707`
- `C3`: `0025`, `0092`, `0148`, `1669`, `1721`
- `R1`: `0450`, `0669`

Intentional ledger delta:

- `0206`, `0707`, and `0148` are included here as rewrite pilots because they exercise the same cursor families as the base feature IDs.
- `0894` remains in WS5 as an ownership-boundary fixture, so its boundary-adjacent read traversal is not a WS3 acceptance target in this phase.

Exit criteria:

- Each node remains owned exactly once at every program point.
- Cursor rewrites do not rely on shared mutable node references or interior mutability.
- Linked-list parity-debt rewrites can share a stable helper/testing pattern.

### WS4: Canonical Rewrite Debt

This workstream rewrites fixtures whose current Sifr version changes the public model, algorithm shape, or asymptotic behavior. Rewrites should land as soon as their prerequisites are available; they do not wait for every WS1-WS3 feature.

| Fixture | Canonical target | Key prerequisites | Acceptance |
| --- | --- | --- | --- |
| `0004_median_of_two_sorted_arrays` | binary-partition median, `O(log(min(m,n)))` | `I1`, numeric sentinel / `Result` conventions | no full merge; odd/even and empty-side cases covered |
| `0023_merge_k_sorted_lists` | min-heap or divide-and-conquer merge of `ListNode` heads | `S1` or pairwise merge choice, `C1`, `N2`, `B1` | public model uses `ListNode`; result chain matches canonical values |
| `0024_swap_nodes_in_pairs` | dummy-cursor pairwise node rewiring | `C1`, `N2`, `B1` | no `list[int]` public model |
| `0146_lru_cache` | `O(1)` LRU cache using hashmap plus explicit recency structure | `I2`, `0146_recency_structure_design` | behavioral: LeetCode sample sequence passes; structural: no linear scans or array shifts in final `get` / `put`; eviction is `O(1)` |
| `0147_insertion_sort_list` | linked-list insertion sort over nodes | `C1`, `C2`, `N2`, `B1` | no drain/sort/rebuild |
| `0148_sort_list` | linked-list merge sort over nodes | `C1`, `C3`, `N2`, `B1` | no flatten/sort/rebuild |
| `0208_implement_trie_prefix_tree` | trie with per-character traversal | `S6` | no word-list scan |
| `0206_reverse_linked_list` | in-place node-chain reversal | `C1`, `N2`, `B1` | no `list[int]` public model |
| `0211_design_add_and_search_words_data_structure` | trie plus wildcard DFS for `.` branches | `S6`, `I2` | no per-word linear scan |
| `0212_word_search_ii` | trie/prefix-pruned board search | `S6`, `I2` | no per-word full-board search |
| `0295_find_median_from_data_stream` | dual-heap median finder | `S1` | no sorted-array insertion |
| `0706_design_hashmap` | hashmap implementation without built-in `dict` delegation | `I1`, `take_at` or `split_first` only if chosen design needs them, `0706_hashmap_storage_design` | behavioral: sample sequence and real `-1` value cases pass; structural: no built-in dict; `remove` deletes rather than writing `-1` |
| `0707_design_linked_list` | linked-list data structure operations | `C1`, `C2`, `B1` | operation-cost profile matches linked-list design intent |

Design prerequisites:

- `0146_recency_structure_design`: one short design note in the rewrite PR or a preceding PR. It must choose the final recency representation and explain how `get`, `put`, update, and eviction meet `O(1)` without interior mutability.
- `0706_hashmap_storage_design`: one short design note in the rewrite PR or a preceding PR. It must choose bucket chaining or open addressing, define deletion semantics, and avoid built-in `dict` delegation.

Exit criteria:

- Each rewritten fixture preserves canonical public shape where compatible with Sifr.
- Each rewrite updates `main()` assertions to prove the canonical property, not only value-list equality.
- Pair scan moves the fixture out of the rewrite-debt list or records an explicit remaining blocker.

### WS5: Architecture Boundary Documentation And Sifr-Native Alternates

Category 4a:

- `0673_number_of_longest_increasing_subsequence`: mutable `nonlocal` closure state remains unsupported. Keep the explicit-state rewrite and remove layered index workaround through `I1`.

Category 4b:

- `0133_clone_graph`
- `0138_copy_list_with_random_pointer`
- `0141_linked_list_cycle`
- `0160_intersection_of_two_linked_lists`
- `0894_all_possible_full_binary_trees`

Pattern continuity:

- `0052_n_queens_ii`
- `0543_diameter_of_binary_tree`
- `0783_minimum_distance_between_bst_nodes`
- `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero`

These are below-cutoff nonlocal-to-explicit-state examples. If a future scan promotes them, they retain Category 4a classification.

Scope:

- Document why canonical object-identity/cyclic input shapes are outside the current single-ownership model.
- Add or preserve value-semantic Sifr-native alternates only when they honestly represent a supported variant.
- Do not claim canonical parity for object-identity problems unless a safe arena / handle model is approved in a separate phase.

Exit criteria:

- Each Category 4 fixture has an explicit classification note in the scan/report artifacts.
- Vacuous tests such as `0141` are marked as boundary-limited rather than correctness evidence.
- No Category 4 fixture remains in the canonical rewrite backlog.

### WS6: Final Rerun, Scorecard, And Closure

Scope:

- Re-run the full LeetCode corpus.
- Re-run the pair divergence scan.
- Regenerate any taxonomy/scorecard artifacts affected by the phase.
- Close only after review sign-off.

Exit criteria:

- `scripts/run_all_tests.sh --profile quick` passes.
- `scripts/run_all_tests.sh` passes before final phase closure.
- Full LeetCode run has no new compile/run failures attributable to this phase.
- Pair scan confirms all high-diff outliers are either canonicalized, intentionally bounded, or separately tracked.
- Review files are stored under `reviews/` and linked from the execution report.

## Execution Order

1. WS0 corpus normalization and baseline refresh.
2. WS1 narrowing design and first compiler slices.
3. WS2 `S1` heap and `S2` DSU, followed by unlocked rewrites such as `0295`.
4. WS3 helper convention and owned-chain cursor slices.
5. WS2 `S3` / `S4` / `S5`, then parser/BFS fixture cleanup.
6. WS2 `S6`, then trie-dependent rewrites `0208`, `0211`, `0212`.
7. WS4 remaining rewrites as prerequisites become available.
8. WS5 boundary documentation and Sifr-native alternate cleanup.
9. WS6 final rerun, scorecard, and review sign-off.

## Required Validation Per PR

- Targeted fixture check/run for every touched LeetCode fixture.
- Focused compiler/e2e regression for every language/compiler change.
- `cargo fmt --check`.
- `cargo clippy --workspace -- -D warnings` when Rust code changes.
- `python3 scripts/check_hir_maintainability_guardrails.py` for compiler/HIR changes.
- `scripts/run_all_tests.sh --profile quick`.
- Full `scripts/run_all_tests.sh` before final phase closure.

## Required Artifacts

- Updated phase execution report: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24-execution.md`.
- Updated pair scan artifact with date-stamped filename.
- Updated full LeetCode run artifact with date-stamped filename.
- Updated failure/divergence taxonomy if counts change.
- Review files under `reviews/`.
- Demo fixture under `demos/` only if a user-visible language/stdlib feature lands.
- `internal_docs/architecture.md`, `internal_docs/roadmap.md`, and `internal_docs/phases/` updated when a workstream changes architecture, roadmap sequencing, or phase status.

## Ready-To-Implement First PRs

1. `WS0_corpus_noise_normalization`
- Files: `audits/leetcode/0104_*.py`, `0130_*.py`, `0200_*.py`, `0516_*.py`
- Acceptance: one canonical implementation remains per fixture, scan rerun shows expected noise drop.

2. `WS1_D0_narrowing_invalidation_design`
- Files: `internal_docs/` design note plus targeted compiler tests.
- Acceptance: documented invalidation rules and diagnostics expectations approved before `N1` / `I1`.

3. `WS2_S1_heap_stdlib`
- Candidate files/crates: `crates/sifr_hir`, `crates/sifr_codegen`, `crates/sifr_driver`, and runtime/stdlib collection shims discovered during implementation.
- Acceptance: heap-backed fixtures can drop hand-rolled priority-queue encodings; `0295` rewrite becomes unblocked.

4. `WS2_S2_dsu_helper`
- Candidate files/crates: `crates/sifr_hir`, `crates/sifr_codegen`, `crates/sifr_driver`, and runtime/stdlib helper shims discovered during implementation.
- Acceptance: DSU fixtures can remove parent/rank boilerplate without weakening dict/index safety.

5. `WS3_B1_fixture_helper_convention`
- Files: test/fixture helper docs and one representative pilot migration.
- Pilot fixture: `0021_merge_two_sorted_lists`.
- Acceptance: linked-list/tree rewrites have one approved helper/testing pattern; the pilot excludes new cursor features not yet landed.

6. `WS4_0146_recency_structure_design`
- Files: design note in `issues/` or `internal_docs/` plus optional fixture-local notes.
- Acceptance: final LRU representation is chosen before rewrite work starts.

7. `WS4_0706_hashmap_storage_design`
- Files: design note in `issues/` or `internal_docs/` plus optional fixture-local notes.
- Acceptance: bucket chaining vs open addressing is chosen before rewrite work starts.

## Review Gate

Before implementation starts, run external review on this phase document and incorporate blocking feedback. The phase is implementation-ready only after the review confirms:

- category-to-workstream mapping is complete,
- no workstream violates Sifr boundaries,
- fixture acceptance criteria are specific enough for developers,
- validation and closure artifacts are sufficient to prove progress toward zero failures/divergence.
