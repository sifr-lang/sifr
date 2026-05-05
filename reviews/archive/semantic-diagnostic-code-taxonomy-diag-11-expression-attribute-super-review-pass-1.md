# Review: Semantic Diagnostic Code Taxonomy — DIAG-11 Expression Attribute/Super

**File**: `crates/sifr_hir/src/lower/expressions.rs`
**File**: `crates/sifr_hir/src/lower/expressions_tests.rs`
**Slice**: Migrate expression attribute/super raw HIR diagnostics to structured diagnostics only
**Pass**: 1 (first review)

## Changes Summary

### `expressions.rs`

Four raw HIR diagnostic sites migrated to structured diagnostics:

1. **Enum missing attribute** (expressions.rs:2041-2045): `ctx.error(...)` → `ctx.error_with_code_at(DiagnosticCode::CLASS_MISSING_MEMBER, ..., attr.attr.range())`
2. **Unsupported attribute-as-expression** (expressions.rs:2052-2058): `ctx.error(...)` → `expression_diagnostics::unsupported_form(ctx, &format!(...), attr.range())` — already using the shared helper that wraps `TYPE_UNSUPPORTED_EXPRESSION_FORM`
3. **super() outside class/parent** (expressions.rs:2088-2092): `ctx.error(...)` → `ctx.error_with_code_at(DiagnosticCode::CLASS_INVALID_BASE, ..., attr.value.range())`
4. **Missing class/static method** (expressions.rs:2120-2125): `ctx.error(...)` → `ctx.error_with_code_at(DiagnosticCode::CLASS_MISSING_MEMBER, ..., attr.attr.range())`

### `expressions_tests.rs`

Four new tests added (immediately after `test_missing_field_has_class_code`):

1. `test_enum_missing_attribute_has_class_code` — verifies enum field access with `CLASS_MISSING_MEMBER` + correct `primary_range`
2. `test_unsupported_attribute_expression_has_type_code` — verifies attribute access on non-class type with `TYPE_UNSUPPORTED_EXPRESSION_FORM` + correct `primary_range`
3. `test_super_outside_parent_has_class_code` — verifies `super()` outside class context with `CLASS_INVALID_BASE` + correct `primary_range`
4. `test_missing_class_static_method_has_class_code` — verifies class/static method lookup failure with `CLASS_MISSING_MEMBER` + correct `primary_range`

## Review Assessment

**No issues found.**

### Diagnostic code taxonomy

| Diagnostic | Code | Used for | Correct? |
|---|---|---|---|
| Enum field not found | `CLASS_MISSING_MEMBER` (SIFR-CLASS-0004) | Field/method access on enum | ✓ |
| Unsupported attribute-as-expression | `TYPE_UNSUPPORTED_EXPRESSION_FORM` (SIFR-TYPE-0003) | Non-class attribute access used as expression | ✓ |
| super() outside class with parent | `CLASS_INVALID_BASE` (SIFR-CLASS-0005) | super() with no valid parent class | ✓ |
| Missing class/static method | `CLASS_MISSING_MEMBER` (SIFR-CLASS-0004) | Class method lookup failure | ✓ |

All four codes are pre-existing, registered constants in `crates/sifr_diagnostics/src/codes.rs`. No new codes introduced.

### Primary ranges

- `attr.attr.range()` used for attribute-name targets (correct — spans just the identifier, not the dot)
- `attr.value.range()` used for `super()` (correct — spans the entire `super()` expression)
- `attr.range()` used for the full attribute expression in the unsupported case (correct)
- `range_for_after(source, "prefix", "needle")` used in tests for consistent, verified range extraction (correct pattern already established in this file)

### Architectural soundness

- `expression_diagnostics::unsupported_form` is the canonical helper for `TYPE_UNSUPPORTED_EXPRESSION_FORM` — its reuse here is consistent with prior DIAG-11 slices (expressions/operators).
- All four sites now use `error_with_code_at` or equivalent structured helpers — no raw `ctx.error` fallback paths remain for these cases.
- No fallback compatibility paths, no dummy placeholder continuations, no unrelated refactors.

### Validation results (pre-confirmed)

All four tests pass:
- `cargo test -p sifr_hir enum_missing_attribute -- --nocapture` — ok
- `cargo test -p sifr_hir unsupported_attribute_expression -- --nocapture` — ok
- `cargo test -p sifr_hir super_outside_parent -- --nocapture` — ok
- `cargo test -p sifr_hir missing_class_static_method -- --nocapture` — ok

Plus full suite checks: `cargo fmt`, `cargo check -p sifr_hir`, `cargo clippy -p sifr_hir -- -D warnings`, `check_hir_maintainability_guardrails.py`, `git diff --check` — all clean.

## Conclusion

**reviewer is satisfied.**
