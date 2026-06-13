# Review: milestone_diag_11 — subscript_type raw HIR diagnostic migration

**File**: `crates/sifr_hir/src/lower/subscript_type.rs`
**Reviewer**: Pass-1
**Status**: SATISFACTORY

---

## Phase Fit

All three diagnostics are type-level errors and correctly use `TYPE_MISMATCH` (`SIFR-TYPE-0002`):

- `"tuple too large for indexing"` — tuple arity exceeds what `usize` can represent; a type-system constraint.
- `"tuple index out of range"` — literal index does not match any element position; a type-index mismatch.
- `"cannot index type 'X' with 'Y'"` — receiver or index type does not support `__getitem__`; classic type mismatch.

No `INDEX_OUT_OF_BOUNDS`-equivalent exists in the Sifr diagnostic code registry, and `TYPE_MISMATCH` is the appropriate stand-in for pre-production code. This is acceptable given project stage.

---

## Primary Range

| Diagnostic | Range used | Test expects | Verdict |
|---|---|---|---|
| tuple too large | `sub.slice.range()` | not tested directly | Correct — slice is the literal |
| tuple index out of range | `sub.slice.range()` | `range_for(source, "2")` | Correct — `slice` = `2` in `pair[2]` |
| cannot index | `sub.range()` | `range_for_after_anchor(source, "bad: int = ", "value[0]")` | Correct — full subscript expression |

The `ExprSubscript` struct has fields `value` (the receiver, e.g. `pair`), `slice` (the index, e.g. `2`), and `range` (the full expression `pair[2]`). Range selection is semantically correct for each error category.

---

## Raw `ctx.error(...)` Elimination

All three call sites converted from `ctx.error(String)` to `ctx.error_with_code_at(DiagnosticCode::TYPE_MISMATCH, String, range)`. No raw `ctx.error(...)` remains. File is listed in `RAW_HIR_ERROR_FREE_FILES` in `check_diagnostic_transport_cleanup.py`.

---

## Tests

Two new tests added in `expressions_tests.rs`:

- `test_tuple_index_out_of_range_has_type_code` — verifies `TYPE_MISMATCH` code + `"tuple index out of range"` message + `primary_range == range_for(source, "2")` (the literal index). Correct.
- `test_invalid_subscript_receiver_has_type_code` — verifies `TYPE_MISMATCH` code + `"cannot index type 'int' with 'int'"` message + `primary_range == range_for_after_anchor(source, "bad: int = ", "value[0]")` (the full subscript). Correct.

Both tests are targeted and assert the correct code/range pairing. Missing: a test for `"tuple too large for indexing"` primary range, but this is a narrow edge case (tuple with >2^63 elements) and the range logic is identical to the out-of-range case.

---

## Guardrail

- `subscript_type.rs` added to `RAW_HIR_ERROR_FREE_FILES`.
- `python3 scripts/check_diagnostic_transport_cleanup.py` passes.
- `check_hir_maintaintainability_guardrails.py` passes.
- `cargo clippy -p sifr_hir -- -D warnings` passes.

---

## Verdict

**Satisfied.** The migration is correct, phase-appropriate, and guardrail-compliant. The three diagnostics are semantically typed as `TYPE_MISMATCH` with primary ranges pointing at the relevant slice or expression. Tests correctly assert code and range. No raw `ctx.error(...)` remains.

No second pass required.
