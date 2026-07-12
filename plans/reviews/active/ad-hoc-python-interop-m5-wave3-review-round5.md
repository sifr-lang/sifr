I've completed the corrective re-review. Here is my substantive analysis.

## Round-3 blocker: closed

- Ordinary `for` iterable (`control_flow.rs:712`) now runs `pyinterop::lower_python_context_owned_expr(&for_stmt.iter, ctx)?`, which wraps `reject_python_context_borrow_storage`. The exact round-3 counter-example (`for item in [transaction]: return item`) now emits `PYCTX_INVALID_DECLARATION`, and there is a dedicated test at `python_interop_tests.rs:624-635` (`python_context_entered_borrow_cannot_escape_through_for_iterable`) that pins it.
- Async-for iterable (`async_for.rs:195`) routes through the same helper.

## Round-1/2 fixes: still in place

- Simple `lower_assign` (`control_flow.rs:300`) calls `reject_python_context_borrow_storage`; attribute/subscript/nested-attribute/nested-subscript variants (lines 68/94/149/194/227) all use `lower_python_context_owned_expr`.
- Discard (`_ = expr`, `control_flow.rs:260`) uses `reject_python_context_borrow_discard`.
- Tuple/star unpack (`tuple_unpack.rs:87, 192`) use `reject_python_context_borrow_unpack`.
- Yield escape (`statement_dispatch.rs:235`), fallible-outside-try dedicated diagnostic (`statement_dispatch.rs:396-404`), and prebound Result manager consumption (`statement_dispatch.rs:383-386, 427-428`) all remain wired correctly. `python_context_borrow_in_owned_expr` still recurses through comprehensions/generators/lambdas/walrus/opaque-typed Calls (`python_interop/context.rs:13-109`).

## New concrete escape (fresh finding, not in prior rounds)

**HIGH — Chained assignment bypasses the storage-borrow guard.** File: `crates/sifr_lowering/src/lower/statements/patterns_and_assignments.rs:479`.

`Stmt::Assign` with `targets.len() > 1` is dispatched to `lower_chained_assign` at `statement_dispatch.rs:132-137`, bypassing `lower_assign`'s guard at `control_flow.rs:300`. Inside `lower_chained_assign`:

```rust
let Some(value) = lower_expr(&assign.value, ctx) else {   // line 479
    return result;
};
```

No `reject_python_context_borrow_storage`, no `python_context_borrow_in_owned_expr`, no `record_must_use_binding`, and the freshly-defined targets (`ctx.scope.define(...)` at 503/533) are never inserted into `python_context_borrows`.

**Reproducible scenario** (would compile cleanly with the current diff; every other alias shape covered by `python_interop_tests.rs:504-519` correctly rejects):
```python
def escape() -> Result[Transaction, PythonError]:
    try:
        with make_transaction() as transaction:
            alias = shortcut = transaction   # chained — bypasses all borrow guards
            return alias                     # transfer_return_ownership at return_lowering.rs:130
                                             # only checks python_context_borrows,
                                             # which never received `alias` → escapes
    except PythonError as error:
        raise error
```

Trace: `lower_chained_assign` produces `HirStmt::Let{name:"shortcut", value: Name("transaction")}` and `HirStmt::Let{name:"alias", value: Name("shortcut")}` without diagnostics. `return alias` reaches `return_lowering.rs:130`: `alias` is not in `python_context_borrows`, its type (`Class{Transaction}`) has no must-use obligation registered for the borrow-created binding (chained path never calls `record_must_use_binding`), so `transfer_return_ownership` falls through with no error. This is exactly the "cannot escape, move, or close independently" shape the wave-3 acceptance criterion requires closed, and it is a direct sibling of the round-1 tuple/star-unpack bypass that was flagged and fixed.

**Fix locus** — the minimal correction (unverified; wave-4 codegen is still gated so I did not implement/run it) is to add, immediately after `lower_expr` at `patterns_and_assignments.rs:479-482`:
```rust
pyinterop::reject_python_context_borrow_storage(&value, assign.value.range(), ctx);
```
mirroring `control_flow.rs:300` and `patterns_and_assignments.rs:395-405`. A test paralleling `python_context_entered_borrow_cannot_be_aliased_or_discarded` at `python_interop_tests.rs:505-519` with `"alias = shortcut = transaction"` would pin it.

## Secondary observation (informational, not blocking)

`match transaction:` with `case bound:` would similarly capture the borrow into `bound` via `bind_pattern_vars` (`patterns_and_assignments.rs:224-226`) without touching `python_context_borrows`. This is less clearly in-scope for wave-3 (pattern matching on opaque Python classes is contrived), but it shares the same class of gap — any binding-introducing shape that doesn't route through the guard is a candidate escape. Recommend a follow-up sweep once the chained-assign fix lands.

VERDICT: NOT SATISFIED
