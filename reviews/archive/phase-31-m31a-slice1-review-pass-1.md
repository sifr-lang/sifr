# Phase 31 m31a Slice 1 Review: Guarded Sequence Index Narrowing

**Reviewer:** Claude Code
**Date:** 2026-03-12
**Slice:** `m31_a_optional_narrowing_core` - slice 1 (guarded sequence index narrowing)
**Status:** **NEEDS WORK** - Clippy and formatting issues in new code

---

## Executive Summary

The slice implements guarded sequence index narrowing, allowing the compiler to prove that list/string index results are definite values when control flow already proves the index is in range. The implementation is architecturally sound with correct HIR guard tracking and appropriate codegen for non-optional indexes. However, the new code has 1 clippy warning and multiple formatting issues that need to be addressed before final merge approval.

---

## 1. HIR Guard Tracking - Correctness

### 1.1 SequenceGuard Data Structure

**Location:** `crates/sifr_hir/src/lower/sequence_guards.rs`

The `SequenceGuard` enum is well-designed with two variants:
- `MinLength { sequence: String, min_len: usize }` - tracks minimum length guarantees
- `IndexVarInRange { sequence: String, index_var: String }` - tracks index variables known to be in range

**Correctness:** The implementation correctly deduplicates guards by sequence name, taking the maximum min_len when multiple `MinLength` guards exist. The `IndexVarInRange` guards correctly avoid duplicates.

**Issue Found:** Line 37 uses wildcard `_ => false` in a match with only two variants. Clippy flags this:
```
error: wildcard matches only a single variant and will also match any future added variants
  --> crates/sifr_hir/src/lower/sequence_guards.rs:37:21
   |
37 |                     _ => false,
   |                     ^ help: try: `SequenceGuard::MinLength { .. }`
```

### 1.2 Guard Detection in Statements

**Location:** `crates/sifr_hir/src/lower/statements.rs` (lines 1970-2096)

The guard detection functions are comprehensive:

- **`detect_true_sequence_guards`**: Detects guards that prove a sequence has elements or an index is in range:
  - Variable truthiness (`if seq:` → min_len >= 1)
  - Comparison: `i < len(seq)` → IndexVarInRange
  - Comparison: `len(seq) == N` → MinLength(N)
  - Comparison: `len(seq) > N` → MinLength(N+1)
  - `not` unary: Inverts false-exit guards

- **`detect_false_exit_sequence_guards`**: Detects guards from early-return patterns:
  - `if not seq: return` → min_len >= 1 after the guard
  - `if len(seq) == 0: return` → min_len >= 1 after the guard

- **`detect_range_sequence_guards`**: Detects `for i in range(len(seq))` patterns

**Correctness:** The implementation correctly saves/restores sequence guards around if/while/for bodies, preserving guards only within the appropriate scopes. The guard tracking is integrated into the statement lowering flow.

### 1.3 Type Narrowing for Subscripts

**Location:** `crates/sifr_hir/src/lower/guarded_index.rs`

The `guarded_sequence_index_result_type` function correctly:
- Checks if the subscript target is a simple name (not a complex expression)
- Matches on `List[T]` and `Str` types
- Calls `has_guarded_sequence_index` to verify the index is guarded
- Returns the element type (not wrapped in Option) when guarded

**Test Coverage:** 5 HIR tests verify the behavior:
- `test_guarded_string_index_in_while_reveals_str` - while loop case
- `test_range_len_list_index_reveals_element_type` - for/range case
- `test_early_return_non_empty_guard_reveals_element_type` - early return case
- `test_early_return_non_empty_guard_let_uses_narrowed_index_type` - let assignment case
- `test_unguarded_list_index_stays_optional` - negative case

**Status:** All tests pass.

---

## 2. Codegen for Proven Non-Optional Indexes

### 2.1 Non-Optional Index Handling

**Location:** `crates/sifr_codegen/src/stmt_support_emitter.rs` (lines 3070-3144)

The `lower_non_option_index_expr_for_ir` function generates different Rust code based on the object type:

- **List**: Direct indexing with `Clone` and cast to `usize`:
  ```rust
  __list.clone()[index as usize]
  ```

- **String**: Uses `chars().nth()` with let-else and unreachable:
  ```rust
  let Some(__indexed_char) = __string.chars().nth(index as usize) else {
      unreachable!("compiler-verified string index should be in range")
  };
  __indexed_char.to_string()
  ```

- **Tuple**: Direct field access for literal indices

### 2.2 Integration with Statement Lowering

**Location:** `crates/sifr_codegen/src/stmt_support_emitter.rs` (lines 1504-1512)

The codegen correctly checks if the index result type is optional:
```rust
if !crate::helpers::is_option_type(ty) {
    if let Some(lowered) = self.lower_non_option_index_expr_for_ir(object, index)? {
        return Ok(Some(lowered));
    }
}
```

This ensures that only proven-safe indexes bypass the structured optional-lowering path.

### 2.3 Test Coverage

10 codegen tests verify the behavior:
- `simple_let_declines_non_optional_list_index_to_allow_structured_lowering`
- `simple_return_declines_non_optional_string_index_to_allow_structured_lowering`
- `simple_compare_condition_wraps_proven_list_index_without_double_option`
- `test_structured_stmt_path_wraps_non_optional_string_index_into_option_local`
- `test_structured_stmt_path_handles_non_optional_string_index_return_expr`
- And more...

**Status:** All tests pass.

---

## 3. Regression Risk in Compare/Let/Return Lowering

### 3.1 Compare Lowering

**Location:** `crates/sifr_hir/src/lower/expressions.rs` (lines 332-450)

The compare lowering is **unchanged** by this slice. The implementation handles:
- `in` / `not in` operators
- All comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`, `is`, `is not`)
- Type checking for comparisons

**Risk:** None - no changes to this code path.

### 3.2 Let Statement Lowering

**Location:** `crates/sifr_hir/src/lower/statements.rs` (lines 931-1000)

Let statements now correctly use the narrowed type from `guarded_sequence_index_result_type` when evaluating the right-hand side. The type checking at lines 959-969 correctly verifies assignability with the narrowed type.

**Test:** `test_early_return_non_empty_guard_let_uses_narrowed_index_type` verifies this works.

**Risk:** Low - the change is additive and preserves existing behavior for non-guarded cases.

### 3.3 Return Statement Lowering

**Location:** `crates/sifr_hir/src/lower/statements.rs` (lines 1550+)

Return statements correctly use the narrowed type when checking return type compatibility. The codegen tests `test_structured_stmt_path_handles_non_optional_string_index_return_expr` verifies this works.

**Risk:** Low - same additive change pattern as let statements.

---

## 4. Demo and Validation Coverage

### 4.1 Demo

**Location:** `demos/phase31_guarded_sequence_index_demo.sifr`

The demo covers all three guard patterns:
1. `while i < len(text)` - string index narrowing
2. `for i in range(len(values))` - list index narrowing
3. `if len(values) == 0: return` - early return pattern

**Execution:**
```bash
cargo run -q -p sifr -- run demos/phase31_guarded_sequence_index_demo.sifr
```
**Status:** Passes (no assertion errors)

### 4.2 E2E Test

**Location:** `crates/sifr/tests/e2e/pass/phase31_guarded_sequence_index_narrowing.sifr`

**Execution:** Passes as part of e2e test suite.

### 4.3 Full Test Suite

- **E2E tests:** 397 pass
- **Unit tests:** 19 pass
- **HIR tests:** 5 pass (all guarded_index tests)
- **Codegen tests:** 10 pass (all index-related tests)

---

## 5. Issues Found

### 5.1 Clippy Issues

**1. Fixed in uncommitted changes:**
- `sequence_guards.rs:37` - wildcard match fixed to `SequenceGuard::MinLength { .. } => false`

**2. NEW CODE - Needs fixing:**
- `expr_render_helpers.rs:8` - `lower_proven_index_option_expr_for_ir` has unused `&self`:
  ```rust
  fn lower_proven_index_option_expr_for_ir(
      &self,  // <-- unused
      ...
  ```
  **Fix:** Remove `&self` since the function body doesn't use it.

**3. Pre-existing issues (not from this slice):**
- `stmt_support_emitter.rs:103` - `wrap_option_local_value_for_ir` has unused `&self`
- `expressions.rs:60` - `single_match_else` warning

### 5.2 Formatting Issues

Run `cargo fmt` to fix formatting issues in new code.

---

## 6. Conclusion

### Slice Completeness: HIGH

The implementation is functionally complete with:
- Correct HIR guard tracking
- Proper codegen for non-optional indexes
- Good test coverage
- Working demo and E2E tests
- Measured corpus impact (3 new passes)

### Issues to Fix Before Merge

1. **Clippy**: Remove unused `&self` from `expr_render_helpers.rs:8`
2. **Formatting**: Run `cargo fmt` to fix formatting issues in new code

### Recommendation

**APPROVE PENDING FIXES** - Once the clippy warning and formatting issues are resolved, this slice is ready for merge. The implementation is sound and well-tested.

---

## Appendix: Files Changed

```
 crates/sifr_codegen/src/expr_render_helpers.rs     |  37 ++-
 crates/sifr_codegen/src/lib.rs                     |   4 +-
 crates/sifr_codegen/src/lib_codegen_tests.rs       | 100 +++++-
 crates/sifr_codegen/src/lower_stmt.rs              | 167 +++++++++-
 crates/sifr_codegen/src/render.rs                   |   4 +-
 crates/sifr_codegen/src/stmt_support_emitter.rs    | 120 +++++++-
 crates/sifr_hir/src/lower/expressions.rs           |  23 +-
 crates/sifr_hir/src/lower/guarded_index.rs         | 148 +++++++++
 crates/sifr_hir/src/lower/mod.rs                   |   6 +
 crates/sifr_hir/src/lower/sequence_guards.rs       |  80 +++++
 crates/sifr_hir/src/lower/statements.rs            | 182 +++++++++++
 demos/phase31_guarded_sequence_index_demo.sifr     |  32 ++
 issues/phase31-ad-hoc-followup-milestones.md       |  13 +
 verification/...m31a_wave1_results.json             | 338 ++++++++++++++++
```
