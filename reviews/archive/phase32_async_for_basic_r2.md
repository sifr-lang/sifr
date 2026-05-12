# Review: Phase 32 Async For Basic — Round 2 (Post-Review Fix)

## Review Scope

Post-review fix to `crates/sifr_hir/src/lower/async_with.rs` in the `phase32-async-for-basic` branch.
The fix addresses a missed await point in timeout/async-with validation.

## What Changed

**Before**: `stmt_contains_await` treated `HirStmt::AsyncFor` the same as `HirStmt::For` — checking `iter`, `body`, and `else_body` for explicit `HirExpr::Await` nodes.

**After**: `HirStmt::AsyncFor { .. }` unconditionally returns `true` in `stmt_contains_await` (line 520).

**Rationale**: `async for` is itself an implicit await point. Every iteration calls `anext().await?`, which is an await that can be interrupted by timeout. The earlier approach would have missed `async for` inside `task.timeout()` when the loop body has no explicit `await` expressions.

**New fail fixture**: `crates/sifr/tests/e2e/fail/async_for_timeout_context_manager_return_type_rejected.sifr` — `async for` over `OneShotStream` inside `task.timeout(1.0)`, no explicit await in body. Enclosing `main() -> Result[None, StreamError]` does not carry `TimeoutError`, so the compiler correctly rejects it.

## Review Findings

### 1. Unconditional `AsyncFor` → `true` is correct

`async for` is a construct that always contains await points (the implicit `anext().await?` per iteration). Even if the body has no explicit `await` expressions, timeout can fire between iterations. The unconditional return is the right choice.

Contrast with `HirStmt::For` — it has no implicit await points, so checking `iter` and `body` is correct. For `AsyncFor`, we care about the *construct itself*, not user-written await expressions inside it.

### 2. New fail fixture is correct and not overclaiming

The fixture tests exactly one thing: `async for` inside `task.timeout()` requires `TimeoutError` in the return type, regardless of whether the body contains explicit awaits.

What the fixture does NOT claim:
- Does NOT test cancellation behavior (what happens when timeout fires mid-iteration)
- Does NOT test `aclose()` / AsyncClosable cleanup
- Does NOT test channel-backed iteration
- Does NOT test nested timeout + async for

The diagnostic message is: `"async with task.timeout(duration) can time out at await points; enclosing function must return Result[..., TimeoutError]"`. This accurately describes the requirement without overclaiming.

The fixture only has `assert value == 1` in the body — no explicit await. This is the precise regression the fix addresses: the earlier code would not have flagged this case.

### 3. Both fail fixtures produce expected diagnostics

```
$ cargo run -q -p sifr -- emit crates/sifr/tests/e2e/fail/async_for_timeout_context_manager_return_type_rejected.sifr
type error: [main] async with task.timeout(duration) can time out at await points; enclosing function must return Result[..., TimeoutError]

$ cargo run -q -p sifr -- emit crates/sifr/tests/e2e/fail/async_for_non_async_iterable_rejected.sifr
type error: [main] invalid for-loop iteration: async for requires AsyncIterator[T, E] with anext() -> Result[Option[T], E], got 'list[int]'
```

Both reject correctly.

### 4. Pass fixture still passes

```
$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/async_for_stream_result.sifr
[sifr-artifact-cache] namespace=run key=e75cb4b9f1b3edc2 cache_hit=true
```

Cache hit (generated output unchanged from prior run). The fix does not affect pass-case codegen.

### 5. Other walkers unchanged — no regressions

The unconditional `true` for `AsyncFor` only affects `stmt_contains_await`. The other two scanners in the same file are unaffected:
- `stmt_contains_task_spawn` (line 594–611): Already handles `AsyncFor` by checking `iter`, `body`, `else_body` — correct for detecting explicit spawns.
- `stmt_contains_scope_early_exit` (line 666–673): Already handles `AsyncFor` recursively through `body` and `else_body` — correct for detecting return/raise/yield.

No changes needed to those functions.

### 6. Validation passes

```
scripts/run_all_tests.sh --profile quick
PASS, 54 pass fixtures, report_signature=dd74546e2bc378d9, wall_time=89.13s
```

All pass fixtures in the quick lane run and pass. The new fixture is a fail fixture and doesn't appear in the pass manifest.

## Summary

The fix is correct and minimal:
- Correctly identifies `async for` as an implicit await point
- New fail fixture covers the regression precisely, with no overclaiming
- No regressions in pass fixtures or existing fail fixtures
- Validation passes

The implementation is sound.

## Recommendation

**SATISFIED** — the post-review fix is correct and ready.

REVIEW_STATUS: SATISFIED