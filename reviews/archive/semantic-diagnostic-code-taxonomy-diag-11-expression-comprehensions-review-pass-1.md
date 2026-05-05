# DIAG-11 Expression Comprehensions/Generator/Walrus — Review Pass 1

## Scope
- `crates/sifr_hir/src/lower/mod.rs`
- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`

---

## Changes Reviewed

### mod.rs — Raw `LowerCtx::error` removal

The now-unused `fn error(&mut self, message: String) -> ErrorTaint` was removed from `LowerCtx`.
Only `error_with_code_at` remains as the structured entry point. This is correct — no raw error
transport remains in the lowering context itself.

### expressions.rs — Three structured helpers introduced

```rust
fn reject_invalid_expression_target(ctx: &mut LowerCtx, message: &str, range: TextRange)
// → FLOW_INVALID_ASSIGNMENT_TARGET

fn reject_invalid_expression_iteration(ctx: &mut LowerCtx, iter_ty: &Type, range: TextRange)
// → FLOW_INVALID_ITERATION

fn reject_unsupported_expression_form(ctx: &mut LowerCtx, message: &str, range: TextRange)
// → TYPE_UNSUPPORTED_EXPRESSION_FORM
```

All call sites migrated:

| Call site | Old message | New helper | Code |
|---|---|---|---|
| `lower_list_comp` empty generators | "list comprehension must have at least one generator" | `reject_unsupported_expression_form` | TYPE_UNSUPPORTED_EXPRESSION_FORM |
| `lower_list_comp` tuple with non-name | "comprehension tuple target must contain only simple names" | `reject_invalid_expression_target` | FLOW_INVALID_ASSIGNMENT_TARGET |
| `lower_list_comp` non-name/tuple target | "comprehension target must be a simple name or tuple" | `reject_invalid_expression_target` | FLOW_INVALID_ASSIGNMENT_TARGET |
| `lower_list_comp` non-iterable | "cannot iterate over type '{iter_ty}'" | `reject_invalid_expression_iteration` | FLOW_INVALID_ITERATION |
| `lower_set_comp` non-name target | "set comprehension target must be a simple name" | `reject_invalid_expression_target` | FLOW_INVALID_ASSIGNMENT_TARGET |
| `lower_set_comp` non-iterable | "cannot iterate over type '{iter_ty}'" | `reject_invalid_expression_iteration` | FLOW_INVALID_ITERATION |
| `lower_dict_comp` tuple with non-name (**new check**) | "dict comprehension tuple target must contain only simple names" | `reject_invalid_expression_target` | FLOW_INVALID_ASSIGNMENT_TARGET |
| `lower_dict_comp` non-name/tuple target | "dict comprehension target must be a simple name or tuple" | `reject_invalid_expression_target` | FLOW_INVALID_ASSIGNMENT_TARGET |
| `lower_dict_comp` non-iterable | "cannot iterate over type '{iter_ty}'" | `reject_invalid_expression_iteration` | FLOW_INVALID_ITERATION |
| `lower_generator_expr` multi-gen | "only single-generator generator expressions are supported" | `reject_unsupported_expression_form` | TYPE_UNSUPPORTED_EXPRESSION_FORM |
| `lower_generator_expr` non-name target | "generator target must be a simple name" | `reject_invalid_expression_target` | FLOW_INVALID_ASSIGNMENT_TARGET |
| `lower_generator_expr` non-iterable | "cannot iterate over type '{iter_ty}'" | `reject_invalid_expression_iteration` | FLOW_INVALID_ITERATION |
| `lower_named_expr` non-name target | "walrus operator target must be a simple name" | `reject_invalid_expression_target` | FLOW_INVALID_ASSIGNMENT_TARGET |

#### Dict comprehension tuple target check (tightening)

The diff shows a **net-new check** in `lower_dict_comp` that was absent before:

```rust
if names.len() != tup.elts.len() {
    reject_invalid_expression_target(
        ctx,
        "dict comprehension tuple target must contain only simple names",
        gen.target.range(),
    );
    return None;
}
```

This mirrors the list comprehension tuple-target check. Previously, `lower_dict_comp` would
only check for `Expr::Name` vs `Expr::Tuple` vs `_` (the catch-all), but did NOT validate
that tuple elements were all simple names — so `values[0]` inside a tuple pattern would slip
through. The new check correctly catches this and uses `gen.target.range()` as the primary
range, matching the list comprehension pattern. This is an intentional tightening, not a
behavior change on valid programs.

### expressions_tests.rs — 10 new tests

| Test | Taxonomy | Range |
|---|---|---|
| `test_list_comprehension_invalid_target_has_flow_code` | FLOW_INVALID_ASSIGNMENT_TARGET | `gen.target.range()` — "values[0]" |
| `test_list_comprehension_non_iterable_has_flow_code` | FLOW_INVALID_ITERATION | `gen.iter.range()` — "value" |
| `test_set_comprehension_invalid_target_has_flow_code` | FLOW_INVALID_ASSIGNMENT_TARGET | `gen.target.range()` — "values[0]" |
| `test_set_comprehension_non_iterable_has_flow_code` | FLOW_INVALID_ITERATION | `gen.iter.range()` — "value" |
| `test_dict_comprehension_invalid_tuple_target_has_flow_code` | FLOW_INVALID_ASSIGNMENT_TARGET | `gen.target.range()` — "(left, values[0])" |
| `test_dict_comprehension_non_iterable_has_flow_code` | FLOW_INVALID_ITERATION | `gen.iter.range()` — "value" |
| `test_generator_expression_multi_generator_has_type_code` | TYPE_UNSUPPORTED_EXPRESSION_FORM | `gen.range()` — full paren expr |
| `test_generator_expression_invalid_target_has_flow_code` | FLOW_INVALID_ASSIGNMENT_TARGET | `gen.target.range()` — "values[0]" |
| `test_generator_expression_non_iterable_has_flow_code` | FLOW_INVALID_ITERATION | `gen.iter.range()` — "value" |
| `test_walrus_invalid_target_has_flow_code` | FLOW_INVALID_ASSIGNMENT_TARGET | `named.target.range()` — "NoneLiteral" |

---

## Taxonomy Consistency Check

| Category | Expected code | Present? |
|---|---|---|
| Invalid assignment target (comprehension/generator/walrus) | FLOW_INVALID_ASSIGNMENT_TARGET | Yes |
| Non-iterable comprehension/generator source | FLOW_INVALID_ITERATION | Yes |
| Unsupported expression form (multi-gen, empty list-comp) | TYPE_UNSUPPORTED_EXPRESSION_FORM | Yes |

Ranges are precise: invalid targets point to the target expression, non-iterable sources point
to the iterable expression, and the multi-generator form points to the full expression shape.

---

## Validation Results

```
rg -n "ctx\.error\(" crates/sifr_hir/src -g'*.rs'  →  (no matches)
cargo check -p sifr_hir                           →  OK
cargo clippy -p sifr_hir -- -D warnings            →  OK
python3 scripts/check_hir_maintainability_guardrails.py → PASS
python3 scripts/check_diagnostic_transport_cleanup.py  → Exit 0 (PASS)
git diff --check                                     →  clean
```

All 11 comprehension/generator/walrus tests pass:
- 6 comprehension tests
- 4 generator expression tests
- 1 walrus test

---

## Accidental Behavior Changes

- None detected. All 12 previously-existing call sites now route through the structured helpers
  with identical messages and taint semantics.
- The `ctx.error()` removal was safe — the method was only called from expressions in the
  comprehension/generator/walrus slice. No other call sites exist in the codebase.

---

## Remaining Raw Diagnostic Transport

`rg -n "ctx\.error\(" crates/sifr_hir/src -g'*.rs'` returns **no matches**. The slice intent
("after this slice, no raw ctx.error calls remain") is satisfied.

---

## Conclusion

**No blocking issues remain.** The migration is complete, consistent with the taxonomy, ranges
are precise, tests are adequate, and all validation gates pass.
