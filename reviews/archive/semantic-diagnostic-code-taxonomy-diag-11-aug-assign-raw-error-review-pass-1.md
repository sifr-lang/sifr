# Review: semantic-diagnostic-code-taxonomy diag-11 aug-assign raw HIR diagnostic migration

## Scope

- `crates/sifr_hir/src/lower/aug_assign_lowering.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`
- `scripts/check_diagnostic_transport_cleanup.py`

## Disposition

Approved.

All raw `ctx.error(String)` emissions in `aug_assign_lowering.rs` were eliminated and replaced with structured diagnostics. `MatMult` (`@=`) now emits `DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR`; non-simple augmented-assignment receiver shapes emit `DiagnosticCode::TYPE_MISMATCH`; binary operator diagnostics from `type_check_binary_op` continue forwarding their existing structured codes.

Range transport is appropriate for this slice: `@=` ranges the augmented-assignment target because the AST operator token is not separately ranged here, and target-shape diagnostics range the non-simple receiver expression such as `make_box()` or `xs + ys`.

The helper functions are appropriately small and avoid repeated raw transport. The new tests cover matrix augmented assignment and five complex attribute/subscript/nested-subscript receiver forms. Existing binary-operator, ownership, nonlocal, and undefined-name diagnostic helper paths are preserved unchanged.

`aug_assign_lowering.rs` is added to `RAW_HIR_ERROR_FREE_FILES`, and `rg -n "ctx\\.error\\(" crates/sifr_hir/src/lower/aug_assign_lowering.rs` returns no output.

Validation reviewed:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_diagnostic_transport_cleanup.py`
- focused HIR tests for matrix and complex-target augmented assignment diagnostics
- `cargo check -p sifr_hir`
- `cargo test -p sifr_hir -- --skip test_e2e_pass`
- `cargo clippy -p sifr_hir -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=56.32s`; group-skew advisory emitted)

Reviewer verdict: satisfied; no required fixes.
