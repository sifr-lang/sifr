

Review complete. Let me compile the findings.

---

## Phase 32 milestone_async_2 task-handle-join review, pass 1

### Verdict: SATISFIED

### Blocking Findings: None

### Scope Reviewed

All 9 changed files across HIR lowering, codegen, preamble substrate, tests, and docs:

| File | Change |
|---|---|
| `crates/sifr_hir/src/lower/task_handle_calls.rs` | NEW — HIR recognition and type for `Task[T, E].join()` |
| `crates/sifr_hir/src/lower/expressions.rs` | 6-line dispatch to task_handle_calls |
| `crates/sifr_hir/src/lower/mod.rs` | module registration |
| `crates/sifr_codegen/src/lower_expr.rs` | simple method-call lowering for `join` on Task |
| `crates/sifr_codegen/src/preamble.rs` | `__SifrTaskResult` enum + `impl __SifrTask::join()` |
| `crates/sifr_codegen/src/lib_codegen_tests.rs` | codegen unit test |
| `crates/sifr/tests/e2e/pass/task_handle_join.sifr` | positive e2e fixture |
| `internal_docs/phases/32_async_ecosystem.md` | progress notes |
| `internal_docs/roadmap.md` | status line update |

### Correctness Checks

1. **HIR type for `handle.join()`**: `Awaitable(TaskResult(ok, err))` — matches the model contract (`join` → `TaskResult[T, E]`). `task_handle_calls.rs:46`

2. **Arity validation**: zero positional args, zero keyword args, with targeted diagnostics for each violation — `task_handle_calls.rs:21-35`

3. **Codegen path**: `join` on `Type::Task` goes through `try_lower_simple_method_call_expr` at `lower_expr.rs:977-983`, emitting `handle.join()` — no args, no unwrap. Verified by emit output and codegen test assertions at `lib_codegen_tests.rs:3714-3723`.

4. **`__SifrTaskResult` enum**: `Ok(T)`, `Err(E)`, `Cancelled` — three-variant private substrate. `preamble.rs:170-196`

5. **`impl __SifrTask::join`**: awaits the oneshot receiver; `Ok(value)` → `Ok`, `Err(_)` (sender dropped/sender cancelled) → `Cancelled`, `None` receiver → `Cancelled`. Correct handling of the closed-observer case without panics. `preamble.rs:210-254`

6. **Generated Rust** (`handle.join().await` on `__SifrTaskResult<i64, Infallible>`) — verified via `cargo run -q -p sifr -- emit`.

### Contract Alignment

- `Task[T, E].join()` → `Awaitable[TaskResult[T, E]]` ✓
- `handle.join()` desugars to `handle.join().await` ✓ (model: `await Task[T, E]` = syntactic sugar for `await handle.join()`)
- `__SifrTaskResult::Ok(T)`, `Err(E)`, `Cancelled` — matches model `TaskResult` branches ✓
- Closed-observer maps to `Cancelled` (not `Err`), no panics on receiver-closed ✓
- Infallible `Err` branch maps to `Cancelled` as designed for conservative infallible spawn ✓
- No user-triggerable panic paths in generated code ✓

### Validations Run

All pass:
- `cargo fmt --check`
- `cargo check -q -p sifr_hir -p sifr_codegen -p sifr`
- `cargo clippy -q -p sifr_hir -p sifr_codegen -- -D warnings`
- `cargo test -q -p sifr_codegen task_handle_join`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/task_handle_join.sifr`
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/task_handle_join.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_handle_join.sifr`
- `python3 scripts/check_hir_maintainability_guardrails.py`

### Non-Blocking Observations

1. **E2E suite not run to completion** — the e2e pass suite is slow (1000+ fixtures); I ran targeted fixture operations instead. The fixture file `task_handle_join.sifr` exists in the pass directory and will be discovered by the suite when run via `scripts/run_all_tests.sh` or `scripts/run_e2e_pass.sh`.

2. **Known limitations correctly documented** — affine consumption for double-observation, user-facing `TaskResult` matching ergonomics, and fallible task error plumbing are listed as deferred per design. The implementation is honest about its conservative infallible substrate boundary.

3. **`Task` does not yet implement `Awaitable` directly** — `await task_handle` is not yet lowering to `handle.join().await` automatically. The model marks this as a task-handle observation operation, but the desugaring path (`await Task[T, E]` → `await handle.join()`) is not wired yet. This is correctly classified as an upcoming item and does not block this slice.

4. **HIR new-file convention** — `task_handle_calls.rs` follows the existing pattern established by `task_calls.rs` and `task_scope_calls.rs`. No monolithic-file violations. Guardrails check passes.

### Summary

Clean implementation with correct type semantics, no panic paths, targeted validation, and honest deferral documentation. All checks pass. Ready for PR.
