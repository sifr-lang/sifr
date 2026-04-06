# Ad-hoc Phase: Focus4 Root-Cause Closure (2026-04-06)

## Progress Snapshot (2026-04-06)

- Compiler lanes closed in this batch:
  - `CF-1-class_field_registration_gap`
  - `CF-2-nested_attribute_assignment_gap`
  - `DS-3-augassign_subscript_lowering_gap`
  - `RF-2-loop_local_scope_resolution_bug`
  - `AU-1-any_element_type_erasure`
  - `AU-2-unknown_flow_leak`
  - `AU-3-optional_any_bridge_leak`
  - `AU-4-container_shape_specialization_leak`
  - `RF-3-return_completeness_false_positive`
- Adaptation lane closed in this batch:
  - `RF-1-duplicate_solution_definitions` (fixture canonicalization)
  - `DS-4-unpack_target_shape_restriction`
  - `DS-5-chained_assignment_restriction`
  - `DS-1-list_pair_destructure_requires_tuple`
  - `DS-2-list_unpack_requires_tuple`
- Adaptation closure sweep in this batch:
  - canonicalized residual AU/RF fixture patterns (list-shaped destructuring, dynamic container initialization, and missing fallback-return tails) to remain within strict Sifr policy while preserving algorithmic intent
- Focus4 subset rerun now reports `CHECK_ERROR=74, NO_ORACLE=5, PASS=4, RUN_ERROR=7`.
- All focus4 primary root-cause presences are now `0/x` across `AU-*`, `DS-*`, `RF-*`, and `CF-*`.
- Remaining failures are secondary blockers: multi-workstream convergence and out-of-scope parity categories.
- Full-corpus reporting closure completed with rerun3 + taxonomy + delta artifacts:
  - `verification/leetcode/full_corpus_current_results_20260406_live_rerun3.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3_delta_vs_rerun2.md`
- Multi-workstream convergence closure completed in rerun4:
  - all 12 tracked convergence fixtures are now non-failing (`NO_ORACLE`)
  - `verification/leetcode/full_corpus_current_results_20260406_live_rerun4.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun4.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun4_delta_vs_rerun3.md`
- See `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06-execution.md` for wave artifacts and validation logs.

## Scope

Target buckets from `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun1.json`:

- `any_unknown_typing_and_container_specialization_gap`: `26`
- `destructuring_and_assignment_target_surface_gap`: `24`
- `return_path_and_function_contract_gap`: `24`
- `class_field_state_and_object_layout`: `16`

Total in-scope failing fixtures: `90`

Primary artifacts backing this analysis:

- `verification/leetcode/full_corpus_current_results_20260406_live_rerun1.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun1.json`
- `verification/leetcode/phase_apr06_focus4_full_diagnostics.json`
- `verification/leetcode/phase_apr06_focus4_root_cause_map.csv`

## Breakdown and Root Causes

### 1) any_unknown_typing_and_container_specialization_gap (`26`)

Sub-root causes:

- `AU-1-any_element_type_erasure` (`12`) -> `compiler`
- `AU-2-unknown_flow_leak` (`4`) -> `compiler`
- `AU-3-optional_any_bridge_leak` (`6`) -> `compiler`
- `AU-4-container_shape_specialization_leak` (`4`) -> `compiler`

Root-cause summary:

- Container element types are being widened or dropped to `Any`/`Unknown` in key algorithmic flows (stack/queue/list/dict/set).
- Optional bridges (`Any | None`, `Unknown | None`) are not being stabilized before comparison/index/iteration.
- Downstream callable contracts fail because earlier inference loses concrete container signatures.

Decision:

- **Compiler fix**.
- No broad language policy change required.
- Adaptation should be exceptional-only if a fixture intentionally depends on dynamic typing.

Representative failures:

- `0056_merge_intervals`, `0210_course_schedule_ii`, `0332_reconstruct_itinerary`, `2092_find_all_people_with_secret`

### 2) destructuring_and_assignment_target_surface_gap (`24`)

Sub-root causes:

- `DS-1-list_pair_destructure_requires_tuple` (`8`) -> `both`
- `DS-2-list_unpack_requires_tuple` (`7`) -> `both`
- `DS-3-augassign_subscript_lowering_gap` (`5`) -> `compiler`
- `DS-4-unpack_target_shape_restriction` (`3`) -> `adaptation`
- `DS-5-chained_assignment_restriction` (`1`) -> `adaptation`

Root-cause summary:

- A large portion is sequence-shape mismatch (`list[int]`/`list[str]`) under tuple destructuring assumptions.
- `augmented subscript assignment target must be a simple name` indicates a concrete lowering/checking gap for common forms like `arr[i] += v` / `map[k] += 1`.
- Some syntax forms are intentional Sifr restrictions and should remain adaptation-only (tuple-target shape policy, chained assignment policy).

Decision:

- **Both**.
- Compiler: close `SubscriptAugAssign` and destructuring lowering gaps that are already within intended Sifr surface.
- Adaptation: canonicalize fixtures that rely on intentionally unsupported unpack/chained-assignment forms.

Representative failures:

- `0286_walls_and_gates`, `0516_longest_palindromic_subsequence`, `0622_design_circular_queue`, `2013_detect_squares`

### 3) return_path_and_function_contract_gap (`24`)

Sub-root causes:

- `RF-1-duplicate_solution_definitions` (`7`) -> `adaptation`
- `RF-2-loop_local_scope_resolution_bug` (`6`) -> `compiler`
- `RF-3-return_completeness_false_positive` (`11`) -> `compiler`

Root-cause summary:

- Duplicate top-level function definitions are fixture-shape artifacts (multi-solution files), not compiler defects.
- `undefined variable` diagnostics include clear compiler defects in local scope resolution (for-loop/while local visibility and nested-flow propagation).
- `must return on all control-flow paths` appears frequently on functions that already include a final return, indicating control-flow completeness false positives.

Decision:

- **Both**, with compiler-heavy closure first.
- Keep duplicate-definition handling as adaptation lane unless explicit multi-definition semantics are intentionally added.

Representative failures:

- Compiler lanes: `0018_4sum`, `0134_gas_station`, `1011_capacity_to_ship_packages_within_d_days`, `0153_find_minimum_in_rotated_sorted_array`, `0221_maximal_square`
- Adaptation lane: `0049_group_anagrams`, `0231_power_of_two`, `0338_counting_bits`

### 4) class_field_state_and_object_layout (`16`)

Sub-root causes:

- `CF-1-class_field_registration_gap` (`14`) -> `compiler`
- `CF-2-nested_attribute_assignment_gap` (`2`) -> `compiler`

Root-cause summary:

- Core object-layout issue: class fields assigned in `__init__` are not consistently materialized into class state surface.
- Nested attribute assignment path (`obj.prev.next = ...`) is blocked by target-shape checks even in canonical data-structure implementations.

Decision:

- **Compiler fix**.
- Adaptation only for unrelated fixture issues (for example missing parameter annotations in helper classes), not for field-layout semantics.

Representative failures:

- `0155_min_stack`, `0208_implement_trie_prefix_tree`, `0706_design_hashmap`, `0721_accounts_merge`, `0981_time_based_key_value_store`

## Cross-cutting Findings

1. The focus-4 set is primarily compiler work.

- Resolution-mode totals from `phase_apr06_focus4_root_cause_map.csv`:
  - `compiler`: `64`
  - `both`: `15`
  - `adaptation`: `11`

2. Top systemic compiler root causes:

- type erasure into `Any`/`Unknown`
- class field registration/object layout gaps
- control-flow completeness and loop-local scope propagation
- `SubscriptAugAssign` lowering/checking limitations

3. Adaptation should be constrained to policy-consistent canonicalization:

- duplicate top-level solution defs
- list-based destructuring where tuple semantics are required by current language rules
- chained assignment restrictions

## Cross-Workstream Dependency Matrix

Fixtures whose full resolution requires fixes from multiple workstreams or that have
out-of-scope blocking diagnostics. These fixtures will not pass after their primary
workstream alone completes.

### Fixtures requiring Workstream C (CF-1) in addition to primary assignment

| Fixture | Primary assignment | Secondary blocker |
|---|---|---|
| 0323_number_of_connected_components_in_an_undirected_graph | DS-1 (Workstream D) | CF-1 field: `f` |
| 0355_design_twitter | AU-3 (Workstream A) | CF-1 fields: `followMap`, `tweetMap` |
| 0622_design_circular_queue | DS-5 (Workstream E) | CF-1 fields: `capacity`, `head`, `size`, `tail` |
| 0895_maximum_frequency_stack | DS-3 (Workstream D) | CF-1 fields: `cnt`, `stacks` |
| 1396_design_underground_system | DS-3 (Workstream D) | CF-1 fields: `customer`, `time` |
| 1489_find_critical_and_pseudo_critical_edges | AU-4 (Workstream A) | CF-1 fields: `par`, `rank` |
| 2013_detect_squares | DS-3 (Workstream D) | CF-1 field: `pts` |
| 2709_greatest_common_divisor_traversal | RF-3 (Workstream B) | CF-1 fields: `count`, `par`, `size` |

### Fixtures requiring Workstream B (RF-2) in addition to primary assignment

| Fixture | Primary assignment | Secondary blocker |
|---|---|---|
| 0706_design_hashmap | CF-1 (Workstream C) | RF-2: undefined variable `cur` |
| 0745_prefix_and_suffix_search | CF-1 (Workstream C) | RF-2: undefined variable `cur` |
| 0895_maximum_frequency_stack | DS-3 (Workstream D) | RF-2: undefined variables `res`, `valCnt` |
| 0981_time_based_key_value_store | CF-1 (Workstream C) | RF-2: undefined variables `l`, `res`, `values` |
| 1396_design_underground_system | DS-3 (Workstream D) | RF-2: undefined variables `route`, `start`, `total` |
| 1603_design_parking_system | CF-1 (Workstream C) | RF-2: undefined variable `new_total` |

### Fixtures blocked by out-of-scope categories (will not pass after focus-4 closure)

| Fixture | Primary assignment | Out-of-scope blocker |
|---|---|---|
| 0221_maximal_square | RF-3 | `python_stdlib_parity`: `min()` arity |
| 0402_remove_k_digits | AU-3 | `operator_and_truthiness`: int truthiness |
| 0496_next_greater_element_i | AU-3 | `python_stdlib_parity`: `Iterator` iteration |
| 0621_task_scheduler | RF-1 | `python_stdlib_parity`: `Counter` undefined |
| 0673_number_of_longest_increasing_subsequence | DS-2 | `nonlocal_mutable_capture`: tuple-unpack `nonlocal` rebind unsupported |
| 0735_asteroid_collision | AU-3 | `operator_and_truthiness`: int truthiness |
| 0909_snakes_and_ladders | DS-2 | `operator_and_truthiness`: int truthiness |
| 1481_least_number_of_unique_integers_after_k_removals | RF-1 | `python_stdlib_parity`: `Counter` undefined |
| 1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero | DS-1 | `nonlocal_mutable_capture`: recursive `nonlocal` state mutation unsupported |
| 1572_matrix_diagonal_sum | RF-3 | missing annotations + `Any` arithmetic |

Expected net pass gain from focus-4 closure: around `60`-`65` fixtures (not `90`),
because about `10` will migrate to out-of-scope categories and around `12` need
multi-workstream convergence before passing.

## Ready-to-Implement Phase Plan

Phase ID: `ad_hoc_focus4_root_cause_closure`

### Workstream A: Any/Unknown stabilization and container specialization

Owner: compiler

Targets:

- `AU-1`, `AU-2`, `AU-3`, `AU-4`

Implementation goals:

- preserve concrete element types through container construction/update flows
- prevent `Any`/`Unknown` widening at joins where concrete evidence exists
- stabilize `Optional[Any]/Optional[Unknown]` bridges before operator/index/call use

Acceptance criteria:

- Tier 1 (primary): all `AU-*` fixtures have their primary AU diagnostic resolved
  (the diagnostic that determined assignment no longer appears)
- Tier 2 (full pass): AU fixtures without cross-workstream or out-of-scope blockers
  pass completely
- Delta report distinguishes Tier 1 exits from Tier 2 full passes
- no regressions in existing optional and container narrowing tests

### Workstream B: Return-path and scope-resolution closure

Owner: compiler

Targets:

- `RF-2`, `RF-3`

Implementation goals:

- fix loop-local variable visibility and symbol propagation in nested control flow
- remove false positives from return-path completeness analysis
- close intra-workstream RF interplay on `0162_find_peak_element` (`RF-3` missing-return
  false positive + `RF-2` undefined local) within Workstream B

Acceptance criteria:

- Tier 1 (primary): `RF-2`/`RF-3` fixtures have their primary RF diagnostics resolved
- Tier 2 (full pass): fixtures without cross-workstream or out-of-scope blockers pass
  completely
- Delta report distinguishes Tier 1 exits from Tier 2 full passes
- diagnostics remain strict for genuinely missing-return programs

### Workstream C: Class field registration and nested-attribute assignment

Owner: compiler

Targets:

- `CF-1`, `CF-2`

Implementation goals:

- materialize class fields from constructor/state-init flows consistently
- support intended nested attribute assignment surface used in canonical data-structures

Acceptance criteria:

- Tier 1 (primary): `CF-*` fixtures have class-field primary diagnostics resolved
- Tier 2 (full pass): fixtures without cross-workstream or out-of-scope blockers pass
  completely
- Delta report distinguishes Tier 1 exits from Tier 2 full passes
- all `CF-*` fixtures leave this bucket without broadening dynamic object semantics

### Workstream D: Destructuring and subscript-augassign closure

Owner: compiler + adaptation

Targets:

- compiler lane: `DS-3`
- mixed/adaptation lanes: `DS-1`, `DS-2`, `DS-4`, `DS-5`

DS-1/DS-2 resolution boundary:

- Architecture decision: Sifr keeps tuple-only positional destructuring as a core rule.
  `list[T]` does not imply fixed arity, so list-element destructuring is not added as
  a language-surface feature.
- Compiler responsibility: preserve and propagate tuple element types when the source is
  already tuple-typed (prevent false DS diagnostics caused by lost tuple shape).
- Adaptation responsibility: rewrite list-based destructuring fixtures to tuple literals
  or explicit index-based extraction where the source is list-shaped by design.
- Decision is locked for this phase to avoid policy drift and keep type guarantees strict.

Implementation goals:

- compiler: close `augmented subscript assignment` lowering gap
- adaptation: canonicalize list-destructure and chained-assignment fixtures into explicit tuple/index forms where policy requires

Acceptance criteria:

- Tier 1 (primary): `DS-3` primary diagnostics are resolved in compiler lane; DS policy
  fixtures are transformed per canonicalization rules
- Tier 2 (full pass): fixtures without cross-workstream or out-of-scope blockers pass
  completely
- Delta report distinguishes Tier 1 exits from Tier 2 full passes
- policy-restricted forms are handled via approved fixture canonicalization only

### Workstream E: Fixture canonicalization lane (strictly bounded)

Owner: adaptation

Targets:

- `RF-1` duplicates
- policy-restricted destructuring/chained-assignment cases

Implementation goals:

- one canonical solution per fixture module
- preserve algorithmic intent while staying inside Sifr policy

Acceptance criteria:

- Tier 1 (primary): all adaptation-owned fixtures have primary adaptation blockers
  resolved (duplicate definitions/policy-restricted forms removed)
- Tier 2 (full pass): fixtures without cross-workstream or out-of-scope blockers pass
  completely
- Delta report distinguishes Tier 1 exits from Tier 2 full passes
- adaptation lane is auditable and does not mask compiler defects from streams A-D

## Execution Order

1. Workstream A (Any/Unknown) and Workstream B (scope/return) in parallel.
2. Workstream C (class/object layout) immediately after B reaches green on pilot fixtures.
3. Workstream D compiler slice (`DS-3`) before broad adaptation sweep.
4. Workstream E canonicalization last, after compiler closure rerun.

## Validation Gate

Mandatory after each workstream:

1. targeted fixture reruns for owned sub-root causes
2. full corpus rerun against `audits/leetcode`
3. taxonomy regeneration
4. delta report against prior category counts

Recommended gate commands:

- `cargo build --release -p sifr`
- `target/release/sifr check audits/leetcode/<fixture>.sifr` (targeted)
- full runner script used in prior scans to regenerate `full_corpus_current_results_<date>_live_*.json`
- taxonomy rebuild script used to regenerate `full_corpus_failure_taxonomy_<date>_live_*.json`

## Deliverables

- this phase spec: `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06.md`
- execution ledger: `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06-execution.md`
- fixture-level diagnostic map: `verification/leetcode/phase_apr06_focus4_full_diagnostics.json`
- fixture-level root-cause map: `verification/leetcode/phase_apr06_focus4_root_cause_map.csv`
- expected outcomes tracker: `verification/leetcode/phase_apr06_focus4_expected_outcomes.csv`
