

Based on my analysis of all three documents and the pass-5 review, here is my final blocker check:

---

**Verdict: Satisfied**

---

## Pass-5 Blockers — All Resolved

| Blocker | Pass-5 Finding | Current State |
|---|---|---|
| **B1**: Async function type callable as sync | No enforcement of `AsyncFunction` vs `Function` subtyping | **FIXED** — `AsyncFunction` is explicitly NOT a subtype of `Function`/`Callable`. Storing, passing, or invoking through a sync path is a compile-time error. (async_concurrency_model.md line 256) |
| **B2**: `Task[T, E]` error type unconstrained | `E` had no constraint, allowing invalid error types | **FIXED** — `Task[T, E]` requires `E: Error`, matching `Result[T, E]`. `Task[T, Never]` is valid as a special case. (line 250) |
| **B3**: Lock guard enforcement mechanism unspecified | Never defined whether sync mutex or lint pass | **FIXED** — `sync.Lock[T]` uses a synchronous Rust mutex internally in v1. `.lock()` is not await-aware. Type checker rejects live `LockGuard` at await. Diagnostic message defined. (lines 394-401) |
| **B4**: `await` type-check rule missing from work items | Rule existed but not in milestone_async_1 work items | **FIXED** — milestone_async_1 work items explicitly include "Add await type-checking: `await x` is valid only when `x: Awaitable[T]`" (line 443) |
| **B5**: Auto-unwrap sequencing unclear | Never explicitly stated that unwrap applies to `Result`, not `Task` | **FIXED** — "auto-unwrap is sequenced after await: first `await` produces `Result[T, E]`, then `try` unwraps that result" (line 368). This is unambiguous. |
| **B6**: `task.scope()` async context manager implicit | Never stated whether `TaskScope` implements `__aenter__`/`__aexit__` | **FIXED** — `TaskScope` is explicitly an async context manager implementing `__aenter__` and `__aexit__`. `__aexit__` waits for children, cancels on abnormal exit, waits for cleanup. Lifetime is scoped to `async with`. (lines 345-348) |

---

## Non-Blocking Refinements — Status

| Item | Pass-5 Request | Current State |
|---|---|---|
| **N-1**: `gather` error behavior | Specify fail-fast vs. collect-all | **ADDRESSED** — First error cancels unfinished children; earliest input-order error is primary, later errors are secondary structured errors. Future collect-all API deferred. (lines 544-547) |
| **N-2**: Async main bootstrap | Make `async def main()` explicit | **ADDRESSED** — "Support `async def main() -> Result[None, E]` where `E: Error` as the canonical async program entrypoint" (line 486) |
| **N-3**: TaskScope vs TaskGroup distinction | Unclear difference | **ADDRESSED** — `TaskScope` owns lifetime; `TaskGroup` owns group error policy. Explicitly distinct. (line 349) |
| **N-4**: `sifr.asyncio.run()` mapping | May be unnecessary | **ADDRESSED** — Explicitly noted as "compatibility shim over direct async entrypoint bootstrap; not needed for new Sifr code" (line 845) |

---

## Cross-Cutting Checks Against Sifr Principles

**Static typing**: `Task[T, E]` constrained to `E: Error`, `AsyncFunction` distinct from sync callable — all enforced at type-check time. No runtime surprises.

**Result/Option error handling**: `await Task[T, E]` always produces `Result[T, E]`. Auto-unwrap applies to the `Result`, not the `Task`. Sequencing is explicit.

**Ownership safety**: Borrow-across-await rules in a table (lines 266-272). Lock guards cannot cross `await` in v1. Spawn boundaries require `Send + 'static` captures.

**Deterministic structured concurrency**: Parent scopes own children (lines 344-348). Sibling cancellation on first failure (line 543). Selection tie-breaking by creation order (line 384).

**No user-triggerable runtime panics**: `catch_unwind` around `__aexit__` (lines 773-777). Lock guard enforcement via type system + diagnostic (lines 394-401). No silent `Arc`/`Mutex` insertion (line 134).

**Subprocess/signal decision**: Locked out of scope for v1. Requires separate model amendment. (lines 361-362, line 989)

---

**Recommendation: ready to proceed to implementation planning**
