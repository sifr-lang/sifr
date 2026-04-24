# LeetCode Divergence Decision Analysis

Date: 2026-04-09
Source ranking: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Scope rule: focus on paired fixtures with `changed_total_lines >= 80`, then manually include important parity-debt exceptions below the cutoff where needed.

## Preconditions Before Using This Analysis

- Raw diff size is only a triage signal, not a calibrated severity score.
- Items near the cutoff, especially in the `70-90` changed-line band, require manual judgment with `similarity_ratio` and signed line delta before escalation.
- Public-surface changes and asymptotic regressions matter more than raw line count.
- Shared helper boilerplate can hide divergence because mirrored dead code cancels out of the diff.
- Some Python fixtures contain multiple full implementations, which inflates divergence artificially.
- Some Sifr fixtures also carry repeated helper boilerplate, such as kitchen-sink `Node` classes and `nodeVal` / `nodeNext` / `hasNode` / `unwrapInt` or tree helper sets, which inflates `changed_sifr_lines` independently of real language divergence.
- "Python-like" pressure must be split into:
  - language/compiler ergonomics
  - stdlib/data-structure parity
  - explicit parity-debt rewrites

## Execution Scope

- This document addresses paired-fixture divergence against the 2026-04-09 diff scan. It is not a compile/run failure report; current failing fixtures must be tracked from a fresh run and cross-linked to this taxonomy.
- "Failure" means a Sifr fixture does not compile or run correctly. "Divergence" means it runs but differs materially from the canonical Python problem shape, public model, asymptotics, or ergonomics.
- Some fixtures whose canonical input requires node aliasing or cycles can pass only vacuous tests under the current owned model. `0141_linked_list_cycle` is the clear case: the Sifr body unconditionally returns `False`, and the fixture has no cycle-input test because that shape is not constructible under single ownership.
- The scan also contains 16 `sifr_only` `_v2` fixtures with no Python pair: `0014_longest_common_prefix_v2`, `0045_jump_game_ii_v2`, `0053_maximum_subarray_v2`, `0055_jump_game_v2`, `0058_length_of_last_word_v2`, `0121_best_time_to_buy_and_sell_stock_v2`, `0134_gas_station_v2`, `0152_maximum_product_subarray_v2`, `0169_majority_element_v2`, `0198_house_robber_v2`, `0238_product_of_array_except_self_v2`, `0300_longest_increasing_subsequence_v2`, `0605_can_place_flowers_v2`, `0704_binary_search_v2`, `1768_merge_strings_alternately_v2`, and `1929_concatenation_of_array_v2`. They need separate triage before using pair-scan metrics as complete coverage.
- Below-cutoff fixtures are out of scope unless explicitly listed as parity-debt exceptions or Category 4 continuity examples.

## Main Decision Categories

### 1. Should Have Parity, Rewrite Mainly

These are not just verbose Sifr ports. They should be rewritten toward the canonical Python problem shape once the necessary surface exists.
Items below the `changed_total_lines >= 80` scope are included only when they show explicit parity debt such as a public-model change or asymptotic regression.

Above-cutoff cases:

- `0023_merge_k_sorted_lists`
- `0147_insertion_sort_list`
- `0148_sort_list`
- `0212_word_search_ii`
- `0707_design_linked_list`

Below-cutoff explicit parity-debt exceptions:

- `0004_median_of_two_sorted_arrays`
- `0024_swap_nodes_in_pairs`
- `0146_lru_cache`
- `0208_implement_trie_prefix_tree`
- `0206_reverse_linked_list`
- `0211_design_add_and_search_words_data_structure`
- `0295_find_median_from_data_stream`
- `0706_design_hashmap`

Why:

- `0004_median_of_two_sorted_arrays` replaces the canonical binary-partition solution with full merge, changing `O(log(min(m,n)))` behavior to `O(m+n)`.
- `0023_merge_k_sorted_lists` changes the public input model from linked lists to `list[list[int]]`.
- `0024_swap_nodes_in_pairs` changes the public model from `ListNode` pairwise rewiring to `list[int]` pairwise value swap; the canonical dummy-cursor implementation is absent.
- `0147_insertion_sort_list` replaces linked-list insertion sort with drain/`sorted()`/rebuild; the canonical algorithm is absent despite the preserved `ListNode` signature.
- `0146_lru_cache` replaces the canonical `O(1)` hashmap plus recency-list design with linear `findIndex`, `pop(index)`, and `pop(0)` array operations.
- `0148_sort_list` replaces linked-list merge sort with flatten/sort/rebuild.
- `0208_implement_trie_prefix_tree` replaces trie insert/search/prefix traversal with linear scans over stored words.
- `0206_reverse_linked_list` changes the public model from linked-list reversal to list-value reversal.
- `0211_design_add_and_search_words_data_structure` replaces trie wildcard DFS with linear scans over stored words.
- `0212_word_search_ii` replaces trie/prefix-pruning with per-word board search.
- `0295_find_median_from_data_stream` replaces heap-based updates with sorted-array insertion and changes asymptotic behavior.
- `0706_design_hashmap` delegates to built-in `dict` for the problem that asks for a hashmap implementation, and `remove` writes a `-1` sentinel instead of deleting.
- `0707_design_linked_list` replaces a linked-list data-structure design with array-backed storage and loses the intended operation-cost profile.

### 2. Should Support Similar Python Features / Ergonomics

These are valid Sifr targets, but the language and stdlib should make the canonical solution shape much easier to express safely.

#### 2a. Recursive Node / Cursor Ergonomics

- `0002_add_two_numbers`
- `0019_remove_nth_node_from_end_of_list`
- `0021_merge_two_sorted_lists`
- `0025_reverse_nodes_in_k_group`
- `0061_rotate_list`
- `0083_remove_duplicates_from_sorted_list`
- `0086_partition_list`
- `0092_reverse_linked_list_ii`
- `0143_reorder_list`
- `0203_remove_linked_list_elements`
- `0234_palindrome_linked_list`
- `0450_delete_node_in_a_bst`
- `0662_maximum_width_of_binary_tree`
- `0669_trim_a_binary_search_tree`
- `0876_middle_of_the_linked_list`
- `1609_even_odd_tree`
- `1669_merge_in_between_linked_lists`
- `1721_swapping_nodes_in_a_linked_list`
- `2130_maximum_twin_sum_of_a_linked_list`

What should improve:

- narrowing after `is not None` on local bindings and recursive-node field projections, scoped to regions with no intervening writes to the binding or projected field
- compiler-preserved narrowing across rebinding when the right-hand side is itself provably non-`None` at the assignment point
- narrowing preserved across repeated checks of the same binding without copy-to-local ceremony
- cursor-style mutation patterns (trailing dummy-head cursors, in-place `.next` skips under double narrowing, and sub-range rewire/reverse operations) expressed as moves and reborrows through an `own`-annotated chain, with each node owned exactly once at every program point; no shared mutable node references or interior-mutability escape hatches
- structural recursion over owned chains and trees, including read-only reborrowed traversal, as a distinct ergonomics question from narrowing; shared ownership of nodes across sibling traversals is out of scope and collides with the `0894` boundary
- once these ergonomics land, each Category 2a fixture still requires an individual rewrite step to restore the canonical cursor/recursive shape; ergonomics alone will not convert drain-to-list-and-rebuild implementations into canonical solutions

#### 2b. Collection / Index / Stdlib Ergonomics

- `0150_evaluate_reverse_polish_notation`
- `0261_graph_valid_tree`
- `0269_alien_dictionary`
- `0286_walls_and_gates`
- `0297_serialize_and_deserialize_binary_tree`
- `0355_design_twitter`
- `0394_decode_string`
- `0417_pacific_atlantic_water_flow`
- `0513_find_bottom_left_tree_value`
- `0567_permutation_in_string`
- `0721_accounts_merge`
- `0743_network_delay_time`
- `0752_open_the_lock`
- `0778_swim_in_rising_water`
- `1203_sort_items_by_groups_respecting_dependencies`
- `1397_find_all_good_strings`
- `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree`
- `1584_min_cost_to_connect_all_points`
- `1631_path_with_minimum_effort`
- `2092_find_all_people_with_secret`
- `2709_greatest_common_divisor_traversal`

What should improve:

- local, flow-sensitive narrowing of `list[T][i]` and `dict[K, V][k]` from `T | None` / `V | None` to `T` / `V` within a basic block where the compiler can prove the access is in bounds or the key is present, and no mutation of the collection intervenes between the proof and use; the subscript operator's abstract return type is unchanged
- dict-entry narrowing after insertions and contains-key checks, especially for parent/representative maps and adjacency maps, invalidated by any call that could alias-mutate the dict and never propagated across function boundaries
- owned collection helpers with clearly typed ownership signatures, such as `drain`, `take_at`, `split_first`, and `iter_mut_indexed`, each expressible in generated Rust without `Rc<RefCell<...>>` or other interior-mutability primitives
- stdlib parity where it materially unblocks canonical algorithms:
  - `heap`
  - `deque`
  - DSU / union-find helpers
  - character-class predicates such as `isdigit` / `isalpha`
  - whole-token integer parsing returning `Result`
  - trie-friendly APIs as an explicit `Trie` type or explicit nested-dict construction helpers; no auto-insert-on-read / `defaultdict` semantics

### 3. Okay The Way They Are

These are high diff mostly because the Python side is noisy or redundant, not because the Sifr version is meaningfully wrong.

- `0104_maximum_depth_of_binary_tree`
- `0130_surrounded_regions`
- `0200_number_of_islands`
- `0516_longest_palindromic_subsequence`

Why:

- `0104_maximum_depth_of_binary_tree.py` stacks three `maxDepth` definitions plus an unused kitchen-sink `Node` class and `tree_to_string` helper; the Sifr version is a clean recursive solution with `changed_sifr_lines=8` against `changed_py_lines=74`.
- `0130_surrounded_regions.py` pairs a primary DFS-with-set `solve` with a triple-quoted alternate 3-pass marker implementation; the Sifr version is the canonical DFS-with-set with `changed_sifr_lines=26` against `changed_py_lines=55`, and exhibits none of the Category 2b symptoms.
- `0200_number_of_islands.py` stacks three `numIslands` definitions (set-tracked DFS, in-place grid-mutation DFS, and BFS via `deque`); the Sifr version is a canonical set-DFS with `changed_sifr_lines=21` against `changed_py_lines=82`.
- `0516_longest_palindromic_subsequence.py` stacks a bottom-up 2D DP with an unreachable memoization block inside the first `longestPalindromeSubseq`, plus a second `longestPalindromeSubseq` redefinition that delegates to a separate `longestCommonSubsequence` helper; the Sifr version is a top-down memoized LCS on `s` and `s[::-1]`, with `changed_sifr_lines=22` against `changed_py_lines=61`.
- `0516_longest_palindromic_subsequence` still contains a small Category 2b-adjacent `memo.get((i, j), -1)` sentinel workaround; it stays here because the algorithm and complexity match canonical LCS, not because dict-Optional ergonomics are fully solved.
- These are strong enough corpus-noise cases that they should not drive language priorities.

### 4. Acceptable Divergence Because Of An Intentional Architecture Boundary

These are not ideal only in the abstract, but the current divergence is acceptable because it sits behind a deliberate Sifr design boundary.
A fixture may carry collection/index/stdlib pressure on top of a Category 4 boundary; the Category 4 label identifies the structural boundary, not a claim that the rest of the fixture is ergonomically clean.

4a. Mutable closure-state boundary:

- `0673_number_of_longest_increasing_subsequence`

4b. Node aliasing / cycle shapes under single ownership:

- `0133_clone_graph`
- `0138_copy_list_with_random_pointer`
- `0141_linked_list_cycle`
- `0160_intersection_of_two_linked_lists`
- `0894_all_possible_full_binary_trees`

Why:

- `0673_number_of_longest_increasing_subsequence` uses mutable `nonlocal` closure state in Python. Sifr intentionally does not support that architecture, and the Sifr fixture rewrites the same algorithmic shape as forward-iterative DP with a post-pass accumulator rather than adding mutable `nonlocal`.
- `0673_number_of_longest_increasing_subsequence` also carries layered Category 2b pressure: the current `valueAt` helper linearly scans to avoid `int | None` from `nums[i]`, which degrades realized complexity until list-index ergonomics improve. The Category 4 boundary is the nonlocal rewrite; asymptotic recovery is a Category 2b follow-up.
- `0673_number_of_longest_increasing_subsequence.py` also trails an unreachable `O(n^2)` DP block after `return res`, inflating `changed_py_lines=56`; the architecture-boundary classification remains primary.
- It should not be used as pressure to add mutable `nonlocal` support.
- `0133_clone_graph` needs cyclic graph identity in both the input and cloned output. Under single ownership, this requires a safe non-owning handle / arena model or an explicitly value-semantic alternate, not a direct object-graph rewrite.
- `0138_copy_list_with_random_pointer` needs copied `random` pointers to alias other copied nodes that are already owned by chain positions. Under current ownership rules, this is a node-aliasing boundary.
- `0141_linked_list_cycle` needs a cyclic input to test Floyd's algorithm. The current Sifr fixture returns `False` for all inputs and only tests an acyclic list, so the canonical problem shape is not expressible under single ownership today.
- `0160_intersection_of_two_linked_lists` needs two list heads sharing the same tail node. This node-identity intersection shape conflicts with the "each node owned exactly once" invariant.
- `0894_all_possible_full_binary_trees` diverges because Python aliases shared subtree nodes across multiple generated parent trees, while Sifr's single-ownership model requires cloning each subtree per parent.
- `0894_all_possible_full_binary_trees` also drops Python's memoized `dp` cache because cached `list[TreeNode]` values would still require clone-out on each hit; this is a consequence of the same ownership boundary, not a separate divergence.
- The same nonlocal-to-pure-return or explicit-stack rewrite pattern appears below the `changed_total_lines >= 80` scope in `0052_n_queens_ii`, `0543_diameter_of_binary_tree`, `0783_minimum_distance_between_bst_nodes`, and `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero`; these are recorded for pattern continuity, not escalation.
- If a future scan promotes any of those above the cutoff, they retain the Category 4 classification and should not be reclassified as 2a under parity pressure.
- Category 4b fixtures do not owe canonical object-identity rewrites under the current language boundaries. Valid follow-up is documentation, a value-semantic Sifr-native alternate, or a separately approved safe arena / handle design.

### 5. Needs Corpus Cleanup Before It Should Drive Design Priorities

This is a secondary label for the same fixtures enumerated in Category 3. The primary classification is "okay as-is"; Category 5 is the cleanup follow-up that removes Python-side multi-implementation baggage, unused helper/kitchen-sink classes, and mirrored helper boilerplate that can distort or hide divergence in the raw scan.

Why:

- applies to the Category 3 fixtures (`0104_maximum_depth_of_binary_tree`, `0130_surrounded_regions`, `0200_number_of_islands`, `0516_longest_palindromic_subsequence`), whose raw diffs are inflated by Python-side multi-implementation or helper baggage rather than Sifr-side divergence

## Feature Ledger

These feature IDs make the categories schedulable. Each implementation issue should reference one or more IDs and list the fixtures expected to improve.

| ID | Scope | Example Fixtures | Exit Signal |
| --- | --- | --- | --- |
| `C0` | Corpus normalization for Category 3/5 and helper-boilerplate noise | `0104`, `0130`, `0200`, `0516` | re-run scan shows expected LOC reduction and no new divergence |
| `D0` | Shared narrowing/invalidation design plus diagnostics | 2a and 2b fixtures | written spec with invalidating writes/calls and diagnostic expectations |
| `N1` | `is not None` narrowing for local bindings with no intervening writes | `0002`, `0021` | fixtures remove local copy-to-narrow ceremony |
| `N2` | recursive-node field projection narrowing, such as `.next` / `.left` / `.right` after explicit proof | `0021`, `0450`, `0669` | field accesses narrow only inside proven safe regions |
| `N3` | preserve narrowing across rebinding when RHS is provably non-`None` | `0019`, `0876` | no re-narrowing after safe cursor advancement |
| `N4` | preserve repeated-check narrowing without copy-to-local ceremony | linked-list and tree traversal fixtures | repeated guard patterns shrink without implicit unwrap |
| `I1` | list-index narrowing after compiler-proven in-bounds access | `0150`, `0297`, `0394`, `0567`, `0673` | removes `valueAt` / sentinel helpers where bounds are proven |
| `I2` | dict-key narrowing after insert, contains-key proof, or sentinel-return `.get(k, s)` patterns | `0261`, `0516`, `0721`, `1203`, `2092`, `2709` | removes dead `None` guards and sentinel workarounds around initialized maps |
| `O1` | owned collection helpers with explicit ownership signatures | fixtures using drains/takes/splits | helpers compile without `Rc<RefCell<...>>` or interior mutability |
| `S1` | heap / priority queue stdlib | `0355`, `0743`, `0778`, `1584`, `1631`, `0295` | canonical heap algorithms no longer hand-roll priority queues |
| `S2` | DSU / union-find stdlib helper | `0261`, `0721`, `1489`, `2092`, `2709` | hand-rolled parent/rank boilerplate disappears |
| `S3` | deque stdlib | `0286`, `0513`, `0752` | BFS fixtures no longer emulate queue with list/head cursor |
| `S4` | character predicates | `0394`, `1397` | manual digit/alpha ladders removed |
| `S5` | whole-token integer parsing returning `Result` | `0150`, `0297` | manual token parsers removed without exceptions/panics |
| `S6` | trie decision and API | `0208`, `0211`, `0212`, `1397` | choose explicit `Trie` type or explicit nested-dict helpers; no auto-insert-on-read |
| `C1` | dummy-head cursor over owned linked-list chains | `0021`, `0023`, `0024` | canonical cursor rewrites compile without shared mutable aliases |
| `C2` | in-place `.next` skip under double narrowing | `0019`, `0203` | skip/delete patterns compile in canonical form |
| `C3` | sub-range rewire/reverse over owned chains | `0025`, `0092`, `1669`, `1721` | reverse/splice fixtures no longer drain to arrays |
| `R1` | structural recursion over owned chains/trees with read-only reborrows | `0450`, `0669`, `0894` boundary-adjacent reads | recursive traversal is explicit about borrow/ownership mode |
| `B1` | shared fixture-helper convention | linked-list and tree rewrites | decide fixture prelude/import vs self-contained duplication before rewrite PRs |

No feature ID in this plan unlocks shared/cyclic object identity. If Sifr later chooses a safe arena / handle model for non-owning graph references, Category 4b can be revisited as a separate language-design phase; it is not assumed here.

## Rewrite Debt Execution Table

Each rewrite needs updated fixture inputs, assertions, and a pair-scan verification after implementation.

| Fixture | Canonical Target | Key Prereqs | Acceptance |
| --- | --- | --- | --- |
| `0004_median_of_two_sorted_arrays` | binary-partition median, `O(log(min(m,n)))` | `I1`, numeric sentinel/Result conventions | no full merge; assertions cover odd/even and empty-side cases |
| `0023_merge_k_sorted_lists` | min-heap or divide-and-conquer merge of `ListNode` heads | `S1` or pairwise merge choice, `C1`, `N2`, `B1` | public input/output use `ListNode`; result chain matches canonical values |
| `0024_swap_nodes_in_pairs` | dummy-cursor pairwise node rewiring | `C1`, `N2`, `B1` | no `list[int]` public model; one-node and empty cases preserved |
| `0146_lru_cache` | `O(1)` LRU cache using hashmap plus explicit recency structure | `I2`, `O1`, recency-structure design | no linear scans or array shifts in `get` / `put`; eviction is `O(1)` |
| `0147_insertion_sort_list` | linked-list insertion sort over nodes | `C1`, `C2`, `N2`, `B1` | no drain/sort/rebuild; node-chain output sorted |
| `0148_sort_list` | linked-list merge sort over nodes | `C1`, `C3`, `N2`, `B1` | no flatten/sort/rebuild; merge-sort shape retained |
| `0208_implement_trie_prefix_tree` | trie with per-character child traversal | `S6` | no word-list scan; insert/search/prefix operations walk trie nodes |
| `0206_reverse_linked_list` | in-place node-chain reversal | `C1`, `N2`, `B1` | no `list[int]` public model; canonical pointer reversal |
| `0211_design_add_and_search_words_data_structure` | trie plus wildcard DFS for `.` branches | `S6`, `I2` | no per-word linear scan; wildcard branches traverse trie children |
| `0212_word_search_ii` | trie/prefix-pruned board search | `S6`, `I2` | no per-word full-board search; prefix pruning exercised |
| `0295_find_median_from_data_stream` | dual-heap median finder | `S1` | update is heap-based, not sorted-array insertion |
| `0706_design_hashmap` | hashmap implementation without delegating to built-in `dict` | `I1`, `O1`, bucket/open-addressing design | `put` / `get` / `remove` do not call built-in dict; `remove` deletes rather than writing `-1` |
| `0707_design_linked_list` | linked-list data-structure operations | `C1`, `C2`, `B1` | operation-cost profile matches linked-list design intent |

## Practical Priority Order

This is a work sequence, not a severity ranking. Rewrites should land as soon as their prerequisites are available; they do not need to wait for every ergonomics wave.

1. Wave 0: Corpus normalization

- mark explicit non-canonical parity-debt fixtures clearly
- normalize helper-boilerplate noise in comparison scripts
- stop treating raw diff buckets as calibrated severity
- complete the four Category 3/5 fixtures in one cleanup PR, then re-run the scan and commit the new baseline

2. Wave 1: Shared narrowing design and diagnostics

- write the `D0` spec before implementing 2a or 2b narrowing
- define invalidating writes/calls, alias-mutation boundaries, and diagnostic wording
- confirm borrow-by-default / ownership prerequisites are mature enough for cursor work

3. Wave 2: Local Optional-flow cleanup

- implement `N1` through `N4`, `I1`, and `I2` in small slices with fixture-specific acceptance tests
- expected early wins: dead Optional-style guards on proven list/dict access and simple cursor advancement

4. Wave 3: Stdlib wave A plus unlocked rewrites

- implement `S1` heap and `S2` DSU first; these unblock the largest cluster of graph/priority-queue fixtures
- immediately rewrite unlocked cases such as `0295_find_median_from_data_stream` after `S1`

5. Wave 4: Owned-chain cursor ergonomics and helper convention

- decide `B1` before rewriting linked-list and tree fixtures
- implement `C1`, `C2`, `C3`, and `R1` as separate features
- rewrite linked-list parity-debt fixtures as their cursor prerequisites land

6. Wave 5: Stdlib wave B and string helpers

- implement `S3` deque, `S4` character predicates, and `S5` whole-token integer parsing returning `Result`
- clean up BFS and parser fixtures without adding exceptions or panics

7. Wave 6: Trie decision and trie-dependent rewrites

- decide `S6` before implementing `0208_implement_trie_prefix_tree`, `0211_design_add_and_search_words_data_structure`, and `0212_word_search_ii`
- reject auto-insert-on-read semantics even if nested dictionaries are used

8. Wave 7: Final rewrite sweep and closure metrics

- finish any remaining Category 1 rewrite table rows
- run the pair scan again and compare against the 2026-04-09 baseline
- close only when high-diff outliers are explained, rewrite-debt fixtures are canonical, and no new compile/run failures are introduced

## Not In This Plan

- Fresh compile/run failures from a new LeetCode execution pass. Track those separately and cross-link them here only when the root cause is one of these divergence categories.
- The 16 `sifr_only` `_v2` fixtures. Triage whether they are deliberate Sifr-native alternates, orphaned fixtures needing Python pairs, or deletion candidates.
- Below-cutoff fixtures not explicitly listed as parity-debt exceptions or Category 4 continuity examples.

## Boundaries To Preserve

- Do not add Python-style truthiness coercions.
- Do not add implicit nullable access.
- Do not weaken ownership to emulate Python aliasing.
- Do not change the abstract return type of `list` / `dict` subscripts; Optional-flow narrowing is local, not universal.
- Do not introduce interior mutability (`Rc<RefCell<...>>`, `Cell`, etc.) into owned-collection or cursor ergonomics; if an ergonomics goal requires it, the goal is out of scope.
- Do not add auto-insert-on-read dictionary semantics such as `defaultdict`; absent-key reads must remain explicit.
- Do not treat every high diff as a language problem before separating:
  - corpus noise
  - stdlib parity gaps
  - real language ergonomics gaps
  - explicit rewrite debt
