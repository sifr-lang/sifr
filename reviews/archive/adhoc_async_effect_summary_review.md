

All validations pass. Here is my review finding:

---

## Review: adhoc_async_effect_0 — Async Suspension Summary Infrastructure

### Severity: None (no blocking findings)

### Code Review

**`crates/sifr_hir/src/lower/async_effects.rs`**

| Aspect | Assessment |
|--------|------------|
| Two-state `enum AsyncSuspensionSummary` | Correct — `NoSuspend` / `Suspends` with monotonic `union` |
| Fixpoint algorithm (lines 29-46) | Correct worklist loop. Terminates in ≤|F| iterations because state can only transition `NoSuspend → Suspends` once per function. |
| `collect_top_level_async_functions` | Only considers top-level `Stmt::FunctionDef` — correct per spec. Nested async functions are not tracked (no impact on the summary use case). |
| `summarize_awaited_expr` (lines 297-315) | Core logic: same-task call to known async function → follow transitive summary; all other awaits → `Suspends`. Correct. |
| `summarize_stmt` | Covers all statement variants. `FunctionDef` / `ClassDef` return `NoSuspend` (nested definitions don't affect the enclosing function's suspension behavior). Correct. |
| `summarize_comprehension` (lines 317-346) | `generators.iter().any(|g| g.is_async)` marks as suspending — covers `async for` inside comprehensions. Correct. |
| Edge cases: `yield` / `yield from` | Marked `Suspends` directly (line 191). Correct. |
| Edge cases: `async for` / `async with` | Marked `Suspends` in `For` and `With` statement handlers. Correct. |
| Unit tests | Four tests cover the four contract points: direct primitive await, transitive same-task propagation, fake async no-suspend, async generator yield. All pass. |

**`crates/sifr_hir/src/lower/mod.rs`**

| Aspect | Assessment |
|--------|------------|
| `async_suspension_summaries` field (line 152) | Added with `HashMap::new()` initialization (line 241) and populated post-collect (line 713). |
| `use async_effects::AsyncSuspensionSummary` | Correct import. |
| Placement of `collect_async_suspension_summaries` call | Placed after all `FunctionDef` entries are registered in `ctx.functions`. This ensures the async function set is complete before summaries are computed. Correct. |

**Positive fixtures**

| Fixture | Assessment |
|---------|------------|
| `async_effect_summary_sleep.sifr` | Direct `await task.sleep(0.0)` — exercises real suspension source. |
| `async_effect_summary_channel_receive.sifr` | Direct `await receiver.receive()` and `await sender.send(...)` — exercises real suspension source through method calls. |
| `async_effect_summary_transitive_await.sifr` | `main → wrapper → leaf` chain — exercises transitive fixpoint propagation. |

**`issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md`**

All four items for milestone 0 checked off. Remaining milestones (`_1` through `_3`) are unchecked and explicitly deferred. Consistent with scope.

**`verification/validation_lanes/quick_e2e_manifest.json`**

Three new fixtures appended to quick lane (lines 44-46). Lexicographic ordering preserved.

---

### Checklist confirmed

| Check | Status |
|-------|--------|
| `cargo test -p sifr_hir -- async_effects` | ✓ 4 tests pass |
| E2E run all three fixtures | ✓ Cache hits (verified) |
| `cargo fmt --check` | ✓ |
| `git diff --check` | ✓ |
| `check_hir_maintainability_guardrails.py` | ✓ PASS |
| `run_all_tests.sh --profile quick` | ✓ Passed (warm wall-time advisory is benign) |

---

### Verdict

**No blocking findings.** The implementation correctly computes the two-state async suspension summary with transitive fixpoint propagation. The storage mechanism (`LowerCtx.async_suspension_summaries`) is in place for later enforcement (milestone `_1`). Rejection behavior is intentionally deferred. Milestone 0 is ready to PR/merge.
