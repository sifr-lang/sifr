I have enough context to render the verdict. Let me consolidate my independent findings.

## Independent Review of PR #2968

I inspected PR #2968 metadata, the full 2044-line diff against `main`, and cross-checked against `AGENTS.md`, the M7/Wave 7 tracker in `plans/issues/active/ad-hoc-declaration-first-python-interop.md:468-593`, the design at `plans/reviews/active/ad-hoc-python-interop-m7-wave7-activation-design.md`, both design review rounds, and the pre-PR implementation review. I also verified the file-size and HIR guardrails pass locally.

### Correctness & safety — verified

- **Gates lifted only for the two required forms.** `crates/sifr_lowering/src/lower/python_interop.rs:60,120,362` drop `reserved_declaration`/`reserved_cleanup` for `(Coroutine, async_def)` and `cleanup=async_close`. All other kinds still fall through to `reserved_declaration`/`reserved_cleanup` at `:170,364`. Sync/async substitution diagnostics preserved.
- **One application-owned loop, one submission registry.** `async_runtime.rs` still owns a single `sifr-python-asyncio` thread and one `BTreeMap` submission registry; `typed_failures_and_concurrent_calls_use_one_terminal_registry_and_loop` (`async_declaration_tests.rs:107`) asserts identical loop/thread identity across two spawned workers.
- **Active vs. independent CancelledError.** `async_cancellation.rs:9-22` promotes `CancelledError` to `PythonTerminalError::ActiveCancellation` only when `SubmissionCancellationBridge::was_requested()` is true; the added `raises_cancelled` fixture and `async_declaration_tests.rs:162-170` verify an independent `CancelledError` stays a `PythonError` with `exception_type` "CancelledError".
- **Terminal race is atomic.** `async_terminal.rs:63-78` stores the outcome and takes the `CancellationClaimLease` under one terminal-state lock, drops the claim outside, then wakes. A late `request_cancel` reaching the exact hook cannot rewrite the stored outcome; `terminal_completion_releases_claim_before_waking_waiter` asserts sequential re-claim after `complete`.
- **Non-catchable native mapping.** `submit_async_declaration` funnels both start-time and terminal `ActiveCancellation` through `async_cancellation::propagate` → `resume_fallback_after_claim` → executor-neutral `yield_once` (`async_cancellation.rs:24-59`). The new closed enum `CancellationResume` (`cancellation.rs:56-64`) is idempotent (`fallback_resumed` guard), invokes the hook at most once, and returns `FallbackUnavailable`/`AlreadyResumed`/`ExactClaimActive`/`NotRequested`/`StateUnavailable` for every non-invocation. The idempotency and cross-path invariants are exercised by `claimed_terminal_resume_invokes_fallback_once_after_lease_release` and `resume_requires_request_and_bound_fallback`.
- **Bounded malformed-fallback path.** `active_cancellation_propagation_failures_are_explicit_and_bounded` exercises both no-fallback and no-op-fallback branches after `wait_for_typed_submission` → the waiter returns `AsyncRuntimeFailed(...)` rather than hanging.
- **Raw API contract preserved.** `submit_coroutine` maps `PythonTerminalError::ActiveCancellation` back to `AsyncSubmissionCancelled` via `terminal_error_to_python` (`async_terminal.rs:127-129`), and the existing `cancellation_before_claim_does_not_submit_python_work` still asserts the raw catchable-runtime contract.
- **Suppression / later-exception / poison / shutdown.** `semantic_async_close_uses_python_terminal_outcome_after_cancellation` (Tokio current-thread runtime + real `AbortHandle` bound as fallback) covers suppression-then-None (clean close), observed cancellation (poisoned + native cancel), and suppression-then-ValueError (later exception wins, poisoned). Shutdown drain is covered by `semantic_async_close_shutdown_and_submission_rejection_poison_safely`.
- **Consuming close.** `python_interop_async/conversions.rs:65-88` restricts the emitted shape to `Self.aclose(own self) -> Result[None, PythonError]`; the pre-existing lowering tests (`async_close_requires_one_consuming_aclose_coroutine`, `active_async_close_consumption_discharges_obligation`) exercise abandonment, wrong receiver, wrong target, and partial-branch cases.
- **No hidden production Tokio.** `crates/sifr_runtime/Cargo.toml` adds `tokio` only under `[dev-dependencies]`; `propagate` is executor-neutral (`poll_fn` + `wake_by_ref`). Generated apps use `#[tokio::main(current_thread)]` so the spawn→bind_fallback order is set up by the parent frame before submission.
- **Later declaration families stay reserved.** `declaration_capabilities.json` leaves `async-context`, `callback-*`, `buffer-protocol`, `arrow-c-data`, `dlpack-transfer` at `implementation_status: reserved`; only `coroutine-declaration` flips to `active`. `later_python_decorator_is_a_hard_error` in `python_interop_tests.rs:113` now uses `@python.callback` (still reserved) since coroutine is no longer a valid negative-example anchor.

### Codegen correctness fixes

- `&"status".to_string()` for `async_record_field(&mut r, &str)` — `&String` auto-derefs to `&str`; correct.
- `Vec::new()` for zero-arg `__sifr_python_args`/`__sifr_python_kwargs` now gets explicit `Vec<PythonAsyncValue>` / `Vec<(String, PythonAsyncValue)>` annotations to unblock inference. Regression covered by `zero_argument_record_wrapper_emits_concrete_frames_and_borrowed_field_names`.

### Evidence

- New checked-in fixture: `httpx_client.sifr` + `python_bridges/client.py` running a real `httpx.AsyncClient` subclass over an offline `ASGITransport`, no network; deterministic stdout marker `sifr-python-interop:async-declaration:status=207:message=async-ready:close=1:loop=shared:failure=covered:conversion=covered` locked by `validate_async_declaration_evidence` in `runner/run.py:397-421`.
- `async-declaration-examples` added to **create-pr, merge, nightly, release** profiles (all four); create-pr `python_interop` step budget raised from 60s to 180s with measured 105,034 ms rationale recorded in `reports/python_interop_exit_evidence.md`.
- Runner support (`runner/run.py`, `runner/async_declaration_examples.py`, `example_packages.py` copytree of `python_bridges/`) is unconditional; validator asserts ≥3 owners per positive/negative/cleanup/cancellation matrix.
- `demos/m7_demo/run.sh` invokes the same fixture through the area runner and prints the marker via `jq -r '.cases[0].stdout' | sed -n '1p'`.
- `pyproject.toml`/`uv.lock` add `httpx>=0.27,<1` hermetically inside the area's locked env.

### Docs / capability ledger / roadmap

- `declaration_capabilities.json` flips only `coroutine-declaration` to `active` with concrete owners on positive/negative/cleanup/cancellation/live; consuming async-close is absorbed into the cleanup row per the design's "no separate row" rule.
- `docs/python-interop.mdx` gains a typed-async section, cancellation-semantics paragraph, and the `--async-declaration-examples` verification step.
- Both interop architecture docs, `internal_docs/architecture.md`, and `plans/roadmap.md` are updated. `SIFR-PYASYNC` explicitly remains reserved in the internal architecture doc (correct — activated code doesn't emit any PYASYNC code).
- The M7 checkbox in `plans/issues/active/ad-hoc-declaration-first-python-interop.md` intentionally stays unchecked; the design says the tracker PR records the merged link.

### File-size & guardrails (verified locally)

`python_interop.rs` 853, `async_runtime.rs` 817, `async_declaration.rs` 359, `async_declaration_tests.rs` 621, new `async_cancellation.rs` 63, `conversions.rs` 862, `cancellation.rs` 408 — all < 900. `python3 scripts/check_file_size_guardrails.py`, `check_hir_maintainability_guardrails.py`, and `cargo fmt --check` all pass.

### Non-blocking notes

- `SubmissionCancellationBridge::was_requested` uses `map_or(true, …)` on a poisoned mutex, which is conservative (fail-active). The bridge only holds `requested`/`submission_id` and never runs user code inside the lock, so poisoning is exceedingly unlikely; the pre-PR review already flagged this as accepted.
- The `&"status".to_string()` codegen shape allocates a `String` just to deref-coerce to `&str` — correct, but wasteful. Not a bug and can be tightened by a future `RustLiteral::StrSlice` variant.

No correctness, safety, evidence, documentation, or workflow blockers found. Every design round-1 gap (harness Tokio migration, named idempotent resume API with closed enum, terminal-lock race, cancel-before-registration classification, independent-CancelledError coverage, fallback-not-bound propagation, cleanup-sub-kind ledger placement, submodule file-size headroom) is directly resolved in this diff.

VERDICT: SATISFIED
