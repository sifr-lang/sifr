# Review Pass 2: Sifr Async and Concurrency Model Proposal

**Reviewer:** Claude Opus 4.7
**Date:** 2026-05-09
**Document:** `internal_docs/async_concurrency_model.md`
**Branch:** `codex/figure-out-concurrency`
**Prior review:** `reviews/async-concurrency-model-review-pass-1.md`

---

## Summary

The pass-1 blockers are **fully resolved**. All three open decisions (task result semantics, lock guard across await, channel close/cancellation semantics) have been locked with precise, unambiguous text. The additional recommendations from pass-1 have also been incorporated: scoped spawn as v1 default, typed `CancellationError`, race/select loser cancellation, MPMC channels, no lock guard across await, `@blocking_io`/`@cpu_bound` annotations, runtime-neutrality gate, asyncio/concurrent mapping table, selectors/contextvars/process pool deferrals, and async generator deferral rationale.

One **new contradiction** is introduced by the edits. The rest of the document is coherent, the milestone sequence is sound, and the model is ready for Phase 32 planning with one text fix.

---

## Blocker: Contradiction in selection API naming

**Severity:** blocker
**Location:** "Default APIs should prefer" list (lines 77-82) vs. all other references in the document.

**Problem:** The "Default APIs should prefer" list (lines 77-82) states:
```
Default APIs should prefer:
- task.scope(...)
- task.TaskGroup
- scope.spawn(...)
- task.gather(...)
- task.select(...)
- task.race(...)
```

But `scope.spawn` is consistent throughout the document, while `task.gather`, `task.select`, and `task.race` appear in this list as free-floating `task.` calls. Everywhere else in the document, these three functions are referenced as `task.gather`, `task.select`, and `task.race` (singular, not plural, and taking multiple task handles). The primary model example (lines 27-40) shows `scope.spawn` on individual tasks but does not call `gather`/`select`/`race`. The compatibility mapping table (lines 731-734) shows `sifr.task.gather(*tasks)` — which matches the `task.` namespace, not `scope.`.

The contradiction is that lines 77-82 list `task.gather(...)` while implying `scope.spawn(...)`, but the canonical model only shows `scope.spawn(...)` for task creation. The question is: can `gather`/`select`/`race` be called without a scope, operating directly on task handles? And if so, is that the intended design, or should these also be scope methods?

The rest of the document (milestones, locked decisions, compatibility table) consistently references these as `task.` namespace functions that take variadic task handles, which is internally consistent. The "Default APIs" list is the only place that creates apparent contradiction by listing both `scope.spawn(...)` and `task.gather(...)` in the same list without clarifying that `gather`/`select`/`race` operate on handles, not on a scope.

**Recommended fix — replace lines 77-82:**

> Default APIs should prefer:
> - `task.scope(...)`
> - `task.TaskGroup`
> - `scope.spawn(...)` — canonical task creation; all tasks are children of a scope
> - `task.gather(*handles)` — wait for multiple task handles, preserve ordering
> - `task.select(*handles)` — first-completion semantics; losers are cancelled by default
> - `task.race(*handles)` — alias for `select`; cancel losers by default

**Rationale:** The revised text makes it explicit that `gather`/`select`/`race` are free-floating `task.` functions that operate on task handles, not scope methods. This resolves the apparent contradiction with `scope.spawn` and aligns with the compatibility mapping table and all milestone references.

---

## Non-Blocking Issues

### N-1: `CancellationError` and `TimeoutError` are defined but their error type location is ambiguous

**Severity:** non-blocking
**Location:** milestone_async_0 work items (lines 295-296): defined as initial types in `sifr.task`?
**Recommendation:** Add a clarifying note in milestone_async_0 that `CancellationError` and `TimeoutError` are error types that live in `sifr.task` and are used as error variants in `Task[T, E]` results. Their precise relationship to `Result` error variants should be documented in the type system section.

> `CancellationError` and `TimeoutError` are Sifr error types. `Task[T, E]` uses `CancellationError` as an error variant when a task is cancelled before completing. `TimeoutError` is used as an error variant when an operation times out. Both are in the `sifr.task` module.

### N-2: `Task[T, E]` vs `Task[T]` — the no-error variant is implied but never stated

**Severity:** non-blocking
**Location:** Type system rules (line 231), milestone_async_0 (lines 292-293), milestone_async_1 acceptance criteria (line 372).

**Recommendation:** Add explicit note that `Task[T]` (without an error type) is a syntactic sugar for `Task[T, Never]` or a dedicated `Task[T]` type where errors are impossible (task completes normally or panics). This distinction matters for HIR lowering: `Task[T]` awaiting is simpler because there is no error path to track.

> `Task[T]` (no error type) is a task that cannot produce a typed error. It either completes with `T` or is cancelled (propagating `CancellationError`). `Task[T, E]` carries a typed error. The error-less variant is a simplification for tasks that are guaranteed to succeed or be cancelled.

### N-3: `select` cancellation — "cancel losers by default" may need a tie-breaking rule

**Severity:** non-blocking
**Location:** milestone_async_3 acceptance criteria (line 469), locked decisions (line 854).

**Recommendation:** Add a note on tie-breaking behavior: when multiple tasks in a `select`/`race` complete in the same tick, the runtime resolves ties non-deterministically but deterministically (e.g., first-wins by handle order). This should be documented in the selection policy.

> Tie-breaking: if multiple tasks in a `select`/`race` resolve in the same reactor tick, the runtime selects one deterministically (handle creation order). Users should not rely on tie-breaking order for correctness — if order matters, use `gather` with explicit priority logic.

### N-4: Missing cancellation determinism validation fixtures

**Severity:** non-blocking (partially addressed)
**Location:** milestone_async_3 positive validation (lines 480-482).

The pass-1 recommendation added `cancellation_scope_timeout.sifr`, `cancellation_nested_scopes.sifr`, and `cancellation_cleanup_runs.sifr`. The `cancellation_group_sibling.sifr` fixture from pass-1 N-8 is missing.

**Recommendation:** Add to milestone_async_3 positive validation:
- `cancellation_group_sibling.sifr`

### N-5: `sifr.threading` compatibility surface could be more explicit

**Severity:** non-blocking
**Location:** milestone_async_6 work items (lines 616-620).

**Recommendation:** The work items define `Thread`, `Lock`, `Event`, `Condition` as the threading compatibility surface. Add a mapping note similar to the asyncio compatibility table, or add to the existing table:

| `sifr.threading` API | Canonical Sifr equivalent |
|---|---|
| `sifr.threading.Thread` | `concurrent.Thread` (v1 via `std::thread`) |
| `sifr.threading.Lock` | `sync.Lock` (v1 via `std::sync::Mutex`) |
| `sifr.threading.Event` | `sync.Notify` (single-wakeup; use `Semaphore` for multi-wakeup) |
| `sifr.threading.Condition` | modelable via `sync.Notify` + `sync.Lock` in most cases |

### N-6: Process pool dependency tracking is present but could be stronger

**Severity:** non-blocking
**Location:** milestone_async_8 (line 719): "blocked on the future typed IPC/serialization contract."

**Recommendation:** Add a note linking to Phase 40 (or equivalent) in the dependency tracking:

> `ProcessPoolExecutor` is blocked on Phase 40 typed IPC/serialization contract. Process pools cannot safely transfer arbitrary Sifr values across process boundaries without a stable serialization model. This dependency is tracked in `issues/` and must be resolved before process pool work begins.

### N-7: Runtime-neutrality validation fixture name is listed but the behavior is not defined

**Severity:** non-blocking
**Location:** Required validation lanes (line 844): "runtime-neutrality checks proving Tokio/runtime-specific types do not leak into public Sifr APIs."

**Recommendation:** The phase should define `runtime_leak_rejected.sifr` explicitly in milestone_async_2 or milestone_async_8 acceptance criteria:

> `runtime_leak_rejected.sifr` — asserts that `sifr._runtime` or equivalent private boundary is the only import path for runtime-specific types, and that no public `sifr.task`, `sifr.sync`, or `sifr.concurrent` API exposes `tokio` or runtime-internal types.

### N-8: Missing `sifr.task.local[T]` rationale in out-of-scope section

**Severity:** non-blocking
**Location:** Out of scope rationale (line 191).

**Recommendation:** Pass-1 N-5 recommended adding rationale for `contextvars` deferral. The rationale is already present at line 191 (lexical scope preference, structured concurrency alignment). Pass-1 N-5 also suggested a future `sifr.task.local[T]` primitive. This is mentioned in line 191 but could be more explicit:

> If later evidence shows a real need for task-local storage with structured inheritance, a `sifr.task.local[T]` primitive can be designed without inheriting Python's ambient context-copying semantics. This would be modeled as a scoped, lexical variable with structured inheritance — not a global, mutable, copy-on-fork store.

### N-9: Dependency graph labels are abbreviated but internally consistent

**Severity:** non-blocking (clarification only)
**Location:** Dependency graph (lines 770-791).

The graph uses `m0` through `m8` labels with descriptive subtitles. This is internally consistent. No change needed.

### N-10: Phase exit gate — "task groups" pluralization inconsistency

**Severity:** non-blocking
**Location:** Phase exit gate (line 799): "supports scoped spawn, join, cancel, sleep, timeout, gather, select/race, and task groups."

**Recommendation:** "task groups" here means both `TaskGroup` (a structured group of tasks with error aggregation) and `TaskScope` (the `async with task.scope()` container). Clarify:

> supports scoped spawn, join, cancel, sleep, timeout, gather, select/race, `TaskGroup` (structured groups), and `TaskScope` (scope containers via `async with task.scope()`).

### N-11: `Lock[T]` vs `AsyncLock[T]` — "async-aware internally" is underspecified

**Severity:** non-blocking
**Location:** Lock policy in milestone_async_0 (lines 322-326).

**Recommendation:** "async-aware internally" is correct — `sync.Lock[T]` generates `tokio::sync::Mutex` which is safe to hold across `.await`. But the current text (line 323) says "`sync.Lock[T]` is async-aware internally" which is correct for v1, but the next sentence says "lock guards must not cross `await` points" which appears to contradict the "async-aware" claim.

**Clarification — replace line 323:**
> `sync.Lock[T]` generates a `tokio::sync::Mutex` in codegen. This is an async-aware lock that is safe to hold across `.await` in Rust terms, but Sifr's guard model restricts the guard to a synchronous scope. The guard itself (the RAII type) must not be live across an `await` point.

The "must not cross await" rule is a Sifr-level restriction on the guard lifetime, not a Rust-level constraint on the mutex. This distinction is important for implementors.

### N-12: `concurrent.ThreadPoolExecutor` module path inconsistency

**Severity:** non-blocking
**Location:** Required surfaces (line 100): `concurrent.ThreadPoolExecutor`, milestone_async_6 (line 615): `concurrent.ThreadPoolExecutor`.

The module is `sifr.concurrent` (per milestone_async_0 line 290). `concurrent.ThreadPoolExecutor` without the `sifr.` prefix is ambiguous — is it a local alias, an import, or a typo? Throughout the rest of the document, module-qualified names use the full `sifr.` prefix (e.g., `sifr.task.gather`, `sifr.sync.Channel`).

**Recommendation:** Use `sifr.concurrent.ThreadPoolExecutor` consistently. Update:
- Line 100: `sifr.concurrent.ThreadPoolExecutor`
- Line 615: `sifr.concurrent.ThreadPoolExecutor`

---

## Pass-1 Blocker Resolution Check

| Pass-1 Blocker | Status | Notes |
|---|---|---|
| Blocker 1: `await Task[T, E]` semantics ambiguous | **Resolved** | Lines 231, 300-302, 372, 853 all state "always produces `Result[T, E]`" with explicit `try`/`except` interaction |
| Blocker 2: Lock guard across await deferred | **Resolved** | Lines 322-326, 564, 574, 856 all lock guards to scope-bound; crossing await is compile error |
| Blocker 3: Channel close/cancellation underspecified | **Resolved** | Lines 554-560, 572-573 specify typed close and cancellation behavior for channels |

---

## Sifr Principles Fit Assessment (Updated)

| Sifr Principle | Model Compliance | Notes |
|---|---|---|
| Borrow-by-default | **Full** | Spawn captures checked; borrow-across-await rejected; `sync.Lock` explicit |
| Result/Option safety | **Full** | `await Task[T, E]` always returns `Result[T, E]`; no context-dependent magic |
| No hidden shared mutable memory | **Full** | No silent Arc/Mutex; explicit `sync.Lock`, `sync.Channel`, `sync.Shared` |
| No user-triggerable panic | **Adequate** | Cancellation is typed `CancellationError`; cleanup errors are secondary |
| No data-dependent `.unwrap()` | **Adequate** | Codegen must avoid unwrap on task results; covered in phase exit gate |
| Structured concurrency default | **Full** | `scope.spawn` is canonical; detached spawn absent in v1; scope owns children |
| Simple path for users | **Good** | `async with task.scope()` + `scope.spawn()` is the one canonical path |
| Misuse is hard | **Good** | Send/Sync checks, borrow-across-await rejection, lock guard crossing rejected |

---

## Modern Language Lessons Comparison (Updated)

| Feature | Proposal | Assessment |
|---|---|---|
| Structured concurrency default | `task.scope()` + `scope.spawn()` | Matches Swift, Kotlin, Go |
| Cancellation as typed result | `CancellationError` in `Task[T, E]` result | Better than Python's ambient `CancelledError` |
| No implicit Arc/Mutex | Explicit `sync.Lock[T]` | Matches Rust's explicit model |
| Async for IO-bound, explicit for CPU-bound | `@blocking_io`/`@cpu_bound` + `spawn_blocking` | Matches Tokio's split |
| Lock guards not crossing await | Scope-bound guards, compile error on crossing | Safer than Swift's `@MainActor` isolation |
| Task as `Result[T, E]` | Always `Result[T, E]` | Matches Go channel model, Swift try/throws |
| MPMC channels | `sync.Channel[T]` + bounded/unbounded constructors | Tokio-compatible |
| Select/race with loser cancellation | Default cancel losers | Matches Swift races |
| Async context managers | `async with` with cleanup under cancellation | Adequate |
| `sifr.threading` | Thin wrappers over `std::thread`/`std::sync` | Correct layer (milestone_async_6) |
| Selectors/contextvars deferred | Out of scope; rationale provided | Correct decision |

---

## Final Verdict

**Satisfied with non-blocking suggestions.**

One blocker exists: the contradiction in the "Default APIs should prefer" list (lines 77-82). It lists `task.gather(...)`, `task.select(...)`, and `task.race(...)` as free-floating calls alongside `scope.spawn(...)`, creating apparent inconsistency with the scope-based model shown in the primary example. The fix is one sentence per line plus three clarifying annotations — no structural change.

The pass-1 blockers are all resolved. The model is coherent, the milestone sequence is sound, and the proposal can be used as the starting point for Phase 32 planning once the one-line text fix is applied.

---

## Required Action Before Phase 32 Planning Begins

**Fix the "Default APIs should prefer" list** (lines 77-82) to clarify that `gather`/`select`/`race` are `task.` namespace functions operating on handles, not scope methods, and that `scope.spawn` is the canonical task creation API:

Replace:
```markdown
Default APIs should prefer:
- `task.scope(...)`
- `task.TaskGroup`
- `scope.spawn(...)`
- `task.gather(...)`
- `task.select(...)`
- `task.race(...)`
```

With:
```markdown
Default APIs should prefer:
- `task.scope(...)`
- `task.TaskGroup`
- `scope.spawn(...)` — canonical task creation; all spawned tasks are children of a scope
- `task.gather(*handles)` — wait for multiple task handles, preserve input ordering
- `task.select(*handles)` — first-completion semantics; losers are cancelled by default
- `task.race(*handles)` — alias for `select`; losers are cancelled by default
```

Once this fix is applied, the proposal is ready for Phase 32 planning.
