# Phase 31 m31a Slice 5 Review: Reverse-Range Recurrence Narrowing

**Status:** Complete
**Reviewed:** 2026-03-13
**Branch:** `codex/phase31-m31a-slice5-recurrence-flow`

## Executive Summary

Slice 5 successfully implements reverse-range recurrence narrowing over sized local constructions, addressing a critical gap in the optional narrowing system. The implementation adds HIR-level sequence shape tracking, extends guarded-index narrowing, and fixes codegen issues with comprehensions, dynamic subscript writes, recurrence assignment ordering, and negative-step range iterators.

**Key Result:** Targeted LeetCode problem 1143 (Longest Common Subsequence) now passes, demonstrating the feature works end-to-end.

---

## Scoped Goals Assessment

### 1. Reverse Range Recurrence Narrowing over Sized Local Constructions

**Status:** ✅ COMPLETE

The implementation adds sequence shape fact tracking in `crates/sifr_hir/src/lower/sequence_shapes.rs`:

- `SequenceShapeFact::SizedByAnchor` tracks lists sized from `range(len(anchor) + extra)`
- `SequenceShapeFact::MatrixSizedByAnchors` tracks 2D matrices with separate outer/inner anchors

These facts are recorded in `statements.rs:1027-1035` when let statements initialize comprehensions, and are queried in `guarded_index.rs:81-86` when checking if an index expression is safe.

The guarded index logic in `guarded_index.rs:125-143` now handles affine `+ literal` offsets (not just `+ 0`):
```rust
Operator::Add => ctx.has_index_var_offset_guard(
    anchor_sequence,
    index_name.id.as_str(),
    offset.saturating_sub(extra_len),
),
```

This enables proofs like:
- `suffix[i + 1]` is safe when `suffix` has length `len(text) + 1` and loop runs `i in range(len(text) - 1, -1, -1)`
- `dp[i + 1][j + 1]` is safe for matrix constructions sized by anchors

**Test coverage:** Unit tests in `guarded_index.rs:337-360` verify:
- `test_reverse_range_suffix_recurrence_reveals_int` - basic +1 offset
- `test_matrix_recurrence_offsets_reveal_int` - nested matrix access

### 2. Structured Codegen for Comprehension-Backed Locals

**Status:** ✅ COMPLETE

The implementation in `stmt_support_emitter.rs:342-474` adds structured lowering for list/dict/set comprehensions used as let initializers:

- `try_lower_comprehension_expr_for_ir()` handles `ListComp`, `DictComp`, and `SetComp`
- Creates mutable local `__sifr_list_comp` initialized to empty collection
- Emits nested for loops with optional filters
- Returns block expression with result identifier

This replaces the previous fallback to `LoweringError` for comprehension-backed locals.

**Test coverage:** Codegen test in `lib_codegen_tests.rs:276` verifies comprehension local initializers lower correctly.

### 3. Dynamic Subscript Writes

**Status:** ✅ COMPLETE

The implementation in `stmt_support_emitter.rs:651-735` adds:

- `lower_subscript_assign_stmt_for_ir()` handles `SubscriptAssign` statements
- For lists: wraps in block with temp value binding
- For dicts: handles borrow-safe key cloning
- `try_lower_structured_subscript_assign_stmt()` entry point
- `try_lower_structured_nested_subscript_assign_stmt()` for matrix writes

Critical fix at lines 667-678:
```rust
RustStmt::Let {
    mutable: false,
    name: "__assign_value".to_string(),
    ty: None,
    value: lowered_value,
},
crate::build_list_subscript_assign_stmt(...),
```

This evaluates the RHS value before taking the mutable borrow, preventing ownership conflicts in recurrence assignments like `suffix[i] = suffix[i + 1] + 1`.

### 4. Borrow-Safe Recurrence Assignment Ordering

**Status:** ✅ COMPLETE

The temp binding pattern described above ensures RHS recurrence reads (which may borrow the same list) are fully evaluated before taking mutable element borrows. This prevents the "evaluating RHS after mutable borrow" codegen failure mentioned in the root cause analysis.

### 5. Negative-Step Range Iterator Lowering

**Status:** ✅ COMPLETE

The implementation in `stmt_support_emitter.rs:252-309`:

- `negative_range_step_magnitude()` extracts negative step value (handles both `-1` literal and unary `-`)
- `try_lower_range_iter_expr_for_ir()` converts reverse Python ranges to Rust:
  - `range(end, start, -1)` → `(end+1..start+1).rev()`
  - For larger steps: `.rev().step_by(n)` instead of invalid `step_by(-n as usize)`

Key transformation at lines 282-297:
```rust
let reversed_iter = RustExpr::MethodCall {
    receiver: Box::new(RustExpr::Range {
        start: Box::new(RustExpr::BinOp {
            left: Box::new(lowered_end),
            op: "+".to_string(),
            right: Box::new(Self::int_i64_literal_expr(1)),
        }),
        end: Box::new(RustExpr::BinOp {
            left: Box::new(lowered_start),
            op: "+".to_string(),
            right: Box::new(Self::int_i64_literal_expr(1)),
        }),
    }),
    method: "rev".to_string(),
    args: vec![],
};
```

This correctly handles the inclusive-exclusive range semantics difference between Python and Rust.

---

## Verification Results

### LeetCode Corpus State

| Metric | Count |
|--------|-------|
| Total problems | 50 |
| PASS | 13 |
| CHECK_ERROR | 35 |
| TIMEOUT | 2 |
| RUNTIME_ERROR | 0 |
| COMPILE_ERROR | 0 |

### New Pass (Slice 5 Delta)

- **1143 Longest Common Subsequence** - moved from CHECK_ERROR to PASS

This confirms the targeted case now works end-to-end.

### Confirmed Outside Scope

- **0053 Maximum Subarray** - still blocked by unguarded parameter head access (`nums[0]`)
- **0746 Min Cost Climbing Stairs** - still blocked by unguarded parameter head access (`cost[0]`)
- **0322 Coin Change** - still blocked by subtractive/value-dependent recurrence indexing

These align with the documented non-goals in the slice specification.

---

## Implementation Quality

### Architecture

The implementation follows established HIR lowering patterns:
- New `sequence_shapes.rs` module with clear fact types
- Extension of existing `sequence_guards.rs` guard infrastructure
- Codegen in `stmt_support_emitter.rs` following existing structured lowering patterns
- Unit tests colocated in `guarded_index.rs` with clear coverage of edge cases

### Edge Case Handling

**Correctly handled:**
- Empty sequences (sized with `range(0)`) - guard check handles `min_len > 0`
- Literal index offsets beyond extra_len - uses `saturating_sub` to avoid underflow
- Nested matrix recurrence `dp[i+1][j+1]` - requires both outer and inner anchor proofs
- Tuple-unpacked comprehensions - correctly rejected (no tuple unpacking in generated code)
- Sliding window left pointer after branch merge - correctly stays optional

**Explicitly out of scope:**
- Unguarded parameter indexing (plain `list[int]` inputs)
- Subtractive offsets (`dp[i - c]`)
- Arbitrary integer sizing (`[0 for i in some_int_var]`)

### Test Coverage

- **Unit tests:** 3 new tests in `guarded_index.rs`
- **E2E tests:** `phase31_reverse_range_recurrence_narrowing.sifr`
- **Demo:** `phase31_reverse_range_recurrence_demo.sifr`
- **Codegen tests:** 1 new test in `lib_codegen_tests.rs`

All execute successfully:
```
cargo run -q -p sifr -- run demos/phase31_reverse_range_recurrence_demo.sifr
# No output = success
```

---

## Findings

### Strengths

1. **Root cause fix rather than workaround** - The implementation addresses the actual compiler gaps (HIR proof + codegen) rather than patching individual LeetCode failures.

2. **Clean separation of concerns** - Sequence shape facts are tracked separately from guards, allowing orthogonal composition.

3. **Proper Rust semantics** - Negative-step range lowering correctly handles Rust's exclusive upper bound vs Python's exclusive lower bound difference.

4. **Borrow safety** - Temp binding pattern ensures recurrence RHS evaluation precedes mutable borrow.

5. **Well-documented scope** - Non-goals are clearly articulated and verified against remaining failures.

### Potential Concerns

1. **Limited to + literal offsets** - The implementation only handles `index + constant` (lines 134-142 in guarded_index.rs), not arbitrary expressions. This is correct for the stated scope but may need extension for more complex recurrences.

2. **Matrix anchor tracking** - The `matrix_sequence_fact` only tracks two levels of nesting. Deeper nesting would require extension.

3. **No filter support in reverse ranges** - The implementation doesn't track guards from `if` conditions inside reverse for loops. This is documented as out of scope.

---

## Conclusion

Slice 5 is **APPROVED** for completion. The implementation:

- ✅ Achieves all scoped goals
- ✅ Fixes the targeted root cause
- ✅ Enables 1143 Longest Common Subsequence to pass
- ✅ Correctly documents and respects out-of-scope boundaries
- ✅ Has appropriate test coverage
- ✅ Follows established architectural patterns

The remaining watched failures (0053, 0746, 0322) are cleanly outside the slice scope and belong to future m31a work on unguarded parameter indexing and subtractive recurrences.

---

## Artifacts

- **Commit:** `eb6060a6` - Fix reverse-range recurrence narrowing
- **Demo:** `demos/phase31_reverse_range_recurrence_demo.sifr`
- **E2E:** `crates/sifr/tests/e2e/pass/phase31_reverse_range_recurrence_narrowing.sifr`
- **Execution doc:** `issues/phase31-m31a-reverse-range-recurrence-execution.md`
