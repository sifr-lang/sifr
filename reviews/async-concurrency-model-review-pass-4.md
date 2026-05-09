

Review written to `reviews/async-concurrency-model-review-pass-4.md` (483 lines).

---

## Summary of Findings

**Verdict: Blockers found.**

The prior three passes resolved all semantic blockers. This adversarial pass treats the model as a real implementation starting point and finds three remaining blockers:

### Blockers

| # | Blocker | Why It's Blocking |
|---|---|---|
| B1 | **No async types in the type system** | `Task[T, E]`, `Awaitable[T]`, `AsyncFunction` are never defined in the `Type` enum (architecture.md). Every subsequent milestone depends on them. |
| B2 | **HIR nodes missing + roadmap out of sync** | The model lists "required HIR concepts" but none map to actual node additions. More critically, `internal_docs/phases/32_async_ecosystem.md` has 4 milestones with conflicting scope vs. this model's 9 milestones. |
| B3 | **Cancellation during async cleanup underspecified** | LIFO cleanup order, cancellation during `__aexit__`, panic handling (double-panic abort prevention), and nested cancellation are all undefined. Without these, the no-user-triggerable-panic guarantee cannot be verified. |

### Non-Blocking Issues (11)

The most important: effect system for `@blocking_io`/`@cpu_bound` is annotation-only (acceptable for v1, formal effects for v2), `sync.Shared` should be clarified as `Arc<T>`, `asyncio.timeout` context manager form not mapped, async comprehensions deferred without distinguishing them from async generators, and runtime selection policy deferred but unspecified.

### Required Actions

All three blockers are resolvable with text additions — no structural redesign needed. Each requires adding a specific contract paragraph to `internal_docs/async_concurrency_model.md`.
 the architecture document's `Type` enum (lines 821–894). More critically, the model does not define the type for an **async function itself** — the callable type of `async def`. Before implementation begins, the type system must answer:

1. What is the type of `async def foo() -> T`? (A: `Callable[[], Task[T, Never]]`? Or a distinct `AsyncFunction[[], T]` type?)
2. What is the type of `Task[T, E]`? Is it a struct? A generic? Does it auto-implement an awaitable protocol?
3. When `await foo()` is valid — what makes `foo` awaitable? Is it a structural property (implements `Awaitable` protocol) or a nominal type (is exactly `Task`)?
4. Can a function that is not `async def` produce a `Task`? (e.g., `def wrapper() -> Task[T, E] { return spawn(something) }` — is this valid?)
5. What is the type of `scope.spawn(fetch_one)` where `fetch_one: async def (str) -> Result[str, NetworkError]`?

The HIR nodes (hir_nodes.rs) have no async markers. The type system (`Type` enum) has no `AsyncFunction`, `Awaitable`, or `Task` variant. This is not a minor omission — every subsequent milestone depends on the answer to these questions.

**Why this is a blocker, not a suggestion:** The model says "await Task[T, E] always produces Result[T, E]" but never defines what `Task[T, E]` *is* in the type system. Implementation cannot begin without this. The type system addition is not optional.

**Recommended fix — add to milestone_async_0 work items:**

> **Define the async type system additions before HIR lowering begins:**
>
> 1. `Type::Task` (with optional error type parameter) as a named type that represents an awaitable handle. It is distinct from `Type::Result` — `Result[T, E]` is a value type that may or may not be produced by awaiting a `Task`. `Task[T, E]` is not a `Result`; it is a handle that yields a `Result` when awaited.
>
> 2. `Type::AsyncFunction` (or structured `Callable` with async capability flag) for the type of `async def` functions. An async function is not directly callable as a sync function — `await fetch_one(...)` is not valid; `fetch_one(...)` must be awaited or spawned. The type system must distinguish `async def` from `def` for the same signature.
>
> 3. Awaitability is a structural property: any type that implements the `Awaitable[T]` protocol is awaitable. `Task[T, E]` implements `Awaitable[Result[T, E]]`. Custom awaitable types (third-party futures, async iterators) may also implement the protocol.
>
> 4. `scope.spawn(fn)` requires `fn: AsyncFunction` — only async functions may be spawned directly. Sync functions that return `Task[T, E]` can be spawned via a compatibility path, but the canonical API is `async def`.
>
> Add to architecture.md `Type` enum:
> ```rust
> // Awaitable types (milestone_async_0)
> Task(Box<Type>, Box<Type>),  // Task[T, E] — typed task handle
> Awaitable(Box<Type>),       // structural protocol for awaitability
> ```

**Implementation note:** The awaitable protocol should be structural (any type implementing a specific trait) rather than nominal (only `Task`). This matches Rust's `Future` model and allows third-party async libraries to integrate cleanly.

---

### Blocker 2: HIR async nodes are missing from the requirements and the roadmap is out of sync

**Severity:** critical blocker — milestone_async_1 cannot be验收ed without explicit HIR node definitions
**Location:** Architecture Targets / Language and HIR section (lines 202–223), milestone_async_1 work items (lines 365–374), HIR nodes (hir_nodes.rs lines 1–592), `internal_docs/phases/32_async_ecosystem.md`

**Problem:** The model lists "Required concepts" for HIR (lines 203–214) but these are not mapped to actual HIR node additions. The current HIR nodes (hir_nodes.rs) contain zero async-related variants. The requirement is to add these, but no explicit node enumeration exists.

More critically: the Phase 32 roadmap document (`internal_docs/phases/32_async_ecosystem.md`) has a different milestone structure (32_1 through 32_4) that conflicts with this model's milestones (async_0 through async_8). The scope differences are real:
- Subprocess and signal: roadmap puts these in milestone_32_4. Model defers them entirely.
- `sifr.asyncio`: roadmap does not mention it. Model puts it in milestone_async_8.
- `sifr.subprocess`: roadmap requires it. Model defers it.
- Milestone count: 4 vs. 9. This affects execution planning and PR sizing.

**Why this is a blocker:** Implementation teams will read both documents and find conflicting requirements. The milestone names and scopes must be synchronized before Phase 32 begins.

**Recommended fix — add to milestone_async_0 work items:**

> **Resolve scope conflict between this model and the Phase 32 roadmap document (`internal_docs/phases/32_async_ecosystem.md`).** Two documents define Phase 32. This model defines `milestone_async_0` through `milestone_async_8`. The Phase 32 roadmap defines `milestone_32_1` through `milestone_32_4`. The two sets must be aligned before implementation begins. Key conflicts:
>
> - Subprocess and signal: roadmap puts these in milestone_32_4. Model defers them entirely.
> - `sifr.asyncio`: roadmap does not mention it. Model puts it in milestone_async_8.
> - `sifr.subprocess`: roadmap requires it. Model defers it.
> - Milestone count: 4 vs. 9. This affects execution planning and PR sizing.
>
> **Decision required:** Is Phase 32 the 4-milestone roadmap scope (runtime + structured concurrency + sync/blocking + ecosystem) or the 9-milestone model scope (runtime + structured concurrency + sync/blocking + async resources + compatibility)? This must be resolved in milestone_async_0 and copied into `internal_docs/phases/32_async_ecosystem.md` before any implementation work begins.

**Also add to milestone_async_1 work items:**

> Enumerate specific HIR node additions:
> - `HirStmt::AsyncFnDef` or `HirFunction::is_async` flag
> - `HirExpr::Await`
> - `HirExpr::AwaitableCall` (async function invocation — distinct from sync call)
> - `HirExpr::TaskSpawn`
> - `HirStmt::AsyncWith`
> - `HirStmt::AsyncFor`
> - `HirType::Task` (awaitable task handle type)
> - `HirType::Awaitable` (structural protocol)
>
> The exact node names must be specified and agreed upon in milestone_async_0 before HIR work begins.

---

### Blocker 3: Cancellation during async cleanup is underspecified

**Severity:** blocking implementation of milestone_async_7 (async context managers) and affects the no-user-triggerable-panic guarantee
**Location:** milestone_async_7 work items (lines 687–692), acceptance criteria (lines 695–703)

**Problem:** The acceptance criteria say:
> When a task is cancelled inside an `async with` block, async exit/cleanup is called before scope exit completes. If cleanup itself fails during cancellation, the cancellation remains the primary result and cleanup failure is surfaced as secondary structured error evidence through the owning scope.

This is the correct high-level behavior, but it does not answer the critical questions for implementation:

1. **Order of cleanup under concurrent cancellation:** When a scope is cancelled, it cancels all child tasks. Those child tasks may be inside `async with` blocks with their own cleanup. What is the cancellation order? Are all `__aexit__` methods called in declaration order, reverse order, or concurrently? In Python asyncio, `__aexit__` runs in LIFO order (reverse). Does Sifr match this?

2. **Cancellation during `__aexit__`:** If `__aexit__` is itself an async function and cancellation arrives while `__aexit__` is running, does the `__aexit__` get cancelled? Or does it run to completion? Swift's structured concurrency runs `defer` even under cancellation. Python asyncio's `__aexit__` runs unless the event loop is shutting down forcefully.

3. **Panic during cleanup:** The Sifr guarantee is "no user-triggerable panics." But `__aexit__` is user-defined code. If `__aexit__` panics, what happens? In Rust, a panic during Drop is a double-panic abort. This means user code in `__aexit__` that panics could crash the process. The model must define this boundary.

4. **Nested scopes with cancellation:** If task A is cancelled but task A has spawned task B (inside its own `async with`), and both have cleanup, what is the observable behavior?
   - Does task A's cancellation also cancel task B?
   - Does task A wait for task B's cleanup before completing its own cleanup?
   - Or does task B get cancelled independently?

These questions are not academic. They determine whether the implementation can be verified correct, and whether the "no user-triggerable panic" guarantee holds when cancellation intersects with user-defined cleanup.

**Recommended fix — add to milestone_async_7 work items:**

> **Define cancellation-cleanup interaction with exact behavioral contract:**
>
> 1. **Cleanup order is LIFO (reverse declaration order).** When a scope exits, `__aexit__` is called in reverse order of `__aenter__` acquisition. This matches Python's established semantics and is predictable.
>
> 2. **`__aexit__` execution under cancellation:**
>    - When a task is cancelled, its `async with` blocks unwind in LIFO order.
>    - Each `__aexit__` is called with the cancellation cause as the exception argument (mimicking Python's `__aexit__(exc_type, exc_val, exc_tb)`).
>    - If `__aexit__` is itself an async function, it runs to completion unless the enclosing scope is also cancelled. Concurrent cancellation does not interrupt an in-progress `__aexit__`.
>    - If `__aexit__` raises, the cancellation error takes precedence. The `__aexit__` error is logged or surfaced as a secondary error through the scope's error aggregator, matching the existing acceptance criterion.
>
> 3. **Panic during `__aexit__`:**
>    - Panics during user-defined `__aexit__` are treated as secondary errors, not primary cancellation results.
>    - The scope catches the panic internally (using `std::panic::catch_unwind` around each `__aexit__` call) and propagates it as a secondary error.
>    - This prevents double-panic abort and preserves the no-user-triggerable-panic guarantee for the enclosing scope, even when user cleanup code panics.
>    - Note: this requires `__aexit__` to be called from within a `catch_unwind` boundary at the runtime layer. This is a codegen/runtime concern, not a language-level decision.
>
> 4. **Nested cancellation:**
>    - Scope cancellation cancels the scope's direct children only.
>    - Child tasks are cancelled independently unless they share the same cancellation scope.
>    - Nested `async with` blocks unwind independently for each task.
>    - If task A cancels and task B is a child of task A, task B's cancellation is triggered by task A's cancellation propagation. Both tasks unwind their `async with` blocks independently.
>
> Add to acceptance criteria:
> - Cancellation of a task with `async with` blocks triggers LIFO `__aexit__` unwinding.
> - `__aexit__` receives the cancellation cause as its exception argument.
> - `__aexit__` errors do not override the primary cancellation result.
> - Nested task cancellation is deterministic: parent cancellation triggers child cancellation, but each task unwinds its own cleanup independently.
> - Panics during `__aexit__` are caught and surfaced as secondary errors, not as process-terminating events.

---

## NON-BLOCKING ISSUES

### N-1: The effect/capability system is underspecified for `@blocking_io` / `@cpu_bound`

**Severity:** non-blocking (clarity gap in the diagnostic model)
**Location:** milestone_async_0 blocking annotation policy (lines 336–340), milestone_async_6 (lines 622–635)

**Problem:** `@blocking_io` and `@cpu_bound` are defined as "compile-time diagnostic annotations." But the model never explains how the diagnostic is produced. Is it static analysis, annotation-driven, or heuristic? Without a defined mechanism, implementation cannot estimate milestone_async_6 work.

**Recommended fix — add to milestone_async_0 blocking annotation policy:**

> **Annotation model:**
> - `@blocking_io` and `@cpu_bound` are declaration-site annotations. They attach to function definitions.
> - The compiler maintains a database of known-blocking stdlib functions (e.g., `sifr.os.read`, `sifr.time.sleep`).
> - When a known-blocking function or `@blocking_io`-annotated function is called inside an `async def` body, a diagnostic is emitted: "blocking call in async context — use spawn_blocking or an async API."
> - The diagnostic is a **warning**, not an error. Compilation proceeds.
> - The annotation does not prevent the call; it informs the developer.

---

### N-2: `sync.Shared` ownership under concurrent cancellation is ambiguous

**Severity:** non-blocking (implementation realism note)
**Location:** milestone_async_5 work items (lines 560–562)

**Problem:** `sync.Shared[T]` is defined as "cheap shared ownership of immutable data." But the ownership semantics when a task is cancelled while holding a `Shared` reference are not defined. The model should clarify that `Shared` is an alias for `Arc<T>` (cheap shared ownership via atomic reference count), with no mutation semantics.

---

### N-3: `asyncio.timeout` as a context manager is not specified

**Severity:** non-blocking (completeness gap in Python async parity)
**Location:** Compatibility mapping table (lines 749–758), milestone_async_8 work items (lines 728–744)

**Problem:** Python's `asyncio.timeout()` (PEP 567, Python 3.11+) is a context manager that cancels a wrapped operation on timeout. The compatibility mapping table maps `asyncio.wait_for` to `task.timeout`, but `asyncio.timeout` context manager form is not specified.

**Recommended fix — add to compatibility mapping table:**

> | `sifr.asyncio.timeout(duration)` | `task.timeout(duration)` context manager form |
> |---|---|
> | `sifr.asyncio.timeout_at(deadline)` | `task.timeout_at(deadline)` (absolute deadline) |

---

### N-4: Async comprehensions are deferred without acceptance criteria distinction

**Severity:** non-blocking (clarity gap)
**Location:** Out of scope (lines 187), milestone_async_7 (line 692)

**Problem:** Async comprehensions and async generators are both deferred but for different reasons. Async comprehensions are syntax sugar over `async for` + collection building. If `async for` is in scope, async comprehensions could be a simple syntactic transformation.

**Recommended fix — add to milestone_async_7 acceptance criteria:**

> **In scope:** `async for x in channel` (channel-backed async iteration).
>
> **Explicitly deferred to v2:**
> - Async generators: `async def` with `yield`. Requires separate `AsyncGenerator` HIR, protocol, and runtime support.
> - Async comprehensions: `[x async for x in iter]`. May be added as syntactic sugar over `async for` after `async for` is stable.

---

### N-5: Phase 32 roadmap and model document have incompatible milestone names and scopes

**Severity:** non-blocking (documentation inconsistency, causes execution confusion)
**Location:** `internal_docs/phases/32_async_ecosystem.md` vs. `internal_docs/async_concurrency_model.md`

**Problem:** Two documents define Phase 32 with 4 vs. 9 milestones and conflicting scope. This is not a documentation hygiene issue — a team implementing Phase 32 will read both and find conflicting requirements.

**Recommended fix — add to milestone_async_0:**

> **Synchronize Phase 32 roadmap with this model.** The `internal_docs/phases/32_async_ecosystem.md` file must be updated to reference this model document and align milestone names. After milestone_async_0 closes, the Phase 32 roadmap should read:
>
> > Phase 32 execution follows `internal_docs/async_concurrency_model.md` milestone_async_0 through milestone_async_8. The 4-milestone structure in this document is superseded by the 9-milestone model.

---

### N-6: `sifr.subprocess` and `sifr.signal` appear in phase 32 roadmap but not in the model

**Severity:** non-blocking (scope ambiguity)
**Location:** `internal_docs/phases/32_async_ecosystem.md` milestone_32_4, `internal_docs/async_concurrency_model.md` out-of-scope section

**Problem:** The phase 32 roadmap requires `sifr.subprocess` and `sifr.signal` as work items in milestone_32_4. The model document defers subprocess entirely and does not mention signal. This is a scope decision that affects what "Phase 32 complete" means.

**Recommendation:** Decide whether subprocess and signal are in or out of scope for Phase 32. If in scope, add them to the model. If out of scope, remove them from the phase 32 roadmap.

---

### N-7: "Async is for waiting" — borrow/own model for async closures is underspecified

**Severity:** non-blocking (ownership semantics gap)
**Location:** Type System rules (lines 224–238), milestone_async_4 (lines 506–551)

**Problem:** The distinction between "borrowed across await" (same task, borrow validity) and "borrowed across spawn" (new task, Send requirement) is correct but scattered. A single table would clarify.

**Recommended fix — add to milestone_async_0 type system rules:**

> **Borrow rules at spawn vs. await boundaries:**
>
> | Capture type | Across `await` (same task) | Across `spawn` (new task) |
> |---|---|---|
> | `&T` (immutable borrow) | Allowed if `T: Sync` and borrow is proven live at await point | Allowed if `T: Sync + Send` |
> | `&mut T` (mutable borrow) | Forbidden (mutable alias safety) | Forbidden (mutable alias not Send) |
> | `T` (owned value) | Always allowed | Allowed if `T: Send` |
> | `Rc<T>` | Forbidden (not Send) | Forbidden (not Send) |
> | `Arc<T>` | Allowed (Send + Sync) | Allowed (Send + Sync) |

---

### N-8: `sifr.asyncio` compatibility mapping omits `Event`, `Condition`, and `Future`

**Severity:** non-blocking (parity gap)
**Location:** Compatibility mapping table (lines 749–758)

**Problem:** The compatibility mapping table covers 8 APIs but omits `asyncio.Event`, `asyncio.Condition`, `asyncio.Barrier`, `asyncio.wait` (done/pending sets), and `asyncio.as_completed`. The curated subset rationale should be documented.

**Recommended fix — add to compatibility mapping table:**

> **Curated subset rationale:**
> - `sifr.asyncio` covers the 8 APIs most commonly used in async Python code.
> - `asyncio.Event` → use `sync.Notify`
> - `asyncio.Condition` → use `sync.Notify + Lock`
> - `asyncio.Barrier` → deferred to v1.1
> - `asyncio.wait` (done/pending sets) → use `task.gather` (all-or-nothing) or `task.select`/`task.race` (first-completion)
> - `asyncio.as_completed` → use `async for x in channel` with a channel-based producer

---

### N-9: Runtime selection is mentioned but the decision process is not specified

**Severity:** non-blocking (runtime architecture note)
**Location:** Runtime Is An Implementation Detail section (lines 145–156), milestone_async_0 work items (lines 283–286)

**Problem:** The model says "implementation may use Tokio" but does not address: what determines which runtime, can users provide their own, can multiple runtimes coexist, what happens with multiple entrypoints?

**Recommended fix — add to milestone_async_0:**

> **Runtime selection policy:**
> - Tokio is the default and only runtime for v1.
> - User-visible runtime configuration is not part of the primary model. Users write `async def main()` and `sifr run` bootstraps Tokio automatically.
> - Multiple runtime instances in the same binary are not supported in v1.
> - Custom runtime injection (`sifr.run(runtime=...)`) is deferred to v2.
> - Runtime configuration (thread count, I/O driver settings) is deferred to v2.

---

### N-10: The `Never` type is referenced but not defined in the type system section

**Severity:** non-blocking (type system completeness)
**Location:** milestone_async_0 work items (line 293): "Task[T] as shorthand for Task[T, Never] plus cancellation"

**Problem:** `Task[T]` is defined as shorthand for `Task[T, Never]` plus cancellation. But `Never` (the bottom type) is not in the architecture document's `Type` enum.

**Recommended fix — add to architecture.md `Type` enum:**

```rust
// Bottom type (milestone_async_0)
Never,  // The type with no values. Used for Task[T, Never], exhaustive matches, etc.
```

---

### N-11: Missing negative validation fixture for cancellation-cleanup panic boundary

**Severity:** non-blocking (validation completeness)
**Location:** milestone_async_7 negative validation (lines 711–716)

**Problem:** The model specifies that panics during `__aexit__` are caught and surfaced as secondary errors. But there is no negative validation fixture that verifies this behavior.

**Recommended fix — add to milestone_async_7 negative validation:**

> - `async_with_cleanup_panic_secondary.sifr` — verifies panic in `__aexit__` does not abort
> - `async_with_nested_cleanup_order.sifr` — verifies LIFO cleanup order under cancellation

---

## Question-by-Question Adversarial Analysis

### Q1: Is this model complete enough as a Phase 32 starting point?

**No, with blockers.** The model has three blockers: missing async type in the type system (blocker 1), missing HIR node specifications and scope conflict with the phase 32 roadmap (blocker 2), and underspecified cancellation-cleanup interaction (blocker 3).

The model is also missing: the awaitable protocol definition, the async function type, the runtime bootstrapping contract, the blocking-call diagnostic mechanism, and the panic-cleanup boundary.

### Q2: Is it elegant and teachable?

**Partially.** The primary model is elegant: `async def`, `await`, `task.scope()`, `scope.spawn()`. But the surface vocabulary has accumulated 5 modules (`task`, `sync`, `concurrent`, `asyncio`, `threading`) plus 2 annotations. Acceptable for v1. Main concern: `sifr.asyncio` familiarity may pull users toward the compatibility layer instead of the cleaner canonical path.

### Q3: Does it avoid known hiccups of other languages?

**Mostly yes, with gaps:**

| Language | Known Issue | Model Avoidance | Residual Gap |
|---|---|---|---|
| Python asyncio | Event-loop leakage, ambient cancellation | Structured concurrency via `scope.spawn` | `CancellationError` is typed but can still leak if `await` is not in a try block |
| Rust/Tokio | Send/'static errors, lock guards across await | Lock guards must not cross await (compile error) | `Send` checking deferred to m4 |
| Rust/Tokio | Too many executor choices | One runtime, auto-selected | Runtime configuration deferred |
| Go | Goroutine leaks | Structured concurrency default | Detached spawn deferred |
| Kotlin/Swift | Structured concurrency edge cases | Cancellation as typed result | Cancellation during nested cleanup (see Blocker 3) |

### Q4: Are structured concurrency, cancellation, select/race, timeout, cleanup, and task result semantics specified well enough?

| Feature | Specification Quality | Gap |
|---|---|---|
| Structured concurrency | **Good** | `scope.spawn` + scope owns children |
| Cancellation | **Good** | `CancellationError` in result |
| select/race | **Adequate** | First-completion + loser cancellation |
| timeout | **Partial** | Function form defined; context manager form not |
| cleanup | **Partial** | Cancellation-cleanup interaction order underspecified (Blocker 3) |
| task result semantics | **Good** | `await Task[T, E]` always yields `Result[T, E]` |

### Q5: Are `@blocking_io` / `@cpu_bound` the right starting abstraction?

**Yes, but v2 should evaluate a formal effect system.** The diagnostic model is the pragmatic choice for v1. A formal effect/capability model (tracking `async`, `blocking`, `cpu`, `io`, `send`, `sync` as effect tokens) would be more robust but requires significant type system work. For v1, diagnostics-only is acceptable.

### Q6: Is the CPU-bound story enough?

**Adequate for v1.** `spawn_blocking` + `ThreadPoolExecutor` + `@cpu_bound` covers practical cases. Process pool deferral to Phase 40 is correct.

### Q7: Is shared memory handled correctly?

**Mostly correct, with ambiguity around `sync.Shared`.** `Lock`, `RwLock`, `Channel`, `Semaphore`, `Notify` are all correct. `sync.Shared` is ambiguous — should be clarified as `Arc<T>`.

### Q8: Are selectors and low-level readiness correctly deferred?

**Yes.** Rationale is sound: users compose tasks and channels rather than file-descriptor readiness APIs.

### Q9: Are all Python async features accounted for?

**~80% coverage.** 8 of 10 major features in-scope or intentionally deferred with rationale. Gaps: `asyncio.timeout` context manager form, subprocess/signal scope conflict, async generators/comprehensions deferred.

### Q10: Is milestone sequencing implementation-realistic?

**Mostly yes, with hidden dependencies.** `m4` (Send/Sync) must validate captures but blocking semantics are defined in `m6`. The dependency direction is acceptable if m4 uses conservative assumptions. Scope conflict between roadmap and model is the bigger sequencing risk.

### Q11: What should be cut, added, locked, renamed, or moved?

| Action | Item | Reason |
|---|---|---|
| **Add** | `Type::Task` and `Type::Awaitable` to the type system | Required before m1 begins (Blocker 1) |
| **Add** | Explicit HIR node enumeration for async | Required before m1 begins (Blocker 2) |
| **Add** | Cancellation-cleanup interaction contract | Required before m7 begins (Blocker 3) |
| **Add** | Effect system note for v2 evaluation | N-1 |
| **Clarify** | `sync.Shared` as `Arc<T>` | N-2 |
| **Add** | `task.timeout` as context manager | N-3 |
| **Clarify** | Async comprehension deferral rationale | N-4 |
| **Synchronize** | Phase 32 roadmap with model document | N-5 |
| **Clarify** | subprocess/signal scope decision | N-6 |
| **Add** | Spawn vs. await borrow rules table | N-7 |
| **Document** | Curated subset rationale for `sifr.asyncio` | N-8 |
| **Add** | Runtime selection policy | N-9 |
| **Add** | `Type::Never` to type system | N-10 |
| **Add** | Panic-cleanup validation fixtures | N-11 |
| **Cut** | Nothing | Scope is well-reasoned |

---

## Findings Ordered by Severity

### Critical (blocks Phase 32 planning)

1. **B1:** Missing async type in the type system — `Task[T, E]`, `Awaitable[T]`, `AsyncFunction` not defined
2. **B2:** HIR async node specifications missing and scope conflict between phase roadmap and model document
3. **B3:** Cancellation during async cleanup underspecified — cleanup order, cancellation during `__aexit__`, panic handling

### Important (should be resolved before Phase 32 execution begins)

4. **N-1:** Effect/capability system underspecified for `@blocking_io`/`@cpu_bound`
5. **N-5:** Phase 32 roadmap and model document incompatible milestone names and scopes
6. **N-7:** Borrow rules for spawn vs. await boundaries scattered
7. **N-9:** Runtime selection policy deferred but unspecified

### Minor (resolvable during Phase 32 execution)

8. **N-2:** `sync.Shared` ownership semantics ambiguous
9. **N-3:** `asyncio.timeout` context manager form not specified
10. **N-4:** Async comprehensions deferred without acceptance criteria distinction
11. **N-6:** `sifr.subprocess`/`sifr.signal` scope conflict between documents
12. **N-8:** `sifr.asyncio` curated subset rationale missing
13. **N-10:** `Never` type referenced but not in type system
14. **N-11:** Missing negative validation for cancellation-cleanup panic boundary

---

## Final Verdict

**Blockers found.**

The model is 85% complete and the prior three passes correctly identified and resolved the critical semantic gaps (task result semantics, lock guard across await, channel close semantics). This adversarial pass identifies the remaining blockers:

1. **The type system has no async types.** The architecture document's `Type` enum has no `Task`, `Awaitable`, or `AsyncFunction`. Implementation cannot begin without these additions.

2. **The HIR node list is incomplete and the roadmap is out of sync.** The model's "Required concepts" list does not map to actual HIR node additions, and the phase 32 roadmap has incompatible milestone names and scopes.

3. **Cancellation during async cleanup is a real implementation concern, not a detail.** The contract must specify LIFO cleanup order, cancellation during `__aexit__` behavior, panic handling, and nested cancellation. Without these, the no-user-triggerable-panic guarantee cannot be verified for async context managers.

All three blockers are resolvable with precise text additions to `internal_docs/async_concurrency_model.md`. None requires a structural redesign. The blocking nature is that each would cause implementation to proceed on incorrect or underspecified assumptions.

Once Blockers 1–3 are resolved, the model is ready for Phase 32 planning.

---

## Required Actions Before Phase 32 Planning Begins

**B1 — Add to milestone_async_0 work items:**
- Define `Type::Task(Box<Type>, Box<Type>)` for task handles
- Define `Type::Awaitable(Box<Type>)` as a structural protocol
- Define `Type::AsyncFunction` or a capability flag on `Callable` for async callable types
- Confirm awaitability is structural (any type implementing `Awaitable[T]` is awaitable)
- Add these types to `internal_docs/architecture.md`

**B2 — Add to milestone_async_0 work items:**
- Resolve scope conflict between `internal_docs/phases/32_async_ecosystem.md` and this model
- Enumerate specific HIR node additions: `HirStmt::AsyncFnDef`/`HirFunction::is_async`, `HirExpr::Await`, `HirExpr::TaskSpawn`, `HirStmt::AsyncWith`, `HirStmt::AsyncFor`
- Synchronize milestone names: either adopt 9-milestone model in roadmap, or adopt 4-milestone structure in model

**B3 — Add to milestone_async_7 work items:**
- Define LIFO cleanup order for `async with` under cancellation
- Define cancellation behavior when `__aexit__` is itself async
- Define panic handling: `catch_unwind` around `__aexit__`, secondary error surfacing, no double-panic abort
- Define nested cancellation: parent cancellation triggers child cancellation, each task unwinds independently
