

## Review: race loser secondary evidence slice

### Decision: **SATISFIED**

---

### Findings

**No critical issues found.**

---

### Verification Against Design

| Design expectation | Implementation | Status |
|---|---|---|
| `TaskResult[T, E]` has `Ok(T)`, `Err(Failure[E])`, `Cancelled(Failure[CancellationError])` | `__SifrTaskResult` enum at preamble.rs:417-443 | ✓ |
| `Failure[E]` has primary + secondary evidence | `__SifrFailure<E>` struct with `primary`, `secondary` fields at preamble.rs:193-289 | ✓ |
| `task.race` cancels losers by default | `abort_handle.abort()` loop at preamble.rs:1076-1078 | ✓ |
| Winner `Err`/`Cancelled`: loser cleanup failures attach as `SecondaryError` evidence | 4-case `match (&mut first, loser)` at preamble.rs:1086-1098 calls `failure.push_secondary_message(...)` | ✓ |
| Winner `Ok`: loser cleanup failures surface at owning `TaskScope` exit | `_ => {}` catch-all arm at preamble.rs:1099 drops losers silently; `__sifr_join_all` will observe them | ✓ |
| Uses existing message-based `SecondaryError` helper | `push_secondary_message("race loser task failed".to_string())` | ✓ |
| `__sifr_task_select` is out of scope | `__sifr_task_select` (preamble.rs:1050-1087) not modified | ✓ (aligned with slice scope) |

---

### Implementation Quality

**preamble.rs:1027-1047 (`__sifr_task_race`):**
- `first` declared as `mut` (line 1070) enabling secondary attachment
- Loser drain loop correctly uses `observer_count.saturating_sub(1)` to skip winner
- Match arms cover all failure-like combinations: `Err+Err`, `Err+Cancelled`, `Cancelled+Err`, `Cancelled+Cancelled`
- `Ok` winner case is explicit no-op (`_ => {}`), consistent with design

**lib_codegen_tests.rs:3825-3876:**
- `test_task_race_lowers_to_private_race_helper` asserts `let Some(mut first)` and `"race loser task failed\".to_string()"`
- `test_task_race_fallible_tasks_keeps_error_parameter_unwrapped` asserts `"race loser task was cancelled\".to_string()"`
- Both tests verify `__SifrTaskResult<T, E>` generic preservation

---

### Residual Risks

1. **Same-tick loser drain race**: If a loser task completes and sends its result in the same scheduler tick as the winner (before `abort()` takes effect), the drain loop will observe it. This is acceptable — the implementation drains all observations regardless, and the loser will typically already be cancelled by `abort()` before sending. The drain is a safety net, not the primary mechanism.

2. **`__sifr_task_select` not addressed**: Per the design doc and phase tracker, this slice covers `task.race` only. `task.select` has the same loser-secondary-evidence concern (locked design decision #7 in `32_async_ecosystem.md`), but it is deferred. No action required.

3. **Secondary evidence ordering**: When multiple losers produce observations, they are attached in whatever order the channel delivers them. This is non-deterministic but acceptable — `SecondaryError` is evidence, not a semantically ordered sequence. The primary error is preserved; secondary evidence is supplementary.

---

### Validation Results Confirmed
- Unit tests pass: `cargo test -p sifr_codegen -- task_race` → 2 passed
- E2E fixture passes: `task_race_cancels_losers.sifr` (confirms loser cancellation, not loser secondary evidence — correct for current slice scope)
- Quick validation profile passed (user-reported)
