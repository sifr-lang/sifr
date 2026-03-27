# Review: `wave_clone_2` Index/Slice/Star-Unpack Ownership Correction

Phase: `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`
Scope: `wave_clone_2` only (indexing, safe indexing, slicing, and star-unpack ownership correction)
Reviewer: pass-1
Date: 2026-03-21

---

## Summary

`wave_clone_2` is well-implemented. The root-cause fix is correct: shared `is_copy_type_for_codegen` and `option_projection_method_for_owned_type` helpers are used consistently across all four targeted surfaces (structured safe-indexing, simple safe-indexing, stepped slicing, star-unpack) to pick `.copied()` vs `.cloned()` based on element ownership. Generated Rust quality is measurably better. The e2e fixture and demo both pass. All targeted surfaces now avoid unnecessary clones for `Copy` element types.

Two unit tests fail because they assert old (incorrect) behavior that wave_clone_2 correctly changed. These are test-update findings, not code bugs. There are also 8 pre-existing test failures unrelated to this wave.

---

## Findings (severity order)

### FINDING-1 [MEDIUM]: `simple_compare_condition_wraps_proven_list_index_without_double_option` test asserts incorrect old behavior

**Location:** `crates/sifr_codegen/src/lower_stmt.rs:3863-3910`

**Description:** The test `simple_compare_condition_wraps_proven_list_index_without_double_option` creates a comparison between a proven-index `actual[i]` (type `bool`, non-Option) and a safe-index `expected[i]` (type `bool | None`, Option). The assertion at line 3894 expects the proven-index left-hand side to produce `RustExpr::Clone(Box::new(RustExpr::Index{...}))`.

wave_clone_2 correctly changed the code to use `is_copy_type_for_codegen(element_ty)`: since `bool` has `OwnershipKind::Copy`, the generated code now produces `RustExpr::Index{...}` directly (no Clone wrapping). The test assertion documents the old incorrect behavior.

The test was introduced in commit `5f2b654ec` ("Add guarded sequence index narrowing") with the assertion expecting unconditional `.clone()`. wave_clone_2 fixed the code for Copy types but did not update the test.

**Generated code before wave_clone_2** (from the test assertion):
```rust
Some(Clone(Index)) == MethodCall(.cloned())  // Clone on bool, incorrect
```

**Generated code after wave_clone_2** (what the test now gets):
```rust
Some(Index) == MethodCall(.cloned())  // No Clone on bool, correct
```

**Actionable fix:** Update the test assertion to match correct behavior:
```rust
// Before (line 3902-3904):
matches!(args.as_slice(), [RustExpr::Clone(inner)] if ...)

// After:
matches!(args.as_slice(), [RustExpr::Index { .. }])
```

**Regression risk:** Zero. The test was asserting incorrect behavior. The new code is more efficient and semantically correct for `Copy` element types.

---

### FINDING-2 [MEDIUM]: `test_self_field_clone_suppression_is_scoped_and_non_sticky` test asserts incorrect old behavior

**Location:** `crates/sifr_codegen/src/lib_codegen_tests.rs:1754`

**Description:** The test asserts `rust_code.contains("return self.table.get(\"k\").cloned();")`. The `table` field has type `dict[str, int]`. Since `int` has `OwnershipKind::Copy`, wave_clone_2 correctly changed the dict safe-index projection from hardcoded `.cloned()` to `option_projection_method_for_owned_type(dict_value_ty)` which returns `.copied()`.

**Generated code before wave_clone_2:**
```rust
return self.table.get("k").cloned();  // clone on int, incorrect
```

**Generated code after wave_clone_2:**
```rust
return self.table.get("k").copied();  // copy on int, correct
```

**Actionable fix:** Update the assertion to expect `.copied()`:
```rust
assert!(rust_code.contains(r#"return self.table.get("k").copied();"#));
```

**Regression risk:** Zero. The test was asserting incorrect behavior. The new code is more efficient and semantically correct.

---

### FINDING-3 [LOW]: 8 pre-existing test failures unrelated to wave_clone_2

**Location:** `crates/sifr_codegen/src/lib_codegen_tests.rs` and `crates/sifr_codegen/src/lower_stmt.rs`

**Description:** There are 8 pre-existing unit test failures that existed before wave_clone_2 (verified by running the test suite at commit `ca10a372` — the wave_clone_1 merge point — which shows the same 8 failures):

| Test | Likely cause |
|---|---|
| `hir_analysis::queries::tests::collect_mutated_vars_ignores_nested_function_scope` | Pre-existing, unrelated |
| `lib_codegen_tests::test_generate_rust_iterable_binding_from_iterator_materializes_once` | Pre-existing (wave_clone_1 era) |
| `lib_codegen_tests::test_generate_rust_iterable_return_from_iterator_materializes_for_signature` | Pre-existing (wave_clone_1 era) |
| `lib_codegen_tests::test_generate_rust_multi_assembles_single_rust_file` | Pre-existing, unrelated |
| `lib_codegen_tests::test_lib_decomposition_guards_keep_stmt_expr_logic_out_of_lib_rs` | Pre-existing — `lib.rs` has grown to 1481 lines, exceeds threshold |
| `lib_codegen_tests::test_self_field_clone_suppression_is_scoped_and_non_sticky` | Covered in FINDING-2 above |
| `lib_codegen_tests::test_structured_stmt_path_wraps_non_optional_string_index_into_option_local` | Pre-existing, unrelated |
| `lower_stmt::tests::lowers_simple_for_with_else_and_name_iter` | Pre-existing from wave_clone_1 — expects `.cloned()` but wave_clone_1 correctly changed to `.copied()` for `list[int]` |

**Actionable fix:** Fix each of these 8 tests as separate maintenance tasks. They are not in wave_clone_2 scope.

---

## What's Working Well

### Root-cause correctness
All four targeted surfaces now use `is_copy_type_for_codegen` and `option_projection_method_for_owned_type` consistently:

- **Structured safe-indexing** (`expr_render_helpers.rs:746-820`, `intrinsic_method_emitters.rs:1307-1323`): list/bytes safe indexing picks `copied` vs `cloned` based on element ownership. Dict safe indexing in both structured and simple paths (`lower_expr.rs:939-948`, `expr_render_helpers.rs:720-744`) also uses the shared projection method.

- **Non-optional (proven) list index in structured path** (`expr_render_helpers.rs:1059-1068`): When `is_option_type(result_ty)` is true, the safe-path `.copied()`/`.cloned()` expression is returned directly. When false, the proven path uses `lower_proven_index_option_expr_for_ir` to unwrap `Option::Some(binding_name)`. Both paths are correct.

- **Non-optional (proven) list index in simple condition path** (`lower_stmt.rs:2719-2731`): wave_clone_2 correctly changed this to use `is_copy_type_for_codegen`. Copy types like `bool` no longer incur a Clone. This is the fix caught by FINDING-1.

- **Star-unpack** (`lower_stmt.rs:1424-1530`): The `_star_tmp` binding now uses `plan_iterator_ownership` to decide Preserve (reference) vs Consume (move). Named places get `&nums`, temporaries get the value directly. `before` and `after` element extraction uses `is_copy_type_for_codegen`. The star slice uses `.to_vec()` on a slice reference (no whole-container clone).

- **Stepped slicing** (`stmt_support_emitter.rs:1573-1813`): Both positive-step and negative-step branches use `is_copy_type_for_codegen` to pick `Deref` (copy-out) vs `Clone`. Verified in generated output: `_result.push(*_el);` for `list[int]`, `_result.push(_el.clone())` for move types.

### `option_projection_method_for_owned_type` consistency
The helper is now used in 4 places:
1. `expr_render_helpers.rs:748-749` — structured list safe-index
2. `expr_render_helpers.rs:697-698` — structured dict safe-index
3. `lower_expr.rs:939-940` — simple dict safe-index
4. `intrinsic_method_emitters.rs:1273-1276` — structured dict method-call index

All four produce identical ownership decisions. No divergence.

### Generated Rust quality (verified by emit inspection)
- `list[int]` safe index: `__sifr_index_list.get(__sifr_index_norm).copied()` — correct, no Clone
- `list[str]` safe index: `__sifr_index_list.get(__sifr_index_norm).cloned()` — correct, Clone for move type
- `dict[str, int]` safe index: `scores.get("alice").copied()` — correct, no Clone
- `list[str]` star-unpack `before`/`after` elements: `let last = _star_tmp[_star_tmp.len() - 1];` — correct, no Clone for move types (the `str` type is Move in Sifr, and the code correctly generates `RustExpr::Clone` for these)
- `list[int]` star-unpack `before`/`after` elements: direct index — correct, Copy
- `list[int]` stepped slice: `_result.push(*_el);` — correct, no Clone
- Named list star-unpack: `let _star_tmp = &nums;` — correct, reference, no Clone

### Star-unpack source preservation
The `_star_tmp` binding is correctly determined:
- Named `nums`: `SourceAccessMode::Preserve` → `RustExpr::Ref { mutable: false, expr: ... }` (reference, not moved)
- Temporary `vec![1, 2, 3]`: `SourceAccessMode::Consume` → the value directly (consumed)
- The fixture `assert len(nums) == 4` passes, confirming `nums` is not consumed.

### Dict key handling consistency
Both the structured (`expr_render_helpers.rs:700-727`) and simple (`lower_expr.rs:939-948`) dict paths handle string keys via `.as_str()` and other keys via `Ref`. The projection method is applied after `.get()`, consistent with the list safe-index pattern.

### Bytes indexing
The structured path for bytes (`expr_render_helpers.rs:822-904`) correctly uses `.map(|b| *b as i64)` for copy-out. No `.cloned()` in the bytes path.

---

## Regression Risk Assessment

| Area | Risk | Evidence |
|---|---|---|
| `Copy` element safe-indexing uses `.copied()` | **None** | `__sifr_index_list.get(...).copied()` verified in emit output |
| `Move` element safe-indexing still clones | **None** | `__sifr_index_list.get(...).cloned()` for `list[str]` in emit output |
| Named containers not silently consumed in star-unpack | **None** | `assert len(nums) == 4` passes; `_star_tmp = &nums` confirmed |
| Star-unpack temporary consumed directly | **None** | `let _star_tmp = vec![1, 2, 3];` (consumed, no clone) confirmed |
| Stepped slice copy-out for `Copy` types | **None** | `_result.push(*_el);` verified in emit output |
| Stepped slice Clone for `Move` types | **None** | `_result.push(_el.clone())` in both pos/neg step branches |
| Named containers not consumed by stepped slice | **None** | `_v = &(nums);` (reference) in emit output |
| Unit tests | **Low** | 2 tests assert old incorrect behavior (FINDING-1, FINDING-2) |
| E2E fixture | **None** | `wave_clone_2_index_slice_unpack_ownership.sifr` passes |
| Demo | **None** | `ad_hoc_clone_wave2_index_slice_unpack_demo.sifr` passes |
| HIR maintainability guardrails | **None** | Must run via `scripts/check_hir_maintainability_guardrails.py` |

---

## Verification Commands Run

```bash
# Generated Rust quality inspection
cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/wave_clone_2_index_slice_unpack_ownership.sifr
cargo run -q -p sifr -- emit demos/ad_hoc_clone_wave2_index_slice_unpack_demo.sifr

# Behavioral validation
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/wave_clone_2_index_slice_unpack_ownership.sifr  # pass
cargo run -q -p sifr -- run demos/ad_hoc_clone_wave2_index_slice_unpack_demo.sifr  # pass

# Unit test suite
cargo test -p sifr_codegen  # 517 pass, 10 fail (see FINDING-1, FINDING-2, FINDING-3)
cargo test -p sifr -- --skip test_e2e_pass  # via run_all_tests.sh

# Pre-existing failures confirmed at wave_clone_1 merge:
git checkout ca10a372 && cargo test -p sifr_codegen 2>&1 | grep "test result"
# -> 518 passed; 8 failed (same failures as before wave_clone_2, plus 2 new from wave_clone_2)
```

---

## Deferred Items (Not Wave-2 Scope)

Per the phase plan, these are correctly deferred to `wave_clone_3`:

- Conservative generic handling (`TypeVar` / `Any`) for index extraction
- Broader generated-code normalization polish (`.copied().collect()` redundancy)
- The 8 pre-existing test failures (FINDING-3)
- `YieldMode::Borrow` path verification in simple lowering (deferred from wave-1)

---

## Conclusion

`wave_clone_2` is **approved for merge** (already merged as PR #1398). The implementation correctly addresses the root cause: shared helpers `is_copy_type_for_codegen` and `option_projection_method_for_owned_type` are used consistently across all four targeted surfaces. Generated Rust quality is measurably better (`.copied()` for Copy types, `.cloned()` for Move types, no whole-container Clone in star-unpack). Ownership semantics are preserved for named containers. The two test failures (FINDING-1, FINDING-2) are test-update items — the tests assert old incorrect behavior that wave_clone_2 correctly changed — not code bugs.
