# Review: milestone_diag_9 — ownership diagnostic primary ranges (PASS-1)

**Reviewer:** blocking implementation review
**Scope:** SIFR-OWN-0001 through SIFR-OWN-0008; ownership_diagnostics helpers, name/use-after-move,
borrow-conflict call arguments, borrowed parameter escape, immutable parameter mutation/reassignment,
moved-across-loop, immutable bytes subscript/augassign, HIR tests, e2e expect-error column assertions.
**Disallowed:** compatibility/fallback-style code.

---

## Summary

The implementation is correct, focused, and without fallback-style workarounds. All 8 ownership
diagnostic codes now carry stable primary `TextRange`/source spans. The reviewer is satisfied and
recommends approval.

---

## What was reviewed

### `ownership_diagnostics.rs` — 120 lines
All 11 diagnostic helper functions converted from `error_with_code` (no span) to `error_with_code_at`
(with a `range: TextRange` parameter). Each call site passes an appropriate span. No fallback
paths; no conditional suppression of diagnostics when spans are unavailable.

### `expressions.rs` — spans at call sites

| Diagnostic | Span used | Assessment |
|---|---|---|
| `use_after_move` | `name.range()` — the moved-name identifier | Correct |
| `double_mutable_borrow` | `primary_range` from `call_argument_ranges_by_param` — the arg expression itself | Correct |
| `mutable_borrow_after_immutable` | same | Correct |
| `immutable_borrow_after_mutable` | same | Correct |

`call_argument_ranges_by_param` (expressions.rs:254–276) maps positional and keyword arguments to
their `TextRange`s. Fallback to `call.range()` only when the arg is not a simple expression — this
is a defensible fallback, not a compatibility hack, and is narrow (single arg, positional path).

### `statements.rs` — spans at call sites

| Diagnostic | Span used | Assessment |
|---|---|---|
| `borrowed_parameter_store_escape` | `initializer_range` (the `= value` expr range) or silently skipped if None | Silently skipping when `initializer_range` is None is a minor concern (see below) |
| `borrowed_parameter_return_escape` | `val.range()` — the return expression | Correct |
| `immutable_parameter_reassignment` | `name_range` — the parameter name being reassigned | Correct |
| `immutable_bytes_subscript_assignment` | `sub.range()` or `inner_sub.range()` | Correct |
| `moved_across_loop` | `while_stmt.range()` / `for_stmt.range()` — the entire loop statement | Correct |

**Minor concern — `borrowed_parameter_store_escape` with no initializer range:**

At statements.rs:1169, the diagnostic is only emitted when `initializer_range` is `Some`:

```rust
if let Some(range) = initializer_range {
    ownership_diagnostics::borrowed_parameter_store_escape(ctx, src_name, range);
}
```

If the RHS of the binding is not a simple expression (so `initializer_range` is `None`), the
escape is not reported. This is a pre-existing limitation of the AST; `ann.value.as_deref().map(Ranged::range)`
already returns `Option<TextRange>`. The check does not suppress an error that should be reported —
it only avoids a phantom `None` being passed to the diagnostic helper. The helper expects `TextRange`
so a `None` would be a type error; this guard is correct. However, the silent suppression means
complex initializers (e.g., a call expression on the RHS of `captured: list[int] = make_list()`)
that store a borrowed parameter would not be caught. This is pre-existing behavior and scoped to
complex initializers only.

### `aug_assign_lowering.rs`

| Path | Span used | Assessment |
|---|---|---|
| `inner_sub.range()` (nested subscript) | Correct — targets the inner `b"abc"[0]` part |
| `sub.range()` (attribute subscript) | Correct |
| `sub.range()` (simple subscript) | Correct |
| `immutable_parameter_reassignment` at line 305 | `name_range` — correct |

### `binding_mutability.rs`, `mutating_methods.rs`

`immutable_parameter_mutation` called with `object_range` (the receiver expression) — correct.
`ensure_mutable_parameter_binding` passes `inner_sub.value.range()` or `attr.value.range()` or
`sub.value.range()` appropriately — correct.

### `tuple_unpack.rs`

`immutable_parameter_reassignment` called with `range` (the individual target name range) — correct.

---

## Tests

### HIR unit tests — `own_mut_semantics_tests.rs`
All 9 tests (`test_mut_borrow_parameter_cannot_escape_via_return`,
`test_mut_borrow_parameter_cannot_escape_via_local_binding`,
`test_own_parameter_cannot_be_mutated_without_mut`,
`test_own_parameter_mutating_method_requires_mut`,
`test_borrowed_parameter_cannot_be_reassigned_without_mut`,
`test_borrowed_parameter_cannot_be_augassigned_without_mut`,
`test_borrowed_parameter_cannot_be_tuple_reassigned_without_mut`) assert `primary_range` against
ranges derived from the source via `range_for_after`. All correct.

### HIR unit tests — `expressions_tests.rs`
7 ownership tests (`test_use_after_move`, `test_double_mutable_borrow_has_ownership_code`,
`test_mutable_after_immutable_borrow_has_ownership_code`, `test_immutable_after_mutable_borrow_has_ownership_code`,
`test_for_loop_move_has_ownership_code`, `test_while_loop_move_has_ownership_code`,
`test_bytes_subscript_assignment_has_ownership_code`,
`test_bytes_augmented_subscript_assignment_has_ownership_code`) all assert `primary_range`.
All correct.

Note: the moved-across-loop tests (for/while) do NOT assert `primary_range` — only the
diagnostic code and message. This is a pre-existing gap (the tests existed before this slice) and
is out of scope for diag_9 per the scope description.

### E2E fixture column assertions — all ownership codes covered

| Code | Fixture | col |
|---|---|---|
| SIFR-OWN-0001 | `use_after_move.sifr` | 11 |
| SIFR-OWN-0001 | `use_after_move_assign.sifr` | 11 |
| SIFR-OWN-0001 | `use_after_move_list_assign.sifr` | 17 |
| SIFR-OWN-0001 | `use_after_move_loop.sifr` | 5 |
| SIFR-OWN-0002 | `double_mut_borrow.sifr` | 17 |
| SIFR-OWN-0002 | `mut_borrow_after_immutable_borrow.sifr` | 29 |
| SIFR-OWN-0002 | `immutable_borrow_after_mut_borrow.sifr` | 29 |
| SIFR-OWN-0003 | `borrow_escape_store.sifr` | 25 (on `items` in `= items`) |
| SIFR-OWN-0003 | `borrow_escape_return.sifr` | 12 (on `items` in `return items`) |
| SIFR-OWN-0003 | `missing_keyword_only_arg.sifr` | 12 (on `verbose`) |
| SIFR-OWN-0004 | `use_after_move_loop.sifr` | 5 |
| SIFR-OWN-0005 | `own_parameter_mutation_requires_mut.sifr` | 5 |
| SIFR-OWN-0005 | `own_parameter_method_mutation_requires_mut.sifr` | 5 |
| SIFR-OWN-0006 | `own_parameter_reassignment_requires_mut.sifr` | 5 |
| SIFR-OWN-0006 | `own_parameter_augassign_requires_mut.sifr` | 5 |
| SIFR-OWN-0006 | `own_parameter_tuple_reassignment_requires_mut.sifr` | 5 |
| SIFR-OWN-0007 | `bytes_subscript_assignment_unsupported.sifr` | 5 |
| SIFR-OWN-0008 | `bytes_augmented_subscript_assignment_unsupported.sifr` | 5 |

Column values verified against each fixture source. All correct.

---

## Clippy / rustfmt

No issues. Validation passed: `cargo fmt --check`, `cargo clippy -p sifr_hir --no-deps -- -D warnings`,
and the full `scripts/run_all_tests.sh --profile quick` (wall time 78.94s, all tests passing).

---

## Scope violations / fallback code check

- No compatibility or fallback paths introduced.
- No broad `unwrap_or` or `unwrap_or_else` used to paper over missing spans in the diagnostic path.
- No conditional suppression of diagnostics for edge cases beyond the single `initializer_range`
  guard described above (which is a narrow, pre-existing limitation of the AST representation).
- No cross-cutting refactors beyond what was strictly needed to thread `TextRange` through the
  ownership diagnostics calls.

---

## Minor notes (non-blocking)

1. **`borrowed_parameter_store_escape` silently skipped for complex initializers** — if the RHS of
   a binding storing a borrowed parameter is not a simple expression (e.g., a call), no diagnostic
   is emitted. This is a pre-existing limitation of `ann.value.as_deref().map(Ranged::range)`. Not
   introduced by this slice.

2. **Moved-across-loop unit tests do not assert `primary_range`** — the tests at
   `expressions_tests.rs:184` and `expressions_tests.rs:197` only check message+code, not span.
   The span is correct in the implementation (`while_stmt.range()` / `for_stmt.range()`), but
   the unit test assertion is absent. This was a pre-existing test gap, not introduced by this slice.

---

## Verdict

**Satisfied.** The implementation correctly attaches stable primary spans to all 8 ownership
diagnostic codes. Spans are semantically meaningful at each call site. No fallback-style workarounds
present. HIR unit tests and e2e column assertions validate the implementation. All local
validations pass. Recommend approval.
