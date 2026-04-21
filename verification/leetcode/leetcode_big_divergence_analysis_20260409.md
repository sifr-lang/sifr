# LeetCode Big-Divergence Analysis

Date: 2026-04-09
Source ranking: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Method: raw line-level diff size between paired `audits/leetcode/*.py` and `audits/leetcode/*.sifr` files

## Scope

- Paired fixtures scanned: `395`
- Raw-diff outliers:
  - `16` pairs at `>= 120` changed lines
  - `33` pairs at `>= 100` changed lines
  - `53` pairs at `>= 80` changed lines

This scan is only a first-pass heuristic. High raw diff does not always mean semantic drift:

- some Python fixtures contain multiple alternative solutions in the same file
- some Python fixtures include helper baggage shared across problem families
- some Sifr fixtures add explicit safety scaffolding that preserves the same algorithmic shape
- the current metric is raw `changed_py_lines + changed_sifr_lines` with no normalization for comments, imports, formatting, or embedded helper boilerplate
- mirrored helper baggage can hide real divergence because shared dead code cancels out of the diff entirely

Because of that, the `>= 120`, `>= 100`, and `>= 80` buckets are triage buckets, not calibrated design thresholds.

## Main Divergence Categories

### 1. Safety scaffolding and surface-ergonomics inflation

Representative cases:

- `1397_find_all_good_strings`
- `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree`
- `1203_sort_items_by_groups_respecting_dependencies`
- `1631_path_with_minimum_effort`
- `0721_accounts_merge`
- `0778_swim_in_rising_water`

Why they diverge:

- Sifr versions often expand small Python expressions into helper layers such as `unwrapInt`, `charAt`, `getBucket`, `setIntAt`, `mapGetInt`, and explicit queue/index plumbing.
- This is usually caused by the need to make Optional flow, indexing, and collection access explicit rather than dynamic or implicit.
- The core algorithm is often still recognizable, but the file grows because every data access must be made locally total or explicitly narrowed.

What this says about Sifr:

- This is mostly not a request for Python dynamism.
- It is a request for better safe ergonomics around already-static operations.
- The missing surface is closer to "safe expression power" than "Python compatibility at any cost".
- Some of the missing pieces here are stdlib parity rather than language features. For example, `deque`, `heapq`, and `defaultdict`-style support should not be misread as a reason to add dynamic Python semantics to the core language.

Language direction:

- Adjust the language and compiler here, but only through explicit safe rules.
- Good candidates:
  - stronger flow-sensitive narrowing after `is not None`
  - safer indexing ergonomics after proven bounds checks
  - more direct field and collection access once local safety is established
  - stdlib parity for core algorithmic tools such as queues, heaps, and common map/list helpers
- Non-goal:
  - Python decorator parity by itself is not the objective. For example, `@cache` in Python is evidence that memoization should be ergonomic, not evidence that Sifr must copy Python's decorator model.
- Do not copy Python's permissive nullable or truthiness behavior just to shrink code.

### 2. Linked-structure ownership and cursor friction

Representative cases:

- `0002_add_two_numbers`
- `0021_merge_two_sorted_lists`
- `0025_reverse_nodes_in_k_group`
- `0061_rotate_list`
- `0092_reverse_linked_list_ii`
- `0143_reorder_list`
- `0148_sort_list`
- `0160_intersection_of_two_linked_lists`
- `1669_merge_in_between_linked_lists`
- `0297_serialize_and_deserialize_binary_tree`

Why they diverge:

- Python freely traverses and rewires object graphs with nullable pointers.
- Sifr keeps ownership and mutability explicit, so linked-list and tree manipulation becomes much heavier.
- In several files, local helper functions exist mainly to compensate for awkward Optional field access and cursor updates.

What this says about Sifr:

- These cases are not an argument against ownership.
- They are an argument that current ownership ergonomics for recursive object graphs are still too expensive in common algorithmic code.
- This is the highest-leverage category because many top raw-diff outliers cluster here.

Language direction:

- Adjust the language and compiler here.
- The target should be safe cursor-oriented manipulation, not Python aliasing semantics.
- Good candidates:
  - better narrowing for `node is not None` so `node.next` and `node.val` become directly usable in that region
  - safe local rebinding patterns for cursor advancement
  - clearer ownership conventions for temporary traversal versus structural mutation
  - direct field-expression support on narrowed recursive nodes
- Do not weaken ownership or introduce implicit shared mutable aliasing.

### 3. Material algorithm or data-model substitution

Representative cases:

- `0148_sort_list`
- `0023_merge_k_sorted_lists`
- `0212_word_search_ii`
- `0295_find_median_from_data_stream`

Why they diverge:

- The Sifr version is not just more verbose; it solves the problem in a materially different way.
- `0148_sort_list`: Python uses linked-list merge sort, while Sifr flattens to values, sorts, and rebuilds the list.
- `0023_merge_k_sorted_lists`: Python uses `ListNode` inputs and linked-list merging, while Sifr changes the interface to `list[list[int]]` and sorts flattened values.
- `0212_word_search_ii`: Python uses a trie with prefix pruning, while Sifr uses per-word board search.
- `0295_find_median_from_data_stream`: Python uses heaps, while Sifr keeps a sorted array and inserts into position.

What this says about Sifr:

- This is where parity risk is real.
- The fixture may still pass, but the representation, asymptotics, or public surface no longer mirrors the Python source closely.
- If the long-term goal is "Sifr can express the same canonical algorithms safely", these are the cases that matter most.

Language direction:

- Some of this is language/compiler work.
- Some of it is stdlib surface work.
- Some of it is corpus debt where a temporary Sifr-safe solution remained in place.
- The key is not to lump these together:
  - `0148` and `0023` primarily expose recursive-object and interface-expression pain
  - `0212` and `0295` also expose missing efficient algorithmic primitives and data-structure ergonomics

Recommended stance:

- `0148`, `0023`, `0212` should move closer to the Python shape over time.
- `0295` should likely move closer as soon as heap support is ergonomic enough.
- None of these require adopting dynamic Python behavior.
- They require that Sifr can express the same data structures and traversal/mutation patterns safely and without excessive ceremony.

### 4. Reference-fixture noise in the Python side

Representative cases:

- `0200_number_of_islands`
- several linked-list files that include shared `Node` helper classes unrelated to the actual algorithm

Why they diverge:

- Some Python fixtures contain multiple complete solutions in one file.
- Some contain generic helper classes copied across many LeetCode problems.
- Raw line diff treats all of that as meaningful divergence even when the Sifr version kept one canonical solution.

Language direction:

- Do not change the language for this.
- Clean the corpus or use a normalized comparison pass before treating these as design signals.

## Per-Problem Readout For The Highest Raw-Diff Cases

| Problem | Main driver | Should language/stdlib change? | Notes |
| --- | --- | --- | --- |
| `1397_find_all_good_strings` | safety scaffolding | yes | mostly explicit string/index/Optional helper inflation |
| `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree` | safety scaffolding | yes | likely container and graph-helper ergonomics more than semantics |
| `1203_sort_items_by_groups_respecting_dependencies` | safety scaffolding | yes | explicit queue, list access, group graph helpers expand code |
| `1631_path_with_minimum_effort` | safety scaffolding | yes | likely priority-queue and grid-access ergonomics |
| `0721_accounts_merge` | safety scaffolding | yes | manual union-find map helpers and account extraction boilerplate |
| `0778_swim_in_rising_water` | safety scaffolding | yes | likely priority-queue and grid traversal verbosity |
| `0148_sort_list` | algorithm substitution + recursive-object friction | yes | should move toward canonical linked-list sort shape |
| `0025_reverse_nodes_in_k_group` | recursive-object friction | yes | likely blocked by cursor/rewiring ergonomics |
| `0212_word_search_ii` | algorithm substitution | yes | trie and pruning support should be expressible safely |
| `0143_reorder_list` | recursive-object friction | yes | linked-list rewiring remains expensive |
| `0061_rotate_list` | recursive-object friction | yes | same cluster as other list rewiring problems |
| `0002_add_two_numbers` | recursive-object friction | yes | nullable node traversal and output building are still noisy |
| `0092_reverse_linked_list_ii` | recursive-object friction | yes | same cluster |
| `0297_serialize_and_deserialize_binary_tree` | recursive-object friction | yes | tree/object handling still expands heavily |
| `0021_merge_two_sorted_lists` | recursive-object friction | yes | same cluster |

## Recommended Priority Order

1. Recursive node and cursor ergonomics

- This hits the largest number of high-divergence linked-list and tree problems.
- It would reduce both verbosity and algorithm substitution pressure.

2. Safe collection and Optional expression ergonomics

- This would shrink many graph, DP, and union-find style solutions without violating Sifr's explicitness.

3. Stdlib parity for algorithmic primitives

- Priority queue / heap
- queue / deque
- tighter map/list helper surface
- trie-friendly dictionary ergonomics and other common competitive-programming containers

4. Corpus cleanup for comparison quality

- Normalize Python references to one canonical solution per file before using raw diffs as a design signal.

## What We Should Not Do

- We should not add Python-style implicit nullable access.
- We should not add dynamic truthiness coercions just to save lines.
- We should not weaken ownership to imitate Python aliasing.
- We should not treat every raw diff outlier as a language defect without checking for corpus noise or intentional temporary simplification.

## Current Conclusion

The top raw-diff cases do not point to one single problem. They split into:

- safe-but-verbose expression surface gaps
- ownership ergonomics gaps for recursive object graphs
- temporary algorithm/data-model substitutions where Sifr could not yet express the canonical Python approach cleanly
- noisy Python reference files

The most important design signal is not "be more dynamic like Python". It is:

- keep Sifr explicit and safe
- make statically safe operations more directly expressible
- improve recursive object and collection ergonomics enough that canonical Python algorithms remain natural to port
