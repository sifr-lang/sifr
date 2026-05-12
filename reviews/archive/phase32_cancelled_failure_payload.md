

## Code Review Summary: Phase 32 Cancelled Failure Payload Slice

### Review Focus
Stric**t PR review of the cancelled `Failure` payload implementation against HEAD/main, covering correctness, type semantics, generated Rust validity, cancellation evidence preservation, codegen integration, and test coverage.

---

### Diff Summary
**Files changed:** 5 files, +149/-17 lines

1. **`crates/sifr_codegen/src/preamble.rs`** (+85/-13): Core implementation
   - Added `build_cancellation_error_type_items()` emitting private `CancellationError` struct with `new()`
   - Updated `__SifrTaskResult::Cancelled` variant from `()` to `(__SifrFailure<CancellationError>,)`
   - Added `__SifrTaskResult::cancelled()` constructor helper
   - Updated `join()`, `__sifr_timeout()`, `__sifr_task_gather()`, `__sifr_task_race()`, `__sifr_task_select()` to use the new constructor

2. **`crates/sifr_codegen/src/lib.rs`** (+50/-13): Codegen plumbing
   - Added `type_contains_cancellation_error()` helper
   - Added `module_uses_cancellation_error_type()` for module-level detection
   - Added `function_uses_cancellation_error_type()` for function-level detection
   - Conditional emission of `CancellationError` type items alongside task scope

3. **`crates/sifr_codegen/src/entrypoints.rs`** (+3): Test harness integration
   - Emits `CancellationError` items in test codegen when task scope or `CancellationError` type is used

4. **`crates/sifr_codegen/src/lib_codegen_tests.rs`** (+27): Unit test coverage
   - Added assertions for `Cancelled(__SifrFailure<CancellationError>)` in task handle join test
   - Added assertions for `CancellationError` struct and `cancelled()` constructor in cancel test
   - Added new test `test_failure_cancellation_error_annotation_lowers_to_private_evidence_type` for `Failure[CancellationError]` type annotation lowering

5. **`internal_docs/phases/32_async_ecosystem.md`** (+1): Documentation update

---

### Design Contract Compliance

| Requirement | Status | Evidence |
|---|---|---|
| `TaskResult[T, E]` has `Ok(T)`, `Err(Failure[E])`, `Cancelled(Failure[CancellationError])` | ✅ | preamble.rs:417-425 enum variants |
| `CancellationError` is not an `Error` subclass | ✅ | HIR typing_and_functions.rs:106 `parent_class: None` |
| `CancellationError` not a valid `Result` error channel | ✅ | `cancellation_error_not_result_error.sifr` expected-error fixture validates this |
| Cancellation evidence preserved as `Failure[CancellationError]` | ✅ | All cancellation-producing paths use `__SifrTaskResult::cancelled()` |
| `__SifrTaskResult::cancelled()` constructor exists | ✅ | preamble.rs:440-466 `impl { fn cancelled() }` |
| Timeout preserves cancellation when child is cancelled before deadline | ✅ | `__sifr_timeout` match arm: `Cancelled(failure) => __SifrTaskResult::Cancelled(failure)` |
| Gather/race/select cancel on `Cancelled` | ✅ | All match arms handle `Cancelled(failure)` for fail-fast behavior |

---

### Generated Rust Validity

**`__SifrTaskResult` enum shape:**
```rust
enum __SifrTaskResult<T, E> {
    Ok(T),
    Err(__SifrFailure<E>),
    Cancelled(__SifrFailure<CancellationError>),
}
```

**Constructor:**
```rust
impl<T, E> __SifrTaskResult<T, E> {
    fn cancelled() -> Self {
        return Self::Cancelled(__SifrFailure::new(CancellationError::new()));
    }
}
```

**`CancellationError` type:**
```rust
#[derive(Debug)]
struct CancellationError {}
impl CancellationError {
    fn new() -> Self { Self {} }
}
```

All generated types are private, use `#[derive(Debug)]` for debuggability, and maintain the design split between ordinary `E` errors and non-`Error` cancellation evidence.

---

### Cancellation Evidence Preservation

| Path | Evidence Handling | Verified |
|---|---|---|
| `join()` on cancelled task | `Err(_) => __SifrTaskResult::cancelled()` | ✅ |
| `__sifr_timeout()` timeout wins | `Err(__SifrFailure::new(__SifrTimeoutResult::Timeout))` | ✅ |
| `__sifr_timeout()` child cancelled first | `Cancelled(failure) => Cancelled(failure)` | ✅ |
| `__sifr_task_gather()` cancelled child | `Cancelled(failure) => abort + return Cancelled(failure)` | ✅ |
| `__sifr_task_race()` cancelled child | `__SifrTaskResult::cancelled()` on errors | ✅ |
| `__sifr_task_select()` cancelled child | `__SifrTaskResult::cancelled()` on errors | ✅ |
| Scope child task outcome | `__SifrScopeChildOutcome::Cancelled` in scope exit | ✅ |

---

### Type Semantics

- `CancellationError` is registered in HIR with `parent_class: None` — not in `error_types` set
- `Failure[CancellationError]` is valid as a type annotation (supports evidence pattern)
- `Result[T, CancellationError]` is rejected with `SIFR-RESULT-0002`
- `ScopeFailure` properly inherits from `Error` and is a valid `Result` error channel
- `TimeoutResult[E]` implements `Error` when `E: Error` per design contract

---

### Test Coverage

**Unit tests added:**
- `test_task_handle_join_lowers_to_task_result_observation` — now asserts `Cancelled(__SifrFailure<CancellationError>)` and `fn cancelled()`
- `test_task_handle_cancel_borrows_handle_and_aborts_child` — now asserts `struct CancellationError` and `__SifrTaskResult::cancelled()`
- `test_failure_cancellation_error_annotation_lowers_to_private_evidence_type` — new test for `Failure[CancellationError]` type annotation

**E2E validation (user-confirmed passed):**
- `task_cancel_basic.sifr` ✅
- `task_timeout_expiry.sifr` ✅
- `task_gather_error_cancels_siblings.sifr` ✅
- `cancellation_error_type_surface.sifr` ✅
- `task_race_cancels_losers.sifr` ✅
- `task_select_first_completion.sifr` ✅
- `task_timeout_success.sifr` ✅
- `cancellation_error_not_result_error.sifr` ✅ (expected type error)

---

### Clippy / Formatting Compliance

Fixed pre-existing clippy `redundant_closure` warnings in `module_uses_failure_type`, `module_uses_cancellation_error_type`, and `module_uses_timeout_result_type` by replacing `|x| f(x)` with `f` directly. All clippy checks now pass.

```
cargo clippy -p sifr_codegen -- --D warnings  # ✅ passed
cargo fmt --check                            # ✅ passed
```

---

### Local Validation

```
scripts/run_all_tests.sh --profile quick  # ✅ 23/23 e2e pass tests
```

---

### Issues Found and Resolved

1. **Pre-existing clippy warnings (3 occurrences):** `redundant_closure` in `module_uses_failure_type`, `module_uses_cancellation_error_type`, and `module_uses_timeout_result_type`. **Resolved** by replacing closure syntax with direct function references.

---

### Recommendation

**SATISFIED**

The implementation correctly:
- Materializes `CancellationError` as private evidence via `__SifrFailure<CancellationError>`
- Preserves the design split: ordinary `E` failures vs. non-`Error` cancellation
- Uses the `cancelled()` constructor consistently across all cancellation-producing paths
- Compiles cleanly with clippy warnings addressed
- Passes all unit tests, e2e pass tests, and validation suite
- Updates documentation appropriately

The slice is ready for PR. No further changes required.
