Reviewing the tracker against all expected facts:

**1. `milestone_async_8` status remains `in_progress`**
- Line 1013: `status: in progress` ✓

**2. PR #2080 implementation note present and correctly scoped**
- Lines 1017-1018: Note is present ✓

**3. Canonical lowering description matches**
- Line 1018: "imported `timeout` and `TaskGroup` now lower through the canonical `task.timeout(duration)` and `task.TaskGroup()` async-with paths, with no second runtime behavior" ✓

**4. Remaining surfaces not claimed; follow-ups listed correctly**
- Line 1018: "while `run`, `create_task`, `Queue`, `Future`, and unsupported-event-loop diagnostics remain follow-up slices" ✓

**5. Positive validation includes required fixtures**
- Line 1073: `asyncio_timeout_subset.sifr` ✓
- Line 1076: `asyncio_task_group_subset.sifr` ✓

**6. PR #2078 note is historically accurate, not contradictory**
- Line 1017: "while `run`, `create_task`, `TaskGroup`, `timeout`, `Queue`, and unsupported-event-loop diagnostics remained follow-up slices" — PR #2078 covered `sleep`, `wait_for`, `gather`; `TaskGroup` and `timeout` are correctly attributed to PR #2080. No contradiction. ✓

All six tracker facts are satisfied. No blockers.

**SATISFIED**
