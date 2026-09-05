I've gathered enough evidence. Writing up the strict review.

## Review — M3 Concurrency Runtime Closeout Wave

### Findings (ordered by severity)

**1 — BLOCKER: Branch fails `cargo clippy --workspace -- -D warnings`.**
A required-baseline command per `AGENTS.md` and `issues/...-execution.md:547-549` errors out with 9 pedantic lints on this branch. The closeout's own diff (`parallel_runtime.rs`) is clippy-clean, but it inherits failures from the merged JoinSet wave:
- `crates/sifr_lowering/src/lower/mod_context.rs:41,43,43` — `clippy::doc_markdown` (missing backticks around `JoinSet`).
- `crates/sifr_lowering/src/lower/task_join_set_calls.rs:134-135, 190-191, 222-223` — `clippy::needless_pass_by_value` on `ok_ty: Type`/`err_ty: Type`.

Failure scenario: any CI gate honoring the project's pedantic lint policy rejects the closeout PR and any subsequent M3-related PR. The closeout cannot merge cleanly, and M3 cannot be declared closed, until these are resolved. Toolchain is `rustc 1.94.0`.

**2 — BLOCKER (evidence): Required-baseline validation was not run.**
The closeout's listed validation (`issues/...-execution.md:493-503`) omits two of the four required baselines:
- `cargo clippy --workspace -- -D warnings` — not run; would have surfaced finding #1.
- `scripts/run_all_tests.sh --profile create-pr` — not run.

Prior M3 waves (spawn_cpu, JoinSet) did run create-pr; the closeout regresses on local-validation rigor at exactly the wave where M3 is supposed to close. This violates the "authoritative gate" promise in `AGENTS.md` and leaves the closeout PR's evidence narrower than the gates require.

**3 — Substantive design gap: panic-hook serialization is parallel-only and leaks across cross-surface races.**
The closeout adds `__SIFR_PARALLEL_PANIC_HOOK_LOCK` only around `__sifr_with_silent_parallel_panic_hook` (`parallel_runtime.rs:121-136`). Two sibling helpers still exist in M3 and use the same process-global `std::panic::set_hook`/`take_hook` with no synchronization:
- `crates/sifr_codegen/src/preamble/cpu_offload_runtime.rs:6-28` — `__sifr_with_silent_cpu_panic_hook` (used by `task.spawn_cpu`).
- `crates/sifr_codegen/src/preamble/join_set_runtime.rs:343-366` — `__sifr_with_silent_join_set_panic_hook` (used by `JoinSet.spawn_cpu`).

Concrete failure scenario (hook leak, not just observability noise):
1. Thread A enters `spawn_cpu`: `take_hook → H0`; `set_hook(silent_cpu)`.
2. Thread B enters `sifr.parallel.map`: acquires the parallel mutex, `take_hook → silent_cpu`; `set_hook(silent_parallel)`.
3. Thread A finishes first: `set_hook(H0)`.
4. Thread B finishes: `set_hook(previous_hook)` — but B's `previous_hook` is `silent_cpu`, not `H0`.

Final state: `silent_cpu` is the process's permanent panic hook until something else swaps it. Subsequent application-level panics (outside any M3 helper) become silent — a real debugging hazard that survives all three helpers returning. Per-item `catch_unwind` still upholds the typed `WorkerRuntimeError`/`WorkerError` boundary, so this is not a correctness regression for documented M3 surfaces, but it is a global-state corruption that the closeout's stated goal ("serialize hook suppression for parallel calls") does not actually achieve in a process that also uses `spawn_cpu` or `JoinSet`. A small follow-up — promote the lock to a single shared static used by all three helpers, or fold them into one helper — closes the cross-surface race cleanly within current scope.

**4 — Doc accuracy: traceability and ledger imply a broader fix than was made.**
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md:42` claims the unsynchronized-hook follow-up is closed "for generated `sifr.parallel` calls" — the qualifier is technically correct but easy to read as a milestone-wide closure given it is filed under "Remaining M3 work" being struck through.
- The same file's `Worker panic boundary` row (line 17) says "Top-level `sifr.parallel` hook suppression is serialized…" without noting that the sibling spawn_cpu and JoinSet helpers retain the unsynchronized swap pattern. Cross-surface readers will miss the gap described in finding #3.
- `issues/...-execution.md:618` (decision-index row "Rayon pool architecture") rewrites the gap to "There is no mutable public shutdown or reconfiguration API; process teardown releases the private default pool." This is accurate for the default pool, but the doc no longer mentions that statics are not Drop-run at exit — workers exit because the OS kills them, not because Rayon joins them. Not a defect, but the wording invites misreading.

**5 — Low: lazy default pool poisons on first failure.**
`__sifr_default_parallel_pool()` (`parallel_runtime.rs:112-119`) uses `OnceLock::get_or_init`, so a one-time `ThreadPoolBuilder::build()` failure is cached forever for the process. `available_parallelism()`-sized builds rarely fail transiently, so this is tolerable, but the contract should be either (a) documented explicitly in the traceability, or (b) replaced with a retry-on-error path. Currently neither.

**6 — Low: no fixture proves OnceLock reuse.**
The traceability now asserts "lazy private default pool reuse through generated `OnceLock` state" (lines 11-12), but no e2e fixture calls `parallel.map`/`try_map` more than once in a single process to exercise that reuse. `OnceLock` semantics make reuse structurally guaranteed, but the asserted evidence has no fixture behind it. A two-call pass fixture (e.g., `parallel_map_default_pool_reuse.sifr`) would harden the claim and add a regression seam if anyone refactors away from `OnceLock`.

**7 — Low: configured-Pool serialization side-effect undocumented.**
`__sifr_pool_map`/`__sifr_pool_try_map` (`parallel_runtime.rs:199, 231`) also call `__sifr_with_silent_parallel_panic_hook`, so configured-Pool work serializes against top-level parallel work via the new mutex. This is consistent and correct, but the traceability row for `Pool`/`PoolConfig` doesn't mention it — readers may be surprised that two independent configured Pools cannot run their bodies concurrently from different threads.

**8 — Low: review artifact is a 0-byte placeholder.**
`reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-1.md` is empty (0 bytes). The ledger references it implicitly via "Pending agent implementation review" but commits an empty file. Either populate it (this review fills the role) or remove the empty file before opening the PR.

### Soundness of the closeout's actual source change

The lazy-default `OnceLock` path is sound: types align (`WorkerRuntimeError: Clone`, `ThreadPool: Sync`), error cloning on cached-failure is fine, and `pool.install` correctly precedes hook installation so caller threads still act as rayon hosts while blocked on the mutex (no deadlock). The mutex correctly survives `body` panics because `_hook_guard` drops on unwind after `set_hook(previous_hook)` runs inside `catch_unwind`. The change is small, contained, and structurally correct for its stated parallel/parallel scope.

### Answers

1. Findings above.
2. **PR-ready?** No. Finding #1 (clippy red) and finding #2 (required-baseline validation not run) must be resolved before opening the closeout PR. The actual source change is sound, but the surrounding gates are not met.
3. **M3 blockers remaining?**
   - Hard blockers: finding #1 (clippy red) and finding #2 (missing baseline validation evidence) — must clear before M3 can close.
   - Soft / recommended-but-not-strict: finding #3 (cross-surface hook leak). Per-item `catch_unwind` keeps M3's typed boundary intact, but the hook-state leak is a real global-state hazard that the closeout's mutex was meant to neutralize and only partially does. If the milestone owner accepts the qualified scope explicitly, this can move to a documented follow-up; otherwise it remains a closure blocker.
   - Non-blocking follow-ups: findings #4–#8 (doc precision, lazy-pool poisoning policy, reuse fixture, configured-pool serialization note, empty review artifact).

RESULT: CHANGES_REQUESTED
