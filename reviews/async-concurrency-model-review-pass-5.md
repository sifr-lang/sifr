

Now let me do the adversarial pass-5 review. I've read all three documents against the pass-4 review and the updated model.

Verdict: Blockers found.

---

## Pass-5 Review: Deep Adversarial Analysis

### Resolved from Pass-4

All three prior blockers are now closed:
- ✅ **B1 (type system)**: `Type::Task`, `Type::Awaitable`, `Type::AsyncFunction` are in architecture.md §Type System (lines 868-870).
- ✅ **B2 (HIR + roadmap sync)**: HIR additions are explicitly enumerated (lines 217-225). Phase 32 roadmap is marked superseded with a pointer to the model.
- ✅ **B3 (cancellation during async cleanup)**: Lines 748-755 define LIFO cleanup order, cancellation during `__aexit__`, panic/catch_unwind boundary, and nested cancellation. This is complete.

### New Blockers Found

---

**B1: Async function type can be called synchronously — no enforcement**

`internal_docs/async_concurrency_model.md` defines `AsyncFunction[Params, T, E]` as "distinct from sync `Callable` with the same parameters" (line 245), but does not specify the subtyping relationship. In particular:

> Can you store an `AsyncFunction` in a variable of type `Function`? Can you call it without `await`?

The model is silent. In Python asyncio, `asyncio.create_task()` guards against this. In Sifr, if `AsyncFunction` is a subtype of `Function`, a user could write:

```python
async def fetch() -> Result[str, Error]: ...

# Function variable holds async function
fn: Function = fetch  # is this allowed?

# Synchronous call — what happens?
result: str = fn()  # calls async function without await
```

Rust emits `Future not awaited`, which is the sharp edge. But Sifr's guarantee is "if it compiles, it works" — an async function called without `await` should be a compile error, not a silent bug.

**Recommended fix** — add to milestone_async_0 Type System rules:

> **No sync calling of async functions.** `AsyncFunction` is NOT a subtype of `Function`. Storing an `AsyncFunction` in a `Function`-typed variable is a compile-time error. An async function must be either:
> - awaited (within an `async def` body): `result = await fetch()`
> - spawned (inside a scope): `scope.spawn(fetch)` which returns a `Task`
>
> The only way to call an async function is through the async call path that produces a `Task[T, E]`.

---

**B2: `Task[T, E]` error type parameter has no constraint, violating result semantics**

The model specifies "await `Task[T, E]` always produces `Result[T, E]`" (line 251). Sifr's error system requires that the `E` in `Result[T, E]` must be a class extending `Error` (architecture.md line 419). But `Task[T, E]` has no such constraint.

This allows:

```python
Task[int, str]      # error type is str — not a Sifr Error
Task[int, int]       # error type is int — not a Sifr Error
Task[int, None]      # error type is None — not a Sifr Error
```

If you `await` these, you get `Result[int, str]`, `Result[int, int]`, `Result[int, None]` — none of which satisfy the error hierarchy requirement. The expression type of `await` is `Result[T, E]`, but `E` is unconstrained.

**Recommended fix** — add to milestone_async_0 Type System rules:

> **`Task[T, E]` requires `E: Error`.** The error type parameter of `Task` must be a class that extends `Error`, matching the constraint on `Result[T, E]`. This ensures that `await Task[T, E]` always produces a valid `Result[T, E]` with a proper error type.
>
> `Task[T]` (shorthand for `Task[T, Never]`) is valid: `Never` represents the absence of errors, not an invalid error type.
>
> Explicit `Task[T, SomeError]` where `SomeError` does not extend `Error` is a compile-time type error.

---

**B3: Lock guard across `await` enforcement mechanism is unspecified**

The model says "lock guards must not cross `await` points in v1" (line 383) and "crossing `await` with a live lock guard is a compile-time diagnostic" (line 384). But the enforcement mechanism is never defined.

In Rust, `tokio::sync::Mutex` allows `.lock().await` — the guard stays live across an await. To prevent this, Sifr must either:

1. Use a sync `Mutex` (not async-aware), so `.lock()` returns a `MutexGuard` that cannot be held across `.await`
2. Emit a compile-time lint that checks for live `MutexGuard` variables across await points in HIR

The model does not specify which approach. This matters because:
- If Sifr uses sync `Mutex`, then `.lock()` blocks the async thread (but not the scheduler). This is correct for v1.
- If Sifr uses `tokio::sync::Mutex`, then the guard CAN cross await, and the compiler must emit a diagnostic. The enforcement logic must be explicitly described.

**Recommended fix** — add to milestone_async_5 work items:

> **Lock implementation for v1:** `sifr.sync.Lock[T]` uses a synchronous mutex (`std::sync::Mutex`) internally, not an async-aware mutex. The `.lock()` call is not `.await`-aware and returns a guard that cannot cross an `await` point. This makes the lock guard safety a property of the type system, not a lint.
>
> If the implementation team determines that sync mutex blocking is unacceptable for the scheduler thread, they may use `tokio::sync::Mutex` with an explicit HIR-level lint pass that tracks live lock guard bindings across await points. In that case, the lint implementation details must be documented in milestone_async_5 before HIR work begins.
>
> The lock guard diagnostic for "live guard across await" is defined as:
> - **Diagnostic family:** `SIFR-ASYNC-000X` (async safety)
> - **Message:** "lock guard is still live at this await point — lock guards cannot cross await points in v1"
> - **Help:** "consider releasing the lock before the await, or use a channel to communicate results instead"

---

**B4: `await` expression type requirement is not in milestone_async_1 work items**

The model correctly requires that `await x` is valid only when `x` has an awaitable type (line 250). But milestone_async_1 work items (lines 421-431) never mention adding a work item for the type-system rule itself. The work items cover syntax and HIR lowering, but the type-level rule that "await requires awaitable type" is absent.

Without this explicit work item, an implementation team could pass milestone_async_1验收 without the type-system rule being checked. The rule exists in the Type System section but is not a milestone work item.

**Recommended fix** — add to milestone_async_1 work items:

> **Add await type-check rule.** The type checker must reject `await x` when `x` does not implement `Awaitable[T]`. The rule is: `await x` is valid only when `x: Awaitable[T]` for some `T`. The result type of the await expression is `T`.
>
> **Add structural protocol implementation rule.** Any type that provides the `.await()` method returning an awaitable result implements `Awaitable[T]` structurally. The type system does not require nominal conformance — structural implementation is sufficient.

---

**B5: `await` auto-unwrapping inside `try` blocks is inconsistent with the expression type rule**

The model states two things that conflict:
1. "await `Task[T, E]` always produces `Result[T, E]`" — this is the expression type (line 251)
2. "Inside a `try` block, that `Result[T, E]` follows existing auto-unwrap semantics" — the result is unwrapped inside `try` (line 251)

But it never defines what happens when you write:

```python
async def main():
    try:
        x = await spawn_critical_task()  # type: Task[str, NetworkError>
        print(x)  # x is str or error was raised
    except NetworkError as e:
        print(f"failed: {e}")
```

The model says `await Task[T, E]` produces `Result[T, E]`. In a `try` block, auto-unwrap says the `Result` is unwrapped and if it's `Err`, the exception handler runs. But `Task[T, E]` error type `E` is `NetworkError`. So the auto-unwrap produces `str` inside the `try`, and the `except NetworkError` runs if there's an error.

This is correct. But the model never explicitly says: **the auto-unwrap applies to the `Result[T, E]` that `await` produces, not to the `Task[T, E]` itself.** A reader might think `Task[T, E]` is auto-unwrapped (it is not — `Task` is a handle, not a result).

**Recommended fix** — add to milestone_async_0 Type System rules:

> **Auto-unwrap applies to the `Result`, not the `Task`.** When `await task_handle` appears in a `try` block where `task_handle: Task[T, E]`:
>
> 1. `await` first produces `Result[T, E]` from the task handle
> 2. Auto-unwrap then applies to the `Result[T, E]`
> 3. If `Err(e)`, control transfers to the matching `except` arm
> 4. The `except` arm's type is `E` (the error type), not `Result[T, E]`
>
> This is equivalent to `let result: Result[T, E] = await task_handle; try { result? }` in Rust.

---

**B6: `async with task.scope()` pattern — is `scope` an async context manager?**

The primary model example (`async_concurrency_model.md` lines 33-34) shows:

```python
async with task.scope() as scope:
    first = scope.spawn(fetch_one("https://example.com/a"))
```

The `task.scope()` returns something that is used with `async with`. So `task.scope()` must return an async context manager. The model says `TaskScope` is a defined type (line 327), but it never says whether `TaskScope` implements `__aenter__`/`__aexit__` or if `task.scope()` returns a separate wrapper type.

This matters for implementation: if `TaskScope` itself is the async context manager, then `scope: TaskScope` after `async with task.scope() as scope`. If it's a wrapper, the wrapper type is unnamed.

**Recommended fix** — add to milestone_async_0 cancellation policy:

> **`task.scope()` returns an async context manager.** The call `task.scope()` returns an object that implements `__aenter__` and `__aexit__` asynchronously. The returned type is the `TaskScope` — it is both the scope container and the async context manager. Inside the `async with` block, the `scope` variable holds the `TaskScope` instance.
>
> `TaskScope.__aexit__` handles deterministic cleanup: it waits for all child tasks to complete, or cancels any remaining children if the scope exits abnormally. The scope must not be used outside its `async with` block.

---

### Non-Blocking Refinements

**N-1: `gather` cancellation behavior should be specified**

`gather` preserves input ordering (line 535), but when some tasks fail, what happens? Python's `asyncio.gather` has `return_exceptions=False` (fails on first error) and `return_exceptions=True` (collects all). Sifr's model doesn't address this.

**N-2: Async entrypoint bootstrap for `async def main()` should be explicit**

The model says "async entrypoints auto-bootstrap the runtime" but never explicitly says `async def main() -> Result[None, Error]` works as the entrypoint with automatic Tokio runtime. This is the canonical use case — it should be explicit.

**N-3: `TaskScope` and `TaskGroup` distinction is unclear**

`TaskScope` (line 327) and `TaskGroup` (line 76) are both mentioned but never distinguished. Looking at the primary example, `task.scope()` returns a scope used with `scope.spawn(...)`. TaskGroup appears to be a structured collection for `task.gather`. The distinction should be explicit.

**N-4: `asyncio.run()` compatibility mapping may be unnecessary**

The compatibility mapping says `sifr.asyncio.run(fn)` maps to "direct async entrypoint bootstrap" (line 822). But if `async def main()` works directly with `sifr run`, why would a user call `sifr.asyncio.run(fn)`? This compatibility API might be unnecessary.

---

### Cross-Document Contradiction Check

| Conflict | Status |
|---|---|
| Phase 32 roadmap vs. model milestone count (4 vs. 9) | ✅ Resolved — roadmap marked superseded |
| subprocess/signal in roadmap vs. out-of-scope in model | ⚠️ Not resolved — "subprocess and signal are not Phase 32 exit criteria unless this model is amended" (line 349) is a non-answer. Decide: yes or no. |
| HIR node enumeration | ✅ Present in model |
| `Type::Task`, `Type::Awaitable`, `Type::AsyncFunction` | ✅ Present in architecture.md |

---

### Comparison with Reference Languages

| Language | Sharp edge | Model avoidance | Residual risk |
|---|---|---|---|
| Python asyncio | `await` outside async, non-awaitable await, cancellation leakage | Structured concurrency, typed cancellation | B4 (await type rule not in milestone work items) |
| Rust/Tokio | Send/Sync errors leak as raw rustc errors | Sifr-native diagnostics, lock guard across await | B3 (enforcement mechanism unspecified) |
| Go | Goroutine leaks, no cancellation result | Structured concurrency default | N-1 (gather error behavior unspecified) |
| Kotlin coroutines | Structured concurrency, but cancellation is exception-based | `CancellationError` as typed result, not exception | B6 (scope is async CM — needs explicit) |
| Swift structured concurrency | Actor isolation, `Sendable` | Send/Sync checked, explicit sharing | B1 (AsyncFunction↛Function subtyping not rejected) |
| C# async | `Task<T>` vs `ValueTask<T>`, Sync-context confusion | Single `Task[T, E]` type | B2 (error type not constrained) |

---

### Final Verdict

**Verdict: Blockers found.**

The model is now 90% complete — the pass-4 blockers are resolved, the roadmap is synchronized, the type system additions are in architecture.md, and cancellation-cleanup is specified. But the six remaining blockers are genuine implementation hazards:

1. **B1**: No enforcement prevents calling an async function synchronously (async function type safety)
2. **B2**: `Task[T, E]` error type is unconstrained, violating result semantics
3. **B3**: Lock guard enforcement mechanism is unspecified (sync mutex vs. lint pass)
4. **B4**: `await` type-check rule is not a milestone_async_1 work item
5. **B5**: Auto-unwrap vs. result type confusion (Task vs. Result)
6. **B6**: `task.scope()` async context manager pattern is implied but not explicit

None requires structural redesign. All are text additions to `internal_docs/async_concurrency_model.md`, specifically in milestone_async_0.

**Recommendation: iterate again before planning.** The model is close to ready, but the remaining blockers would cause implementation teams to make incorrect assumptions about async function type safety, error type constraints, and lock enforcement. Adding the six fixes and resolving the subprocess/signal scope decision will make the model unambiguously ready.
