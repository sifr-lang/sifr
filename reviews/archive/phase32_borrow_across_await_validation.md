Now I have a complete picture of the implementation. Let me compile my review findings.

---

## Review: Phase 32 Borrow-Across-Await Validation Slice

### Files Reviewed

| Area | Files | Status |
|------|-------|--------|
| Diagnostic code | `crates/sifr_diagnostics/src/codes.rs` | ✓ |
| Diagnostic docs | `docs/errors/SIFR-OWN-0009.md`, `docs/errors/diagnostic-codes.md` | ✓ |
| Internal docs | `internal_docs/diagnostic_codes.md` | ✓ |
| HIR lowering | `crates/sifr_hir/src/lower/typing_and_functions.rs` | ✓ |
| Diagnostic emission | `crates/sifr_hir/src/lower/ownership_diagnostics.rs` | ✓ |
| Unit tests | `crates/sifr_hir/src/lower/expressions_tests.rs` | ✓ |
| E2E tests | `await_without_live_borrow.sifr`, `borrow_across_await_rejected.sifr` | ✓ |
| Phase tracker | `internal_docs/phases/32_async_ecosystem.md` | ✓ |

### Correctness Assessment

**Diagnostic registration** (`codes.rs`): `OWN_BORROW_ACROSS_AWAIT` is correctly declared as `Error` severity, uses `binding` as the declared/dedupe arg, points to the negative fixture, and is included in `ACTIVE_DIAGNOSTIC_CODES`. All validation tests pass.

**Await scanner** (`first_await_range_in_stmts`/`first_await_range_in_expr`): Syntactic recursive descent over all statement types (expr, return, assign, if/elif/else, while, for, with, try/except/finally) and all meaningful expression positions. The `_ => None` fallthrough for remaining AST nodes (function defs, lambdas, class defs, match, etc.) is correct — those are separate analysis scopes.

**Diagnostic trigger** (`lower_function` lines 1086–1101): Correctly checks `func.is_async && await_exists` before any body lowering. Only flags parameters where `is_mut_borrow() && ownership == Move && !TypeVar`. This precisely targets the design intent: mutable borrowed Move-type parameters that cannot safely suspend.

**Positive case** (`await_without_live_borrow.sifr`): Sync function `mutate_local` takes ownership via function call, borrow completes before `await`. Lowering succeeds because the parameter isn't a borrowed param — it's consumed by the call. This correctly validates the "completed same-task mutable borrows can be followed by await" design rule.

**Negative case** (`borrow_across_await_rejected.sifr`): `mut items: list[int]` is a borrowed mutable Move-type param in an async function with an await. Correctly emits SIFR-OWN-0009 at the `await` with the binding name and source span.

**Unit tests** (`expressions_tests.rs`): Two tests cover the slice — one verifying the diagnostic fires at the first await with correct code and range, one verifying the positive case lowers without error.

**Design alignment**: The borrow rule table in `async_concurrency_model.md` says "mutable borrow: rejected when it would remain live across `await`". The implementation correctly rejects at the parameter level, which is the conservative v1 approach. Immutable borrows and owned values are not flagged, matching the table.

### Observations

1. **Local borrows not checked (intentional)**: `items[0]` (index access) or `items.foo` (field access) on a local variable followed by `await` is not flagged — only parameter-level borrowed params are checked. This matches the design's focus on "borrowed parameter" escape scenarios. The HIR ownership tracker (`is_mut_borrowed`) could theoretically be used for a deeper analysis, but that would be a separate slice.

2. **One error per param per await point**: If multiple params violate the rule, each gets its own diagnostic at the same `await_range`. The dedupe key is `binding`, so repeated violations for the same param would be deduplicated. This is appropriate behavior.

3. **Template vs. implementation message**: The registry template is `"mutable borrow {binding} cannot cross await"` (minimal), while the actual emitted message adds a user-friendly suggestion: `"mutable borrow \`{name}\` cannot cross await; finish the mutation before awaiting or transfer ownership with \`own\`"`. The suggestion is good UX. Template and implementation are internally consistent.

4. **Phase tracker update**: The `implementation_progress` section for `milestone_async_4` correctly records the in-progress borrow-across-await validation slice.

5. **No HIR range loss**: The check runs on the raw AST (before HIR lowering), so the `await_range` from the parser is preserved precisely. This is the intentional design: "diagnostic can point at the await before HIR range loss."

### Missing Tests (Minor)

No test covers the case where a function has **both** an immutable borrow param AND a mutable borrow param, only the mutable one should error. Not critical for this slice but worth a note for completeness.

### Summary

The slice is well-scoped, correctly implements the design contract, and has complete diagnostic code/docs/HIR/test integration. The syntactic scanner tradeoff (simplicity over deep lifetime analysis) is the right choice for v1 milestone coverage.

**SATISFIED**
