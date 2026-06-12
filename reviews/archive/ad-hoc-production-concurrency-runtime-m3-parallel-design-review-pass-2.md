# Implementation-Readiness Review — M3 `sifr.parallel` First Wave (Pass 2)

## Verdict: PASS — wave-1 may proceed as a PR; recorded follow-ups remain open M3 work.

The three Pass 1 blockers are closed. The phase contract has been amended to admit the typed-result public shapes the implementation actually emits, the runtime helper no longer reaches for `process::abort()`, and every Rayon worker invocation is wrapped in `catch_unwind` that converts panics into typed evidence. The wave boundary is documented in the phase contract, the execution ledger, and the M3 traceability doc.

---

## Pass-1 Blocker Verification

### Blocker 1 (`std::process::abort()` in pool construction) — closed

`crates/sifr_codegen/src/preamble/parallel_runtime.rs:139-147`:

```rust
fn __sifr_build_parallel_pool(workers: usize) -> Result<rayon::ThreadPool, WorkerRuntimeError> {
    match rayon::ThreadPoolBuilder::new().num_threads(workers).build() {
        Ok(pool) => Ok(pool),
        Err(error) => Err(WorkerRuntimeError::new(format!(
            "parallel worker pool could not start: {}",
            error
        ))),
    }
}
```

Grep across the whole helper file shows no `abort`, no `panic!`, no `.unwrap()` and no `.expect()`. Pool-construction failure is propagated through `Result` in `__sifr_parallel_map` / `__sifr_parallel_try_map` (parallel_runtime.rs:165, 187-188) and stored on the `Pool` for later surfacing in `Pool.map` / `Pool.try_map` (parallel_runtime.rs:86-106, 211-220, 243-252). Phase invariants on lines 27, 238, 277, 606, and 882 of the phase contract are now consistent with the emitted code.

### Blocker 2 (no typed runtime-failure channel on the public surface) — closed

The phase contract was amended in this branch (issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:588-593, 606) to:

- `parallel.map(items, fn) -> Result[list[U], WorkerRuntimeError]`
- `parallel.try_map(items, fn) -> Result[list[U], WorkerError]`
- `Pool.map` / `Pool.try_map` mirror the same Result shapes.
- Wave-1 explicitly accepts non-generic `WorkerError` and defers `WorkerError[E]` to the JoinSet/scoped-offload wave.

`lib/sifr/parallel.sifr:28-44` declares those signatures. The Sifr → HIR lowering builds the right `Type::Result(list[U], WorkerRuntimeError|WorkerError)` shape (`crates/sifr_lowering/src/lower/parallel_calls.rs:261-281`). The codegen helpers route to `__sifr_parallel_map`, `__sifr_parallel_try_map`, `__sifr_pool_map`, `__sifr_pool_try_map`, all of which return the matching Rust `Result`. The five pass fixtures bind the Result-shaped surface (`parallel_map_basic`, `parallel_try_map_basic`, `parallel_pool_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`).

`Pool(config) -> Pool` (total constructor) was retained from the phase contract; pool-construction failure is stored on the `Pool` value and surfaced at the first `map` / `try_map` call. Pass 1 accepted this as an acceptable resolution path (review pass-1, Required fix #4: "lazy is uglier but acceptable").

### Blocker 3 (Rayon worker panics not wrapped) — closed

Every worker invocation is wrapped:

- `__sifr_parallel_map` (parallel_runtime.rs:170-173) → `WorkerRuntimeError`
- `__sifr_parallel_try_map` (parallel_runtime.rs:193-199) → `WorkerError`
- `__sifr_pool_map` (parallel_runtime.rs:225-228) → `WorkerRuntimeError`
- `__sifr_pool_try_map` (parallel_runtime.rs:257-262) → `WorkerError`

Each uses `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker(item)))` per-item, so per-item panics map to per-item typed evidence. `__sifr_with_silent_parallel_panic_hook` (parallel_runtime.rs:149-158) suppresses the default panic-message stderr output during the install body so test assertions on the returned error remain clean. The `parallel_map_worker_panic_typed` fixture exercises this path end-to-end.

### Wave-1 boundary documented honestly — confirmed

- Phase contract line 606 records the wave-1 vs JoinSet/scoped-offload split for `WorkerError` typing.
- Execution ledger (issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:405): "M3: pending."
- M3 traceability doc (verification/stdlib/concurrency_runtime_m3_offload_traceability.md:34-41) explicitly lists the remaining M3 work: `spawn_cpu`, scoped offload methods, `JoinSet[T, E]`/`JoinItemId`, ordered `join_all` / `cancel_all`, live-set drop diagnostics, generic `WorkerError[E]`, fuller closure-capture sendability, and the lazy private default pool shutdown design.
- Execution ledger Rayon pool architecture row was amended to acknowledge the fresh-per-call top-level pool deviation from "lazily initialized" until M3 closure.

The wave is not represented as full M3 closure.

---

## New-blocker scan

None. The recorded follow-ups do not force a breaking change to the wave-1 public-API shapes — adding a type parameter to `WorkerError` later is additive at the user signature site, and the lazy default pool change is internal to the runtime helper.

---

## Non-blocking observations (not gating wave-1 PR)

1. `__sifr_with_silent_parallel_panic_hook` mutates `std::panic::set_hook` globally. Concurrent top-level `parallel.map` callers (e.g., from sibling `spawn_cpu` tasks) can race on hook installation and leave the silent hook installed across the process. The contract requirement — panics become typed evidence — is satisfied independently by the per-worker `catch_unwind`, so the race is observability-only, but a follow-up should replace the global suppression with a thread-local flag or `panic::update_hook`-style approach.
2. `__sifr_parallel_try_map` collapses user `E` to `WorkerError::new(format!("{}", error))`. Phase line 606 explicitly accepts this for wave-1; when `WorkerError[E]` lands with JoinSet/scoped offload, the `Display`-based capture should become a structural wrap.
3. Top-level `parallel.map` / `parallel.try_map` rebuilds a fresh private Rayon pool on every call (`__sifr_build_parallel_pool` invoked inside the helpers). The phase doc still says "lazily initialized private default `rayon::ThreadPool`"; the M3 traceability records this gap. Worth a short comment in the helper itself pointing at the traceability entry.
4. `lib/sifr/parallel.sifr:42-44` defines `try_map` as a no-op stub that returns an empty `list[U]`. It is stripped by `replace_parallel_runtime_items` before emission, so the user-observable runtime is correct, but a short comment in `parallel.sifr` noting that the runtime helper replaces this body at codegen would prevent future confusion.
5. `PoolConfig` accepts any `int` and clamps non-positive values to one worker via `__sifr_parallel_worker_count`. The phase contract specifies `PoolConfig { workers: PositiveInt }`; current behavior is a silent clamp rather than a typed positive integer. Reasonable for wave-1 since Sifr does not currently expose a `PositiveInt` type, but worth either adding a runtime validation diagnostic or recording the clamp as a deliberate decision.

---

RESULT: PASS
