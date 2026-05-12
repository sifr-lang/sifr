

## Phase 32 Review: Gather Secondary Evidence Slice

**Review scope:** `git diff HEAD` on branch `phase32-gather-secondary-evidence`

### Files changed

| File | Change |
|------|--------|
| `crates/sifr_codegen/src/preamble.rs` | Added `push_secondary_message` method to `__SifrFailure<E>`; rewrote `__sifr_task_gather` to drain remaining children and record secondary evidence on the primary failure |
| `crates/sifr_codegen/src/lib_codegen_tests.rs` | Added codegen assertions for `failure_results` and `push_secondary_message` in gather tests |
| `internal_docs/phases/32_async_ecosystem.md` | Updated implementation progress note |

### Correctness assessment

**Rust generated code:**
- ✅ `push_secondary_message` signature is `fn push_secondary_message(&mut self, message: String)` — properly takes `&mut self` for in-place mutation of the `Failure` struct
- ✅ All four failure-like combinations (Err→Err, Err→Cancelled, Cancelled→Err, Cancelled→Cancelled) correctly call `push_secondary_message` with appropriate string messages
- ✅ `failure_results: Vec<Option<__SifrTaskResult<Vec<T>, E>>>` is correctly typed — stores by input index, flattened in iteration order for primary selection
- ✅ `cancelling` flag prevents double-abort when multiple failures arrive concurrently
- ✅ Drain loop completes `remaining` countdowns after abort to avoid leaking channel receivers
- ✅ Success path (ordered values) is unchanged — `Ok(ordered_values)` still preserves input order

**Ownership and move validity:**
- ✅ `failure` in match arms is accessed mutably via `failure.push_secondary_message(...)` — no ownership issues
- ✅ `failure_results.into_iter().flatten()` consumes the vector once, no double-consumption

**Type semantics:**
- ✅ `__SifrTaskResult<Vec<T>, E>` for gather return type matches the design contract
- ✅ Primary failure can be `Err(Failure<E>)` or `Cancelled(Failure<CancellationError>)` — both failure-like outcomes are handled

**Evidence preservation:**
- ✅ Later sibling failures/cancellations attach as `SecondaryError` messages to the earliest input-order primary
- ✅ `SecondaryError::new(message)` is called correctly with the string argument

### Behavioral regression check

| Scenario | Behavior | Status |
|----------|----------|--------|
| All tasks succeed | Returns `Ok([ordered values])` | ✅ Unchanged |
| First task fails | Primary is first failure, later siblings become secondary evidence | ✅ Implemented |
| Task cancelled during gather | If no ordinary error observed first, `Cancelled(Failure<CancellationError>)` becomes primary with secondary evidence from siblings | ✅ Implemented |
| Mixed Err/Cancelled | Input order determines primary; other becomes secondary evidence | ✅ Implemented |

### Test coverage

| Test | Purpose | Status |
|------|---------|--------|
| `task_gather_ordered.sifr` | Success ordering regression | ✅ Pass |
| `task_gather_error_cancels_siblings.sifr` | Fail-fast + sibling cancellation | ✅ Pass |
| `test_task_gather_lowers_to_private_gather_helper` | Codegen: infallible gather helper | ✅ Pass |
| `test_task_gather_fallible_tasks_keeps_error_parameter_unwrapped` | Codegen: fallible gather with secondary messages | ✅ Pass |

### Scope constraint: no cooperative finally cleanup

The implementation correctly uses string-based secondary messages (`"sibling task failed"`, `"sibling task was cancelled"`) rather than attempting cooperative finally-cleanup observation. This matches the design contract: this slice does not pretend to implement cooperative finally cleanup.

### Validation results

- `cargo fmt --check` ✅
- `git diff --check` ✅
- `cargo test -p sifr_codegen -- task_gather*` ✅ (2 tests pass)
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_gather_error_cancels_siblings.sifr` ✅
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_gather_ordered.sifr` ✅
- `scripts/run_all_tests.sh --profile quick` ✅ (62.49s wall time, 23 e2e pass tests, 0 failures)

---

## **SATISFIED**

The slice correctly implements:
1. `__SifrFailure::push_secondary_message` for evidence attachment
2. Gather draining: after first failure-like result triggers cancellation, remaining children are observed
3. Primary selection: earliest input-order failure-like outcome becomes primary
4. Secondary evidence: later sibling failures/cancellations attach as `SecondaryError` messages to primary
5. Success path preservation: `Ok(ordered_values)` with input ordering unchanged

The uncommitted diff is correct, minimal, and ready for commit.
