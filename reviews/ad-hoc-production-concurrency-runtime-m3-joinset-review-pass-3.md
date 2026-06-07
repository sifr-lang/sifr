Here is the review of the M3 JoinSet wave.

## Pass-1 finding resolution

| # | Finding | Fix location | Verdict |
| --- | --- | --- | --- |
| 1 | AsyncGenerator/Unknown sort-key collision | `union.rs:208-215` (JoinSet=25, Awaitable=26, AsyncIterator=27, AsyncGenerator=28, Unknown/Any/Never=29) | Resolved |
| 2 | Live JoinSet rebind was silent | `control_flow.rs:327-339` — emits SIFR-OWN-0001 with `name_range` before clearing tracking | Resolved (`join_set_reassign_live_rejected.sifr`) |
| 3 | `cancel_all` on added Task/BlockingTask did not abort | `preamble/join_set_runtime.rs:149-184` (`abort_handle: Option<AbortHandle>`), `:250` (`__sifr_add_task` stores `Some(abort_handle)`), `:268` (`__sifr_add_blocking_task` derives abort from inner `Option<JoinHandle>`), `:319` (cancel path) | Resolved |
| 4 | `Type::JoinSet` missing in type-var collection | `type_var_collection.rs:37` | Resolved |
| 5 | `Type::JoinSet` missing in generic inference | `generic_inference.rs:74` | Resolved |
| 6 | Live-set diagnostic non-deterministic | `annotations_and_function_lowering.rs:696-714` sorts `live_sets` before iteration; per-function save/restore at `:592-603` | Resolved |
| 7 | Bound terminal awaitables rejected | `task_join_set_calls.rs:105-128`, wired from `control_flow.rs:340, 379` and `async_await.rs:69` | Resolved (`join_set_bound_terminal_await.sifr`) |
| 8 | Cancel evidence fixture only counted outcomes | `join_set_cancel_all_task_cancelled.sifr:1-16` sleeps 10s and asserts `"[Cancelled]"` | Resolved (companion fixture) |
| 9 | `Ok(TaskResult::Cancelled(_))` with finished wrapper -> AlreadyStarted | `preamble/join_set_runtime.rs:319` routes `Ok(__SifrTaskResult::Cancelled(_))` to `CancelOutcome::Cancelled`; `was_finished` gate retained only on `Err(is_cancelled)` | Resolved |
| 10 | Non-CPU JoinSet pulled Rayon | `lib_join_set_needs.rs:4-41`, `lib_modules_and_codegen.rs:566-575, 750`, `preamble/join_set_runtime.rs:341-411` (Rayon items live in `build_join_set_cpu_items`) | Resolved (`emit … rg rayon` empty for spawn_blocking-only) |

## Non-blocking observations

1. **Use-after-bind gap.** `record_join_set_terminal_awaitable` (`task_join_set_calls.rs:117-128`) maps `pending -> joins` but does not mark `joins` moved. Because `__sifr_join_all`/`__sifr_cancel_all` take `self` by value (`preamble/join_set_runtime.rs:304, 316`), a `joins.add(...)` or `joins.spawn_*` between `pending = joins.join_all()` and `await pending` produces a `rustc` use-of-moved-value error rather than a Sifr SIFR-OWN diagnostic. Diagnostic-quality follow-up; "if it compiles, it works" still holds.
2. **Sort-key tiebreaks.** `union.rs:212-217` keeps the preexisting 3-way `Unknown/Any/Never` collision at 29 and now groups `Intersection/Alias/Class` at 31 (Class was 31 alone, Alias was 30 pre-diff). Class vs Alias now interleave by name instead of Class always sorting last. Not a new correctness regression; worth tightening to unique keys later.
3. **`join_set_cancel_all_evidence.sifr` is weak relative to the new fixture.** It only asserts `len(outcomes) == 1`, which the new sleep-based fixture supersedes. Retire or strengthen it next pass.

## Validation re-check on this tree

- `cargo test -p sifr -- --skip test_e2e_pass` -> 34 passed, 0 failed.
- Re-ran all six pass fixtures via `cargo run -q -p sifr -- run …` -> all PASS.
- `cargo run -q -p sifr -- emit … join_set_spawn_blocking.sifr | rg "rayon|__sifr_spawn_cpu|ThreadPoolBuilder|__sifr_with_silent_join_set_panic_hook"` -> no matches.
- Phase docs, execution ledger, both validation-lane manifests, and the traceability/workload-DB artifacts all reference the new fixtures and the `list[TaskResult[T, E]]` surface.

No blocking findings remain; the three observations above are non-blocking and scoped as follow-ups. Pass-2 review artifact written to `reviews/ad-hoc-production-concurrency-runtime-m3-joinset-review-pass-2.md`.

RESULT: PASS
