

## Code Review: Phase 32 semaphore/notify surface slice

### Severity: Informational

No correctness or regression issues found. All components are implemented correctly within the stated slice scope.

---

### 1. `classes.rs` — async method signature wrapping ✅ CORRECT

`method_signature_return_type` (lines 38–43) correctly wraps async method return types:

- `Result[T, E]` → `Coroutine[T, E]`
- non-Result `T` → `Coroutine[T, Never]`

This is applied consistently across all three code paths (regular methods line 385, protocol methods line 428, nested class methods line 642), and exactly mirrors the logic from `async_await::coroutine_result_type` (defined at `async_await.rs:63–68`).

Verified via emitted Rust: `async fn acquire(&mut self) -> Result<SemaphorePermit, ClosedError>` — coroutine-returning signature is preserved.

---

### 2. `bootstrap.rs` — stdlib metadata consistency ✅ CORRECT

`method_type_from_hir` (lines 371–378) uses identical logic:
- `coroutine_type_from_surface_return` matches HIR's `coroutine_result_type` exactly
- operator_impls (line 163–167) correctly use raw `function_type_from_params` without wrapping — operators are not async in this stdlib

The generated `Type::Class { methods, ... }` metadata correctly reflects that `acquire: Fn(&mut self) -> Coroutine<SemaphorePermit, ClosedError>`.

---

### 3. `stmt_support_emitter.rs` — structured await lowering ✅ CORRECT

The new code path (lines 1783–1816) only intercepts `HirExpr::Await { value, .. }` under the non-timeout branch. It correctly handles:

1. **`task.sleep(d)`** → `tokio::time::sleep(...).await` — preserves existing behavior, same output as the timeout branch
2. **Complex values** (method calls, task handles) → `lower_stmt_expr_for_ir` handles recursively
3. **Task/BlockingTask** → `.join()` added (via `resolve_alias_type_for_plain_call` on lines 1808–1814)
4. **Other types** (Coroutine, etc.) → passed through unchanged

The `await semaphore.acquire()` case (`Await { value: MethodCall { ... } }`) produces `acquire(...).await` — correct.

No existing `await` behavior is changed. The only behavioral delta is that method-call await expressions now lower correctly instead of being rejected.

---

### 4. `sync.sifr` — surface types ✅ CORRECT

All deferred items are explicitly documented as such:
- `acquire` always returns (no real blocking/wakeup)
- `try_acquire` raises `WouldBlockError` unconditionally
- `Notify` methods are pass-throughs

Matches scope definition.

---

### 5. E2E fixtures ✅ CORRECT

- `semaphore_basic.sifr` exercises `await semaphore.acquire()` — the method-call await path
- `notify_basic.sifr` exercises sync + async methods
- Both added to `quick_e2e_manifest.json` at positions 30–31

---

### 6. Phase docs ✅ CORRECT

`32_async_ecosystem.md` line 682 documents the slice with explicit deferral note matching the design doc's scope boundaries.

---

## Verdict

**SATISFIED**

The slice is correct: async method signature wrapping is consistent between HIR and stdlib bootstrap, structured await lowering preserves all existing task/await semantics and correctly handles method-call await expressions, surface types match the documented scope, and all tests pass.
