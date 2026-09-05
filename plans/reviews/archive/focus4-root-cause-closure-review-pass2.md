# Review: Focus-4 Root-Cause Closure Package (Pass 2)

Reviewer: agent
Date: 2026-04-06

## 1. File Presence and Readability

| File | Status |
|---|---|
| `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06.md` | Present, readable, 264 lines |
| `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06-execution.md` | Present, readable, 49 lines |
| `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun1.json` | Present, readable, 1096 lines |
| `verification/leetcode/phase_apr06_focus4_full_diagnostics.json` | Present, readable, 460 lines |
| `verification/leetcode/phase_apr06_focus4_root_cause_map.csv` | Present, readable, 92 lines (header + 91 data rows) |

Verdict: **All 5 files present and machine-readable. No structural corruption.**

---

## 2. Category Breakdown Validation

### 2a. Taxonomy-to-spec count alignment

| Category | Taxonomy JSON | Phase Spec | Match |
|---|---|---|---|
| `any_unknown_typing_and_container_specialization_gap` | 26 | 26 | YES |
| `destructuring_and_assignment_target_surface_gap` | 24 | 24 | YES |
| `return_path_and_function_contract_gap` | 24 | 24 | YES |
| `class_field_state_and_object_layout` | 16 | 16 | YES |
| **Total in-scope** | **90** | **90** | **YES** |

### 2b. Sub-root-cause counts (spec vs CSV)

| Root-cause ID | Spec count | CSV count | Match |
|---|---|---|---|
| AU-1-any_element_type_erasure | 12 | 12 | YES |
| AU-2-unknown_flow_leak | 4 | 4 | YES |
| AU-3-optional_any_bridge_leak | 6 | 6 | YES |
| AU-4-container_shape_specialization_leak | 4 | 4 | YES |
| CF-1-class_field_registration_gap | 14 | 14 | YES |
| CF-2-nested_attribute_assignment_gap | 2 | 2 | YES |
| DS-1-list_pair_destructure_requires_tuple | 8 | 8 | YES |
| DS-2-list_unpack_requires_tuple | 7 | 7 | YES |
| DS-3-augassign_subscript_lowering_gap | 5 | 5 | YES |
| DS-4-unpack_target_shape_restriction | 3 | 3 | YES |
| DS-5-chained_assignment_restriction | 1 | 1 | YES |
| RF-1-duplicate_solution_definitions | 7 | 7 | YES |
| RF-2-loop_local_scope_resolution_bug | 6 | 6 | YES |
| RF-3-return_completeness_false_positive | 11 | 11 | YES |
| **Grand total** | **90** | **90** | **YES** |

### 2c. Resolution-mode totals (spec vs CSV)

| Resolution mode | Spec | CSV | Match |
|---|---|---|---|
| compiler | 64 | 64 | YES |
| both | 15 | 15 | YES |
| adaptation | 11 | 11 | YES |

### 2d. Diagnostics JSON coverage

All 90 in-scope fixture slugs from the taxonomy and CSV appear as keys in `phase_apr06_focus4_full_diagnostics.json`. No fixture is missing diagnostic detail. **Full coverage confirmed.**

### 2e. Taxonomy-to-CSV fixture-level cross-reference

Every fixture slug listed under the four focus-4 categories in the taxonomy JSON has a corresponding row in the CSV root-cause map. No orphans in either direction. **Bijective mapping confirmed.**

Verdict: **Category breakdown is internally consistent across all 4 artifacts. No count mismatches, no orphaned fixtures.**

---

## 3. Root-Cause Mapping Quality

### 3a. Strengths

1. **Clean sub-root-cause taxonomy**: 14 sub-root-cause IDs with clear naming, no ambiguous catch-alls.
2. **Resolution-mode discipline**: Each sub-root-cause has a definite `compiler`/`both`/`adaptation` assignment. The adaptation scope is appropriately narrow (11/90 = 12%).
3. **Representative failures**: The phase spec cites concrete fixture examples per category. All cited fixtures exist in the data.
4. **Workstream decomposition**: 5 workstreams with explicit ownership, targets, acceptance criteria, and execution ordering.

### 3b. Critical quality issue: single-root-cause assignment masks multi-root-cause fixtures

The CSV maps each fixture to exactly one `root_cause_id`. However, the diagnostics JSON reveals that **many fixtures have diagnostics spanning multiple sub-root-causes or even multiple top-level categories**. Fixing the assigned root cause alone will not make these fixtures pass -- they will migrate to a different failure bucket rather than go green.

#### Multi-root-cause fixtures (cross-workstream blockers)

The following fixtures are assigned to one root-cause but have **blocking** diagnostics from other root-causes that also need resolution:

**Fixtures assigned outside CF but blocked by CF-1 (class field registration):**

| Fixture | Assigned root-cause | CF-1 diagnostics present |
|---|---|---|
| `0622_design_circular_queue` | DS-5 (adaptation) | `has no field 'capacity'`, `'head'`, `'size'`, `'tail'` |
| `0895_maximum_frequency_stack` | DS-3 (compiler) | `has no field 'cnt'`, `'stacks'` |
| `1396_design_underground_system` | DS-3 (compiler) | `has no field 'customer'`, `'time'` |
| `2013_detect_squares` | DS-3 (compiler) | `has no field 'pts'` |
| `0355_design_twitter` | AU-3 (compiler) | `has no field 'followMap'`, `'tweetMap'` |
| `0323_number_of_connected_components` | DS-1 (both) | `has no field 'f'` |
| `1489_find_critical_and_pseudo_critical_edges` | AU-4 (compiler) | `has no field 'par'`, `'rank'` |
| `2709_greatest_common_divisor_traversal` | RF-3 (compiler) | `has no field 'count'`, `'par'`, `'size'` |

These 8 fixtures (across workstreams A, B, D) cannot pass without Workstream C (CF-1) also completing.

**Fixtures assigned outside RF-2 but blocked by RF-2 (undefined variable / loop-local scope):**

| Fixture | Assigned root-cause | RF-2 diagnostics present |
|---|---|---|
| `0162_find_peak_element` | RF-3 (compiler) | `undefined variable: 'mid'` |
| `0895_maximum_frequency_stack` | DS-3 (compiler) | `undefined variable: 'res'`, `'valCnt'` |
| `0981_time_based_key_value_store` | CF-1 (compiler) | `undefined variable: 'l'`, `'res'`, `'values'` |
| `1396_design_underground_system` | DS-3 (compiler) | `undefined variable: 'route'`, `'start'`, `'total'` |
| `1603_design_parking_system` | CF-1 (compiler) | `undefined variable: 'new_total'` |
| `0706_design_hashmap` | CF-1 (compiler) | `undefined variable: 'cur'` |
| `0745_prefix_and_suffix_search` | CF-1 (compiler) | `undefined variable: 'cur'` |

**Fixtures assigned to focus-4 root-causes but also blocked by out-of-scope categories:**

| Fixture | Assigned root-cause | Out-of-scope diagnostic |
|---|---|---|
| `0402_remove_k_digits` | AU-3 | `if condition must be bool...got 'int'` (operator_and_truthiness) |
| `0735_asteroid_collision` | AU-3 | `if condition must be bool...got 'int'` (operator_and_truthiness) |
| `0909_snakes_and_ladders` | DS-2 | `if condition must be bool...got 'int'` (operator_and_truthiness) |
| `0621_task_scheduler` | RF-1 | `undefined function: 'Counter'` (python_stdlib_parity) |
| `1481_least_number_of_unique_integers` | RF-1 | `undefined function: 'Counter'` (python_stdlib_parity) |
| `0496_next_greater_element_i` | AU-3 | `cannot iterate over type 'Iterator[...]'` (python_stdlib_parity) |
| `0221_maximal_square` | RF-3 | `min() takes 1 or 2 arguments` (python_stdlib_parity) |
| `2101_detonate_the_maximum_bombs` | AU-2 | `undefined function: 'sqrt'` (python_stdlib_parity) |
| `1572_matrix_diagonal_sum` | RF-3 | missing annotations (multiple) + `Any` arithmetic |

**Impact**: At minimum **17 of the 90 fixtures** (19%) have confirmed cross-workstream or out-of-scope blocking diagnostics. The actual number is likely higher -- many `undefined variable` errors in the diagnostics JSON could be either RF-2 (loop-local scope) or downstream effects of CF-1 (field not registered -> methods fail -> variables never bound). The true "will go green after single workstream" count is closer to ~60-65, not 90.

### 3c. DS-1/DS-2 "both" resolution boundary is unspecified

The spec marks DS-1 and DS-2 as `resolution_mode=both` but does not define:
- Which fixtures get compiler fixes (e.g., accepting `list` in tuple-destructuring position)?
- Which fixtures get adaptation (converting `list` literals to `tuple` literals)?
- Whether the compiler fix is "accept list[T] as destructurable" (language surface change) or "infer tuple type from list literal in specific positions" (inference improvement)?

This is a blocking ambiguity for Workstream D implementation.

---

## 4. Gaps Blocking Implementation Readiness

### GAP-1: No cross-workstream dependency matrix (BLOCKING)

**Problem**: The phase spec and execution ledger track fixtures per sub-root-cause independently. There is no artifact that answers: "Which fixtures need fixes from multiple workstreams before they can pass?" Without this, validation gates after each workstream will produce systematically misleading results -- fixtures that are "fixed" for their assigned root cause will still fail from cross-cutting diagnostics.

**Impact**: High. Affects validation gate design, progress measurement, and execution ordering.

### GAP-2: Acceptance criteria conflate "exits category" with "passes" (BLOCKING)

**Problem**: Acceptance criteria like "all AU-* fixtures leave any_unknown_typing... gap" are ambiguous. A fixture can "leave" the AU bucket by having its primary AU diagnostic fixed, only to surface as a CF-1 or RF-2 failure. The acceptance criteria do not distinguish between:
- (a) the fixture's primary diagnostic is resolved (exits this category)
- (b) the fixture passes completely (all diagnostics clear)

**Impact**: High. Without this distinction, workstream completion will be declared prematurely. Delta reports will show category count drops but mask category migrations.

### GAP-3: DS-1/DS-2 compiler-vs-adaptation boundary undefined (BLOCKING)

**Problem**: See section 3c above. The `both` resolution mode for 15 fixtures has no specification of what the compiler change should be vs. what adaptation should do.

**Impact**: Medium-high. Blocks Workstream D implementation start.

### GAP-4: Execution ledger missing multi-workstream fixture tracking (NON-BLOCKING but recommended)

**Problem**: The execution ledger has one checkbox per sub-root-cause. It does not track which fixtures are expected to remain failing after their primary workstream completes due to cross-workstream dependencies.

**Impact**: Medium. Affects progress reporting accuracy.

### GAP-5: No expected-outcome annotation per fixture (NON-BLOCKING but recommended)

**Problem**: No artifact records the expected outcome for each fixture after all focus-4 workstreams complete. Some fixtures (e.g., 0621_task_scheduler) will remain failing due to out-of-scope dependencies (Counter, stdlib parity). Without explicit expected-outcome annotations, the final validation rerun will show "still failing" fixtures that are actually correctly unresolved (out of scope), but they will be indistinguishable from implementation regressions.

**Impact**: Medium. Affects final validation gate interpretation.

---

## 5. Required Document Edits

### EDIT-1: Add cross-workstream dependency table to phase spec

**File**: `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06.md`

**Location**: New section after "Cross-cutting Findings" (after line 140), before "Ready-to-Implement Phase Plan"

**Content to add**:

```markdown
## Cross-Workstream Dependency Matrix

Fixtures whose full resolution requires fixes from multiple workstreams or that have
out-of-scope blocking diagnostics. These fixtures will NOT pass after their primary
workstream alone completes.

### Fixtures requiring Workstream C (CF-1) in addition to primary assignment

| Fixture | Primary assignment | Secondary blocker |
|---|---|---|
| 0323_number_of_connected_components | DS-1 (Workstream D) | CF-1 field: 'f' |
| 0355_design_twitter | AU-3 (Workstream A) | CF-1 fields: 'followMap', 'tweetMap' |
| 0622_design_circular_queue | DS-5 (Workstream E) | CF-1 fields: 'capacity', 'head', 'size', 'tail' |
| 0895_maximum_frequency_stack | DS-3 (Workstream D) | CF-1 fields: 'cnt', 'stacks' |
| 1396_design_underground_system | DS-3 (Workstream D) | CF-1 fields: 'customer', 'time' |
| 1489_find_critical_and_pseudo_critical_edges | AU-4 (Workstream A) | CF-1 fields: 'par', 'rank' |
| 2013_detect_squares | DS-3 (Workstream D) | CF-1 field: 'pts' |
| 2709_greatest_common_divisor_traversal | RF-3 (Workstream B) | CF-1 fields: 'count', 'par', 'size' |

### Fixtures requiring Workstream B (RF-2) in addition to primary assignment

| Fixture | Primary assignment | Secondary blocker |
|---|---|---|
| 0162_find_peak_element | RF-3 (Workstream B) | RF-2: undefined variable 'mid' |
| 0706_design_hashmap | CF-1 (Workstream C) | RF-2: undefined variable 'cur' |
| 0745_prefix_and_suffix_search | CF-1 (Workstream C) | RF-2: undefined variable 'cur' |
| 0895_maximum_frequency_stack | DS-3 (Workstream D) | RF-2: undefined variables 'res', 'valCnt' |
| 0981_time_based_key_value_store | CF-1 (Workstream C) | RF-2: undefined variables 'l', 'res', 'values' |
| 1396_design_underground_system | DS-3 (Workstream D) | RF-2: undefined variables 'route', 'start', 'total' |
| 1603_design_parking_system | CF-1 (Workstream C) | RF-2: undefined variable 'new_total' |

### Fixtures blocked by out-of-scope categories (will NOT pass after focus-4 closure)

| Fixture | Primary assignment | Out-of-scope blocker |
|---|---|---|
| 0221_maximal_square | RF-3 | python_stdlib_parity: min() arity |
| 0402_remove_k_digits | AU-3 | operator_and_truthiness: int truthiness |
| 0496_next_greater_element_i | AU-3 | python_stdlib_parity: Iterator iteration |
| 0621_task_scheduler | RF-1 | python_stdlib_parity: Counter undefined |
| 0735_asteroid_collision | AU-3 | operator_and_truthiness: int truthiness |
| 0909_snakes_and_ladders | DS-2 | operator_and_truthiness: int truthiness |
| 1481_least_number_of_unique_integers | RF-1 | python_stdlib_parity: Counter undefined |
| 1572_matrix_diagonal_sum | RF-3 | missing annotations + Any arithmetic |
| 2101_detonate_the_maximum_bombs | AU-2 | python_stdlib_parity: sqrt undefined |

Expected net pass gain from focus-4 closure: ~65-70 fixtures (not 90), because ~9 will
migrate to out-of-scope categories and ~15-17 need multi-workstream convergence before
passing.
```

### EDIT-2: Revise acceptance criteria to use two-tier metric

**File**: `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06.md`

**Location**: Replace the "Acceptance criteria" block in each workstream section (lines 160-162, 178-180, 197-198, 215-217, 233-234).

**Template for each workstream's acceptance criteria**:

```markdown
Acceptance criteria:

- Tier 1 (primary): all owned-root-cause fixtures have their primary diagnostic resolved
  (the diagnostic that determined their category assignment no longer appears)
- Tier 2 (full pass): fixtures with no cross-workstream or out-of-scope secondary
  blockers pass completely
- Delta report must distinguish Tier 1 exits from Tier 2 full passes
```

### EDIT-3: Define DS-1/DS-2 compiler-vs-adaptation boundary

**File**: `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06.md`

**Location**: Workstream D section (after line 209), expand the compiler/adaptation boundary.

**Content to add after "compiler lane: `DS-3`" / "mixed/adaptation lanes: `DS-1`, `DS-2`, `DS-4`, `DS-5`"**:

```markdown
DS-1/DS-2 resolution boundary:

- Compiler: if Sifr is intended to accept `for (a, b) in list_of_lists` where elements are
  homogeneous 2-element lists, the type checker must infer tuple-like destructuring from
  list element types. Define whether this is a type inference improvement or a language
  surface expansion and document the decision.
- Adaptation: if the language policy is that only `tuple` types support positional
  destructuring, then all DS-1/DS-2 fixtures must be rewritten to use `tuple` literals or
  explicit indexing.
- Decision required before implementation begins. The choice affects 15 fixtures.
```

### EDIT-4: Add expected-outcome column to CSV or add a supplementary tracking artifact

**File**: `verification/leetcode/phase_apr06_focus4_root_cause_map.csv`

**Change**: Add an `expected_outcome_after_focus4` column with values:
- `pass` -- fixture expected to pass after all focus-4 workstreams complete
- `migrate:<category>` -- fixture expected to migrate to a different failure category
- `multi_workstream` -- fixture needs multiple workstreams but expected to pass after all converge

This can alternatively be a supplementary file (`phase_apr06_focus4_expected_outcomes.csv`) if modifying the primary CSV is undesirable.

### EDIT-5: Add multi-workstream tracking to execution ledger

**File**: `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06-execution.md`

**Location**: New section before "Validation and Reporting" (after line 42).

**Content to add**:

```markdown
## Multi-Workstream Convergence Tracking

Fixtures that require fixes from two or more workstreams before they can pass.
Mark only after ALL required workstreams have merged and the fixture is confirmed green.

- [ ] `0162_find_peak_element` (B.RF-3 + B.RF-2)
- [ ] `0323_number_of_connected_components` (D.DS-1 + C.CF-1)
- [ ] `0355_design_twitter` (A.AU-3 + C.CF-1)
- [ ] `0622_design_circular_queue` (E.DS-5 + C.CF-1)
- [ ] `0706_design_hashmap` (C.CF-1 + B.RF-2)
- [ ] `0745_prefix_and_suffix_search` (C.CF-1 + B.RF-2)
- [ ] `0895_maximum_frequency_stack` (D.DS-3 + C.CF-1 + B.RF-2)
- [ ] `0981_time_based_key_value_store` (C.CF-1 + B.RF-2)
- [ ] `1396_design_underground_system` (D.DS-3 + C.CF-1 + B.RF-2)
- [ ] `1489_find_critical_and_pseudo_critical_edges` (A.AU-4 + C.CF-1)
- [ ] `1603_design_parking_system` (C.CF-1 + B.RF-2)
- [ ] `2013_detect_squares` (D.DS-3 + C.CF-1)
- [ ] `2709_greatest_common_divisor_traversal` (B.RF-3 + C.CF-1)

## Fixtures Expected to Remain Failing (out-of-scope blockers)

These fixtures will not pass after focus-4 closure due to diagnostics in
categories outside focus-4 scope. They should be excluded from pass-rate
calculations for this phase.

- `0221_maximal_square` -> python_stdlib_parity (min arity)
- `0402_remove_k_digits` -> operator_and_truthiness (int truthiness)
- `0496_next_greater_element_i` -> python_stdlib_parity (Iterator)
- `0621_task_scheduler` -> python_stdlib_parity (Counter)
- `0735_asteroid_collision` -> operator_and_truthiness (int truthiness)
- `0909_snakes_and_ladders` -> operator_and_truthiness (int truthiness)
- `1481_least_number_of_unique_integers` -> python_stdlib_parity (Counter)
- `1572_matrix_diagonal_sum` -> missing annotations + Any arithmetic
- `2101_detonate_the_maximum_bombs` -> python_stdlib_parity (sqrt)
```

---

## 6. Summary Verdict

| Dimension | Status |
|---|---|
| Files present and readable | PASS |
| Category counts consistent across artifacts | PASS |
| Sub-root-cause counts consistent | PASS |
| Resolution-mode totals consistent | PASS |
| Fixture-level bijective mapping (taxonomy <-> CSV <-> diagnostics) | PASS |
| Single-root-cause mapping accuracy | PASS (primary assignment is correct) |
| Multi-root-cause coverage | **FAIL -- not tracked** |
| Acceptance criteria precision | **FAIL -- conflates exit with pass** |
| DS-1/DS-2 compiler/adaptation boundary | **FAIL -- undefined** |
| Validation gate design for cross-workstream fixtures | **FAIL -- not addressed** |
| Out-of-scope blocker tracking | **FAIL -- not tracked** |

**Overall implementation readiness: NOT READY**

The data quality is high -- counts are consistent, mappings are bijective, root-cause assignments are defensible. The **blocking gaps** are all in the planning/tracking layer:

1. No cross-workstream dependency tracking (EDIT-1, EDIT-5)
2. Ambiguous acceptance criteria (EDIT-2)
3. Undefined DS-1/DS-2 boundary (EDIT-3)

After applying EDITs 1-5, the package will be implementation-ready. The underlying analysis is sound; only the tracking and specification precision need tightening.
