I've completed the review. Here are the findings and verdict.

## Scope check
- 12 crate files + 5 review artifacts. All changes are scoped to Wave 6: sealed-identity lifecycle (`foreign_object.rs`), semantic close transport (`async_value.rs`, `async_declaration.rs`), codegen selection (`class_method_emitter.rs`, `python_interop_direct.rs`, `python_interop_async/conversions.rs` + module split), affine hook for `Self.aclose` (`python_interop.rs`), and matching tests. No unrelated churn; no gates lifted.

## Design conformance (against the two design reviews)
1. **Atomic Open→Closing.** `foreign_object.rs:98-109` flips status and bumps `active_leases` under the same mutex that guards `lease()`/`clone_ref()`; aliases correctly see `closing`/`closed`/`poisoned`.
2. **Terminal keyed by Python outcome only.** `async_declaration.rs:130` finalizes via `outcome.is_ok()` computed from `convert_output` — suppressed cancellation returning clean `None` → Closed; CancelledError, non-None, non-awaitable, panic, conversion error → Poisoned. Aligns with `python_interop_protocol_architecture.md:104-106`.
3. **Pre-registration self-guarding request.** `PythonAsyncRequest::Drop` (`async_value.rs:163-167`) plus explicit `request.finish_semantic_close(false)` at every submission-error map_err (`async_declaration.rs:68, 83, 91, 240`) ensures Closing→Poisoned on every synchronous failure path including `ensure_started`, `validate_keywords`, `reserve_submission` under Stopping, `call_soon_threadsafe`, callback build errors, and setup-callback catch_unwind. Ordering is: state transition first, then `finish_submission` (bookkeeping errors don't override).
4. **Emitter guard (six-predicate).** `conversions.rs:74-88` requires class cleanup==AsyncClose, decorator==Coroutine, target==`["Self","aclose"]`, `consumes_receiver`, empty params, `Result[None, PythonError]`. Extra guard `if owns_async_close && consumes_receiver && !semantic_close { return None; }` rejects any consuming non-aclose shape (HIR already blocks this at `class_body_lowering.rs:614-674`, so codegen doesn't paper over).
5. **Single shutdown authority.** Semantic close rides the ordinary typed submission registry — `submit_typed` uses the same `reserve_submission`/`register_submission`/`finish_submission` path. `run_registered_async_cleanup` remains a no-op stub. `semantic_async_close_shutdown_and_submission_rejection_poison_safely` verifies both terminal-drain-with-finally-marker and Stopping-lifecycle rejection.
6. **Runtime owner-drop invariant.** `ForeignObjectInner::Drop` (`foreign_object.rs:169-179`) releases the object safely on the last Arc regardless of status (Open/Poisoned included). Affine tracking prevents Sifr code from getting to that path; the runtime is a safety containment for compiler bugs.

## Affine / gate contracts
- `method_consumes_receiver` (`python_interop.rs:696-719`) recognises `@python.coroutine(Self.aclose)` with `own self` as consuming. `mod_context.rs:243` maps `AsyncClose` → `CloseLike`, so `await client.aclose()` discharges the obligation via `python_consuming_methods` receiver-move.
- Both public gates stay closed: `PYRES_UNIMPLEMENTED_DECLARATION` fires for `@python.coroutine` (`python_interop.rs:60-63`) and `cleanup=async_close` (`python_interop.rs:364-367`). Contract tests (`python_coroutine_contract_tests.rs:91-181`) confirm consumption discharges, abandonment/partial/duplicate raise `OWN_USE_AFTER_MOVE`, and shape-negatives raise `PYCALL_INVALID_SHAPE`.

## Validation claims
- `scripts/check_file_size_guardrails.py`: PASS (2539 files, cap 900). `python_interop_direct.rs`=900, `conversions.rs`=853, `class_method_emitter.rs`=758, `python_interop.rs`=858 — all within.
- `scripts/check_hir_maintainability_guardrails.py`: PASS.
- User's stated create-pr profile + 130/130 E2E green matches PR description; the runtime and codegen test additions cover the enumerated matrices (exact-once, Closing/Closed/Poisoned reuse, dropped-request poison, Python/non-None/non-awaitable, suppressed vs. observed cancellation with finally marker, shutdown drain + Stopping rejection, codegen selection guard including the non-close borrowed-method fall-through).

## Non-blocking observations
- `plans/reviews/active/python-interop-m7-wave6-async-close-implementation-review-round{2,3}.md` are one-line verdict files. Fine for follow-up rounds; content lives in round 1.
- The strict guard's "return None" for a consuming non-aclose coroutine on an `async_close` class (`conversions.rs:86`) is harmless today (HIR already rejects) but should be revisited when Wave 7 adds the httpx client fixture. Already flagged in round-1 impl review; no action needed for Wave 6.

VERDICT: SATISFIED
