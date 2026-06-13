I have enough now — the phase contract, the remediated runtime helper, the public Sifr surface, the HIR lowering, the fixtures, and the traceability ledger are all in hand. Below is the blocker-only adjudication.

---

# Implementation-Readiness Review — M3 `sifr.parallel` First Wave, Pass 2

## Verdict: PASS — acceptable to proceed as the wave PR with the recorded gaps as follow-up M3 work.

The three blockers from Pass 1 are all closed, the phase contract has been amended to admit the new shapes, and the remaining M3 work is explicitly listed in the traceability ledger and the phase doc.

---

## Pass 1 blocker closure

**Blocker 1 (`std::process::abort()` in pool construction)** — closed.
`crates/sifr_codegen/src/preamble/parallel_runtime.rs:139-147` now returns `Result<rayon::ThreadPool, WorkerRuntimeError>` from `__sifr_build_parallel_pool`. A grep across the whole helper finds no `abort`, no data-dependent `.unwrap()`/`.expect()`, and no `panic!`. Pool construction failure is propagated up as typed evidence both in the top-level helpers (`__sifr_parallel_map`/`__sifr_parallel_try_map`, lines 165 and 187) and in `Pool::new`, which records the failure on the pool value (lines 86-106) and surfaces it on the first `map`/`try_map` call (lines 211-220 and 243-252). Phase lines 27, 238, 277, 606, and 882 are now consistent with the implementation.

**Blocker 2 (no typed runtime-failure channel on the public surface)** — closed.
Phase line 588-593 has been amended to:
- `parallel.map(items, fn) -> Result[list[U], WorkerRuntimeError]`
- `parallel.try_map(items, fn) -> Result[list[U], WorkerError]`
- `Pool.map`/`Pool.try_map` mirror the same shapes.

`lib/sifr/parallel.sifr` declares those signatures, the Sifr→Rust lowering in `crates/sifr_lowering/src/lower/parallel_calls.rs:261-281` constructs the corresponding `Type::Result(list[U], WorkerRuntimeError|WorkerError)`, the codegen helpers in `crates/sifr_codegen/src/lower_expr/leaves_and_plain_calls.rs:603-630` route to the four `__sifr_*` helpers, and the runtime helpers return the right shapes. The fixtures `parallel_map_basic`, `parallel_try_map_basic`, and `parallel_pool_map_basic` bind the `Result`-shaped output, matching the new contract.

The wave-1 non-generic `WorkerError` (no `[E]`) is a deliberate, documented scope choice: the phase amendment at line 606 records `WorkerError` as the wave-1 wrapper and explicitly defers `WorkerError[E]` to the `JoinSet`/scoped offload wave. The traceability ledger lists this gap, and the user-error string is preserved through `format!("{}", error)` so observable evidence is not lost. This does not violate a phase invariant and does not force a breaking API: adding a type parameter later is additive at the type level, and the contract amendment is recorded in the same wave the surface lands.

**Blocker 3 (worker panics not wrapped)** — closed.
Each Rayon worker invocation in `__sifr_parallel_map` (lines 170-173), `__sifr_parallel_try_map` (lines 193-198), `__sifr_pool_map` (lines 225-228), and `__sifr_pool_try_map` (lines 257-262) is wrapped in `std::panic::catch_unwind(AssertUnwindSafe(...))` and converted to `WorkerRuntimeError`/`WorkerError`. The install body runs under `__sifr_with_silent_parallel_panic_hook` (lines 149-158) so a converted panic does not also print as an unhandled runtime panic. `parallel_map_worker_panic_typed` exercises this end-to-end.

---

## Wave-scope check

- Phase line 588-606 records the wave shapes; lines 595-606 record the runtime contract; line 606 records the wave-1 vs. JoinSet split for `WorkerError` typing.
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md` lists the wave fixtures (`parallel_map_basic`, `parallel_try_map_basic`, `parallel_pool_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, plus the two fail fixtures) and explicitly enumerates the remaining M3 follow-ups: `spawn_cpu`, scoped offload methods, `JoinSet[T, E]`/`JoinItemId`, ordered `join_all`/`cancel_all`, live-set drop diagnostics, generic `WorkerError[E]`, fuller closure-capture sendability, and the lazy private default pool shutdown design.
- `verification/validation_lanes/create_pr_e2e_manifest.json` and `merge_e2e_manifest.json` both include the five wave fixtures.
- Generated `Cargo.toml` rayon emission is gated by `features_for_stdlib_module("sifr.parallel")` in `crates/sifr_stdlib/src/features.rs:412` plus the preamble `stdlib_preamble.contains("rayon::")` gate in `crates/sifr_codegen/src/lib_modules_and_codegen.rs:727-729`, so non-parallel projects do not pull in Rayon.

None of the recorded follow-ups force a breaking public-API change to this wave's `parallel.map`/`try_map`/`Pool.map`/`Pool.try_map` shapes, and none violate a phase invariant (the cross-cutting `WorkerRuntimeError`/`WorkerError` typed-error map now exists in this PR, satisfying the M3 DoD precondition that consumers of those types — `JoinSet`, `spawn_cpu`, etc. — depend on).

---

## Non-blocking observations (not gating this wave)

1. `__sifr_with_silent_parallel_panic_hook` uses the global `std::panic::set_hook`. Concurrent top-level callers of `parallel.map` from independent OS threads can race on hook installation. The contract is still satisfied because every worker invocation is independently wrapped in `catch_unwind`, but a follow-up wave should replace the global suppression with per-call `panic::update_hook` or a thread-local suppression flag. Track with the rest of the M3 panic-boundary work.
2. `__sifr_parallel_try_map` collapses user `E` to `WorkerError::new(format!("{}", error))`. Once `WorkerError[E]` lands with JoinSet, the formatter call should become a structural wrap. The current `Display`-based capture is good enough as observable evidence for the wave-1 fixtures.
3. The Sifr-side `try_map` body in `lib/sifr/parallel.sifr` is a no-op stub (no loop). It is stripped by `replace_parallel_runtime_items` before emission and so does not run, but if anything ever bypassed that strip step the user would see empty results rather than a runtime helper invocation. Worth a short comment in `parallel.sifr` noting the runtime is replaced at codegen.

None of these block the wave PR.

RESULT: PASS
