

Based on my thorough review of the staged diff and implementation, I found one **critical type mismatch bug** that requires fixing:

## Critical Issue: `__sifr_task_gather` Return Type Mismatch

**File**: `crates/sifr_codegen/src/preamble.rs:930-931`

The codegen function `__sifr_task_gather` declares:
```rust
ret: Some(RustType::Named("__SifrTaskResult<Vec<T>, E>".to_string())),
```

But the body emits `__SifrTaskResult::Err(err)` where `err` is already `__SifrFailure<E>` (from the spawn helper wrapping). This creates a type mismatch: the return type says `Err(E)` but the code returns `Err(__SifrFailure<E>)`.

The same issue exists for `__sifr_task_race` at line 955.

**Fix required:**
- `__sifr_task_gather`: `__SifrTaskResult<Vec<T>, E>` → `__SifrTaskResult<Vec<T>, __SifrFailure<E>>`
- `__sifr_task_race`: `__SifrTaskResult<T, E>` → `__SifrTaskResult<T, __SifrFailure<E>>`

**Additionally:** The HIR lowering in `crates/sifr_hir/src/lower/task_calls.rs:104-106` should probably return `Type::TaskResult(List(result_ok_ty), Failure(result_err_ty))` to reflect the wrapped error type, but since this is a codegen-only slice, the type system/HIR layer may be out of scope.

## Recommendations

1. **Fix the return types** in `preamble.rs` for `__sifr_task_gather` and `__sifr_task_race`

2. **Add test coverage** for `task.gather` with fallible tasks to catch this case:
   ```rust
   // Should verify __SifrTaskResult<Vec<T>, __SifrFailure<E>> (not __SifrTaskResult<Vec<T>, E>)
   ```

## Everything Else: SATISFIED

The rest of the implementation is correct:
- `__SifrFailure<E>` struct with `new()` and `map_primary()` helpers ✓
- `__SifrTaskResult::Err(__SifrFailure<E>)` variant ✓
- Spawn helpers wrapping with `__SifrFailure::new(err)` ✓
- Timeout handling preserving secondary evidence via `map_primary` ✓
- `SecondaryError` forced-inserted when task scope is used ✓
- Test assertions for the new codegen output ✓
