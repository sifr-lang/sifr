# Review Pass 2 — semantic-diagnostic-code-taxonomy-diag-11-expression-subscript-slices

## Branch
`codex/diag-11-raw-hir-expression-subscript-slices`

## Pass 1 Finding (Resolved)
- **Issue**: Unsupported slice receiver diagnostic used `sub.value.range()` instead of `sub.slice.range()`, pointing the error at the receiver instead of the slice expression itself.
- **Fix applied**: `expressions.rs` now passes `sub.slice.range()` to `expression_diagnostics::type_mismatch()` for the "cannot slice type '...'" diagnostic (line ~1928).
- **Verification**: Test `test_unsupported_slice_receiver_has_type_code` expects primary range to point at `"0:1"` via `range_for_after_anchor(source, "value[", "0:1")`, and test passes.

## Validation Results (All Pass)

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --check` | ✅ Pass |
| Unit test | `cargo test -p sifr_hir unsupported_slice_receiver -- --nocapture` | ✅ 1 passed |
| Compilation | `cargo check -p sifr_hir` | ✅ Pass |
| Clippy | `cargo clippy -p sifr_hir -- -D warnings` | ✅ No warnings |
| Guardrails | `python3 scripts/check_hir_maintainability_guardrails.py` | ✅ HIR maintainability guardrails: PASS |
| Whitespace | `git diff --check` | ✅ No issues |

## Additional Changes Observed (from `git diff HEAD`)
Diff also shows additional diagnostic migrations in the same file beyond the slice case:
- `dict unpacking (**) not supported` → `expression_diagnostics::type_mismatch` with `item.value.range()`
- `tuple too large for slicing index computation` → `expression_diagnostics::type_mismatch` with `sub.slice.range()`
- `tuple slice indices out of range` → `expression_diagnostics::type_mismatch` with `sub.slice.range()`
- `tuple slicing requires compile-time constant indices` → `expression_diagnostics::type_mismatch` with `sub.slice.range()`

All use appropriate ranges.

## Review Result
**APPROVED** — Pass 1 finding is resolved. No required fixes remain. All local validation checks pass.
