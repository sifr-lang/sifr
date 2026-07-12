Summary:

1. **HIGH — yield escape (statement_dispatch.rs:235):** `yield transaction` from inside a Python `with` block is unchecked; a generator's caller receives the entered borrow after the block has exited.
2. **HIGH — tuple/star unpack escape (tuple_unpack.rs:55/169):** `a, b = transaction, 1` and `[first, *rest] = [transaction, 1]` bypass `reject_python_context_borrow_storage`, so the borrow can be rebound into an outer-scope binding.
3. **HIGH — Result-typed prebound manager not consumed (statement_dispatch.rs:405):** When the ?-wrap fires, `value` becomes `QuestionMark(Name(res))` and the `HirExpr::Name` guard skips `mark_moved_with_flow`; the manager can be re-used by a second `with res as u:`.
4. **MEDIUM — aggregate/comprehension escape (python_interop/context.rs:12):** `python_context_borrow_in_owned_expr` doesn't recurse into ListComp/SetComp/DictComp/GeneratorExpr/Lambda/Call/MethodCall/Walrus/Index/FieldAccess, so `stored = [transaction for _ in range(1)]` and similar shapes escape storage rejection. Parity with `transfer_return_ownership`'s recursion is missing.
5. **LOW — misleading diagnostic (statement_dispatch.rs:373-395):** A Python cleanup=context factory called inline outside a try block falls through to the native path and emits "context manager missing" for `Result`; a dedicated PYCTX diagnostic ("requires enclosing try") should fire.

Findings 1-3 are correctness/ownership gaps in the wave-3 scope's own acceptance criteria (return/aggregate/escape prohibition; hidden owner consumed exactly once). Finding 4 is a coverage gap that composes with 1/2 for further escape shapes. Finding 5 is a diagnostic-quality issue.

VERDICT: NOT SATISFIED
