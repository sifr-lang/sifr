**FAIL**

---

**1. `executor.map()` excluded from worker typing contract**

Pass 5 fix 5 addresses `submit()` worker callables (Callable[..., T] or Callable[..., Result[T,E]]) and `Future.result` typed evidence. `executor.map(fn, *iterables)` is a distinct API with different failure semantics: it raises on first exception, yields results in order, and has no Future intermediary. If map() is in scope for this phase, it needs its own callable type contract and error surface (does it return `Iterator[T]` or `Iterator[Result[T,E]]`? Does it propagate `ExecutorError::WorkerRuntime`?). If out of scope, it needs a compile-time diagnostic. Neither is currently specified. The omission creates an inconsistency: users who switch from submit/Future to map() lose the safety guarantees mid-phase.

**2. `Future.result(timeout=N)` is an unresolved typed terminal state**

Pass 5 added `ExecutorError::Cancelled` as a typed distinct state. `Future.result(timeout=N)` raises `TimeoutError` in CPython — a third terminal state alongside `Cancelled` and `WorkerRuntime`. The current fix doesn't classify timeout, leaving it ambiguous: does it become `ExecutorError::Timeout`, a separate `TimeoutError` type, or an undocumented panic path? The typed evidence guarantee for `Future.result` is incomplete without specifying timeout as a first-class terminal state with a CPython fixture.

**3. `io.TextIOWrapper` encoding policy not covered by open() fix**

Pass 5 fix 3 mandates explicit encoding for text-mode `open()`. However, `io.TextIOWrapper(binary_stream)` is a standard alternative for wrapping binary sockets, pipes, and other streams with a text codec. It accepts the same `encoding=` parameter and defaults to `locale.getpreferredencoding()` in CPython. The open() encoding fix doesn't extend to `TextIOWrapper`, creating a bypass: a user who calls `TextIOWrapper(sys.stdin.buffer)` without encoding gets no diagnostic despite the same implicit-locale hazard. The policy needs explicit extension or `TextIOWrapper` must be marked unsupported in text/i18n.

**4. `shutdown(cancel_futures=True)` interaction with `ExecutorError::Cancelled` is unspecified**

Pass 5 fix 2 adds `ExecutorError::Cancelled` as a typed terminal state triggered by `Future.cancel()`. Python 3.9+ `executor.shutdown(cancel_futures=True)` bulk-cancels pending futures at shutdown. The typed contract for this path is absent: do futures cancelled via shutdown carry `ExecutorError::Cancelled`? Are they observably distinct from `Future.cancel()`-triggered cancellations? If shutdown is in scope (it's a core lifecycle API), the interaction must be specified. If cancel_futures is unsupported, a compile-time diagnostic is required when the keyword argument is passed.

**5. `as_completed()` / `wait()` heterogeneous Future type erasure is unresolved**

The typed Future[T] returned by submit() creates a soundness question for `as_completed(futures)` and `wait(futures)`: these functions accept `Iterable[Future]` over futures with potentially different T types. CPython uses structural subtyping and runtime erasure. In Sifr's static type system, iterating a mixed `[Future[int], Future[str]]` collection needs either a type-erased `Future[Any]` (unsafe), a sum type, or a restriction to homogeneous collections. This is not a minor edge case — concurrent.futures patterns almost always mix future types in wait/as_completed calls. The phase must declare one of: (a) all futures in a set must share T, (b) a type-erased handle exists, or (c) as_completed/wait are out of scope with diagnostics. Silence here makes typed Futures less useful than CPython's untyped ones for real patterns.

---

**Summary of actionable items:**
- Specify executor.map() type contract or mark unsupported with diagnostic
- Add `ExecutorError::Timeout` (or equivalent) as typed terminal state for `Future.result(timeout=N)`
- Extend explicit-encoding mandate to `io.TextIOWrapper` or mark it unsupported
- Specify `shutdown(cancel_futures=True)` interaction with `ExecutorError::Cancelled`
- Resolve heterogeneous Future collection type strategy for `as_completed()`/`wait()`
