

# Review Pass 3: Sifr Async and Concurrency Model Proposal

**Reviewer:** agent
**Date:** 2026-05-09
**Document:** `internal_docs/async_concurrency_model.md`
**Branch:** `codex/figure-out-concurrency`
**Prior reviews:** `reviews/async-concurrency-model-review-pass-1.md`, `reviews/async-concurrency-model-review-pass-2.md`

---

## Summary

The pass-2 blocker is **fully resolved**. All 12 non-blocking items from pass-2 have been addressed. The proposal is internally consistent, coherent, and ready for Phase 32 planning.

---

## Pass-2 Blocker Resolution

**Contradiction in selection API naming (lines 77-82):** The list now reads:

```
- `scope.spawn(...)`: canonical task creation; all spawned tasks are children of a scope
- `task.gather(*handles)`: wait for multiple task handles, preserving input ordering
- `task.select(*handles)`: first-completion semantics; losers are cancelled by default
- `task.race(*handles)`: alias for `select`; losers are cancelled by default
```

This is correct. `scope.spawn` is canonical task creation; `gather`/`select`/`race` are `task.` namespace functions operating on handles. The primary example (lines 27-40) and all milestone references are consistent with this. **Resolved.**

---

## Pass-2 Non-Blocking Item Resolution

| Item | Resolution | Location |
|------|------------|----------|
| N-1: `CancellationError`/`TimeoutError` location | **Done** | Lines 296-297 (defined), 311-313 (policy), 874 (locked) |
| N-2: `Task[T]` shorthand for error-less variant | **Done** | Line 293: "`Task[T]` as shorthand for `Task[T, Never]` plus cancellation" |
| N-3: `select` tie-breaking rule | **Done** | Lines 319-320: handle creation order breaks ties deterministically |
| N-4: `cancellation_group_sibling.sifr` missing | **Done** | Line 492 |
| N-5: `sifr.threading` compatibility mapping table | **Done** | Lines 640-645, 638-646 (both present in m6) |
| N-6: ProcessPool dependency tracking | **Done** | Lines 740-741: blocked on Phase 40 typed IPC/serialization |
| N-7: `runtime_leak_rejected.sifr` fixture | **Done** | Line 438 in milestone_async_2 positive validation |
| N-8: `sifr.task.local[T]` rationale | **Done** | Line 191: scoped, lexical value with structured inheritance |
| N-9: Dependency graph labels | **No change needed** | Internally consistent |
| N-10: Phase exit gate "task groups" clarification | **Minor residual** | See below |
| N-11: Lock guard "async-aware" clarification | **Done** | Lines 330-335: async-aware internally, but guard must not cross await |
| N-12: `sifr.concurrent.ThreadPoolExecutor` consistency | **Done** | Lines 100, 627, 653 all use full `sifr.concurrent.` prefix |

**N-10 residual:** The phase exit gate (line 821) says "task groups" without clarifying that this means both `TaskGroup` (structured groups) and `TaskScope` (scope containers). The term is unambiguous in context — the full list on the same line covers `TaskGroup` and `TaskScope` explicitly. The plural "task groups" reads as a category label, not an ambiguity. No change required.

---

## Final Assessment: Blockers, Contradictions, and Elegance

### Blockers

**None.** The proposal has no remaining blockers. All three pass-1 blockers are resolved, the pass-2 blocker is resolved, and all 12 non-blocking items have been addressed.

### Contradictions

**None.** The model is internally consistent across:
- Task creation: `scope.spawn` canonical, `task.gather`/`select`/`race` operate on handles
- Type system: `Task[T]` = `Task[T, Never]` + cancellation; `Task[T, E]` always awaits to `Result[T, E]`
- Lock policy: async-aware internally, but scope-bound guard must not cross await
- Compatibility: `sifr.asyncio` and `sifr.threading` are explicit veneers over the canonical model
- Process pools: explicitly blocked on Phase 40, not merely deferred

### Elegance and Simplicity

The proposal achieves the stated goal: one coherent model with `async with task.scope()` + `scope.spawn()` as the canonical path. Key strengths:

- **Simple path for users:** `async def` + `await` + `scope.spawn` is the only first-class story
- **Hard to misuse:** Send/Sync boundary checking, borrow-across-await rejection, lock guard crossing compile error
- **Typed cancellation:** `CancellationError` and `TimeoutError` in `Result[T, E]`, not ambient exceptions
- **No hidden shared mutable state:** Explicit `sync.Lock`, `sync.Channel`, `sync.Shared`
- **Structured concurrency by default:** Scope owns children; scope exit cancels/joins all children

---

## Requested Concerns Coverage

| Concern | Coverage | Notes |
|---------|----------|-------|
| IO-bound diagnostics/decorators | **Full** | `@blocking_io`/`@cpu_bound` as diagnostics; stdlib blocking-call annotation database |
| CPU-bound work | **Full** | `spawn_blocking`, `ThreadPoolExecutor`, `@cpu_bound` |
| Shared memory | **Full** | `sync.Lock`, `sync.RwLock`, `sync.Shared`, `sync.Channel` (MPMC) |
| Structured concurrency | **Full** | `scope.spawn` canonical; scope owns child lifetimes; deterministic exit |
| Selectors | **Deferred** | Out of scope with rationale; runtime may use internally |
| Concurrent/threading/process pools | **Full** | `sifr.concurrent.ThreadPoolExecutor`, `sifr.threading` veneer; process pools blocked on Phase 40 |
| CPython parity layering | **Full** | `sifr.asyncio` and `sifr.threading` compatibility veneers after canonical model |
| Python async features with Rust-compatible semantics | **Full** | Typed cancellation, no ambient exceptions, explicit result handling |

---

## Milestone Boundaries and Dependency Coherence

The dependency graph (lines 792-813) is sound:

```
m0 → m1 → m2 → m3 → m4 → m5 → m7
                           ↘
                             m6 → m7 → m8
```

Key dependencies:
- `m1` must complete HIR awaitable model before `m2` can wire runtime
- `m4` (Send/Sync) must validate ownership rules before `m5` (sync primitives) ships channel Send/Sync behavior
- `m5` and `m6` both feed into `m7` (async resources) — both sync primitives and blocking offload must exist before async context manager protocol can be validated
- `m7` feeds into `m8` (compatibility veneers) — compatibility APIs must be tested against the canonical model

The sequence is coherent. No milestone is blocked by an unsequenced future decision. All open decisions are locked in `milestone_async_0` (m0).

---

## Sifr Principles Fit Assessment

| Sifr Principle | Model Compliance | Notes |
|---|---|---|
| Borrow-by-default | **Full** | Spawn captures checked; borrow-across-await rejected; `sync.Lock` explicit |
| Result/Option safety | **Full** | `await Task[T, E]` always returns `Result[T, E]`; no context-dependent magic |
| No hidden shared mutable memory | **Full** | No silent Arc/Mutex; explicit `sync.Lock`, `sync.Channel`, `sync.Shared` |
| No user-triggerable panic | **Full** | Cancellation is typed `CancellationError`; cleanup errors are secondary; no `.unwrap()` on task results in generated code |
| No data-dependent `.unwrap()` | **Full** | Codegen must avoid unwrap on task results; covered in phase exit gate |
| Structured concurrency default | **Full** | `scope.spawn` canonical; detached spawn absent; scope owns children |
| Simple path for users | **Full** | `async with task.scope()` + `scope.spawn()` is the one canonical path |
| Misuse is hard | **Full** | Send/Sync checks, borrow-across-await rejection, lock guard crossing compile error |

---

## Modern Language Lessons Comparison

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
| `sifr.threading` | Thin wrappers over `std::thread`/`std::sync` | Correct layer |
| Selectors/contextvars deferred | Out of scope; rationale provided | Correct decision |
| Process pools blocked | Blocked on Phase 40 typed IPC | Correct dependency tracking |

---

## Final Verdict

**Satisfied.**

The proposal is ready for Phase 32 planning. There are no blockers, no contradictions, and no structural gaps. All pass-1 and pass-2 feedback has been incorporated. The model is elegant, simple, and hard to misuse according to Sifr principles. Milestone boundaries and dependencies are coherent. The proposal can serve as the authoritative starting point for Phase 32 implementation planning.

---

## Appendix: Pass-1 Blocker Resolution (Complete)

| Pass-1 Blocker | Status |
|---|---|
| Blocker 1: `await Task[T, E]` semantics ambiguous | **Resolved** (pass 1) |
| Blocker 2: Lock guard across await deferred | **Resolved** (pass 1) |
| Blocker 3: Channel close/cancellation underspecified | **Resolved** (pass 1) |
| Pass-2 Blocker: Selection API naming contradiction | **Resolved** (pass 3) |
