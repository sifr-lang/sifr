# Review: Focus-4 Root-Cause Closure Package (Pass 3)

Reviewer: agent
Date: 2026-04-06

## Prior Review History

- **Pass 1b**: Verdict **Mostly Ready**. Key findings: cross-bucket contamination (F1), DS-1/DS-2 boundary undefined (F2), RF-3 too thin (F3), AU-1 lumps mechanisms (F4), CF-1/RF-2 overlap (F5), cascade inflation (F6), misclassifications (F7), out-of-scope residuals (F8).
- **Pass 2**: Verdict **NOT READY**. Blocking gaps: no cross-workstream dependency matrix (GAP-1), acceptance criteria conflate exit with pass (GAP-2), DS-1/DS-2 boundary undefined (GAP-3), no multi-workstream tracking in ledger (GAP-4), no expected-outcome annotations (GAP-5). Prescribed 5 edits.

---

## 1. Prior Blocking Edits: Closure Status

### Pass 2 GAP-1 / EDIT-1: Cross-workstream dependency matrix

**STATUS: FULLY ADDRESSED**

Phase spec now contains a "Cross-Workstream Dependency Matrix" section (lines 142-188) with three tables:
- 8 fixtures requiring Workstream C (CF-1) in addition to primary assignment
- 7 fixtures requiring Workstream B (RF-2) in addition to primary assignment
- 9 fixtures blocked by out-of-scope categories

All 8 CF-1 secondary fixtures verified against diagnostics JSON. All 7 RF-2 secondary fixtures verified. All 9 out-of-scope fixtures verified with specific blocker category and diagnostic cited.

### Pass 2 GAP-2 / EDIT-2: Two-tier acceptance criteria

**STATUS: FULLY ADDRESSED**

All 5 workstreams now carry tiered acceptance criteria:
- Tier 1 (primary): diagnostic that determined assignment no longer appears
- Tier 2 (full pass): fixtures without cross-workstream or out-of-scope blockers pass completely
- Delta report must distinguish Tier 1 exits from Tier 2 full passes
- Workstream-specific guards preserved (e.g., Workstream B: "diagnostics remain strict for genuinely missing-return programs")

### Pass 2 GAP-3 / EDIT-3: DS-1/DS-2 compiler-vs-adaptation boundary

**STATUS: FULLY ADDRESSED**

Workstream D now includes a locked architecture decision (lines 270-278):
- Sifr keeps tuple-only positional destructuring as a core rule
- Compiler responsibility: preserve tuple element types when source is already tuple-typed
- Adaptation responsibility: rewrite list-based destructuring to tuple literals or explicit indexing
- Decision is locked for this phase to avoid policy drift

This resolves the prior ambiguity completely. The 15 `both`-mode fixtures now have a clear resolution path.

### Pass 2 GAP-4 / EDIT-5: Multi-workstream tracking in execution ledger

**STATUS: FULLY ADDRESSED**

Execution ledger now includes:
- "Multi-Workstream Convergence Tracking" section (lines 44-60): 13 fixtures with explicit workstream dependencies
- "Fixtures Expected to Remain Failing" section (lines 62-76): 9 out-of-scope fixtures
- "Validation and Reporting" section with explicit gate checklist

### Pass 2 GAP-5 / EDIT-4: Expected-outcome annotations per fixture

**STATUS: FULLY ADDRESSED**

New artifact `phase_apr06_focus4_expected_outcomes.csv` created with columns:
- `category`, `fixture_slug`, `root_cause_id`, `resolution_mode`
- `expected_outcome_after_focus4`: values `pass` / `migrate:<category>` / `multi_workstream`
- `expected_blocker_category`, `notes`

Row count: 91 (header + 90 data rows). Covers all 90 in-scope fixtures. Breakdown:
- 68 `pass`
- 9 `migrate:*` (4 python_stdlib_parity, 3 operator_and_truthiness, 1 mixed_out_of_scope)
- 13 `multi_workstream`

Sum: 68 + 9 + 13 = 90. Consistent with phase spec estimate of "around 65-70" net pass gain.

---

## 2. Pass 1b Findings: Closure Status

| Pass 1b Finding | Severity | Status |
|---|---|---|
| F1: Cross-bucket contamination | CRITICAL | CLOSED via dependency matrix and expected outcomes |
| F2: DS-1/DS-2 split unspecified | SIGNIFICANT | CLOSED via locked policy decision |
| F3: RF-3 sub-pattern catalog | SIGNIFICANT | OPEN (not added) |
| F4: AU-1 sub-mechanism breakdown | MODERATE | OPEN (not added) |
| F5: CF-1 / RF-2 overlap | MODERATE | CLOSED via dependency matrix RF-2 table |
| F6: Cascade inflation awareness | MODERATE | PARTIALLY CLOSED (matrix implies it; no explicit validation-gate note) |
| F7: 3 misclassifications | MINOR | OPEN (0162, 0118, 1572 unchanged in CSV) |
| F8: Out-of-scope residuals | MINOR | CLOSED via out-of-scope tables and expected outcomes |

Items F3, F4, and F7 remain open but were rated non-blocking by both prior reviews. F6 is substantively addressed by the dependency matrix even though no explicit cascade note exists in the validation gate section.

---

## 3. New Findings (Pass 3)

### N1 - SIGNIFICANT: 6 fixtures marked "pass" have out-of-scope blocking diagnostics

The expected outcomes CSV marks 68 fixtures as `pass`. Cross-referencing against `phase_apr06_focus4_full_diagnostics.json`, at least 6 of these carry diagnostics from categories outside focus-4 scope that will persist after all workstreams complete:

| Fixture | Expected | Out-of-scope diagnostic | Blocker category |
|---|---|---|---|
| `0056_merge_intervals` | pass | `sort() got an unexpected keyword argument 'key'` | python_stdlib_parity |
| `0253_meeting_rooms_ii` | pass | `sort() got an unexpected keyword argument 'key'` | python_stdlib_parity |
| `0239_sliding_window_maximum` | pass | `cannot index type 'deque' with 'int'` | python_stdlib_parity |
| `0862_shortest_subarray_with_sum_at_least_k` | pass | `cannot index type 'deque' with 'int'` | python_stdlib_parity |
| `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero` | pass | `recursive nested function 'dfs' cannot mutate captured state with 'nonlocal' yet` | nonlocal_mutable_capture |
| `0673_number_of_longest_increasing_subsequence` | pass | `tuple unpacking cannot rebind captured state with 'nonlocal' yet` | nonlocal_mutable_capture |

**Evidence**:
- `sort() key`: Sifr's `sort()` builtin does not accept `key=` keyword argument. No focus-4 workstream addresses builtin keyword-argument parity. Confirmed by diagnostics in 0056 and 0253.
- `deque` indexing: `deque` type does not support `[]` subscript access. This is a type surface gap in the deque implementation, not a type-erasure issue (AU-1 won't resolve it). Confirmed in 0239 and 0862.
- `nonlocal` capture: `nonlocal_mutable_capture_not_supported` is its own taxonomy category, not in focus-4 scope. Confirmed in 1466 and 0673.

Note: 1466 and 0673 were flagged in Pass 1b (F8) as having out-of-scope blockers, but were not captured when the expected outcomes CSV was created. 0056, 0253, 0239, and 0862 are newly identified in this pass.

**Impact**: Expected pass count drops from 68 to 62. Expected migrate count rises from 9 to 15. Phase spec's "around 65-70" estimate should be revised to "around 60-65".

**Required fix**: In `phase_apr06_focus4_expected_outcomes.csv`, reclassify:
- 0056, 0253, 0239, 0862: `pass` -> `migrate:python_stdlib_parity`
- 1466, 0673: `pass` -> `migrate:nonlocal_mutable_capture`

In the phase spec's out-of-scope list (lines 173-185), add these 6 fixtures. Update the net-gain estimate accordingly.

In the execution ledger's "Fixtures Expected to Remain Failing" section, add these 6 entries.

### N2 - MINOR: 0162_find_peak_element is intra-workstream, not cross-workstream

The execution ledger lists `0162_find_peak_element` in the Multi-Workstream Convergence Tracking section as `(B.RF-3 + B.RF-2)`. Both RF-3 and RF-2 are targets within **Workstream B**. This is an intra-workstream dependency, not a cross-workstream one. The convergence section header says "Fixtures that require fixes from two or more workstreams before they can pass" -- 0162 requires only Workstream B.

**Impact**: Negligible. The fixture is conservatively tracked rather than missed. Multi-workstream count would drop from 13 to 12, and pass count would increase by 1 (to 63 after N1 correction).

**Recommended fix**: Move 0162 out of multi-workstream tracking. Reclassify to `pass` in expected outcomes (since both RF-3 and RF-2 are Workstream B targets). Add an inline note in Workstream B that 0162 has intra-workstream dependency between RF-3 and RF-2.

### N3 - MINOR: Phase spec estimates "15-17" multi-workstream but actual count is 13

The prose at line 187 says "around 15-17 need multi-workstream convergence before passing" but the dependency matrix enumerates exactly 13 fixtures (or 12 after N2 correction). The estimate should be tightened to match the actual enumeration.

---

## 4. Internal Consistency Verification

### 4a. Count reconciliation across all artifacts

| Metric | Phase spec | Taxonomy JSON | Root cause CSV | Expected outcomes CSV | Match |
|---|---|---|---|---|---|
| Total in-scope fixtures | 90 | 90 | 90 | 90 | YES |
| AU-* | 26 | 26 | 26 | 26 | YES |
| DS-* | 24 | 24 | 24 | 24 | YES |
| RF-* | 24 | 24 | 24 | 24 | YES |
| CF-* | 16 | 16 | 16 | 16 | YES |
| compiler mode | 64 | -- | 64 | 64 | YES |
| both mode | 15 | -- | 15 | 15 | YES |
| adaptation mode | 11 | -- | 11 | 11 | YES |

### 4b. Fixture-level bijection

- Every fixture in taxonomy JSON (focus-4 categories) has a row in root cause CSV: **confirmed**
- Every fixture in root cause CSV has an entry in diagnostics JSON: **confirmed**
- Every fixture in root cause CSV has a row in expected outcomes CSV: **confirmed**
- Every fixture in expected outcomes CSV has a corresponding taxonomy entry: **confirmed**
- No orphans in any direction: **confirmed**

### 4c. Cross-workstream dependency matrix vs execution ledger alignment

The 13 fixtures in the execution ledger's multi-workstream section match the union of the spec's two dependency tables (CF-1 secondary: 8, RF-2 secondary: 7, with overlaps producing 13 unique). **Aligned.**

### 4d. Out-of-scope fixture alignment (spec vs ledger vs expected outcomes)

All three artifacts list the same 9 out-of-scope fixtures with consistent blocker categories. **Aligned** (though incomplete per finding N1).

### 4e. Diagnostic cross-reference (spot check)

| Fixture | Root cause CSV first_diagnostic | Taxonomy JSON first_diagnostic | Match |
|---|---|---|---|
| 0056_merge_intervals | `cannot index type 'Any' with 'int'` | `cannot index type 'Any' with 'int'` | YES |
| 0155_min_stack | `type 'MinStack' has no field 'minStack'` | `type 'MinStack' has no field 'minStack'` | YES |
| 0018_4sum | `undefined variable: 's'` | `undefined variable: 's'` | YES |
| 0516_longest_palindromic_subsequence | `augmented subscript assignment target must be a simple name` | `augmented subscript assignment target must be a simple name` | YES |
| 2709_greatest_common_divisor_traversal | `function 'canTraverseAllPairs' must return...` | `function 'canTraverseAllPairs' must return...` | YES |

All spot checks pass. **Consistent.**

---

## 5. Verdict

### What is resolved

All 5 blocking gaps from Pass 2 have been fully addressed:
1. Cross-workstream dependency matrix: present and accurate
2. Two-tier acceptance criteria: present in all 5 workstreams
3. DS-1/DS-2 boundary: locked architecture decision with clear compiler/adaptation split
4. Multi-workstream tracking in execution ledger: 13 fixtures tracked
5. Expected outcomes CSV: created with full coverage

The data quality is high. Counts are consistent across all artifacts. The workstream structure, acceptance criteria, execution order, and validation gate design are sound.

### What remains

| Finding | Severity | Blocking? |
|---|---|---|
| N1: 6 fixtures misclassified as "pass" (have out-of-scope blockers) | SIGNIFICANT | No -- mechanical CSV fix, does not affect workstream plans |
| N2: 0162 is intra-workstream, not cross-workstream | MINOR | No |
| N3: "15-17" estimate vs 13 actual multi-workstream | MINOR | No |
| F3 (Pass 1b): RF-3 sub-pattern catalog not added | MODERATE (carried) | No |
| F4 (Pass 1b): AU-1 sub-mechanism breakdown not added | MODERATE (carried) | No |

### Final Verdict: **READY**

The package is implementation-ready. The prior blocking gaps (dependency tracking, acceptance criteria precision, DS-1/DS-2 boundary) are closed. The remaining issues are data-accuracy corrections in tracking artifacts, not planning or design gaps.

**Condition**: Before using `phase_apr06_focus4_expected_outcomes.csv` as a validation-gate baseline, apply the N1 fix: reclassify 6 fixtures from `pass` to `migrate:*` and update the out-of-scope lists in the phase spec and execution ledger accordingly. This is a 15-minute mechanical edit that can happen in parallel with Workstream A/B kickoff.
