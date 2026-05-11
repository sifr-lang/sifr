# Ad Hoc Async Effect And Offload Diagnostics Execution

## Status

proposed

## Source

- [ad-hoc-async-effect-and-offload-diagnostics.md](./ad-hoc-async-effect-and-offload-diagnostics.md)
- [async_concurrency_model.md](../internal_docs/async_concurrency_model.md)

## Checklist

### adhoc_async_effect_0: Effect Summary Infrastructure

- [ ] Add internal async suspension summaries.
- [ ] Mark known async primitives with direct suspension effects.
- [ ] Compute transitive summaries through same-task coroutine awaits.
- [ ] Add positive validation fixtures:
  - [ ] `async_effect_summary_sleep.sifr`
  - [ ] `async_effect_summary_channel_receive.sifr`
  - [ ] `async_effect_summary_transitive_await.sifr`

### adhoc_async_effect_1: Reject Fake Async And Fake Await

- [ ] Reject `async def` bodies with `NoSuspend`.
- [ ] Reject awaiting same-task coroutines with transitive `NoSuspend`.
- [ ] Preserve existing non-awaitable hard errors.
- [ ] Add explicit protocol-conformance escape hatch only if implementation requires it.
- [ ] Add validation fixtures:
  - [ ] `async_no_suspend_rejected.sifr`
  - [ ] `async_transitive_no_suspend_await_rejected.sifr`
  - [ ] `await_sync_function_rejected.sifr`
  - [ ] `async_protocol_no_suspend_requires_escape_hatch.sifr`

### adhoc_async_effect_2: Upgrade Direct Workload Calls In Async

- [ ] Change direct `@io_bound` calls in async code from warning to error.
- [ ] Change direct `@cpu_bound` calls in async code from warning to error.
- [ ] Keep direct cheap sync helper calls in async code allowed.
- [ ] Update diagnostic docs and registry metadata.
- [ ] Add validation fixtures:
  - [ ] `io_bound_direct_call_in_async_rejected.sifr`
  - [ ] `cpu_bound_direct_call_in_async_rejected.sifr`
  - [ ] `cheap_sync_helper_in_async_allowed.sifr`

### adhoc_async_effect_3: Restrict Blocking Offload Targets

- [ ] Reject `task.spawn_blocking` on unannotated local sync functions.
- [ ] Reject `ThreadPoolExecutor.submit` on unannotated local sync functions.
- [ ] Allow annotated `@io_bound` targets.
- [ ] Allow annotated `@cpu_bound` targets.
- [ ] Preserve existing sendability, arity, and result/error constraints.
- [ ] Add validation fixtures:
  - [ ] `spawn_blocking_io_bound_allowed.sifr`
  - [ ] `spawn_blocking_cpu_bound_allowed.sifr`
  - [ ] `spawn_blocking_unannotated_rejected.sifr`
  - [ ] `thread_pool_submit_unannotated_rejected.sifr`
  - [ ] `spawn_blocking_known_stdlib_blocking_allowed.sifr`

## Local Validation

- [ ] `scripts/run_all_tests.sh --profile quick`
- [ ] `scripts/run_all_tests.sh`

## Review Notes

- Existing Phase 32 fixtures `io_bound_annotation_warning.sifr` and `cpu_bound_annotation_warning.sifr` (currently in the quick lane) test the exact behavior being upgraded from warning to error. They must be updated to negative fixtures or removed. `spawn_blocking_basic.sifr` passes an unannotated sync function and will also need updating (add `@io_bound` to the helper).
- The stdlib annotation database and FFI/external contract classification registry are infrastructure for `adhoc_async_effect_3`. Implementation should define the registry interface and note it as deferred infrastructure while unannotated functions remain rejected as offload targets.
- Claude reviewer sign-off recorded in `reviews/ad-hoc-async-effect-and-offload-diagnostics-claude-pass-1.md`.
