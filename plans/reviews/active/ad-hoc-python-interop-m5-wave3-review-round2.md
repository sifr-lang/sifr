## Summary

**Round-1 findings closure (verified):**

1. ✓ **yield escape** — `statement_dispatch.rs:235-245` now runs `python_context_borrow_in_owned_expr` on the yield value and emits PYCTX_INVALID_DECLARATION with "cannot escape by yield". Covered by `python_context_entered_borrow_cannot_escape_by_yield`.
2. ✓ **tuple/star unpack** — `tuple_unpack.rs:87, 192` call `reject_python_context_borrow_unpack` on the RHS. Covered by `python_context_entered_borrow_cannot_escape_through_unpacking`.
3. ✓ **Result-typed prebound consumption** — `statement_dispatch.rs:383-386` captures `context_owner` from the pre-?-wrap `HirExpr::Name`, and `:427-429` calls `mark_moved_with_flow` after the QuestionMark wrap so the outer Result binding is consumed by the first `with`. Covered by `result_typed_prebound_python_manager_is_consumed_by_first_with`.
4. ~**Partial**~ **owned-expression recursion** — Lambda / walrus / GeneratorExpr(expr,iter,filter) / opaque-gated Call/MethodCall/Index/FieldAccess/Await added, but **ListComp/SetComp/DictComp still skip their generators (iter/filter)**, breaking parity with `transfer_return_ownership` (see finding above).
5. ✓ **Fallible outside try** — `statement_dispatch.rs:396-404` emits the dedicated "requires an enclosing try block" PYCTX diagnostic before falling into the native protocol path. Covered by `fallible_python_context_outside_try_has_dedicated_diagnostic`.

**Independent audit:** HIR consumers of `HirWithItem` were updated consistently (effects.rs, nonlocal_support.rs, async_with.rs analyzers, snapshot tests, all codegen dispatch/validation/traversal paths). Codegen returns a hard `CodegenError` on `HirWithItemKind::Python` (wave 4 remains gated). `mark_moved_with_flow` refuses to move a name registered in `python_context_borrows`, catching nested-with-reentry and pass-into-`own` shapes. No user-triggerable panics; the `let else` on `HirWithItemKind::Python` is unreachable but safe. Nested with borrow save/restore in `previous_context_borrows` correctly rescopes shadowing.

**Blocking issue:** the ListComp/SetComp/DictComp generator-recursion gap is a real storage-escape that the wave-3 acceptance criteria's "covers comprehensions … parity with transfer_return_ownership" explicitly required.

VERDICT: NOT SATISFIED
