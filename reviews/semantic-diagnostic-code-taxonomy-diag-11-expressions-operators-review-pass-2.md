# Review: DIAG-11 expressions/operators — raw HIR diagnostic migration (pass 2, post-refactor)

## Scope
Expression form and operator chunk after the refactor that moved `lower_binop`, `lower_unaryop`, `lower_compare` into `expression_operators.rs`. Files: `expression_diagnostics.rs`, `expression_operators.rs`, `expressions.rs`, `expressions_tests.rs`, `codes.rs`, generated docs, `unsupported_yield_expression.sifr`.

---

## Findings (ordered by severity)

### No required fixes remain. All checks pass.

---

## Detailed validation

### 1. Refactor integrity — `lower_binop`, `lower_unaryop`, `lower_compare`

**Before refactor**: these three functions lived in `expressions.rs` alongside the expression-kind dispatch in `lower_expr`.

**After refactor**:
- `expressions.rs:27` imports from `expression_operators`: `use super::expression_operators::{lower_binop, lower_compare, lower_unaryop};`
- `expressions.rs:81-83` dispatches: `Expr::BinOp(binop) => lower_binop(...)`, `Expr::UnaryOp(unary) => lower_unaryop(...)`, `Expr::Compare(cmp) => lower_compare(...)`
- All three functions are defined `pub(super)` in `expression_operators.rs` (lines 29, 91, 117) — visible only within the `lower` module, matching the original `expressions.rs` visibility
- No function signatures changed; all call sites in `expressions.rs` remain valid

**Conclusion**: refactor is a pure mechanical move with no semantic change.

---

### 2. `TYPE_UNSUPPORTED_EXPRESSION_FORM` / SIFR-TYPE-0012 — taxonomy and registry

**Code definition** (`codes.rs:45`):
```rust
pub const TYPE_UNSUPPORTED_EXPRESSION_FORM: Self = Self::new("SIFR-TYPE-0012", Severity::Error);
```
Correctly declared as Error severity in TYPE family.

**Registry entry** (`codes.rs:701-711`):
- `owner_module: "sifr_hir::lower::expressions"` — matches the lowering module
- `message_template: "unsupported expression form: {form}"`
- `declared_args: [arg!("form")]` — message+json format
- `dedupe_args: ["form"]`
- `representative_fixture_path: "crates/sifr/tests/e2e/fail/unsupported_yield_expression.sifr"`
- State: Active

**Emit site** (`expressions.rs:100-106`):
```rust
_ => {
    expression_diagnostics::unsupported_form(
        ctx,
        "unsupported expression type",
        expr.range(),
    );
    None
}
```
Static string `"unsupported expression type"` is passed as the `form` parameter. The diagnostic is emitted at `expr.range()`, which for yield expressions covers the full `yield 1` span. The fallback arm correctly handles any expression kind not yet lowered.

**Diagnostic helper** (`expression_diagnostics.rs:6-12`):
```rust
pub(super) fn unsupported_form(ctx: &mut LowerCtx, form: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        format!("unsupported expression form: {form}"),
        range,
    );
}
```
Uses `ctx.error_with_code_at` (structured, code-bearing) — no fallback behavior.

---

### 3. Reuse of `TYPE_UNSUPPORTED_OPERATOR` (SIFR-TYPE-0005)

`TYPE_UNSUPPORTED_OPERATOR` is correctly reused for:

| Operator | Location | Message |
|---|---|---|
| Matrix multiplication (`@`) | `expression_operators.rs:53-56` via `matrix_multiplication()` | `"matrix multiplication operator (@) is not supported"` |
| `in` on non-collection | `expression_operators.rs:128-146` via `unsupported_operator()` | `"unsupported operator in for {type}"` |
| `not in` on non-collection | `expression_operators.rs:159-176` via `unsupported_operator()` | `"unsupported operator not in for {type}"` |
| Unsupported comparison op | `expression_operators.rs:207-213` via `unsupported_operator()` | `"unsupported operator comparison for unsupported comparison operator"` |

All reuse `TYPE_UNSUPPORTED_OPERATOR` (SIFR-TYPE-0005) via `expression_diagnostics::unsupported_operator` — appropriate because all are operator/operand compatibility failures in the TYPE family.

**Diagnostic helper** (`expression_diagnostics.rs:14-25`):
```rust
pub(super) fn unsupported_operator(
    ctx: &mut LowerCtx,
    operator: &str,
    operand_types: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
        format!("unsupported operator {operator} for {operand_types}"),
        range,
    );
}
```

**Matrix multiplication helper** (`expression_diagnostics.rs:27-33`):
```rust
pub(super) fn matrix_multiplication(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
        "matrix multiplication operator (@) is not supported".to_string(),
        range,
    );
}
```

---

### 4. Primary ranges

| Diagnostic | Range target | Assessment |
|---|---|---|
| `unsupported_expression_form` | `yield 1` (full yield expr, via `expr.range()`) | Correct |
| `matrix_binop` | `1 @ 2` (full binop expr, via `binop.range()`) | Correct |
| `in`/`not in` non-collection | `comparator.range()` (the non-collection operand) | Correct |
| Unsupported comparison | `comparator.range()` | Correct |

All primary ranges point to the relevant source construct.

---

### 5. Unit tests

All three focused tests pass:

```
cargo test -p sifr_hir -- matrix_binop              ... ok
cargo test -p sifr_hir -- unsupported_expression_form ... ok
cargo test -p sifr_hir -- in_operator_non_collection ... ok
```

`test_matrix_binop_has_unsupported_operator_code` (line 714): verifies `TYPE_UNSUPPORTED_OPERATOR`, message `"matrix multiplication operator (@) is not supported"`, range over `1 @ 2`.

`test_unsupported_expression_form_has_type_code` (line 730): verifies `TYPE_UNSUPPORTED_EXPRESSION_FORM`, message `"unsupported expression form: unsupported expression type"`, range over `yield 1`.

`test_in_operator_non_collection_has_unsupported_operator_code` (line 745): verifies `TYPE_UNSUPPORTED_OPERATOR`, message `"unsupported operator in for int"`, range over `2`.

---

### 6. E2E fixture

`unsupported_yield_expression.sifr`:
```sifr
# expect-error[col=10]: SIFR-TYPE-0012
# Test: unsupported expression forms use structured expression-form diagnostics.

def main():
    x = (yield 1)
```

The fixture exercises the `lower_expr` fallback arm for `yield` expressions, using the `SIFR-TYPE-0012` code.

---

### 7. Documentation and schema

- `docs/errors/SIFR-TYPE-0012.md` exists, generated by `gen-error-docs`, correct content (owner, template, fixture, declared-args all match registry).
- `DIAGNOSTIC_REGISTRY` test (`registry_skeleton_is_internally_consistent`) passes for all entries.
- `active_diagnostic_docs_pages_exist_with_exact_casing` passes — all active diagnostics have doc pages.

---

### 8. HIR maintainability guardrail compliance

```
python3 scripts/check_hir_maintainability_guardrails.py
HIR maintainability guardrails: PASS
```

`expression_operators.rs` (34 lines) and `expression_diagnostics.rs` (33 lines) are small, focused files — no monolithic file issues. Both are private submodules of `lower`, visible only within the crate.

---

### 9. No fallback behavior

- `lower_expr` fallback: `unsupported_form()` → structured diagnostic → `return None`
- `lower_binop` MatMult path: `matrix_multiplication()` → structured diagnostic → `return None`
- `lower_compare` unsupported op: `unsupported_operator()` → structured diagnostic → `return None`
- `in`/`not in` on non-collection: `unsupported_operator()` fires as supplementary diagnostic, lowering continues with valid `ContainsOp` HIR node — this is intentional (the node is type-correct HIR, the diagnostic is supplemental)

No `.unwrap()`, `.expect()`, or panicking in user-facing code paths.

---

### 10. Module organization

`mod.rs:27-28`:
```rust
mod expression_diagnostics;
mod expression_operators;
```

Both are private (`mod`, not `pub mod`) internal submodules of `lower`. The dispatch in `expressions.rs` uses `use super::expression_operators::{...}` to import the moved functions. Correct pattern — no public re-exports leak outside the crate.

---

### 11. Validation tooling (per user confirmation)

| Check | Result |
|---|---|
| `cargo fmt --check` | Pass (no output = formatted correctly) |
| `cargo check -p sifr_hir -p sifr_diagnostics` | Pass |
| `cargo clippy -p sifr_hir -p sifr_diagnostics -- -D warnings` | Pass |
| `cargo test -p sifr_hir -- matrix_binop` | Pass |
| `cargo test -p sifr_hir -- unsupported_expression_form` | Pass |
| `cargo test -p sifr_hir -- in_operator_non_collection` | Pass |
| `scripts/check_hir_maintainability_guardrails.py` | Pass |

---

## Summary

| Check | Result |
|---|---|
| Refactor integrity (pure move of `lower_binop`/`lower_unaryop`/`lower_compare`) | Pass |
| `TYPE_UNSUPPORTED_EXPRESSION_FORM` taxonomy and registry | Pass |
| Reuse of `TYPE_UNSUPPORTED_OPERATOR` for operator failures | Pass |
| Primary ranges | Pass |
| Unit tests | Pass |
| E2E fixture | Pass |
| Docs/schema consistency | Pass |
| HIR maintainability guardrail compliance | Pass |
| No fallback behavior | Pass |
| Module organization (private submodules) | Pass |
| `cargo fmt` | Pass |
| `cargo check` / `cargo clippy` | Pass |

**No required fixes remain.**
