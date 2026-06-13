# Concurrency M1 — Structured Async Runtime Review

## Result: **PASS**

The M1 implementation closes the four focus areas with deterministic diagnostics, working pass/fail fixtures, and consistent traceability. Reserved `ctx=None` is purely syntactic, `task.spawn_scoped` routes through the same scope-spawn pipeline (preserving sendability / borrow / affine-handle checks), `task.select` enforces the named-branch contract, and validation evidence (71 e2e pass + targeted lowering/codegen tests + create-pr lane) matches the ledger entry. M1 is ready to PR once this review is recorded in the planning-reviews section of the execution ledger.

## Focus-area verification

- **`task.TaskGroup(ctx=None)` reserved M5 slot** — `crates/sifr_lowering/src/lower/async_with.rs:758` calls `validate_reserved_task_context_keyword`, defined at `crates/sifr_lowering/src/lower/task_context_keywords.rs:6`. The HIR `HirAsyncWithKind::TaskGroup` carries no ctx field, so codegen lowers identically to the no-arg form (no runtime propagation). Positive/negative coverage: `task_runtime_m1.rs:4` (`ctx=None` accepts) and `task_runtime_m1.rs:14` (non-`None` rejected with `SIFR-TYPE-0002` and the M5 message). The unexpected-keyword paths return `SIFR-CALL-0002` from `task_context_keywords.rs:15-37`.

- **`task.spawn_scoped(..., ctx=None)`** — Dispatched at `crates/sifr_lowering/src/lower/task_calls.rs:36`, implemented at `task_calls.rs:50`. The helper requires `current_function_is_async`, `active_task_owner_depth > 0`, and a named binding pulled from `active_task_owner_bindings.last()` (see `task_owner_scope_state.rs:33` for the push site). It then routes through `lower_task_scope_spawn_from_object_allowing_reserved_ctx` (`task_scope_calls.rs:37`), so the existing direct-coroutine, borrowed-capture, non-send and `TaskGroup`-error-homogeneity guards apply unchanged. Tests:
  - `task_runtime_m1.rs:27` — lowers through named owner, asserts method `__sifr_spawn_infallible` on `Name("group")` and `Type::Task(int, Never)`.
  - `task_runtime_m1.rs:59` — no active owner rejected.
  - `task_runtime_m1.rs:85` — unnamed `task.TaskGroup():` rejected with "requires a named active task owner".
  - `task_runtime_m1.rs:72` — non-`None` ctx rejected with the M5 message.
  - E2E pass: `crates/sifr/tests/e2e/pass/task_spawn_scoped_named_owner.sifr` (in create-pr manifest at `verification/validation_lanes/create_pr_e2e_manifest.json:26` and the merge manifest at `merge_e2e_manifest.json:68`).
  - E2E fail: `crates/sifr/tests/e2e/fail/task_spawn_scoped_without_owner_rejected.sifr` (expects `SIFR-TYPE-0002`).

- **`task.select(first=..., second=...)`** — `crates/sifr_lowering/src/lower/task_calls.rs:261-361`. Positional rejected with `SIFR-CALL-0001` (`task_calls.rs:270`), wrong named-branch count rejected (`task_calls.rs:279`), unpacked kwargs rejected (`task_calls.rs:289`), duplicate branch names rejected (`task_calls.rs:300`). Branch values must be `Type::Task` and are marked moved (`task_calls.rs:348-349`). Codegen tests `crates/sifr_codegen/src/lib_codegen_tests/async_task_runtime_codegen_tests.rs:186` and `:225` cover the named form for infallible and fallible branches. Existing ownership test `expressions_tests/ownership_and_async.rs:467-468` reuses the named form to assert handle consumption.

- **Existing M1 guarantees intact** — `async_task_runtime_codegen_tests.rs` confirms unchanged emission for TaskGroup fail-fast (`group.__sifr_join_all().await` Err handling at `:24`), `__sifr_task_gather` ordered results / abort_handle (`:48-51`), `__sifr_task_race` loser cancellation (`:143-149`), `__SifrTaskResult::cancelled()` semantics (`:317`), and `__SifrTimeoutResult` `biased;` ordering (`:336-345`). No raw `tokio::*` types appear in any user-visible diagnostic or emission, and `sifr.asyncio` is not referenced in the M1 lowering. The TaskGroup open-state ladder (`task_scope_calls.rs:445`) and group-error homogeneity check (`task_scope_calls.rs:462`) still apply via the shared spawn path.

- **Doc / ledger / manifest updates**
  - `internal_docs/async_concurrency_model.md:98,527` describe `task.select(first=, second=)` as the binary named-branch form; the model invariant `7` and signatures at `:370` are consistent.
  - `verification/stdlib/concurrency_runtime_m1_traceability.md` enumerates all required fixtures and CPython family mappings.
  - Manifests now include `task_spawn_scoped_named_owner` (create-pr + merge) and `task_select_first_completion` (merge); create-pr count is 71 lines, matching the validation report.
  - Execution ledger `M1 Implementation Ledger` section records the wave, fixtures, and the full local-validation set; only the "Pending Reviews" needs this review's PASS appended.

## Non-blocking findings

1. **Stale demo using positional `task.select`** — `demos/structured_concurrency_demo/main.sifr:52` still calls `task.select(scope.spawn(fast()), scope.spawn(slow_writes_marker()))`. The file is not in any validation manifest, so the create-pr / merge gates pass, but `cargo run -q -p sifr -- check demos/structured_concurrency_demo/main.sifr` now fails with `SIFR-CALL-0001`. Update the demo to the named-branch form (`first=…, second=…`) so the checked-in demo continues to compile under the M1 surface.
2. **`async_concurrency_model.md` does not mention `task.spawn_scoped`** — `grep "spawn_scoped" internal_docs/async_concurrency_model.md` returns nothing. The model doc is the canonical contract; adding a one-line `task.spawn_scoped(coro, *, ctx=None)` entry alongside the other `task.*` signatures (around `:368-378`) would close the doc/ledger consistency loop. M1 traceability covers the API, so this is documentation polish rather than a blocker.
3. **Historical `task.select(a, b)` references in archived phase docs** — `internal_docs/phases/32_async_ecosystem.md:452,512,535` describe prior positional-form PR slices; they are archival narrative, but a future reader may misread them as current API. Optional editorial cleanup.

## Required remediation before PR

None of the findings block the PR. Recommended (not gating) follow-ups:

- Update `demos/structured_concurrency_demo/main.sifr:52` to use `task.select(first=..., second=...)` so the demo still compiles.
- Optionally add a `task.spawn_scoped` signature line to `internal_docs/async_concurrency_model.md` so the canonical contract reflects the new module-level helper.

## Ready-to-PR statement

M1 is ready to PR once the execution ledger's planning-reviews section records this review's `PASS`. The implementation satisfies all definition-of-done items: deterministic task lifetime / cancellation, typed task failure / cancellation / timeout evidence, TaskGroup fail-fast behavior, observed handle semantics, no Tokio / event-loop leakage, no `sifr.asyncio` dependency path, reserved `ctx` slots without runtime propagation, named `task.select` enforcement, and complete fixture + manifest coverage.
