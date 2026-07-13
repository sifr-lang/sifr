I've completed the review. Here is my report:

# Python Interop M8 Wave 2 — Independent Review

## Scope Executed

Read the full working tree diff against `main` (38 files, +487/-112) plus the three untracked additions:
- `demos/m8_demo/` (README.md, run.sh)
- `verification/areas/python_interop/fixtures/async_context/` (aiosqlite_session.sifr, async_context_evidence.json, python_bridges/session.py)
- `verification/areas/python_interop/runner/async_context_examples.py`

Cross-checked against the M8 spec at `plans/issues/active/ad-hoc-declaration-first-python-interop.md:596-653`.

## Coverage Traced

| Required behavior | Status |
|---|---|
| Only aenter/aexit/`cleanup=async_context` reservations lifted, M9-M12 intact | ✅ `python_interop.rs:69,188,415` still emit `PYRES_UNIMPLEMENTED_DECLARATION` for other kinds; `parse_opaque_class` now returns Some(AsyncContext) instead of erroring |
| Lowering obligations, entered-value rules, direct-exit, `Error` supertype, negative cases | ✅ 9 focused contract tests including `async_context_rejects_distinct_entered_resource_without_drop_cleanup`, `async_context_obligation_is_reported_on_the_active_surface`, `active_async_with_accepts_python_errors_under_the_builtin_error_supertype`, `async_context_exit_cannot_be_called_directly` |
| Codegen: all body outcomes, Python-only suppression/replay, Sifr unsuppressibility, secondary evidence, exact-once, nested LIFO, cancellation masking, one-loop | ✅ `python_context/async_context.rs` + `sync.rs`; unit tests + focused fixture matrix |
| Root-cause fixes: awaits in sync ctx, suppression-aware reachability, Result[None] try-capture, nested return envelope, `Error` mapping, function-scope isolation of envelope depth | ✅ Every fix has a matching test (`sync_python_context_uses_async_closure_when_nested_body_awaits`, `python_async_context_suppression_keeps_following_return_reachable`, `test_direct_try_capture_converts_result_none_to_unit`, `nested_async_python_context_preserves_outer_context_outcome_envelope`, `async_python_context_converts_enter_failures_to_the_active_error_type`, class/function/generator scope saves/restores in `class_method_emitter.rs`, `generator_bodies.rs`, `scope_and_function_types.rs`) |
| Real offline aiosqlite fixture covering all matrices | ✅ `aiosqlite_session.sifr` walks 7 concrete cases; `python_bridges/session.py` subclasses `aiosqlite.Connection` over `sqlite3.connect(":memory:")` (no network); marker asserts `enter=7:exit=7:close=7:loop=shared:...` |
| Unconditional suite/profile registration | ✅ `async-context-examples` added to create-pr, merge, nightly, release profile `python_interop.suites`; manifest.json declares kind=adapter, timeout=600s; runner registers self-tests |
| Capability ledger + docs consistency | ⚠ mostly consistent — see finding #2 |

## Findings

### Non-blocking (should clean up)

**1. Stale reservation scaffolding in `parse_opaque_class`**
- File: `crates/sifr_lowering/src/lower/python_interop.rs:356,383,415`
- Wave 1 introduced `reserved_cleanup_seen` to keep the "requires `cleanup=`" diagnostic silent for the previously-reserved `async_context`. Now that `async_context` is active, `cleanup` is always `Some(AsyncContext)` on that path, so the `let Some(cleanup) = cleanup else { if !reserved_cleanup_seen { … } }` branch can never observe `reserved_cleanup_seen == true` — the variable is unreachable inside its own guard. This is dead code and stale reservation language.
- Fix: remove the `let mut reserved_cleanup_seen = …;` declaration on line 356, the `reserved_cleanup_seen = true;` assignment on line 383, and the `if !reserved_cleanup_seen` wrapper on line 415 (replace with a bare `invalid_shape(...)`).
- Impact: quality only; behavior is unchanged.

**2. Tracker/roadmap inconsistency for this activation PR**
- `plans/roadmap.md:129` updated to "M0-M8 implementation is active … with callback and zero-copy protocols sequenced next".
- `plans/issues/active/ad-hoc-declaration-first-python-interop.md:145` still shows `- [ ] M8 async context managers`, and line 647 still shows `- [ ] Atomically activate async contexts and close M8 evidence`, with no merged-PR link.
- This matches the M7 pattern (activation PR #2968 then closure docs PR #2969), so it is likely resolved in a follow-up docs PR. Non-blocking, but reviewers reading this PR alone will see a two-file inconsistency. Consider either (a) closing both checkboxes here so the tree is coherent at the merge boundary, or (b) leaving both `plans/roadmap.md` and the tracker unchanged in this PR and updating both in a separate docs PR.

**3. Defensive stdout parsing in `demos/m8_demo/run.sh`**
- `run.sh:14` uses `jq -r '.cases[0].stdout' "${REPORT}" | sed -n '1p'` to display the marker.
- Correct today because the fixture prints exactly one line, and the runner already enforces marker presence (`case_config.stdout_marker in proc.stdout` → status `example-failed`). If a future stdout line ever precedes the marker (e.g., a runtime warning) the demo would silently show the wrong first line while the underlying suite still passes. Optional hardening: `grep -F 'sifr-python-interop:async-context:' <(jq -r '.cases[0].stdout' "${REPORT}") | head -n1`.

### Observations (no action requested)

**4. `mapped_internal_error`/`mapped_result_error` asymmetry**
- `async_context.rs:394-406` special-cases `active_error_type == Error` to emit `Error::new(runtime.to_string())`. This is correct because the runtime PythonError is a function call (`PythonError::without_replay(...)`), and `bridge_error_expr`'s Error branch would fall through to `_ => value` (no `impl From<Error> for Error`). `mapped_result_error` uses `bridge_error_expr` and relies on the caller's `.into()` to convert `PythonError → Error`. The asymmetry is intentional (bound value vs. constructed value) but worth noting for anyone hunting later.

**5. Nested try + distinct try error types**
- `python_context_envelope_depth`-based nesting emits `return Ok(Ok(Some(__sifr_context_return)))` for the inner async context, assuming the outer context's envelope has the same `return_expression_type` (which depends on `current_return_type`, `try_closure_option_wrap`, and `active_error_type`). Both contexts inside the same `try` share these, so all fixture/test cases are correct. If a future user writes `try: async with a: try: async with b: return X` where the inner and outer `try` have different error types, the two envelope shapes diverge. This is preexisting (not a Wave 2 regression) and is not exercised by the fixture; noting it because Wave 2 is what makes async-with-inside-try common in real code.

**6. `rust_stmts_contain_await` traversal of `RustStmt::LocalFn`**
- `iterator_lowering.rs:678` recurses into `LocalFn { body, .. }`. If a synchronous Python context ever wraps a nested inner `async fn` declaration, the outer closure would be marked async because the inner's body contains awaits. Not exercised by the added tests or fixtures. Preexisting and out of Wave 2 scope; noted for future callers.

## Validation Signal (as reported in the task)

- Focused lowering (9), full `sifr_codegen` (783), Python runtime async_context tests (5), runner self-test, compiled async-context suite, demos/m8_demo, guardrails, and `--profile create-pr` (130 e2e, 0 failing, Python lane 27.6s/180s): all green.
- I did not re-run these; the diff is consistent with the described results and the marker validator in `run.py:473-496` enforces the exact stdout string against `async_context_evidence.json:28`.

## Verdict

Every required Wave 2 behavior is implemented, tested, and evidenced. The only actionable items are code-hygiene (dead `reserved_cleanup_seen`), a doc-consistency call between roadmap and tracker, and a defensive tweak in the demo script — all non-blocking cleanups. No correctness, ownership, cancellation, typing, evidence, or profile-selection bugs surfaced.

VERDICT: NEEDS_CHANGES
