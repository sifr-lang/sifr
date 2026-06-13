## Review — M3 Concurrency Runtime Closeout Wave (Pass 2)

Strict review of the closeout-wave working tree against the pass-1 findings in `reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-1.md`.

### Pass-1 blocker verification

**Pass-1 #1 — Workspace clippy red.** Remediated.
- `crates/sifr_lowering/src/lower/mod_context.rs:41-44` — the in-scope `JoinSet` references now use backticks (` `JoinSet` `), clearing `clippy::doc_markdown`.
- `crates/sifr_lowering/src/lower/task_join_set_calls.rs:131-137,187-193,219-225` — `lower_join_set_add`, `lower_join_set_spawn_blocking`, and `lower_join_set_spawn_cpu` all take `ok_ty: &Type, err_ty: &Type` now; `validate_worker_result_type`/`validate_worker_result_ok_type` propagate the borrows, clearing `clippy::needless_pass_by_value`.
- Confirmed locally: `cargo clippy --workspace -- -D warnings` -> PASS (re-run during this review, `Finished dev profile`, no warnings).

**Pass-1 #2 — Required-baseline validation not run.** Remediated.
- The ledger entry `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:493-512` now records the full required-baseline set, including `cargo clippy --workspace -- -D warnings` (line 506) and `scripts/run_all_tests.sh --profile create-pr` (line 512).
- The latest report `target/validation_lane_reports/create-pr.latest.json` shows all 14 `lane_steps` PASS (`core_guardrails`, `diagnostic_contracts`, `frontend_syntax_guardrails`, `developer_tooling_checks`, `performance_budget_checks`, `verification_hardening_self_tests`, `distribution_validation`, `generated_code_quality_checks`, `crate_tests`, `validation_contract_matrix`, `platform_golden`, `e2e_pass_suite`, `verification_hardening_suites`, `extra_e2e_checks`), `within_warm_budget=True`, `advisories=[]`, `cache_hit_rate=0.957`.

**Pass-1 #3 — Cross-surface panic-hook race.** Remediated.
- One shared serialization primitive: `crates/sifr_codegen/src/preamble/cpu_offload_runtime.rs:3-36` emits `__SIFR_WORKER_PANIC_HOOK_LOCK: std::sync::Mutex<()>` and `__sifr_with_silent_worker_panic_hook<T, F>(body: F) -> T` with poison recovery (`Err(poisoned) => poisoned.into_inner()`).
- All four hook-mutating call sites now route through that single helper:
  - `crates/sifr_codegen/src/preamble/parallel_runtime.rs:128,150,182,214` (top-level `__sifr_parallel_map`, `__sifr_parallel_try_map`, configured `__sifr_pool_map`, configured `__sifr_pool_try_map`).
  - `crates/sifr_codegen/src/preamble/cpu_offload_runtime.rs:65,101` (`__sifr_spawn_cpu_infallible`, `__sifr_spawn_cpu_result`).
  - `crates/sifr_codegen/src/preamble/join_set_runtime.rs:380` (`JoinSet.__sifr_spawn_cpu`).
- Old per-surface helpers (`__sifr_with_silent_parallel_panic_hook`, `__sifr_with_silent_cpu_panic_hook`, `__sifr_with_silent_join_set_panic_hook`) are removed; emit confirmed:
  - `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/parallel_map_basic.sifr | rg -n "__sifr_with_silent_(parallel|cpu|join_set)_panic_hook"` -> no matches.
  - Same for `spawn_cpu_worker_panic_typed.sifr` and `join_set_spawn_cpu_join_all_ordered.sifr`.
- Wiring is correct in both emission paths:
  - Multi-module + sifr.parallel: `crates/sifr_codegen/src/lib_modules_and_codegen.rs:413-415` derives `uses_worker_panic_hook` from `uses_spawn_cpu || uses_join_set_spawn_cpu || stdlib_preamble.contains("__sifr_with_silent_worker_panic_hook")`, then `:578-580` extends `preamble_items` with `build_worker_panic_hook_items()`. Forward reference is sound because both the parallel runtime call sites (inside `stdlib_preamble`) and the helper definition (in `preamble_items`, emitted after `stdlib_preamble` in the rendered file) live at module scope.
  - Test entrypoints: `crates/sifr_codegen/src/entrypoints.rs:75-77` emits the shared helper before `build_join_set_cpu_items()`/`build_cpu_offload_items()` whenever CPU-offload or JoinSet CPU is used.
- Correctness checks on the helper body: previous hook is captured under the mutex, restored before the function returns (and before `resume_unwind`), and the `_hook_guard` drops on both the normal and panic paths.

**Pass-1 #4 — Doc precision for hook scope.** Remediated.
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:17` (Worker panic boundary row) explicitly enumerates the four surfaces sharing the mutex: "`sifr.parallel`, configured `Pool` work, `task.spawn_cpu`, and `JoinSet.spawn_cpu` share one generated mutex around temporary panic-hook suppression while Rust's panic hook remains process-global."
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:42` mirrors the same enumeration in the Open Follow-up Boundaries section.

**Pass-1 #5 — Lazy default pool poisons on first failure (low).** Remediated by documentation.
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:11` calls out: "A default-pool construction failure is cached in that generated `OnceLock` for the process lifetime and returned as a typed runtime error on later top-level calls."
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:43` repeats this guarantee in the Open Follow-up Boundaries.
- Mechanism unchanged and sound: `crates/sifr_codegen/src/preamble/parallel_runtime.rs:109-119` uses `OnceLock::get_or_init` and returns a cloned `WorkerRuntimeError` on cached failure (`WorkerRuntimeError: Clone, PartialEq, Eq, Hash` in the existing preamble).

**Pass-1 #6 — No fixture proves OnceLock reuse (low).** Remediated.
- `crates/sifr/tests/e2e/pass/parallel_map_default_pool_reuse.sifr` calls `sifr.parallel.map` twice in `main` (`doubled = map(first_values, double)`; `incremented = map(second_values, increment)`) and asserts both outputs. Emit shows two distinct calls into the same `__sifr_default_parallel_pool()` (`emit ...parallel_map_default_pool_reuse.sifr` lines 303-304).
- Fixture is registered in both lanes: `verification/validation_lanes/create_pr_e2e_manifest.json:77` and `verification/validation_lanes/merge_e2e_manifest.json:92`.
- The e2e pass suite count moved from 85 (prior wave) to 86 in `target/validation_lane_reports/create-pr.latest.json`, consistent with the added fixture.

**Pass-1 #7 — Configured-Pool serialization side-effect undocumented (low).** Remediated.
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:13` (Pool / PoolConfig row) explicitly adds: "Configured pool workers use the same serialized generated panic-hook guard as top-level `sifr.parallel` and CPU offload work."
- Mechanism verified at `crates/sifr_codegen/src/preamble/parallel_runtime.rs:182,214` — both `__sifr_pool_map` and `__sifr_pool_try_map` route through the shared helper.

**Pass-1 #8 — Empty pass-1 artifact (low).** Remediated.
- `reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-1.md` is now a populated 64-line review with all eight findings and a `RESULT: CHANGES_REQUESTED` footer.

### Soundness checks beyond the pass-1 finding list

- Helper body ordering: `let _hook_guard = ...; let previous_hook = take_hook(); set_hook(silent); let result = catch_unwind(...); set_hook(previous_hook); return match result { ... resume_unwind ... }` — `previous_hook` is restored before any path that propagates panic, and `_hook_guard` releases the mutex during unwind. Poison recovery (`poisoned.into_inner()`) keeps the serialization gate live across earlier panics.
- The mutex is non-reentrant: a worker closure that itself called `sifr.parallel.map` from inside another `sifr.parallel.map` would self-deadlock. This is pre-existing behavior carried forward from the prior parallel-only mutex; not a regression introduced by the closeout, and the offload-only API surface makes it unreachable from typed Sifr code that goes through the validated CPU-heavy / non-Send rejections.
- `__sifr_default_parallel_pool()` returns `Result<&'static rayon::ThreadPool, WorkerRuntimeError>` and is called before `pool.install(...)`; `pool.install` runs the body on the caller thread so the caller acts as a Rayon participant while holding the worker-hook mutex, matching the documented serialization model. `WorkerRuntimeError: Clone` is preserved (existing parallel preamble Clone+Eq+Hash derives unchanged).
- Generated-code emit order is sound: rendered file is `imports → stdlib_preamble → enum_items → preamble_items → body_items` (`crates/sifr_codegen/src/lib_modules_and_codegen.rs:587-717`), so the helper definition in `preamble_items` is in-scope for forward calls from `stdlib_preamble` (Rust allows forward references for module-level fns).
- Feature gating preserved: `crates/sifr_codegen/src/lib_modules_and_codegen.rs:754-759` still includes Rayon as a required feature when `stdlib_preamble.contains("rayon::")` or `uses_spawn_cpu || module_uses_join_set_spawn_cpu(module)` — both paths still cover their original surface.
- Validation manifest JSON parsability sanity: both `create_pr_e2e_manifest.json` and `merge_e2e_manifest.json` parse cleanly (`python3 -m json.tool`).

### Answers

1. **Pass-1 blockers fully remediated?** Yes — all three blocker findings (#1 clippy, #2 missing baseline validation, #3 cross-surface hook race) and all five low-severity findings (#4-#8) are addressed with code and documentation changes verifiable at the file/line references above. Re-run of `cargo clippy --workspace -- -D warnings` during this review returned clean.
2. **Closeout PR ready to open and merge?** Yes — after the user's existing local validation (clippy, fmt, create-pr lane all PASS in the recorded report) the closeout is open-able. Normal post-open gates (CI matching the create-pr/merge lanes, code-owner review) apply as usual; nothing here requires a follow-up commit before the PR opens.
3. **Remaining blockers for `milestone_concurrency_runtime_3` closure?** None strict. The closeout legitimately closes M3's contract surface. The remaining items listed in `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:36-43` are intentional out-of-scope follow-ups (scoped CPU offload owner methods, full `WorkerError[E]` typing, closure capture sendability beyond item/output/error sendability, OS thread creation failure handling, and the residual process-global panic-hook reality that Rust does not expose per-thread). Marking M3 complete after this PR merges and `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:34` flips is consistent with the recorded scope.

RESULT: PASS
