

All five review dimensions check out. Here's my assessment:

**1. Rule enforcement for async generator yield suspension** ✓
The implementation gates on `is_async_generator` (which requires `func.is_async && function_body_contains_yield(...)`) and filters params by `convention.is_mut_borrow() && param.ty.ownership() == OwnershipKind::Move`, excluding TypeVars. This correctly mirrors the existing `mutable_borrow_across_await` path.

**2. Scope correctness (async generators only)** ✓
- Sync generators: no `async def` → no trigger. Verified: `def sync_stream(mut items: list[int])` passes.
- Async functions without yield: `is_async_generator` is false → no trigger. Verified: `async def process(mut items: list[int])` passes.
- Shared borrows (non-`mut`): `is_mut_borrow()` returns false → no trigger. Verified: `async def stream(items: list[int])` passes.

**3. Diagnostic code/message/range** ✓
Uses existing `DiagnosticCode::OWN_BORROW_ACROSS_AWAIT` (SIFR-OWN-0009), which is already in `codes.rs` and `ACTIVE_DIAGNOSTIC_CODES`. Message clearly distinguishes "async generator yield" from the await case. Range points to the `yield` token.

**4. AST yield-range traversal** ✓
`first_yield_range_in_stmt` mirrors `first_await_range_in_stmt` structurally, covering all statement types (expr, return, assign, if/elif/else, while, for, with, try/except/finally). `first_yield_range_in_expr` handles yield, yield-from, and all container expressions (call, attribute, subscript, binop, boolop, compare, if-expr, list/tuple/set/dict). No `.unwrap()` or `panic` paths — uses `find_map` with `None` defaults throughout. Handles nested control flow correctly (verified with `if True: yield len(items)` case).

**5. Fixture and review artifacts** ✓
E2E fail fixture `async_generator_mut_borrow_across_yield_rejected.sifr` is untracked and properly formed with `# expect-error: SIFR-OWN-0009`. All review artifacts in `reviews/` remain untracked.

Local validation passed, and my interactive spot-checks confirm the rule is scoped correctly.

SATISFIED
