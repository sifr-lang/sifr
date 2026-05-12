

**SATISFIED**

**Rationale:**

The fixture is correct and well-scoped. Here's my analysis:

**What the fixture covers:**
1. `async def lengths(items: list[int])` — an async generator taking an immutable borrowed move-type parameter
2. `yield len(items)` and `yield len(items) + 1` — immutable borrows across yield suspension points
3. Caller checks that the generator yields derived values (3, then 4)
4. Caller verifies `len(items) == 3` after full generator consumption — the caller's list remains usable

**Alignment with contract:**

| Contract requirement | Covered by fixture |
|---|---|
| Immutable borrowed move-type parameter across yield | `items: list[int]` parameter, `len(items)` reads |
| Yield derived values | `len(items)`, `len(items) + 1` |
| Caller's list usable after generator consumption | `assert len(items) == 3` |
| `AsyncGenerator[T, E]` type signature | `-> AsyncGenerator[int, GeneratorCloseError]` |
| `anext()` exhaustion protocol | Third `anext` returns `Ok(None)` |

**Complementary negative validation:**

`async_generator_mut_borrow_across_yield_rejected.sifr` covers the opposite: `mut items: list[int]` with `# expect-error: SIFR-OWN-0009`. The positive fixture is the necessary counterpart proving immutable borrows are allowed.

**No overclaim:**

The fixture does not claim `await` inside the generator, non-`None` return values, cancellation cleanup, `send()`, `throw()`, `yield from`, or lazy state-machine lowering. It stays within the current supported surface.

**Assertions match ownership semantics:**

`assert len(items) == 3` correctly verifies list usability post-consumption, consistent with Phase 10 borrow-by-default semantics and the model invariant that immutable borrows leave the referent intact.

The fixture is a correct and useful Phase 32 slice — ready for PR.
