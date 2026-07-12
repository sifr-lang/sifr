I have two independently reproducible escape paths that pass the current diff cleanly. I'm now confident in the NOT SATISFIED verdict.

## Findings

### HIGH — `lower_aug_assign` RHS bypasses the context-borrow guard.

**File:** `crates/sifr_lowering/src/lower/aug_assign_lowering.rs:311` (name target), `:218` (attribute target), `:218/139` (subscript-of-attribute / nested-subscript), `:79` (attribute augmented).

Every `let value = lower_expr(&aug.value, ctx)?;` in `lower_aug_assign` lowers the RHS without invoking `python_context_borrow_in_owned_expr` / `reject_python_context_borrow_storage`. `HirStmt::AugAssign` is emitted as `Mutation` (see `flow_graph/effects.rs:63-69`) rather than an `Assign`, so `record_must_use_binding` never re-tightens the target's obligation set either. Result: an opaque context borrow can be aggregated into a caller-owned list and thereby outlive the `with` block.

**Reproducible via `cargo run -q -p sifr -- check` (no errors emitted):**

```python
# CONTEXT_OPAQUE_PREFIX abbreviated — Transaction is @python.opaque cleanup=context

def leak(mut stored: list[Transaction]) -> Result[None, PythonError]:
    try:
        with make_transaction() as transaction:
            stored += [transaction]        # bypasses PYCTX_INVALID_DECLARATION
        return None
    except PythonError as error:
        raise error
```

I verified this literally: writing that program to `/tmp/aug_test/leak4.sifr` and running `cargo run --manifest-path .../Cargo.toml -q -p sifr -- check ...` prints `no errors found`. Rewriting the same body as `stored = stored + [transaction]` (a plain `Stmt::Assign`) immediately fires `SIFR-OWN-0001` / `PYCTX_INVALID_DECLARATION`, so the gap is specifically the aug-assign statement — the round-5 chained-assign fix pattern (`patterns_and_assignments.rs:482`) is missing here.

The identical hole reproduces with an attribute target (`box.items += [transaction]`, `class Box: items: list[Transaction]`) and with a subscript target (`outer[0] += [transaction]`) — I confirmed both compile cleanly on the current diff.

**Fix locus (unverified; do not implement in this pass):** after each `let value = lower_expr(&aug.value, ctx)?;` at `aug_assign_lowering.rs:79, 139, 218, 311`, invoke `python_interop::reject_python_context_borrow_storage(&value, aug.value.range(), ctx)`. Mirror the tuple/star-unpack (`tuple_unpack.rs:87, 192`) and chained-assign (`patterns_and_assignments.rs:482-486`) fixes. Regression coverage should parallel `python_context_entered_borrow_cannot_be_aliased_or_discarded` with augmented forms.

### HIGH — Walrus at expression-statement level escapes the guard.

**File:** `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs:255-269` (`Stmt::Expr` arm).

`Stmt::Expr` lowers its expression via bare `lower_expr` and emits `HirStmt::Expr`. For an outer walrus (`(alias := transaction)`) that means `lower_named_expr` (`methods_lambdas_and_comprehensions.rs:889`) runs `ctx.scope.define("alias", Class{Transaction})` and returns without ever consulting `python_context_borrow_in_owned_expr`, so `alias` is a fresh binding whose type is the borrow's, but `alias` is never inserted into `python_context_borrows`. `transfer_return_ownership` therefore treats `alias` as an ordinary name and lets it escape.

**Reproducible via `cargo run -q -p sifr -- check` (no errors emitted):**

```python
def leak() -> Result[Transaction, PythonError]:
    try:
        with make_transaction() as transaction:
            (alias := transaction)
            return alias
    except PythonError as error:
        raise error
```

I verified this compiles cleanly on the current diff (`walrus_leak.sifr`, `no errors found`). Rewriting as `alias = transaction; return alias` immediately fires `PYCTX_INVALID_DECLARATION`, confirming the divergence is specifically in `Stmt::Expr`.

**Fix locus (unverified):** in the `Stmt::Expr` arm at `statement_dispatch.rs:255-269`, either (a) run `python_context_borrow_in_owned_expr` on the lowered `expr` and reject when it captures a borrow, or (b) detect `HirExpr::WalrusExpr` and route it through `reject_python_context_borrow_storage`. The chained-assign fix at `patterns_and_assignments.rs:482` is the closest structural analogue.

---

Both gaps are sibling shapes to the round-1 tuple/star-unpack bypass, the round-3 for/async-for iterable bypass, and the round-5 chained-assign bypass — the "any binding-introducing shape that doesn't route through the guard is a candidate escape" concern the round-5 report explicitly flagged. Two independent statement-level paths still evade every guard, which reproduces the exact "cannot escape, move, or close independently" failure mode that wave-3 acceptance requires closed.

VERDICT: NOT SATISFIED
