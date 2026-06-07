I have enough evidence. Writing up the strict review.

## Review — M3 Concurrency Runtime Closeout Wave (Post-Rebase, Pass 3)

Strict verification of the post-rebase working tree against the rebase-resolution intent: extend the shared worker panic-hook guard to scoped owner CPU offload introduced by PR #2323 while preserving the rest of the pass-2 model.

### Rebase-resolution verification

**1 — Stale per-surface scoped CPU hook removed.** Verified.
- `git diff crates/sifr_codegen/src/preamble/task_scope_offload_runtime.rs` shows `build_task_scope_cpu_offload_items()` no longer emits `__sifr_with_silent_scope_cpu_panic_hook`; both scoped bodies (infallible at `task_scope_offload_runtime.rs:93` and result-shaped at `:125`) now call `__sifr_with_silent_worker_panic_hook(...)`.
- Grep across the working tree confirms no production-code call site or definition of `__sifr_with_silent_(parallel|cpu|join_set|scope_cpu)_panic_hook` remains — only review/ledger docs mention them as historical context.

**2 — Shared helper definition unchanged and sound.** Verified.
- `crates/sifr_codegen/src/preamble/cpu_offload_runtime.rs:3-36` defines exactly one `__SIFR_WORKER_PANIC_HOOK_LOCK: std::sync::Mutex<()>` and one generic `__sifr_with_silent_worker_panic_hook<T, F: FnOnce() -> T>(body) -> T`.
- Body order at `:30` is sound: `lock()` (with `Err(poisoned) => poisoned.into_inner()` recovery) → capture `previous_hook` → install silent hook → `catch_unwind(AssertUnwindSafe(body))` → restore `previous_hook` → `resume_unwind(payload)` on Err. `_hook_guard` drops on both normal and unwind paths, so a panic inside `body` cannot leak the silent hook to the next caller.

**3 — All five CPU-offload surfaces route through the single helper.** Verified at file/line:
- `crates/sifr_codegen/src/preamble/parallel_runtime.rs:128,150,182,214` — `__sifr_parallel_map`, `__sifr_parallel_try_map`, `__sifr_pool_map`, `__sifr_pool_try_map`.
- `crates/sifr_codegen/src/preamble/cpu_offload_runtime.rs:65,101` — `__sifr_spawn_cpu_infallible`, `__sifr_spawn_cpu_result`.
- `crates/sifr_codegen/src/preamble/join_set_runtime.rs:380` — `__SifrJoinSet<T, WorkerError>::__sifr_spawn_cpu`.
- `crates/sifr_codegen/src/preamble/task_scope_offload_runtime.rs:93,125` — scoped `__sifr_scope_spawn_cpu_infallible`, `__sifr_scope_spawn_cpu_result`.

**4 — Emission predicates include `uses_task_scope_spawn_cpu` (the critical post-rebase fix).** Verified in both emit paths:
- Multi-module / stdlib path at `crates/sifr_codegen/src/lib_modules_and_codegen.rs:414` reads the flag, `:416-419` builds `uses_worker_panic_hook = uses_spawn_cpu || uses_join_set_spawn_cpu || uses_task_scope_spawn_cpu || stdlib_preamble.contains("__sifr_with_silent_worker_panic_hook")`, and `:588-590` emits `build_worker_panic_hook_items()` when that flag is true. `:582-584` still emits `build_task_scope_cpu_offload_items()` whenever scoped CPU offload is used.
- Test entrypoint path at `crates/sifr_codegen/src/entrypoints.rs:60` reads the flag, `:77-79` emits `build_task_scope_cpu_offload_items()`, and `:83-85` emits `build_worker_panic_hook_items()` whenever `uses_join_set_spawn_cpu || uses_spawn_cpu || uses_task_scope_spawn_cpu`. Scoped CPU offload is in the predicate — if it had been missed, scoped-CPU-only modules generated through the test entrypoint would fail to compile against an undefined helper.
- Rayon feature gating at `lib_modules_and_codegen.rs:766-771` also includes `module_uses_task_scope_spawn_cpu(module)`, so scoped-CPU-only modules still get Rayon as a generated cargo feature.

**5 — Emission ordering is sound for forward references.** Verified.
- In `lib_modules_and_codegen.rs`, `preamble_items` is assembled with `build_task_scope_cpu_offload_items()` (`:582-584`) before `build_worker_panic_hook_items()` (`:588-590`), then rendered after `stdlib_preamble`. Since both helpers are module-scope `fn`s, Rust allows forward references — no ordering bug.
- The rendered file order is `imports → stdlib_preamble → enum_items → preamble_items → body_items` (`:687-722`), so scoped-CPU spawn code (in `preamble_items`) and the helper (also in `preamble_items`) are in the same module scope; both are visible to each other and to anything in `stdlib_preamble` that references `__sifr_with_silent_worker_panic_hook` (e.g., generated `sifr.parallel` runtime code).

**6 — Emit-level traceability for scoped CPU offload.** Re-verified live during this review:
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/task_group_spawn_cpu.sifr | grep -E "__SIFR_WORKER_PANIC_HOOK_LOCK|__sifr_with_silent_worker_panic_hook|__sifr_with_silent_scope_cpu_panic_hook"` returns exactly two scoped call sites (infallible + result) both routed through `__sifr_with_silent_worker_panic_hook`, plus one shared `__SIFR_WORKER_PANIC_HOOK_LOCK` and one shared helper definition. Zero occurrences of `__sifr_with_silent_scope_cpu_panic_hook`.
- Same probe on `parallel_map_default_pool_reuse.sifr` and `join_set_spawn_cpu_join_all_ordered.sifr` shows only the shared helper — no per-surface variants leaked back during the rebase.

**7 — Lowering ledger correctly carries forward `uses_task_scope_spawn_cpu`.** Verified.
- `mod_context.rs:41-50` keeps `JoinSet` doc-comment backticks (pass-1 clippy fix) and `Vec<(String, Type)>` ownership for `active_task_owner_bindings`.
- `task_join_set_calls.rs:131-225` keeps borrowed `ok_ty: &Type, err_ty: &Type` parameters for `lower_join_set_add` / `_spawn_blocking` / `_spawn_cpu` (pass-1 clippy fix). Both pass-1 remediations survive the rebase.

### Documentation / ledger consistency

**8 — Traceability doc enumerates all five surfaces.** Verified.
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:13` Pool / PoolConfig row: "Configured pool workers use the same serialized generated panic-hook guard as top-level `sifr.parallel`, scoped owner CPU offload, and CPU offload work."
- `:15` TaskScope/TaskGroup row documents scoped CPU offload error mapping into `WorkerRuntimeError`/`WorkerError` plus Rayon gating.
- `:18` Worker panic boundary row enumerates: "`sifr.parallel`, configured `Pool` work, `TaskScope`/`TaskGroup` scoped CPU offload, `task.spawn_cpu`, and `JoinSet.spawn_cpu` share one generated mutex around temporary panic-hook suppression while Rust's panic hook remains process-global."
- `:42` follow-up section repeats the same five-surface enumeration. No drift between rows.

**9 — Execution ledger records post-rebase validation.** Verified.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:516-537` records the closeout-wave commands, including the post-rebase probes against `task_group_spawn_cpu.sifr` (line 532: scoped-CPU emit uses the shared hook and no `__sifr_with_silent_scope_cpu_panic_hook`) and against `spawn_cpu_worker_panic_typed.sifr` / `join_set_spawn_cpu_join_all_ordered.sifr` (lines 531, 533).
- Line 537 records `scripts/run_all_tests.sh --profile create-pr` PASS with `89 passed`, `0 failed`, `cache_hits=21/23`, platform golden `pass=5, skip=2`, advisory only on warm wall-time. Counts are consistent (88 → 89 after the rebase carried PR #2323's fixtures into the closeout PR's create-pr lane).
- Line 254 ledger row already records PR #2323 merged at `2768218fa27118d0c6b7f6d019002a7309eeb0d7`; line 411 "Implementation PRs" lists `M3 scoped owner offload wave: https://github.com/sifr-lang/sifr/pull/2323`. Pass-1 review file (line 539-541) and pass-2 review file (line 542) are referenced; pass-3 is not yet listed in the ledger, but that is a follow-up update on this PR, not a blocker.

**10 — Manifest entries cover the rebased coverage.** Verified.
- `verification/validation_lanes/create_pr_e2e_manifest.json:40-42` lists `task_scope_spawn_blocking`, `task_group_spawn_cpu`, `task_group_spawn_cpu_user_error`; `:80` adds `parallel_map_default_pool_reuse`.
- `verification/validation_lanes/merge_e2e_manifest.json:77-79, 95` mirror the same entries. JSON parsability sanity is implied by the create-pr lane PASS reported in the ledger.

### Soundness checks beyond rebase scope

**11 — Helper is non-reentrant by design.** A worker closure that re-enters `sifr.parallel.map` / `task.spawn_cpu` / scoped CPU offload from inside another such call will self-deadlock. This is unchanged from pass-2; the typed CPU-heavy / non-Send rejections in lowering keep typical Sifr code from constructing reentrant chains. Not a regression introduced by the rebase. Already documented in pass-2 §Soundness.

**12 — Scoped CPU helper still cancels via abort_handle.** `scoped_task_body` (lines 143-147) preserves the abort_handle / observed wiring used by scoped owner cancellation semantics. The rebase only changed which helper the scoped body calls; the scope-child machinery is untouched.

**13 — Lazy default pool still cached.** `parallel_runtime.rs:109-119` `__sifr_default_parallel_pool()` is unchanged; first-failure caching of `WorkerRuntimeError` in the generated `OnceLock` is intact, and the new fixture `parallel_map_default_pool_reuse.sifr` exercises a second call.

**14 — Rebase is mid-application but resolution is consistent.** `git status` shows `UU` for `entrypoints.rs`, `lib_modules_and_codegen.rs`, `issues/...execution.md`, and `verification/stdlib/...traceability.md`, plus ` M` for `task_scope_offload_runtime.rs`. The working-tree contents of all five are the correct post-rebase merge result (verified above); they simply have not been `git add`-ed yet, and the rebase has not been continued. `git diff --check` is clean; no conflict markers remain in code/docs (Grep matches in `third_party/ruff/` are vendored and unrelated). The pending `git add` + `git rebase --continue` is mechanical, not substantive.

### Minor non-blockers

- `reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-3.md` is a 0-byte untracked placeholder. If the intent is to commit this pass-3 review with the rebased branch, the file needs content (or removal). Not a code blocker.
- After force-push, the ledger should be updated to add a one-line entry referencing this pass-3 review under "M3 closeout wave review loop" (currently `issues/...execution.md:539-542` only references pass-1 and pass-2). Trivial doc tail-end update; not a code blocker.

### Answers

1. **Are the rebase-conflict resolutions correct?** Yes. The four `UU` files (`entrypoints.rs`, `lib_modules_and_codegen.rs`, `issues/...execution.md`, `verification/stdlib/...traceability.md`) plus the ` M` `task_scope_offload_runtime.rs` are resolved consistent with the pass-2 model extended to PR #2323's scoped CPU surface: the per-surface `__sifr_with_silent_scope_cpu_panic_hook` is deleted from `build_task_scope_cpu_offload_items()`; both scoped CPU bodies call `__sifr_with_silent_worker_panic_hook`; `uses_task_scope_spawn_cpu` is included in the worker-panic-hook emission predicate in both `lib_modules_and_codegen.rs:416-419` and `entrypoints.rs:83-85`; the traceability doc Worker-panic-boundary row enumerates all five surfaces; and the execution ledger records the post-rebase emit/run/all-tests evidence. No conflict markers remain in code/docs (only in unrelated vendored `third_party/ruff/`). The only remaining mechanical step is `git add` + `git rebase --continue`.

2. **Is PR #2325 still ready to merge after force-pushing this rebased branch?** Yes. Pass-2's PASS verdict carries forward because (a) every pass-1 remediation survives the rebase, (b) the new scoped CPU surface is correctly folded into the shared-hook model without regressing the four prior surfaces, (c) emit/run/create-pr-lane validation has been re-run and remains PASS (`89 passed`, `0 failed`, `cache_hits=21/23`), and (d) the doc/ledger updates accurately describe the post-rebase shape. Recommend: (i) finish the `git add` + `git rebase --continue`, (ii) before force-push, populate or delete `reviews/...pass-3.md` and add a one-line pass-3 reference to `issues/...execution.md:539-542` so the merged ledger reflects this review pass.

3. **Are there any remaining blockers to M3 closure after this PR and the final ledger update merge?** None strict. The remaining items in `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:36-43` are intentional post-M3 follow-ups (full `WorkerError[E]` typing, closure-capture sendability beyond item/output/error, OS-thread-creation-failure handling, the residual process-global Rust panic-hook reality, and the absence of a mutable public default-pool shutdown/reconfiguration API). Flipping `issues/...execution.md:34` from `[ ]` to `[x]` for `milestone_concurrency_runtime_3` once this PR and the ledger update merge is consistent with the recorded scope.

RESULT: PASS
