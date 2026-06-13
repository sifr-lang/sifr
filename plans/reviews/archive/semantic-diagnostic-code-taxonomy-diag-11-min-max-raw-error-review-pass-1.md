# Review: semantic diagnostic taxonomy — diag_11 min/max raw HIR diagnostic migration

## Files reviewed
- `crates/sifr_hir/src/lower/min_max_validation.rs`
- `crates/sifr_hir/src/lower/expressions.rs` (diff)
- `crates/sifr_hir/src/lower/expressions_tests.rs` (diff)
- `scripts/check_diagnostic_transport_cleanup.py` (diff)

## Verdict: APPROVED

## Checklist

### Raw diagnostic elimination
- [x] `min_max_validation.rs` contains **zero** `ctx.error(String)` calls.
- [x] Both error sites in `validate_two_arg_min_max_operands` use `ctx.error_with_code_at(DiagnosticCode::TYPE_MISMATCH, …)` with explicit `TextRange`.

### Diagnostic code correctness
- [x] Both optional-operand and incompatible-operand paths emit `DiagnosticCode::TYPE_MISMATCH`.
- [x] Optional-operand error ranges the operand that is `None`-containing (left or right, per `union_contains_none`).
- [x] Incompatible-operand error ranges `right_range` (second operand), matching the stated intent ("optional operands range the optional operand; incompatible min/max pairs range the second incompatible operand").

### Range transport
- [x] `validate_variadic_min_max_operands` accepts `&[Expr]` (original AST args) as `operand_ranges`, avoiding any extra local range vectors in `expressions.rs`.
- [x] Call sites in `expressions.rs` pass `&call.arguments.args` directly — clean, minimal diff.
- [x] `Ranged::range` is called inside `min_max_validation.rs`, not at the call site.

### expressions.rs maintainability
- [x] Diff is exactly two lines: both `validate_variadic_min_max_operands` call sites now pass `&call.arguments.args` as a third argument.
- [x] No new local variables, no new borrow chains, no growth in line count.

### Test coverage
- [x] `test_max_two_arg_rejects_optional_operand` upgraded to assert `DiagnosticCode::TYPE_MISMATCH` and `primary_range == Some(range_for(source, "d[k]"))`.
- [x] New `test_min_max_incompatible_operands_have_type_codes` asserts `DiagnosticCode::TYPE_MISMATCH`, exact message, and `primary_range == Some(range_for(source, "\"x\""))`.

### Script update
- [x] `min_max_validation.rs` added to `RAW_HIR_ERROR_FREE_FILES` in `check_diagnostic_transport_cleanup.py`.

### No fallback paths
- [x] `validate_two_arg_min_max_operands` returns `false` immediately after emitting an error; no fallthrough to further validation.
- [x] `validate_variadic_min_max_operands` returns `false` as soon as any `validate_two_arg_min_max_operands` call returns `false`.

### Local validation (confirmed in scope)
- `python3 scripts/check_hir_maintainability_guardrails.py` — green
- `python3 scripts/check_diagnostic_transport_cleanup.py` — green (no `ctx.error` found, script passes)
- `rg "ctx\.error\(" min_max_validation.rs` — no output
- `cargo test -p sifr_hir test_max_two_arg_rejects_optional_operand` — passes
- `cargo test -p sifr_hir test_min_max_incompatible_operands_have_type_codes` — passes
- `cargo clippy -p sifr_hir -- -D warnings` — clean

## Notes

The `operand_ranges: &[Expr]` parameter naming in `min_max_validation.rs` is slightly imprecise (it's AST expressions, not ranges), but since `Expr: Ranged`, calling `.range()` on each element is self-documenting at the call site and the intent is unambiguous. No change required.

The change from `operands.windows(2)` to index-based iteration in `validate_variadic_min_max_operands` is necessary to map each operand pair to its corresponding AST expression range — `windows(2)` would lose the index information needed to index into `operand_ranges`. Correct and clean.
