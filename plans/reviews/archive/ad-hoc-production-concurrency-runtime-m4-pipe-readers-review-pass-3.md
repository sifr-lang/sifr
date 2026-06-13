# M4 Sync stdout/stderr Pipe Readers — Review Pass 3 (post-PR #2350 rebase)

Verdict: **CHANGES_REQUESTED**

Scope under review: working-tree diff against `origin/main` on branch
`codex/concurrency-runtime-m4-owned-pipes` after the user-reported rebase
"onto current origin/main after PR #2350 / method-form blocking diagnostics
merged", including the unstaged ledger update in
`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`.

## Summary

All wave-scope invariants verified in pass 1 and pass 2 still hold under the
new rebase: sync-only stdout/stderr pipe readers, `Command.stdout` /
`Command.stderr` default inherit, `Child.stdout()` / `Child.stderr()`
one-shot `PipeReader` transfer, `PipeReader.read_all()` typed double-read
error, panic-free generated helpers with typed `ProcessError`, plain
`Command.output()` does not emit child/pipe-table helpers, file-size
guardrails honored, manifests carry `process_spawn_pipe_readers`, and the
traceability doc retains both `process_child_wait_method_direct_async_rejected`
and `process_child_kill_method_direct_async_rejected` while adding
`process_spawn_pipe_readers` to the `Command`, `Stdio`, `PipeReader`, and
`Sync spawn/wait/Child.wait` rows.

However, the rebase resolution silently dropped two evidence lines that
PR #2351 added to the method-form block on `origin/main`. The user's
acceptance criterion "The execution ledger should preserve the
method-diagnostics block, then the pipe-reader block, and record final
post-PR-2350 validation accurately" is therefore not met. This is the only
blocker.

## Blocker — ledger lost PR #2351 evidence lines

**File:** `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`

**Region:** `M4 method-form blocking workload diagnostics review loop:`
(current working tree at lines 782–784).

**On `origin/main`** (lines 782–785), the method-form review loop reads:

```
M4 method-form blocking workload diagnostics review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-process-method-workloads-review-pass-1.md`: `PASS`; reviewer verified qualified class-method workload collection, stdlib/project import propagation, method-call async diagnostics, bounded false-positive risk, new fail fixtures, and traceability honesty. Non-blocking note: keep unrelated network-phase files out of this PR.
- Merged as PR #2350: https://github.com/sifr-lang/sifr/pull/2350 (`cdfca07b19a6675463113c881525df620fa6eb44`).
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`343.95s`, warm target `<=2m`). Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`97 passed`, `0 failed`, `cache_hits=26/26`, `report_signature=36054c952f8fafec`).
```

**On the branch** (lines 782–784), only the pass-1 bullet remains; the
"Merged as PR #2350" and "Merge-ledger validation" bullets have been
removed. Confirmed two ways:

- `git diff origin/main -- issues/...md` shows both lines as deletions
  (`-`), with no replacement inside the method-form block.
- `grep -n "Merged as PR #2350"` against the working tree returns no
  match; the same grep against `origin/main` returns lines 784–785.

`git log --oneline cdfca07b1..origin/main` shows the branch is two
commits behind `origin/main`: `9b1b0a604 Record M4 process method
workload merge` (the commit that introduced those two lines via PR #2351)
and `399330465 Merge pull request #2351`. `git merge-base origin/main
HEAD` is `cdfca07b1` (the PR #2350 merge commit), not `origin/main` HEAD
`399330465`, so the branch is rebased onto the PR #2350 merge — *before*
PR #2351's evidence landed. When this PR merges, those two lines will
silently disappear from `main`, regressing the recorded merge of
PR #2350 and the merge-ledger validation evidence (run signature
`36054c952f8fafec`, `97 passed`, `cache_hits=26/26`).

This contradicts the user's stated acceptance criterion that the
method-diagnostics block be preserved and the final post-PR-2350
validation be recorded accurately.

### Required fix

Re-rebase the branch onto current `origin/main` (`399330465`) so the
"Merged as PR #2350" + "Merge-ledger validation" bullets from PR #2351
are picked up automatically, then re-resolve so the pipe-readers blocks
come *after* that preserved evidence. Equivalently, edit the working
tree to restore those two bullets at the end of the
`M4 method-form blocking workload diagnostics review loop:` section
(immediately after the pass-1 bullet, before the blank line that
precedes `M4 sync stdout/stderr pipe readers targeted local
validation:`), verbatim from `origin/main` lines 784–785.

After restoring, please rerun `scripts/run_all_tests.sh --profile
create-pr` so the final pipe-readers "Post-`origin/main` rebase rerun"
line reflects the actually-current rebase target (it currently
references "after the method-form blocking diagnostics merge", which is
the PR #2350 merge state, but the branch is in fact behind
`origin/main` by PR #2351 as well — the rerun text and the actual
rebase target should match).

## Wave invariants reverified (no new findings)

- **Rebase/conflict correctness with PR #2350 method diagnostics.**
  `verification/stdlib/concurrency_runtime_m4_process_traceability.md`
  status line lists PR #2350 method-form blocking diagnostics merged
  and adds "sync stdout/stderr pipe readers are in progress" without
  dropping prior entries. `Sync spawn/wait/Child.wait` row retains
  `process_child_wait_method_direct_async_rejected` and adds
  `process_spawn_pipe_readers`. `Sync kill, Child.kill` row retains
  `process_child_kill_method_direct_async_rejected`. Fail-suite lane
  retains both method-direct-async-rejected fixtures.
- **Sync-only.** No new async pipe surface, no `communicate`, no
  tokio `process` import on the sync path. Pipe table is
  `std::sync::Mutex` over `Box<dyn std::io::Read + Send>`.
- **`Command.stdout` / `Command.stderr` default inherit.**
  `lib/sifr/process.sifr:157–158` initialise both fields to `"inherit"`;
  `lib/sifr/process.sifr:178–182` overwrites only when callers invoke
  `Command.stdout(mode)` / `Command.stderr(mode)`.
- **`Child.stdout()` / `Child.stderr()` one-shot transfer.**
  `lib/sifr/process.sifr:108–122` returns `PipeReader(handle)` via
  `process_child_stdout` / `process_child_stderr`. Generated body in
  `crates/sifr_codegen/src/preamble/process_child_pipes.rs:86–176` uses
  `Option::take()` on the child's `stdout` / `stderr` field and returns
  a typed `ProcessError` on second call.
- **`PipeReader.read_all()` one-shot with typed error.**
  `lib/sifr/process.sifr:75–82` flips `_closed = True` *before*
  delegating to `process_pipe_read_all`; later calls raise
  `ProcessError("process pipe reader is already closed")`. Generated
  `__sifr_process_pipe_read_all`
  (`crates/sifr_codegen/src/preamble/process_child_pipes.rs:178–263`)
  removes the boxed reader inside a scoped `__pipes` block (mutex guard
  dropped before the IO call) and uses `std::io::Read::read_to_end` with
  `map_err` into `ProcessError`.
- **Panic-freedom.** No `panic!` / `unwrap()` / `expect(` on data paths
  in the new helpers — confirmed by reading
  `process_child_pipes.rs` and the new sections in
  `process_runtime.rs:247–528`. Poisoned-lock recovery uses
  `unwrap_or_else(|err| err.into_inner())`.
  `__sifr_process_stdio_from_mode` returns a typed
  `ProcessError { message: "unsupported process stdio mode: …" }` for
  unknown modes — no `unreachable!`.
- **Plain `Command.output()` does not emit child/pipe helpers.**
  Reproduced `cargo run -q -p sifr -- emit
  crates/sifr/tests/e2e/pass/process_sync_output_text.sifr | grep
  '__sifr_process_spawn|__sifr_process_stdio_from_mode|__SIFR_PROCESS_PIPE_READERS|__sifr_process_child_stdout|__sifr_process_child_stderr|__sifr_process_pipe_read_all'`
  -> 0 matches. Same grep on
  `process_spawn_pipe_readers.sifr` -> 16 matches. Filter gates in
  `crates/sifr_codegen/src/stdlib_filter/implementation.rs:311–328,
  349–367, 380–397` correctly key the shared process child bundle on
  the new spawn/pipe helper names.
- **Run fixture green.** `cargo run -q -p sifr -- run
  crates/sifr/tests/e2e/pass/process_spawn_pipe_readers.sifr` exits 0
  (cache hit; double-read produces `"closed"`, double child stdout
  produces `"already taken"`, status `kind="success"`).
- **File-size guardrail.** `process_runtime.rs` 853,
  `process_child_pipes.rs` 263, `registry/process.rs` 897 (unchanged
  carry-forward note from pass 1: next process-intrinsic wave must plan
  a split), `registry.rs` 712, `stdlib_filter/implementation.rs` 722,
  `process.sifr` 413, `sifr_stdlib/process.rs` 264 — all under the
  900-line cap.
- **Docs do not overclaim.** `verification/stdlib/concurrency_runtime_m4_process_traceability.md`
  `PipeReader` row explicitly preserves "stdin `PipeWriter`, streaming
  reads, async pipes, and sendability/shareability checks remain later
  M4 work"; follow-up boundaries retain bullets for `PipeWriter` +
  owned stdin lifecycle, async spawn/wait/communicate, graceful
  terminate, scoped `TaskGroup.spawn_process`, method-form
  `@blocking_io`, and Windows. `verification/platform/supported_host_matrix.md`
  marks Windows `host-limited` pending a deterministic Windows fixture.
- **Validation manifests.**
  `verification/validation_lanes/create_pr_e2e_manifest.json` and
  `merge_e2e_manifest.json` both add `process_spawn_pipe_readers`
  immediately after `process_spawn_wait_status`; both parse with
  `python3 -m json.tool`.

## Outstanding non-blocking notes carried from pass 1 / pass 2

Unchanged after this rebase, none turning blocking:

1. Helper duplication across `preamble/process_runtime.rs:249–328`,
   `preamble/process_child_pipes.rs:5–84`, and
   `intrinsics/registry/process.rs:45–108`
   (`process_error_expr`, `process_map_err`,
   `process_child_handles_lock_expr`, etc., redeclared three times).
   The parameterized `poisoned_lock_expr(static_name)` in
   `process_child_pipes.rs:31–52` would be a nice consolidation target
   in a follow-up.
2. `registry/process.rs` sits at 897 / 900 lines; the next process
   intrinsic must plan a responsibility split before adding code.
3. `Command.stdin_bytes(...)` is silently inert for `spawn` (the spawn
   intrinsic does not receive `has_stdin_data` / `stdin_data`).
4. `Stdio("garbage")` succeeds; the typed
   "unsupported process stdio mode: garbage" error only surfaces from
   `spawn`. `__sifr_process_stdio_from_mode` is the single
   normalization point, so this is a coverage/UX note, not a panic
   risk.
5. The shared process child bundle (pipe table, `PipeReader` struct,
   four new helpers) is now emitted for any program that calls
   `spawn` because `spawn` is keyed off the child handle table. Dead
   code in fixtures that do not call `.stdout()` / `.stderr()` /
   `.read_all()`; harmless (private items, optimised away by rustc).

## Coverage attestation

The wave's correctness, generated-code safety, panic-freedom, ownership,
gating, validation, and doc-overclaim invariants are intact. The only
blocker is the ledger regression in the method-form review loop, which
needs the two PR #2351 evidence lines restored (preferably by
re-rebasing onto current `origin/main` HEAD `399330465`) and the
post-rebase rerun line in the pipe-readers block re-recorded to match
the actually-current rebase target. Once those are restored, this wave
is ready to merge.

**CHANGES_REQUESTED** — restore the dropped PR #2351 evidence lines in
`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`
(and re-rebase so future merges don't regress them).
