# Review: Optional/Any Root-Cause Closure Phase (Pass 3 — Final Audit)

Reviewer: agent
Date: 2026-04-06
Phase document: `issues/ad-hoc-phase-optional-any-root-cause-closure-2026-04-06.md`
Execution ledger: `issues/ad-hoc-phase-optional-any-root-cause-closure-2026-04-06-execution.md`
Prior reviews: `reviews/optional-any-root-cause-phase-review-pass1b.md`, `reviews/optional-any-root-cause-phase-review-pass2.md`

---

## Verdict: READY

No blockers remain. All pass1b edits are applied. All pass2 confirmations hold. The full 58-fixture audit below confirms every category assignment, root-cause ID, and resolution mode is correct and justified. The phase plan is implementation-ready with no required edits.

---

## Scope of This Review

This pass performs a line-by-line audit of all 58 mapped fixtures, validates every field against the source taxonomy, confirms all prior review edits are still applied, and evaluates remaining risks. It is the final gate before implementation begins.

---

## Section 1: Full 58-Fixture Audit

### AU-1-heapq-unknown-container-shape (4 fixtures)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 1 | `0621_task_scheduler` | `expected 'list[T]', got 'Unknown'` (heapq_heapify) | AU | compiler | Correct |
| 2 | `0767_reorganize_string` | `expected 'list[T]', got 'Unknown'` (heapq_heapify) | AU | compiler | Correct |
| 3 | `1481_least_number_of_unique_integers_after_k_removals` | `expected 'list[T]', got 'Unknown'` (heapq_heapify) | AU | compiler | Correct |
| 4 | `1985_find_the_kth_largest_integer_in_the_array` | `expected 'list[T]', got 'Unknown'` (heapq_heapify) | AU | compiler | Correct |

Assessment: All four share the identical diagnostic pattern. The container shape reaching the heapq compat boundary is Unknown rather than a concrete `list[T]`. Classification as AU (not ON) is correct because the problematic type is Unknown, not `T | None`. Compiler ownership is correct because the type system must stabilize the container type before it reaches the compat entry point. Workstream W4 is the correct target.

### AU-2-any-unknown-flow-and-operator-leak (16 fixtures)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 5 | `0079_word_search` | `'>' not supported: 'Any' and 'Any'` | AU | compiler | Correct |
| 6 | `0084_largest_rectangle_in_histogram` | `'>' not supported: 'Unknown' and 'int'` | AU | compiler | Correct |
| 7 | `0118_pascals_triangle` | `cannot index type 'Any' with 'int'` | AU | compiler | Correct |
| 8 | `0225_implement_stack_using_queues` | `cannot index type 'Any' with 'int'` | AU | compiler | Correct |
| 9 | `0269_alien_dictionary` | `cannot index type 'Unknown' with 'int'` | AU | compiler | Correct |
| 10 | `0383_ransom_note` | `'<' not supported: 'Any' and 'Any'` | AU | compiler | Correct |
| 11 | `0496_next_greater_element_i` | `'in' operator not supported for type 'Unknown'` | AU | compiler | Correct |
| 12 | `0739_daily_temperatures` | `'>' not supported: 'int' and 'Unknown'` | AU | compiler | Correct |
| 13 | `0901_online_stock_span` | `'<=' not supported: 'Any' and 'int'` | AU | compiler | Correct |
| 14 | `0909_snakes_and_ladders` | `cannot index type 'Any' with 'int \| None'` | AU | compiler | Correct (dual-cause noted) |
| 15 | `0953_verifying_an_alien_dictionary` | `'<' not supported: 'Any' and 'Any'` | AU | compiler | Correct |
| 16 | `1049_last_stone_weight_ii` | `subscript assignment not supported for 'Unknown'` | AU | compiler | Correct |
| 17 | `1189_maximum_number_of_balloons` | `for-loop iterable element type 'Unknown'` | AU | compiler | Correct |
| 18 | `1462_course_schedule_iv` | `'in' operator not supported for type 'Unknown \| None'` | AU | compiler | Correct (boundary note below) |
| 19 | `1572_matrix_diagonal_sum` | `len() got 'Any'` | AU | compiler | Correct |
| 20 | `2306_naming_a_company` | `'not in' operator not supported for type 'Unknown'` | AU | compiler | Correct |

Assessment: All 16 share the pattern of Any/Unknown escaping type stabilization and reaching operator, index, iteration, or membership sites. Classification as AU-2 is correct across all. Two fixtures have boundary notes:

- **`0909` (dual cause)**: Diagnostic involves `Any` container indexed with `int | None`. Primary root cause is AU-2 (the container is Any). The secondary ON root cause (index is `int | None`) is properly documented in the rationale per pass1b Edit 2. If W3 resolves the Any to a concrete type, an ON failure may surface. The focused rerun will catch this.
- **`1462` (AU-2/AU-3 boundary)**: Diagnostic type is `Unknown | None`, which has AU-3 characteristics (optional bridge). Classification as AU-2 is defensible because the proximate failure is the operator site, and both AU-2 and AU-3 route to W3. **New observation**: like `0909`, if W3 resolves Unknown to a concrete type, the result would be `concrete_type | None`, producing an ON-2-style failure. This cascading risk is analogous to `0909` but was not explicitly annotated. Impact is low (the rerun catches it), but noted here for completeness.

### AU-3-any-unknown-optional-bridge (5 fixtures)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 21 | `0155_min_stack` | `min(): got 'int' and 'Any \| None'` | AU | compiler | Correct |
| 22 | `0232_implement_queue_using_stacks` | `return mismatch: expected 'int', got 'Any \| None'` | AU | compiler | Correct |
| 23 | `0303_range_sum_query_immutable` | `'-': 'Any \| None' and 'Any \| None'` | AU | compiler | Correct |
| 24 | `0535_encode_and_decode_tinyurl` | `return mismatch: expected 'str', got 'Any \| None'` | AU | compiler | Correct |
| 25 | `1642_furthest_building_you_can_reach` | `unary -: 'None \| Unknown'` | AU | compiler | Correct |

Assessment: All five exhibit the bridge pattern where `Any | None` or `Unknown | None` reaches a return, operator, or contract site without the Any/Unknown arm being resolved first. Classification as AU (not ON) is correct because the primary unknown is the non-None arm. If the Any/Unknown resolves to a concrete type via W3, the `| None` would either be eliminated (if narrowing closes it) or cascade to an ON root cause. This is the designed behavior of the phase plan.

### AU-4-unknown-stdlib-contract-surface (1 fixture)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 26 | `0350_intersection_of_two_arrays_ii` | `defaultdict() initial mapping must be dict, got 'Unknown'` | AU | compiler | Correct |

Assessment: Unknown flows into a strict stdlib constructor contract. Classification as AU-4 (stdlib contract surface) is precise. W4 is the correct workstream. Depends on W3 having stabilized the Unknown type first.

### AU-5-signature-annotation-required (1 fixture)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 27 | `1472_design_browser_history` | `parameter 'next' in ListNode.__init__ is missing a type annotation` | AU | adaptation | Correct |

Assessment: This is a fixture conformance issue, not a compiler gap. The fixture needs an explicit type annotation to satisfy current Sifr policy. Adaptation is the only correct resolution mode. A1 is the correct workstream. Independent of all compiler workstreams.

### AU-6-list-unknown-specialization (1 fixture)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 28 | `0001_two_sum` | `return mismatch: expected 'list[int]', got 'list[Unknown]'` | AU | compiler | Correct |

Assessment: List element type remains Unknown at the function boundary instead of being specialized to `int`. Classification as AU-6 (distinct from AU-2 which covers operator/index sites) is correct. W4 is the correct workstream.

### ON-1-optional-arithmetic-operator-leak (15 fixtures)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 29 | `0076_minimum_window_substring` | `+: 'int \| None' and 'int'` | ON | compiler | Correct |
| 30 | `0134_gas_station` | `-: 'int' and 'int \| None'` | ON | compiler | Correct |
| 31 | `0149_max_points_on_a_line` | `-: 'int \| None' and 'int \| None'` | ON | compiler | Correct |
| 32 | `0153_find_minimum_in_rotated_sorted_array` | `min(): 'float' and 'int \| None'` | ON | compiler | Correct |
| 33 | `0286_walls_and_gates` | `+: 'int \| None' and 'int'` | ON | compiler | Correct |
| 34 | `0338_counting_bits` | `+: 'int' and 'int \| None'` | ON | compiler | Correct |
| 35 | `0410_split_array_largest_sum` | `-: 'int' and 'int \| None'` | ON | compiler | Correct |
| 36 | `0658_find_k_closest_elements` | `-: 'int' and 'int \| None'` | ON | compiler | Correct |
| 37 | `0918_maximum_sum_circular_subarray` | `max(): 'int' and 'int \| None'` | ON | compiler | Correct |
| 38 | `1011_capacity_to_ship_packages_within_d_days` | `+: 'int \| None' and 'int'` | ON | compiler | Correct |
| 39 | `1074_number_of_submatrices_that_sum_to_target` | `+: 'int \| None' and 'int \| None'` | ON | compiler | Correct |
| 40 | `1288_remove_covered_intervals` | `unary -: 'int \| None'` | ON | compiler | Correct |
| 41 | `1475_final_prices_with_a_special_discount_in_a_shop` | `-: 'int' and 'int \| None'` | ON | compiler | Correct |
| 42 | `2001_number_of_pairs_of_interchangeable_rectangles` | `/: 'int \| None' and 'int \| None'` | ON | compiler | Correct |
| 43 | `2482_difference_between_ones_and_zeros_in_row_and_column` | `+: 'int \| None' and 'int \| None'` | ON | compiler | Correct |

Assessment: All 15 exhibit `int | None` (or `float` and `int | None`) reaching arithmetic operators (+, -, /, unary -, min(), max()) without the None arm being narrowed away. The non-None arm is always a concrete type (int or float), confirming ON classification. All are compiler-closeable via W1 dominator-based narrowing. No borderline cases.

### ON-2-optional-container-boundary-leak (6 fixtures)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 44 | `0210_course_schedule_ii` | `cannot iterate over 'list[int] \| None'` | ON | compiler | Correct |
| 45 | `0347_top_k_frequent_elements` | `'None \| list[int]' has no method 'append'` | ON | compiler | Correct |
| 46 | `0785_is_graph_bipartite` | `cannot iterate over 'list[int] \| None'` | ON | compiler | Correct |
| 47 | `0787_cheapest_flights_within_k_stops` | `cannot index 'list[float]' with 'int \| None'` | ON | compiler | Correct (ON-1 variant noted) |
| 48 | `2092_find_all_people_with_secret` | `'not in' not supported for 'dict[...] \| None'` | ON | compiler | Correct |
| 49 | `2101_detonate_the_maximum_bombs` | `cannot iterate over 'list[int] \| None'` | ON | compiler | Correct |

Assessment: Five of six have optional containers (`list[T] | None` or `dict[...] | None`) reaching iteration, method call, or membership sites. The sixth (`0787`) has a concrete container `list[float]` but an optional index `int | None`. The `0787` rationale was updated per pass1b Edit 1 to note this is an ON-1 variant at the index position. Classification under ON-2 is acceptable because the failure site is a container boundary (indexing), and both ON-1 and ON-2 share overlapping narrowing infrastructure. W2 is the primary workstream but W1 may also resolve `0787`.

### ON-3-optional-element-contamination (3 fixtures)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 50 | `0253_meeting_rooms_ii` | `append 'tuple[int \| None, int]' to list of 'tuple[int, int]'` | ON | compiler | Correct |
| 51 | `0280_wiggle_sort` | `subscript assignment 'int \| None' to list element 'int'` | ON | compiler | Correct |
| 52 | `0456_132_pattern` | `element mismatch: expected 'int', got 'int \| None'` | ON | compiler | Correct |

Assessment: All three exhibit optional element types contaminating container value domains after guarded writes or build patterns. The optional type infects a container whose declared/inferred element type is concrete. Classification as ON-3 (distinct from ON-1 arithmetic and ON-2 container boundary) is precise. W2 is the correct workstream.

### ON-4-optional-contract-and-return-closure (4 fixtures)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 53 | `0208_implement_trie_prefix_tree` | `return mismatch: expected 'bool', got 'None \| bool'` | ON | both | Correct |
| 54 | `0752_open_the_lock` | `argument: expected 'str', got 'str \| None'` | ON | both | Correct |
| 55 | `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero` | `argument: expected 'int', got 'int \| None'` | ON | both | Correct |
| 56 | `1845_seat_reservation_manager` | `return mismatch: expected 'int', got 'None \| int'` | ON | both | Correct |

Assessment: These are return or argument contract mismatches where optional unions aren't closed by control-flow analysis. The `both` resolution mode is correct: the compiler (W5) should attempt CFG-based closure first, and any residuals move to adaptation (A2). The non-None arm is always a concrete type, confirming ON classification.

### ON-5-optional-string-surface-guarding (2 fixtures)

| # | Fixture | Diagnostic | Category | Mode | Verdict |
|---|---------|-----------|----------|------|---------|
| 57 | `0332_reconstruct_itinerary` | `'not in': element 'str \| None' vs collection 'str'` | ON | both | Correct |
| 58 | `0929_unique_email_addresses` | `'None \| str' has no method 'replace'` | ON | both | Correct |

Assessment: Optional string types reaching membership or method call sites. The `both` mode is correct for the same reason as ON-4: compiler tries narrowing first, residuals go to adaptation. W5 is the compiler workstream, A2 handles residuals.

### Fixture Audit Summary

| Sub-root-cause | Expected Count | Audited Count | All Correct |
|---|---|---|---|
| AU-1 | 4 | 4 | Yes |
| AU-2 | 16 | 16 | Yes |
| AU-3 | 5 | 5 | Yes |
| AU-4 | 1 | 1 | Yes |
| AU-5 | 1 | 1 | Yes |
| AU-6 | 1 | 1 | Yes |
| ON-1 | 15 | 15 | Yes |
| ON-2 | 6 | 6 | Yes |
| ON-3 | 3 | 3 | Yes |
| ON-4 | 4 | 4 | Yes |
| ON-5 | 2 | 2 | Yes |
| **Total** | **58** | **58** | **Yes** |

No misclassified fixtures. No incorrect resolution modes. All rationale text is accurate and reflects pass1b edits.

---

## Section 2: Cross-Document Consistency

### 2.1 Root-cause map JSON internal consistency

| Check | Result |
|---|---|
| `fixture_count` field = 58 | Pass |
| `category_counts` AU(28) + ON(30) = 58 | Pass |
| `resolution_mode_counts` compiler(51) + both(6) + adaptation(1) = 58 | Pass |
| `root_cause_counts` sum: 4+16+5+1+1+1+15+6+3+4+2 = 58 | Pass |
| Actual `rows` array length = 58 | Pass |
| Row-level root_cause_id values match `root_cause_counts` keys | Pass |
| Row-level category values match `category_counts` keys | Pass |
| Row-level resolution_mode values match `resolution_mode_counts` keys | Pass |
| No duplicate `fixture_slug` in rows | Pass |

### 2.2 CSV vs JSON consistency

| Check | Result |
|---|---|
| CSV data rows (excl. header) = 58 | Pass |
| Every CSV row has matching JSON row (fixture_slug, category, root_cause_id, resolution_mode, first_diagnostic) | Pass |
| Rationale text for `0787` matches between CSV and JSON (pass1b Edit 1) | Pass |
| Rationale text for `0909` matches between CSV and JSON (pass1b Edit 2) | Pass |

### 2.3 Root-cause map vs full taxonomy

| Check | Result |
|---|---|
| All 28 AU fixtures appear in taxonomy under `any_unknown_typing_and_container_specialization_gap` | Pass |
| All 30 ON fixtures appear in taxonomy under `optional_none_flow_and_narrowing_gap` | Pass |
| `first_diagnostic` matches for all 58 fixtures between map and taxonomy | Pass |
| Taxonomy AU count (28) matches map AU count (28) | Pass |
| Taxonomy ON count (30) matches map ON count (30) | Pass |
| No fixture in map is absent from taxonomy | Pass |
| No AU/ON fixture in taxonomy is absent from map | Pass |

### 2.4 Phase document vs root-cause map

| Check | Result |
|---|---|
| Phase doc ON count (30) matches map | Pass |
| Phase doc AU count (28) matches map | Pass |
| Phase doc total (58) matches map | Pass |
| Phase doc resolution split (51/6/1) matches map | Pass |
| All 11 sub-root-cause IDs in phase doc match map | Pass |
| All 11 sub-root-cause counts in phase doc match map | Pass |
| Workstream-to-root-cause mapping is complete (every root cause assigned to a workstream) | Pass |
| Full-corpus gate includes 53 non-targeted fixture clause (pass1b Edit 4) | Pass |

### 2.5 Execution ledger vs phase document

| Check | Result |
|---|---|
| Workstream checklist covers W1-W5, A1-A2 (all 7 from phase doc) | Pass |
| Suggested execution order present (pass1b Edit 3) | Pass |
| Tier ordering respects dependency chain | Pass |
| Review log reflects pass1b and pass2 completion | Pass |

### 2.6 Non-targeted fixture count verification

Taxonomy total failures: 111. Targeted (ON + AU): 58. Non-targeted: 53.

| Non-targeted category | Count |
|---|---|
| callable_argument_contract_mismatch | 1 |
| codegen_runtime_build_gap | 6 |
| destructuring_and_assignment_target_surface_gap | 1 |
| nonlocal_mutable_capture_not_supported | 2 |
| operator_and_truthiness_typing_gap | 11 |
| other_type_surface_and_api_mismatch | 12 |
| ownership_and_mutability_boundary | 4 |
| python_stdlib_and_builtin_parity_gap | 10 |
| recursive_node_and_field_expression_surface | 2 |
| return_path_and_function_contract_gap | 2 |
| signature_invalid_fixture_surface | 2 |
| **Total** | **53** |

Confirmed: 53 non-targeted fixtures across 11 categories. These must remain stable per the full-corpus exit gate.

---

## Section 3: Prior Review Edit Verification

### Pass1b edits (all 4 confirmed still applied)

| Edit | Target | Status | Evidence |
|---|---|---|---|
| Edit 1: `0787` rationale update | JSON + CSV | Applied | Rationale reads: "Optional index value (int \| None) not narrowed before container indexing; note: container itself is concrete, issue is in index position (ON-1 variant)" |
| Edit 2: `0909` rationale update | JSON + CSV | Applied | Rationale reads: "...secondary: index type is int \| None (potential ON root cause may surface after AU-2 closure)" |
| Edit 3: Execution order | Execution ledger | Applied | Tiered ordering present: W1/W2/A1 -> W3 -> W4 -> W5 -> A2 |
| Edit 4: 53-fixture regression clause | Phase doc exit gates | Applied | Full-corpus gate includes: "the 53 non-targeted fixtures across all other taxonomy categories must not change status" |

### Pass2 status (confirmed)

- All pass1b edits verified applied
- W5 split criterion deferred to implementation time (non-blocking)
- Data consistency re-check passed
- No remaining blockers

---

## Section 4: Risk Assessment

### 4.1 Weak assumptions

| Risk | Severity | Mitigation |
|---|---|---|
| W3 may not fully resolve all Any/Unknown types, leaving some AU-2 fixtures unclosed | Low | Focused rerun will identify unclosed fixtures. They remain in the same workstream for subsequent passes. |
| AU-3 bridge fixtures may cascade to ON root causes after W3 resolves the Any/Unknown arm | Low | This is the designed behavior. Cascaded fixtures will appear in the focused rerun under ON categories and be picked up by W1/W2 if still open. |
| `1462` has an undocumented ON cascade risk similar to `0909` | Low | If W3 resolves Unknown in `Unknown \| None` to a concrete type, the result `concrete \| None` would be ON-2. The focused rerun catches this. No action required beyond this note. |
| W5 compiler/adaptation split criterion remains undefined | Low | Deferred to W5 implementation time per pass2. The 6 affected fixtures (4 ON-4 + 2 ON-5) are small enough to evaluate case-by-case during implementation. |

### 4.2 Ordering risks

The tiered execution order is sound:

- **Tier 1 (W1, W2, A1)**: No file overlaps between W1 (`narrow.rs`, `check.rs`, `function_flow.rs`, `expressions.rs`) and W2 (`container_literal_specialization.rs`, `guarded_index.rs`, `infer.rs`, `union.rs`). A1 is fixture-only. Safe to parallelize.
- **Tier 2 (W3)**: Shares `check.rs` with W1 and `infer.rs`/`union.rs` with W2. Must follow both W1 and W2 completion. The tier structure ensures this (Tier 2 runs after Tier 1).
- **Tier 3 (W4)**: Depends on W3 for stabilized types at compat entry points. Correctly sequenced.
- **Tier 4 (W5)**: Benefits from W1+W2 narrowing improvements. Correctly sequenced after Tier 1.
- **Tier 5 (A2)**: Handles residuals from W5. Correctly last.

**One clarification note**: The execution ledger says "Tier 2 (after W1): W3" but W3 also depends on W2's changes to `infer.rs`/`union.rs`. The tier structure implicitly ensures W3 follows W2 (Tier 2 starts after all of Tier 1 completes), but the explicit dependency annotation only mentions W1. This is cosmetic — the tier numbering alone is sufficient to enforce correct ordering. No action required.

### 4.3 Gate ambiguities

No ambiguities found. The three exit gates are well-defined:

1. **Root-cause presence gate**: Binary pass/fail per fixture — targeted signatures must be removed or explicitly transferred to the approved adaptation list.
2. **Full-corpus gate**: Requires new rerun artifact, taxonomy regeneration, no net regressions, and 53 non-targeted fixtures stable. All criteria are measurable.
3. **Policy gate**: No weakening of ownership/mutability, parse safety, or nonlocal mutable capture policy. Testable via `scripts/run_all_tests.sh --profile quick`.

---

## Section 5: Outstanding Non-Blocking Items

### 5.1 W5 compiler/adaptation split criterion (carried from pass1b, deferred per pass2)

The W5 acceptance criteria still reads "compiler-owned part of ON-4 and ON-5 removed; remaining residuals are explicit adaptation candidates only" without a formal decision rule for which cases are compiler-closeable vs adaptation-requiring. This was flagged in pass1b Improvement 1 and explicitly deferred to implementation time in pass2.

**Pass 3 assessment**: Acceptable to defer. The 6 affected fixtures are enumerable and small. The W5 implementer will evaluate each against the CFG and can document the criterion at that point. This does not gate Tier 1 (W1, W2, A1) or any subsequent tier.

### 5.2 `1462` cascade documentation (new observation)

Fixture `1462_course_schedule_iv` has type `Unknown | None` classified under AU-2. If W3 resolves `Unknown` to a concrete type, the remaining `concrete | None` pattern would be ON-2. This cascade risk is analogous to `0909` (which has an explicit rationale annotation) but `1462` lacks this annotation.

**Recommendation**: Optional. Could add to `1462` rationale: "secondary: if Unknown resolves, residual `concrete | None` may surface as ON-2 failure". Impact is negligible since the focused rerun catches all cascades.

---

## Section 6: Required Edits

None. No blocking edits remain. The optional `1462` annotation in Section 5.2 is cosmetic and does not affect implementation correctness.

---

## Section 7: Final Determination

**READY — the phase plan is approved for implementation.**

Justification:

1. All 58 fixtures have been individually audited. Every category assignment (ON vs AU) is correct. Every sub-root-cause ID accurately identifies the proximate type-system failure pattern. Every resolution mode (compiler/both/adaptation) is justified and defensible.

2. All data artifacts (JSON, CSV, taxonomy, phase document, execution ledger) are mutually consistent. No discrepancies found across any cross-reference check.

3. All four pass1b edits remain applied. Pass2 confirmations hold. The only outstanding item (W5 split criterion) was explicitly deferred to implementation time across two prior reviews and remains non-blocking.

4. The workstream structure (W1-W5, A1-A2) is complete — every root cause maps to exactly one workstream, every workstream has measurable acceptance criteria, and the tiered execution order respects all file-level and logical dependencies.

5. Exit gates are unambiguous: root-cause presence, full-corpus regression, and policy compliance are all binary and measurable.

6. Risk exposure is low and well-mitigated: cascade risks (0909, 1462, AU-3 bridge fixtures) are caught by the focused rerun by design. No ordering hazards exist in the tier structure.

Implementation may begin with Tier 1 (W1, W2, A1 in parallel).
