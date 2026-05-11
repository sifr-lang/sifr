

All validations confirmed:

**Fixture correctness:**
- `cancelled_task_use_rejected.sifr` (line 485 checklist match): `task.select` consumes both handles, winner completes, loser is cancelled, then trying to `await second` (the loser) fails with `use of moved value` - ✅ semantics correct
- `task_group_error_type_not_carried_rejected.sifr` (line 489 checklist match): function returns `Result[None, ValueError]` but contains a `TaskGroup` that can surface `ScopeFailure` at exit - ✅ compiler correctly rejects it with the scope-failure return-typing diagnostic

**Demo correctness:**
- Demonstrates scope-owned `gather` with deterministic input ownership
- Demonstrates `select` loser cancellation (fast task wins, slow task is cancelled before it can write the marker file)
- Demonstrates `TaskGroup` fail-fast sibling cancellation (the failing child cancels the slow sibling before its write completes)
- Assertions verify the expected outcomes pass at runtime - ✅

**Doc update:** One honest progress entry added for this slice - ✅

**Compiler changes needed?** None. This slice correctly uses existing compiler behavior (moved-value ownership tracking, scope-failure return typing) to validate the structured concurrency semantics from the already-merged PR #1936 - ✅

**SATISFIED**
