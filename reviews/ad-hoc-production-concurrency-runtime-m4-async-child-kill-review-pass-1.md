RESULT: PASS

# M4 Async Child Kill Implementation Review (Pass 1)

Branch: `codex/concurrency-runtime-m4-async-child-kill`
Scope: Method-form `sifr.process.AsyncChild.kill()` async force-kill with preserved wait observation.
Reviewer: Claude Opus 4.7 (designated phase reviewer).

## Verdict

No blockers. The slice cleanly adds the `AsyncChild.kill()` async lifecycle API on top of the recently merged async spawn/wait substrate while honoring every explicitly named scope deferral. The public surface, intrinsic metadata, intrinsic lowering, generated helper, child-table preservation, prelude gating, fixture coverage, validation lanes, host matrix, traceability, and execution ledger are mutually consistent and honest about what is shipping versus deferred.

## Detailed Findings Against The Review Questions

### Q1. Public surface — no Tokio leakage, no broken borrowed top-level async wrapper

PASS.

- `lib/sifr/process.sifr:168-178` exposes `AsyncChild` with `_handle: int`, `kill() -> Awaitable[Result[None, ProcessError]]`, and `wait() -> Awaitable[Result[Status, ProcessError]]`. Both return typed `Result` payloads wrapped in `Awaitable`. No public type leaks `tokio::process::Child` or any Rust process type.
- `lib/sifr/process.sifr` does not declare a top-level `async_kill(child)`; only `async_wait(own child)` continues to exist. The intentional omission matches the scope statement that current Sifr stdlib async wrappers cannot safely return/preserve a borrowed class handle. The method form preserves `self._handle` exactly because `kill()` is an instance method whose receiver is borrowed, not consumed.
- `process_async_kill` intrinsic metadata at `crates/sifr_codegen/src/intrinsics/registry.rs:322-327` declares `("handle", Type::Int)` and `Type::Awaitable(Box::new(result_ty(Type::None, "ProcessError")))`, again typed in Sifr-domain terms (Int handle, `ProcessError`, `None`).

### Q2. Stdlib metadata, intrinsic lowering, and generated helper signature agreement

PASS.

- Metadata: `crates/sifr_codegen/src/intrinsics/registry.rs:321-327` — one `i64`-equivalent handle parameter and `Awaitable<Result<None, ProcessError>>` return.
- Lowering dispatch: `crates/sifr_codegen/src/intrinsics/registry.rs:636-639` routes `process_async_kill` to `process_async::lower_process_async_kill` and tags `StdlibFeature::Tokio`, consistent with the other async process intrinsics.
- Lowerer: `crates/sifr_codegen/src/intrinsics/registry/process_async.rs:106-114` validates `args.len() == 1`, then emits `Box::pin(__sifr_process_async_kill(handle))` via `boxed_async_process_helper_call`, matching the wait lowerer’s shape.
- Helper signature: `crates/sifr_codegen/src/preamble/process_async_runtime.rs:817-827` emits `async fn __sifr_process_async_kill(handle: i64) -> Result<(), ProcessError>` using `process_async_wait_params()` (one `handle: i64`) and `process_async_ret("()")`. Verified end-to-end by emitting the fixture: `async fn __sifr_process_async_kill(handle: i64) -> Result<(), ProcessError>` is present in the output.

### Q3. Helper requests termination without removing the child from the async child table

PASS.

- The kill helper body at `crates/sifr_codegen/src/preamble/process_async_runtime.rs:627-637` performs `__children.get_mut(&handle)` (mutable borrow only) and calls `start_kill()`. There is no `__children.remove(&handle)`. This contrasts intentionally with the wait helper at `crates/sifr_codegen/src/preamble/process_async_runtime.rs:612-625`, which `remove`s the entry before awaiting — so once-and-only-once status observation remains the wait helper’s responsibility, and the kill helper leaves the table entry intact for a subsequent `async_wait(own child)` or `AsyncChild.wait()`.
- The fixture at `crates/sifr/tests/e2e/pass/process_async_child_kill_wait.sifr:7-30` exercises both observation paths (top-level `await async_wait(child)` and method-form `await method_child.wait()`) after `await child.kill()`, and asserts `status.kind == "signal"` plus `status.signal is not None`. It also verifies that after `wait()` removes the entry, a second `kill()` correctly fails with a typed `ProcessError` containing `"closed or unknown"`.

### Q4. No mutex guard held across an await

PASS.

- `start_kill()` is the synchronous `tokio::process::Child::start_kill` API (`io::Result<()>`), not the awaiting `kill()` future. The helper acquires the mutex inside a scoped `{ ... }` block, performs the lookup and `start_kill` call, then drops the guard at the closing brace before `return Ok(())`. There is no `.await` anywhere in the helper body.
- The fact that the helper is declared `is_async: true` (so callers `.await` `Box::pin(__sifr_process_async_kill(handle))`) is purely a calling-convention concern — the body itself contains no suspension point, and therefore the `std::sync::Mutex` guard cannot straddle one. This is the same pattern the spawn helper uses for inserting into the table.

### Q5. Typed errors with no user-triggerable panic

PASS.

- Missing/closed handle: `__children.get_mut(&handle).ok_or_else(|| ProcessError { message: format!("async process child handle {} is closed or unknown", handle) })?` produces a typed `Err` rather than panicking. Identical phrasing to the wait helper, so the `closed or unknown` substring assertion in the fixture is stable across both paths.
- `start_kill` IO errors are mapped through `map_err(|__sifr_process_error| ProcessError { message: __sifr_process_error.to_string() })?`, again typed.
- Mutex poisoning is handled with `unwrap_or_else(|__err| __err.into_inner())`, which recovers the inner guard rather than panicking — consistent with the wait and spawn helpers.
- I see no `.unwrap()` / `.expect()` on user-data-dependent results introduced in this slice.

### Q6. Prelude filtering/gating emits async child helper/table only when needed; sync child tables not pulled in for async-only usage

PASS.

- `SharedPreludeProcessAsyncNeeds` gains a `needs_kill` flag at `crates/sifr_codegen/src/stdlib_filter/implementation.rs:52-60`. Both the AST-based collector (`crates/sifr_codegen/src/stdlib_filter/implementation.rs:404-406`) and the text-scan fallback (`crates/sifr_codegen/src/stdlib_filter/implementation.rs:343`) detect `__sifr_process_async_kill` references.
- `is_shared_prelude_item` strips the kill helper from per-module dedup at `crates/sifr_codegen/src/stdlib_filter/implementation.rs:451` so it isn’t duplicated; the static table guard already covers `__SIFR_PROCESS_ASYNC_CHILDREN` and `__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID` at `crates/sifr_codegen/src/stdlib_filter/implementation.rs:428-429`.
- `lib_modules_and_codegen.rs:399-402` plumbs `needs_kill` through the per-module aggregation, and the consolidated `needs_process_async` predicate at `lib_modules_and_codegen.rs:429-435` now ORs in `needs_kill`. The preamble emission at `lib_modules_and_codegen.rs:606-614` forwards all seven flags into `build_process_async_items`, which is the sole site for the kill helper.
- `process_async_child_table_items(needs_spawn, needs_wait, needs_kill)` at `crates/sifr_codegen/src/preamble/process_async_runtime.rs:246-335` emits the `__SIFR_PROCESS_ASYNC_CHILDREN` table whenever any of the three lifecycle flags is set (correct: wait/kill both need to look it up), but emits the `__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID` allocator and `__sifr_next_process_async_child_id` only when `needs_spawn` is true. Without spawn, the table cannot be populated, so wait/kill lookups will deterministically miss with a typed error rather than panic — that gating is sound.
- Verified emission against the new fixture: the generated Rust contains `__SIFR_PROCESS_ASYNC_CHILDREN`, `__sifr_process_async_kill`, `__sifr_process_async_wait`, `__sifr_next_process_async_child_id`, and `start_kill`, and contains zero references to `__SIFR_PROCESS_CHILDREN`, `__sifr_process_kill`, `__sifr_process_terminate`, or `__sifr_process_spawn` (sync child table). This matches the ledger’s claim at `issues/...substrate-execution.md:1038`.

### Q7. Documentation honesty (fixture, manifests, host matrix, traceability, execution ledger)

PASS.

- Fixture: `crates/sifr/tests/e2e/pass/process_async_child_kill_wait.sifr` covers (a) top-level `async_wait(own child)` after method-form `kill`, (b) method-form `AsyncChild.wait()` after method-form `kill`, (c) typed `ProcessError` on a second `kill` once the entry has been removed by `wait`. That triple matches the production claims.
- Validation lane manifests: both `verification/validation_lanes/create_pr_e2e_manifest.json:94` and `verification/validation_lanes/merge_e2e_manifest.json:109` list `process_async_child_kill_wait` adjacent to `process_async_spawn_wait`. Both files parse as valid JSON per the user-supplied targeted validation run.
- Host matrix: `verification/platform/supported_host_matrix.md:24` adds a dedicated row marking Unix as `supported` and Windows as `host-limited` until a deterministic fixture and status mapping land. The row’s wording explicitly cites the new fixture.
- Traceability: `verification/stdlib/concurrency_runtime_m4_process_traceability.md:5,12,19,34,40,41` advances the M4 status line, lists `process_async_child_kill_wait` evidence under `Status`, expands the async lifecycle row to call out that `AsyncChild.kill()` preserves the handle while top-level borrowed `async_kill(child)` is intentionally not shipped pending borrowed-handle wrapper support, updates the CPython asyncio mapping row, and registers the fixture in both create-pr and merge lanes.
- Execution ledger: `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:1020-1039` records what was added, what was intentionally excluded (public async graceful terminate, public async owned pipes, cancellation-safe observation, process-group supervision, Windows status mapping), the targeted local validation outcomes, and the pending Claude Opus review entry. The deferral list mirrors the scope statement provided for this review.
- The two unrelated network/http review notes/ledger entries also in the working tree do not contaminate this slice’s claims; they are clearly scoped to the network/http substrate and are not referenced from the M4 process traceability or execution ledger sections touched by this wave.

### Q8. File-size guardrail (especially `process_async_runtime.rs`)

PASS.

- `crates/sifr_codegen/src/preamble/process_async_runtime.rs` is 830 lines (700 lines of new helper body and the surrounding fixtures already existed). 830 < 900.
- Other touched hand-maintained files (`stdlib_filter/implementation.rs`, `intrinsics/registry.rs`, `intrinsics/registry/process_async.rs`, `lib_modules_and_codegen.rs`, `sifr_stdlib/src/process.rs`, `lib/sifr/process.sifr`) remain well under the 900-line cap. The targeted run reports `2194 files checked, 900-line limit, PASS`.

## Non-Blocking Follow-Ups (Out Of Scope For This Slice)

These are intentionally deferred per the slice scope and the execution ledger. They are listed here only as a forward-pointing punch list and are not blockers:

- Top-level borrowed `async_kill(child)` wrapper once the stdlib async wrapper machinery can safely thread a borrowed class handle through an `Awaitable` return. The current method-form preserves the affine-handle invariant.
- Cancellation-safe async observation (e.g., aborting an in-flight `AsyncChild.wait()` future cleanly under structured cancellation).
- Async graceful terminate (Unix `SIGTERM` analogue to the sync `terminate`), public async owned pipes, scoped process supervision (`TaskGroup.spawn_process` -> `ProcessHandle`), and Windows-specific kill/terminate status mapping.
- Subprocess text-mode closeout once text/i18n M1 lands.
- Possible future hardening: a dedicated regression that asserts a second `kill()` *before* any `wait()` still succeeds (today `start_kill` is idempotent on a still-tracked handle, so this would just lock in current behavior; the current fixture only covers second-kill-after-wait).
- The previously recorded non-blocking suggestion from the sync-terminate review — narrowing the child-table mutex hold around any host signal request, and eventually replacing the host `kill` command path with a reviewed Rust host-signal dependency — remains forward-pointing for a later sync-lifecycle hardening wave; it is unaffected by this async slice because the async path uses `tokio::process::Child::start_kill` directly rather than shelling out.

## Summary

The slice is small, focused, internally consistent with the recently merged async spawn/wait wave, and faithful to its explicit scope deferrals. It honors the project core expectations: no panic on user-data, typed `ProcessError` with stable substring evidence, no Tokio types in public surface, no mutex guard across an await, no sync child table emission for async-only usage, and no premature top-level borrowed wrapper that the current stdlib wrapper machinery cannot keep sound.

Recommendation: merge.
