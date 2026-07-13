## Confirmation Review

### Coercion fix verification

**Enter path** (`async_context.rs:128-131`):
```
Err(error) => {
    sifr_runtime::python::poison_object({manager_name}.__sifr_python_object);
    return Err(({enter_error}).into());
}
```
`{enter_error}` is `bridge_error_expr(Ident("error"), enter_error_type)`. For the real `PythonError` class (with all 5 fields), it renders a Sifr `PythonError` struct literal using field borrows (`error.message.to_string()`, …) then moves `error` last into `Some(error)`. `poison_object` consumes `manager.__sifr_python_object` (partial move) once, before the return. All ownership transitions are exact-once and NLL-compliant. `.into()` targets `active_error_type`, degenerating to identity when it equals `PythonError` (same pattern the sync `?` path already relies on).

**Conversion path** (`async_context.rs:134-140`):
```
Err(mut {conversion_error}) => {
    {conversion_exit}       // format!("{}", conversion_error) — borrow only
    return Err({conversion_error}.into());
}
```
`sifr_error_exit(..., return_primary=false)` emits `submit_async_context_exit({manager}.__sifr_python_object, …)` (mandatory `__aexit__` after successful `__aenter__`) and only borrows `conversion_error` via `format!`; the final `.into()` moves it. Manager object is consumed exactly once by the exit submission, so the mandatory-cleanup contract is preserved.

### No regression on resolved findings

| Finding | Location in post-fix code | Status |
|---------|--------------------------|--------|
| #1 yield-after-abort | `async_context.rs:184-193` | Untouched — `yield_now().await` on `Invoked`/`AlreadyResumed` still present |
| #2 direct enter await under child scope | `async_context.rs:119-132` | Untouched — no biased enter race remains |
| #3 Suppress recording in None arm | `async_context.rs:172-183` | Untouched — `record_context_ignored_suppression("cancellation:CancellationError")` on `Suppress`, empty on `Propagate`, `record_context_cleanup_evidence` on `Err` |

The two edits are confined to error-return sites in the enter/conversion branches; they touch none of the cancellation-cleanup match, the biased select, the terminal-consumption, or the `release_and_resume_parent` sequence.

### Test coverage

`async_python_context_converts_enter_failures_to_the_active_error_type` (`python_async_context_tests.rs:152-166`) constructs an `active_error_type = Error` HIR and asserts both `return Err((error).into());` and `conversion_error_0.into())` render. The test HIR uses empty-field class types so `bridge_error_expr` falls through to identity, producing bare `(error).into()` — consistent with the assertions. The existing four codegen tests continue to lock in yield_now, biased select, Suppress recording, evidence, `PythonAsyncExitCause::Python(replay.clone())`, all body outcomes, and syn validity.

### Wave 2 activation risk (pre-existing, not introduced here)

For `active_error_type` other than `PythonError` (e.g., `Error` supertype), `.into()` needs `impl From<PythonError> for Error`, which Sifr's current codegen does not generate (PythonError is not in `BUILTIN_ERROR_CLASSES` at `builtin_errors.rs:2`, so `build_error_into_error_impl` skips it, and no class-driven emitter builds it either). This gap equally affects the sync path's `mapped_try` (`?`) whenever active supertype differs from enter error type — it's not created by this fix. Current sync fixtures (`sqlite_context/context_codegen_smoke.sifr`) use `Result[_, PythonError]` exclusively, so it's not exercised. This risk was already flagged in Round 2 as Wave 2 planning; the `.into()` rendering is the codegen-side prerequisite so Wave 2 can either standardize on `Result[_, PythonError]` or add the missing From-impl generation without a further codegen change. It is not a Wave 1 blocker (SIFR-PYRES-0002 still gates all real compilation).

### Diff scope check

The rest of the Wave 1 working tree diff (HIR variant, lowering split, runtime `async_context.rs`, ambient main carrier, request/terminal wiring) is unchanged from Round 2's satisfied state; file sizes remain under 900 (`async_context.rs` at 395 lines, `python_interop.rs` 876). No new files were touched by this coercion fix beyond `async_context.rs` (2 lines) and the new test.

VERDICT: SATISFIED
