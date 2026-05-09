

Based on my thorough review, here is my assessment:

---

## Review: Phase 32 / milestone_async_2 — `async with task.timeout(duration)` context-manager

### Verdict: **SATISFIED**

---

### 1. Findings (ordered by severity)

#### No blockers found

All core correctness criteria pass:

**HIR Type Enforcement:**
- `crates/sifr_hir/src/lower/async_with.rs:362-373` — Timeout blocks containing awaits require enclosing function return type `Result[..., TimeoutError]`
- Type check is correct: uses `resolve_alias()` on return type before matching `Type::Result`
- Duration validation (int or float) at lines 338-347
- `TimeoutError` properly registered as `Error` subclass

**Codegen Await Wrapping:**
- `crates/sifr_codegen/src/stmt_support_emitter.rs:1765-1776` — `active_timeout_durations` stack intercepts `Await` expressions during body lowering
- `crates/sifr_codegen/src/render.rs:900` — renders as `match tokio::time::timeout(duration, future).await { Ok(v) => v, Err(_) => return Err(TimeoutError::new(...)) }`
- Return in body correctly returns from enclosing function (body not wrapped in closure)
- `TimeoutError` struct generated via existing error class infrastructure

**No-Await Path Preserved:**
- Confirmed: `async_with_timeout_builtin.sifr` (no await) compiles to empty async block, no tokio::time::timeout emitted
- `task_timeout_context_manager.sifr` (with await) correctly wraps each await

**Timeout Actually Fires:**
- End-to-end test confirms: `task.sleep(10.0)` inside `task.timeout(0.1)` produces `TimeoutError { message: "task timeout expired" }`

---

### 2. Required Fixes

**None** — no correctness issues found.

---

### 3. Test/Validation Gaps

**Pre-existing test failures (unrelated):**
- 22 unit tests in `sifr_codegen` were already failing on `main` branch (verified via `git stash` + test on clean HEAD)
- These are in unrelated areas: fieldless classes, iterator bindings, option truthiness, etc.

**Sufficient coverage:**
- HIR unit test: `test_task_timeout_context_manager_requires_timeout_error_result_for_awaits`
- Codegen unit test: `test_task_timeout_context_manager_wraps_awaits`
- E2E pass: `task_timeout_context_manager.sifr` (runs successfully)
- E2E fail: `task_timeout_context_manager_return_type_rejected.sifr` (rejects `-> None`)
- Runtime verification: timeout actually fires on `task.sleep(10.0)` inside 0.1s timeout

---

### 4. Phase/Design Alignment

| Contract | Status |
|----------|--------|
| Built-in `TimeoutError` availability | ✓ Added to HIR builtin errors, codegen error refs, intrinsic registry |
| HIR requires `Result[..., TimeoutError]` when body contains await | ✓ Implemented with correct type checking |
| No-await timeout blocks remain valid | ✓ Verified via `async_with_timeout_builtin.sifr` |
| Await in timeout body wraps with `tokio::time::timeout` | ✓ Each await intercepted and wrapped |
| Body `return` returns from enclosing function (no closure) | ✓ Body lowering is direct, not closure-wrapped |
| Conservative scope: no spawn boundary, no public runtime object | ✓ Uses private `tokio::time::timeout` substrate |
| Phase docs updated | ✓ `32_async_ecosystem.md` reflects slice status |

---

### 5. Minor Observations (non-blocking)

1. **Nested timeouts use innermost only** — Nested `async with task.timeout(...)` blocks each independently wrap their awaits with their own duration. This is conservative but correct behavior for v1. A future slice could implement nested timeout semantics.

2. **`__sifr_timeout_value` name collision risk** — This identifier is used for the ok-arm binding in `render.rs:900`. It's not reserved, but in practice it's unlikely to conflict with user code in the narrow scope of the timeout match.

3. **No test for `task.timeout(handle)` (task handle timeout)** — The existing `task_timeout_handle` slice (milestone_async_1) is separate. This slice only covers the context-manager form `task.timeout(duration)`. The docs correctly scope this.

4. **Pre-existing unit test regressions** — 22 tests in `sifr_codegen` were failing before this change. Not a blocker for this slice.

---

### 6. Validation Summary

| Check | Result |
|-------|--------|
| `cargo check -p sifr_hir -p sifr_codegen` | ✓ |
| `cargo clippy -p sifr_hir -p sifr_codegen` | ✓ |
| `cargo fmt --check` | ✓ |
| `cargo test -p sifr_hir task_timeout_context` | ✓ 1 passed |
| `cargo test -p sifr_codegen task_timeout_context` | ✓ 1 passed |
| `cargo run -p sifr -- emit task_timeout_context_manager.sifr` | ✓ Generates correct Rust |
| `cargo run -p sifr -- run task_timeout_context_manager.sifr` | ✓ Executes successfully |
| `cargo run -p sifr -- emit async_with_timeout_builtin.sifr` | ✓ No-await path unchanged |
| `cargo test -p sifr -- test_e2e_fail` | ✓ 1 passed (return type rejection) |
| Timeout fires at runtime | ✓ `Error: TimeoutError { message: "task timeout expired" }` |
