I have verified each pass-1 blocker is closed, the post-pass-2 harness dependency fix, and the broader invariants. Summary follows.

---

# Implementation-Readiness Review — M3 `sifr.parallel` First Wave (PR-ready)

## Verdict: PASS — no remaining blockers for opening or merging the first-wave PR.

The three Pass-1 blockers stay closed in the current tree, the post-pass-2 Rayon harness fix is correctly scoped, and traceability still records the open M3 follow-ups honestly.

---

## Pass-1 blocker re-verification

**1. No `abort()`/`panic!`/data-dependent `.unwrap()`/`.expect()` in pool construction or worker execution.** `crates/sifr_codegen/src/preamble/parallel_runtime.rs:139-147` returns `Result<rayon::ThreadPool, WorkerRuntimeError>` from `__sifr_build_parallel_pool`. `Pool::new` (lines 86-106) stores the failure on the `Pool` value rather than aborting, and `__sifr_pool_map` / `__sifr_pool_try_map` (lines 211-220, 243-252) surface it on first call. A scan of the helper finds no `abort`, no `panic!`, and no data-dependent unwrap/expect.

**2. Typed runtime-failure channel on every public surface.** `lib/sifr/parallel.sifr:28-44` declares the amended signatures; `crates/sifr_lowering/src/lower/parallel_calls.rs:261-281` builds `Type::Result(list[U], WorkerRuntimeError|WorkerError)`; codegen routes through `__sifr_*` helpers at `crates/sifr_codegen/src/lower_expr/leaves_and_plain_calls.rs:603-630`; and the phase contract amendment is recorded in `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:588-593, 606` plus the resolved decision row at line 907 (Pool instance API).

**3. Rayon worker panics wrapped per-item.** Each of `__sifr_parallel_map`, `__sifr_parallel_try_map`, `__sifr_pool_map`, `__sifr_pool_try_map` wraps the worker invocation in `std::panic::catch_unwind(AssertUnwindSafe(...))` and converts it to `WorkerRuntimeError` or `WorkerError`. `__sifr_with_silent_parallel_panic_hook` (parallel_runtime.rs:149-158) is for observability only; the per-item conversion is independent of the hook.

## Post-pass-2 Rayon harness fix (delta scope)

`crates/sifr/tests/e2e_support/fixture_compilation.rs` adds two arms in `generate_cargo_toml`:
- line 305-307: `"sifr.parallel"` in `stdlib_modules` → `rayon = "1.12.0"`
- line 409-411: `"rayon"` in `required_crates` → `rayon = "1.12.0"`

This is symmetric with how every other crate is mapped and is the correct grouped-test fix because `build_batch_group` (batch_execution.rs:55-65) unions `case.stdlib_modules` and `case.required_crates` across cases and passes them straight into `generate_cargo_toml`. Either entry is sufficient to emit the dep:
- `StdlibFeature::Rayon` (id = `"rayon"`, see `crates/sifr_stdlib/src/features.rs:59`) flows into `required_crates` via `compile_source_with_metadata` whenever the preamble contains `rayon::`, which itself is only emitted when `module_name == "sifr.parallel"` (lib_modules_and_codegen.rs:370-371) and gated again at lib_modules_and_codegen.rs:727-729.
- The stdlib_modules arm covers the case where a fixture imports `sifr.parallel` but the dep set hasn't been re-inferred.

No runtime feature-gating issue is hidden: non-parallel fixtures cannot reach either trigger because the preamble injection is module-gated, so `rayon` is added only when actually used. The earlier miss was the symmetric arm being absent for grouped builds, not a leak of rayon into unrelated builds.

I confirmed end-to-end:
- `cargo check -p sifr_lowering -p sifr_codegen -p sifr_stdlib -p sifr` → clean.
- `cargo run -p sifr -- run` on `parallel_map_basic.sifr` and `parallel_map_worker_panic_typed.sifr` → pass.
- `cargo run -p sifr -- check` on both fail fixtures emits `SIFR-ASYNC-0004` and `SIFR-TYPE-0002` at the expected ranges.

## Sendability / typed-error diagnostics

`parallel_calls::validate_parallel_map_like_call` (parallel_calls.rs:103-216) rejects:
- direct calls in `async def` bodies with `SIFR-ASYNC-0004`,
- keyword args and wrong arity,
- non-list first args,
- non-send item types via `non_send_reason` with `SIFR-TYPE-0002`,
- non-function worker args, wrong worker arity, plain-vs-Result worker mismatch,
- non-send output and non-send error types,
- and tracks ownership move on the items list for owned-list semantics.

The two fail fixtures cover the async-direct and non-send-item paths; the pass fixtures cover map / try_map / Pool.map ordering, worker panic conversion, and user-error formatting.

## Manifests and traceability

- `verification/validation_lanes/create_pr_e2e_manifest.json` (lines 67-71) and `verification/validation_lanes/merge_e2e_manifest.json` (lines 82-86) both include the five new pass fixtures.
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:32-41` enumerates the explicit M3 follow-ups still open (spawn_cpu, scoped offload, JoinSet[T, E]/JoinItemId, ordered join_all/cancel_all, drop diagnostics, generic WorkerError[E], wider closure-capture sendability, lazy private default pool shutdown design, per-call panic-hook suppression). Wave-1 is not represented as M3 closure.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:405` keeps M3 PR row as "pending."

## Non-blocking observations (carried forward, do not gate the PR)

These were already flagged in pass-2 and remain accurate:
1. `__sifr_with_silent_parallel_panic_hook` mutates the global panic hook; concurrent independent OS-thread callers can race on installation. The contract is satisfied by per-item `catch_unwind`; this is observability-only.
2. `__sifr_parallel_try_map` collapses user `E` to `WorkerError::new(format!("{}", error))`. Phase line 606 admits this for wave-1.
3. Top-level `parallel.map` / `parallel.try_map` rebuild a fresh private Rayon pool per call rather than using a lazy private default pool; the traceability ledger records the gap.
4. `PoolConfig.workers: int` is clamped to ≥1 instead of being typed `PositiveInt`; reasonable wave-1 because Sifr does not yet expose `PositiveInt`.
5. The Sifr-side `try_map` body in `lib/sifr/parallel.sifr` is a no-op stub (stripped by `replace_parallel_runtime_items`); a short comment in the .sifr noting this would help future readers but is not blocking.

---

RESULT: PASS
