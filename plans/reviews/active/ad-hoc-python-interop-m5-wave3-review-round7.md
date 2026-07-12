## Round 6 verification

Both round-6 findings are closed:

- **Aug-assign RHS** — all five paths (`aug_assign_lowering.rs:80/140/219/279/312`) now route the RHS through `lower_python_context_owned_expr`. Reproduction `stored += [transaction]` fires `SIFR-PYCTX-0001` (verified via `cargo run -- check`). Attribute and subscript variants also fire.
- **Walrus at expression-statement level** — `lower_named_expr` at `expressions/methods_lambdas_and_comprehensions.rs:887` calls `reject_python_context_borrow_storage(&value, named.value.range(), ctx)` before defining the walrus binding in scope. Reproduction `(alias := transaction)` fires `SIFR-PYCTX-0001` (verified via `cargo run -- check`).

HIR consumers (`hir_analysis/traversal/traversal_impl.rs:580-596`, `lib_codegen_tests/*`, `hir_snapshot_tests.rs`, `stmt_block_helpers.rs`, `with_yield_and_match.rs`, `async_with_and_for.rs`) are correctly adapted to the new `HirWithItem`/`HirWithItemKind::{Native,Python}` shape; Python-context codegen is gated behind an explicit "not active until M5 wave 4" error, keeping native `with` unchanged.

## Fresh finding — Stmt::Expr bare-expression discard bypasses the guard

**File:** `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs:255-269` (the general `Stmt::Expr` fallthrough, i.e. the path taken for any `expr_stmt.value` that is not `Expr::Yield`).

The `Stmt::Expr` arm lowers the bare expression via `let expr = lower_expr(&expr_stmt.value, ctx)?;` and only checks for `Type::Result(_,_)` "unused Result" plus the async-generator hook. **It never invokes `python_context_borrow_in_owned_expr` / `reject_python_context_borrow_discard`.** The explicit-discard sibling at `control_flow.rs:257-260` does invoke `reject_python_context_borrow_discard` for `_ = expr`, so the two supposedly-equivalent discard forms diverge — implicit discard is unchecked.

Result: at least three reproducible escape shapes compile cleanly on the current diff (each verified by writing the program to `/tmp/wave3audit/*.sifr` and running `target/debug/sifr check`; each returns `no errors found`):

1. **Bare Name discard**
   ```python
   def leak() -> Result[None, PythonError]:
       try:
           with make_transaction() as transaction:
               transaction         # implicit discard of the context borrow
           return None
       except PythonError as error:
           raise error
   ```
   Rewriting the same body as `_ = transaction` immediately fires `PYCTX_INVALID_DECLARATION` ("cannot be discarded"), confirming the divergence is specifically the `Stmt::Expr` arm.

2. **Bare aggregate discard**
   ```python
   with make_transaction() as transaction:
       [transaction]                # temp list containing the borrow; dropped at end of stmt
   ```
   Rewriting as `_ = [transaction]` immediately fires `PYCTX_INVALID_DECLARATION` on the aggregate; the bare form does not. `{transaction}` and `(transaction, 1)` reproduce identically.

3. **Call-arg aggregate escape**
   ```python
   def sink_list(l: list[Transaction]) -> None: return None
   def sink_tuple(t: tuple[Transaction, int]) -> None: return None
   def sink_set(s: set[Transaction]) -> None: return None
   ...
       with make_transaction() as transaction:
           sink_list([transaction])       # arg is a ListLiteral, not a Name
           sink_tuple((transaction, 1))
           sink_set({transaction})
   ```
   Compiles cleanly. `regular_calls.rs:121-127` only calls `mark_moved_with_flow` when the arg is a bare `HirExpr::Name` (so aggregate args skip the borrow-move check); `python_context_borrow_in_owned_expr` only walks `HirExpr::Call` args when the call return type can hold a Python opaque (`context.rs:82-97`), so a `-> None` sink never triggers the recursive walk into the aggregate arg either. The temp aggregate holding the borrow is dropped when the call returns — closing the borrow independently of the with block's `__exit__`, exactly the "cannot be moved or closed independently" shape wave-3 acceptance requires closed.

All three are sibling shapes to the round-6 walrus escape (which the diff fixes inside `lower_named_expr`) and to the round-5 chained-assign escape — "any binding/discard-introducing shape that doesn't route through the guard is a candidate escape," which round 5 flagged and which round 6 explicitly warned still needs a sweep.

**Fix locus (unverified; do not implement in this pass):**

- Primary: immediately after `let expr = lower_expr(&expr_stmt.value, ctx)?;` at `statement_dispatch.rs:255`, add `python_interop::reject_python_context_borrow_discard(&expr, expr_stmt.value.range(), ctx);`. This mirrors `control_flow.rs:260` and closes cases (1) and (2) above.
- Secondary: for case (3), extend `python_context_borrow_in_owned_expr` (`python_interop/context.rs:82-97`) so `HirExpr::Call`/`PythonCall`/`IntrinsicCall`/`IteratorCall`/`SuperCall`/`MethodCall` walks its args regardless of the return type's opaque-hold — the storage happens in the temp aggregate arg itself, not in the call's result. Add regression coverage paralleling `python_context_entered_borrow_cannot_be_aliased_or_discarded` (`python_interop_tests.rs:504-519`) with bare `"transaction"`, `"[transaction]"`, and a `sink_list([transaction])` case.

Severity: HIGH — reproducible, no diagnostic emitted, and the failure mode ("closed independently") is exactly what wave-3 acceptance names.

VERDICT: NOT SATISFIED
