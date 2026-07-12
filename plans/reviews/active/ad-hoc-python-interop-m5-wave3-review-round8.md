Now I have enough evidence. Let me compose the round-8 review.

## Round-7 verification

All three round-7 findings are closed:

- **Bare Name discard** at `statement_dispatch.rs:255-260` now calls `python_interop::reject_python_context_borrow_discard` on every non-yield `Stmt::Expr`. Reproduction `transaction` as a bare statement fires `SIFR-PYCTX-0001 … cannot be discarded` (verified via `target/debug/sifr check /tmp/wave3audit8/bare_name_discard.sifr`).
- **Bare aggregate discard** fires the same diagnostic since `python_context_borrow_in_owned_expr` at `context.rs:21-30` walks `ListLiteral`/`SetLiteral`/`TupleLiteral` elements unconditionally.
- **Regular call-arg aggregate escape** is closed by `reject_python_context_borrow_in_temporary_argument` at `regular_calls.rs:13-21`, invoked at `regular_calls.rs:114-118` (Callable-object path) and `regular_calls.rs:309-319` (normal function path). Reproduction `sink_list([transaction])` fires `SIFR-PYCTX-0001 … cannot be stored outside its context binding` (verified via `regular_call_agg.sifr`). Direct-Name arguments remain accepted because the helper skips them.

## Fresh finding — method calls with aggregate arguments still leak the borrow

**File:** `crates/sifr_lowering/src/lower/expressions/methods_lambdas_and_comprehensions.rs:21-285` (the entire `lower_method_call` path), reinforced by the shape of `python_context_borrow_in_owned_expr` at `crates/sifr_lowering/src/lower/python_interop/context.rs:91-98`.

`lower_method_call` never invokes `reject_python_context_borrow_in_temporary_argument` (or any per-argument storage guard) after lowering `args`. The statement-level fallback in `reject_python_context_borrow_discard` (`statement_dispatch.rs:256`) only helps if `python_context_borrow_in_owned_expr` recurses into `HirExpr::MethodCall::args` — but the arm at `context.rs:91-98` is guarded by `type_can_hold_python_opaque(ty, ctx)`. When the method returns `None`/`bool`/etc., the arm never fires and the aggregate arg is never walked. This is the exact analog of round-7 case (3), fixed for regular calls but not for method calls.

Concrete reproductions — each returns `no errors found`:

1. **Instance-method escape (list arg)** — `/tmp/wave3audit8/method_call_aggregate.sifr`
   ```python
   class Sink:
       def collect(self, values: list[Transaction]) -> None:
           return None

   def try_leak(sink: Sink) -> Result[None, PythonError]:
       try:
           with make_transaction() as transaction:
               sink.collect([transaction])            # MethodCall, ty=None → args not walked
           return None
       except PythonError as error:
           raise error
   ```
   Verified: `target/debug/sifr check /tmp/wave3audit8/method_call_aggregate.sifr` → `no errors found`.

2. **Instance-method escape (tuple arg)** — `/tmp/wave3audit8/method_call_bare_list.sifr` with `sink.collect_tuple((transaction, 1))` and parameter `pair: tuple[Transaction, int]`. Also `no errors found`.

3. **Instance-method escape (set arg)** — `/tmp/wave3audit8/method_set_arg.sifr` with `sink.collect_set({transaction})` and parameter `values: set[Transaction]`. Also `no errors found`.

4. **`ClassName.method(...)` (staticmethod-shape) escape** — `/tmp/wave3audit8/classmethod_agg.sifr`.
   ```python
   class Registry:
       @staticmethod
       def store(values: list[Transaction]) -> None: return None
   ...
       with make_transaction() as transaction:
           Registry.store([transaction])
   ```
   This path in `methods_lambdas_and_comprehensions.rs:61-89` synthesizes a `HirExpr::Call { func: "Registry::store", args, ty: None }` *directly*, bypassing `lower_regular_call` and its temp-argument guard, and the resulting Call has `ty=None` so `context.rs:81-90` also declines to recurse. Verified: `no errors found`.

Sibling shape (`context.rs:81-90`): the same `type_can_hold_python_opaque` guard also applies to `HirExpr::PythonCall`, `HirExpr::IntrinsicCall`, `HirExpr::IteratorCall`, and `HirExpr::SuperCall`. Only regular `HirExpr::Call` args are backstopped, because the temp-arg guard at `regular_calls.rs:309` runs on those args before the node is materialised. Any code path that materialises a `PythonCall`, `MethodCall`, `IntrinsicCall`, `IteratorCall`, `SuperCall`, or synthesised `HirExpr::Call` (see the `Registry::store` path above) without going through `lower_regular_call` — with an aggregate arg carrying the borrow and a non-opaque return type — will slip past the guard.

Fix locus (unverified; do not implement in this pass):

- Primary: in `lower_method_call` (`methods_lambdas_and_comprehensions.rs`), after `args` is finalised (around line 166 for the class/protocol/other branches), iterate over `args` and call `reject_python_context_borrow_in_temporary_argument` (or `reject_python_context_borrow_storage` on every non-Name arg) with each arg's `TextRange`. Do the same in the `ClassName.method(...)` branch at lines 66-79 and the `super().method(...)` branch at lines 32-48.
- Secondary: in `python_context_borrow_in_owned_expr` (`context.rs:81-98`), drop the `type_can_hold_python_opaque(ty, ctx)` guard for the Call/PythonCall/IntrinsicCall/IteratorCall/SuperCall/MethodCall arms — the storage occurs in the temp aggregate arg itself, independent of what the callee returns. This mirrors the round-7 fix locus proposal, extended to MethodCall and the other four call arms.
- Regression coverage: add sibling tests to `python_interop_tests.rs` alongside `python_context_entered_borrow_cannot_move_through_temporary_call_argument` (`python_interop_tests.rs:598-610`) exercising `sink.collect([transaction])`, `sink.collect_tuple((transaction, 1))`, `sink.collect_set({transaction})`, and `Registry.store([transaction])`.

Severity: HIGH — reproducible, no diagnostic emitted, and the failure mode is "context-scoped borrow stored in a temp aggregate, closed independently of `__exit__`" — exactly the "cannot be moved or closed independently" invariant wave-3 acceptance is meant to enforce.

VERDICT: NOT SATISFIED
