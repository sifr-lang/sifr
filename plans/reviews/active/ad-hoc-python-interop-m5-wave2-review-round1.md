# M5 Wave 2 Code Review — Runtime Exception Replay, Boundary Errors, Cleanup Evidence

## Scope

Reviewed against `plans/issues/active/ad-hoc-declaration-first-python-interop.md` M5 wave-2 acceptance, and `internal_docs/python_interop_protocol_architecture.md` §"Exit Cause And Decision Types" and §"Synchronous Context Managers". Files inspected:

- New: `crates/sifr_runtime/src/python/python_error.rs`, `.../context_ops.rs`, `.../context_ops_tests.rs`
- Modified (runtime): `python.rs`, `object_ops.rs`, `object_ops_tests.rs`, `foreign_object` interactions, `arrow_ops`/`buffer_ops`/`callback_ops`/`dlpack_ops`/`recursive_ops`/`resource_identity` (all `replay: None` migration only)
- Modified (stdlib/codegen/lowering/driver/package): all use `PythonError::without_replay(...)` migration + small Rust-1.94 Clippy cleanups

## Contract Verification

**1. Replay capability preserves live triple.** `python_error.rs:12` `PythonExceptionReplay::capture` stores `(type, value, traceback)` into a `PyDict`, wrapped in a `ForeignObject`. `from_pyerr` invokes it for every Python-originating error. All non-Python-origin errors (structural, closed-handle, argument, conversion, trust) correctly use `replay: None` / `without_replay(...)`. No path skips replay for a `PyErr`-sourced error.

**2. Clone-sharing and single final release.** `ForeignObject` is `Arc<ForeignObjectInner>`-backed (`foreign_object.rs:12-38`). Cloning `PythonError` → clones `Box<PythonExceptionReplay>` → clones `ForeignObject` → increments `Arc`. Final drop hits `ForeignObjectInner::drop` exactly once, releasing the triple via the off-GIL pending queue in `release_object` (`foreign_object.rs:102-109`). Nested exits borrow via `PythonError::replay` (`python_error.rs:130-142`), which `clone_ref`s new `Py<PyAny>` for each call — original identity preserved.

**3. Privacy of the capability.** `python_error.rs:56` `replay` is `pub(crate)`, `PythonExceptionReplay` is `pub(crate)`, `PythonExceptionReplay::resolve` is `pub(super)`, `PythonError::replay` method is `pub(super)`. External crates (`sifr_stdlib`, `sifr_codegen`) can only construct with `without_replay` (which is `#[doc(hidden)] pub`) and cannot invoke resolution. Struct-literal construction across crates is blocked by the private field. `PartialEq` (`python_error.rs:145-153`) excludes `replay`. ✓

**4. Runtime init registers `SifrBoundaryError`.** `python.rs:222` calls `context_ops::register_boundary_error` via `Python::try_attach` from `initialize_runtime`. `context_ops.rs:51-78` idempotently constructs a `RuntimeError` subclass with `cause_kind`, `sifr_type`, `message` attributes and installs it in `sys.modules['__sifr_context__']`. `OnceLock`-guarded; the `is_err` return from `set` is intentionally swallowed after the same-value guard. ✓

**5. Exit calls translate arguments and truthiness exactly.**
- Normal: `context_ops.rs:124-135` sends `(None, None, None)`; `exit_decision` maps truthy → `Suppress`, else → `Propagate`. Test 2 asserts False→Propagate and True→Suppress.
- Python error: `context_ops.rs:137-162` borrows the replay via `error.replay(py)` and forwards the exact `(type, value, traceback)`. Test 1 asserts `exc_type is Marker and exc_value is ORIGINAL and tb is ORIGINAL.__traceback__` succeeds across two nested exits.
- Sifr cause: `context_ops.rs:164-186` constructs `SifrBoundaryError(kind_label, sifr_type, message)` with `None` traceback (matching doc's "no fabricated Python traceback beyond the adapter frame"). Test 2 verifies all five fields.

**6. Consumed exactly once; failed exit poisons/releases without fallible Drop.** `finish_context_exit` (`context_ops.rs:188-202`) is a total match: `Ok → object.close()`, `Err → object.poison()`. Both are infallible on `ForeignObject`. Manager is passed by value; local drop at function exit hits `ForeignObjectInner::drop`, which routes through `release_object` — off-GIL goes to pending queue with no user-visible panic. Test 3 verifies the manager's `__del__` runs only after the returned error drops and `attach()` drains the queue.

**7. Secondary evidence.** `attach_secondary_python_error` (`context_ops.rs:98-109`) appends `"<Type>: <message>"` to the primary Python error's `context` string. `record_context_cleanup_evidence` (`context_ops.rs:111-118`) pushes to a `LazyLock<Mutex<Vec<ContextCleanupEvidence>>>` sink for non-Python primary causes. Test 4 exercises both. `take_context_cleanup_evidence` uses `mem::take`, and `reset_context_state_for_tests` clears it — no cross-test leakage. Consistent with docs.

**8. GIL/PyO3 safety.**
- All Python-facing operations require `Python<'_>` and go through `attach`.
- `release_object` uses `unsafe { ffi::PyGILState_Check() }` correctly to route off-GIL drops to the pending queue (`foreign_object.rs:102-109`).
- `PythonError: Send + Sync` transitively (through `Arc<Mutex<...>> + Py<PyAny>: Send + Sync`), so the existing `Fn(...) -> Result<_, PythonError> + Send + Sync` bounds in `callback_ops.rs` still hold.
- No `unwrap/expect` on user paths; `test_guard` now recovers a poisoned mutex (`python.rs:531`).

**9. Test coverage matrix (spec item 8).**

| Required aspect | Covered by |
| --- | --- |
| Exact type/value identity | Test 1 (`exc_type is Marker`, `exc_value is ORIGINAL`) |
| Traceback identity | Test 1 (`tb is ORIGINAL.__traceback__`) |
| Nested replay | Test 1 (both exits log `True`) |
| Final release queue | Test 1 (`pending_release_count == 1`, drained by `attach`) |
| Truthiness translation | Test 2 (False→Propagate, True→Suppress) |
| Boundary error fields | Test 2 (`cause_kind`, `sifr_type`, `message`, `tb is None`) |
| Failed exit release | Test 3 (`__del__` fires only after error drop + drain) |
| Secondary evidence (both sinks) | Test 4 (`attach_secondary_python_error` + `record_context_cleanup_evidence`) |

Prior modified test files (`object_ops_tests.rs`, `callback_ops.rs` inline tests) correctly add `drop(error)` sites so `pending_release_count`/`live_objects` assertions remain deterministic after `PyErr`-sourced errors now carry replay refs.

## Minor observations (non-blocking)

- `python_error.rs:56` `pub(crate) replay` is wider than needed. `pub(super)` would suffice for the `replay: None` writes in `object_ops.rs`, `arrow_ops.rs`, `dlpack_ops.rs`, etc. (all in `sifr_runtime::python`). Not a security or correctness issue since resolution is `pub(super)` and `PythonExceptionReplay` is opaque outside its module.
- `PythonExceptionReplay::capture` silently degrades to `None` on `set_item` failure (`python_error.rs:14-19`). The failure would only occur under Python allocation stress or dict corruption; downstream `context_exit_python_error` cleanly returns a `PythonRuntimeError` in that case. Acceptable, but a comment tying this to the doc's fidelity guarantee would help future auditors.
- `attach_secondary_python_error` folds the secondary into the primary's `.context` string. This is functional but unstructured; a later wave may want an explicit `secondary_evidence` field so downstream diagnostics can render it cleanly.
- `context_ops.rs:76` `let _already_registered = BOUNDARY_ERROR_TYPE.set(...).is_err();` — the leading guard on line 52 already protects against reregistration; the `_already_registered` binding is dead. Cosmetic.
- Small Rust-1.94 Clippy cleanups (`method_call_args.rs` iter_mut, `python_interop.rs` `is_none_or`, `python_interop_plan.rs` `write!`, `python_probe.rs` `&io::Error`, `dlpack_ops.rs` nested-or, `rust_interop_direct.rs` `without_replay`) are all semantics-preserving.

## Verdict

All eight required behaviors are implemented with matching test evidence; the replay capability is safe, sendable, private, and released exactly once; the boundary error surface and the two-sink secondary evidence model match the architecture; no user-triggerable panic, GIL soundness issue, or capability exposure was found; the runtime/stdlib/codegen `replay: None` migration is complete.

VERDICT: SATISFIED
