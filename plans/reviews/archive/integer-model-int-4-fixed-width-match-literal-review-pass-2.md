

## Review: INT-4 Fixed-Width Match Literal Fitting (Final Pass)

---

### Summary

This slice extends the INT-4 fixed-width validation from variable assignments and module constants into the `lower_pattern` path for `Pattern::MatchValue` literal patterns. The pass-1 non-blocking gap (missing positive coverage) has been resolved.

---

### Findings

**No blocking issues.**

#### 1. Correctness (`statements.rs:781-791`)

The validation path is sound. After lowering the literal value expression, `validate_fixed_width_initializer` is called with the subject type as the target. Three outcomes are handled correctly:
- `Fits` → the in-range folded literal (`255`, `0`, etc.) is folded into the pattern, correct
- `Rejected` → diagnostic already emitted inline; `return None` suppresses the arm, correct
- `NotConst` → the original expression used (e.g., enum variant `Color.RED` is not const-evaluable so it passes through unchanged), correct

#### 2. Negative case (`expressions_tests.rs:266-284`, `fixed_width_match_literal_out_of_range.sifr`)

`case 256:` against `uint8` produces exactly one `SIFR-INT-0001` with message `"integer value 256 does not fit target type uint8; valid range is 0..=255"`. Primary range correctly points to `"256"`. No other diagnostics are emitted (the second `assert` guards this). E2E fail fixture uses `#expect-error: SIFR-INT-0001` which is clean.

#### 3. Positive case (`expressions_tests.rs:286-292`, `fixed_width_match_literal_fitting.sifr`)

`case 255:` against `uint8` lowers without error. The E2E pass fixture covers both boundary literals (`255`, `0`) and runtime assertions to verify the generated Rust code is correct end-to-end.

#### 4. `NotConst` is correct

`validate_fixed_width_initializer` returns `NotConst` when the value expression is not a const-integer (e.g., a module constant name resolved via `ctx.const_integer_values`, a binary expression with non-const operands, etc.). In all these cases, skipping range validation is correct — the value is validated at the point it was originally defined.

#### 5. No panic risk

`return None` from `lower_pattern` is the established mechanism for skipping arms (used by `Pattern::MatchClass` invalid-class-name path at `statements.rs:810`, `Pattern::MatchValue` invalid-attr path at `statements.rs:772`). The caller at `statements.rs:884-888` handles `None` by skipping the arm.

#### 6. Contract compliance

`internal_docs/integer_model.md` line 415-434: *"Matching a fixed-width subject with an out-of-range literal is a compile-time error."* — implemented. Diagnostic format matches the spec for range errors (value + type + valid range).

---

### Non-blocking observations

None — the pass-1 notes are all resolved.

---

### Validation results

All locally verified per user report:
- `cargo fmt --check` ✓
- `git diff --check` ✓  
- Unit tests: 2 tests pass (`test_fixed_width_match_literal_out_of_range_has_int_code`, `test_fixed_width_match_literal_in_range_lowers`) ✓
- E2E pass fixture ✓
- E2E fail fixture fails cleanly with only `SIFR-INT-0001` ✓

---

**Final verdict: satisfied**
