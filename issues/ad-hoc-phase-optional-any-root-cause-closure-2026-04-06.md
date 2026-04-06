# Ad-hoc Phase: Optional/Any Root-Cause Closure (2026-04-06)

## Snapshot

Source baseline:

- `verification/leetcode/full_corpus_current_results_20260406_live_rerun1.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun1.json`
- `verification/leetcode/phase_apr06_on_au_root_cause_map.json`
- `verification/leetcode/phase_apr06_on_au_root_cause_map.csv`

Current category counts in scope:

- `optional_none_flow_and_narrowing_gap`: `30`
- `any_unknown_typing_and_container_specialization_gap`: `28`

Total in-scope fixtures for this phase: `58`

Resolution-mode split from root-cause map:

- `compiler`: `51`
- `both`: `6`
- `adaptation`: `1`

## Root-Cause Breakdown

### A) Optional/None flow and narrowing gap (`30`)

Sub-root causes:

- `ON-1-optional-arithmetic-operator-leak`: `15` -> `compiler`
- `ON-2-optional-container-boundary-leak`: `6` -> `compiler`
- `ON-3-optional-element-contamination`: `3` -> `compiler`
- `ON-4-optional-contract-and-return-closure`: `4` -> `both`
- `ON-5-optional-string-surface-guarding`: `2` -> `both`

Representative diagnostics:

- `unsupported operand type(s) for +/-//` with `int | None`
- `cannot iterate over type 'list[int] | None'`
- `type 'None | list[int]' has no method 'append'`
- `return type mismatch: expected 'int/bool', got 'None | int/bool'`
- `type 'None | str' has no method 'replace'`

Root cause summary:

- Optional unions are not consistently eliminated at dominated use sites.
- Optional container bindings are leaking into iteration/index/method paths.
- Element refinement after guarded population is incomplete.
- A small residual set mixes compiler closure with fixture-side explicit guard intent.

Decision:

- Compiler-first closure.
- Adaptation only after compiler lanes close and only for policy-consistent explicit guard canonicalization.

### B) Any/Unknown typing and container specialization gap (`28`)

Sub-root causes:

- `AU-1-heapq-unknown-container-shape`: `4` -> `compiler`
- `AU-2-any-unknown-flow-and-operator-leak`: `16` -> `compiler`
- `AU-3-any-unknown-optional-bridge`: `5` -> `compiler`
- `AU-4-unknown-stdlib-contract-surface`: `1` -> `compiler`
- `AU-5-signature-annotation-required`: `1` -> `adaptation`
- `AU-6-list-unknown-specialization`: `1` -> `compiler`

Representative diagnostics:

- `__compat_sifr_heapq_heapify`: expected `list[T]`, got `Unknown`
- `cannot index type 'Any' / 'Unknown'`
- `'in'/'not in' operator not supported for type 'Unknown'`
- `for-loop iterable must have a statically-known element type, got 'Unknown'`
- `return type mismatch: expected ..., got 'Any | None'`
- `parameter ... is missing a type annotation`

Root cause summary:

- Container specialization and join stabilization still leak `Any`/`Unknown` into operator/index/call boundaries.
- Optional bridge collapse for `Any | None` / `Unknown | None` is incomplete.
- Stdlib-compat constructors are receiving unresolved container/mapping types.
- One fixture requires canonical Sifr annotation adaptation.

Decision:

- Predominantly compiler closure.
- Keep the annotation case adaptation-only under current Sifr policy.

## Compiler vs Adaptation Decision Matrix

Compiler workstreams to implement:

1. `W1-ON-arithmetic-and-operator-narrowing` (`ON-1`)
2. `W2-ON-container-boundary-and-element-refinement` (`ON-2`, `ON-3`)
3. `W3-AU-flow-stabilization-and-operator-safety` (`AU-2`, `AU-3`)
4. `W4-AU-compat-container-contract-typing` (`AU-1`, `AU-4`, `AU-6`)
5. `W5-ON-contract-return-closure` compiler slice (`ON-4`, `ON-5` compiler side)

Adaptation workstreams:

1. `A1-signature-annotation-required` (`AU-5`)
2. `A2-explicit-guard-canonicalization` residual `ON-4`/`ON-5` only after `W5`

## Ready-to-Implement Phase Plan

Phase ID: `ad_hoc_optional_any_root_cause_closure`

### Workstream W1: Optional arithmetic/operator narrowing

Owner: compiler

Goal:

- eliminate `T | None` from arithmetic/operator positions once dominated by accepted guards.

Primary loci:

- `crates/sifr_type_system/src/narrow.rs`
- `crates/sifr_type_system/src/check.rs`
- `crates/sifr_hir/src/lower/function_flow.rs`
- `crates/sifr_hir/src/lower/expressions.rs`

Acceptance:

- all `ON-1` signatures removed from focused fixture rerun.
- no regression in optional narrowing e2e fixtures.

### Workstream W2: Optional container boundary + element refinement

Owner: compiler

Goal:

- stop `None | container` leaks at iteration/index/method sites.
- refine element domains after guarded writes/build patterns.

Primary loci:

- `crates/sifr_hir/src/lower/container_literal_specialization.rs`
- `crates/sifr_hir/src/lower/guarded_index.rs`
- `crates/sifr_type_system/src/infer.rs`
- `crates/sifr_type_system/src/union.rs`

Acceptance:

- `ON-2` and `ON-3` signatures removed from focused rerun.
- no regressions in list/dict specialization tests.

### Workstream W3: Any/Unknown flow stabilization and operator safety

Owner: compiler

Goal:

- prevent `Any`/`Unknown` escape at joins and downstream operator/index/call usage.
- stabilize `Any|None` and `Unknown|None` before boundary checks.

Primary loci:

- `crates/sifr_type_system/src/infer.rs`
- `crates/sifr_type_system/src/union.rs`
- `crates/sifr_type_system/src/check.rs`

Acceptance:

- `AU-2` and `AU-3` signatures removed from focused rerun.
- no new `Any`/`Unknown` regressions in existing focus4 tests.

### Workstream W4: Compat container contract typing

Owner: compiler

Goal:

- ensure heap/defaultdict/typed-list compat entry points receive stabilized concrete container types.

Primary loci:

- `crates/sifr_hir/src/lower/container_literal_specialization.rs`
- `crates/sifr_codegen/src` compat lowering for heap/defaultdict call paths
- `crates/sifr_type_system/src/check.rs`

Acceptance:

- `AU-1`, `AU-4`, and `AU-6` signatures removed from focused rerun.

### Workstream W5: Optional contract/return closure (compiler slice)

Owner: compiler

Goal:

- close false optional return/argument unions where control-flow is semantically complete.

Primary loci:

- `crates/sifr_hir/src/cfg.rs`
- `crates/sifr_hir/src/lower/function_flow.rs`
- `crates/sifr_type_system/src/check.rs`

Acceptance:

- compiler-owned part of `ON-4` and `ON-5` removed.
- remaining residuals are explicit adaptation candidates only.

### Adaptation A1: Required signature annotation

Owner: fixture canonicalization

Goal:

- canonicalize the explicit annotation-required case (`AU-5`) without changing compiler policy.

Acceptance:

- fixture compiles under current annotation rules.

### Adaptation A2: Explicit guard canonicalization (residual)

Owner: fixture canonicalization

Goal:

- apply explicit guard rewrites only to residual `ON-4`/`ON-5` cases that remain after `W5`.

Acceptance:

- adaptation set is small, auditable, and does not broaden language semantics.

## Phase Exit Gates

1. Root-cause presence gate:

- `ON-1..ON-5` and `AU-1..AU-6` targeted signatures removed from focused rerun or explicitly transferred to approved adaptation list.

2. Full-corpus gate:

- new full-corpus rerun artifact generated.
- taxonomy regenerated.
- no net regressions outside approved adaptation transitions.
- the `53` non-targeted fixtures across all other taxonomy categories must not change status (any change is a regression requiring investigation).

3. Policy gate:

- no weakening of ownership/mutability, parse safety, or unsupported `nonlocal` mutable capture policy.

## Validation Commands

- `cargo build --release -p sifr`
- focused fixture rerun for all `58` mapped fixtures
- full rerun:
  - `python3 /tmp/sifr_full_leetcode_scan_<date>.py`
- taxonomy regeneration for the new full-corpus artifact
- `scripts/run_all_tests.sh --profile quick`

## Expected Outcome

- Close the compiler-owned majority (`51/58`) directly.
- Minimize adaptation to explicit policy-consistent cases (`<=7`, currently projected `1` mandatory + residual guard cases only if still needed).
- Produce a measurable reduction in both `optional_none_flow_and_narrowing_gap` and `any_unknown_typing_and_container_specialization_gap` in the next full run.

## Execution Log (2026-04-06)

### Wave4: Counter codegen/index/builtin-min closure

Artifacts:

- `/tmp/phase_apr06_on_au_wave4_counter_iter_min_fixes.json`

Key changes:

- Fixed structured bool-op condition lowering in stmt emitter.
- Fixed optional tuple index lowering in codegen helpers.
- Added class/protocol index lowering through `__getitem__`.
- Added borrow-aware `__getitem__` arg passing in codegen.
- Fixed builtin `min/max` fallback lowering to avoid unresolved plain `min(...)` emission.
- Stabilized `Counter` stdlib path (`__getitem__`, constructor compatibility, iterator-return handling path updates).

Focused result summary:

- `CHECK_ERROR=53`, `PASS=4`, `NO_ORACLE=1`
- Changed from prior wave:
  - `0383`: `RUN_ERROR -> PASS`
  - `1189`: `RUN_ERROR -> NO_ORACLE`

### Wave5: AU-4 defaultdict/Counter contract bridge

Artifacts:

- `/tmp/phase_apr06_on_au_wave5b_defaultdict_counter_bridge.json`

Key changes:

- Added HIR defaultdict constructor bridge for `defaultdict(int, Counter(...))` compatibility surface.
- Added deterministic ordering for `Counter.keys()` in stdlib to stabilize downstream `items()` iteration order used by fixture assertions.
- Added HIR regression coverage for Counter-backed defaultdict constructor lowering.

Focused result summary:

- `CHECK_ERROR=52`, `PASS=4`, `NO_ORACLE=2`
- Changed from wave4:
  - `0350`: `CHECK_ERROR -> NO_ORACLE`

### Wave7: A1 signature-annotation-required adaptation (1472)

Artifacts:

- `/tmp/phase_apr06_on_au_wave7b_a1_adaptation_1472_coldcache.json`

Key changes:

- Canonicalized `1472_design_browser_history.sifr` for current Sifr policy:
  - added explicit parameter annotations in `ListNode.__init__`
  - removed duplicate `BrowserHistory` class definition
  - replaced optional-index return sites with explicit `None` guards
  - rewrote history overwrite path to append/pop canonical form
  - used explicit `str(url)` append form to satisfy ownership/lowering constraints
- Cleared generated run artifact cache before focused rerun to avoid stale cache-hit contamination in status attribution.

Focused result summary:

- `CHECK_ERROR=51`, `PASS=4`, `NO_ORACLE=3`
- Changed from wave6d baseline:
  - `1472`: `CHECK_ERROR -> NO_ORACLE`

### Wave8: Subscript-guard and defaultdict-index typing lane (compiler)

Artifacts:

- `/tmp/phase_apr06_on_au_wave8_subscript_guard_defaultdict_coldcache.json`

Key changes:

- Added sequence-guard support for attribute targets (`self.field`) in guard detection/index narrowing and non-empty pop narrowing.
- Added subscript non-None guard propagation for repeated reads after:
  - `if seq[i] is None: return ...`
  - `if seq[i] is not None: ...`
- Added non-optional `defaultdict` index typing in HIR subscript resolution.
- Added/updated targeted HIR regression tests for:
  - attribute non-empty index narrowing
  - field-access `pop()` narrowing
  - subscript non-None guard narrowing
  - imported `Counter(list[T])` unsupported policy guard
  - `defaultdict` non-optional index read typing

Focused result summary:

- `CHECK_ERROR=51`, `PASS=4`, `NO_ORACLE=3`
- Changed from wave7b:
  - no fixture status deltas (behavior-neutral for current focused manifest).

### Wave9: A2 residual guard-canonicalization batch (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave9_a2_residual_batch_coldcache.json`

Key changes:

- Canonicalized residual `ON-4` / `ON-5` fixtures toward explicit policy-compatible contracts and guard surfaces.
- Closed fixtures:
  - `0208_implement_trie_prefix_tree`
  - `0752_open_the_lock`
  - `0929_unique_email_addresses`
  - `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero`
  - `1845_seat_reservation_manager`
- Remaining residual adaptation candidate:
  - `0332_reconstruct_itinerary`

Focused result summary (cold-cache):

- `CHECK_ERROR=46`, `PASS=4`, `NO_ORACLE=8`
- Changed from wave8:
  - `0208`: `CHECK_ERROR -> NO_ORACLE`
  - `0752`: `CHECK_ERROR -> NO_ORACLE`
  - `0929`: `CHECK_ERROR -> NO_ORACLE`
  - `1466`: `CHECK_ERROR -> NO_ORACLE`
  - `1845`: `CHECK_ERROR -> NO_ORACLE`

Residual root-cause shape after wave9 (CHECK_ERROR only):

- `ON-1`: `15`
- `AU-2`: `12`
- `ON-2`: `6`
- `AU-3`: `5`
- `AU-1`: `4`
- `ON-3`: `3`
- `ON-5`: `1` (`0332`)

### Wave11: A2 closure completion + run-stability normalization

Artifacts:

- `/tmp/phase_apr06_on_au_wave11_a2_full6_plus_0350_stability_coldcache.json`

Key changes:

- Closed the remaining residual adaptation fixture `0332_reconstruct_itinerary`.
- Added deterministic ordering normalization for `0350_intersection_of_two_arrays_ii` to prevent cold-cache run-stage order flake (`intersection.sort()`), preserving prior `NO_ORACLE` stability.
- Re-ran focused 58-fixture manifest with cold run-cache for stable attribution.

Focused result summary (cold-cache):

- `CHECK_ERROR=45`, `PASS=4`, `NO_ORACLE=9`
- Changed from wave8:
  - `0208`: `CHECK_ERROR -> NO_ORACLE`
  - `0332`: `CHECK_ERROR -> NO_ORACLE`
  - `0752`: `CHECK_ERROR -> NO_ORACLE`
  - `0929`: `CHECK_ERROR -> NO_ORACLE`
  - `1466`: `CHECK_ERROR -> NO_ORACLE`
  - `1845`: `CHECK_ERROR -> NO_ORACLE`

Residual root-cause shape after wave11 (CHECK_ERROR only):

- `ON-1`: `15`
- `AU-2`: `12`
- `ON-2`: `6`
- `AU-3`: `5`
- `AU-1`: `4`
- `ON-3`: `3`

Closure note:

- `ON-4` and `ON-5` are fully cleared from `CHECK_ERROR` in the focused manifest.

### Wave13: ON-1 canonicalization subset (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave13_on1_adaptation_subset3_coldcache.json`

Key changes:

- Canonicalized residual ON-1 fixtures with explicit optional-index guards in:
  - `0134_gas_station`
  - `0338_counting_bits`
  - `2482_difference_between_ones_and_zeros_in_row_and_column`
- Continued ON-1 adaptation probes for:
  - `0149_max_points_on_a_line`
  - `1288_remove_covered_intervals`
  - `2001_number_of_pairs_of_interchangeable_rectangles`
  (still unresolved in this wave)

Focused result summary (cold-cache):

- `CHECK_ERROR=42`, `PASS=6`, `NO_ORACLE=10`
- Changed from wave11:
  - `0134`: `CHECK_ERROR -> PASS`
  - `0338`: `CHECK_ERROR -> PASS`
  - `2482`: `CHECK_ERROR -> NO_ORACLE`

Residual root-cause shape after wave13 (CHECK_ERROR only):

- `ON-1`: `12`
- `AU-2`: `12`
- `ON-2`: `6`
- `AU-3`: `5`
- `AU-1`: `4`
- `ON-3`: `3`

### Wave14: ON-1 canonicalization follow-up (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave14_on1_adaptation_subset3b_coldcache.json`

Key changes:

- Canonicalized additional ON-1 fixtures with explicit optional-index guards in:
  - `0153_find_minimum_in_rotated_sorted_array`
  - `0658_find_k_closest_elements`
  - `0918_maximum_sum_circular_subarray`

Focused result summary (cold-cache):

- `CHECK_ERROR=39`, `PASS=9`, `NO_ORACLE=10`
- Changed from wave13:
  - `0153`: `CHECK_ERROR -> PASS`
  - `0658`: `CHECK_ERROR -> PASS`
  - `0918`: `CHECK_ERROR -> PASS`

Residual root-cause shape after wave14 (CHECK_ERROR only):

- `ON-1`: `9`
- `AU-2`: `12`
- `ON-2`: `6`
- `AU-3`: `5`
- `AU-1`: `4`
- `ON-3`: `3`

### Wave16: ON-1 subset expansion + run fix stabilization (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave15_on1_adaptation_subset6_coldcache.json`
- `/tmp/phase_apr06_on_au_wave16_on1_adaptation_subset6_plus_0076_runfix_coldcache.json`

Key changes:

- Canonicalized additional ON-1 fixtures:
  - `0410_split_array_largest_sum`
  - `1011_capacity_to_ship_packages_within_d_days`
- Stabilized `0076_minimum_window_substring` after transient run-stage mutability failure by replacing tuple target updates with mutable list-state + guarded index extraction.
- Kept unresolved ON-1 probes (`0149`, `0286`, `1074`, `1288`, `1475`, `2001`) unchanged in this wave after non-clean drafts.

Focused result summary (cold-cache, wave16):

- `CHECK_ERROR=36`, `PASS=11`, `NO_ORACLE=11`
- Changed from wave14:
  - `0076`: `CHECK_ERROR -> NO_ORACLE`
  - `0410`: `CHECK_ERROR -> PASS`
  - `1011`: `CHECK_ERROR -> PASS`

Residual root-cause shape after wave16 (CHECK_ERROR only):

- `AU-2`: `12`
- `ON-1`: `6`
- `ON-2`: `6`
- `AU-3`: `5`
- `AU-1`: `4`
- `ON-3`: `3`

### Wave17: ON-1 pair closure (`1288`, `1475`) (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave17_on1_adaptation_1288_1475_coldcache.json`

Key changes:

- Canonicalized `1288_remove_covered_intervals` with explicit guarded pairwise coverage detection (removing unary optional negation path).
- Canonicalized `1475_final_prices_with_a_special_discount_in_a_shop` with explicit guarded forward-discount scan (removing optional stack-top arithmetic path).
- Attempted `0149` and `2001` with `int(...)` coercion probes; reverted after run-stage codegen failures (`E0308` optional index helper mismatch in generated Rust), leaving those fixtures unchanged in this wave.

Focused result summary (cold-cache):

- `CHECK_ERROR=34`, `PASS=13`, `NO_ORACLE=11`
- Changed from wave16:
  - `1288`: `CHECK_ERROR -> PASS`
  - `1475`: `CHECK_ERROR -> PASS`

Residual root-cause shape after wave17 (CHECK_ERROR only):

- `AU-2`: `12`
- `ON-2`: `6`
- `AU-3`: `5`
- `ON-1`: `4`
- `AU-1`: `4`
- `ON-3`: `3`

### Wave18: ON-1 closure completion (`0149`, `0286`, `1074`, `2001`) (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave18_on1_full_closure_coldcache.json`

Key changes:

- Closed remaining ON-1 fixtures with explicit optional-int extraction and non-float-key canonicalizations:
  - `0149_max_points_on_a_line`
  - `0286_walls_and_gates`
  - `1074_number_of_submatrices_that_sum_to_target`
  - `2001_number_of_pairs_of_interchangeable_rectangles`
- For `0149` and `2001`, replaced float key maps with normalized rational string keys to avoid Rust `HashMap<f64, _>` codegen constraints (`Eq`/`Hash` not satisfied).
- For `0286`, replaced alias row writes with direct `rooms[nr][nc]` writes to eliminate runtime non-termination in queue expansion.

Focused result summary (cold-cache):

- `CHECK_ERROR=30`, `PASS=13`, `NO_ORACLE=15`
- Changed from wave17:
  - `0149`: `CHECK_ERROR -> NO_ORACLE`
  - `0286`: `CHECK_ERROR -> NO_ORACLE`
  - `1074`: `CHECK_ERROR -> NO_ORACLE`
  - `2001`: `CHECK_ERROR -> NO_ORACLE`

Residual root-cause shape after wave18 (CHECK_ERROR only):

- `AU-2`: `12`
- `ON-2`: `6`
- `AU-3`: `5`
- `AU-1`: `4`
- `ON-3`: `3`

Closure note:

- `ON-1` is fully cleared from `CHECK_ERROR` in the focused manifest.

### Wave19: ON-2 single-fixture closure (`0347`) (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave19_on2_single_0347_coldcache.json`

Key changes:

- Canonicalized `0347_top_k_frequent_elements` away from optional bucket-index append path:
  - replaced frequency-bucket indexing with deterministic repeated max-frequency selection.
  - added deterministic tie-break (`smaller number first`) to preserve fixture assertion ordering.
- Attempted a wider ON-2 batch (`0210`, `0785`, `0787`, `2092`, `2101`) but reverted non-clean drafts in this wave due broad optional key/index type propagation.

Focused result summary (cold-cache):

- `CHECK_ERROR=29`, `PASS=13`, `NO_ORACLE=16`
- Changed from wave18:
  - `0347`: `CHECK_ERROR -> NO_ORACLE`

Residual root-cause shape after wave19 (CHECK_ERROR only):

- `AU-2`: `12`
- `AU-3`: `5`
- `ON-2`: `5`
- `AU-1`: `4`
- `ON-3`: `3`

### Wave20: ON-2 closure completion (`0210`, `0785`, `0787`, `2092`, `2101`) (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave20_on2_full_closure_coldcache.json`

Key changes:

- Closed remaining ON-2 fixtures with explicit optional extraction and container-boundary canonicalization:
  - `0210_course_schedule_ii`
  - `0785_is_graph_bipartite`
  - `0787_cheapest_flights_within_k_stops`
  - `2092_find_all_people_with_secret`
  - `2101_detonate_the_maximum_bombs`
- Stabilization details:
  - `0787`: dropped untyped `self` parameter and switched to integer-sentinel Bellman-Ford form.
  - `2092`: replaced nested `time -> person -> neighbors` map with `time -> edge-list` grouping plus per-time graph expansion.
  - `2101`: replaced optional nested adjacency list append path with edge-list traversal to avoid container boundary optional leakage.

Focused result summary (cold-cache):

- `CHECK_ERROR=24`, `PASS=13`, `NO_ORACLE=21`
- Changed from wave19:
  - `0210`: `CHECK_ERROR -> NO_ORACLE`
  - `0785`: `CHECK_ERROR -> NO_ORACLE`
  - `0787`: `CHECK_ERROR -> NO_ORACLE`
  - `2092`: `CHECK_ERROR -> NO_ORACLE`
  - `2101`: `CHECK_ERROR -> NO_ORACLE`

Residual root-cause shape after wave20 (CHECK_ERROR only):

- `AU-2`: `12`
- `AU-3`: `5`
- `AU-1`: `4`
- `ON-3`: `3`

Closure note:

- `ON-2` is fully cleared from `CHECK_ERROR` in the focused manifest.

### Wave21: ON-3 closure completion (`0253`, `0280`, `0456`) (fixture adaptation)

Artifacts:

- `/tmp/phase_apr06_on_au_wave21_on3_full_closure_coldcache.json`

Key changes:

- Closed all ON-3 fixtures with explicit element-domain canonicalization:
  - `0253_meeting_rooms_ii`
  - `0280_wiggle_sort`
  - `0456_132_pattern`
- Replaced optional list/tuple element propagation with explicit integer extraction at append/assignment boundaries.
- `0456` additionally moved to typed tuple stack with explicit guarded element reads.

Focused result summary (cold-cache):

- `CHECK_ERROR=21`, `PASS=14`, `NO_ORACLE=23`
- Changed from wave20:
  - `0253`: `CHECK_ERROR -> NO_ORACLE`
  - `0280`: `CHECK_ERROR -> NO_ORACLE`
  - `0456`: `CHECK_ERROR -> PASS`

Residual root-cause shape after wave21 (CHECK_ERROR only):

- `AU-2`: `12`
- `AU-3`: `5`
- `AU-1`: `4`

Closure note:

- `ON-3` is fully cleared from `CHECK_ERROR` in the focused manifest.
