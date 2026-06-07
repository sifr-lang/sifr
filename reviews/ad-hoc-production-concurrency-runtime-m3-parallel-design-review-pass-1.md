I have enough now — the phase contract, the implementation, the helper code, and the M3 entry-gate text are all in hand. Below is the blocker-only adjudication.

---

# Implementation-Readiness Review — M3 `sifr.parallel` First Wave

## Verdict: FAIL — must amend before PR

The phase contract is internally contradictory on this exact surface, and the implementation has resolved the contradiction by reaching for `std::process::abort()`. That resolution is not shippable. Three independent blockers below; the second is the root one.

---

## Blocker 1 — `std::process::abort()` in pool construction

`crates/sifr_codegen/src/preamble/parallel_runtime.rs:79-84`:

```rust
fn __sifr_build_parallel_pool(workers: usize) -> rayon::ThreadPool {
    match rayon::ThreadPoolBuilder::new().num_threads(workers).build() {
        Ok(pool) => pool,
        Err(_) => std::process::abort(),
    }
}
```

This is reachable from every `parallel.map`, `parallel.try_map`, and `Pool(config)` call. `ThreadPoolBuilder::build()` returns `Err` on OS thread-creation failure (`EAGAIN`, `RLIMIT_NPROC`, OOM during stack mmap, etc.). These are environmental but not "internal invariant" — they are reachable from user runtime context.

Direct violations:
- Phase line 27: "panic-free emitted Rust"
- Phase line 238: "no user-triggerable `.unwrap()`, `.expect()`, or `panic!`"
- Phase line 277 (Rayon row): "user CPU closures are wrapped so panics/failures become typed `WorkerRuntimeError`/`WorkerError` evidence rather than user-triggerable process panics"
- Phase line 606 (M3 scope): "Map task, worker, foreign/runtime boundary, and panic-like runtime failures into typed evidence"
- Quality contract line 882: "No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths"

`process::abort()` is strictly more severe than `panic!` (skips unwinding) and the phase wording explicitly forbids "user-triggerable process panics" — abort qualifies.

## Blocker 2 — No typed runtime-failure channel on the public surface (root cause)

The phase doc lists, in the same milestone, two requirements that cannot both be satisfied:

- The listed signatures (lines 588-593):
  - `parallel.map(items, fn) -> list[U]`
  - `parallel.try_map(items, fn) -> Result[list[U], E]` — `E` is the **user's** worker error
  - `Pool(config: PoolConfig)` returning `Pool`
- The typed-evidence requirements (lines 277, 342, 606): pool-construction failure and user-closure panic must become typed `WorkerRuntimeError`/`WorkerError` evidence.

There is no `WorkerRuntimeError`/`WorkerError` slot in the listed signatures. The implementation faithfully follows the signatures, so it has nowhere to put the runtime failure — and reaches for `abort()`. The contradiction is structural; it cannot be resolved by code changes alone.

The `try_map` signature is also wrong-shaped under the contract: phase line 342 lists `WorkerError`, `WorkerRuntimeError`, and `OffloadError` distinct from the user's `E`. `Result[list[U], E]` collapses worker-runtime failure into user `E`, which only works if `E` is `WorkerError[E_user]`. The phase doc does not say this; the implementation does not encode this. This same tension is flagged in the Resolved Decisions register for `TaskGroup` offload binding (line 898), and it explicitly cross-references the `JoinSet` shape — M0 was supposed to reconcile this **before** M3's first PR. The reconciliation is not in the ledger or in the M3 scope text.

## Blocker 3 — User closure panics are not wrapped

`__sifr_parallel_map`, `__sifr_parallel_try_map`, `__sifr_pool_map`, `__sifr_pool_try_map` call `pool.install(|| items.into_par_iter().map(worker).collect())`. Rayon catches panics inside worker closures via `catch_unwind` and **re-raises** them on the install-caller thread. That is a user-triggerable panic in generated Rust.

It is true that well-typed Sifr closures should not panic. But the phase contract is not "trust the type system"; it is line 277 verbatim: "user CPU closures are wrapped so panics/failures become typed `WorkerRuntimeError`/`WorkerError` evidence". No `catch_unwind` boundary exists in the emitted helper.

This blocker has the same root cause as Blocker 2: there is no error channel to put the typed evidence into.

---

## First-wave scope check

M3 scope (lines 578-594) owns six surface families: `spawn_blocking`, `spawn_cpu`, `JoinSet[T, E]`, `parallel.map`/`try_map`, `PoolConfig`, `Pool`. Shipping only `sifr.parallel` as a first wave is **not** itself a blocker — milestones may land in waves. But the `WorkerError`/`WorkerRuntimeError`/`OffloadError` typed-error map (phase line 342) is a cross-cutting type consumed by **every** M3 surface, and the M3 DoD (line 624) explicitly requires "Worker runtime failures become typed evidence". You cannot defer it; defining it after `sifr.parallel` ships forces a breaking signature change on the first user-facing surface in M3. So the typed-error shape must land **in or before** this first PR even if the JoinSet/spawn_* runtime code lands later.

---

## Required fix — concrete path

Adopt user option **A** (amend the `sifr.parallel` public API result shapes), with these specifics:

1. **Land the typed-error contract first**, in the same PR or a prerequisite PR:
   - `WorkerRuntimeError` (runtime/foreign boundary failure: pool construction, panic-converted-to-typed-evidence, foreign panic capture)
   - `WorkerError[E]` (sum of user `E` and `WorkerRuntimeError`)
   - These are the types referenced in phase lines 277, 342, 606. They are also the types `JoinSet.join_all().await -> list[Result[T, WorkerError[E]]]` (line 586) consumes — same wave's worth.

2. **Amend `sifr.parallel` signatures** (phase amendment via a new issue, since the doc currently locks the wrong shape):
   - `parallel.map(items, fn) -> Result[list[U], WorkerRuntimeError]`
   - `parallel.try_map(items, fn) -> Result[list[U], WorkerError[E]]`
   - `Pool.map` / `Pool.try_map` — same shape change
   - `Pool(config) -> Result[Pool, WorkerRuntimeError]` (or accept lazy pool construction at first call, surfacing in the call's `Result`). Eager fallible construction is cleaner; lazy is uglier but acceptable.

3. **Replace `process::abort()` in `__sifr_build_parallel_pool`** with a typed error return that the helper functions propagate.

4. **Add `catch_unwind` around the user closure invocation** inside the Rayon parallel iterator (per-item, so per-item panic maps to per-item `WorkerRuntimeError`). The natural place is wrapping the user `worker` into a closure that returns `Result<U, WorkerRuntimeError>` (for `map`) or merges into `WorkerError[E]` (for `try_map`). The early-exit semantics for `map`/`try_map` on first runtime failure are themselves a small design decision but a tractable one — record it in the execution ledger.

5. **Update the e2e fixtures** to reflect the new `Result`-shaped public surface. The two pass fixtures (`parallel_map_basic`, `parallel_try_map_basic`, `parallel_pool_map_basic`) currently bind `list[U]`; they need `.unwrap()`-equivalent typed handling at the Sifr level.

Why not the other paths:
- **B (keep shapes, internal non-panicking strategy)**: impossible. There is no non-panicking strategy for pool-construction failure that is not a fallback (sequential degradation forbidden) or a panic (forbidden). The error must surface.
- **C (defer until `WorkerError`/`JoinSet` exists)**: equivalent to A in effect — `WorkerError` has to be defined to land `sifr.parallel` correctly. A is just the explicit form.
- **D (compiler diagnostics/docs only)**: leaves the milestone without a runtime API and discards working code that would survive the amendment with a signature change only.

The blocker is real but the remediation is local: one new typed-error file, one signature amendment, one closure-wrap, and the `abort()` deletion. The lowering and sendability work already in `parallel_calls.rs` is largely correct and survives the change.

RESULT: FAIL
