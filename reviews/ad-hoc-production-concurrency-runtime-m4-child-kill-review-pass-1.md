# M4 Sync Child Kill Review — Pass 1

Verdict: **PASS** (with non-blocking follow-ups)

## Scope verified

This wave adds sync forceful child termination only:

- `lib/sifr/process.sifr:62-72,157-159` exposes `Child.kill()` and top-level `kill(child)` as `@blocking_io` returning `Result[None, ProcessError]`. No `terminate`, timeout, signal, or cancellation surface is claimed.
- `_sifr.process` registers `process_kill(handle: int) -> Result[None, ProcessError]` (`crates/sifr_stdlib/src/process.rs:52-58`).
- `crates/sifr_codegen/src/intrinsics/registry/process.rs:531-581` lowers `process_kill` using `__SIFR_PROCESS_CHILDREN.lock()...get_mut(&__handle).ok_or_else(...)` and then `__child.kill().map_err(...)`. The handle is **not** removed — only `wait` consumes the entry. This matches the requested "kill preserves child for later `wait`" contract.
- `crates/sifr_codegen/src/intrinsics/registry.rs:601` registers the new lowerer in alphabetical order.
- New fixtures `crates/sifr/tests/e2e/pass/process_child_kill_wait.sifr` and `crates/sifr/tests/e2e/fail/process_kill_direct_async_rejected.sifr`.
- Validation lane manifests gain `process_child_kill_wait` in both `create_pr_e2e_manifest.json:89` and `merge_e2e_manifest.json:104`. The fail fixture is recognized by the lex-discovered fail suite (no manifest edit needed; brief confirms 419 fail tests).
- Traceability (`verification/stdlib/concurrency_runtime_m4_process_traceability.md:17`) adds a dedicated `Sync kill, Child.kill` row that explicitly defers graceful `terminate`, timeout escalation, structured cancellation, and host-specific signal evidence; the follow-up boundaries list refines the open item from "`terminate`, `kill`" to "graceful `terminate`, termination escalation, signal termination evidence, …".

## Concrete blocker checks

1. **Honest sync forceful slice.** No claim of `terminate`, timeout escalation, structured cancellation, or signal evidence in either the public process surface or the traceability table. The traceability row states "forceful child termination through `std::process::Child::kill`; callers must still observe the final status with `wait`" and explicitly defers the rest. ✅
2. **Typed `ProcessError` for closed/unknown handles, no user-path panics.** Generated code for `kill` (see `emit` of `process_child_kill_wait.sifr`):
   - Handle lookup: `__children.get_mut(&__handle).ok_or_else(|| ProcessError { message: format!("process child handle is closed or unknown: {}", __handle) })?`
   - `std::process::Child::kill` failure: `.map_err(|e| ProcessError { message: e.to_string() })?`
   - The only `unwrap*` calls are `unwrap_or_else(|__err| __err.into_inner())` (mutex poison recovery; cannot panic) and `unwrap_or(-1)` on absent exit codes. No data-dependent `unwrap`/`expect` on user input. ✅
3. **Kill preserves handle for later `wait`.** `lower_process_kill` uses `get_mut` (not `remove`); `lower_process_wait` (`process.rs:465-529`) is the only path that `remove`s. The pass fixture exercises this exactly — `kill(child)` then `wait(child)` then `kill(child)` again, expecting the third call to fail with `closed or unknown`. Confirmed in the emitted Rust and `cargo run` succeeds. ✅
4. **Direct async `kill(child)` triggers SIFR-ASYNC-0003.** Verified locally — `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_kill_direct_async_rejected.sifr` emits `error[SIFR-ASYNC-0003]: blocking_io function 'kill' called directly from async context`. The mechanism is imported workload metadata on the top-level `@blocking_io kill(child)` definition in `lib/sifr/process.sifr:157-159`, consistent with the existing wait/blocking fail fixtures. The method form `Child.kill()` is intentionally not gated (parallel to `Child.wait()`); the traceability says only "Top-level `kill(child)` is `@blocking_io` and direct async calls are rejected," so the asymmetry is documented, not overclaimed. ✅
5. **Preamble/runtime gating intact.** `__SIFR_PROCESS_CHILDREN` and `__sifr_next_process_child_id` are emitted only when the stdlib filter detects references in the consumed stdlib code (`crates/sifr_codegen/src/stdlib_filter/implementation.rs:264-336`, `lib_modules_and_codegen.rs:574-576`). The new `process_kill` lowering uses the same identifiers, so any module that imports `kill` or `Child.kill` pulls the table in; modules that only use output-style process APIs (e.g., `run_command`, `output`) continue not to emit the child table. Confirmed by emitting the new fixture (single `__SIFR_PROCESS_CHILDREN` static, no duplication) and noting `process_runtime_and_platform.sifr` imports `sifr.os` not `sifr.process`. ✅
6. **No data-dependent panics introduced elsewhere.** Grep of generated code for the kill fixture shows only the safe `unwrap*` patterns above. The new lowerer adds no `unwrap`/`expect` on a user-controlled value. ✅
7. **Docs honest about portability.** The traceability names `std::process::Child::kill` explicitly (which has documented host-specific semantics: SIGKILL on Unix, `TerminateProcess` on Windows). Signal termination evidence and supported-host matrix updates are explicitly listed as remaining M4 follow-up work in the follow-up boundaries. ✅

## Fixture robustness (item 5 in brief)

`process_child_kill_wait.sifr` spawns `sh -c "sleep 5"`. The functional behavior is fine: `kill` returns `Ok(None)` regardless of whether `sleep` becomes orphaned, `wait` then observes the killed `sh` (or, more commonly, the exec-optimized `sleep`) and the test asserts only `not status.success` and `status.kind == "nonzero"`. On the supported hosts in practice, POSIX shells (`bash` on macOS in `/bin/sh`, `dash` on Linux) exec the single command following `-c`, so `Child::kill()` lands on the `sleep` process directly and no orphan is created. This matches the existing convention used by `process_spawn_wait_status.sifr` (`sh -c "exit 7"`), so I am not treating it as a blocker. See non-blocking follow-up #1 for an optional hardening.

## Empirical verification done in-review

- Inspected diff of all 8 touched files; no incidental edits.
- Read the generated Rust for `process_child_kill_wait.sifr` (via `sifr emit`) and confirmed:
  - `fn kill(child: &Child) -> Result<(), ProcessError>` uses `get_mut` and propagates `ok_or_else(...)` and `.map_err(...)?` — no panics.
  - `Child::kill(&self)` does the same.
  - `__SIFR_PROCESS_CHILDREN` and `__sifr_next_process_child_id` appear exactly once.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_child_kill_wait.sifr` → exit 0 (cache hit).
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_kill_direct_async_rejected.sifr` → expected `SIFR-ASYNC-0003`.

Brief-supplied evidence (`cargo check`, full fail suite 419/419, `cargo fmt --check`, file-size + HIR guardrails, `scripts/run_all_tests.sh --profile create-pr` with 94/0 e2e pass) is internally consistent with the diff.

## Non-blocking follow-ups

1. **Test fixture hardening.** Prefer `Command("sleep")` with `args(["30"])` over `sh -c "sleep 5"`. This removes a layer of shell-fork-exec semantics entirely (no dependence on the host shell's single-command exec optimization) and eliminates the theoretical "sleep orphan briefly running on the host" tail risk on shells that don't optimize. Functionally a no-op for the current assertions.
2. **Doc precision on kill scope.** The traceability row could note (in one phrase) that `std::process::Child::kill` targets only the immediate child handle and does not propagate to descendant processes (no process-group / supervision semantics) — this is implicit from the named API but explicitly stating it would forestall future readers reading "forceful child termination" as transitive. Pair this with the existing "Scoped process supervision entry point accepted by M0" follow-up bullet so the contrast is visible.
3. **Method-form async diagnostic gap.** `Child.kill()` (and `Child.wait()` from the prior wave) is not subject to `SIFR-ASYNC-0003` because the workload-metadata gate is only wired for the top-level function. The traceability is honest about this slice, but the asymmetry between top-level and method-form `@blocking_io` is a real lifecycle wave concern. Worth carrying as a tracked follow-up (e.g., "method-form `@blocking_io` enforcement" under the structured cancellation / supervision work).
4. **Pre-PR housekeeping.** `reviews/ad-hoc-production-concurrency-runtime-m4-child-kill-review-pass-1.md` was committed as an empty placeholder; this pass now fills it. Confirm the issue-execution doc's "M4 sync child kill review loop" entry is updated to reference this review and the eventual PR # / merge SHA before/after merge, mirroring the prior wave's pattern.

## Verdict

**PASS.** The wave is a clean, honest sync forceful-kill slice. The implementation respects all the stated guardrails (typed errors, preserved handles, gated preamble, imported async metadata), and the surface/docs do not overclaim beyond `std::process::Child::kill`. Recommend addressing follow-up #1 opportunistically before PR, but it is not a blocker.
