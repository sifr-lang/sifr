# Ad Hoc M4 Async Process Spawn/Wait Review — Pass 4 (Blocker Closure)

Scope: re-review of the M4 async process spawn/wait wave after the pass-3 ledger blocker fix on `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`.

Reviewer date: 2026-06-08.

## Verdict

`PASS` — the pass-3 single blocker (duplicate `- M4 async process spawn/wait: in progress.` line in the implementation PR list) is fully closed. The PR list now reads in strict PR-number order, no conflict markers were introduced anywhere in the tree, `git diff --check` is clean, and no new blocker was introduced by the one-line fix. All other pass-1/pass-2/pass-3 evidence remains intact and is not contradicted.

## Pass-3 blocker — verified closed

`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:425-432` after the fix:

```
- M4 async process output timeout: https://github.com/sifr-lang/sifr/pull/2362
- M4 async stdin-byte communicate: https://github.com/sifr-lang/sifr/pull/2365
- M4 sync process terminate: https://github.com/sifr-lang/sifr/pull/2367
- M4 async process spawn/wait: in progress.
- M4: in progress.
- M5: pending.
- M6: pending.
- M7: pending.
```

- Exactly one `- M4 async process spawn/wait: in progress.` line remains, at line 428, sitting after the `#2367` row and before the `M4: in progress.` summary — the order pass 3 explicitly requested.
- `rg -n "M4 async process spawn/wait: in progress" issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` returns one match at line 428.
- PR-number ordering is preserved across the run-up entries (`#2362 → #2365 → #2367 → in progress`).
- The `M4: in progress.` summary on the next line still makes sense once the duplicate is collapsed.

## Unstaged-diff shape (one-line fix is the substantive change)

`git diff issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` shows two hunks:

1. `-` of the earlier duplicate `M4 async process spawn/wait: in progress.` line in the PR list (the exact pass-3 blocker fix).
2. `+` of one post-merge validation evidence line at `:978` recording the create-pr lane rerun after preserving PR #2367 sync terminate evidence: `wall_time=225.06s`, `103 passed`, `cache_hits=25/27`, `report_signature=2593463768412da4`.

The second hunk is the same evidence pass 3 already verified verbatim against the user-reported create-pr lane rerun (pass-3 review, `:62`); pass 3 inspected this evidence in the working tree, so its presence here is expected and is not a new claim. The wall-time, pass count, cache-hit ratio, and report signature all still match. No new validation claim is introduced.

## Conflict-marker / whitespace scan

- `rg -n "<<<<<<<|=======|>>>>>>>"` against `issues/`, `verification/`, and `reviews/` -> no matches.
- The only `=======` hits in the tree live under `third_party/ruff/` (typeshed `subprocess.pyi`, `statistics.pyi`, `pdb.pyi`, `ty_server` snap, and a numpy fixture) — vendored content unrelated to this branch and unchanged by the merge.
- `git diff --check` -> PASS (no whitespace or conflict-marker warnings).
- `git status` reports `MM` on the ledger file (the staged side carries the pass-3 baseline with the duplicate plus the sync terminate ledger; the unstaged side carries the one-line fix and the post-merge validation line). This is the expected mid-merge resolution shape.

## Code and evidence cross-checks (still honest after the fix)

The one-line fix is documentation-only and touches only the PR list. Everything verified in passes 1–3 is unaffected:

- Async spawn/wait surface (`lib/sifr/process.sifr:167-174, 416-431`), stdlib metadata (`crates/sifr_stdlib/src/process.rs:297-319`), intrinsic gating (`crates/sifr_codegen/src/intrinsics/registry.rs:628-635`), async lowerers (`process_async.rs`, 104 lines), and async runtime preamble (`process_async_runtime.rs`, 798 lines) are unmodified by this fix.
- Sync terminate surface from PR #2367 (`process_child_lifecycle.rs`, sync `__sifr_process_terminate` helper, traceability row, host-matrix row, and the `process_child_terminate_wait` plus both `process_terminate_direct_async_rejected` / `process_child_terminate_method_direct_async_rejected` fixtures) remains preserved end-to-end.
- The implementation, validation, and review-loop ledger blocks for both async spawn/wait (`:955-984`) and sync terminate (`:986-1014`) are intact. The PR-list fix does not touch them.
- File-size guardrail: `process.rs` 692, `process_async.rs` 104, `process_child_lifecycle.rs` 262, `process_runtime.rs` 699, `process_async_runtime.rs` 798 — all under the 900-line cap.
- Documentation honesty (no overclaim of async pipes, async kill/terminate, async cancellation-safe observation, scoped supervision, async shell, full text mode, Windows process support, or non-Unix terminate) carries forward unchanged.

## Residual non-blocking notes carried forward

Same as pass 3:

- AST-collector branches for the async spawn statics remain effectively dead under current codegen (mirrors the sync `__sifr_process_terminate` shape).
- Async wait does not directly exercise Unix signal-status flow; the new `process_child_terminate_wait` fixture extends sync signal evidence (SIGTERM + sync `wait` -> `signal == 15`) but is explicitly sync.
- `process_async_runtime.rs` at 798 lines remains the closest async preamble to the 900-line cap; a pre-cap responsibility split should be planned before the next async-process slice.
- Explicit `stderr(Stdio("pipe"))` rejection is still not directly fixtured (only `stdin` and `stdout` are); the runtime guard is a single symmetric boolean across all three modes.
- PR #2367's pass-1 non-blocking follow-ups (mutex held across host `kill` fork/exec/wait; shelling to `/bin/kill` rather than using a Rust signal binding) remain open as PR-#2367 follow-ups, not async-spawn/wait blockers.

## Bottom line

The pass-3 blocker is fully closed by the one-line deletion. The PR list is in strict PR-number order with exactly one in-progress spawn/wait entry after `#2367`. No new blocker was introduced. The merge can now be concluded.
