

Now I have a complete picture. Let me provide the structured review.

---

## Senior Code Review: Phase 32 `milestone_async_5` — Lock Guard Return-Escape Validation

### Files Reviewed

| File | Change |
|---|---|
| `crates/sifr_hir/src/lower/task_scope_calls.rs` | Expose `is_lock_guard_type` as `pub(super)` |
| `crates/sifr_hir/src/lower/async_await.rs` | Reuse shared `is_lock_guard_type` (removes local duplicate) |
| `crates/sifr_hir/src/lower/statements.rs` | Add lock guard return-escape check in `lower_return` |
| `crates/sifr_hir/src/lower/ownership_diagnostics.rs` | Add `lock_guard_return_escape` using SIFR-OWN-0003 |
| `crates/sifr_hir/src/lower/expressions_tests.rs` | Add HIR unit test |
| `crates/sifr/tests/e2e/fail/lock_guard_escape_rejected.sifr` | E2E negative fixture |

### Design/Phase Alignment

**Correct.**

- The implementation enforces the lock guard return restriction as documented in `async_concurrency_model.md` ("Lock guards must not cross `await` points" / "lock guards cannot cross task boundaries"). Return escape is a natural third boundary.
- The check intentionally blocks only user code: `allow_intrinsic_imports` (set for stdlib `.sifr` files via `lower_module_stdlib` / `lower_module_stdlib_with_externals`) is the gate. This correctly allows `sifr.sync.Lock.lock()` and `sifr.sync.RwLock.read()/write()` to define the primitive API without triggering the diagnostic.
- Field/global/container escape checks are correctly deferred per the scope note. This is a cohesive slice, not a partial fix.
- The `is_lock_guard_type_name` helper correctly handles `__compat_sifr_sync_` prefix stripping, covering all three guard types: `LockGuard`, `RwLockReadGuard`, `RwLockWriteGuard`.

### Soundness

**Correct.**

- `lower_return` checks `is_lock_guard_type(&expr_ty)` on the lowered expression type *after* `lower_expr`, so it operates on resolved alias types. This matches the pattern used in `lower_await`.
- The guard check is placed *after* the `HirExpr::Name` borrowed-param escape check and *before* the Result-wrapping and type-assignment checks. This ordering is clean — the guard diagnostic fires independently without interfering with Result wrapping or type checking.
- The diagnostic uses the exact return value range (`val.range()`), which gives precise column reporting for `# expect-error[col=12]`.
- Diagnostic reuse of `OWN_BORROWED_PARAMETER_ESCAPES` (SIFR-OWN-0003) is acceptable: it belongs to the ownership escape family, the existing code registry covers the semantic, and prior PRs established this pattern. No new code required for this slice.

### False Positive / False Negative Assessment

**No false positives found.**

- Stdlib lock methods produce `LockGuard` via `def lock(self) -> LockGuard[T]` in `sync.sifr`. Those `.sifr` files are lowered via `lower_module_stdlib` which sets `allow_intrinsic_imports = true`, so the return check is suppressed for the stdlib implementation.
- User code that imports `LockGuard` from `sifr.sync` and returns it in a user function (e.g., `make_guard`) is correctly rejected with column-12 error on the `return` keyword. The e2e test `lock_guard_escape_rejected.sifr` confirms this with `# expect-error[col=12]: SIFR-OWN-0003`.

**No false negatives found.**

- The type check on `expr_ty` (post-`lower_expr`) catches all return forms: direct name binding, method call result, field access, etc. For the e2e case `guard = lock.lock(); return guard`, the `LockGuard[int]` type flows through the name binding and is correctly detected at `return guard`.

### Diagnostics

**Sound.**

- Message: `"cannot return lock guard: lock guards cannot escape their local critical section"` — describes the policy clearly.
- The message uses "lock guard" rather than "borrowed parameter", which is semantically correct for this case. The shared code (`SIFR-OWN-0003`) carries the generic name; the per-diagnostic message is the user-facing content.
- No mention of borrowing in the lock-guard-specific message, which is appropriate.

### Test Adequacy

**Sufficient, with one minor gap.**

- **E2E negative fixture** (`lock_guard_escape_rejected.sifr`): Uses real `sifr.sync.Lock` / `LockGuard`. Tests the return-escape path with `make_guard` returning a guard from a user function. Passes `test_e2e_fail`.

- **HIR unit test** (`test_lock_guard_return_escape_rejected`): Uses a mock `class LockGuard[T]: pass` to isolate the lowering logic. Passes. This is fine for unit coverage.

- **Minor gap**: `RwLockReadGuard` and `RwLockWriteGuard` return-escape are not separately tested in either the e2e fixture or the HIR unit test. The `is_lock_guard_type_name` function covers all three, but test coverage only explicitly exercises `LockGuard`. This is a minor test gap, not a blocking issue.

- The existing `test_lock_guard_across_await_rejected` HIR test covers the await boundary, and `lock_across_task_boundary_rejected.sifr` covers the task-boundary case.

### False Positives/Falses Negatives Worth Fixing in This Slice

None.

### HIR Maintainability

**Satisfied.** No new module created. The change exposes `is_lock_guard_type` as `pub(super)` in `task_scope_calls.rs` and reuses it in both `async_await.rs` (replacing a local duplicate) and `statements.rs`. This is the correct approach.

---

### Blocking Fixes

None. The implementation is sound, design-aligned, and correctly gated for stdlib compatibility.

---

**REVIEW_STATUS: SATISFIED**
