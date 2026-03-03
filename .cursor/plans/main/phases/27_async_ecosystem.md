# Phase 27: Async and Ecosystem Foundation

**Why now:** Safety is solid, ownership model is proven, stdlib is deep and fully generic (Phase 13). The type system is complete — generics, pattern matching, enums, and auto-init are all in place. The codegen architecture is sound (Phase 14) — all new async codegen patterns will be built on structured IR, not string templates. The async runtime can be built on a stable, expressive foundation where generic types, exhaustive error handling, and clean class definitions are available from day one.

---

## milestone_async_core: Async Runtime Core

status: pending

**Goal:** Add the minimum viable async language support: `async def`/`await` syntax, Tokio runtime auto-bundling, and basic task spawning. This is the foundational compiler feature that all other async milestones build on.

**Depends on:** milestone_codegen_structural_passes (Phase 14 must be complete — the codegen architecture provides structured IR for all new async codegen patterns)

### Language Features

- `async def` / `await` -> Rust `async fn` / `.await`
- Tokio runtime auto-bundled when any `async def` is used
- `sifr.task`: `spawn`, `sleep`, `timeout`
- `try`/`except` auto-unwrap works across `.await` points (the compiler inserts `?` in HIR for `Result`-returning async calls inside `try` blocks, same as sync — no user-facing `?` operator)

### Compiler Changes

- Parser: `async def` already parsed (Python syntax). Validate `await` only appears inside `async def`.
- HIR: new `HirAwait` node. Async functions produce `Future` types.
- Type checker: `await` on a non-`Future` type is a compile error. `Result` auto-unwrap inside `try` blocks works across `.await` boundaries.
- Codegen: `async def` emits `async fn`. `await` emits `.await`. Main function with async calls gets `#[tokio::main]`.
- Send-bound diagnostics: `sifr.task.spawn` requires the spawned closure to be `Send + 'static`. The compiler must translate `rustc`'s Send-bound errors into Sifr-level diagnostics (e.g., "type X cannot be sent between tasks because field Y is not Send") rather than leaking raw Rust error messages. Full Send/Sync checking infrastructure is deferred to `milestone_async_sync`, but the spawn boundary must produce clear Sifr diagnostics from day one.

### Definition of Done (milestone_async_core)

- `async def` compiles to Rust `async fn`
- `await` compiles to `.await`
- Tokio runtime is automatically bundled when async is used
- `try`/`except` auto-unwrap works across `.await` points (compiler-internal `?` in HIR, not user-facing)
- `sifr.task.spawn` works for concurrent tasks
- `sifr.task.sleep` works for async delays
- `sifr.task.timeout` wraps an async call with a deadline
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: async_basic, await_chain, task_spawn, async_error_propagation, task_sleep, task_timeout
- E2E fail tests: spawn_non_send (clear Sifr diagnostic when spawning a non-Send type)
- Milestone demo in `./demos/milestone_async_core_demo.sifr`

---

## milestone_typed_serde_core: Typed Serialization (Core)

status: pending

**Goal:** Web-independent typed serialization. This does NOT include web extractors — those are delivered in a later web phase. Typed serde is kept in the async phase to make typed payload handling available early.

**Depends on:** milestone_async_core (async runtime must exist for async-compatible serde patterns; generics from Phase 13 enable `loads(s, T)`)

### Work Items

- Auto-derive `Serialize`/`Deserialize` on all classes
- `dumps(obj)` serializes any class to JSON string
- `loads(s, T)` deserializes JSON string to typed class, returns `Result[T, JSONDecodeError]`
- Nested classes, lists, dicts, optionals, unions serialize correctly
- E2E tests for typed JSON roundtrip independent of any web framework

### Definition of Done (milestone_typed_serde_core)

- Classes auto-derive `Serialize`/`Deserialize` — no manual annotation needed
- `dumps(obj)` serializes any class to JSON string
- `loads(s, T)` deserializes JSON string to typed class, returns `Result[T, JSONDecodeError]`
- Nested classes, lists, dicts, optionals, unions serialize correctly
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: typed_json_roundtrip, nested_class_serde, union_serde, optional_serde
- E2E fail tests: json_parse_wrong_type, missing_required_field

---

## milestone_async_sync: Async Synchronization Primitives

status: pending

**Goal:** Add cross-task synchronization primitives and Send/Sync checking at spawn boundaries. These are needed for production async code but are not required for basic async functionality.

**Depends on:** milestone_async_core (sync primitives are fundamental concurrency tools independent of networking; they depend only on the async runtime)

### Work Items

- `sifr.sync.Lock` — maps to `Arc<Mutex<T>>`, async-aware
- `sifr.sync.Channel` — maps to `tokio::sync::mpsc`, typed channels
- `sifr.sync.Semaphore` — maps to `tokio::sync::Semaphore`
- Send/Sync checking at spawn boundaries: when a value is sent to `sifr.task.spawn`, the compiler verifies the value is `Send`. If not, it emits a clear error explaining which field is not sendable (leverages borrow-by-default from Phase 10)
- Async closures captured across `.await` are checked for `Send + 'static`

### Definition of Done (milestone_async_sync)

- `sifr.sync.Lock` works for shared mutable state across tasks
- `sifr.sync.Channel` works for typed message passing between tasks
- `sifr.sync.Semaphore` works for concurrency limiting
- Send/Sync checking at spawn boundaries produces clear diagnostics
- Async closures are checked for `Send + 'static`
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: lock_basic, channel_basic, semaphore_basic, send_sync_check
- E2E fail tests: non_send_spawn (clear error for non-Send type in spawn)

---

## milestone_async_advanced: Advanced Async Features

status: pending

**Goal:** Add advanced async features that build on the core runtime and sync primitives. These are powerful but not needed for basic async applications.

**Depends on:** milestone_async_sync (sync primitives must exist for advanced patterns)

### Work Items

- `async with` — async context managers (`__aenter__` / `__aexit__`)
- Async generators — `yield` inside `async def` produces async iterators
- Async comprehensions — `[await x async for x in stream]`

### Definition of Done (milestone_async_advanced)

- `async with` works for async context managers
- Async generators (`yield` in `async def`) produce async iterators
- Async comprehensions compile correctly
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: async_with_basic, async_generator_basic, async_comprehension
- Milestone demo in `./demos/milestone_async_advanced_demo.sifr`

---

## Milestone Ordering

- **milestone_async_core first:** The async runtime must exist before anything else. This is the minimum viable async — `async def`/`await`, Tokio, basic task spawning.
- **milestone_typed_serde_core second:** Typed serialization is web-independent and stays in this phase.
- **milestone_async_sync third:** Synchronization primitives (Lock, Channel, Semaphore) and Send/Sync checking depend only on the async runtime.
- **milestone_async_advanced last:** Advanced features (async with, async generators, async comprehensions) build on everything above.

## Quality Contract
- Entry criteria: Phase 26 is completed and codegen architecture from Phase 14 remains intact.
- Exit criteria: Async runtime core, typed serialization core, sync primitives, and advanced async features are all delivered with regression coverage.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 27 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 27 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`
