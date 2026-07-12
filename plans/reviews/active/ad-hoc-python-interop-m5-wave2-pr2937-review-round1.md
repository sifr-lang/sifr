## PR #2937 Review — M5 Wave 2 (workflow / PR-level pass)

### Verification

**PR-level packaging**
- `origin/codex/python-interop-m5-runtime-replay` == HEAD (nothing local unpushed).
- Two commits: `3ab7634d8` (code + 2 review artifacts) and `7dab5ba1e` (tracker only).
- `gh pr view 2937` — MERGEABLE, CLEAN, checks passing (only Mintlify skipped).
- Files (25): all within M5 wave-2 scope — new `python_error.rs` / `context_ops.rs` / `context_ops_tests.rs` / `python_test_support.rs` + a mandatory `replay: None` / `without_replay(...)` migration across stdlib/codegen/runtime + three semantics-preserving Rust 1.94 Clippy fixes (`method_call_args.rs` iter_mut, `python_interop.rs` `is_none_or` — verified equivalent, `python_interop_plan.rs` `write!`, `python_probe.rs` `&io::Error`, `dlpack_ops.rs` nested-or). No unintended files (secrets, build artifacts).
- File-size guardrail respected: `python.rs` 827, `object_ops.rs` 790, all new files ≤ 241.
- Tracker commit `7dab5ba1e` correctly flips `[ ]` → `[x]` for wave 2 with `PR #2937` link at `plans/issues/active/ad-hoc-declaration-first-python-interop.md:337-338`.

**Contract compliance vs `internal_docs/python_interop_protocol_architecture.md` §"Exit Cause And Decision Types" / §"Synchronous Context Managers"**
- Exception replay: `python_error.rs:12-23` `PythonExceptionReplay::capture` stores `(type, value, traceback)` in a `PyDict` inside a `ForeignObject`; `from_pyerr` (`:90-110`) captures for every `PyErr`; every non-`PyErr` construction uses `replay: None` / `without_replay(...)`. `Py<PyAny>: Send + Sync` + `Arc<Mutex<...>>` → `PythonError: Send + Sync` — the `Send + Sync` closure bounds in `callback_ops.rs:47,74,84` still hold, and the build proves it.
- Nested/final release: `Arc<ForeignObjectInner>` clone-shared between `PythonError` clones (`foreign_object.rs:12-38`). `PythonError::replay` (`python_error.rs:130-142`) `clone_ref`s new `Py<PyAny>` per call — identity preserved across nested exits. Test 1 (`context_ops_tests.rs:12-68`) asserts `exc_type is Marker`, `exc_value is ORIGINAL`, `tb is ORIGINAL.__traceback__` across two nested exits, then verifies release goes through the off-GIL pending queue (`pending_release_count == 1` after final drop, drained by next `attach`).
- `SifrBoundaryError` registration: `python.rs:222-224` calls `context_ops::register_boundary_error` via `Python::try_attach` at init; `context_ops.rs:51-78` idempotently constructs a `RuntimeError` subclass and installs it in `sys.modules['__sifr_context__']` under an `OnceLock` guard. `SifrExitCauseKind::label()` (`context_ops.rs:26-33`) matches the doc's cause taxonomy; `call_exit_sifr_cause` (`:164-186`) passes `py.None()` for traceback (matching "no fabricated Python traceback beyond the adapter frame"). Test 2 asserts all five boundary fields.
- Manager consume/poison: `finish_context_exit` (`context_ops.rs:188-202`) is total — `Ok → object.close()`, `Err → object.poison()`. `close()` releases the manager `Py` via off-GIL pending queue; `poison()` keeps the `Py` for downstream error reporting, and drop after error consumption routes to the same queue. Test 3 verifies `__del__` fires only after the caller drops the error and re-attaches.
- Secondary evidence: two sinks (`attach_secondary_python_error` folds into the primary Python error's `.context`; `record_context_cleanup_evidence` pushes to a `LazyLock<Mutex<Vec<_>>>` for non-Python primaries). `take_context_cleanup_evidence` uses `mem::take`; `reset_context_state_for_tests` clears it (`python.rs:524`). Test 4 covers both.

**Coverage**
Four wave-2 acceptance items (exact triple identity, nested replay, boundary field fidelity, poison-on-cleanup + secondary evidence) each have a matching test.

### Findings (non-blocking; already flagged in the round-1 local review)
- `plans/reviews/active/ad-hoc-python-interop-m5-wave2-review-round1.md:1` — accidental preamble line "Now writing the code review." leaks the model's meta-narration into the committed review artifact. Cosmetic only, but the artifact is now part of the PR record.
- `crates/sifr_runtime/src/python/context_ops.rs:76` — `let _already_registered = BOUNDARY_ERROR_TYPE.set(...).is_err();` is dead: the `.get().is_some()` guard on `:52` already protects re-registration.
- `crates/sifr_runtime/src/python/python_error.rs:56` — `pub(crate) replay` is wider than needed; `pub(super)` suffices. Non-blocking (resolution is `pub(super)`).
- `crates/sifr_runtime/src/python/python_error.rs:12-19` — `capture` silently degrades to `None` on `set_item` failure. Downstream `context_exit_python_error` becomes a runtime error, so the manager's `__exit__` is not called with the real triple in that pathological case. Acceptable degradation; a comment tying this to the doc's fidelity guarantee would help future auditors.

No correctness, safety, privacy, lifecycle, or coverage regression; nothing that warrants a re-push.

VERDICT: SATISFIED
