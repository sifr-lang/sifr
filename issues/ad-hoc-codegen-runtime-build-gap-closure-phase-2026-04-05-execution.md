# Ad-hoc Phase: Codegen Runtime Build Gap Closure — Execution Log

Status: in_progress (started 2026-04-05)
Owning phase: `issues/ad-hoc-codegen-runtime-build-gap-closure-phase-2026-04-05.md`

## Wave log

### 2026-04-05 wave-0 (baseline capture)
- scope:
  - initialize execution tracking for the 58-case `codegen_runtime_build_gap` bucket
  - capture reproducible pre-fix diagnostics for wave deltas
- artifacts:
  - pending
- notes:
  - execution started from `main` at clean worktree state
  - workstream order follows locked sequence in the owning phase doc

### 2026-04-05 wave-1 (ws1 type-contract patchset A)
- scope:
  - close invalid `None` compare lowering surfaces (`is/is not` and `==/!=`)
  - harden simple/structured `if` let-else synthesis for `a is None or b is None`
  - remove spurious auto-Display trait obligations on nested class fields
  - fix string-key clone path in attribute-subscript dict assignment lowering
- compiler files touched:
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/helpers.rs`
  - `crates/sifr_codegen/src/class_emitter.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
- artifacts:
  - baseline snapshot: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_start.json` (`0 pass / 20 fail`)
  - after patchset A: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_after_patch1.json` (`2 pass / 18 fail`)
- observed deltas:
  - `0189_rotate_array`: FAIL -> PASS
  - `0783_minimum_distance_between_bst_nodes`: FAIL -> PASS
  - `0211_design_add_and_search_words_data_structure`: compile failure -> runtime assertion failure
  - `0729_my_calendar_i`: `E0277` removed; residual `E0596` remains

### 2026-04-05 wave-2 (ws1 type-contract patchset B)
- scope:
  - fix simple list-of-string subscript `+=` lowering (`push_str`/`as_str` path)
  - route method-call registry lowering through effective local binding types when expr types are `Any`/`Unknown`
  - resolve alias-backed object types in registry method dispatch
  - add guarded fallback rewrite for list `append` in stmt-only method-call fallback path
- compiler files touched:
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/methods/mod.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
- artifacts:
  - after patchset B: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_after_patch2.json`
    - summary: `3 pass / 17 fail` (same fail count as latest run gate, but build-gap shape improved)
  - probe rerun after additional guarded-compare lowering attempts:
    - `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_after_patch3.json`
    - summary: `3 pass / 17 fail` (no status/error-code delta vs patchset B)
- observed deltas vs wave-1 patchset A:
  - `0006_zigzag_conversion`: FAIL (`E0308`) -> PASS
  - `0046_permutations`: FAIL (`E0308`) -> FAIL (runtime; no Rust error code)
- notes:
  - `0046` now compiles; residual is runtime behavior (`[]` produced), indicating follow-up semantic bug in option-bool/index truthiness handling rather than Rust build-gap emission.
  - `0567_permutation_in_string` remains blocked on option-vs-scalar compare emission in guarded conjunction (`c is not None and c == ch` lowering currently emits `Option<String> == String`).
  - attempted `Some(mut x)` let-else narrowing tweak for `detect_is_none_var` to remove `0729` `E0596`; this removed `E0596` but introduced broad `E0507` regressions (including `0783`) and was reverted.
  - post-revert confirmation artifact: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_after_patch5.json` (`3 pass / 17 fail`, no delta vs patchset B steady state).

### 2026-04-05 wave-3 (ws1 type-contract patchset C)
- scope:
  - close borrowed-name guarded compare emission where effective local binding type is `Option<T>` while peer side is scalar (`0567` pattern)
  - keep borrow semantics safe by cloning non-`Copy` borrowed scalars before wrapping with `Some(...)`
  - route guarded option-compare lowering to preserve plain name identity where the guard already established option context
- compiler files touched:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- artifacts:
  - after patchset C: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave2_after_patch6.json`
    - summary: `4 pass / 16 fail`
- observed deltas vs wave-2 steady state (`wave1_after_patch5`):
  - `0567_permutation_in_string`: FAIL (`E0308`) -> PASS
- PR:
  - draft: https://github.com/yaseralnajjar/sifr/pull/1575

### 2026-04-05 wave-4 (ws1 type-contract patchset D)
- scope:
  - close recursive optional-field assignment mismatch (`Option<T>` -> `Option<Box<T>>`) for both direct field writes and constructor-call argument adaptation
  - include field assignments in mutation analysis so class-instance locals become mutable when fields are reassigned
  - harden option-pattern lowering to use mutable bindings for non-borrowed option locals, eliminating residual `E0594/E0596` mutability errors in narrowed paths
- compiler files touched:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/hir_analysis/queries.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
- artifacts:
  - after patchset D: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave3_after_patch7.json`
    - summary: `10 pass / 10 fail`
- observed deltas vs wave-3 patchset C (`wave2_after_patch6`):
  - `0105_construct_binary_tree_from_preorder_and_inorder_traversal`: FAIL (`E0308`) -> PASS
  - `0106_construct_binary_tree_from_inorder_and_postorder_traversal`: FAIL (`E0308`) -> PASS
  - `0108_convert_sorted_array_to_binary_search_tree`: FAIL (`E0308`) -> PASS
  - `0450_delete_node_in_a_bst`: FAIL (`E0308`) -> PASS
  - `0617_merge_two_binary_trees`: FAIL (`E0308`) -> PASS
  - `0701_insert_into_a_binary_search_tree`: FAIL (`E0308`) -> PASS
  - `0729_my_calendar_i`: FAIL (`E0596`) -> FAIL (runtime assertion; no Rust error code)
  - `0894_all_possible_full_binary_trees`: FAIL (`E0308/E0599/E0631`) -> FAIL (`E0382/E0599/E0631`)
- notes:
  - ws1 remaining failures are now split between runtime semantics (`0046`, `0211`, `0729`) and reduced compile-surface residuals (`0048`, `0124`, `0138`, `0435`, `0572`, `0894`, `1958`).

### 2026-04-05 wave-5 (ws1 type-contract patchset E)
- scope:
  - close nested-list subscript assignment type mismatch when source expression is `Option<T>` and destination element is `T`
  - align both structured and simple nested-subscript assignment lowering to guard assignment behind `Some(...)` in option-valued source paths
- compiler files touched:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
- artifacts:
  - after patchset E: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave4_after_patch8.json`
    - summary: `11 pass / 9 fail`
- observed deltas vs wave-4 patchset D (`wave3_after_patch7`):
  - `0048_rotate_image`: FAIL (`E0308`) -> PASS
