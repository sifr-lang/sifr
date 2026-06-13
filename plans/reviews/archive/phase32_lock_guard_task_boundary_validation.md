

## Review Summary

**Slice scope:** `milestone_async_5` lock-guard task-boundary validation — rejects `LockGuard`, `RwLockReadGuard`, `RwLockWriteGuard` values when passed as `scope.spawn` coroutine arguments.

### Soundness

**Diagnostic correctness:** `SIFR-OWN-0010` is the established code for non-send task-boundary captures (introduced in milestone_async_4 PR #1965). Reusing it here for lock guard values at task boundaries is correct — the enforcement point is the spawn boundary, not the guard type itself.

**Guard-type detection:** `is_lock_guard_type_name` correctly checks `LockGuard`, `RwLockReadGuard`, `RwLockWriteGuard` via `public_type_name` which strips `__compat_sifr_sync_` for stdlib compat compatibility names. This is consistent with `async_await.rs` which uses the same pattern for await-point rejection (`SIFR-OWN-0009`). The diagnostic template `{value} of type {type_name}` is defined for `SIFR-OWN-0010` (verified via `docs/errors/SIFR-OWN-0010.md`).

**Diagnostic message:** "scope.spawn() cannot move `guard` of type `LockGuard` across a task boundary because `LockGuard` is a lock guard; use an explicit synchronization primitive or keep the value in the current task" — correctly surfaces the value name (`guard`), type name (`LockGuard`), and reason text (`is a lock guard`).

**E2E fixture:** `lock_across_task_boundary_rejected.sifr` uses actual `sifr.sync` imports (`Lock`, `LockGuard`) rather than a synthetic class, matching the design doc's "actual sifr.sync Lock/LockGuard" requirement. The `# expect-error[col=30]: SIFR-OWN-0010` annotation correctly targets the `scope.spawn(` position. The `own` parameter on `worker` correctly triggers move semantics.

**HIR unit test:** `test_scope_spawn_rejects_lock_guard_argument` in `expressions_tests.rs` uses a synthetic `class LockGuard[T]: pass` (no fields), which correctly exercises the direct `is_lock_guard_type_name` check without needing field recursion.

### Phase/Design Alignment

**Design contract:** `internal_docs/async_concurrency_model.md` locked decision #11: "lock guards cannot cross await" (enforced by PR #1977) and the ownership model requires spawned tasks to capture owned, sendable, static values. Lock guards are by definition tied to their owning lock's lifetime and cannot be safely transferred across task boundaries.

**Diagnostic reuse:** Using `SIFR-OWN-0010` / `OWN_NON_SEND_TASK_CAPTURE` is semantically correct. The spawn boundary is exactly where the owned-sendable-static check is enforced for all non-send values — lock guards are non-send specifically because they hold exclusive access to locked state that cannot be proven safe across thread/task boundaries.

**Phase doc:** `internal_docs/phases/32_async_ecosystem.md` correctly records this as an in-progress slice for `milestone_async_5` with the correct negative fixture name and diagnostic code.

### False Positives / False Negatives

**No false positive risk identified:** The check fires only on direct `LockGuard`, `RwLockReadGuard`, `RwLockWriteGuard` types. Structs containing lock guards are checked via field recursion in `non_send_reason_inner` (visited with the `visiting` HashSet cycle guard). Types aliased to guard types are handled via `Type::Alias` in `non_send_reason_inner`.

**No false negative identified:** The guard check fires before the `NonSend` marker check, so user-defined subclasses of lock guard types that don't inherit `NonSend` still get caught by the structural type name check.

### Diagnostics

- `non_send_task_capture` in `ownership_diagnostics.rs` correctly uses `DiagnosticCode::OWN_NON_SEND_TASK_CAPTURE`.
- The `reason` string (``LockGuard` is a lock guard`) is passed as JSON-only field per the diagnostic contract.
- The diagnostic message correctly suggests "use an explicit synchronization primitive or keep the value in the current task" — this is accurate guidance for lock guard values at task boundaries.

### Test Adequacy

- **E2E fail fixture:** `lock_across_task_boundary_rejected.sifr` — covers the full negative path with actual stdlib imports, correct error annotation.
- **HIR unit tests:** `test_scope_spawn_rejects_lock_guard_argument` covers the synthetic case; `test_scope_spawn_rejects_non_send_field_argument` covers field recursion (struct containing non-send cell); `test_scope_spawn_rejects_self_with_non_send_field` covers self-parameter case.
- **Reuse of existing coverage:** Lock guards at await points already covered by `lock_guard_across_await_rejected.sifr` (PR #1977), separate diagnostic code `SIFR-OWN-0009`.

### Code Quality

- `non_send_reason_inner` is well-structured with cycle protection via `visiting` HashSet.
- `public_type_name` strips compat prefixes for clean diagnostic text.
- The check ordering (guard type → NonSend marker → field recursion) is correct.
- All local validations pass: unit tests, e2e fail suite, full quick lane.

### One Observation (non-blocking)

The phase doc says "In progress lock-guard task-boundary validation slice" without a PR number. The context states this is the slice being reviewed. If a PR is to be opened, it should be recorded with the correct number once filed.

---

REVIEW_STATUS: SATISFIED
