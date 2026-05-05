# Review: milestone_diag_9 TypeVar annotation-shape primary ranges

## What changed

- `typevar_annotations.rs` created — contains `encode_typevar_constraint`, `decode_typevar_constraint`, `invalid_typevar_shape`, `parse_typevar_bound_expr`, `parse_typevar_declaration_specs` extracted from `lower/mod.rs`
- `lower/mod.rs` — removed the above items; re-exports them via `pub(super)`
- `lower/expressions_tests.rs` — three new tests asserting `primary_range` on TypeVar shape diagnostics
- Seven TypeVar e2e fail fixtures — all migrated from `expect-error: SIFR-TYPE-0007` to `expect-error[col=N]: SIFR-TYPE-0007` to verify column accuracy

---

## Findings

**1. `error_with_code_at` vs `error_with_code` in new file**

The original `invalid_typevar_shape` (mod.rs) called `ctx.error_with_code(DiagnosticCode::TYPE_INVALID_ANNOTATION, message.into())` — no primary range. The new version calls `ctx.error_with_code_at(DiagnosticCode::TYPE_INVALID_ANNOTATION, message.into(), range)` — range-aware. Good.

**2. Bound/constraints conflict range in `parse_typevar_declaration_specs`**

In the original code the conflict was emitted once at the end of the function body with no range. In the new file it is emitted at `arg_name.range()` when the second conflicting keyword is encountered (lines 86–92, 105–111). This is more precise — it pinpoints the offending keyword rather than the whole declaration. Good.

**3. Module-level re-export is `pub(super)`**

```rust
pub(super) use typevar_annotations::{
    decode_typevar_constraint, encode_typevar_constraint, parse_typevar_bound_expr,
    parse_typevar_declaration_specs,
};
```
Makes items available to sibling modules within the crate but not externally. Correct for `type_var_collection` and `type_bounds` consumers. Good.

**4. `ExprCall` import removed from `mod.rs`**

`use sifr_python_ast::{Expr, Stmt};` replaces `use sifr_python_ast::{Expr, ExprCall, Stmt};` since `parse_typevar_declaration_specs` moved. Correct. Good.

**5. Guardrail check**

`python3 scripts/check_hir_maintainability_guardrails.py` passed. New file is small (144 lines, focused). Good.

**6. E2E fixture column values**

| Fixture | col |
|---|---|
| `pep695_typevar_bound_shape` | 13 |
| `pep695_typevar_constraint_shape` | 19 |
| `typevar_bound_and_constraints_shape` | 23 |
| `typevar_bound_shape` | 24 |
| `typevar_keyword_constraint_element_shape` | 36 |
| `typevar_keyword_constraint_shape` | 30 |
| `typevar_positional_constraint_shape` | 18 |

All column offsets land on the invalid token (e.g., `1` in `T: 1`, `1` in `T: (int, 1)`, `bound=` keyword for the conflict case). Good.

**7. Test assertions verify both `primary_range` and `code`**

Three new tests check `error.message`, `error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)`, and `error.primary_range == Some(range_for_after_anchor(...))`. No fallback or raw diagnostic path. Good.

**8. `reported_bound_constraints_conflict` flag**

The original boolean check at the end is replaced by an inline flag emitted on the second conflicting arg. More precise, same semantics, no double-reporting. Good.

---

## Verdict

**Satisfied.** The slice correctly:
- Extracts TypeVar shape helpers into a dedicated file satisfying HIR guardrails
- Attaches concrete `TextRange` primary ranges to all TypeVar shape diagnostics (SIFR-TYPE-0007)
- Removes the old no-range `invalid_typevar_shape` completely
- Uses `expect-error[col=N]` in e2e fixtures to lock in column accuracy
- Adds unit-level `primary_range` assertions covering bound shape, bound+constraints conflict, and PEP 695 constraint shape

No fallback or raw diagnostics. No dead code. No structural issues detected.
