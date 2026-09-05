

Now I have a complete picture. Let me produce a comprehensive analysis with exact doc edits needed.

## Analysis and Recommendation

### 1. Semantic Shape for Async Generators and Async Comprehensions

**Async Generator type model:**

```sifr
# Core type
AsyncGenerator[T, E, R]

# where:
# - T = yielded item type (yield x produces x: T)
# - E = error channel (raise E maps to Err in the stream)
# - R = return type (return x sets final return; default None)
# - E: Never when no explicit error handling
# - R: None when no explicit return
```

**Why `AsyncGenerator[T, E, R]` instead of `AsyncIterator[Result[T, E]]`:** In Sifr, errors are a typed `Result` channel, not runtime exceptions to be caught at iteration boundaries. Separating the error channel from the yielded value keeps the stream clean (`yield T` not `yield Result[T, E]`) and lets the caller decide how to handle generator-originated failures — matching the same decision the caller makes with `Coroutine[T, E]`.

**End-of-stream:** `anext(agen)` returns `Option[T]` — `Some(value)` for yielded items, `None` after the generator's final return or after `GeneratorExit` is injected on cancellation. This mirrors sync iterator semantics. No sentinel value, no `StopIteration` exception.

**Cancellation cleanup:** When an `AsyncGenerator` is cancelled (its parent scope exits abnormally, or the generator is explicitly closed), the runtime injects `GeneratorExit` at the current suspension point. `GeneratorExit` is in the cancellation-control family — it is not catchable by `except Error`. The generator's `finally` blocks run, and then the generator terminates. This matches Sifr's active cancellation semantics exactly.

**Send/throw/close:** Defer `send()` and `throw()` to a future milestone. The generator protocol in v1 supports only `anext()` iteration, `aclose()` for cancellation-triggered cleanup, and explicit cancellation via scope exit. This keeps the first model simple: async generators are async iterators with cleanup.

**`yield` inside `async def`:** `yield` in an async function changes its type from `Coroutine[T, E]` to `AsyncGenerator[T, E, R]`. The compiler must enforce that `yield` is only valid inside `async def` bodies, that the yielded type is consistent across all `yield` statements, and that the generator's error channel is surfaced to callers.

---

### 2. Type Model Details

```sifr
# AsyncGenerator type declaration
class AsyncGenerator[T, E, R]:
    anext() -> Option[T]

# The protocol for async iteration
protocol AsyncIterator[T]:
    anext() -> Option[T]

# AsyncGenerator implements AsyncIterator
# anext() yields Option[T] — Some(T) for items, None when exhausted
# Cancellation injects GeneratorExit, runs finally blocks, then None

# Generator return type semantics
async def integers() -> AsyncGenerator[int, Never, None]:
    for i in range(10):
        yield i
    return  # explicit return, R = None

async def fetch_pages() -> AsyncGenerator[Page, NetworkError, Summary]:
    page: Page = await fetch_first()
    while page is not None:
        yield page
        page = await fetch_next(page)
    return Summary(completed=True)  # R = Summary

# anext() on a generator that raises:
async def possibly_failing() -> AsyncGenerator[int, DatabaseError, None]:
    try:
        row: Row = await db.query("SELECT ...")
        yield row.value
    except DatabaseError as e:
        raise DatabaseError(f"failed at row: {e}")  # E channel carries this
```

**Generator exit injection:** When `aclose()` is called or the generator is cancelled:
- Inject `GeneratorExit` at the current suspension point
- Run all `finally` blocks in order
- If the generator is suspended at a `yield`, the `yield` expression raises `GeneratorExit`
- After cleanup, `anext()` returns `None` on subsequent calls

**Error propagation in iteration:** There is no automatic error wrapping at the iterator boundary. If a generator raises through its error channel during `anext()`, the error propagates to the caller of `anext()` exactly as it would in an async function. Callers must handle `anext()` errors explicitly.

---

### 3. Interaction with Sifr Safety

**No exceptions:** `raise` in an async generator maps to the `E` error channel — it is not a Python-style exception that propagates through the call stack. The `yield`/`await` boundary is clean. A caller iterating over an async generator sees typed errors, not opaque exception objects.

**Cancellation cleanup:** `GeneratorExit` injection on cancellation ensures all `finally` blocks run. This is the same cleanup protocol as async context managers. `finally` blocks in async generators follow the LIFO order defined for `async with`. Cleanup failures become `SecondaryError` evidence attached to the cancellation result — never replacing the primary cancellation cause.

**Resource cleanup:** Async generators that hold resources (open connections, file handles) must use `finally` blocks or `async with` for deterministic cleanup. The compiler enforces no leaked resource patterns only if the generator's body is well-formed. There is no automatic `aclose()` on generator garbage collection in v1 — generators are cancelled only when their parent scope exits or when explicitly closed. This is consistent with Python's behavior but requires disciplined use of `finally`/`async with`.

**No panics:** No user code path in an async generator's generated Rust can panic. `yield` expressions compile to a state-machine that yields control without panicking. `anext()` calls that would raise are typed errors, not panics.

**Ownership/borrows across yield/await:** The same borrow rules apply inside async generators as in any other `async def`. Mutable borrows cannot remain live across `yield` or `await` points. This is checked by the type checker just like borrow across `await` in regular async functions. Async generators introduce an additional boundary: values captured across `yield` must be live through the generator's lifetime, which is bounded by the spawning scope. The compiler checks that captured values are not borrowed in ways that would outlive the generator's scope.

**Send/Sync task boundaries:** An `AsyncGenerator[T, E, R]` is `Send` when `T`, `E`, and `R` are all `Send` and the generator holds no non-send resources across suspension points. Since async generators suspend at `yield`, the compiler must ensure that any captured mutable state is either owned by the generator or protected by synchronization. In practice, most async generators are `Send` if their captured types are `Send` — the state machine itself is thread-safe.

---

### 4. Async Comprehensions

**In-scope for first model:**
- `async for` over async generators and async iterables is already in scope
- Async list comprehension: `[x async for x in agen]`
- Async set comprehension: `{x async for x in agen}`
- Async dict comprehension: `{k: v async for k, v in agen}`
- Async generator expression (explicit about async nature): `gen = (x async for x in agen)` — this is sugar over `async def gen(): ... yield ...`

**Out of scope for first model:**
- Async generator expressions used as arguments (e.g., `func(x async for x in agen)`) — this requires generator protocol support for delegation (`yield from`) which is deferred
- Nested async comprehensions (complex but secondary concern)
- Async comprehension with `if`/`else` clauses that include await — would require await-in-comprehension which adds parsing/HIR complexity

**Error propagation:** Async comprehensions follow the same error semantics as async generators. If the async iteration raises, it propagates to the comprehension's caller. There is no automatic suppression or swallowing.

**Cancellation:** If the surrounding task is cancelled during async comprehension evaluation:
1. The iteration is cancelled (same as cancellation of an `async for` loop)
2. All `finally` blocks in the async generator run
3. Cleanup errors become `SecondaryError` evidence
4. The comprehension result is abandoned (the variable assignment never completes)

**Sync vs async comprehension distinction:** The compiler must reject sync comprehensions inside `async def` bodies where the iterable is async (this is already caught by `async for` type checking). Async comprehensions are only valid in contexts where `async for` is valid.

---

### 5. Milestone Placement

**Recommendation: Sub-milestone `milestone_async_7b` inside `milestone_async_7`**

This is the right split because:

- `milestone_async_7` already covers "Async Context Managers, Async Iteration, and Resource Cleanup"
- Async generators are the natural extension of async iteration (you can't meaningfully iterate async without generator support)
- Async comprehensions are sugar over async generators — they land together
- Creating a new top-level milestone `milestone_async_9` adds an extra phase-gate and delays the feature unnecessarily
- The sub-milestone approach keeps the feature tracked without bloating the milestone count

**Sub-milestone structure within `milestone_async_7`:**

| Sub-milestone | Scope |
|---|---|
| `milestone_async_7a` | (existing) User-defined async context managers, `async with`, `async for` over channel/stream |
| `milestone_async_7b` | (new) Async generators and async comprehensions |

Both sub-milestones share the same depends-on chain: `milestone_async_5` and `milestone_async_6`. The sub-milestones can execute in parallel if needed, but 7a should close first as it establishes the iteration protocol that 7b builds on.

---

### 6. Exact Doc Edits

Here are the precise changes needed in each file.

#### A. `internal_docs/async_concurrency_model.md`

**Change 1:** Update the "Out Of Scope" section (line ~210-223). Remove async generators/comprehensions from out of scope and replace with the new model entries.

Replace lines 207-224:
```
### In Scope

- async syntax lowering
- awaitable type model
- runtime bootstrapping
- task handles
- scoped task groups
- cancellation and timeout semantics
- gather/select/race composition
- async context managers
- async iteration
- task-boundary ownership and Send/Sync checking
- explicit synchronization primitives
- explicit blocking/thread offload
- diagnostics for the model

### Out Of Scope

...

- async generators and async comprehensions in the first async model
```

With:
```
### In Scope

- async syntax lowering
- awaitable type model
- runtime bootstrapping
- task handles
- scoped task groups
- cancellation and timeout semantics
- gather/select/race composition
- async context managers
- async iteration
- async generators
- async comprehensions (list, set, dict forms)
- task-boundary ownership and Send/Sync checking
- explicit synchronization primitives
- explicit blocking/thread offload
- diagnostics for the model

### Out Of Scope

...

- async generator expression as function argument
- `yield from` delegation in async generators
- `send()` on async generators
- `throw()` on async generators
- multiprocessing
- process pools
```

**Change 2:** Add new type definitions after the existing type system section (after line ~270). Insert a new section "Async Generator Types":

```
## Async Generator Types

`AsyncGenerator[T, E, R]` is an async function body containing `yield`. It is not a `Coroutine` — `yield` changes the function's type. It is an async iterator that can be used in `async for`.

```sifr
# Type parameters:
# - T: yielded item type
# - E: error channel (Never when no explicit error handling)
# - R: return type (None when no explicit return)
AsyncGenerator[T, E, R]

# The async iterator protocol
protocol AsyncIterator[T]:
    anext() -> Option[T]

# AsyncGenerator[T, E, R] implements AsyncIterator[T]
```

Rules:
- `yield` in `async def` changes the function type from `Coroutine[T, E]` to `AsyncGenerator[T, E, R]`.
- `yield x` produces `x: T` where `T` must be consistent across all yield statements in the function.
- `raise E` in an async generator propagates through the `E` error channel — it is not a Python-style exception caught by the iteration boundary.
- `return x` sets the generator's final return value `R`. If absent, `R = None`.
- `anext()` returns `Option[T]`: `Some(value)` for yielded items, `None` after the generator terminates (final return or `GeneratorExit` injection).
- `GeneratorExit` is injected when the generator is closed or cancelled. It is in the cancellation-control family and is not catchable by `except Error`. All `finally` blocks run before the generator terminates.
- Cancellation of a parent scope aborts the async generator by injecting `GeneratorExit` and awaiting cleanup. Cleanup failures become `SecondaryError` evidence.
- `send()` and `throw()` on async generators are deferred to a future model amendment.
- `yield from` delegation is deferred.

Ownership and Send/Sync rules for async generators:
- Captured values must satisfy the same spawn-boundary rules as `scope.spawn`.
- An `AsyncGenerator[T, E, R]` is `Send` when `T`, `E`, and `R` are `Send` and no non-send state is held across suspension points.
- Mutable borrows cannot remain live across `yield` or `await` — enforced by the same borrow-across-await analysis used for regular async functions.

```sifr
# Examples
async def counter() -> AsyncGenerator[int, Never, None]:
    i: int = 0
    while i < 5:
        yield i
        i = i + 1

async def fetch_pages() -> AsyncGenerator[Page, NetworkError, Summary]:
    page: Page = await fetch_first()
    while page is not None:
        yield page
        page = await fetch_next(page.id)
    return Summary(completed=True)
```

## Async Comprehension Types

Async comprehensions desugar to async generator functions. The supported forms in the first model are:

- **List comprehension:** `result: list[int] = [x async for x in agen]`
- **Set comprehension:** `result: set[int] = {x async for x in agen]`
- **Dict comprehension:** `result: dict[str, int] = {k: v async for k, v in agen_pairs}`

Generator expression form is supported: `gen_expr = (x async for x in agen)` produces an `AsyncGenerator`. It must be consumed by `async for` or awaited to completion; an abandoned async generator expression leaks resources.

Error propagation: if the async iteration raises, the error propagates to the comprehension's enclosing context — matching the behavior of `async for`. Cancellation during comprehension evaluation cancels the iteration, runs cleanup, and abandons the comprehension result.

The following are deferred:
- Async generator expression as function argument (`func(x async for x in agen)`)
- `yield from` in async generators
- `send()` and `throw()` on async generators
```

**Change 3:** Update the "Async Resource Protocols" section (lines ~470-484). Change the last paragraph about async generators/comprehensions:

Original (line 484-485):
```
`async for` works for async iterable values such as channel-backed streams. Async generators and async comprehensions are separate future features.
```

Replace with:
```
`async for` works for async iterable values such as channel-backed streams and async generators.

Async generators (`async def` with `yield`) and async comprehensions are part of the async iteration model. Async generators implement the `AsyncIterator[T]` protocol via `anext()`. Async comprehensions desugar to async generator functions. `send()`, `throw()`, and `yield from` are deferred.
```

**Change 4:** Update the "Model Invariants" section (lines ~527-546). Add new invariants for async generators. Insert after invariant 16:

```
17. Async generators (`async def` with `yield`) have type `AsyncGenerator[T, E, R]`; `yield` in `async def` changes the function type from `Coroutine[T, E]` to `AsyncGenerator[T, E, R]`.
18. `anext()` on an async generator returns `Option[T]` — `Some(T)` for yielded items, `None` when the generator is exhausted (final return or `GeneratorExit` injection).
19. `GeneratorExit` is injected on generator close/cancellation; it is in the cancellation-control family and is not catchable by `except Error`; all `finally` blocks run before termination.
20. Async generator error channels (`E`) propagate through the `E` type — errors are not wrapped or swallowed at the iteration boundary.
21. Async comprehensions desugar to async generator functions; supported forms are list, set, and dict comprehension. Generator expressions produce `AsyncGenerator` and must be consumed.
22. `send()`, `throw()`, `yield from`, and async generator expressions as function arguments are deferred to a future model amendment.
```

---

#### B. `internal_docs/phases/32_async_ecosystem.md`

**Change 1:** Update the "Non-Goals And Deferrals" section (lines 38-58). Remove async generators/comprehensions from the deferral list.

Change line 53 from:
```
- async generators and async comprehensions
```
to:
```
- `send()` and `throw()` on async generators
- `yield from` delegation in async generators
- async generator expression as function argument
```

**Change 2:** Update "Locked V1 Decisions" (around line 70-90). Add new locked decisions for async generators after decision 18:

Insert after decision 18 (around line 90):
```
19. `async def` with `yield` produces `AsyncGenerator[T, E, R]` instead of `Coroutine[T, E]`; `yield` is valid only inside `async def` bodies.
20. `anext()` on `AsyncGenerator` returns `Option[T]` — `Some(T)` for yielded items, `None` when exhausted. There is no `StopIteration` exception or sentinel value.
21. `GeneratorExit` is injected on generator close or cancellation; it is in the cancellation-control family, not catchable by `except Error`; all `finally` blocks run before termination.
22. Async generator error channels propagate errors as typed `E` values — errors are not wrapped at the iteration boundary. Callers must handle `anext()` errors explicitly.
23. Async comprehensions desugar to async generator functions; list, set, and dict forms are in scope; async generator expressions produce `AsyncGenerator` and must be consumed; async generator expressions as function arguments are deferred.
24. `send()`, `throw()`, `yield from`, and async generator expressions as function arguments are deferred.
```

**Change 3:** In `milestone_async_7` (lines 569-625), split into sub-milestones. The existing `milestone_async_7` scope becomes `milestone_async_7a`. Add a new `milestone_async_7b` after it.

The existing `milestone_async_7` should be renamed to `milestone_async_7a: User-Defined Async Context Managers, Async Iteration, and Resource Cleanup` with its scope updated to clarify that async generators and async comprehensions are moved to 7b.

Replace the `milestone_async_7` section with:

```
### milestone_async_7a: User-Defined Async Context Managers, Async Iteration, and Resource Cleanup

status: proposed

**Goal:** Complete general user-defined async control-flow protocols without dragging in broad ecosystem APIs.

**Depends on:** `milestone_async_5` and `milestone_async_6`

**Scope:**

- Generalize `async with` beyond the built-in `task.scope()` form from `milestone_async_1`.
- Define and enforce the user-defined async context-manager protocol.
- Implement async iterable protocol.
- Implement `async for` over async iterables (channels, streams, generators).
- Define cancellation cleanup behavior for async context managers:
  - cleanup order is LIFO,
  - cancelling inside `async with` unwinds active async context managers,
  - async exit receives the cancellation cause,
  - async exit runs to completion unless the runtime is forcefully aborted,
  - errors from async exit during cancellation become `SecondaryError` evidence attached to the owning scope result,
  - panic-like failures from async exit are caught at the runtime/codegen boundary and surfaced as secondary errors,
  - parent cancellation triggers child cancellation, but each task unwinds its own cleanup independently.
- Define channel-backed async iteration.
- Define async generator iteration (async generators implement `AsyncIterator[T]` via `anext()`).

**Definition of done:**

- `async with` calls async enter/exit protocol methods correctly.
- Async resource cleanup runs under cancellation.
- If cleanup fails during cancellation, the original cancellation remains primary and cleanup failure is secondary evidence.
- `SecondaryError` never masks the primary result.
- Async exit cleanup order is LIFO.
- Panic-like failures in async exit do not become process-terminating double-panic paths.
- Nested cancellation is deterministic.
- `async for` works for channel/stream-like values and async generators.
- Non-async iterables are rejected in `async for`.
- Async comprehensions are sugar over async generator functions (covered in 7b).

**Positive validation:**

- `async_with_basic.sifr`
- `async_with_cancel_cleanup.sifr`
- `async_with_nested_cleanup_order.sifr`
- `async_for_channel.sifr`
- `async_for_stream_result.sifr`
- `async_for_generator.sifr` (covered in 7b)

**Negative validation:**

- `async_with_missing_protocol_rejected.sifr`
- `async_for_non_async_iterable_rejected.sifr`
- `async_resource_cleanup_error_typed.sifr`
- `async_with_cleanup_panic_secondary.sifr`

**Demo:**

- `demos/m32_async_resource_demo.sifr`

---

### milestone_async_7b: Async Generators and Async Comprehensions

status: proposed

**Goal:** Bring async generators and async comprehensions into the first async model as first-class iteration primitives.

**Depends on:** `milestone_async_7a`

**Scope:**

- Define `AsyncGenerator[T, E, R]` type:
  - `T` = yielded item type
  - `E` = error channel (Never when no explicit error handling)
  - `R` = return type (None when no explicit return)
  - `yield` in `async def` changes function type from `Coroutine[T, E]` to `AsyncGenerator[T, E, R]`
  - `yield x` produces `x: T`
  - `raise E` propagates through the `E` error channel
  - `return x` sets the generator's final return value `R`
- Implement `AsyncIterator[T]` protocol for `AsyncGenerator[T, E, R]`:
  - `anext()` returns `Option[T]`: `Some(value)` for yielded items, `None` when exhausted
  - no `StopIteration` exception, no sentinel value
- Implement `GeneratorExit` injection:
  - injected on generator close or cancellation
  - in the cancellation-control family; not catchable by `except Error`
  - all `finally` blocks run before termination
  - cleanup failures become `SecondaryError` evidence
- Implement generator state machine in codegen:
  - suspension at `yield` preserves local state
  - resume from suspension point on `anext()`
  - clean termination on final return or after `GeneratorExit` cleanup
- Define borrow/across-yield rules:
  - mutable borrows cannot remain live across `yield` or `await`
  - same analysis as borrow across await in regular async functions
- Define Send/Sync rules for async generators:
  - `AsyncGenerator[T, E, R]` is `Send` when `T`, `E`, `R` are `Send` and no non-send state is held across suspension points
  - capture validation uses the same spawn-boundary rules
- Implement async comprehensions:
  - list: `[x async for x in agen]`
  - set: `{x async for x in agen}`
  - dict: `{k: v async for k, v in agen_pairs}`
  - desugar to async generator functions
  - error propagation through the async iteration channel
  - cancellation during comprehension evaluation cancels iteration, runs cleanup, abandons result
- Implement async generator expression:
  - `gen_expr = (x async for x in agen)` produces `AsyncGenerator`
  - must be consumed by `async for` or awaited to completion
  - abandoned async generator expression is a compile-time warning (resource leak risk)
- Deferred surfaces (documented, negative tests):
  - `send()` on async generators
  - `throw()` on async generators
  - `yield from` delegation
  - async generator expression as function argument
- Add diagnostics for:
  - `yield` outside `async def`
  - `yield` in a non-generator async function (after `return` or in incompatible context)
  - inconsistent yield types within one async generator
  - abandoned async generator expression warning

**Definition of done:**

- `yield` in `async def` produces `AsyncGenerator[T, E, R]` and is type-checked correctly.
- `anext()` returns `Option[T]` — no `StopIteration` exception in generated Rust.
- `GeneratorExit` injection on cancellation runs all `finally` blocks; cleanup failures are secondary evidence.
- Async generator errors propagate through the `E` channel, not as exceptions at the iteration boundary.
- Borrow across `yield` is rejected the same way borrow across `await` is rejected.
- Async generator Send/Sync is validated at the spawn boundary.
- Async list/set/dict comprehensions desugar to async generator functions and work.
- Async generator expression produces `AsyncGenerator` and is consumed or warned against.
- Deferred surfaces (`send`, `throw`, `yield from`) are documented and have negative/waiver tests.
- No new user-triggerable panic paths in generated async generator code.

**Positive validation:**

- `async_generator_basic.sifr`
- `async_generator_yield_types.sifr`
- `async_generator_return_value.sifr`
- `async_generator_anext_option.sifr`
- `async_generator_cancel_cleanup.sifr`
- `async_generator_borrow_yield.sifr`
- `async_generator_send_boundary.sifr`
- `async_comprehension_list.sifr`
- `async_comprehension_set.sifr`
- `async_comprehension_dict.sifr`
- `async_generator_expression.sifr`

**Negative validation:**

- `yield_outside_async_def_rejected.sifr`
- `yield_in_sync_generator_rejected.sifr`
- `async_generator_inconsistent_yield_types_rejected.sifr`
- `async_generator_send_not_supported.sifr`
- `async_generator_throw_not_supported.sifr`
- `yield_from_async_not_supported.sifr`
- `async_generator_expr_not_consumed_warning.sifr`

**Demo:**

- `demos/m32_async_generator_demo.sifr`

---

```

**Change 4:** Update the milestone dependency graph (lines ~703-725) to show the sub-milestone relationship:

```
    m7a["m32.7a Async Context Managers + Iteration"]
    m7b["m32.7b Async Generators + Comprehensions"]

    m5 --> m7a
    m6 --> m7a
    m7a --> m7b
    m7b --> m8
```

And update the "Implementation order" section (lines ~727-736):

Replace the `milestone_async_7` reference with:
```
- `milestone_async_7` is split into 7a (context managers + iteration) and 7b (generators + comprehensions).
- `milestone_async_7a` closes before `milestone_async_7b` begins, but both must close before `milestone_async_8`.
- The dependency chain is: `milestone_async_0` → ... → `milestone_async_6` → `milestone_async_7a` → `milestone_async_7b` → `milestone_async_8`.
```

**Change 5:** Update the "Exit Gate" section (lines ~824-842). Add to the exit criteria list:

After the line `async with` and `async for` work for protocol-conforming values, add:
```
- async generators (`yield` in `async def`) work with correct type semantics
- `anext()` returns `Option[T]` with no `StopIteration` exception
- `GeneratorExit` injection on cancellation runs cleanup; cleanup failures are secondary evidence
- async comprehensions (list, set, dict) desugar to async generator functions
- deferred surfaces are documented with negative/waiver tests
```

---

#### C. `internal_docs/architecture.md`

**Change:** Update the "Type System Design" section (lines ~832-915) to add `AsyncGenerator` and `AsyncIterator` to the Type enum. Insert after the async/concurrency model types (after `AsyncFunction`):

```rust
// Async generators and iteration (Phase 32, milestone_async_7b)
AsyncGenerator(Box<Type>, Box<Type>, Box<Type>), // AsyncGenerator[T, E, R]
AsyncIterator(Box<Type>),                        // AsyncIterator[T] -- anext() -> Option[T]
```

And add to the "Async/concurrency model" comment:

```
// Async/concurrency model (Phase 32)
// Coroutine, Task, TaskResult, Awaitable, AsyncFunction (milestone_async_1-4)
// AsyncIterator, AsyncGenerator (milestone_async_7a, 7b)
```

**Change 2:** In the "Cross-cutting Contracts" section, contract #8 "Concurrency Safety" (lines ~662-679), add a note about async generators:

In the table or following text, after the existing concurrency rules, add:

```
- **Async generators:** `async def` with `yield` produces `AsyncGenerator[T, E, R]`, not `Coroutine`. `anext()` returns `Option[T]` — `Some(T)` for items, `None` when exhausted. `GeneratorExit` is injected on close/cancellation; it is in the cancellation-control family and is not catchable by `except Error`. Errors from the generator's `E` channel propagate through `anext()` calls — callers handle errors explicitly. Mutable borrows cannot remain live across `yield`. `AsyncGenerator[T, E, R]` is `Send` when `T`, `E`, `R` are `Send` and no non-send state crosses suspension points. `send()`, `throw()`, and `yield from` are deferred.
```

---

### 7. Risks and Gaps

| Risk | Source | Mitigation |
|---|---|---|
| **Generator state machine complexity** | Rust async generators are unstable (`gen` keyword); Tokio doesn't have first-class `AsyncGenerator` — must implement as manual state machine | Use a struct-based state machine in generated Rust (stable, tested pattern). Mirror the sync generator approach already validated in the codebase. Do NOT rely on nightly `gen` blocks. |
| **Cancellation cleanup is async** | Async generators need to run `finally` blocks which may contain `await`. `GeneratorExit` injection must be async-safe. | The generator state machine must support async cleanup during `aclose()`. The `GeneratorExit` injection is an async operation that awaits all cleanup before terminating the generator. |
| **`anext()` returning `Option[T]` breaks Python's `StopIteration` convention** | Python's `__anext__` raises `StopAsyncIteration` to signal exhaustion. Sifr's `Option[T]` is different. | This is an intentional Sifr divergence for safety. Document it clearly. CPython parity tests for async iteration should use the `async for` protocol which hides the `Option[T]` surface. The `Option[T]` return is only visible at the protocol level. |
| **Abandoned async generator expressions** | `func(x async for x in agen)` creates an `AsyncGenerator` that is never consumed if `func` doesn't iterate it. | Emit a warning at compile time when an async generator expression is not consumed (not used in `async for`, not awaited, not stored in a typed variable that will be consumed). This is a lint-level concern in v1. |
| **Send/Sync across suspension points** | Async generators hold state across `yield` points. The captured state must remain valid across suspension. | The compiler's borrow-across-await analysis already handles this. Add explicit checks that captured values do not hold non-send state across `yield`. The state machine itself must be `Send`. |
| **Async comprehensions with `if`/`else` clauses** | Python's async comprehensions support `if` filtering and conditional expressions. Sifr's first model can defer these. | Only support the basic `async for` source clause in v1. Filter clauses (`if`) work the same as sync comprehensions. Conditional expressions inside comprehensions (`x if cond else y async for x in agen`) require await-in-comprehension and should be deferred. |
| **Generator type vs coroutine type confusion** | Users might expect `async def foo(): yield x` to still be awaitable like a coroutine. | The compiler must reject `await foo()` when `foo` is an async generator (it is not an `Awaitable`). `async for` is the only valid consumption path. Diagnostics must be clear: "async generator cannot be awaited; use `async for` instead." |
| **Nested async generators** | `async def outer(): yield inner()` where `inner` is also an async generator. Works in Python but needs careful state machine implementation. | Test nested async generator scenarios. The state machine should handle nesting correctly as long as each generator is properly consumed. |
| **Memory lifetime of generator state** | Generator state is held in the struct between `yield` calls. If the generator is stored in a long-lived variable, the captured state is also long-lived. | The compiler already handles ownership for async functions. Add specific checks for generator capture lifetimes — a generator stored in a variable holds its captured state until it is exhausted or closed. |
| **CPython parity gap** | Python's `async for` uses `StopAsyncIteration` exception for exhaustion. Sifr uses `Option[T]`. Some CPython async iterator tests may not translate directly. | Map CPython's `StopAsyncIteration` exhaustion signals to `None` in parity tests. The `async for` protocol is the same; only the underlying `anext()` return type differs. |

---

### 8. Verdict

**SATISFIED with current docs, as amended by the exact changes above.**

The current docs are well-structured and the model is sound. The deferral of async generators and async comprehensions was an intentional early decision — the model now has enough foundation (async iteration, cancellation, cleanup) to safely add them. The exact edits above:

1. Move async generators and async comprehensions from "Out of Scope" to "In Scope"
2. Add the complete type model with `AsyncGenerator[T, E, R]`, `AsyncIterator[T]`, end-of-stream via `Option[T]`, `GeneratorExit` semantics
3. Add async comprehension support (list, set, dict) and clarify generator expression semantics
4. Split `milestone_async_7` into 7a (context managers + iteration) and 7b (generators + comprehensions)
5. Add all locked decisions for v1 semantics
6. Add invariants 17-22 to the model contract
7. Update the architecture doc with the new types and concurrency safety rules
8. Document all deferred surfaces with clear rationale and negative tests

Once these edits are applied, the docs will have a complete, implementation-ready plan for async generators and async comprehensions in the first async model. The implementation can proceed with confidence.
