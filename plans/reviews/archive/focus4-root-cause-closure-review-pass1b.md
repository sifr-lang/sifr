# Review: Focus4 Root-Cause Closure - Implementation Readiness (Pass 1b)

**Phase**: `ad_hoc_focus4_root_cause_closure`
**Date**: 2026-04-06
**Reviewer**: agent (automated deep review)
**Verdict**: **Mostly Ready**

The bucket breakdown counts are internally consistent (90 fixtures, ownership split 64/15/11), the workstream structure is sound, and the root-cause taxonomy is directionally correct. However, the phase **cannot achieve zero failures across all 4 buckets** without addressing the findings below. These are planning/documentation gaps, not fundamental design flaws.

---

## Findings (ordered by severity)

### F1 - CRITICAL: Cross-bucket contamination not accounted for

The root-cause map assigns each fixture to exactly one sub-root-cause. But many fixtures carry diagnostics from **multiple target buckets**. Fixing the assigned root cause migrates the fixture to another target bucket rather than resolving it. The doc's implied promise that fixing sub-root-cause X resolves its N fixtures is incorrect for multi-rooted fixtures.

**Identified cross-contaminated fixtures** (primary assignment -> secondary blocker bucket):

| Fixture | Assigned | Secondary blocker(s) in target buckets | Migration target |
|---------|----------|---------------------------------------|-----------------|
| 0355_design_twitter | AU-3 | CF-1 (`has no field 'followMap'`, `'tweetMap'`), RF-2 (`undefined variable: 'index'`, `'tweetId'`) | class_field -> return_path |
| 1489_find_critical_...mst | AU-4 | CF-1 (`has no field 'par'`, `'rank'`), DS-1 (`tuple target...got 'list[int]'`) | class_field -> destructuring |
| 0056_merge_intervals | AU-1 | DS-1 (`tuple target...got 'list[int]'`) | destructuring |
| 0253_meeting_rooms_ii | AU-1 | DS-1 (`tuple target...got 'list[int]'`) | destructuring |
| 1029_two_city_scheduling | AU-1 | DS-1 (`tuple target...got 'list[int]'`) | destructuring |
| 1288_remove_covered_intervals | AU-1 | DS-1 (`tuple target...got 'list[int]'`) | destructuring |
| 1851_minimum_interval...query | AU-1 | DS-2 (`cannot unpack non-tuple type 'list[int]'`), RF-2 (`undefined variable: 'r'`) | destructuring -> return_path |
| 2092_find_all_people_with_secret | AU-2 | DS-1 (`tuple target...got 'list[int]'`), RF-2 (`undefined variable: 'visit'`) | destructuring -> return_path |
| 2101_detonate_the_maximum_bombs | AU-2 | DS-2 (`cannot unpack non-tuple type 'list[int]'`), RF-2 (`undefined variable: 'dst'`) | destructuring -> return_path |
| 0516_longest_palindromic_subseq | DS-3 | RF-1 (`duplicate function definition`) | return_path |
| 0895_maximum_frequency_stack | DS-3 | CF-1 (`has no field 'cnt'`, `'stacks'`), RF-2 (`undefined variable: 'res'`, `'valCnt'`) | class_field -> return_path |
| 1396_design_underground_system | DS-3 | CF-1 (`has no field 'customer'`, `'time'`), RF-2 (`undefined variable: 'route'`, `'start'`, `'total'`) | class_field -> return_path |
| 2013_detect_squares | DS-3 | CF-1 (`has no field 'pts'`), DS-2 (`cannot unpack non-tuple type 'list[int]'`) | class_field / destructuring |
| 0622_design_circular_queue | DS-5 | CF-1 (`has no field 'capacity'`, `'head'`, `'size'`, `'tail'`) | class_field |
| 0323_number_of_connected_components | DS-1 | CF-1 (`has no field 'f'`), RF-2 (`undefined variable: 'y'`) | class_field -> return_path |
| 0076_minimum_window_substring | DS-2 | RF-3 (`must return...on all control-flow paths`), RF-2 (`undefined variable: 'r'`) | return_path |
| 0286_walls_and_gates | DS-2 | RF-2 (`undefined variable: 'r'`) | return_path |
| 2709_greatest_common_divisor_traversal | RF-3 | CF-1 (`has no field 'count'`, `'par'`, `'size'`) | class_field |
| 1481_least_number_unique_integers | RF-1 | RF-3 (`must return...on all control-flow paths`) | return_path (same bucket) |
| 0162_find_peak_element | RF-3 | RF-2 (`undefined variable: 'mid'`) | return_path (same bucket) |

**Impact**: ~20 of 90 fixtures (22%) have cross-bucket contamination. After fixing their assigned root cause, they will inflate counts in other target buckets. All 4 buckets can still reach zero only if every workstream is complete, but intermediate validation gates will be misleading.

**Required doc edit**: Add a "Cross-Bucket Dependency Matrix" section listing these fixtures and documenting that:
1. Per-workstream acceptance criteria should measure "fixtures no longer exhibiting workstream-specific diagnostics" rather than "fixtures leave the bucket"
2. A final full-corpus rerun after ALL workstreams is the only valid zero-check
3. The execution order should account for cascading (A/B fixtures will inflate C/D counts mid-phase)

---

### F2 - SIGNIFICANT: DS-1/DS-2 compiler vs adaptation split is unspecified

15 fixtures are marked `resolution_mode: both` (DS-1: 8, DS-2: 7) but the phase plan routes all of them to "mixed/adaptation lanes" under Workstream D. There is no per-fixture decision stating which specific ones need compiler changes and which need fixture canonicalization.

The core question is unanswered: **Does Sifr intend to support `for x, y in list_of_lists` (list-element destructuring)?**

- If yes: these are compiler fixes, not adaptation.
- If no: these need fixture rewrites to use tuples or explicit indexing.
- If "depends on pattern": the doc must enumerate which patterns are policy-restricted.

**Affected fixtures** (DS-1, all showing `for loop tuple target expects iterable elements of tuple type, got 'list[T]'`):
0012, 0323, 0787, 0994, 1091, 1462, 1466, 2001

**Affected fixtures** (DS-2, all showing `cannot unpack non-tuple type 'list[T]'`):
0076, 0286, 0673, 0752, 0909, 0929, 1260

**Required doc edit**: For each of the 15 DS-1/DS-2 fixtures, add a column to `phase_apr06_focus4_root_cause_map.csv` (or a new table in the phase spec) with:
- `resolution_path`: `compiler_list_destructure_support` | `adaptation_to_tuple` | `adaptation_to_index`
- A policy statement: "Sifr [does/does not] support list-element destructuring in for-loop targets. Fixtures requiring this form are resolved via [compiler/adaptation]."

---

### F3 - SIGNIFICANT: RF-3 implementation spec too thin for 11-fixture scope

RF-3 (`return_completeness_false_positive`) is the largest single sub-root-cause (11 fixtures) but the implementation spec only says "remove false positives from return-path completeness analysis." There is no sub-pattern catalog.

Analysis of the 11 RF-3 fixtures reveals at least 3 distinct patterns:

**Pattern A - Pure while-loop binary search** (function returns inside while-loop and has a post-loop return that the analyzer misses):
- 0153_find_minimum_in_rotated_sorted_array
- 0367_valid_perfect_square
- 0167_two_sum_ii_input_array_is_sorted

**Pattern B - Accumulator with early-return branches** (function accumulates a result, returns conditionally in loop, always returns after):
- 0221_maximal_square
- 0347_top_k_frequent_elements
- 0463_island_perimeter
- 0918_maximum_sum_circular_subarray

**Pattern C - Fixtures that are NOT pure RF-3** (have additional root-cause blockers that may cause the false positive as a downstream effect):
- 0118_pascals_triangle (also has RF-2: `undefined variable: 'Len'`, `'ListPrec'`)
- 0162_find_peak_element (also has RF-2: `undefined variable: 'mid'`)
- 1572_matrix_diagonal_sum (also has AU-1-like: `len()...got 'Any'`, `+: 'Any' and 'Any'`)
- 2709_greatest_common_divisor_traversal (also has CF-1: `has no field 'count'`, `'par'`, `'size'`)

**Impact**: Pattern C fixtures (4 of 11) may not exhibit RF-3 after their real root cause is fixed. The true RF-3 count could be 7, not 11.

**Required doc edit**: Add a sub-pattern table to Workstream B listing which fixtures exhibit which return-path pattern (A/B/C). Flag Pattern C fixtures as "RF-3 may resolve as downstream effect of other fixes - verify after Workstreams A/C complete."

---

### F4 - MODERATE: AU-1 lumps 4+ distinct inference failure mechanisms under one ID

AU-1 covers 12 fixtures but the diagnostics reveal at least 4 distinct mechanisms:

1. **Container literal inference failure**: `list[Any]` instead of `list[int]` from `[]` initialization
   - 1137_n_th_tribonacci_number (`+: 'Any' and 'Any'` - array initialized as empty list)

2. **Dict/defaultdict value type erasure**: subscript returns `Any`
   - 2306_naming_a_company (`set[Any] | None` from dict access)

3. **Sort/iteration type loss**: sorted container loses element type through `.sort(key=...)` or iteration
   - 0056_merge_intervals, 0253_meeting_rooms_ii, 1029_two_city_scheduling, 1288_remove_covered_intervals (all involve sort + index)

4. **Stack/deque pop type erasure**: `.pop()` or deque indexing returns `Any`/`Any | None`
   - 0084_largest_rectangle, 0239_sliding_window, 0456_132_pattern, 0739_daily_temperatures, 0862_shortest_subarray

Each mechanism likely requires a different compiler-side fix. A single "preserve concrete element types" change won't cover all patterns.

**Required doc edit**: Split AU-1 into sub-mechanisms (AU-1a through AU-1d or similar) in the phase spec. This doesn't require CSV changes but the implementation goals should list each mechanism separately with its pilot fixture.

---

### F5 - MODERATE: Several CF-1 fixtures also have RF-2-like undefined variables

Many CF-1 fixtures show `undefined variable` diagnostics alongside the `has no field` errors:

| Fixture | CF-1 diagnostic | Also shows |
|---------|----------------|------------|
| 0208_implement_trie | `has no field 'root'` | `undefined variable: 'curr'` |
| 0303_range_sum_query | `has no field 'prefix'` | `undefined variable: 'r'` |
| 0706_design_hashmap | `has no field 'map'` | `undefined variable: 'cur'` |
| 0745_prefix_and_suffix_search | `has no field 'root'` | `undefined variable: 'cur'` |
| 0981_time_based_key_value_store | `has no field 'keyStore'` | `undefined variable: 'l'`, `'res'`, `'values'` |
| 1603_design_parking_system | `has no field 'parking'` | `undefined variable: 'new_total'` |

Question: Are these `undefined variable` errors **downstream effects** of CF-1 (field not registered -> subsequent attribute access on that field produces `undefined variable`)? Or are they independent RF-2 scope bugs?

If downstream: fixing CF-1 resolves them automatically. Good.
If independent: these fixtures also need RF-2 fixes and won't clear the CF bucket from CF-1 alone.

**Required doc edit**: Investigate 2-3 of these fixtures to determine causality. Add a note to Workstream C: "Several CF-1 fixtures also show `undefined variable` diagnostics. Pilot fixture verification must confirm whether these resolve as downstream effects of field registration or require independent scope fixes."

---

### F6 - MODERATE: Execution order doesn't account for cascade inflation

The proposed order is:
1. A (Any/Unknown) + B (scope/return) in parallel
2. C (class/object) after B
3. D compiler (DS-3) before adaptation sweep
4. E canonicalization last

But after Workstream A completes:
- ~5 fixtures migrate from AU bucket into DS bucket (0056, 0253, 1029, 1288, 1851 all have DS-1/DS-2 secondaries)
- ~2 fixtures migrate from AU bucket into CF bucket (0355, 1489 have CF-1 secondaries)

After Workstream D (DS-3) completes:
- ~3 fixtures migrate from DS bucket into CF bucket (0895, 1396, 2013 have CF-1 secondaries)

If validation gates run between workstreams, they will see **increasing** counts in downstream buckets, which could be alarming without context.

**Required doc edit**: Add to "Validation Gate" section: "Expected cascade: after Workstream A, DS bucket may temporarily increase by ~5 and CF bucket by ~2. After Workstream D, CF bucket may temporarily increase by ~3. These are known cross-contaminated fixtures, not regressions. The authoritative zero-check is the final full-corpus rerun after all workstreams."

---

### F7 - MINOR: 3 likely misclassifications

**0162_find_peak_element** (assigned RF-3, likely RF-2):
- Diagnostics: `function 'findPeakElement' must return a value of type 'int' on all control-flow paths` + `undefined variable: 'mid'`
- `mid` is defined inside a while-loop (binary search). The scope bug prevents the analyzer from seeing the `return nums[mid]` path, causing the false-positive return diagnostic.
- **Recommendation**: Reclassify as RF-2. The RF-3 diagnostic will likely resolve as a downstream effect.

**0118_pascals_triangle** (assigned RF-3, likely RF-2 or multi-root):
- Has `undefined variable: 'Len'`, `undefined variable: 'ListPrec'` alongside the return completeness error
- Variables named `Len` and `ListPrec` suggest unconventional naming that may interact with scope resolution
- `unsupported operand type(s) for -: 'Never' and 'int'` confirms an unresolved variable typed as `Never`
- **Recommendation**: Reclassify as RF-2 or mark as "multi-root: RF-2 + RF-3, resolve RF-2 first."

**1572_matrix_diagonal_sum** (assigned RF-3, likely AU-1 + adaptation):
- Has `len()...got 'Any'` and `+: 'Any' and 'Any'` alongside the return completeness error
- Also has missing parameter annotations on helper functions (`CrossSum`, `PrimeSum`)
- The `Any` propagation from untyped parameters causes downstream failures
- **Recommendation**: Reclassify as AU-1 or mark as "multi-root: missing annotations cause Any leak, which cascades to false-positive return analysis."

---

### F8 - MINOR: Fixtures with out-of-scope blockers should be flagged

These fixtures will migrate OUT of the 4 target buckets (good) but will NOT pass (expected). They should be pre-flagged to avoid false expectations:

| Fixture | In-scope root cause | Out-of-scope blocker |
|---------|-------------------|---------------------|
| 0621_task_scheduler | RF-1 (duplicate def) | `undefined function: 'Counter'` (stdlib gap) |
| 0496_next_greater_element_i | AU-3 | `cannot iterate over type 'Iterator[tuple[int, int]]'` (stdlib gap) |
| 2101_detonate_the_maximum_bombs | AU-2 | `undefined function: 'sqrt'` (stdlib gap) |
| 1466_reorder_routes | DS-1 | `recursive nested function 'dfs' cannot mutate captured state with 'nonlocal' yet` (nonlocal gap) |
| 0673_number_of_longest_increasing_subseq | DS-2 | `tuple unpacking cannot rebind captured state with 'nonlocal' yet` (nonlocal gap) |

**Required doc edit**: Add a "Known out-of-scope residuals" list in the phase spec noting these fixtures will exit the 4 target buckets but remain failing in other categories.

---

## Concrete Doc Edits Required

### Edit 1: Phase spec - Add cross-contamination matrix (CRITICAL)

Insert after "## Cross-cutting Findings" section:

```markdown
### Cross-Bucket Dependency Matrix

The following fixtures have diagnostics spanning multiple target buckets.
Their primary root cause determines workstream ownership, but full resolution
requires fixes from additional workstreams.

[Insert table from F1 above]

Implication: Per-workstream validation gates should verify that
workstream-specific diagnostics are eliminated, not that fixtures
leave the bucket entirely. The authoritative zero-check is the
final full-corpus rerun after all workstreams complete.
```

### Edit 2: Phase spec - Add DS-1/DS-2 policy decision (SIGNIFICANT)

Insert into Workstream D section:

```markdown
#### Language policy decision required before implementation

- List-element destructuring (`for x, y in list_of_lists`): [PENDING DECISION]
  - If supported: DS-1 and DS-2 become compiler fixes
  - If restricted: DS-1 and DS-2 become adaptation (rewrite to tuple or index access)
  - Per-fixture resolution path must be documented before adaptation begins
```

### Edit 3: Phase spec - Add RF-3 sub-pattern catalog (SIGNIFICANT)

Replace current Workstream B RF-3 description with:

```markdown
#### RF-3 sub-patterns

- **Pattern A** (pure binary-search / while-loop return): 0153, 0167, 0367
- **Pattern B** (accumulator with conditional return): 0221, 0347, 0463, 0918
- **Pattern C** (RF-3 likely downstream of other root cause - verify after
  Workstreams A/C): 0118 (RF-2), 0162 (RF-2), 1572 (AU-1/annotation),
  2709 (CF-1)

Implementation priority: Pattern A first (smallest, clearest fix),
then Pattern B. Pattern C fixtures should be re-evaluated after their
primary root causes are fixed in other workstreams.
```

### Edit 4: Phase spec - Add AU-1 sub-mechanism breakdown (MODERATE)

Insert into Workstream A section:

```markdown
#### AU-1 sub-mechanisms

- **AU-1a** (container literal type inference): empty `[]`/`{}` losing element type
- **AU-1b** (dict/defaultdict value type erasure): subscript access returning Any
- **AU-1c** (sort/iteration type loss): `.sort(key=...)` or sorted iteration losing element type
- **AU-1d** (stack/deque pop type erasure): `.pop()` or deque indexing returning Any/Any|None

Each mechanism likely requires a distinct compiler-side fix. Pilot fixtures per mechanism:
- AU-1a: 1137_n_th_tribonacci_number
- AU-1b: 2306_naming_a_company
- AU-1c: 0056_merge_intervals
- AU-1d: 0084_largest_rectangle_in_histogram
```

### Edit 5: Phase spec - Update validation gate for cascade awareness (MODERATE)

Add to "## Validation Gate" section:

```markdown
### Expected intermediate cascade effects

After Workstream A: DS bucket may temporarily increase by ~5, CF bucket by ~2
(known cross-contaminated fixtures migrating from AU bucket).

After Workstream D (DS-3): CF bucket may temporarily increase by ~3
(known cross-contaminated fixtures migrating from DS bucket).

These are expected migrations, not regressions. The authoritative zero-check
is the final full-corpus rerun after ALL workstreams complete.
```

### Edit 6: Phase spec - Add out-of-scope residuals (MINOR)

Add before "## Deliverables":

```markdown
## Known Out-of-Scope Residuals

These fixtures will exit the 4 target buckets but remain failing in other categories:

- 0621_task_scheduler: needs `Counter` (python_stdlib_and_builtin_parity_gap)
- 0496_next_greater_element_i: needs `Iterator[tuple]` iteration (python_stdlib)
- 2101_detonate_the_maximum_bombs: needs `sqrt()` (python_stdlib)
- 1466_reorder_routes: needs nonlocal mutable capture support
- 0673_number_of_longest_increasing_subseq: needs nonlocal capture support
```

### Edit 7: Root-cause map - Reclassify 3 fixtures (MINOR)

In `phase_apr06_focus4_root_cause_map.csv`:

| Fixture | Current | Recommended | Reason |
|---------|---------|-------------|--------|
| 0162_find_peak_element | RF-3 | RF-2 | `undefined variable: 'mid'` is the real blocker; RF-3 is downstream |
| 0118_pascals_triangle | RF-3 | RF-2 | `undefined variable: 'Len'`, `'ListPrec'` cause the return analysis failure |
| 1572_matrix_diagonal_sum | RF-3 | RF-3 + AU-1 (multi) | Missing param annotations cause Any leak; mark as multi-root |

If applied: RF-3 count drops from 11 to 8-9, RF-2 count increases from 6 to 8.

---

## Reclassification Summary

| Change | From | To | Fixtures affected | Net count impact |
|--------|------|----|-------------------|-----------------|
| RF-3 -> RF-2 | RF-3 (11) | RF-2 (6) | 0162, 0118 | RF-3: 9, RF-2: 8 |
| RF-3 -> multi(RF-3+AU-1) | RF-3 (11) | multi | 1572 | RF-3: 8 pure |

---

## Verdict Summary

| Criterion | Assessment |
|-----------|-----------|
| Bucket counts accurate | Yes - all 90 fixtures accounted for, CSV/JSON consistent |
| Root-cause taxonomy quality | Good for primary causes; misses multi-root fixtures |
| Ownership split quality | Directionally correct; DS-1/DS-2 policy decision pending |
| Implementation detail level | Insufficient for RF-3 (11 fixtures) and AU-1 (12 fixtures) |
| Cross-bucket awareness | Missing - ~22% of fixtures are cross-contaminated |
| Execution order soundness | Order is correct; cascade effects undocumented |
| Achievability of zero-failures | Yes, if all edits above are applied and all workstreams complete |

**Final verdict: Mostly Ready**

The phase plan is structurally sound and the analysis work is high quality. Apply Edits 1-3 (critical/significant) before beginning implementation. Edits 4-7 should be applied before the first validation gate. No fundamental redesign needed.
