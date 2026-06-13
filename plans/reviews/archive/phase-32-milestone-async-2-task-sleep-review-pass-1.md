# Review: Phase 32 milestone_async_2 task.sleep slice

Date: 2026-05-09
Reviewer: Claude

## Verdict: SATISFIED

This slice is acceptable to open as a PR. All concrete review criteria pass; no blockers remain.

---

## 1. HIR Lowering Correctness

**`crates/sifr_hir/src/lower/task_calls.rs`**
- `lower_task_module_call` checks all four call-site validity requirements before lowering:
  - Must be called on `task` module attribute (not a free function or other module)
  - Must be `task.sleep` specifically (future `task.X` calls fall through to `NotTaskModuleCall`)
  - Must be inside an async function (error: `task.sleep() is only valid inside async functions`)
  - Must have exactly 1 positional arg (error: `task.sleep() takes exactly one duration argument`)
  - Duration arg must lower to `Int | Float` type (error: `task.sleep() duration must be int or float, got '{type}'`)
- Keyword arguments are rejected with diagnostic: `task.sleep() does not accept keyword arguments`
- All errors use `DiagnosticCode::TYPE_MISMATCH` — correct for call-site type mismatches
- On success, emits `HirExpr::Call { func: "__sifr_task_sleep", args: [duration], ty: Awaitable(None) }` — correct affine call encoding

**`crates/sifr_hir/src/lower/expressions.rs:393-398`**
- `lower_task_module_call` is checked before `lower_method_call` — correct precedence so `task.sleep` does not fall through to method-call lowering
- `NotTaskModuleCall` falls through cleanly; `Rejected` propagates `None` (no double-reporting)

**Async context tracking**
- `ctx.current_function_is_async` is set by `lower_function_signature` before processing body (verified at `typing_and_functions.rs:949-952`)
- `task.sleep` diagnostics inside async functions are sound; diagnostics outside async functions are correct

---

## 2. Codegen Correctness and Safety

**`crates/sifr_codegen/src/lower_expr.rs:559-610` (`try_lower_task_sleep_call_expr`)**
- Duration cast to `f64` covers both `Int` and `Float` HIR input types
- Guard pattern:
  ```rust
  if __sifr_task_sleep_seconds.is_finite() && (__sifr_task_sleep_seconds > 0.0) {
      __sifr_task_sleep_seconds
  } else {
      0.0
  }
  ```
  - `is_finite()` handles `NaN`, `inf`, `-inf`
  - `> 0.0` handles negative values
  - Combined: any invalid input defaults to `Duration::from_secs_f64(0.0)` — no panic, no unwrap
- `std::time::Duration::from_secs_f64` is called with the guard expression as argument — Tokio receives a safe `Duration`
- `tokio::time::sleep(duration).await` — correct async sleep primitive

**No user-triggerable runtime panics**: All data-dependent decisions use conditional branching, not `.unwrap()`/`.expect()`. Invalid durations produce zero-duration sleep, which is well-defined Tokio behavior.

**Both lowering paths covered:**
- `try_lower_leaf_expr` (for `await task.sleep(x)`) at line 386-405
- `try_lower_simple_call_expr` (for bare `task.sleep(x)`) at line 512-514

Both route through `try_lower_task_sleep_call_expr`.

---

## 3. Tokio Dependency Detection

**Normal codegen** (`crates/sifr_codegen/src/lib.rs:745`):
```rust
if has_async_main_entrypoint || uses_task_sleep {
    crates.insert("tokio".to_string());
}
```
Tokio is added if either: (a) async `main()` detected, or (b) `task.sleep` is used anywhere in the module.

**Test codegen** (`crates/sifr_codegen/src/entrypoints.rs:145-147`):
```rust
if uses_task_sleep {
    crates.insert("tokio".to_string());
}
```
Tokio is added for test executables that use `task.sleep` even without async `main()`.

**`module_uses_task_sleep`** (`lib.rs:782-849`):
- Walks constants, functions, and class methods
- Uses `traversal::walk_expr_until` and `traversal::walk_stmts_until` with `INCLUDE_NESTED_FUNCTIONS` config
- Short-circuits on first `HirExpr::Call { func: "__sifr_task_sleep", ... }` match
- No false negatives for nested functions, class methods, or constant expressions

---

## 4. Fixture and Test Coverage

**E2E pass fixture**: `crates/sifr/tests/e2e/pass/task_sleep.sifr`
- `async def main()` with two calls: `await task.sleep(0.0)` and `await task.sleep(-0.01)`
- Passes `check`, `emit`, and `run`
- Negative-duration case covered (defaults to 0.0)

**E2E fail fixtures**:
- `task_sleep_outside_async.sifr`: `def main()` calls `task.sleep(0.0)` → `SIFR-TYPE-0002`
- `task_sleep_invalid_duration.sifr`: `async def main()` calls `await task.sleep("soon")` → `SIFR-TYPE-0002`

**Unit tests** (`crates/sifr_codegen/src/lib_codegen_tests.rs`):
- `test_task_sleep_lowers_to_tokio_sleep_and_requires_tokio`: Verifies `tokio::time::sleep` in output, `Duration::from_secs_f64` in output, and `tokio` in required crates
- `test_task_sleep_requires_tokio_without_async_main`: Verifies `tokio` is added without `#[tokio::main(...)]` for non-`main` async functions

**Validation run results** (all passed locally):
- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `cargo check -q -p sifr_hir -p sifr_codegen -p sifr`
- `cargo test -q -p sifr_codegen task_sleep` (2 tests)
- E2E pass suite with fixture manifest
- `sifr check` on both fail fixtures (correct diagnostics)
- `scripts/run_all_tests.sh --profile quick`

---

## 5. Documentation Updates

**`internal_docs/phases/32_async_ecosystem.md`** (lines 382-383):
```
- In progress task sleep slice: lower `task.sleep(duration)` inside async functions to the private runtime substrate, reject invalid duration/call sites during HIR lowering, and require Tokio only when generated code references the private sleep primitive.
- Added validation coverage for `task_sleep.sifr`, `task_sleep_outside_async.sifr`, and `task_sleep_invalid_duration.sifr`.
```
Status accurately reflects slice as in-progress. Coverage list matches fixtures added.

**`internal_docs/roadmap.md`** (line 63):
```
| 32 | Async and Ecosystem Foundation | in_progress | ... | milestone_async_1 is complete and milestone_async_2 runtime bootstrap plus `task.sleep` are underway
```
Correct: `task.sleep` is part of `milestone_async_2`.

---

## Minor Observations (not blockers)

1. **`task_calls.rs` module name**: "task_calls" is slightly generic given it currently only handles `task.sleep`. Future additions (e.g., `task.timeout`, `task.spawn`) will justify the plural. Acceptable as-is.

2. **Negative duration in fixture**: `await task.sleep(-0.01)` is handled gracefully (defaults to 0.0) but not explicitly tested. The unit test `test_task_sleep_lowers_to_tokio_sleep_and_requires_tokio` only checks `0.0`. This is a post-launch improvement opportunity, not a blocker.

3. **No demo file**: No `demos/m32_task_sleep.sifr` exists. The design doc shows `task.sleep` usage in examples, and `demos/m32_task_core_demo.sifr` exists per the phase file. Not a blocker — demo coverage can follow as a separate slice.

4. **No negative-duration HIR diagnostic**: If a user passes a negative float literal, the HIR lowering succeeds and the guard handles it at runtime. This is intentional per the design: invalid durations produce zero-duration sleep (well-defined Tokio behavior). No change needed.

5. **`RustExpr::Block` with stmts in function-call arg position**: The generated code uses block-with-let as a tokio sleep argument. Verified that `render_block_expr` handles `stmts` + `expr` correctly, and other code paths already use `RustExpr::Block` in call-arg positions (confirmed via grep — many existing uses). No concern.

---

## Summary

| Criterion | Status |
|---|---|
| HIR lowering correctness | PASS |
| Diagnostics on invalid call sites | PASS |
| Tokio `Duration` safety (no panics) | PASS |
| Tokio dep detection (normal codegen) | PASS |
| Tokio dep detection (test codegen) | PASS |
| `module_uses_task_sleep` traversal | PASS |
| E2E pass fixture | PASS |
| E2E fail fixtures with diagnostics | PASS |
| Unit test coverage | PASS |
| HIR maintainability guardrails | PASS |
| Docs updated (phase file + roadmap) | PASS |

No concrete blockers. Ready to open PR.