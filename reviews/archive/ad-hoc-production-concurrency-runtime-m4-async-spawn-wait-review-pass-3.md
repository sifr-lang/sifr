# Ad Hoc M4 Async Process Spawn/Wait Review — Pass 3 (Post-Merge)

Scope: re-review of the M4 async process spawn/wait wave after merging `origin/main` (which had picked up PR #2367 sync process terminate) into PR #2369's branch and resolving the resulting conflicts.

Reviewer date: 2026-06-08.

## Verdict

`CHANGES_REQUESTED` — the merge resolution is overwhelmingly correct: pass-1/pass-2 implementation, generated runtime, fixtures, manifests, host matrix, traceability, and review-loop entries are all intact and not contradicted by PR #2367's evidence; PR #2367's traceability row, host-matrix row, fixture-coverage, follow-up-boundary edit, implementation/validation/review-loop ledger blocks, and merge record are all preserved. However the PR-list section at the top of the execution ledger contains one stale duplicate line that contradicts itself and is unambiguously a merge artifact. It needs to be removed before this branch is ready.

## The single blocker

`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:425-429`:

```
- M4 async process output timeout: https://github.com/sifr-lang/sifr/pull/2362
- M4 async stdin-byte communicate: https://github.com/sifr-lang/sifr/pull/2365
- M4 async process spawn/wait: in progress.
- M4 sync process terminate: https://github.com/sifr-lang/sifr/pull/2367
- M4 async process spawn/wait: in progress.
```

The `- M4 async process spawn/wait: in progress.` line appears twice (lines 427 and 429) sandwiching the `#2367` entry. Both sides of the merge had added an `in progress` line for spawn/wait (origin/main pre-#2367 and this branch pre-merge), and the conflict resolution kept both rather than collapsing them. The intended chronological-by-PR-number order is:

- `M4 async stdin-byte communicate: #2365`
- `M4 sync process terminate: #2367`
- `M4 async process spawn/wait: in progress.`

Fix: delete the line at `:427` (or `:429`, either occurrence) so a single in-progress spawn/wait entry sits after the `#2367` row. No other change is needed in that section. Once collapsed, the PR list will read in PR-number order and the `M4: in progress.` summary on the next line continues to make sense.

This is the only blocker. Everything below is verified-OK after the merge.

## Conflict-marker scan

`git grep -nE '^(<<<<<<<|=======|>>>>>>>)'` returns no matches anywhere in the tree, and `git diff --check` is clean per the prompt's reported run. No raw conflict markers leaked through.

## PR #2367 sync terminate evidence — preserved end-to-end

- Implementation PR list: `issues/...-execution.md:428` carries `- M4 sync process terminate: https://github.com/sifr-lang/sifr/pull/2367` (the duplicate adjacent to it is the spawn/wait blocker above, not this entry).
- Traceability status header (`verification/stdlib/concurrency_runtime_m4_process_traceability.md:5`): explicitly lists `sync terminate merged in PR #2367` in chronological PR order; the same line still ends with `async spawn/wait is in the current implementation wave`, so both waves are honestly represented.
- Traceability surface row (`:23`): dedicated `Sync terminate, Child.terminate` row, fixture-referenced (`process_child_terminate_wait`, plus both async-rejection fixtures), Unix-SIGTERM-only scope and non-Unix typed-unsupported deferral both called out.
- Traceability surface row delta to `Sync spawn/wait/Child.wait` (`:21`) and `Sync kill/Child.kill` (`:22`): "async termination" replaces the prior "termination" in the open-work tail for the wait row, and the kill row drops the now-resolved "graceful terminate" follow-up — both consistent with PR #2367 having shipped sync `terminate`.
- Traceability "Imported workload metadata" row (`:26`): adds `process_child_terminate_method_direct_async_rejected` alongside the existing kill/wait method async-rejected entries.
- Traceability CPython family mapping (`:32`): augments the `Lib/test/test_subprocess.py` mapping with `POSIX terminate evidence` and references `process_child_terminate_wait`.
- Traceability validation coverage (`:40-42`): both create-PR and merge lanes list `process_child_terminate_wait` after `process_child_kill_wait` (lexicographic neighbour). Fail suite lists both new `process_terminate_direct_async_rejected` and `process_child_terminate_method_direct_async_rejected` between `process_child_kill_method_direct_async_rejected` and `process_pipe_writer_method_direct_async_rejected`.
- Traceability follow-up boundaries (`:50, :53`): "Graceful `terminate`" is removed from the open work list, replaced with "Termination escalation, non-Unix signal status evidence, ..., non-Unix process termination behavior" and "termination escalation" (instead of "termination") in the drop/cleanup bullet. No overclaim — Windows termination is still listed as deferred.
- Supported host matrix (`verification/platform/supported_host_matrix.md:24`): dedicated `Sync subprocess graceful terminate` row at `supported`/`supported`/`host-limited`, referencing `process_child_terminate_wait` and `Status(kind="signal", signal=15)`. Windows row stays host-limited with a documented escape hatch.
- Manifests: `verification/validation_lanes/create_pr_e2e_manifest.json:98` and `merge_e2e_manifest.json:113` each add `process_child_terminate_wait` immediately after `process_child_kill_wait`; both files remain valid JSON (per the prompt's reported `python3 -m json.tool` PASS).
- Implementation ledger block (`:986-992`), validation block (`:994-1008`), review-loop block (`:1010-1014`), and merge-ledger record are all present and unmodified by this merge.

## Async spawn/wait evidence — preserved end-to-end (pass-2 baseline still honest)

- Public surface (`lib/sifr/process.sifr:167-174, 416-431`): `AsyncChild` still carries only `_handle: int` (pass-2 cleanup intact), `async_spawn` forwards 9 args including `stdin_mode`/`stdout_mode`/`stderr_mode`/`has_stdin_data`, `async_wait` consumes `own child` and forwards `child._handle`, and `AsyncChild.wait` delegates to `process_async_wait(self._handle)`.
- Stdlib metadata (`crates/sifr_stdlib/src/process.rs:297-319`): `process_async_spawn` keeps the 9-arg `Awaitable[Result[AsyncChild, ProcessError]]` signature, `process_async_wait` keeps the 1-arg `Awaitable[Result[Status, ProcessError]]` signature. `process_async_child_class()` still declares `fields: [("_handle", Int)]` only — no metadata regression from the merge.
- Intrinsic registry (`crates/sifr_codegen/src/intrinsics/registry.rs:628-635`): `process_async_spawn` and `process_async_wait` are still gated `Some(StdlibFeature::Tokio)` and dispatch to `process_async::lower_process_async_spawn`/`lower_process_async_wait` (unchanged module). The new `process_child_lifecycle` module that PR #2367 introduced replaces `process::lower_process_spawn`/`kill`/`wait` (`:599-602`) but does not touch the async lowerers — clean separation.
- Async lowerers (`crates/sifr_codegen/src/intrinsics/registry/process_async.rs`): 104 lines, untouched by the merge. Spawn still requires 9 args and clones stdout/stderr modes; wait still requires 1 arg and boxes `__sifr_process_async_wait(handle)`.
- Generated async runtime (`crates/sifr_codegen/src/preamble/process_async_runtime.rs`): 798 lines, unchanged. `__SIFR_PROCESS_ASYNC_CHILDREN`, `__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID`, `__sifr_next_process_async_child_id`, `__sifr_process_async_spawn`, and `__sifr_process_async_wait` are still emitted with the same gating, the same poison-recovery, and the same drop-mutex-before-await wait ordering pass 1 verified.
- Shared-prelude classification (`crates/sifr_codegen/src/stdlib_filter/implementation.rs:336-343, 396-404, 444-446`): async spawn/wait paren-suffix scans, AST collector branches, and `is_shared_prelude_item` predicates all intact. The merge adds `__sifr_process_terminate` references at `:324, :371, :431` to the sync `process_children` group — that group is the same shared static set used by sync spawn/kill/wait and pipe helpers, so terminate gets co-emitted exactly when those do; this matches the sync child-table lifecycle and does not pollute the independently-gated async helpers.
- Async spawn/wait fixture (`crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr`): the pass-2 8-assertion shape is intact (nonzero `async_spawn`+`async_wait`, success method-form `AsyncChild.wait()`, second-wait `"closed or unknown"`, `stdin_bytes` rejection, `stdin(Stdio("pipe"))` rejection, `stdout(Stdio("pipe"))` rejection). The fixture file was not touched by the merge.
- Manifests still list `process_async_spawn_wait` at `create_pr_e2e_manifest.json:93` and `merge_e2e_manifest.json:108` — adjacent neighbour `process_child_terminate_wait` is added later in the list under the `process_child_*` group rather than disturbing the async-block ordering.
- Async spawn/wait host-matrix row (`supported_host_matrix.md:23`): unchanged from pass 2; the new sync-terminate row was inserted at `:24` between it and the existing `Subprocess signal status evidence` row, preserving lexical/scope grouping.
- Async spawn/wait ledger blocks (`issues/...-execution.md:955-984`): implementation, validation, and review-loop blocks remain. The post-merge validation evidence line at `:979` quoting `wall_time=225.06s`, `103 passed`, `cache_hits=25/27`, `report_signature=2593463768412da4` matches the user-reported create-pr lane rerun verbatim.

## Code-level merge cross-checks

- Async-runtime panic surface: untouched. `__sifr_process_async_spawn` still rejects `has_stdin` then non-`"inherit"` stdio modes before constructing the Tokio command; `__sifr_process_async_wait` still maps a missing table entry to a typed `ProcessError` and drops the mutex guard before `.await`. Lock-poison fallback is still `unwrap_or_else(|err| err.into_inner())`. No `.unwrap()`/`.expect()` introduced on data-dependent values.
- Sync-side panic surface (new from #2367): `__sifr_process_terminate` Unix branch (`crates/sifr_codegen/src/preamble/process_runtime.rs:407-512`) maps `std::process::Command::new("kill").arg("-TERM").arg(&__pid).status()` failure through `process_map_err`, and explicitly returns `Err(process_error_expr(format!("process terminate failed with status: {}", __status)))` when the host `kill` exits non-zero. Non-Unix branch (`:514-532`) returns a typed unsupported `ProcessError`. No panic path introduced.
- Helper-gating regression check: `__sifr_process_terminate` is added to `build_process_child_items` (`process_runtime.rs:688-691`), which is already gated by the sync `process_children` need flag — i.e. terminate only emits when at least one sync child API (`spawn`/`kill`/`wait`/`terminate`/pipe helpers) is reachable. The async spawn/wait need flags are independent (`needs_spawn`/`needs_wait` on the async branch), so a user that only invokes `async_spawn`/`async_wait` does not pull in the sync terminate helper, and vice-versa. Verified via the AST collector arms at `:365-380` (sync set) and `:396-404` (async set) being disjoint.
- Argument-order check: `lower_process_terminate` (`process_child_lifecycle.rs:254-262`) forwards a single `int` to `__sifr_process_terminate`, matching the stdlib metadata at `process.rs:182-186`. No mismatch.
- File-size guardrail: `process.rs` 692 lines, `process_async.rs` 104, `process_child_lifecycle.rs` 262, `process_runtime.rs` 699, `process_async_runtime.rs` 798 — all under the 900-line cap. The pass-1/pass-2 note about `process_async_runtime.rs` approaching the cap still applies; the merge did not move it.
- Direct-async fail fixtures: `crates/sifr/tests/e2e/fail/process_terminate_direct_async_rejected.sifr` and `crates/sifr/tests/e2e/fail/process_child_terminate_method_direct_async_rejected.sifr` each carry `# expect-error: SIFR-ASYNC-0003`, import only sync `terminate`/`Child`, and call it inside `async def stop(...)`. Symmetric with the existing `kill`/`wait` direct-async-rejected fixtures.

## Documentation honesty after merge

After the merge the docs still:

- Do not claim public async pipes (`Command.stdin/stdout/stderr` with `"pipe"` mode is still typed-deferred in both surface rows).
- Do not claim async `kill`/`terminate` (traceability row 19 explicitly lists "Async kill/terminate ... remain later M4 work"; row 23 only claims sync terminate; follow-up bullet 2 lists "async kill/terminate" as open).
- Do not claim cancellation-safe async observation (still open in row 19 and follow-up bullet 2).
- Do not claim scoped process supervision (open in row 19, follow-up bullet 3).
- Do not claim async shell APIs (open in row 18).
- Do not claim full text-mode closeout (open in follow-up bullet 4).
- Do not claim Windows process support (host-matrix rows 19-24 keep Windows at `host-limited`; row 24 specifically calls out "Windows terminate semantics remain host-limited until a deterministic Windows fixture and status mapping are added"; follow-up bullet 3 calls out non-Unix signal-status evidence).
- Do not claim non-Unix terminate (row 23 explicit on non-Unix typed-unsupported deferral).

No overclaim was introduced by the merge.

## Pre-merge residual notes (still accurate, still non-blocking)

Carried forward from passes 1–2:

- AST-collector branches for the async spawn statics/`__sifr_next_process_async_child_id` remain effectively dead under current codegen (matches the sync precedent, which after the merge also includes `__sifr_process_terminate` in the same dead-branch shape).
- Async spawn/wait fixture does not directly exercise Unix signal-status flow through async wait; sync signal-status evidence still lives in `process_signal_status`. The new `process_child_terminate_wait` fixture extends sync signal evidence (it asserts `signal == 15` after SIGTERM + sync `wait`) — useful new evidence, but explicitly sync, so it does not change the async-side gap.
- `process_async_runtime.rs` at 798 lines remains the closest async preamble file to the 900-line cap. The pre-cap responsibility split should still be planned before the next async-process slice rather than after.
- Explicit `stderr(Stdio("pipe"))` rejection is still not directly fixtured (only `stdin` and `stdout` are). The runtime guard is a single symmetric boolean across all three modes.
- PR #2367's review pass-1 already noted non-blocking follow-ups for the new sync terminate helper (mutex held across the host `kill` fork/exec/wait; shelling to `/bin/kill` rather than using a Rust signal binding). Those are PR-#2367 follow-ups, not async-spawn/wait blockers, but they are now part of the merged sync-terminate surface this branch must continue to live alongside.

## Bottom line

The merge correctly preserves both wave's evidence and does not regress any code-level guarantee verified in passes 1–2. The only blocker is the duplicate `- M4 async process spawn/wait: in progress.` line in the implementation-PR list at `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:427` / `:429` — collapsing it to a single occurrence after the `#2367` entry will close this review.
