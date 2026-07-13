# Python Interop M8 Milestone Review — Round 1

Reviewer: Claude Opus 4.7 (milestone-level, post-merge). Scope: all M8 code and history reachable from `main` HEAD `5aa9d4b86` (`feat(python): activate typed async contexts (#2972)`) plus prior substrate at `dd88ebf49` (PR #2970). No repo modifications. Branch `codex/python-interop-m8-closure`; the two untracked files (`plans/reviews/active/ad-hoc-python-interop-m8-milestone-review-round1.md` and `…-wave2-pr2972-review-round2.md`) are review artifacts, not source deltas.

## Spec-to-code coverage

M8 requirement (`plans/issues/active/ad-hoc-declaration-first-python-interop.md:596-653`) maps to the following anchors on `main`:

| M8 requirement | Where implemented | Evidence |
|---|---|---|
| `@python.context.aenter` / `.aexit` decorators active | `crates/sifr_lowering/src/lower/python_interop.rs:152-156, 176-183`; `context/declarations.rs:67-141, 143-218` | `python_async_context_contract_tests.rs:41-44, 47-84, 87-167` |
| `cleanup=async_context` active | `python_interop.rs:381`; `must_use_obligations.rs:6-11`; `mod_context.rs:240-242`; `class_body_lowering.rs:663-665` | `annotations_and_function_lowering.rs:821-848` (`"must be consumed by 'async with' before function exit"`); contract test at `:125-139` |
| Dedicated Python async-with HIR | `crates/sifr_ir/src/hir_nodes.rs:199-224` (`HirAsyncWithKind::Python { … active_error_type }`) | `lower/python_interop/context/async_with.rs:172-237` (distinct from native `UserDefined`) |
| Concrete outcome classification **before** native erasure | `crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:159-200` matches concrete `Some(Ok(Ok(None)))` / `Some(Ok(Ok(Some(_))))` / `Some(Ok(Err(false|true)))` / `Some(Err(_))` / `None` and dispatches to Python replay vs. `SifrExitCause`; runtime `AsyncExitCause` is not the classification source | `python_async_context_tests.rs:96-132`; `runtime/python/async_context_tests.rs:132-191` |
| Python replay | `async_context.rs:294-331` (`python_error_exit` → `PythonAsyncExitCause::Python(replay.clone())`); runtime `async_context.rs:53-56` (`error.replay(py)`) | `async_context_tests.rs:46-64` |
| `SifrBoundaryError` for Sifr causes | `runtime/python/async_context.rs:57-74` + `context_ops.rs:51-78, 223-232` (subclass of `RuntimeError`) | `async_context_tests.rs:66-90` (`SifrBoundaryError:task cancelled`) |
| Suppression **only** for originating Python | `async_context.rs:313-314` honors `Suppress` on Python path; `sifr_error_exit(..., return_primary=true)` at `:340-344, 86-100` always returns primary Sifr error | `python_async_context_tests.rs:63-94, 96-132`; fixture `unsuppressible_sifr_case:95-103` |
| Ignored truthy suppression + cleanup failures recorded | `async_context.rs:360-368`; `python_error_exit:315-322` calls `attach_secondary_python_error` (preserves original) or `record_context_cleanup_evidence` | `context_ops.rs:98-127`; runtime `…_failure_poisons_manager_and_enter_conversion_leaves_it_exit_capable` |
| Cancellation masked; terminal exit reached | `async_context.rs:169-199` — the `None =>` arm builds a fresh `CancellationCarrier`, submits `submit_async_context_exit(…Sifr(Cancellation)…)`, records evidence, then `resume_parent_cancellation` before returning `internal_error` | `runtime/python/async_context_tests.rs:132-191` |
| Exact parent claim resume incl. enter failure | `async_context.rs:137-144` (Err on enter → poison → notification check → `resume_parent_cancellation` → `internal_error`); `resume_parent_cancellation:224-237` handles every `CancellationResume` variant | `python_async_context_tests.rs:195-217` |
| Cleanup vs. body-error precedence | Python path: `Suppress => {}`, `Propagate => return Err(body_error)`, `Err(cleanup) => attach_secondary` + return primary. Sifr path: post-cleanup `return Err(error)` regardless of decision | Fixtures `secondary_exit_failure_case`, `exit_failure_case`, `unsuppressible_sifr_case` |
| Manager vs. entered-resource ownership; scoped borrows | `context/async_with.rs:200-223` — `mark_moved_with_flow(owner)`; `python_context_borrows.insert(name, range)` with save/restore around body | `borrows.rs` reject-tests |
| Never-entered obligation on active surface | `AsyncContext → AsyncContextOnly` via `must_use_obligation_for_type`; reported with `OWN_USE_AFTER_MOVE` | Contract `async_context_obligation_is_reported_on_the_active_surface` |
| Distinct entered resource requires `cleanup=drop` | `declarations.rs:323-341` (`"distinct opaque \`X\` with cleanup policy … only the manager identity or \`cleanup=drop\` is allowed"`) | Contract `async_context_rejects_distinct_entered_resource_without_drop_cleanup` |
| Direct `__aexit__` rejection | `expressions/methods_lambdas_and_comprehensions.rs:263-268` — `PYCTX_INVALID_DECLARATION` + `"cannot be called directly"` | Contract `async_context_exit_cannot_be_called_directly` |
| Exact-once close/poison | Runtime `context_ops.rs:197-211` (`finish_context_exit`: Ok → `close`, Err → `poison`); `async_value.rs:159-194` (semantic-close consumes handle exactly once with Drop guard) | Runtime `…_borrows_manager_and_exit_consumes_it_exactly_once`, `…poisons_manager_and_enter_conversion_leaves_it_exit_capable` |
| Nested sync/async ordering & envelope | `python_context_envelope_depth` save/restore across class/function/generator (`class_method_emitter.rs:590-737`, `generator_bodies.rs:263-421`, `scope_and_function_types.rs:516-598`); nested return arm `async_context.rs:102-104` | `nested_async_python_context_preserves_outer_context_outcome_envelope`; fixture `nested_case` + `nested=lifo` marker |
| Return/break/continue/error paths | `rewrite_context_control_flow` in `sync.rs:533-611`; return arm `:102-114`; loop arms `:417-427`; body-error arm `:168` | `async_python_context_emits_all_concrete_body_outcomes` |
| Active `Error` supertype accepted | `context/async_with.rs:253-265` widens `PythonError → Error`; `mapped_internal_error` `Error::new(runtime.to_string())` at `async_context.rs:394-407`; `impl From<PythonError> for Error` emission gated on `uses_async_python` | Contract `active_async_with_accepts_python_errors_under_the_builtin_error_supertype`; codegen `async_python_context_converts_enter_failures_to_the_active_error_type` |
| One owned loop, no nested loop/executor | Enter/exit run under `__SIFR_TASK_CANCELLATION.scope(child, submit_async_context_{enter,exit}…)` at `async_context.rs:128-134, 171-184, 305-312`; no `spawn_blocking`/`Runtime::new` in the M8 surface | Fixture asserts `evidence.loop_identity != "drift"` from a single `(id(loop), thread)` sample gathered across every bridge entry (`session.py:24-26, 184`) |
| Offline `aiosqlite` matrix, exact 7:7:7 counts | `aiosqlite_session.sifr:73-217` walks normal / Python suppression / Sifr unsuppressed / exit failure / secondary failure / nested / cancellation, asserts `enter=exit=close=7`, `python_suppressed`, `sifr_truthy_seen`, `exit_failure_seen`, `secondary_exit_failure_seen`, `cancellation_ordered`, `nested_lifo`; marker locked in fixture print, `async_context_evidence.json:28`, `async_context_examples.py:14-17`, `demos/m8_demo/README.md:20`, and `run.py:490-496` `SystemExit` guard | — |
| Unconditional 4-profile ownership | `async-context-examples` in `create-pr.json:121`, `merge.json:119`, `nightly.json:132`, `release.json:131`; manifest at `manifest.json:197-210` (`kind=adapter, timeout=600s`); dispatched by `runner.py:89-92, 118` and `runner/run.py:15-18, 196, 247-253, 490-496` | — |
| Capability ledger active | `declaration_capabilities.json:95-108` — `implementation_status=active`, all 5 evidence categories `passing` | — |
| Demo | `demos/m8_demo/README.md`, `demos/m8_demo/run.sh` (hardened marker check via `grep -F -m1`) | — |
| Public/internal docs | `docs/python-interop.mdx:195, 207`; `internal_docs/python_interop_declaration_architecture.md:151, 302-322`; `python_interop_protocol_architecture.md:132, 252, 287, 289, 300, 305-306`; `architecture.md:834, 845`; `verification/areas/python_interop/README.md:20, 194-200`; `reports/python_interop_exit_evidence.md:134-174` | — |
| M9-M12 still reserved | `python_interop.rs:68-71, 187-190` (bare `_ => reserved_declaration(...)`); `python_interop.rs:414-425` (`bridge` targets); `declaration_capabilities.json:110-199` (all six `implementation_status=reserved`) | Reservation regression `python_interop_tests.rs:109-123` still active |

## Root-cause anchors from prior wave reviews (retained on `main`)

- Wave-1 `reserved_cleanup_seen` scaffolding fully removed: `python_interop.rs:376-381, 410-412` no longer references it (verified via grep; zero hits). Companion cleanup in `context/declarations.rs` also gone.
- `demos/m8_demo/run.sh:14` uses `jq -r … | grep -F -m1 'sifr-python-interop:async-context:'` (round-1 hardening).
- Suppression-aware reachability, `Result[None]` direct-try-capture, nested envelope preservation, and `Error`-active mapping all have named regression tests referenced in wave-2 PR-level round-1 review.

## Findings

### Blockers
None.

### Actionable follow-ups (not blockers)
The tracker/roadmap deferral is intentional and matches the M7 pattern (`PR #2968` activation → `PR #2969` closure docs). This branch (`codex/python-interop-m8-closure`) is exactly the closure PR and should flip:
- `plans/issues/active/ad-hoc-declaration-first-python-interop.md:145` — `- [ ] M8 async context managers` → `- [x] …  — [PR #2970](...), [PR #2972](...)`.
- Same file `:647` — the Wave 2 checkbox to `[x]` with the `#2972` link.
- `plans/roadmap.md:129` — `M0-M7 implementation is active` → `M0-M8`.
- Archive `plans/reviews/active/ad-hoc-python-interop-m8-*.md` review artifacts per the repo's post-milestone convention.

These are the only remaining bookkeeping actions and are exactly what a SATISFIED verdict authorizes.

### Observations (non-blocking, preexisting)
- Two `unreachable!()` sites in generated code at `async_context.rs:112` and `:424` mirror the sync-context design (`sync.rs:225-233, 250-260`); guarded by upstream state (`try_closure_depth`, `loop_else_stack`) — compile-time invariants, not data-dependent panics. Not a Wave 2 regression.
- `mapped_internal_error` vs. `mapped_result_error` asymmetry (`async_context.rs:378-415`) is intentional — constructed `PythonError::without_replay` vs. bound `?`-lifted value; documented in wave-2 review-round-1 obs #4.
- Nested `try` with distinct error types across nested `async with` — preexisting shape not exercised by any fixture (obs #5).
- `rust_stmts_contain_await` traverses `RustStmt::LocalFn` bodies — preexisting, not exercised (obs #6).

### File-size guardrail
All M8-authored files are well under 900: runtime `async_context.rs` 76; runtime `async_context_tests.rs` 263; codegen `python_context/async_context.rs` 427; lowering `python_interop/context/async_with.rs` 265; `context/declarations.rs` 342; `context/borrows.rs` 364; codegen `python_async_context_tests.rs` 217; lowering `python_async_context_contract_tests.rs` 167. Larger files inherited from prior milestones (`python_interop.rs` 860, `async_with.rs` 888, `class_body_lowering.rs` 839, `methods_lambdas_and_comprehensions.rs` 898, `python.rs` 895) remain below 900.

### Validation signal
The prior PR-level reviews cover the exhaustive local matrix (focused lowering 9, `sifr_codegen` 783, runtime async-context 5, compiled fixture, demos, guardrails, and `--profile create-pr` 130/130). On this branch `git status` shows no source-code deltas and no unexpected untracked files (only the two review artifacts). No revalidation was required and none is warranted.

## Final assessment

Every M8 requirement — active decorators, dedicated Python async-with HIR/lowering with pre-erasure classification, Python replay, `SifrBoundaryError` construction, Python-only suppression with recorded ignored-truthy/secondary-failure evidence, terminal masked exit under cancellation with exact parent resume including enter-failure, exact-once close/poison, distinct-entered `cleanup=drop`, direct-exit rejection, nested sync/async LIFO with envelope-depth save/restore, no nested event loop, offline aiosqlite matrix with exact 7:7:7 counts and locked marker, four-profile unconditional ownership, capability ledger and public/internal doc consistency, and preserved M9-M12 reservations with retained regression coverage — is present on `main` at `5aa9d4b86`. No blockers, no false evidence claims, no stale reservation scaffolding, no tracker-closing gap inconsistent with the M7 workflow, no guardrail violations, no runtime panic paths in user code. The tracker checkbox at `plans/issues/active/ad-hoc-declaration-first-python-interop.md:145` may be closed by the pending closure PR.

VERDICT: SATISFIED
