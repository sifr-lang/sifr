I inspected the working tree directly against the pass-1 findings, the M3 JoinSet phase source, and the focused-fixture validation already recorded. All ten pass-1 blockers are remediated in code and covered by fixtures. The remaining observations below are non-blocking.

## Pass-1 finding resolution

| # | Pass-1 finding | Fix location | Verdict |
| --- | --- | --- | --- |
| 1 | AsyncGenerator/Unknown sort-key collision | `crates/sifr_type_system/src/union.rs:208-215` (JoinSet=25, Awaitable=26, AsyncIterator=27, AsyncGenerator=28, Unknown/Any/Never=29) | Resolved — AsyncGenerator no longer collides with Unknown/Any/Never. |
| 2 | Reassigning a live JoinSet was silent | `crates/sifr_lowering/src/lower/statements/control_flow.rs:327-339` | Resolved — emits SIFR-OWN-0001 with `name_range` before clearing live tracking; `join_set_reassign_live_rejected.sifr` proves it. |
| 3 | `cancel_all` on added Task/BlockingTask did not abort | `crates/sifr_codegen/src/preamble/join_set_runtime.rs:149-184` (`__SifrJoinEntry.abort_handle: Option<AbortHandle>`), `:250` (`__sifr_add_task` stores `Some(abort_handle)`), `:268` (`__sifr_add_blocking_task` derives abort from inner `Option<JoinHandle>`), `:319` (cancel path aborts inner) | Resolved — inner abort handle is preserved end-to-end; outer wrapper observes `__SifrTaskResult::cancelled()` via dropped sender. |
| 4 | `Type::JoinSet` missing from type-var collection | `crates/sifr_lowering/src/lower/type_var_collection.rs:37` | Resolved — added to the `(ok, err)` arm. |
| 5 | `Type::JoinSet` missing from generic inference | `crates/sifr_lowering/src/lower/generic_inference.rs:74` | Resolved — added to the `(ok_a, err_a) / (ok_b, err_b)` arm. |
| 6 | Live-set diagnostic was non-deterministic | `crates/sifr_lowering/src/lower/typing_and_functions/annotations_and_function_lowering.rs:696-714` (`reject_live_join_sets_at_function_exit` sorts `live_sets` before iteration; per-function save/restore at :592-603) | Resolved — sorted iteration; per-function state is captured and restored. |
| 7 | Bound terminal awaitables (`pending = joins.join_all(); await pending`) were rejected | `crates/sifr_lowering/src/lower/task_join_set_calls.rs:105-128` (lookup-then-consume in `consume_awaited_join_set_terminal`; record in `record_join_set_terminal_awaitable`), wired from `lower_assign` (`control_flow.rs:340, 379`) and `lower_await` (`async_await.rs:69`) | Resolved — `join_set_bound_terminal_await.sifr` passes; rebind to a non-terminal value clears the entry, and `join_set_terminal_must_be_awaited_rejected.sifr` still rejects unbound terminal awaitables. |
| 8 | `cancel_all` evidence fixture only counted outcomes | `crates/sifr/tests/e2e/pass/join_set_cancel_all_task_cancelled.sifr:1-16` (sleeps 10s in an added async task and asserts `str(outcomes) == "[Cancelled]"`) | Resolved — a real cancellation path is now exercised; the original `join_set_cancel_all_evidence.sifr` is kept but is supplementary. |
| 9 | `Ok(__SifrTaskResult::Cancelled(_))` with `was_finished` mapped to `AlreadyStarted` | `crates/sifr_codegen/src/preamble/join_set_runtime.rs:319` (`Ok(__SifrTaskResult::Cancelled(_)) => CancelOutcome::Cancelled`) | Resolved — finished-with-cancelled-wrapper now reports `Cancelled`; the `was_finished` gate remains only on `Err(join_error) if is_cancelled()` to disambiguate aborts of already-completed handles. |
| 10 | Non-CPU JoinSet programs pulled Rayon | `crates/sifr_codegen/src/lib_join_set_needs.rs:4-41` (`module_uses_join_set_spawn_cpu` matches only `__sifr_spawn_cpu`), `lib_modules_and_codegen.rs:566-575, 750` (CPU items + Rayon feature flag gated on `uses_join_set_spawn_cpu`), `preamble/join_set_runtime.rs:341-411` (Rayon-touching items live in `build_join_set_cpu_items()`) | Resolved — `emit join_set_spawn_blocking.sifr | rg rayon` produces no matches; `emit join_set_spawn_cpu_join_all_ordered.sifr` does. |

## Non-blocking observations

1. **Use-after-bind of a JoinSet between `pending = joins.join_all()` and `await pending`.** `record_join_set_terminal_awaitable` (`task_join_set_calls.rs:117-128`) only registers the pending->owner mapping; it does not mark `joins` moved. At the Sifr level, a subsequent `joins.add(...)` or `joins.spawn_*` between binding and awaiting compiles, but at the Rust level `__sifr_join_all`/`__sifr_cancel_all` take `self` by value (see `params: vec![RustParam::SelfValue]` at `preamble/join_set_runtime.rs:304, 316`), so `joins` is moved and the second use becomes a `rustc` error instead of a SIFR-OWN diagnostic. "If it compiles, it works" still holds, but the diagnostic experience is degraded. Worth a focused follow-up that marks the owner moved at bind time and rejects further method calls with SIFR-OWN-0001.

2. **Sort-key tiebreaks at keys 29 and 31.** `union.rs:212-217` keeps the preexisting 3-way `Unknown/Any/Never` collision at 29 and now puts `Intersection`/`Alias`/`Class` together at 31 (Class was 31 alone pre-diff, Alias was 30). For Class vs. Alias the secondary key is the type name, so they now interleave alphabetically rather than always sorting Alias before Class. These are not new failure modes (Unknown/Any/Never has always collided, and Class/Alias rarely co-occur in normalized unions), but the table would benefit from a follow-up that gives each leaf a unique key.

3. **`join_set_cancel_all_evidence.sifr` is now supplementary.** With `join_set_cancel_all_task_cancelled.sifr` providing real cancel evidence (`task.sleep(10.0)` in an added async task, asserted `[Cancelled]`), the original fixture's `len(outcomes) == 1` assertion is the weakest one in the suite — a CPU worker that returns immediately can satisfy it as `AlreadyCompleted`. Either tighten its assertion to `str(outcomes) in ("[Cancelled]", "[AlreadyCompleted]")` or retire it; not blocking because the stronger fixture already gates the contract.

## Validation re-check on this tree

- `cargo test -p sifr -- --skip test_e2e_pass` -> 34 passed; 0 failed (locally re-run during this review).
- Re-ran all six pass fixtures (`join_set_spawn_cpu_join_all_ordered`, `join_set_add_task_join_all`, `join_set_cancel_all_evidence`, `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`, `join_set_bound_terminal_await`) via `cargo run -q -p sifr -- run …` -> all PASS.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/join_set_spawn_blocking.sifr | rg "rayon|__sifr_spawn_cpu|ThreadPoolBuilder|__sifr_with_silent_join_set_panic_hook"` -> no matches.
- Phase source (`issues/ad-hoc-production-concurrency-runtime-platform-substrate.md`) and execution ledger updated to reflect the `list[TaskResult[T, E]]` join_all surface, the `JoinSet[T, WorkerError]` spawn_cpu boundary, and the bound-terminal consumption rule. Verification artifacts (`concurrency_runtime_m3_offload_traceability.md`, `concurrency_runtime_workload_database.md`, both validation-lane manifests) list every new pass and fail fixture.

## Verdict

All ten pass-1 blockers are remediated with code changes and fixtures; the three observations above are non-blocking nits or scope-out follow-ups (use-after-bind hardening, sort-key uniqueness, evidence-fixture cleanup). Local validation at create-pr profile remains green.

RESULT: PASS
