# Phase 32: Async and Ecosystem Foundation

Status note: superseded for planning by `../async_concurrency_model.md`. The older 4-milestone structure in this file is retained as historical context until Phase 32 is rewritten. Implementation planning must follow the 9-milestone async/concurrency model (`milestone_async_0` through `milestone_async_8`) unless that model is explicitly amended. Older references below to subprocess or signal delivery are not Phase 32 v1 exit criteria.

**Why now:** Safety, ownership, diagnostics, and stdlib parity are established enough that Sifr can add concurrency without importing Python's sharp edges wholesale. This phase must turn async into a simple, explicit model that fits Sifr's borrow-by-default and `Result`/`Option` contracts instead of exposing the full complexity of Python's event-loop ecosystem.

**Scope note:** Typed serialization and validation remain deferred to Phase 40. That matters directly for concurrency: anything that needs stable cross-process transport of arbitrary values is not in scope until Sifr has a canonical typed data/IPC story.

## Design Principles

- **One canonical async model:** user code should learn `async def`, `await`, and `sifr.task`. Sifr should not require explicit event-loop objects, loop policies, or callback-style orchestration as primary APIs.
- **Structured concurrency first:** parent scopes own child tasks. "Fire-and-forget" work must be explicit and rare.
- **Async is for waiting, not for CPU:** CPU-bound work and blocking OS calls must go through explicit offload APIs (`spawn_blocking`, thread-pool executors), never the cooperative task scheduler by accident.
- **No implicit shared mutable memory:** cross-task or cross-thread sharing must use explicit primitives. The compiler must never silently invent `Arc`, `Mutex`, or equivalent wrappers.
- **Compatibility is layered, not defining:** CPython parity matters, but low-level `asyncio` internals should be exposed only when they still make sense in Sifr's model.
- **Cancellation and shutdown are part of the contract:** timeouts, task cancellation, subprocess teardown, and signal-driven shutdown cannot be left as ad hoc library details.

---

### milestone_32_1: Async Runtime Core and Task API

status: pending

**Goal:** Establish the canonical async execution model: `async def`, `await`, automatic runtime bootstrapping, and the core `sifr.task` APIs. This milestone should make ordinary async I/O code straightforward without introducing Python's loop-management complexity.

**Depends on:** Phase 14 codegen structure, Phase 27 diagnostics/runtime-safety contract, Phase 10 borrow-by-default, and Phase 31 compatibility hardening.

### Language and Runtime Features

- `async def` / `await` lower to Rust async functions and `.await`
- Async entrypoints auto-bootstrap the runtime; there is no user-visible event-loop object in the primary model
- `sifr.task.spawn` returns an awaitable task handle
- `sifr.task.sleep`, `sifr.task.timeout`, and task-handle `cancel` / `join`
- `try`/`except` auto-unwrap works across `.await` points exactly as it does in sync code

### Compiler and Runtime Changes

- Parser/lowering validate `await` placement and track async function boundaries
- HIR gains canonical async/task nodes instead of encoding async behavior ad hoc in expressions
- Type checker enforces that only awaitable values may be awaited
- Runtime selection is automatic when async is used; the compiler wires required runtime crates without user configuration
- Spawn-boundary diagnostics translate `Send + 'static` failures into Sifr diagnostics rather than leaking raw Rust errors
- Known blocking stdlib calls used directly in async contexts should produce targeted diagnostics where Sifr can prove the misuse

### Definition of Done (milestone_32_1)

- `async def` compiles and runs correctly
- `await` lowering is correct and rejects non-awaitable operands
- Async entrypoints auto-bootstrap the runtime
- `sifr.task.spawn`, `sleep`, `timeout`, task-handle `join`, and task-handle `cancel` work
- `try`/`except` auto-unwrap remains correct across await points
- All existing E2E tests still pass
- `cargo test` passes, `cargo clippy --workspace -- -D warnings` passes, and no new unjustified `unsafe` is introduced
- E2E pass tests include: `async_basic`, `await_chain`, `task_spawn`, `task_join`, `task_cancel`, `async_error_propagation`, `task_sleep`, `task_timeout`
- E2E fail tests include: `await_outside_async`, `await_non_awaitable`, `spawn_non_send`
- Milestone demo in `./demos/m32_async_core_demo.sifr`

---

### milestone_32_2: Structured Concurrency, Cancellation, and Selection

status: pending

**Goal:** Make concurrent composition safer and simpler than Python's traditional `asyncio` style by making structured concurrency the default and defining cancellation/selection semantics explicitly.

**Depends on:** milestone_32_1

### Work Items

- `sifr.task.TaskGroup` / `sifr.task.scope` as the preferred way to run sibling tasks
- `sifr.task.gather` for "await all" composition with deterministic result ordering
- `sifr.task.select` / `sifr.task.race` for "first completion wins" composition
- Cancellation propagation rules:
  - timeout cancels the enclosed operation
  - task-group failure cancels unfinished siblings
  - cancellation is observable and typed, not an ambient exception leak
- `async with` for async context managers
- Async iteration protocol and `async for` for streaming/task-produced values

### Design Decisions Locked in This Milestone

- Structured task groups are the default concurrency surface; detached tasks, if exposed at all, must be explicit (`spawn_detached`) and outside the "pit of success"
- Selection is task-level (`select` / `race`), not file-descriptor-level, for ordinary user code
- Async generators and async comprehensions are **not** exit-gate features for this phase; they can be revisited later once task, stream, and cancellation semantics are solid

### Definition of Done (milestone_32_2)

- `TaskGroup` / scoped-task composition works
- Sibling cancellation on failure is deterministic and covered by regression tests
- `gather`, `select`, and `race` work with documented cancellation behavior
- `async with` works for async context managers
- Async iterables compile and `async for` works
- All existing E2E tests still pass
- `cargo test` passes, `cargo clippy --workspace -- -D warnings` passes, and no new unjustified `unsafe` is introduced
- E2E pass tests include: `task_group_basic`, `task_group_error_cancels_siblings`, `task_gather`, `task_select`, `task_race`, `async_with_basic`, `async_for_channel`
- E2E fail tests include: `cancelled_task_use`, `async_with_missing_protocol`, `async_for_non_async_iterable`
- Milestone demo in `./demos/m32_structured_concurrency_demo.sifr`

---

### milestone_32_3: Explicit Sharing, Synchronization, and CPU-Bound Offload

status: pending

**Goal:** Define the "Rust-style but simpler" concurrency story for Sifr: task-local by default, explicit sharing when needed, and a separate path for CPU-bound or blocking work.

**Depends on:** milestone_32_2

### Work Items

- `sifr.sync.Shared[T]` for cheap shared ownership of immutable data
- `sifr.sync.Lock[T]` and `sifr.sync.RwLock[T]` for shared mutable state
- `sifr.sync.Channel[T]` as the primary safe producer/consumer primitive
- `sifr.sync.Semaphore` and `sifr.sync.Notify` for coordination and concurrency limiting
- Send/Sync diagnostics at task/thread boundaries, including captured async state across await points
- Rejection of borrowed values that would escape across task boundaries or live across invalid await points
- `sifr.task.spawn_blocking` for short blocking/CPU-bound offload from async code
- `sifr.concurrent.ThreadPoolExecutor` as a compatibility/ergonomics layer over the blocking pool and thread-based execution substrate

### Important Scope Boundaries

- `Channel` is the primary Sifr answer for queue-like concurrency; a future `sifr.queue` module may wrap it for CPython compatibility, but queue types are not the canonical primitive
- `Shared`, `Lock`, and `RwLock` are the Sifr surface; raw `Arc<Mutex<T>>`-style concepts remain implementation details
- `ProcessPoolExecutor` is **deferred** until Sifr has a stable typed IPC/serialization contract. Shipping process pools before that would force an unstable transport format into the language

### Definition of Done (milestone_32_3)

- Immutable shared-state handles, locks, rwlocks, channels, semaphores, and notifications work
- `spawn_blocking` prevents CPU-bound/blocking work from occupying the async scheduler
- `ThreadPoolExecutor` works for thread-based parallel tasks
- Send/Sync and borrow-across-await diagnostics are clear and Sifr-native
- All existing E2E tests still pass
- `cargo test` passes, `cargo clippy --workspace -- -D warnings` passes, and no new unjustified `unsafe` is introduced
- E2E pass tests include: `shared_basic`, `lock_basic`, `rwlock_basic`, `channel_basic`, `channel_backpressure`, `semaphore_basic`, `notify_basic`, `spawn_blocking_basic`, `thread_pool_executor_basic`
- E2E fail tests include: `non_send_spawn`, `borrow_across_await_rejected`, `shared_mut_without_sync_rejected`
- Milestone demo in `./demos/m32_sync_and_blocking_demo.sifr`

---

### milestone_32_4: Async Ecosystem Surfaces and Compatibility Layer

status: pending

**Goal:** Add the async-facing stdlib and compatibility surfaces that matter in real programs without letting the compatibility layer redefine the core model.

**Depends on:** milestone_32_3

### Work Items

- `sifr.subprocess` async process API:
  - `Popen` / `Process`
  - `wait`, `communicate`, `poll`, `terminate`, `kill`
  - async stdin/stdout/stderr pipes
  - timeout-aware process management
- `sifr.signal` revisited for async-safe shutdown:
  - `ctrl_c()`
  - `terminate()`
  - `shutdown_token()` / `shutdown_channel()` for graceful shutdown orchestration
- `sifr.asyncio` as a **compatibility veneer** over `sifr.task` / `sifr.sync`, not as the primary design center
  - approved initial surface: `run`, `create_task`, `gather`, `TaskGroup`, `sleep`, `wait_for`, `Queue`
- `sifr.concurrent` parity layer:
  - `Future`
  - `ThreadPoolExecutor`
  - `ProcessPoolExecutor` remains deferred pending typed IPC/serialization
- `selectors` decision:
  - runtime internals may use Tokio/mio-style readiness machinery
  - a public `sifr.selectors` module is **not required** for Phase 32 exit unless low-level socket work proves it necessary
  - if exposed later, it should be a thin compatibility module with curated tests, not the core user-facing model

### CPython Parity Strategy

- Use CPython tests as the behavioral source for `asyncio`, `subprocess`, `signal`, `queue`, and `concurrent.futures`
- Port only the subsets that still make sense under Sifr's contracts:
  - `Result`/`Option` instead of exception propagation
  - structured cancellation instead of ambient loop-driven behavior
  - no user-visible event-loop policy/configuration surface in the primary model
- Keep low-level `asyncio` loop-policy, callback, transport/protocol, and raw-future internals out of the merge-blocking parity scope

### Explicit Deferrals

- raw event-loop APIs and policy objects
- transport/protocol callback APIs
- public `selectors` unless demanded by low-level socket work
- `contextvars`
- `multiprocessing`
- `ProcessPoolExecutor`
- async generators and async comprehensions

### Definition of Done (milestone_32_4)

- Async subprocess management works and is cancellation/timeout aware
- Async-safe graceful shutdown primitives exist in `sifr.signal`
- The approved `sifr.asyncio` compatibility subset works on top of the canonical task/sync model
- `ThreadPoolExecutor` parity behavior is covered; `ProcessPoolExecutor` remains documented as deferred
- Curated CPython-derived parity tests exist for approved surfaces and intentional divergences are recorded
- All existing E2E tests still pass
- `cargo test` passes, `cargo clippy --workspace -- -D warnings` passes, and no new unjustified `unsafe` is introduced
- E2E pass tests include: `async_subprocess_wait`, `async_subprocess_communicate`, `signal_ctrl_c_shutdown`, `asyncio_task_group_subset`, `asyncio_wait_for_subset`, `concurrent_thread_pool_subset`
- E2E fail tests include: `subprocess_timeout_cancels`, `signal_handler_invalid_context`, `process_pool_not_yet_available`
- Milestone demo in `./demos/m32_async_ecosystem_demo.sifr`

---

## Milestone Ordering

- **milestone_32_1 first:** establish syntax/runtime/task basics before higher-level composition
- **milestone_32_2 second:** lock down structured concurrency and cancellation semantics before exposing broad compatibility layers
- **milestone_32_3 third:** add explicit sharing and blocking/parallel execution once task semantics are stable
- **milestone_32_4 last:** layer ecosystem-facing modules and CPython compatibility shims on the canonical model, not the other way around

## Quality Contract

- Entry criteria: Phase 31 is completed and Phase 27 runtime-safety/diagnostics guarantees remain green
- Phase 27 non-regression invariants remain mandatory: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, spans, URLs, severity, suggestions); canonical diagnostics model with renderer-only presentation differences; deterministic recovery/ordering; stable exit-code behavior
- No fallback, migration, or legacy compatibility architecture is allowed; implement the canonical async/concurrency model directly
- No hidden concurrency upgrades are allowed: the compiler/runtime must never silently insert shared-memory wrappers or detach work that should remain structured
- CPU-bound execution must have an explicit offload path; merge-blocking validation must prove the async scheduler is not the accidental execution path for blocking workloads
- Validation evidence must be recorded in the phase execution checklist issue before merge
- Each milestone requires at least one positive-path and one negative-path validation case tied to its documented goals

### Validation Planning Goals

- `milestone_32_1`: validate async lowering, runtime bootstrapping, core task APIs, and await-time diagnostics
- `milestone_32_2`: validate task-group behavior, structured cancellation, selection semantics, and async resource/iteration flows
- `milestone_32_3`: validate explicit sharing primitives, send/sync enforcement, borrow-across-await rejection, and blocking/CPU offload behavior
- `milestone_32_4`: validate async subprocesses, graceful shutdown/signal handling, approved compatibility subsets, and documented deferrals

## Exit Gate

- Async runtime core is delivered
- Structured concurrency, cancellation, and task-selection semantics are delivered
- Explicit sharing/synchronization primitives and CPU-bound offload paths are delivered
- Async subprocess and signal integration are delivered
- Compatibility layers (`sifr.asyncio`, `sifr.concurrent`) cover the approved subset only, with intentional divergences documented
- Phase 27 non-regression contract remains green
