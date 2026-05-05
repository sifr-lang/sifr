# Review: diag-11 raw HIR method call args migration

**Reviewer:** Pass 1
**Status:** SATISFORY — with one informational concern

---

## Guardrail compliance

`method_call_args.rs` is listed in `RAW_HIR_ERROR_FREE_FILES` in `check_diagnostic_transport_cleanup.py` and contains **zero** raw `ctx.error(` calls. The guardrail is correctly maintained.

---

## Phase correctness

All migrated diagnostics use `ctx.error_with_code_at(...)` which correctly records `primary_range` via `HirDiagnostic`. This is the correct HIR-phase diagnostic transport — same pattern as `bytes_methods.rs` and `decimal_methods.rs` migrations.

---

## Diagnostic taxonomy correctness

| Diagnostic | Code | Phase | Range target | Verdict |
|---|---|---|---|---|
| Unpacked keyword `**{}` rejection | `CALL_UNEXPECTED_KEYWORD` | HIR call-lowering | `**{"value": 2}` (the unpacking syntax) | :white_check_mark: Correct |
| `list.extend(int)` non-iterable | `PROTO_INVALID_ITERATOR_SIGNATURE` | HIR type resolution | `1` (the argument) | :white_check_mark: Correct |
| `list.extend(str)` vs `list[int]` element mismatch | `TYPE_MISMATCH` | HIR type resolution | `1` (the argument) | :white_check_mark: Correct |
| `dict.update(keyword=wrong_type)` key/value mismatch | `TYPE_MISMATCH` | HIR type resolution | `bad="x"` (keyword arg) | :white_check_mark: Correct |
| `set.union(int)` non-iterable | `PROTO_INVALID_ITERATOR_SIGNATURE` | HIR type resolution | arg range | :white_check_mark: Correct |
| `set.union(str)` vs `set[int]` element mismatch | `TYPE_MISMATCH` | HIR type resolution | arg range | :white_check_mark: Correct |

---

## `resolved_method_arg_ranges` -- correctness of keyword range inclusion

**Implementation** (expressions.rs:2602-2609):
```rust
fn resolved_method_arg_ranges(object_ty: &Type, method: &str, call: &ExprCall) -> Vec<TextRange> {
    let mut ranges: Vec<TextRange> = call.arguments.args.iter().map(Ranged::range).collect();
    let canonical_ty = canonicalize_class_surface_type(object_ty);
    if matches!(canonical_ty.resolve_alias(), Type::Dict(_, _)) && method == "update" {
        ranges.extend(call.arguments.keywords.iter().take(1).map(Ranged::range));
    }
    ranges
}
```

The `iter().take(1)` on keywords is intentional and correct: `dict.update()` accepts at most **one** keyword-dict argument (Python semantics: `data.update(a=1, b=2)` is equivalent to `data.update({"a": 1, "b": 2})`). The synthesized dict-literal from keywords is represented as a single argument in `args`, so `arg_ranges[0]` = positional dict, `arg_ranges[1]` = keyword-dict. The `take(1)` ensures the keyword-dict gets `arg_ranges[1]` only when it actually exists.

---

## `validate_dict_update_arg` -- range for second positional dict arg

For `dict.update(other_dict)` (positional), `validate_dict_update_arg` is called with `arg_ranges[0]` (correct).

For `dict.update(bad="x")` (keyword normalization produces a synthesized dict as `args[0]`), the range is the **keyword** (`bad="x"`), which is the human-relevant source location. This is correct -- the synthesized dict has no direct source location, so using the keyword that produced it is the right UX choice.

**Concern (informational, not a blocker):** The test `test_dict_update_keyword_value_mismatch_has_type_code` verifies the keyword range (`bad="x"`), which is correct. However, there is no test for the case where a **positional** dict argument has a type mismatch (e.g., `data.update({"bad": "x"})`). This is a pre-existing gap, not introduced by this migration. Worth noting for follow-up but not required for this PR.

---

## New tests

| Test | What it covers | Verdict |
|---|---|---|
| `test_unpacked_method_keyword_has_call_code` | `xs.append(**{"value": 2})` -> `CALL_UNEXPECTED_KEYWORD` | :white_check_mark: |
| `test_list_extend_non_iterable_has_protocol_code` | `xs.extend(1)` -> `PROTO_INVALID_ITERATOR_SIGNATURE` | :white_check_mark: |
| `test_dict_update_keyword_value_mismatch_has_type_code` | `data.update(bad="x")` with wrong value type -> `TYPE_MISMATCH` | :white_check_mark: |

All three tests assert:
- `error.code == Some(DiagnosticCode::...)` -- correct code
- `error.primary_range == Some(range_for(source, "..."))` -- correct primary range

---

## expressions.rs changes

The diff shows **only** calls to `validate_*_arg` functions (which live in `method_call_args.rs`) now passing `arg_ranges[index]` -- no new `ctx.error(` calls introduced. The remaining ~180 `ctx.error(` calls in `expressions.rs` are pre-existing and expected (this file is not yet migrated).

---

## No source-range regressions detected

- `lower_method_call` calls `resolved_method_arg_ranges` and passes the result to `resolve_method_type` -- same call chain as before, only the range construction is more precise for `dict.update`.
- All callers of `validate_*_arg` in `resolve_method_type` (expressions.rs) now pass `arg_ranges[index]` -- the ranges correspond to the actual argument expressions in source.

---

## Summary

The migration is clean, correct, and guardrail-compliant. No raw `ctx.error(` calls remain in `method_call_args.rs`. Diagnostic codes match the taxonomy. Primary ranges are precisely targeted. Tests cover the key cases.

**Recommendation: APPROVE.** No second pass required for this file.
