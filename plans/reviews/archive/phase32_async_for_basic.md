# Review: Phase 32 Async For Basic Slice

## Review Scope

- Branch: `phase32-async-for-basic`
- Phase doc: `internal_docs/phases/32_async_ecosystem.md`
- Design doc: `internal_docs/async_concurrency_model.md`
- Follows merged PRs #2028 (user-defined `async with` normal-exit) and #2030 (named async context state).
- This slice: first `async for` protocol slice — normal exhaustion, ordinary error propagation for user-defined async iterators. AsyncClosable early-exit cleanup, channel-backed async iteration, cancellation cleanup, and async generators are follow-up slices.

## What's Been Added

### HIR
- `HirStmt::AsyncFor { target, target_ty, iter, iter_error_ty, body, else_body }` added to `hir_nodes.rs`.
- New module `crates/sifr_hir/src/lower/async_for.rs` that lowers `StmtFor { is_async: true }` only inside async functions.
- Structural protocol check for `anext() -> Result[Option[T], E]` on `AsyncIterator[T, E]` / `AsyncGenerator[T, E]` types and compatible classes with a no-arg async method.
- Enforces that the enclosing async function return type carries the iterator error type (`return_type_accepts_error`).
- Rejects non-simple targets (not a bare `Name`) and `async for ... else` in this v1 slice.
- `else_body` is always `None` — the `else` clause is rejected at lowering time, not after construction.

### HIR Wiring
The new node is plumbed through every existing walker that handles `HirStmt::For`:
- **CFG** (`cfg.rs`): pattern-matched alongside `HirStmt::For` in `build_stmt` and `stmt_label`; produces the same loop CFG shape with `break`/`continue` targets.
- **Function flow** (`function_flow.rs`): recursively walks `body` and `else_body`.
- **Nonlocal/function-call detection** (`nonlocal_support.rs`): recursively walks `iter`, `body`, and `else_body`.
- **Numeric/container patches** (`numeric_sentinels.rs`, `container_literal_specialization.rs`): recursively applies patches to `body` and `else_body`.
- **Field-assignment detection** (`classes.rs`): checks `body` for field assignments.
- **Async-with body scanners** (`async_with.rs`): three independent scanners (`stmt_contains_await`, `stmt_contains_task_spawn`, `stmt_contains_scope_early_exit`) all handle `AsyncFor` alongside `For` and `While`.
- **For-loop safety** (`for_loop_safety.rs`): recurses for loop body safety checking.

### Codegen
- **Statement shape validation** (`lower_stmt.rs`): `AsyncFor` included in both `is_simple_stmt_candidate` and `validate_stmt_lowering_shape` alongside `For`.
- **Error refs** (`error_refs.rs`): `collect_stmt_error_refs` handles `iter`, `iter_error_ty`, `body`, and `else_body`.
- **Traversal** (`hir_analysis/traversal.rs`): walks `iter`, `body`, and `else_body`.
- **Local-def/mutation queries** (`hir_analysis/queries.rs`):
  - `collect_mutated_vars`: marks `iter` name as mutated (enabling `anext(&mut self)` on local bindings).
  - `collect_locally_defined_vars`: registers `target` as defined.
- **Bigint helper** (`helpers.rs`): checks `target_ty` alongside `For`.
- **Strict/production lowering** (`lib.rs`): dispatches `AsyncFor` to `try_lower_async_for_stmt_for_ir`.
- **Statement support emitter** (`stmt_support_emitter.rs`): `try_lower_async_for_stmt_for_ir` produces:
  - For named iterators: direct `loop { let __sifr_async_next = stream.anext().await?; match __sifr_async_next { Some(target) => body, None => break } }`
  - For non-name iterators: materializes once into `__sifr_async_iter` (mutable let) then advances inside the loop.

### Tests
- `crates/sifr/tests/e2e/pass/async_for_stream_result.sifr`: user-defined `CounterStream` with `anext() -> Result[Option[int], StreamError]`, `async for value in stream`, error propagates through enclosing `main() -> Result[None, StreamError]`.
- `crates/sifr/tests/e2e/fail/async_for_non_async_iterable_rejected.sifr`: `async for value in values` where `values: list[int]` — rejected with `SIFR-FLOW-0008` diagnostic.
- Pass fixture added to `verification/validation_lanes/quick_e2e_manifest.json`.

## Review Findings

### 1. `HirStmt::AsyncFor` is the right narrow shape for this slice ✅

The dedicated node matches the precedent set by `HirStmt::AsyncWith` vs `HirStmt::With`. It isolates the async-specific semantics (the `iter_error_ty` channel, mutation tracking for `anext(&mut self)`) from the synchronous `For`. The `else_body` field is kept in the type for forward compatibility but is always `None` in this slice — the lowering explicitly rejects the `else` clause. This is a sound design: the field can be populated in a future slice without a type change.

### 2. Structural protocol check is sound ✅

The `async_iterator_parts` function checks four cases:
1. `AsyncIterator[T, E]` / `AsyncGenerator[T, E]` types — derecognized directly.
2. `Class` with methods — looks up `"anext"` on the method table, verifies no params, resolves the `Coroutine` return type, and extracts `Option[T]` from the `Result`.
3. `Protocol` with methods — same as Class.

The `async_result_parts` helper correctly unpacks `Type::Coroutine(ok_ty, err_ty)` to get the error channel. The `option_value_type` helper correctly extracts `T` from `Option[T]` by checking for `Type::None` presence and uniqueness. This is consistent with how other structural protocol checks are done in the codebase.

### 3. Error type propagation enforcement is correct ✅

`return_type_accepts_error` checks that the enclosing async function's `Result` error type can accept the iterator's error type via `is_assignable_to`. This ensures `Err(E)` from `anext()` propagates through ordinary Sifr error handling without silent loss.

### 4. All HIR walkers are updated ✅

Every walker that pattern-matches on `HirStmt::For` also handles `HirStmt::AsyncFor`. The pattern is consistent: iterate over `body` and `else_body`, and for expression-bearing walkers, also over `iter`. There are no missing arms or forgotten recursions.

The async-with body scanners (`stmt_contains_await`, `stmt_contains_task_spawn`, `stmt_contains_scope_early_exit`) all treat `AsyncFor` identically to `For` and `While` — which is correct since `async for` contains await points internally even if the loop construct itself doesn't have a user-written `await`.

### 5. Codegen is semantically correct for named and non-name iterators ✅

**Named iterator** (`HirExpr::Name`):
```rust
loop {
    let __sifr_async_next = stream.anext().await?;
    match __sifr_async_next {
        Some(value) => { body },
        None => break,
    }
}
```
The iterator name is used directly. The iterator binding is marked as mutated in `collect_mutated_vars`, so `anext(&mut self)` works on local bindings like `stream` in the test.

**Non-named iterator**:
```rust
let mut __sifr_async_iter = <materialized iter expr>;
loop {
    let __sifr_async_next = __sifr_async_iter.anext().await?;
    match __sifr_async_next {
        Some(value) => { body },
        None => break,
    }
}
```
The expression is materialized once into a mutable local (`__sifr_async_iter`), then advanced in the loop. This avoids re-evaluation on each iteration. Correct.

The `try_lower_async_for_stmt_for_ir` function produces this lowering correctly. The `?` operator on the `Try` wraps propagates `Err(E)` as ordinary error flow.

### 6. Generated Rust output is clean ✅

The emitted code for `async_for_stream_result.sifr` shows:
- `let mut stream: CounterStream = CounterStream::new(...)` — mutable binding, correct.
- `loop { let __sifr_async_next = stream.anext().await?; match __sifr_async_next { Some(value) => { ... }, None => break } }` — exactly the expected shape.
- No extra `.clone()`, no repeated evaluation of the iterator expression.

### 7. Fixture coverage matches the limited scope ✅

The pass fixture exercises:
- User-defined async iterator with `anext()` returning `Result[Option[T], E]`.
- `async for` over a named local variable.
- Error type propagation (`StreamError`) through the enclosing function's `Result[None, StreamError]`.
- Correct exhaustion behavior (exits loop when `anext()` returns `Ok(None)`).

The fail fixture correctly rejects non-async iterables (`list[int]`) with the expected diagnostic.

**Scope gaps correctly documented as follow-up** (from the PR context and phase doc):
- AsyncClosable early-exit cleanup (`aclose()` on `break`/`return`/cancellation) is NOT in this slice.
- Channel-backed async iteration (`ChannelReceiver` as `AsyncIterator`) is NOT in this slice.
- Cancellation cleanup is NOT in this slice.
- Async generators are NOT in this slice.

### 8. Minor observations

**Observer 1: `iter_error_ty` field on `HirStmt::AsyncFor`** — The error type is stored redundantly (it's also in the iterator's type). The reason it exists is for codegen error-ref collection (`error_refs.rs` passes it to `collect_type_error_refs`). This is consistent with other HIR patterns that store type facts for codegen convenience. Acceptable.

**Observer 2: `else_body` always `None`** — The field is present but never populated in this slice. This is forward-compatible: a future slice can populate `else_body` without changing the type. The rejection at lowering time (`"async for else clauses are not supported in v1"`) is correct.

**Observer 3: `simple_for_target_name` restriction** — The slice correctly rejects non-simple targets. This matches the stated design intent for this narrow v1. The diagnostic message says "must be a simple name" which is accurate.

**Observer 4: Local binding types in codegen** — `try_lower_async_for_stmt_for_ir` does not use `local_binding_types` from the emitter context. The function relies on `lower_stmt_expr_for_ir` to handle expression lowering. For this v1 with simple name targets and expressions, this is sufficient.

**Observer 5: Validation run** — `scripts/run_all_tests.sh --profile quick` passes with all 54 pass fixtures. The pass fixture `async_for_stream_result.sifr` is in the quick lane.

## Summary

The implementation is sound:

- **Correct HIR node shape** with appropriate fields and forward-compatible `else_body` slot.
- **Correct structural protocol checking** for `anext() -> Result[Option[T], E]`.
- **Correct error type propagation enforcement** through the enclosing async function's return type.
- **All walkers updated** — no missed references, error refs, mutation tracking, or flow analysis regressions.
- **Correct codegen lowering** for both named and non-named iterators, avoiding repeated evaluation.
- **Fixture coverage matches the limited scope** — no AsyncClosable/channel/cancellation behavior claimed prematurely.
- **Validation passes** — quick profile, pass and fail fixtures, generated Rust output verified.

The implementation correctly follows the model contract in `async_concurrency_model.md` and the milestone_async_7a scope defined in `32_async_ecosystem.md`.

## Recommendation

**SATISFIED** — the slice is ready for merge.

REVIEW_STATUS: SATISFIED
