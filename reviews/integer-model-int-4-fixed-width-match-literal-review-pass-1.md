

All validation passed. Here's my review:

---

## Review: INT-4 Fixed-Width Match Literal Fitting

### Implementation Summary

**Code change** (`statements.rs:778-791`): In `lower_pattern` for `Pattern::MatchValue`, after lowering the value expression, the code now calls `validate_fixed_width_initializer(ctx, subject_ty, &expr, val_pat.value.range())` and handles all three outcomes:
- `Fits` → use the folded value (const-evaluable, in range)
- `Rejected` → return `None` (out-of-range, diagnostic already emitted)
- `NotConst` → use the original expression (e.g., enum variant)

### Findings

**No blocking issues.**

1. **Correctness** (`statements.rs:781-791`): The validation is applied to literal patterns against fixed-width subjects. When `256` is used against `uint8`, `validate_fixed_width_initializer` correctly emits `SIFR-INT-0001` with message "integer value 256 does not fit target type uint8; valid range is 0..=255". The `Rejected` branch returns `None`, which propagates up and suppresses that arm while the other arms (e.g., `case _:`) continue processing.

2. **Recursive coverage**: `MatchOr` patterns (`statements.rs:794-799`) recursively call `lower_pattern` on each sub-pattern, so each alternative is independently validated against the subject type. `MatchClass` (`statements.rs:801-850`) passes the field type (not the subject type) to `lower_pattern`, which is correct.

3. **No panic risk**: `return None` from `lower_pattern` is safe — the match lowering handles `None` by skipping the arm (`statements.rs:884-888`), and the diagnostic is already emitted before returning.

4. **Test coverage**:
   - Unit test: `test_fixed_width_match_literal_out_of_range_has_int_code` in `expressions_tests.rs:266-285` — verifies exact diagnostic code, message, and primary range
   - E2E fail fixture: `fixed_width_match_literal_out_of_range.sifr` — verifies the failure propagates to exit code

### Non-blocking observations

- **Test for positive case missing** (not blocking): The validation matrix calls for both positive cases (in-range literal arms) and negative cases (out-of-range). No positive test exists yet, but this is a pre-existing gap — not a regression from this PR.

- **E2E harness note**: The reviewer noted the fail harness runs the full corpus rather than honoring `SIFR_E2E_FIXTURE_MANIFEST`, and prints an unrelated CFG panic from the broader corpus. This is a pre-existing harness behavior, not caused by this change.

### Contract compliance with `internal_docs/integer_model.md`

Section "Pattern Matching, Enums, and Containers" (lines 415-434):
> Matching a fixed-width subject with an out-of-range literal is a compile-time error.

The implementation correctly rejects `case 256:` against `uint8` with `SIFR-INT-0001`. The diagnostic includes the value, target type, and valid range — matching the "fixed-width narrowing out of range" diagnostic spec (lines 440-443).

### Validation results

- `cargo fmt --check`: passed
- `git diff --check`: passed  
- `cargo test -p sifr_hir test_fixed_width_match_literal_out_of_range_has_int_code`: passed
- `scripts/run_all_tests.sh --profile quick`: all lanes green (24 pass, 0 fail)

---

**Final verdict: satisfied**
