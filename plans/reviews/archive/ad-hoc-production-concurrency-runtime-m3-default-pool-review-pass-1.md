# Review — M3 Default Parallel Pool Closure Wave (Pass 1)

Branch: `codex/concurrency-runtime-m3-default-pool`
Scope: top-level `sifr.parallel.map` / `try_map` default-pool reuse via process-lifetime `OnceLock`-backed private Rayon pool.

## Verdict

**RESULT: PASS**

## Contract checks

### 1. Single private default pool per process; no per-call rebuild

`crates/sifr_codegen/src/preamble/parallel_runtime.rs:99-120` declares
`static __SIFR_DEFAULT_PARALLEL_POOL: std::sync::OnceLock<Result<rayon::ThreadPool, WorkerRuntimeError>>` and exposes
`__sifr_default_parallel_pool()` which dispatches through `OnceLock::get_or_init`. The init closure calls
`__sifr_build_parallel_pool(__sifr_default_parallel_worker_count())`, so construction runs at most once for the process.

Top-level entry points were rewired:

- `__sifr_parallel_map` at `parallel_runtime.rs:138` now does `let pool = __sifr_default_parallel_pool()?;` — was `__sifr_build_parallel_pool(__sifr_default_parallel_worker_count())?`.
- `__sifr_parallel_try_map` at `parallel_runtime.rs:160` does `__sifr_default_parallel_pool().map_err(__sifr_worker_error_from_runtime)?;` — same swap.

Confirmed against emitted Rust via `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/parallel_default_pool_reused.sifr`:
the emitted top-level `__sifr_parallel_map` / `__sifr_parallel_try_map` no longer contain a fresh
`__sifr_build_parallel_pool(__sifr_default_parallel_worker_count())` call. Only `__sifr_default_parallel_pool()` is invoked,
and `__sifr_build_parallel_pool` is reached either through the `OnceLock` init closure (one-time) or through `Pool::new`.

OnceLock soundness: `rayon::ThreadPool: Send + Sync` (designed for shared use via `install`), and `WorkerRuntimeError`
in emitted code derives `Clone` (verified in emit at the generated `#[derive(Debug, Clone)] struct WorkerRuntimeError`),
so `Err(error.clone())` on cached-failure paths is well-typed.

### 2. Typed pool-construction failure preserved

- `__sifr_parallel_map` returns `Result<Vec<U>, WorkerRuntimeError>` and propagates the cached `WorkerRuntimeError` via
  `?` from `__sifr_default_parallel_pool()`.
- `__sifr_parallel_try_map` returns `Result<Vec<U>, WorkerError>` and goes through
  `.map_err(__sifr_worker_error_from_runtime)?` to convert the cached `WorkerRuntimeError` into a `WorkerError` with
  the runtime message preserved (`WorkerError::new(error.message)` at `parallel_runtime.rs:241-243`).

This matches the `WorkerRuntimeError` for `map` / `WorkerError` for `try_map` contract.

### 3. Configured `Pool(config)` semantics unchanged

- `Pool::new` still calls `__sifr_build_parallel_pool(workers)` directly (`parallel_runtime.rs:47-65`); the configured
  pool is owned by the `Pool` struct and drops with it.
- `__sifr_pool_map` / `__sifr_pool_try_map` still consume `pool._pool.as_ref()` and surface `pool._failure.clone()`
  (`parallel_runtime.rs:177-239`). No changes in this wave.
- The diff (`git diff HEAD -- crates/sifr_codegen/src/preamble/parallel_runtime.rs`) only touches the static, the new
  helper, and the two top-level entry-point bodies — no edits to `Pool`, `__sifr_pool_map`, or `__sifr_pool_try_map`.

### 4. Validation coverage and traceability honesty

- New fixture `crates/sifr/tests/e2e/pass/parallel_default_pool_reused.sifr` exercises both `map` and `try_map`
  through the default-pool path in one process. (Reuse itself is enforced structurally by the `OnceLock` + `static`,
  not by the assertions — the fixture is a smoke path, not an instrumented reuse counter. Acceptable since reuse is
  a property of the emitted code, verified by emit-grep.)
- Listed in `verification/validation_lanes/create_pr_e2e_manifest.json:84` and `verification/validation_lanes/merge_e2e_manifest.json:99`.
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md`:
  - Updated default-pool row mentions `parallel_default_pool_reused` and "process-lifetime private default Rayon pool
    reuse via `OnceLock` without configuring Rayon's global pool".
  - Removed the prior "lazy private default pool shutdown design" follow-up — consistent with the chosen design
    (process-lifetime static pool; worker threads are reaped at process exit, no explicit shutdown).
  - Configured `Pool` row unchanged and remains accurate.
  - `Open Follow-up Boundaries` items remaining are independent of this wave.

The `.sifr` fixture's `str(checked).find("odd value") is not None` idiom matches the established Sifr `Optional[int]`
convention used in sibling fixtures (e.g. `parallel_try_map_user_error_typed.sifr:17`,
`spawn_cpu_user_error_typed.sifr:13`), so the assertion meaningfully checks the error message rather than being a
Python-style no-op.

### 5. No global Rayon pool mutation

Implementation uses `rayon::ThreadPoolBuilder::new()...build()` plus `pool.install(...)`. There is no `build_global`
call anywhere in the preamble — Rayon's process-wide global pool is left at its default and never reconfigured. This
matches the phase contract.

## Non-blocking polish (not required)

- The fixture name "default_pool_reused" implies behavioral observation of reuse, but reuse is guaranteed
  structurally rather than asserted in the fixture. A `print(...)` on a thread-id-aware counter could make the
  reuse observable from inside Sifr, but it would also expose nondeterministic thread identities. The current
  approach (structural guarantee + emit-grep) is reasonable.
- A cached `OnceLock` init failure permanently poisons the default pool for the remainder of the process. This is
  reasonable for the runtime (ThreadPoolBuilder failures are not retry-friendly) and consistent with `Pool`'s
  cached `_failure` semantics, but the trade-off is currently undocumented in traceability. A one-line note that
  default-pool construction failure is cached and reused would tighten the docs.
- Removing the "lazy default pool shutdown" follow-up is honest given the new design, but traceability could
  briefly note that default-pool worker threads outlive `main` and are reaped at process exit (matches typical
  Rust static thread-pool leaks). Pure polish; the design choice is unambiguous from reading the implementation.

None of these block the wave.

## Local checks run during this review

- Inspected diff against working tree (`git diff HEAD -- crates/sifr_codegen/src/preamble/parallel_runtime.rs`,
  `verification/stdlib/concurrency_runtime_m3_offload_traceability.md`).
- Read the full `parallel_runtime.rs` preamble after the edit.
- Confirmed manifests contain `parallel_default_pool_reused` in both create-pr and merge lanes.
- Ran `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/parallel_default_pool_reused.sifr` and confirmed:
  - `static __SIFR_DEFAULT_PARALLEL_POOL: std::sync::OnceLock<...>` is emitted.
  - Top-level `__sifr_parallel_map` / `__sifr_parallel_try_map` route through `__sifr_default_parallel_pool()`.
  - No residual `__sifr_build_parallel_pool(__sifr_default_parallel_worker_count())` call at top level.
  - `WorkerRuntimeError` and `WorkerError` are emitted as `#[derive(Debug, Clone)]`, satisfying the cached-clone path.

Pre-existing agent validations (cargo fmt --check, cargo check -p sifr_codegen, e2e pass runs, file-size and HIR
guardrails) are not re-run here; they were reported PASS upstream and are consistent with the static review above.
