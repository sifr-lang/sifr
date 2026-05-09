

Based on my comprehensive review of the staged changes, here is my analysis:

---

## Review: Phase 32 / milestone_async_2 task-handle timeout

### Verdict: SATISFIED ✓

No correctness bugs, phase-contract mismatches, or blocking issues found. Implementation is clean and aligned with the design contract.

---

### 1. Correctness Analysis

**Type System** (`crates/sifr_type_system/src/types.rs`):
- `TimeoutResult(Box<Type>)` added correctly as a private wrapper
- Correctly includes error type only (not Ok type) per the design
- Sort key 21 in union handling (`union.rs`) is correctly inserted before BlockingTask
- `is_assignable_to()` handles `TimeoutResult` correctly

**HIR Lowering** (`crates/sifr_hir/src/lower/task_calls.rs:88-155`):
- All validation checks implemented correctly:
  - Async-only (lines 89-96)
  - Keyword rejection (lines 97-104)
  - Exact 2 args (lines 105-112)
  - First arg Type::Task check (lines 114-127)
  - Duration Int/Float check (lines 129-142)
  - Handle consumption via `mark_moved()` (lines 143-145)
- Result type `Awaitable[TaskResult[T, TimeoutResult[E]]]` is correct

**Codegen** (`crates/sifr_codegen/src/preamble.rs:205-223, 302-320`):
- `__SifrTimeoutResult<E>` enum with `Inner(E)` and `Timeout` variants is correct
- `__sifr_timeout` method signature and behavior match the design:
  - `biased` select for deterministic inner-wins same-tick behavior ✓
  - Abort + await receiver on timeout ✓
  - Returns `TaskResult.Err(TimeoutResult::Timeout)` on expiry ✓
  - Returns `Cancelled` when receiver is None ✓

---

### 2. Phase/Document Alignment

| Design item from `32_async_ecosystem.md` | Status |
|---|---|
| `task.timeout(handle, duration)` accepts task handles | ✓ Implemented |
| Consumes handle through ownership tracking | ✓ `mark_moved()` called |
| Races observation against private timeout | ✓ `tokio::select!` |
| Cancels on deadline expiry | ✓ `abort_handle.abort()` |
| Returns `TimeoutResult`-carrying `TaskResult` | ✓ Correct type |
| Same-tick inner completion priority | ✓ `biased` select |
| Private type-system representation | ✓ `TimeoutResult` |
| Handle-form timeout (not context-manager) | ✓ Deliberate scope |

The implementation correctly defers the context-manager form (`task.timeout(duration)` as `async with`) per the design document.

---

### 3. Test Coverage

| Fixture | Type | Coverage | Verified |
|---|---|---|---|
| `task_timeout_success.sifr` | pass | Inner completion before deadline | ✓ Runs successfully |
| `task_timeout_expiry.sifr` | pass | Deadline expiry with cleanup | ✓ Runs successfully |
| `task_timeout_error_type.sifr` | fail | Non-task first argument rejected | ✓ Diagnostic SIFR-TYPE-0002 |
| `task_timeout_double_observe_rejected.sifr` | fail | Handle consumed, second await rejected | ✓ Diagnostic SIFR-OWN-0001 |

HIR unit test (`test_task_timeout_consumes_handle_binding`) and codegen unit test (`test_task_timeout_handle_lowers_to_private_timeout_result`) both pass.

---

### 4. Minor Observations (Non-blocking)

1. **Generated code style**: The `__sifr_timeout` body is emitted as a raw string literal. This is acceptable since it's a private codegen path, but the implementation correctly avoids any data-dependent `.unwrap()`/`.expect()` in user runtime paths.

2. **E2E manifest**: The new fixtures are lexically discovered (per `e2e.rs` contract), so no manifest update is needed. Run confirmed: both pass fixtures execute successfully, both fail fixtures produce expected diagnostics.

3. **Validation run**: User reported full `scripts/run_all_tests.sh --profile quick` passed. My spot-checks confirm the same.

---

### 5. Required Fixes

**None.** All correctness criteria are met.

---

### 6. Test/Validation Gaps

**None identified.** The implementation covers:
- Happy path (inner completes before deadline)
- Expiry path (deadline fires, cleanup awaited)
- Type error path (non-Task first argument)
- Ownership path (handle consumed, double observe rejected)
- Unit tests for HIR and codegen layers

---

### Summary

This implementation correctly delivers the conservative handle-form timeout slice as specified in `milestone_async_2`. The type system, HIR lowering, codegen, tests, and documentation are all consistent with the design contract. The `biased` select ensures deterministic inner-completion-priority on same tick, and the abort+await pattern correctly implements cleanup-before-return on expiry. No blockers remain.
