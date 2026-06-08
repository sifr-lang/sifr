# M4 stdin guardrails - review pass 1

Verdict: PASS.

Scope: blocker-oriented implementation review of the stdin-guardrail follow-up on branch `codex/concurrency-runtime-m4-stdin-guardrails`, following merged PR #2357 (sync stdin `PipeWriter`) and PR #2358 (pipe-writer merge ledger), with stale duplicate PR #2356 closed as superseded. The change set is intentionally narrow: prevent silent stdin misbehavior by making sync `spawn(command)` reject `Command.stdin_bytes(...)` and making async run/output/timeout reject non-inherit `Command.stdin(...)` modes with typed owned-pipe deferral errors until async owned pipes/communicate land.

## Evidence checked

1. `lib/sifr/process.sifr`:
   - `spawn(command)` raises a typed `ProcessError("process spawn does not consume Command.stdin_bytes; use stdin(Stdio(\"pipe\")) and Child.stdin()")` when `command.has_stdin_data` is set, before `process_spawn(...)` is invoked (lines 251-253).
   - `async_run(...)` passes 6 args ending in `command.stdin_mode` (line 377).
   - `async_run_timeout(...)` passes 7 args with `command.stdin_mode` immediately before `seconds` (lines 388-389).
   - `async_output(...)` passes 7 args with `command.stdin_mode` immediately before `command.has_stdin_data` (lines 400-401).

2. `crates/sifr_stdlib/src/process.rs`:
   - `process_async_run` declares 6 params: `program, args, env, cwd, has_cwd, stdin_mode` (lines 216-227).
   - `process_async_output` declares 7 params with `stdin_mode` before `has_stdin` (lines 229-243).
   - `process_async_run_timeout` declares 7 params with `stdin_mode` before `timeout_seconds` (lines 244-258).

3. `crates/sifr_codegen/src/intrinsics/registry/process_async.rs`:
   - `async_process_owned_args` now emits 6 owned arguments (positions 0-5), cloning the new `stdin_mode` at position 5 alongside the other string-typed args; `has_cwd` at position 4 remains a non-clone bool pass-through.
   - `lower_process_async_run` requires `args.len() == 6`.
   - `lower_process_async_run_timeout` requires `args.len() == 7` and appends `arg_expr(args, 6)` (timeout) as the final argument.
   - `lower_process_async_output` requires `args.len() == 7` and appends `arg_expr(args, 6)` (has_stdin) as the final argument.
   - The "timeout last" / "has_stdin last" ordering is preserved.

4. `crates/sifr_codegen/src/preamble/process_async_runtime.rs`:
   - `process_async_params` (lines 39-73) appends `stdin_mode: String` as the sixth positional helper param, with the optional 7th `has_stdin: bool` still gated by `include_stdin`; `process_async_timeout_params` extends that with `timeout_seconds: f64`.
   - `process_async_stdin_mode_guard()` (lines 75-90) emits the exact typed error string "async process stdin mode requires owned pipe support" inside a `return Err(ProcessError { message: ... })` early-return when `stdin_mode != "inherit"`.
   - Run helper body inserts the stdin-mode guard before command setup (line 214).
   - Run-timeout helper body inserts the stdin-mode guard before the finite/non-negative timeout-validation block, which is unchanged in shape and still emits `"process timeout must be finite and non-negative, got {}"` (lines 236-266).
   - Output helper body inserts the stdin-mode guard ahead of the existing `has_stdin` deferral, preserving the "async process stdin bytes require owned pipe support" message (lines 304-316).
   - Helper emission/gating in `build_process_async_items` is unchanged: each helper is still pushed only when its `needs_*` flag is set, and the helper signatures match the new param shapes (`process_async_params(false)` for run, `process_async_timeout_params()` for run_timeout, `process_async_params(true)` for output) (lines 466-498).

5. Fixtures:
   - `crates/sifr/tests/e2e/pass/process_spawn_pipe_writer.sifr` (lines 46-52) calls `Command("cat").stdin_bytes(b"ignored")` and expects sync `spawn(...)` to raise `ProcessError` whose message contains `"stdin_bytes"`. The outer `except ProcessError` still asserts `False`, so the new inner block must catch the error.
   - `crates/sifr/tests/e2e/pass/process_async_run_output.sifr` retains the existing `stdin_bytes(b"deferred") -> async_output` rejection (lines 25-31) and adds a `Stdio("pipe")` deferral case against `async_run` (lines 33-39), both asserting `"owned pipe"` in the message.
   - `crates/sifr/tests/e2e/pass/process_async_run_timeout.sifr` retains negative/overflow timeout validation (lines 19-31) and adds a `Stdio("pipe")` deferral case against `async_run_timeout(_, 2.0)` (lines 33-39), asserting `"owned pipe"` in the message; the existing happy-path timeout evidence at lines 6-17 is untouched.

6. Traceability and ledger:
   - `verification/stdlib/concurrency_runtime_m4_process_traceability.md` updates the `Command` row to state that sync `spawn(...)` rejects `stdin_bytes(...)` with a typed `ProcessError`, and the async-process row to state that async run/output/timeout return typed owned-pipe deferral errors for `stdin_bytes(...)` where applicable and for non-inherit `stdin(...)` modes. It explicitly leaves async spawn/wait/communicate, owned pipes, shell async APIs, async output timeout, streaming reads, cancellation, scoped supervision, and non-Unix signal-status as later M4 work - no overclaim of async owned pipes, async communicate, streaming, cancellation, scoped supervision, or Windows support.
   - `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` adds an "M4 stdin guardrails follow-up" block that records the stale-PR closure, the sync `spawn` typed error, the `stdin_mode` threading, the async typed deferral errors, the fixture extensions, and a targeted local-validation log with the exact evidence the user quoted in this review brief (cargo fmt, manifest JSON validation, cargo check, three fixtures, emission checks, cargo fmt --check, git diff --check, file-size guardrail at 2187 files, HIR guardrail, 423 fail tests, and `scripts/run_all_tests.sh --profile create-pr` PASS with `pass=5/skip=2` platform golden and `100 passed / 0 failed / cache_hits=24/26 / report_signature=458ad42c8c1b262c` for the create-pr e2e, with the warm-wall-time advisory called out honestly). The review-loop line is marked "Pending reviewer pass," which is consistent with this pass being written now.

7. File-size guardrail:
   - `crates/sifr_codegen/src/preamble/process_async_runtime.rs` is 501 lines, well under the 900-line cap.
   - The other touched first-party files (`lib/sifr/process.sifr` at 469, `crates/sifr_stdlib/src/process.rs` at 306, `crates/sifr_codegen/src/intrinsics/registry/process_async.rs` at 65) are also under the cap.
   - The validation log records 2187 files checked against the 900-line limit as PASS.

## Blockers

None.

## Non-blocking notes

- The async stdin-mode guard string ("async process stdin mode requires owned pipe support") and the pre-existing has_stdin guard string ("async process stdin bytes require owned pipe support") are intentionally distinct surfaces; the fixtures match both via the shared `"owned pipe"` substring assertion, which keeps the test stable if the messages are later harmonized into a single phrasing. Worth keeping the two messages distinct only so long as they describe genuinely different deferral causes; if a future change unifies them, the fixtures will still pass without edits.
- `Command.stdin_mode` is currently a plain `str` carried through the helpers, with the guard checking literal equality against `"inherit"`. That is fine for this slice; when async owned pipes land, the typed `Stdio` surface should drive the guard rather than string equality, otherwise drift between `Stdio.mode` and the helper-side literal could resurface silent misbehavior. Not a blocker for this PR.
- The "Pending reviewer pass" marker in the issue ledger should be updated to point at this review file (`reviews/ad-hoc-production-concurrency-runtime-m4-stdin-guardrails-review-pass-1.md`) once the reviewer-pass evidence is recorded, mirroring the prior review-loop entries that link to their PASS files.
