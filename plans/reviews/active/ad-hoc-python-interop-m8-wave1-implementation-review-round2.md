I've completed the review. Let me finalize the analysis and write up the findings.

## M8 Wave 1 Re-Review After Corrective Wave

### Round 1 blocking findings — all verified resolved

**Finding #1 (cleanup-resume race) — FIXED at `crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:184-194`.** The cancellation-cleanup arm now pattern-matches `release_and_resume_parent()` and awaits `tokio::task::yield_now().await` on `Invoked | AlreadyResumed` before the bounded `return Err({internal_error})`. This gives Tokio a poll boundary to observe the parent's `AbortHandle::abort()` and drop the future, so the enclosing task surfaces as `JoinError::Cancelled` rather than as `SifrPythonAsyncContextError`. The shape now mirrors M7's `async_cancellation.rs::propagate` (`yield_once` → error). Codegen test locks in `tokio::task::yield_now().await` and the `Invoked`/`AlreadyResumed` cases.

**Finding #2 (biased enter race dropping entered value) — FIXED at `async_context.rs:119-132`.** The enter is now awaited directly under `__SIFR_TASK_CANCELLATION.scope({child}.clone(), submit_async_context_enter(...))`. There is no biased race with the sticky notification, so a successful `__aenter__` value always reaches the body-cancellation select and the mandatory `__aexit__`. A cancellation during enter is detected by M7 (`propagate`) and surfaces as `Err(propagation_error)`, which we route through the manager-poison branch. The enter-race identifiers are gone (test asserts `!rendered.contains("enter_cancel")`), and the runtime test `child_carrier_waits_for_python_finally_before_cancellation_race_wins` continues to prove the child carrier waits for Python `finally` before the sticky arm can win.

**Finding #3 (missing ignored-Suppress record on cancellation cleanup) — FIXED at `async_context.rs:172-183`.** The cancellation cleanup now matches all three cases: `Ok(Suppress) → record_context_ignored_suppression("cancellation:CancellationError")`, `Ok(Propagate) → {}` (symmetric with `sifr_error_exit`), and `Err(cleanup_error) → record_context_cleanup_evidence(...)`. Codegen test locks in the exact `record_context_ignored_suppression(\n"cancellation:CancellationError"` string.

### Round 1 optional findings — persist consistently

- **Finding #4** (enter-failure path doesn't resume parent's abort when cancellation caused the propagation error): the `Err(error) => { poison; return Err({enter_error}); }` branch at lines 128-131 still doesn't inspect the runtime kind or call `release_and_resume_parent`. This was optional in Round 1 and is framed as "an optional related enter-cancellation misclassification" in the corrective summary; not claimed fixed.
- **Finding #5** (codegen tests are syn-only, not compile-checked) still holds. Optional.
- **Finding #6** (body-error paths don't resume the parent's abort) still holds. Optional.

### New considerations (non-blocking, Wave 2 activation risk)

The enter-error return at `async_context.rs:128-131` renders as `return Err(PythonError { ... })` (bridge_error_expr against `enter_error_type = PythonError`), and the conversion-error return at 136-139 does the same with `{conversion_error}` whose Rust type is the enter-error's PythonError. The enclosing async function returns `Result<_, active_error_type>`. Sifr's lowering only requires that `enter_error_type.is_assignable_to(active_error_type)`; when `active_error_type` is a supertype (e.g., `Error`) rather than `PythonError` itself, Rust's `return Err(x)` does not invoke `From::from`, so the rendered code would fail rustc even though M5's sync path (which uses `mapped_try`/`?` and therefore gets the automatic `From::from`) does not. This is masked in Wave 1 because SIFR-PYRES-0002 gates the syntax and the syn-only codegen test uses PythonError as active error. Wave 2 fixtures should either standardize on `Result[_, PythonError]` or the codegen should be updated to render `.into()`/`.map_err(...)?` on the enter/conversion paths. Optional; not required by the design; flagged for Wave 2 planning.

### Everything else

- Ambient root carrier is installed at generated async `main` when `uses_async_python` is true (`lib_async_main_cancellation.rs`, wired in `lib_modules_and_codegen.rs:592-594`), and Sifr-spawned tasks continue to install their own carriers via `__SifrCancellationCarrier::new` with abort-as-fallback (`preamble/task_cancellation_runtime.rs`).
- Terminal completion in `async_terminal.rs:65-79` still drops the exact claim before waking the waiter, so `propagate`'s `resume_fallback_after_claim` reliably returns `Invoked` rather than `ExactClaimActive`.
- `submit_async_context_enter` (borrowed method) and `submit_async_context_exit` (semantic-close via `PythonAsyncRequest::semantic_context_exit_method`) route through the same M7 setup/terminal engine, and `PythonTerminalValue::ExitDecision` remains the only accepted terminal for the exit path (`async_declaration.rs:75-97`).
- `register_boundary_error` runs before the owned loop starts; `record_context_ignored_suppression`, `attach_secondary_python_error`, and `record_context_cleanup_evidence` remain the shared evidence surfaces used by both sync and async exit renderers.
- File sizes all under 900 (largest touched: `python_interop.rs` 876, `python_context/sync.rs` 675, `python_context/async_context.rs` 394); `git diff --check` and `cargo fmt --check` clean; module split preserves the previous public API (verified against `git show HEAD:...context.rs`).

The declaration contract behind SIFR-PYRES-0002 (real-source diagnostics ahead of the reservation, exact-once obligation, non-substitutable sync/async, non-directly-callable exit) is intact.

VERDICT: SATISFIED
