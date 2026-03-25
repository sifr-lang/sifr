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
