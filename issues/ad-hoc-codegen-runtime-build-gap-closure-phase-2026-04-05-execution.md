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
