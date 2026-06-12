## Review — M3 Concurrency Runtime Closeout Wave (Pass 4, post-PR-2326 final rebase)

### Scope and inputs

- Branch: `codex/concurrency-runtime-m3-closeout` at `0e3734197` ("Close M3 parallel runtime hooks").
- Base: `origin/main` at `fa24b6b1d` (post-PR-#2326 default parallel pool closure merge).
- Range under review: `git log origin/main..HEAD` is a single commit (`0e3734197`), confirming the second rebase landed cleanly with one closeout commit on top of both PR #2323 (scoped owner CPU offload) and PR #2326 (lazy default Rayon pool).
- Files inspected: the eight production paths plus two manifests and the issue/traceability ledger documents named in the request, plus the three historical review files added by this branch.

### Findings (ordered by severity)

**1 — PASS: Second rebase preserved both the default-pool closure and the shared-hook closeout.**

`crates/sifr_codegen/src/preamble/parallel_runtime.rs:99-120` still carries PR #2326's lazy `OnceLock<Result<rayon::ThreadPool, WorkerRuntimeError>>` named `__SIFR_DEFAULT_PARALLEL_POOL` plus `__sifr_default_parallel_pool()`, which clones the cached typed error on every subsequent failure observation. `__sifr_parallel_map` (`parallel_runtime.rs:122-139`), `__sifr_parallel_try_map` (`parallel_runtime.rs:141-164`), `__sifr_pool_map` (`parallel_runtime.rs:166-193`), and `__sifr_pool_try_map` (`parallel_runtime.rs:195-228`) all route through the shared `__sifr_with_silent_worker_panic_hook` helper. The deleted per-surface helper from this file (`__sifr_with_silent_parallel_panic_hook`, formerly at lines ~119) is gone and not re-introduced by the rebase.

**2 — PASS: Shared worker panic-hook helper is the only one emitted; no per-surface helpers remain in production codegen.**

`crates/sifr_codegen/src/preamble/cpu_offload_runtime.rs:3-36` now defines a new `build_worker_panic_hook_items()` that emits `static __SIFR_WORKER_PANIC_HOOK_LOCK: std::sync::Mutex<()>` plus the `__sifr_with_silent_worker_panic_hook` helper that takes a process-wide lock (poison-tolerant via `into_inner`) before swapping `std::panic::set_hook`. The CPU offload bodies (`cpu_offload_runtime.rs:65, 101`), scoped-task CPU offload bodies (`task_scope_offload_runtime.rs:93, 125`), and JoinSet CPU body (`join_set_runtime.rs:380`) all call `__sifr_with_silent_worker_panic_hook` — and the prior per-surface helpers `__sifr_with_silent_cpu_panic_hook`, `__sifr_with_silent_scope_cpu_panic_hook`, and `__sifr_with_silent_join_set_panic_hook` are deleted from `cpu_offload_runtime.rs`, `task_scope_offload_runtime.rs`, and `join_set_runtime.rs` respectively. `grep -rn '__sifr_with_silent_' crates/` returns hits only for `__sifr_with_silent_worker_panic_hook` and the `stdlib_preamble.contains(...)` substring probe at `lib_modules_and_codegen.rs:419`. Confirmed via four emit checks against `parallel_default_pool_reused.sifr`, `task_group_spawn_cpu.sifr`, `spawn_cpu_worker_panic_typed.sifr`, and `join_set_spawn_cpu_join_all_ordered.sifr` — each emits exactly one `__SIFR_WORKER_PANIC_HOOK_LOCK` static, one `__sifr_with_silent_worker_panic_hook` helper, and zero per-surface helpers.

**3 — PASS: Emission predicates correctly cover every surface that needs the helper.**

`crates/sifr_codegen/src/lib_modules_and_codegen.rs:416-419` computes `uses_worker_panic_hook = uses_spawn_cpu || uses_join_set_spawn_cpu || uses_task_scope_spawn_cpu || stdlib_preamble.contains("__sifr_with_silent_worker_panic_hook")`. The substring guard is sound because `parallel_runtime_rust_code()` (`parallel_runtime.rs:21-234`) embeds the literal symbol four times — once for each of `__sifr_parallel_map`, `__sifr_parallel_try_map`, `__sifr_pool_map`, `__sifr_pool_try_map` — so any module that imports `sifr.parallel` (top-level or configured `Pool`) trips the predicate and gets the helper. `lib_modules_and_codegen.rs:585-596` emits the items in order: task-scope items → task-scope-offload → task-scope-cpu-offload → join-set → worker-panic-hook → join-set-cpu → cpu-offload. Rust hoists `static`/`fn` items at module scope, so the fact that `task_scope_cpu_offload_items` precedes `worker_panic_hook_items` does not affect visibility. `crates/sifr_codegen/src/entrypoints.rs:83-91` mirrors the same predicate in the test-codegen path (`generate_rust_test`); sifr.parallel is not exercised through that path (it goes through `generate_rust_with_stdlib`), so the omission of a parallel-only branch in `generate_rust_test` is consistent with how the rest of the parallel runtime is gated.

**4 — PASS: No duplicate/stale default-pool fixture remains.**

`crates/sifr/tests/e2e/pass/` lists exactly one default-pool fixture: `parallel_default_pool_reused.sifr` (the canonical PR #2326 fixture). The pre-rebase closeout fixture `parallel_map_default_pool_reuse.sifr` is removed. Both validation manifests reference the canonical fixture exactly once (`verification/validation_lanes/create_pr_e2e_manifest.json:84` and `verification/validation_lanes/merge_e2e_manifest.json:99`); no closeout-variant name appears in either manifest. `git ls-files | xargs grep -l 'parallel_map_default_pool_reuse'` returns only the three historical review files (`reviews/...-pass-1.md`, `-pass-2.md`, `-pass-3.md`), which is correct historical provenance — they describe past states and were written when the closeout still carried the duplicate. The stale references in `target/sifr_e2e_cache/...` are gitignored generated artifacts, not source.

**5 — PASS: Lowering refactor is purely a clippy clean-up.**

`crates/sifr_lowering/src/lower/task_join_set_calls.rs:67-104` now resolves the JoinSet alias once into `join_set_ty`, borrows `ok_ty: &Type` and `err_ty: &Type`, and threads those borrows into `lower_join_set_add` (`:131-138`), `lower_join_set_spawn_blocking` (`:187-208`), and `lower_join_set_spawn_cpu` (`:219-248`). Internal call sites clone only where a `Box<Type>` constructor demands ownership (`:91-92`). This resolves the `clippy::needless_pass_by_value` lints flagged in pass-1 without changing behavior; `is_assignable_to(ok_ty)` and `is_assignable_to(err_ty)` accept `&Type` so the call sites are equivalent. `crates/sifr_lowering/src/lower/mod_context.rs:41,43` adds backticks around `JoinSet` in doc comments, resolving the `clippy::doc_markdown` lints. Both production lints are now clean.

**6 — PASS: Docs/ledger consistent with current main.**

`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:253-256` records PR #2320, PR #2323, and PR #2326 as merged with their reviewer-confirmed PASS records, then adds a fourth bullet for the closeout review (pass-2 PASS) — this is consistent with both rebases having landed. The pool-architecture decision row at `:673` was updated by PR #2326 to describe the lazy default pool with cached typed first-construction failure; the closeout PR does not alter it. The traceability doc `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:5` correctly reads "M3 closeout wave active after the merged default parallel pool closure wave"; the worker-panic-boundary row (`:18`) lists all five surfaces (`sifr.parallel`, configured `Pool` work, scoped `TaskScope`/`TaskGroup` CPU offload, `task.spawn_cpu`, `JoinSet.spawn_cpu`) sharing one generated mutex; the `Pool` row (`:13`) explicitly notes configured Pool workers use the same serialized hook guard. The Open Follow-up Boundaries section was renamed to "Follow-up Boundaries" with "Intentional post-M3 follow-up boundaries" as the lead, and the prior open per-call hook-suppression item now reads as a satisfied serialization decision plus a documented stable-Rust process-global constraint, which is accurate.

**7 — LOW (non-blocking, advisory): Static-item ordering in `lib_modules_and_codegen.rs` mixes consumers before producer.**

`lib_modules_and_codegen.rs:582-589` emits `task_scope_cpu_offload_items` (which references `__sifr_with_silent_worker_panic_hook`) before `worker_panic_hook_items` (which defines it). This compiles correctly in Rust thanks to item hoisting and was confirmed by all four emit checks producing buildable output, but a future reader skimming the assembly order may find it disorienting. Optional improvement (post-merge or in a separate housekeeping PR): move `build_worker_panic_hook_items()` to run before `task_scope_cpu_offload_items` so the source order in generated files matches the call-graph direction. Not a closure blocker.

**8 — LOW (non-blocking, advisory): "Intentional post-M3 follow-up boundaries" bullets read as state-of-the-world, not work items.**

`verification/stdlib/concurrency_runtime_m3_offload_traceability.md:39-43` keeps the last two bullets describing what M3 *did* (serialized mutex; lazy default `OnceLock` pool) rather than what remains for a future milestone. The section header was renamed from "Open Follow-up Boundaries" to "Follow-up Boundaries" with the lead "Intentional post-M3 follow-up boundaries," which makes the framing defensible — these are declared intentional surface shapes — but the phrasing is slightly mixed (the first three bullets are forward-looking gaps; the last two are statements of present state). Optional polish: either split into a "Decisions" sub-section for the two state bullets or move them out of the follow-up list into the Production Surface Traceability rows. Not a closure blocker.

### Cross-cutting checks

- `cargo fmt --check`: PASS (re-run by reviewer; no output).
- `cargo clippy --workspace -- -D warnings`: PASS (re-run by reviewer; no output).
- `git diff --check origin/main..HEAD`: PASS (no whitespace errors).
- `python3 -m json.tool` on both validation manifests: PASS.
- Emit probes on the four canonical fixtures named in the request: PASS — all show one `__SIFR_WORKER_PANIC_HOOK_LOCK` + one `__sifr_with_silent_worker_panic_hook` + zero per-surface helper symbols. Runtime fixtures (`parallel_default_pool_reused.sifr`, `task_group_spawn_cpu.sifr`, `spawn_cpu_worker_panic_typed.sifr`, `join_set_spawn_cpu_join_all_ordered.sifr`) all report cache-hit PASS via `cargo run -q -p sifr -- run`.
- Stated branch-level validation in the task brief (create-pr 89 passed/0 failed, platform golden pass=5/skip=2, cache_hits=21/23, warm wall-time advisory only) is consistent with prior M3 wave reports and the ledger entry at `issues/...:556`. Re-running the full create-pr profile was not necessary for this review since fmt/clippy/json all pass and the four shared-helper fixtures still build and run.

### Answers to the explicit review questions

1. **Did the second rebase over PR #2326 preserve both the default-pool closure and the shared-hook closeout?** Yes. Finding #1 confirms PR #2326's `OnceLock`-backed lazy default pool and its typed first-failure caching survive verbatim, while the closeout's shared worker-panic-hook helper rides on top with all five surfaces routed through it.

2. **Is there any duplicate/stale default-pool fixture or manifest entry left from the closeout branch?** No. Finding #4 confirms only the canonical `parallel_default_pool_reused.sifr` fixture remains, with one manifest reference per lane and no live source references to the legacy `parallel_map_default_pool_reuse` name (the historical review files retain that name only as provenance, which is correct).

3. **Do any old per-surface hook helpers remain in production generated runtime code?** No. Finding #2 confirms `__sifr_with_silent_parallel_panic_hook`, `__sifr_with_silent_cpu_panic_hook`, `__sifr_with_silent_join_set_panic_hook`, and `__sifr_with_silent_scope_cpu_panic_hook` are all removed from `parallel_runtime.rs`, `cpu_offload_runtime.rs`, `join_set_runtime.rs`, and `task_scope_offload_runtime.rs` respectively; the only emitted helper is `__sifr_with_silent_worker_panic_hook` from the new `build_worker_panic_hook_items()`.

4. **Is PR #2325 ready to force-push and merge?** Yes, from the strict-review perspective. All pass-1 blockers are remediated (clippy, missing baselines, cross-surface hook race, doc precision); pass-2 and pass-3 each confirmed PASS at their respective rebase points; pass-4 (this review) confirms the second rebase over PR #2326 cleanly preserved both waves' invariants and the shared-helper routing. The two LOW items above are non-blocking advisory polish.

5. **Are there any remaining strict blockers to M3 closure after this PR and final ledger update merge?** No strict blockers. The traceability's "Follow-up Boundaries" section now correctly distinguishes intentional post-M3 surface shapes from forward work items, the panic-hook race flagged in pass-1 is fully resolved across all five Rayon/CPU surfaces, the lazy default pool with cached typed first-failure semantics is in place, and the worker-panic boundary row enumerates all five covered surfaces. After PR #2325 merges and the ledger entry transitions the closeout review from pass-2 PASS to "PR merged at <sha>," M3 can be declared closed.

---

RESULT: PASS
