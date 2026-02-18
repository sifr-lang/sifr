# Async and Ecosystem Foundation

**Why now:** Safety is solid, ownership model is proven, stdlib is deep and fully generic (Phase 13). The type system is complete — generics, pattern matching, enums, and auto-init are all in place. The async runtime can be built on a stable, expressive foundation where generic types, exhaustive error handling, and clean class definitions are available from day one.

---

## milestone_async: Async Runtime

status: pending

**Goal:** Add async/await language support. This is a language feature milestone — it adds the async primitives that the web framework builds on.

**Depends on:** milestone_stdlib_generic_rewrite (Phase 13 must be complete — the type system and generic stdlib provide the foundation for async features)

### Language Features

- `async def` / `await` -> Rust `async fn` / `.await`
- Tokio runtime auto-bundled
- `sifr.task`: spawn, sleep, timeouts
- `sifr.net`: TCP/UDP sockets (async)
- `async with`, async generators
- `sifr.sync`: Lock, Channel, Semaphore
- Send/Sync checking at spawn boundaries (leverages borrow-by-default from Phase 10)

### Definition of Done (milestone_async)

- `async def` compiles to Rust `async fn`
- `await` compiles to `.await`
- Tokio runtime is automatically bundled when async is used
- `?` operator works across `.await` points
- Async closures captured across `.await` are checked for `Send + 'static`
- `sifr.task.spawn` works for concurrent tasks
- `async with` works for async context managers
- Async generators (`yield` in `async def`) produce async iterators
- `sifr.sync.Lock`, `sifr.sync.Channel`, `sifr.sync.Semaphore` work for cross-task coordination
- E2E pass tests: async_basic, await_chain, task_spawn, async_error_propagation, async_with_basic, async_generator_basic, lock_basic, channel_basic
- Milestone demo in `./demos/milestone_async_demo.sifr`

---

## milestone_networking_stdlib: Networking Standard Library

status: pending

**Goal:** Add networking-related stdlib modules that depend on the async runtime.

**Depends on:** milestone_async (async runtime must exist)

### Modules

- `sifr.subprocess` — async Popen API (extends the sync `run()` from Phase 11's `milestone_new_modules` with async process management)
- `sifr.socket` — TCP/UDP
- `sifr.http` — HTTP client (wraps `reqwest`)
- `sifr.url` — URL parsing

### Definition of Done (milestone_networking_stdlib)

- Each networking module compiles and works with async I/O
- All fallible operations return `Result` or `Option`
- E2E pass tests: subprocess_async, socket_tcp, http_get, url_parse
- Integration with the async runtime (tokio) is seamless

---

## milestone_typed_serde_core: Typed Serialization (Core)

status: pending

**Goal:** Web-independent typed serialization. This does NOT include web extractors — those depend on the web framework and are delivered in Phase 15.

**Depends on:** milestone_networking_stdlib (networking modules should exist; typed serde benefits from the full async foundation)

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
- E2E pass tests: typed_json_roundtrip, nested_class_serde, union_serde, optional_serde
- E2E fail tests: json_parse_wrong_type, missing_required_field

---

## Milestone Ordering

- **milestone_async first:** The async runtime must exist before networking modules that require async I/O.
- **milestone_networking_stdlib second:** Networking modules bridge sync stdlib and web framework.
- **milestone_typed_serde_core third:** Typed serialization is web-independent but benefits from the full async/networking foundation being in place.
