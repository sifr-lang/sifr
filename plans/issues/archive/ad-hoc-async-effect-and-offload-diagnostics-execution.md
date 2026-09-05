# Ad Hoc Async Effect And Offload Diagnostics Execution

## Status

completed

## Source

- [ad-hoc-async-effect-and-offload-diagnostics.md](./ad-hoc-async-effect-and-offload-diagnostics.md)
- [async_concurrency_model.md](../internal_docs/async_concurrency_model.md)

## Checklist

### adhoc_async_effect_0: Effect Summary Infrastructure

- [x] Add internal async suspension summaries.
- [x] Mark known async primitives with direct suspension effects.
- [x] Compute transitive summaries through same-task coroutine awaits.
- [x] Add positive validation fixtures:
  - [x] `async_effect_summary_sleep.sifr`
  - [x] `async_effect_summary_channel_receive.sifr`
  - [x] `async_effect_summary_transitive_await.sifr`

### adhoc_async_effect_1: Reject Fake Async And Fake Await

- [x] Reject `async def` bodies with `NoSuspend`.
- [x] Reject awaiting same-task coroutines with transitive `NoSuspend`.
- [x] Preserve existing non-awaitable hard errors.
- [x] Add explicit protocol-conformance escape hatch only if implementation requires it.
- [x] Add validation fixtures:
  - [x] `async_no_suspend_rejected.sifr`
  - [x] `async_transitive_no_suspend_await_rejected.sifr`
  - [x] `await_sync_function_rejected.sifr`
  - [x] `async_protocol_no_suspend_requires_escape_hatch.sifr`

### adhoc_async_effect_2: Enforce Workload Annotations

- [x] Reject `@blocking_io` on `async def`.
- [x] Reject `@cpu_heavy` on `async def`.
- [x] Reject direct `@blocking_io` calls in async code.
- [x] Reject direct `@cpu_heavy` calls in async code.
- [x] Keep direct cheap sync helper calls in async code allowed.
- [x] Update diagnostic docs and registry metadata.
- [x] Add validation fixtures:
  - [x] `blocking_io_on_async_def_rejected.sifr`
  - [x] `cpu_heavy_on_async_def_rejected.sifr`
  - [x] `blocking_io_direct_call_in_async_rejected.sifr`
  - [x] `cpu_heavy_direct_call_in_async_rejected.sifr`
  - [x] `cheap_sync_helper_in_async_allowed.sifr`

### adhoc_async_effect_3: Restrict Blocking Offload Targets

- [x] Reject `task.spawn_blocking` on unannotated local sync functions.
- [x] Reject `ThreadPoolExecutor.submit` on unannotated local sync functions.
- [x] Allow annotated `@blocking_io` targets.
- [x] Allow annotated `@cpu_heavy` targets.
- [x] Preserve existing sendability, arity, and result/error constraints.
- [x] Add validation fixtures:
  - [x] `spawn_blocking_blocking_io_allowed.sifr`
  - [x] `spawn_blocking_cpu_heavy_allowed.sifr`
  - [x] `spawn_blocking_unannotated_rejected.sifr`
  - [x] `thread_pool_submit_unannotated_rejected.sifr`
  - [x] `spawn_blocking_known_stdlib_blocking_allowed.sifr`

## Local Validation

- [x] `scripts/run_all_tests.sh --profile quick`
- [x] `scripts/run_all_tests.sh`

## Review Notes

- 2026-05-12 annotation vocabulary slice ([#2096](https://github.com/sifr-lang/sifr/pull/2096)): compiler workload annotations, quick-lane fixtures, and the blocking-offload demo now use `@blocking_io` / `@cpu_heavy`. Focused fixture checks and `scripts/run_all_tests.sh --profile quick` passed; the quick run reported a warm wall-time advisory but exited successfully.
- 2026-05-12 effect summary infrastructure ([#2097](https://github.com/sifr-lang/sifr/pull/2097)): internal two-state async suspension summaries are computed to a fixpoint and covered by quick-lane positive fixtures. `scripts/run_all_tests.sh --profile quick` passed; the quick run reported a warm wall-time advisory but exited successfully.
- 2026-05-12 fake async rejection ([#2098](https://github.com/sifr-lang/sifr/pull/2098)): `SIFR-ASYNC-0001` rejects async functions without a suspension source and `SIFR-ASYNC-0002` rejects awaiting same-task coroutines with transitive no-suspend summaries. agent reviewer approval is recorded in `reviews/adhoc_reject_fake_async_review.md`.
- 2026-05-12 workload annotation enforcement ([#2099](https://github.com/sifr-lang/sifr/pull/2099)): `@blocking_io` and `@cpu_heavy` are sync-only annotations; direct annotated workload calls in async contexts now fail while cheap sync helpers remain allowed. agent reviewer approval is recorded in `reviews/adhoc_enforce_workload_annotations_review.md`.
- 2026-05-12 blocking offload target restriction ([#2100](https://github.com/sifr-lang/sifr/pull/2100)): `task.spawn_blocking()` and `ThreadPoolExecutor.submit()` now require classified `@blocking_io`, `@cpu_heavy`, or known stdlib offload targets and preserve existing arity/sendability/result diagnostics. agent reviewer approval is recorded in `reviews/adhoc_restrict_blocking_offload_targets_review.md`.
- 2026-05-12 full local validation: `scripts/run_all_tests.sh` passed with `profile=pr`, 73 pass fixtures, 0 failures, 28 hardening variants, and the existing group-skew advisory.
- 2026-05-12 final phase closure review: agent reviewer approved closure in `reviews/adhoc_async_effect_and_offload_diagnostics_final_review.md`.
