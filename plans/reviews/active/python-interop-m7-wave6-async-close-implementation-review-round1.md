Review complete. Here is my assessment:

## Summary of what I verified

**Runtime (`foreign_object.rs`, `async_value.rs`, `async_declaration.rs`)**
- `begin_semantic_close` atomically flips Open→Closing under the same mutex that guards `lease()`/`clone_ref()` and bumps `active_leases` — no TOCTOU. `clone_ref`/`lease`/subsequent `begin_semantic_close` all reject non-Open, so aliases correctly see `closing`/`closed`/`poisoned` per state.
- `finish_semantic_close` is idempotent: guarded by `status != Closing`, transitions once, releases the payload only when it's safe to do so, and delegates decref to `release_object` outside the mutex.
- `PythonAsyncRequest` owns the receiver; `Drop` calls `finish_semantic_close(false)`, poisoning any request that leaves the machinery mid-lifecycle. `submit_typed` wraps in `Arc` and explicitly poisons on every failure branch (`build_done_callback`, `build_setup_callback`, `call_soon_threadsafe`); the setup-callback closure poisons before terminal publication when its own path errors; the done callback keys the transition on `outcome.is_ok()` before `finish_submission` — so `finish_submission` bookkeeping errors don't override the Python-outcome-driven state (clean None → Closed, CancelledError/other → Poisoned).
- No new `unsafe`, raw pointer handoff, or identity resurrection. `catch_unwind` guards both callbacks against panics.
- Shutdown uses the ordinary typed submission registry (`cancel_registered_submissions` + `wait_for_submissions_to_drain`); `run_registered_async_cleanup` remains a no-op — single terminal authority.

**Codegen (`python_interop_async/conversions.rs`, `class_method_emitter.rs`, `python_interop_direct.rs`)**
- The class's opaque declaration is threaded through `python_interop_method_body(..., class.python_opaque_declaration())` into `async_python_method_body`, which selects `semantic_close_method` only when all six predicates hold (class cleanup==AsyncClose, kind==Coroutine, target=="Self.aclose", `consumes_receiver`, empty params, `Result[None, PythonError]`). Otherwise defers to the Wave-5 owned/borrowed constructor. Sync `close` is untouched.
- The `python_interop_async.rs` monolith was cleanly split into a coordinator + `python_interop_async/conversions.rs` (853 lines). `class_method_emitter.rs` (758) and `python_interop_direct.rs` (900) remain within the 900-line cap; file-size and HIR guardrails pass.

**Lowering (`python_interop.rs`, `python_coroutine_contract_tests.rs`)**
- `method_consumes_receiver` recognises `@python.coroutine(Self.aclose)` with `own self` as consuming; because `python_consuming_methods` marks the method-call receiver moved via `mark_moved_with_flow`, awaiting `client.aclose()` discharges the `CloseLike` obligation already mapped from `PythonCleanupPolicy::AsyncClose` in `mod_context.rs`.
- Contract tests exercise discharge, abandonment (unused binding), partial-branch consumption, and duplicate close — each producing `OWN_USE_AFTER_MOVE`. Coroutine and `cleanup=async_close` gates remain closed via `PYRES_UNIMPLEMENTED_DECLARATION`.

**Tests**
- Exact-once, Closing/Closed/Poisoned reuse rejection, dropped-request poison, Python/non-None/non-awaitable failures, suppressed vs. observed cancellation with `finally` marker verification, shutdown-drain ordering plus `Stopping`-lifecycle submission rejection, and the codegen selection guard are all covered by `async_declaration_tests.rs` and `python_interop_async_tests.rs`.

## Findings

None that rise to blocking. Design conformance, lifecycle atomicity, cancellation semantics, single-authority shutdown, gate closure, and the file/guardrail budget are all satisfied. One minor observation not blocking Wave 6 scope: the codegen predicate `if owns_async_close && !semantic_close { return None; }` in `crates/sifr_codegen/src/python_interop_async/conversions.rs:86-88` also returns `None` for a *non-close* borrowed coroutine method on an `async_close` class (e.g., an `httpx`-style `get`/`post`). Wave 6 tests never construct such a class, the coroutine gate remains closed, and design review round 2 explicitly accepted the strict interpretation ("or the shape is rejected"), so it doesn't manifest. It's flagged only because when Wave 7 lifts the gate and adds the httpx client fixture, that path will need to fall through to the Wave-5 `borrowed_method` selection rather than return `None`.

VERDICT: SATISFIED
