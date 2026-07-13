Based on the exhaustive review, I have enough context to render a verdict. Let me summarize my findings.

## Summary of Review

I reviewed the M7 Wave 7 activation across every changed file and the design/round-1/round-2 review artifacts.

### What is delivered end-to-end

- **Gates lifted correctly.** `python_interop.rs:60`/`:120` and `:362` remove `reserved_declaration`/`reserved_cleanup` for coroutine and `cleanup=async_close`, without affecting the other reserved kinds (`async_context`, etc.). Sync/async substitution diagnostics stay live (`python_coroutine_contract_tests.rs:66-83`).
- **One owned loop, one submission registry.** `async_runtime.rs` still uses one dedicated `sifr-python-asyncio` OS thread and one submission BTreeMap. Concurrency test `typed_failures_and_concurrent_calls_use_one_terminal_registry_and_loop` asserts identical loop/thread markers.
- **Cancellation classification.** `terminal_for_submission` now returns `PythonTerminalError::ActiveCancellation` on `CancelledBeforeClaim`; `classify_task_error` in `async_cancellation.rs:9` promotes `CancelledError` to `ActiveCancellation` only when `SubmissionCancellationBridge::was_requested()`; independent `CancelledError` (test `raises_cancelled`) stays a `PythonError`.
- **Non-catchable native mapping.** `submit_async_declaration` funnels `ActiveCancellation` through `async_cancellation::propagate` → `resume_fallback_after_claim` → executor-neutral `yield_once`. The new closed-enum `CancellationResume` (`cancellation.rs:57`) is idempotent; `FallbackUnavailable` and no-op-fallback both produce explicit runtime errors, verified by `active_cancellation_propagation_failures_are_explicit_and_bounded`.
- **Terminal race handled.** `PythonTerminal::complete` takes the outcome and `cancellation_claim` under one state lock, then drops the claim outside; a late `request_cancel` cannot rewrite a stored outcome.
- **Consuming async close.** Codegen `python_interop_async/conversions.rs:74-88` refuses to emit any consuming shape other than `Self.aclose(own self) -> Result[None, PythonError]`; lowering tests cover abandonment, partial-branch close, duplicate close, wrong target, wrong receiver.
- **Suppression / later exception / poison / shutdown** all covered by `semantic_async_close_uses_python_terminal_outcome_after_cancellation` (Tokio current-thread runtime, real `AbortHandle` bound as fallback) and `semantic_async_close_shutdown_and_submission_rejection_poison_safely`.
- **Codegen correctness fixes.** `async_record_field(&mut r, "status")` now generates `&"status".to_string()` (`&String` auto-derefs to `&str`), fixing latent broken generated Rust for record outputs; zero-argument `Vec::new()` gets explicit `Vec<PythonAsyncValue>` / `Vec<(String, PythonAsyncValue)>` type annotations to unblock inference.
- **No hidden production Tokio in the runtime.** `crates/sifr_runtime/Cargo.toml` adds `tokio` only under `[dev-dependencies]`; `propagate` uses `poll_fn`/self-waking, and `yield_once` is executor-agnostic. Generated apps use `#[tokio::main(flavor = "current_thread")]`, so the (already-existing) spawn→bind_fallback ordering pattern is race-free by construction.
- **Evidence.** New `verification/areas/python_interop/fixtures/async_declaration/{httpx_client.sifr, python_bridges/client.py, async_declaration_evidence.json}`; runner integration in `run.py` with a matrix validator (`validate_async_declaration_evidence`) enforcing ≥3 rows per positive/negative/cleanup/cancellation and locking the stdout marker; `example_packages.py` now copies `python_bridges/` into the compiled example package; `pyproject.toml`/`uv.lock` add `httpx`; example package still symlinks the area's `.venv`/`uv.lock`/`pyproject.toml` (locked read-only environment).
- **Profiles.** `async-declaration-examples` unconditionally added to create-PR, merge, nightly, and release profiles; create-PR python_interop step budget raised from 60s to 180s with the measured 105,034 ms rationale recorded in exit evidence.
- **Capability ledger.** `declaration_capabilities.json` flips `coroutine-declaration` to `active` with concrete owners for each evidence kind; cleanup row absorbs async close per the design's "no separate row" rule.
- **Docs.** `docs/python-interop.mdx`, both interop architecture docs, `internal_docs/architecture.md`, verification README/exit evidence, and `plans/roadmap.md` are updated. `SIFR-PYASYNC` explicitly remains reserved for later protocols (correct — no diagnostic in `PYASYNC` is emitted by activated code).
- **File sizes.** `python_interop.rs` 853, `async_runtime.rs` 817, `async_declaration.rs` 359, new `async_cancellation.rs` 63, `conversions.rs` 862, `cancellation.rs` 408 — all under the 900-line guardrail.

### Minor non-blocking notes

- `SubmissionCancellationBridge::was_requested` uses `map_or(true, …)` on a poisoned mutex — conservative and effectively unreachable given no user code runs inside that lock.
- The M7 checkbox in `plans/issues/active/ad-hoc-declaration-first-python-interop.md` remains unchecked and the merged PR link is absent, which is intentional per the design ("recorded by the normal follow-up tracker PR after this implementation PR merges").

The implementation directly resolves every finding from design round 1 (harness Tokio migration, named idempotent resume API with closed enum, terminal-lock race, cancel-before-registration classification, independent-CancelledError coverage, fallback-not-bound propagation, sub-kind ledger placement, file-size headroom) and matches design round 2's SATISFIED verdict.

VERDICT: SATISFIED
