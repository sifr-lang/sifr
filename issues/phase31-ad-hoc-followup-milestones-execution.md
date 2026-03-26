# Phase 31 Follow-up Execution Tracker

Status: active (started 2026-03-26)
Owner: phase31 follow-up execution loop
References:
- `issues/phase31-ad-hoc-followup-milestones.md`
- `issues/phase31-strategy-synthesis-review.md`

Loop contract per milestone: Plan -> Implement -> Validate -> Demo -> PR -> Review -> Merge -> Docs update -> Next milestone

## Global Gates
- [x] Scope constrained to the active follow-up milestone
- [x] Root-cause fixes only (no fallback semantics)
- [x] Demo evidence recorded before milestone close
- [x] Local validation gates run: `scripts/run_all_tests.sh --profile quick`
- [x] Local validation gates run: `scripts/run_all_tests.sh`
- [ ] PR opened/reviewed/merged for this milestone

## Full Milestone To-Do (ordered)
1. [x] `m31_g_container_literal_specialization_and_state_tracking`
2. [ ] `m31_a_optional_flow_completion`
3. [ ] `m31_b_destructuring_and_composite_lvalues`
4. [ ] `m31_d_nested_function_pipeline_completion`
5. [ ] `m31_e_recursive_tree_surface_leetcode_closure`
6. [ ] `m31_l_tree_local_state_follow_on_closure`
7. [ ] `m31_h_local_name_binding_and_shadowing`
8. [ ] `m31_j_own_mut_leetcode_closure`
9. [ ] `m31_k_canonical_sifr_fixture_normalization`
10. [ ] `m31_i_corpus_fixture_canonicalization_for_multi_solution_files`

## Milestone: `m31_g_container_literal_specialization_and_state_tracking`

### Scope
- Specialize empty dict literals from first typed writes.
- Remove `Any` leakage through `dict.get(..., default)` during growth.
- Enforce deterministic conflict diagnostics for incompatible writes after specialization.
- Ensure specialized types patch the original `let` binding so codegen does not keep `HashMap<Any, Any>`.

### Root-cause changes
- Added container-specialization lowering module:
  - `crates/sifr_hir/src/lower/container_literal_specialization.rs`
- Integrated specialization + patching into statement-lowering flow:
  - `crates/sifr_hir/src/lower/statements.rs`
- Added/updated lowering state to carry pending specialization patches:
  - `crates/sifr_hir/src/lower/mod.rs`
- Improved dict method typing to avoid `Any` leakage and enforce key compatibility:
  - `crates/sifr_hir/src/lower/expressions.rs`
- Enabled dict index typing for assignable/Any dict key domains:
  - `crates/sifr_type_system/src/types.rs`

### Regression coverage
- Non-seed specialization regression:
  - `test_empty_dict_literal_specializes_from_first_subscript_write_and_get_default`
- Deterministic conflict diagnostic regression:
  - `test_empty_dict_literal_conflicting_write_reports_deterministic_error`
- Type-system regression:
  - `test_index_result_type` (dict[Any, V] indexing)

### Milestone demo
- Demo file: `demos/m31_g_container_literal_specialization_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- check demos/m31_g_container_literal_specialization_demo.sifr` (pass)
  - `cargo run -q -p sifr -- run demos/m31_g_container_literal_specialization_demo.sifr` (pass)

### Targeted corpus evidence
- Artifact: `verification/leetcode/phase31_m31g_wave1_results.json`
- Targeted ids: `0001`, `0242`, `0424`, `0523`, `0560`
- Status snapshot:
  - `0001` moved past prior `dict[Any, Any]` check failure (now run-stage optional/index follow-on)
  - `0242` moved past `Any` arithmetic (now dict comparability/optional key follow-on)
  - `0424` moved past `dict[Any, Any]` and `Any` arithmetic (now local-name follow-on)
  - `0523` moved past `dict[Any, Any]` and `Any` arithmetic (now optional-flow follow-on)
  - `0560` moved past `dict[Any, Any]` and `Any` arithmetic (now optional-flow follow-on)

### Local validation evidence
- `scripts/run_all_tests.sh --profile quick` (pass)
- `scripts/run_all_tests.sh` (pass)

### Closeout status
- `m31_g` definition of done satisfied for removal of `dict[Any, Any]` / `Any` arithmetic blockers across the five target ids.
- Follow-on failures are reclassified into downstream milestones (`m31_a`, `m31_h`, and run-stage optional narrowing closure).

## Milestone: `m31_a_optional_flow_completion` (slice 6: dict membership guarded indexing)

### Scope for this slice
- Add compositional flow proof for keyed dict indexing under explicit membership guards.
- Keep unguarded dict indexing behavior unchanged (`T | None` outside proven-safe flow).
- Ensure codegen follows HIR-proven non-optional dict index results without leaking Optional types.

### Root-cause changes
- Added dict membership guard shape to sequence-guard model:
  - `crates/sifr_hir/src/lower/sequence_guards.rs`
- Added detection for:
  - `key in dict`
  - `key in dict.keys()`
  - false-exit narrowing from `key not in dict`
  - `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
- Added dict-index narrowing when guard proof exists:
  - `crates/sifr_hir/src/lower/guarded_index.rs`
- Fixed codegen to respect HIR result type for dict index:
  - keep optional projection for optional HIR index type
  - emit guarded unwrap (`expect`) for proven non-optional index type
  - `crates/sifr_codegen/src/lower_expr.rs`

### Regression coverage
- `test_dict_index_narrows_after_in_membership_guard`
- `test_dict_index_narrows_after_keys_membership_guard_with_expression_key`
- `test_dict_index_narrows_after_not_in_early_return_guard`
- `lowers_dict_index_to_optional_projection_for_optional_hir_type`
- `lowers_dict_index_to_expect_for_non_optional_hir_type`

### Demo evidence
- Demo file: `demos/phase31_dict_membership_guard_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- check demos/phase31_dict_membership_guard_demo.sifr` (pass)
  - `cargo run -q -p sifr -- run demos/phase31_dict_membership_guard_demo.sifr` (pass)

### Targeted corpus evidence
- Artifact: `verification/leetcode/phase31_m31a_wave6_dict_membership_results.json`
- Targeted ids: `0001`, `0523`, `0560`
- Status snapshot:
  - `0523` moved to `PASS`
  - `0560` moved to `PASS`
  - `0001` remains `RUN_ERROR` on raw fixture missing guaranteed return path (not a dict-membership optional-flow failure)

### Local validation evidence
- `scripts/run_all_tests.sh --profile quick` (pass)
- `scripts/run_all_tests.sh` (pass)

### Slice closeout status
- Slice 6 goal satisfied for dict membership guarded narrowing and codegen parity.
- `m31_a` milestone remains open for remaining optional-flow closure items listed in the parent milestone plan.

## Milestone: `m31_a_optional_flow_completion` (slice 7: len-alias range guards)

### Scope for this slice
- Teach range-guard detection to compose through local aliases of `len(sequence)`.
- Preserve existing optional behavior for unproven index shapes.
- Keep fixes general (flow fact propagation), not fixture pattern matching.

### Root-cause changes
- Added lowering-time `len(...)` alias facts:
  - `crates/sifr_hir/src/lower/len_aliases.rs`
  - tracks assignments such as `n = len(nums)` and alias propagation through simple name assignment
- Recorded/cleared len-alias facts across assignment forms:
  - `crates/sifr_hir/src/lower/statements.rs`
- Extended range guard detection to resolve alias-backed length anchors:
  - `range(n)` where `n` aliases `len(seq)`
  - `range(n - 1, -1, -1)` where `n` aliases `len(seq)`
  - `crates/sifr_hir/src/lower/sequence_guard_detection.rs`

### Regression coverage
- `test_range_len_alias_list_index_reveals_element_type`
- `test_reverse_range_len_alias_list_index_reveals_element_type`

### Demo evidence
- Demo file: `demos/phase31_len_alias_range_guard_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- check demos/phase31_len_alias_range_guard_demo.sifr` (pass)
  - `cargo run -q -p sifr -- run demos/phase31_len_alias_range_guard_demo.sifr` (pass)

### Targeted corpus evidence
- Artifact: `verification/leetcode/phase31_m31a_wave7_len_alias_results.json`
- Targeted ids: `0053`, `0127`, `0238`, `0322`, `0502`, `0743`, `0746`
- Status snapshot:
  - count-level status remains `CHECK_ERROR=7`
  - `0238` reduced from three optional arithmetic errors to two; `nums[i]` under `for i in range(n)` now narrows after `n = len(nums)`
  - remaining `0238` optional errors are localized to sized-local `result[i]` flow (next optional-flow slice)

### Local validation evidence
- `scripts/run_all_tests.sh --profile quick` (pass)
- `scripts/run_all_tests.sh` (pass)

### Slice closeout status
- Slice 7 goal satisfied for len-alias range-guard propagation.
- `m31_a` milestone remains open for sized-local append growth proofs, subtractive/value-dependent recurrence indexing, and residual canonicalization follow-ons.

## Milestone: `m31_a_optional_flow_completion` (slice 8: alias-backed end-pointer while guards)

### Scope for this slice
- Propagate end-pointer facts through `len(...)` aliases (`n = len(seq)` then `i = n - 1`).
- Use `while i >= 0` as an in-range proof when `i` is a known end-pointer for the sequence.
- Preserve existing optional behavior for unproven index shapes.

### Root-cause changes
- Extended sequence-pointer recording to resolve alias-backed `len(...) - 1` patterns:
  - `crates/sifr_hir/src/lower/sequence_pointers.rs`
- Extended true-guard detection for `i >= 0` when `i` is a known end-pointer:
  - emits `IndexVarInRange` for the associated sequence
  - `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
- Added guarded-index regression for while-loop end-pointer alias narrowing:
  - `crates/sifr_hir/src/lower/guarded_index.rs`

### Regression coverage
- `test_while_end_pointer_len_alias_reveals_element_type`

### Demo evidence
- Demo file (updated): `demos/phase31_len_alias_range_guard_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- check demos/phase31_len_alias_range_guard_demo.sifr` (pass)
  - `cargo run -q -p sifr -- run demos/phase31_len_alias_range_guard_demo.sifr` (pass)

### Targeted corpus evidence
- Artifact: `verification/leetcode/phase31_m31a_wave8_end_pointer_alias_results.json`
- Targeted ids: `0053`, `0127`, `0238`, `0322`, `0502`, `0743`, `0746`
- Status snapshot:
  - count-level status remains `CHECK_ERROR=7`
  - `0238` reduced from two optional arithmetic errors to one
  - remaining `0238` optional failure is localized to sized-local `result[i]` flow under append growth

### Local validation evidence
- `scripts/run_all_tests.sh --profile quick` (pass)
- `scripts/run_all_tests.sh` (pass)

### Slice closeout status
- Slice 8 goal satisfied for alias-backed end-pointer while-guard narrowing.
- `m31_a` remains open for sized-local growth proofs and subtractive/value-dependent recurrence indexing.

## Milestone: `m31_a_optional_flow_completion` (slice 9: append-growth sized-list facts)

### Scope for this slice
- Recognize local append-growth loops as sequence-shape evidence.
- Propagate that evidence into guarded index narrowing.
- Keep matching strict to root-cause flow forms (single append per iteration over proven range bounds).

### Root-cause changes
- Added append-growth sequence-shape module:
  - `crates/sifr_hir/src/lower/append_growth_shapes.rs`
  - detects `for i in range(...): target.append(value)` and records `SizedByAnchor` facts
- Integrated append-growth shape recording into for-loop lowering:
  - `crates/sifr_hir/src/lower/statements.rs`
- Added guarded-index regression for append-growth sized list under alias-backed while guards:
  - `crates/sifr_hir/src/lower/guarded_index.rs`
- Reused slice 7/8 alias and end-pointer infrastructure to compose full proof path.

### Regression coverage
- `test_append_growth_shape_allows_index_under_alias_guard`

### Demo evidence
- Demo file (updated): `demos/phase31_len_alias_range_guard_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- check demos/phase31_len_alias_range_guard_demo.sifr` (pass)
  - `cargo run -q -p sifr -- run demos/phase31_len_alias_range_guard_demo.sifr` (pass)

### Targeted corpus evidence
- Artifact: `verification/leetcode/phase31_m31a_wave9_append_growth_results.json`
- Targeted ids: `0053`, `0127`, `0238`, `0322`, `0502`, `0743`, `0746`
- Status snapshot:
  - `PASS=1`, `CHECK_ERROR=6`
  - confirmed new pass: `0238`

### Local validation evidence
- `scripts/run_all_tests.sh --profile quick` (pass)
- `scripts/run_all_tests.sh` (pass)

### Slice closeout status
- Slice 9 goal satisfied for append-growth sized-list shape propagation.
- `m31_a` remains open for residual cases (`0053`, `0322`, and source-canonicalization/mutability follow-ons in `0127`, `0502`, `0743`, `0746`).

## Milestone: `m31_a_optional_flow_completion` (slice 10: guarded pop/popleft narrowing)

### Scope for this slice
- Remove optional leakage for `pop`/`popleft` when control flow proves a non-empty sequence.
- Keep unguarded pop behavior unchanged (`T | None`).

### Root-cause changes
- Refined method-call return typing after method resolution:
  - for receiver names with active non-empty guards, `pop`/`popleft` returns are narrowed from `T | None` to `T`
  - narrowed domain is constrained to zero-arg `list/deque pop/popleft` only
  - files: `crates/sifr_hir/src/lower/expressions.rs`, `crates/sifr_hir/src/lower/nonempty_method_narrowing.rs`
- Added codegen parity for narrowed pop calls:
  - compiler-proven non-empty pop expressions now unwrap `Some(...)` with invariant `unreachable!` guard
  - files: `crates/sifr_codegen/src/intrinsic_method_emitters.rs`, `crates/sifr_codegen/src/stmt_support_emitter.rs`
- Added regressions for guarded vs unguarded pop:
  - file: `crates/sifr_hir/src/lower/expressions_tests.rs`
  - coverage includes guarded indexed `list.pop(i)` and guarded `dict.pop(key)` staying optional
- Added codegen regression:
  - file: `crates/sifr_codegen/src/lib_codegen_tests.rs`
- Added slice demo:
  - `demos/phase31_pop_guard_narrowing_demo.sifr`

### Regression coverage
- `test_guarded_list_pop_narrows_to_element_type`
- `test_unguarded_list_pop_stays_optional`

### Targeted corpus evidence
- Artifact: `verification/leetcode/phase31_m31a_wave10_pop_guard_results.json`
- Targeted ids: `0053`, `0127`, `0322`, `0502`, `0743`, `0746`
- Status snapshot:
  - count-level status remains `CHECK_ERROR=6`
  - `0127` moved past optional pop leakage (`None | T` -> `T`) and now fails only on generic-type precision plus canonical mutability (`wordList` must be `mut`)

### Local validation evidence
- `scripts/run_all_tests.sh --profile quick` (pass)
- `scripts/run_all_tests.sh` (pass)

### Slice closeout status
- Slice 10 goal satisfied for guarded pop/popleft optional-flow narrowing.
- Remaining `m31_a` work is now concentrated in fixed-index parameter head reads, subtractive recurrence indexing, and cross-milestone canonicalization/generic follow-ons.
