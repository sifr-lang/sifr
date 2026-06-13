# Review: INT-1 exact-int division diagnostic scaffold PR #1857

## Verdict

**APPROVED**

## Blocking Findings

None.

## Non-Blocking Findings

1. **Positive `//=` unit test missing.** `test_exact_int_division_by_nonzero_literal_still_lowers_as_int` covers `//` and `%` with non-zero literals but not `//=` / `%=` with a non-zero literal right-hand side. The augassign lowering rewrites `value //= 3` to `value = value // 3`, so the proven-nonzero path should keep lowering as `Type::Int`, but this is not directly tested. A single additional test — e.g., `value //= 3` in a let-binding context with a non-zero literal — would close the gap. Tracked as follow-up, not a requirement for this scaffold.

2. **`EXACT_INT_DIVISION_REQUIRES_HANDLING` constant name conflates division with modulo.** The short constant name reflects the original scaffold ticket but the diagnostic and message template also cover `%`. This is intentional per the integer model design (`SIFR-INT-0005` covers both `//` and `%`), and the mismatch only affects the internal constant name — not user-facing output. No action required for this scaffold.

## Validation Notes

The validation suite is sufficient for this scoped scaffold:

- **Lint/format**: `cargo fmt --check`, `check_hir_maintainability_guardrails.py` — both clean.
- **Diagnostic docs sync**: `check_diagnostic_docs_sync.py`, `check_diagnostic_schema_sync.py`, `check_diagnostic_code_coverage.py` — all clean, confirming `SIFR-INT-0005` is properly registered in codes.rs, docs/errors/, and internal_docs/diagnostic_codes.md.
- **Unit tests**: `cargo test -p sifr_hir exact_int_division` and `cargo test -p sifr_hir exact_int_mod_augassign` cover:
  - SIFR-INT-0005 fires for `//` with an unproven (variable) divisor.
  - SIFR-INT-0005 fires for `%=` with an unproven (variable) divisor.
  - SIFR-INT-0005 does NOT fire for `//` and `%` when the divisor is a syntactically non-zero integer literal (including negative literals and unary-negated literals).
- **E2E fail fixture**: `test_e2e_fail` covers the representative fail fixture `exact_int_division_requires_handling.sifr`.
- **E2E pass fixture guard**: `test_emit_pass_fixtures_do_not_include_unwrap_or_expect` ensures pass fixtures in the suite don't regress to using `.unwrap()` or `.expect()` — relevant because `exact_int_floor_mod_literals.sifr` and `error_builtin_classes.sifr` were adapted to use literal divisors.
- **Quick profile**: `scripts/run_all_tests.sh --profile quick` (61.85s) completed cleanly with signature `e1bf653aaa770517`.

No gaps in test coverage that are specific to this scaffold's scoped contract.

## Residual Risks

**Acceptable.** The implementation is a scoped scaffold: it gates user-code exact-int `//`, `%`, `//=`, and `%=` with an active diagnostic while the full `Result[int, DivisionError]` lowering and non-literal proven-nonzero analysis are not yet implemented. The following risks are intentional and bounded by the scaffold's scope:

1. **Stdlib exemption is intentionally broad.** The `is_stdlib_lowering()` / `allow_intrinsic_imports` exemption means all stdlib `.sifr` files are exempt from SIFR-INT-0005, regardless of whether their specific division/modulo divisors are actually proven non-zero. The PR notes this explicitly: "trusted stdlib lowering remains exempt until broader guard/proof tracking covers its internal non-zero loops." This is the correct call for a scaffold — full stdlib coverage with per-expression proof tracking is a later slice.

2. **`is_proven_nonzero_integer_expr` is syntactically limited.** Only integer literals (non-zero) and unary-negated non-zero literals are recognized as proven non-zero. Expressions like `LIMIT // 2` (derived from a module constant), `divisor - 3` (arithmetic on a variable), or `if condition: 5 else: 3` are not recognized and will correctly emit SIFR-INT-0005. The full proven-nonzero analysis (flow-sensitive, constant-propagating, branching to non-zero paths) is a later slice.

3. **Codegen does not yet emit `Result[int, DivisionError]` for blocked expressions.** When SIFR-INT-0005 fires, lowering returns `None` and the expression is rejected — it does not yet produce a `Result[int, DivisionError]` that propagates to codegen. The PR scope is intentionally limited to the diagnostic scaffold; full result-type lowering is a later slice.

4. **Pass fixtures adapted for this scaffold only.** `multiple_return.sifr` was changed from `(a // b, a % b)` to `(a // 5, a % 5)` to avoid triggering SIFR-INT-0005. This is correct for the scaffold. When later slices add proper `Result[int, DivisionError]` lowering, these fixtures can be reverted to use `Result`-returning patterns that the diagnostic would still correctly suppress (e.g., after flow-sensitive proven-nonzero analysis covers function parameters).

The scaffold is a clean, well-scoped addition: it adds the diagnostic, wires it into both binary and augmented assignment lowering, exempts stdlib via the existing `allow_intrinsic_imports` mechanism, adapts the two affected pass fixtures, and adds a representative fail fixture. It is ready to merge.
