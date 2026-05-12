# Review: Reject Fake Async And Fake Await (SIFR-ASYNC-0001/0002)

## Verdict: **APPROVED**

The milestone is complete and correct. No blocking issues.

---

## Correctness

### SIFR-ASYNC-0001 — async def with NoSuspend rejected
- **Location**: `typing_and_functions.rs:1228-1242`
- **Trigger condition**: `effective_is_async && matches(summaries.get(func.name), Some(NoSuspend))`
- **Emission point**: Uses `func.name.range()` — points at the function name token
- **Error code**: `ASYNC_NO_SUSPEND` (`sifr_diagnostics::DiagnosticCode::ASYNC_NO_SUSPEND`)
- **Message**: "async function 'X' has no real suspension effect; use 'def' unless an explicit async protocol escape hatch is required"

### SIFR-ASYNC-0002 — await of NoSuspend coroutine rejected
- **Location**: `async_await.rs:42-57`
- **Trigger condition**: `HirExpr::Call` to an async function where `summaries.get(func) == Some(NoSuspend)`
- **Emission point**: Uses `await_expr.value.range()` — points at the call expression
- **Error code**: `ASYNC_AWAIT_NO_SUSPEND` (`sifr_diagnostics::DiagnosticCode::ASYNC_AWAIT_NO_SUSPEND`)
- **Message**: "awaited async function 'X' has no real suspension effect; remove await and make it a synchronous function"

### Transitive detection
- `collect_async_suspension_summaries` in `async_effects.rs` iterates to fixed-point, propagating NoSuspend up the call chain.
- Test `keeps_fake_async_wrapper_chain_no_suspend` verifies `leaf` + `wrapper` both get `NoSuspend` when the leaf has no suspension points.
- Test `propagates_transitive_same_task_await_summaries` verifies `Suspends` propagates when the leaf has `task.sleep`.

### task.sleep() is recognized as Suspends
- Intrinsic lowering in `task_calls.rs` produces `HirExpr::Call { func: "__sifr_task_sleep", ... }`.
- `summarize_expr` in `async_effects.rs` catches this via the `Await` → `summarize_awaited_expr` → falls through to `Suspends` path.
- Unit tests confirm `marks_direct_timer_wait_as_suspending` passes.

---

## Diagnostic Code Registry

- `ASYNC` family registered in `codes.rs` with `reserved_base: "SIFR-ASYNC-0000"` ✅
- `ASYNC_NO_SUSPEND` and `ASYNC_AWAIT_NO_SUSPEND` defined, registered, and included in `ACTIVE_DIAGNOSTIC_CODES` ✅
- Both entry metadata complete: owner, severity, message template, deduplication args ✅
- Docs generated for both codes ✅

---

## Test Coverage

### Fail fixtures
| Fixture | Expected errors | What it tests |
|---|---|---|
| `async_no_suspend_rejected.sifr` | SIFR-ASYNC-0001 (1×) | Standalone NoSuspend async def |
| `async_transitive_no_suspend_await_rejected.sifr` | SIFR-ASYNC-0001 (2×), SIFR-ASYNC-0002 (1×) | Transitive NoSuspend chain + await |
| `async_protocol_no_suspend_requires_escape_hatch.sifr` | SIFR-ASYNC-0001 (1×) | Protocol-shaped also rejected |
| `await_sync_function_rejected.sifr` | SIFR-TYPE-0002 (existing) | Sync await preserved as hard error |

### Pass fixtures updated
All 12 pass fixtures that had bare `async def` bodies now include `await task.sleep(0.0)` to maintain `Suspends` classification.

### Unit tests
All 14 unit tests in `expressions_tests.rs` updated with `await task.sleep(0.0)` in worker functions — maintains test coverage for spawn/handle semantics without triggering the new diagnostics.

---

## Preserved Behaviors

- **Sync await still hard errors**: `await_sync_function_rejected.sifr` shows `SIFR-TYPE-0002` still fires when awaiting a non-awaitable (sync) function.
- **Workload annotation warnings preserved as warnings**: `blocking_io_annotation_warning.sifr` and `cpu_heavy_annotation_warning.sifr` continue to pass with `SIFR-TYPE-0903` as a warning. This is intentional per the design — hard-error enforcement is a later milestone.

---

## Validation Results

```
cargo test -p sifr_hir async_effects    # 4/4 pass
cargo test -p sifr -- test_e2e_pass    # pass
cargo test -p sifr -- test_e2e_fail    # pass
cargo test -p sifr_diagnostics -- codes::tests::registry_skeleton_is_internally_consistent  # pass
scripts/run_all_tests.sh --profile quick  # pass (pre-existing local validation)
```

---

## Minor Observation (Non-blocking)

The `async_protocol_no_suspend_requires_escape_hatch.sifr` fixture comment says "reviewed reason-bearing escape hatch", but the current implementation does not provide any escape hatch mechanism — it rejects all NoSuspend functions equally. This matches the design intent for this milestone (reject all fake async). A future escape-hatch mechanism would belong to a later milestone. No action needed now.

---

## Summary

The implementation is correct, well-tested, and ready to merge. The two new diagnostics fire at the right points (async def declaration and await expression), transitive NoSuspend propagation is verified, non-awaitable hard errors are preserved, and workload annotation warnings remain warnings.
