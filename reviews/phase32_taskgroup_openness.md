

## Review Summary

**VERDICT: SATISFIED**

The implementation is correct, complete, and phase-appropriate. Here are my findings:

### 1. Owner tracking is sound

- `task_group_spawn_owner` extracts the group name from `group.spawn(...)` method calls only when the object is a `TaskGroup` (not `TaskScope`).
- `task_handle_group_owners` maps handle binding → group name only at the spawn site (new variables in `Let` statements).
- `enforce_task_group_is_open` checks the group name only when the spawn target is a `TaskGroup`.
- Ordinary `scope.spawn()` on `TaskScope` is unaffected.

### 2. Observations are correctly placed

All 7 observation paths call `mark_task_handle_observed`:
- `await handle` in `async_await.rs`
- `handle.join()`, `handle.cancel_and_join()` in `task_handle_calls.rs`
- `task.timeout(handle, ...)` in `task_calls.rs`
- `task.gather([...])` for named list handles
- `task.race([...])` for named list handles
- `task.select(a, b)` for individual named handles

`mark_task_handle_names_moved` also recurses through list/tuple literals to reach individual handles.

### 3. Reassignment clears stale owner tracking

At line 1569 in `statements.rs`, reassignment removes the binding from `task_handle_group_owners`. Reassignment also invalidates other stale binding facts via `invalidate_rebound_binding_facts`.

### 4. Negative fixture is meaningful

`task_group_spawn_after_failure_rejected.sifr` covers the exact intended scenario: `group.spawn` → `await handle` → `group.spawn`. The diagnostic fires on the second spawn with the correct message. This is conservative as designed — the child failing doesn't matter, only that the handle was observed.

### 5. Phase doc progress note is accurate

Line 509: "task handles spawned from a named `TaskGroup` remember their owner, and v1 conservatively rejects later `group.spawn(...)` on a path after one of that group's child handles has been observed."

This matches the implementation. It does not overclaim.

### 6. No blockers identified

The implementation is complete for the TaskGroup openness slice of `milestone_async_3`.

---

**One minor note (non-blocking):** The mod.rs diff shows several blank-line removals at lines 278, 285, 292, 351 — these are stylistic and do not affect correctness.
