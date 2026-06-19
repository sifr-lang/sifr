# Review Round 2 — milestone_py_6 embedded Python interop

## Blocker status: resolved

The round 1 blocker (body-failure path never invoking `__exit__` with failure context and leaking the entered handle) is fixed in `lib/sifr/python.sifr:250-293`:

- `_object_ref` (line 116) gives the body an aliased wrapper, keeping the original `entered` available in the helper after the body returns or raises.
- The body call is now inside the `try`, with `body_completed` set only on the success line *after* the call. The `except` distinguishes body failure from `exit_context(obj)` failure by checking `body_completed`, captures `PythonError` fields into locals (since `e` cannot survive the `except` scope), and re-raises the original error.
- `finally` runs unconditionally: if `body_failed`, it reconstructs the body error and calls `_exit_context_with_error(obj, body_error)`; then it `close(entered)` regardless of which branch ran. Both cleanup calls are wrapped in `try/except` so a secondary error during cleanup cannot mask the original failure.

Walk through:
- **Body succeeds, exit_context succeeds**: returns None; finally closes `entered`. ✔
- **Body raises**: except sets `body_failed=True`, re-raises body's PythonError; finally invokes `__exit__` with the captured (kind/exception_type/message/traceback/context) and then closes `entered`; body error propagates. ✔
- **Body succeeds, `exit_context` raises**: `body_completed=True` so `body_failed` stays False; except re-raises the exit error; finally just closes `entered` (no second `__exit__` attempt — correct, since `__exit__` was already attempted and failed). ✔

The runtime test `exit_context_with_error_passes_sifr_error_context_to_python_exit` (`crates/sifr_runtime/src/python/resource_ops.rs:107-158`) now uses `RecordingContext` and asserts `exc_type` + `exc_value` are non-None and `tb` is None — this would catch a regression that always passed `(None, None, None)`, addressing round 1 finding 2.

The JSON fixture `context_manager_cleanup.json:53-62` adds the body-failure negative case with `expected_exit_args.exc_type: "non_null"`, `exc_value: "Sifr PythonError wrapper"`, `traceback: "none_sifr_traceback_embedded_in_exception_message"`, and `expected_entered_handle_release: "closed_after_exit_with_error"`. The `tb=None` adaptation is now explicit in the contract, addressing round 1 finding 3.

`context_manager_body_failure.sifr` exercises the new path through the lowering surface; runner registration in `verification/python_interop/runner/run.py:62-72` enforces its presence.

Phase plan updates in `plans/issues/active/ad-hoc-embedded-python-interop.md:331,351,425,436` switch to `py.with_context` consistently and document that exact `py.with(...)` is parser-blocked, addressing round 1 finding 4.

The stdlib type registration (`crates/sifr_stdlib/src/python.rs:106-120`), intrinsic lowering test (`crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:110-125`), and codegen lowering (`crates/sifr_codegen/src/intrinsics/registry/python.rs:256-277`) all line up on the same 7-argument shape (handle, token, kind, exception_type, message, traceback, context).

## Non-blocking observations

These are not milestone blockers; carrying them forward is fine.

1. **Cleanup errors are silently swallowed** in `_with_entered_context`'s `finally` (`lib/sifr/python.sifr:288-293`) — both the `_exit_context_with_error` failure and the `close(entered)` failure bind `cleanup_context` and never read it. That's intentional (preserve the original body error), but it means a programmer-visible double-close or runtime-state error during cleanup goes unobserved. Worth a diagnostic-counter increment in a follow-up so the existing `SIFR-PYRES` counters can surface it.

2. **`body_entered` aliases `entered` via the same `(handle, token)` tuple**. If a body ever calls `close(body_entered)`, the subsequent `close(entered)` in the helper's `finally` would trigger the deterministic double-close error and mask the body's own outcome. The contract "body must not close its parameter" is not documented in the JSON fixture or the phase plan. Worth an explicit rule under "Resource Cleanup" in the issue plan before py12 docs ship.

3. **Unused `result` binding** in `with_context` (`lib/sifr/python.sifr:244`) — round 1 finding 5, still present. Cosmetic.

4. **`context_manager_failure.sifr` and `context_manager_body_failure.sifr` both leak handles** (`contextlib`, `value`, `manager`) on the early-return-via-raise path. Consistent with the "callers own cleanup on error paths" contract, but together they leave the verification story without an end-to-end example of the *full* clean unwind where every handle is closed under failure. Once the runner can execute fixtures, a third fixture that catches the propagated error in `main` and explicitly closes the source/manager handles would complete the picture.

## Conclusion

reviewer satisfied: no blockers
