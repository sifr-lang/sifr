# Independent PR Review — sifr-lang/sifr#2972 (M8 Wave 2 activation)

## Scope executed

- Verified `git rev-parse HEAD` == remote `origin/codex/python-interop-m8-activation` == `bd609ac64` (commit `feat(python): activate typed async contexts`).
- Confirmed PR file set: 45 files, +1075/−119, `mergeable: MERGEABLE`, base `main`, head `codex/python-interop-m8-activation`.
- Cross-checked against M8 spec at `plans/issues/active/ad-hoc-declaration-first-python-interop.md:596-653`.
- Re-read frozen-diff `plans/reviews/active/ad-hoc-python-interop-m8-wave2-review-round1.md` and `-round2.md`.
- The other untracked file, `plans/reviews/active/ad-hoc-python-interop-m8-wave2-pr2972-review-round1.md`, is 0 lines and not part of the PR; it is scratch state from a prior review invocation and does not affect commit `bd609ac64`.

## Round 1 resolution audit

| Round 1 finding | Status |
|---|---|
| #1 Stale `reserved_cleanup_seen` in `python_interop.rs` | ✅ Fully removed: the variable, the `[name] if name == "async_context" => { reserved_cleanup_seen = true; ... }` branch, the `if !reserved_cleanup_seen` guard, and the `fn reserved_cleanup` helper are gone (`python_interop.rs:376-381,410-412`). Companion Wave-1 scaffolding in `context/declarations.rs:321-346` also removed (net −35 lines). |
| #2 Tracker/roadmap inconsistency | ✅ Option (b) selected and documented in the PR description ("Tracker and roadmap checkbox/link closure are intentionally kept together in the post-merge tracker-only PR, matching the established milestone workflow"). Neither `plans/roadmap.md` nor `plans/issues/active/ad-hoc-declaration-first-python-interop.md` is modified in this PR, so the tree is coherent on both sides of the merge boundary and mirrors the M7 activation pattern (PR #2968 → docs PR #2969). |
| #3 Defensive stdout parsing in `demos/m8_demo/run.sh:14` | ✅ Hardened to `jq -r '.cases[0].stdout' "${REPORT}" \| grep -F -m1 'sifr-python-interop:async-context:'`. |

Round 2 (`review-round2.md`) verdict SATISFIED matches this state.

## Acceptance re-audit

- **Only async context reservations activated**: `parse_opaque_class` accepts `async_context` as active (`python_interop.rs:381`); the `_ =>` arms at `python_interop.rs:68-71` (functions) and `187-190` (methods) still call `reserved_declaration(...)` with `PYRES_UNIMPLEMENTED_DECLARATION` for `Callback`/`Buffer`/`Arrow`/`Dlpack`. Class-body match at `class_body_lowering.rs:663-665` now maps `AsyncContext` to `ContextAsyncExit` alongside the prior `Context → ContextExit` case.
- **Ownership/entered-resource diagnostics**: `python_async_context_contract_tests.rs` covers `async_context_rejects_distinct_entered_resource_without_drop_cleanup`, `async_context_obligation_is_reported_on_the_active_surface` (OWN_USE_AFTER_MOVE with "must be consumed by `async with`"), `async_context_exit_cannot_be_called_directly`, and every assertion also checks `!PYRES_UNIMPLEMENTED_DECLARATION` is emitted.
- **Python-only suppression / replay**: `async_context.rs:294-331 (python_error_exit)` replays via `body_error.__sifr_python_error.as_ref()`; suppression is honored only for originating Python causes, and cleanup failures attach through `attach_secondary_python_error` or `record_context_cleanup_evidence`.
- **Sifr unsuppressibility**: `sifr_error_exit(..., return_primary=true)` (`async_context.rs:333-376,86-100`) returns the primary Sifr error regardless of `PythonExitDecision::Suppress`, and truthy decisions land in `record_context_ignored_suppression` with the correct `cause_label:error_type` key.
- **Exact-once cancellation-safe cleanup**: the `None` arm in the `tokio::select!` (`async_context.rs:169-199`) issues `submit_async_context_exit` with `PythonAsyncExitCause::Sifr(Cancellation)` under a fresh `CancellationCarrier`, then always resumes the parent claim before returning the internal error. Enter failure and conversion failure paths also drive exit / poison exactly once.
- **Nested context return/order**: `python_context_envelope_depth` is bumped inside both `async_context.rs:61-63` and `sync.rs:13-15`, saved/restored across class/function/generator boundaries (`class_method_emitter.rs`, `function_emitter/generator_bodies.rs`, `function_emitter/scope_and_function_types.rs`), and drives the `Some(Ok(Ok(Some(__sifr_context_return))))` return arm at `async_context.rs:102-105` when the outer envelope wraps this one. Regression covered by `nested_async_python_context_preserves_outer_context_outcome_envelope` (`python_async_context_tests.rs`).
- **One owned loop**: enter/exit use `__SIFR_TASK_CANCELLATION.scope(child.clone(), sifr_runtime::python::submit_async_context_{enter,exit}...)` (`async_context.rs:128-134,171-184,305-312`) — all traffic goes through the application-owned loop; the fixture asserts `loop_identity != "drift"` from a single `id(loop):thread` sample (`python_bridges/session.py:24-26,182-184`).
- **Real offline `aiosqlite` evidence**: `python_bridges/session.py:29-31` subclasses `aiosqlite.Connection` over `sqlite3.connect(":memory:")` (no network); marker asserts `enter=7:exit=7:close=7:loop=shared:suppression=covered:sifr=unsuppressed:cancellation=ordered:nested=lifo:exit-failure=covered` and is locked in `runner/run.py:473-496`, `async_context_evidence.json:28`, `async_context_examples.py:14-17`, and `demos/m8_demo/README.md:20`.
- **Unconditional four-profile suite**: `"async-context-examples"` appears in `verification/profiles/{create-pr,merge,nightly,release}.json` and is registered in `manifest.json:198-210` (`kind=adapter`, `timeout=600s`), wired to `runner.py:89-93,118`.
- **Capability/docs consistency**: `declaration_capabilities.json:95-108` moves `async-context` to `implementation_status=active` with positive/negative/cleanup/cancellation/live evidence populated. `docs/python-interop.mdx:189-213`, `internal_docs/python_interop_architecture.md`, `python_interop_declaration_architecture.md`, `python_interop_protocol_architecture.md`, `architecture.md`, `verification/areas/python_interop/README.md`, and `reports/python_interop_exit_evidence.md` all describe the activated surface consistently.

## Root-cause fix coverage (envelope, awaits, none-like, control flow, Error mapping)

Every root-cause fix identified in Round 1 has an anchoring regression:
- Envelope-depth save/restore across function/method/nested-function/generator scopes → `class_method_emitter.rs:590-737`, `generator_bodies.rs:263-418`, `scope_and_function_types.rs:516-595`.
- Async closure when a sync-context body awaits → `sync.rs:167-184` + `sync_python_context_uses_async_closure_when_nested_body_awaits` test.
- Suppression-aware reachability → `hir_analysis/queries/queries_impl.rs:203-231` + `python_async_context_suppression_keeps_following_return_reachable`.
- `Result[None]` direct-try-capture conversion → `result_type_helpers.rs:3-17` + `test_direct_try_capture_converts_result_none_to_unit`.
- `Error`-typed active mapping in `mapped_internal_error` → `async_context.rs:394-407` + `async_python_context_converts_enter_failures_to_the_active_error_type`.

## File-size guardrail

All touched first-party files remain under the 900-line cap (largest: `stmt_block.rs` at 890, `python_interop.rs` at 860, `queries_impl.rs` at 728, `sync.rs` at 684, `async_context.rs` at 427).

## Validation signal

PR description records: focused lowering 9 pass, `sifr_codegen` 783 pass, runtime `python::async_context_tests` 5 pass, runner self-test pass, compiled `async-context-examples` + `demos/m8_demo/run.sh` pass with exact marker, guardrails green, `scripts/run_all_tests.sh --profile create-pr` 130/130 e2e. Two known repo-wide advisories on `main` (Ruff line-wrap `cargo fmt`, `sifr_driver` `format_push_string` Clippy) are unrelated to this diff. I did not re-run — CLAUDE.md explicitly makes local validation authoritative and per task scope no CI wait is required.

## Non-blocking observations (nothing actionable)

- The `mapped_internal_error` vs. `mapped_result_error` asymmetry for the `Error` supertype is intentional (constructed value vs. bound `?`-lifted value) and covered by the enter-failure test. Documented in Round 1 observation #4.
- `python_context_envelope_depth` correctly propagates for outer contexts that share the same `return_expression_type`/`active_error_type` (as guaranteed by a single enclosing `try`). Nested async-with inside distinct-error `try` blocks is a preexisting shape not introduced here.
- `Self::rust_stmts_contain_await` traverses `RustStmt::LocalFn` bodies; not exercised by these fixtures and preexisting.

## Verdict

Every Wave 2 acceptance requirement, every Round 1 finding, and every root-cause anchor is present in commit `bd609ac64`. The PR contains exactly the reviewed diff plus the frozen review artifacts, with no accidental/unrelated files. Tracker and roadmap closure are intentionally deferred to a post-merge docs PR, matching the M7 workflow that this milestone documented in advance.

VERDICT: SATISFIED
