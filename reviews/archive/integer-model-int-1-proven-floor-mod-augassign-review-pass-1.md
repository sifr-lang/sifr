

# PR #1856 Review: Lower proven exact-int floor modulo augassign

**Author:** yaseralnajjar  
**Branch:** int-1-sifrint-proven-floor-mod-augassign  
**Base:** main  
**State:** OPEN

---

## Summary

Extends the PR #1855 known-nonzero floor division/modulo lowering to cover augmented assignment forms (`//=`, `%=`) when the RHS is a syntactically non-zero integer literal. Adds e2e coverage and updates the INT-1 tracker residual.

---

## Changes Reviewed

### 1. `expr_render_helpers.rs` — new augassign rewrite branch (lines 572–589)

A new match arm in the `AugAssign` handler intercepts `//=` and `%=` (via `is_sifr_int_checked_floor_op`) where:
- RHS passes `is_proven_nonzero_integer_expr` (literal `!= 0`, or `SifrInt::from_i64` wrapping same)
- target is a registered or forced SifrInt local

It rewrites to a plain `Assign` using `sifr_int_known_nonzero_floor_expr`, which dispatches to `floor_div_known_nonzero` / `floor_mod_known_nonzero`.

**Correctness:** The guard conditions are precise. The new arm is placed before the existing arithmetic-op augassign arm (which handles `+`, `-`, `*`), matching the pattern established in PR #1855 for binary expressions. No ordering hazard exists since `is_sifr_int_checked_floor_op` and `is_sifr_int_arithmetic_op` are disjoint sets.

**Generated-code safety:** The rewrite produces `augmented = augmented.floor_div_known_nonzero(&SifrInt::from_i64(3))` — a value call (`SifrInt::from_i64(3)`) not a borrow, consistent with how the same pattern is handled in `coerce_expr_to_sifr_int_comparison_operand` for binary forms. No runtime panic path. No ownership violations introduced.

**Ownership/value semantics:** The reborrow implicit in `&SifrInt::from_i64(3)` is explicit in the generated Rust. The `SifrInt` methods take `&SifrInt` — consistent with the immutable value semantics of `SifrInt`.

### 2. Unit test (lines 2093–2141)

`rewrites_sifr_int_floor_mod_augassign_by_nonzero_literal_to_assignment` validates both `/=` and `%=` ops produce the correct method name, receiver identity, and `SifrInt::from_i64` argument structure. Pattern-matches the full structure including the `Ref` wrapping of the `FnCall`.

### 3. E2E fixture `exact_int_floor_mod_literals.sifr`

Adds augmented-assign coverage after an oversized exact-int promotion (`BIG_LIMIT + 2`):

```sifr
augmented: int = BIG_LIMIT + 2
augmented //= 3
augmented %= 5
assert str(augmented) == "4"
```

Verified by `emit` that generated Rust is:
```rust
let mut augmented: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(2);
augmented = augmented.floor_div_known_nonzero(&SifrInt::from_i64(3));
augmented = augmented.floor_mod_known_nonzero(&SifrInt::from_i64(5));
```

### 4. Tracker update

INT-1 residual updated accurately: item 6 is marked `[x]` with PR #1856, item 7 correctly remains `[ ]` for the remaining HIR `Result[int, DivisionError]` / `SIFR-INT-0005` work.

---

## Blocking Findings

**None.** The implementation is correct and aligned with the PR #1855 design.

---

## Non-Blocking Notes

1. **Test divergence from prior test at line 2078:** The existing `rewrites_sifr_int_arithmetic_op_augassign` test at line 2078 tests value `2` (non-zero), while the new test tests value `3`. Both are non-zero so the behavioral difference does not affect correctness, but using `3` in the new test (vs `2` in the arithmetic test) means the two tests are not sampling the same value. This is intentional in this PR but worth noting if future regression tests want to cover a wider range of non-zero values.

2. **Scope of `is_proven_nonzero_integer_expr`:** The predicate correctly covers only the syntactic cases that can be proven at compile time: integer literals (any size), unary negation of same, parenthesized variants, and `SifrInt::from_i64` wrapping same. It does not attempt to prove non-zero for arbitrary expressions (e.g. `x + 1`). This matches the stated design intent and leaves the remaining unproven cases for the HIR `Result[int, DivisionError]` / `SIFR-INT-0005` work item.

3. **Codegen emit confirms correct mutability:** The emitted `let mut augmented` correctly marks the variable mutable before the re-assignment, which is required Rust syntax. The HIR-to-codegen path correctly propagates mutability from the augmented assignment back to the `let` binding.

4. **No e2e signature drift:** The PR reuses the same fixture manifest entry used by PR #1855 (`exact_int_floor_mod_literals.sifr`), so the e2e signature `554367484e3fa236` is preserved across both PRs.

5. **Validation summary matches actual results:** Author's reported validation results were confirmed locally:
   - Unit test passes
   - E2E fixture runs and produces correct output (`4` for `augmented` final value)
   - `cargo fmt --check` clean

---

## Verdict

**Approve.** The implementation correctly extends the PR #1855 known-nonzero proven-divisor lowering to augmented assignment forms, with appropriate test coverage, correct Rust codegen, and an accurate tracker update. No safety, correctness, or design-alignment issues identified.
