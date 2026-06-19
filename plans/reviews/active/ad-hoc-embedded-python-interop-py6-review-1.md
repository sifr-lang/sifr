I've reviewed the milestone py6 implementation. Here are my findings, ordered by severity.

---

## Blocker

### 1. `with_context` never invokes `__exit__` with failure context when the body fails — entered handle is leaked

**File:** `lib/sifr/python.sifr:233-257`

The phase contract (plans/issues/active/ad-hoc-embedded-python-interop.md:436) requires:

> Python `__exit__(exc_type, exc, tb)` receives Sifr/Python failure context before the final `Result` is produced

and (line 434):

> Do not rely on Python `__del__` for correctness-critical resource cleanup.

The supporting infrastructure is fully plumbed: `exit_context_with_error` in `crates/sifr_runtime/src/python/resource_ops.rs:30-53`, the `py_exit_context_with_error` intrinsic in `crates/sifr_codegen/src/intrinsics/registry/python.rs:106-120`, the lowering in `.../python.rs:256-277`, and the private Sifr helper `_exit_context_with_error` at `lib/sifr/python.sifr:221-230`. None of that is ever called.

`_with_entered_context` (lib/sifr/python.sifr:246-257) places `body_result: None = body(entered)` *outside* the try/except:

```python
def _with_entered_context(obj, entered, body) -> Result[None, PythonError]:
    body_result: None = body(entered)        # outside try
    try:
        exited: None = exit_context(obj)
        closed: None = close(entered)
        return None
    except PythonError as e:
        raise e
```

When `body(entered)` raises a `PythonError`:
- `exit_context(obj)` is never called
- `close(entered)` is never called
- `_exit_context_with_error(obj, e)` is never called
- the error propagates with `entered` leaked

The existing `context_manager_failure.sifr` fixture only exercises `enter_context` failing on a non-context-manager (an `int`). It does not cover the body-failure path that the contract is specifically about. So the milestone DoD ("Fixtures cover ... context manager success/failure") is not met for the failure half.

This is a milestone-blocking gap: an entire branch of plumbing was implemented (intrinsic + codegen + runtime + stdlib helper) but the user-facing `with_context` never reaches it.

---

## Non-blocking findings

### 2. `exit_context_with_error` runtime test does not actually verify the args reached `__exit__`

**File:** `crates/sifr_runtime/src/python/resource_ops.rs:104-137`

The test uses `contextlib.nullcontext`, whose `__exit__` accepts and ignores all args. So the test confirms that the call returns `Ok(())`, not that `exc_type`/`exc_value`/`context` were actually propagated to Python. Use a custom Python class that records the `__exit__` args and assert they are non-None — otherwise a regression that, e.g., always passes `(None, None, None)` would still go green.

### 3. `exit_context_with_error` always passes `py.None()` for the traceback

**File:** `crates/sifr_runtime/src/python/resource_ops.rs:46`

The Sifr-side traceback string is folded into the PyRuntimeError message, which is reasonable as an adaptation, but Python `__exit__` receives `tb=None` even when Sifr has a non-empty traceback. Worth documenting this adaptation in the JSON fixture (alongside the existing `helper` note) or in an internal doc so consumers know not to expect a real Python `TracebackType`.

### 4. Parser limitation around `py.with(...)` — acceptable milestone adaptation, but the phase plan still uses the old surface

The `with_context` rename and fixture JSON note are reasonable adaptations for the current parser. However, the phase plan at `plans/issues/active/ad-hoc-embedded-python-interop.md:331,351` still lists `try py.with(obj, lambda entered: ...)` as the canonical API. Either update the plan to mirror the implemented `with_context` name, or open a follow-up to add soft-keyword support for `with` after `.` before py12 docs ship. Not blocking py6 since the JSON contract is explicit, but consumers reading the plan will be misled.

### 5. Unused `result` binding in `with_context`

**File:** `lib/sifr/python.sifr:240`

`result: None = _with_entered_context(obj, entered, body)` binds a value that is never used. Harmless; minor cleanup.

### 6. `context_manager_failure.sifr` itself leaks `value` on the early-return path

**File:** `verification/python_interop/fixtures/resource_cleanup/context_manager_failure.sifr:1-16`

Because `with_context(value, observe)` will raise (int has no `__enter__`), the trailing `close(value)` never runs and `value` is leaked. This is consistent with the documented contract that callers own cleanup on error paths, but pairing it with a fixture that demonstrates the *body-throws → __exit__-with-error → entered released* path (once finding 1 is fixed) would close the verification story.

---

## Recommendation

Fix the with_context body-failure path before merging. The shape needed:

- Move `body(entered)` inside a try/except in `_with_entered_context`.
- On body PythonError: call `_exit_context_with_error(obj, e)` (best-effort, may itself error), then close `entered`, then re-raise the original.
- Add a fixture/test that exercises body failure and asserts `live_objects == 0` and `leaked_objects == 0` after the failure unwinds.

Then strengthen the runtime test in finding 2 with a recording context manager so the wiring stays honest under refactoring.

Everything else (the future-resource-types JSON contract, double-close determinism, diagnostics counters for the existing `py.Object` type, the trust/parser adaptations) looks consistent with the milestone scope.

Blockers: 1.
