# Concurrency Runtime structured-task capability Traceability

Capability: `concurrency-runtime structured tasks`

## Production Surface Traceability

| Surface | Evidence | Notes |
| --- | --- | --- |
| `task.TaskGroup(ctx=None)` | `crates/sifr_lowering/src/lower/expressions_tests/task_runtime_rules_tests.rs::test_task_group_accepts_reserved_none_context`; `task_group_basic`; `task_group_error_cancels_siblings`; `task_group_fail_fast_spawn_order` | Reserves `ctx` shape for concurrency/runtime readiness. Non-`None` context values are rejected until propagation semantics exist. |
| `task.spawn_scoped(..., ctx=None)` | `task_runtime_rules_tests.rs::test_task_spawn_scoped_lowers_through_named_owner_with_reserved_none_context`; `task_spawn_scoped_named_owner`; `task_spawn_scoped_without_owner_rejected` | Module-level helper proves an active named structured owner and lowers through the same task-boundary enforcement as `group.spawn`. |
| `scope.spawn(...)` / `TaskGroup.spawn(...)` | `spawn_owned_send_value`; `spawn_owned_move_value`; `spawn_borrowed_value_escapes_rejected`; `task_group_heterogeneous_error_rejected`; `ownership_and_async::test_scope_spawn_rejects_non_send_field_argument`; `task_runtime_rules_tests.rs::test_sequential_same_name_task_groups_do_not_share_error_type_state`; `task_runtime_rules_tests.rs::test_sequential_same_name_task_groups_do_not_share_open_state` | Direct coroutine call, affine handle, homogeneous group-error, borrowed capture, non-send capture, and sequential same-name owner cleanup checks stay intact. |
| `task.gather(...)` | `task_gather_ordered`; `task_gather_error_cancels_siblings`; `task_gather_cleanup_error_secondary` | Homogeneous task collection, ordered result, failure/cancellation evidence. |
| `task.race(...)` | `task_race_cancels_losers`; `task_runtime_rules_tests.rs::test_task_race_consumes_handle_collection_binding` | Homogeneous collection, first completion, loser cancellation, input handle consumption. |
| `task.select(first=..., second=...)` | `task_select_first_completion`; `cancelled_task_use_rejected`; `task_runtime_rules_tests.rs::test_task_select_rejects_positional_branches`; `task_runtime_rules_tests.rs::test_task_select_rejects_single_named_branch` | Named branch call form replaces positional select. Current binary result container maps branch order to `Select2.First` / `Select2.Second`. Duplicate branch names are rejected by the parser before lowering. |
| `task.sleep(...)` | `task_scope_unobserved_child_waits`; `task_handle_await`; `async_effect_summary_sleep` | Real suspension and task scheduling fixture coverage. |
| `task.timeout(...)` / timeout context manager | `task_timeout_success`; `task_timeout_expiry`; `task_timeout_completion_wins_tie`; `cancellation_scope_timeout`; `test_task_timeout_context_manager_requires_timeout_error_result_for_awaits` | Typed timeout evidence, cancellation, and return-type enforcement. |

## CPython Asyncio Family Mapping

| CPython family | Sifr disposition | Representative fixtures |
| --- | --- | --- |
| `Lib/test/test_asyncio/test_taskgroups.py` | `adapted-for-sifr-api` | `task_group_basic`, `task_group_error_cancels_siblings`, `task_group_fail_fast_spawn_order`, `task_spawn_scoped_named_owner` |
| `Lib/test/test_asyncio/test_tasks.py` | `adapted-for-sifr-api` | `task_handle_await`, `task_handle_join`, `task_handle_collection_consumed`, `task_gather_ordered`, `task_race_cancels_losers`, `task_select_first_completion` |
| `Lib/test/test_asyncio/test_waitfor.py` | `adapted-for-sifr-api` | `task_timeout_success`, `task_timeout_expiry`, `task_timeout_double_observe_rejected` |
| `Lib/test/test_asyncio/test_timeouts.py` | `adapted-for-sifr-api` | `cancellation_scope_timeout`, `cancellation_nested_scopes`, `task_timeout_completion_wins_tie` |
| `Lib/test/test_asyncio/test_runners.py` | `waived-with-rationale` | Raw event loop and runner policy APIs are rejected by baseline and legacy-subprocess rejection capabilities namespace diagnostics; Sifr code uses compiler-owned async entrypoint lowering. |

## Validation Coverage

| Lane | Representative entries |
| --- | --- |
| Create PR | `task_spawn_scoped_named_owner`, sendability fixtures, cancellation fixtures, channel/sync representatives |
| Merge | `task_scope_unobserved_child_waits`, `task_group_basic`, `task_spawn_scoped_named_owner`, `task_gather_ordered`, `task_race_cancels_losers`, `task_select_first_completion`, `task_spawn_fallible_result`, cancellation fixtures |

## Open Capability Boundaries

structured-task capability reserves task context shape only. `sifr.task.Context` propagation, explicit context keys, and request handoff semantics remain owned by concurrency/runtime readiness. Raw event-loop policy, callbacks/transports, and `contextvars` parity remain rejected or deferred under baseline and legacy-subprocess rejection capabilities decisions.
