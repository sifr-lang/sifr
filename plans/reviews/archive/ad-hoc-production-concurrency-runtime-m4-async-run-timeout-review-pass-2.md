RESULT: PASS

Pass-2 review after the origin/main merge (sync pipe reader work) and the file-size-driven split of async preamble helpers from `process_runtime.rs` into `process_async_runtime.rs`.

## Scope re-verified vs. origin/main

- Diff against `origin/main` covers three commits: `cbdf6360d` (timeout core), `3f81eaa1a` (merge from main bringing sync pipe readers), and `19d4d049e` (split).
- The split commit moves `process_async_params`, `process_async_command_setup`, `status_code_expr`, `status_signal_expr`, `process_status_from_parts`, `process_async_ret`, and `build_process_async_items` verbatim out of `process_runtime.rs` into the new `process_async_runtime.rs` and re-exports it via `crates/sifr_codegen/src/preamble.rs`. No behavioral change in that commit. `process_runtime.rs` is now 511 lines (was over the 900-line cap after the merge), `process_async_runtime.rs` is 472 lines; both under the guardrail.
- The merge from main added the sync pipe reader code (`process_child_pipes.rs`, expanded `process_runtime.rs` items, intrinsic registry rows, stdlib metadata). None of those paths were re-touched after the merge except for the split, so the M4 pipe reader behavior brought in from PR #2352 is preserved as-is.

## Correctness re-verifications

Re-emitted the timeout fixture with `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_async_run_timeout.sifr` and confirmed:

- The 6-arg `__sifr_process_async_run_timeout` is emitted with the expected body: `is_finite()` + `< 0.0` guard, `Duration::try_from_secs_f64` with `map_err` to `ProcessError`, full `Command` setup (program, args, env split on `=`, optional `cwd`), `__cmd.spawn()`, then `tokio::select! { biased; __child.wait() => ...; tokio::time::sleep(__duration) => __child.kill().await? + Status with kind="timeout", timed_out=true, success=false }`.
- The shared-preamble disambiguation works: only `__sifr_process_status_from_exit`, `__sifr_process_exit_signal`, and `__sifr_process_async_run_timeout` are emitted for the timeout-only fixture. The async run (5-arg) and async output helpers are not emitted, confirming `derive_shared_needs_text_scan`'s `"__sifr_process_async_run("` (trailing-paren) probe and the syn visitor's exact ident match correctly separate `_run` from `_run_timeout`. The `SharedPreludeProcessAsyncNeeds` split into `needs_run` / `needs_run_timeout` / `needs_output` flows through `build_process_async_items(..)` so only the requested helper is emitted.
- Kill+reap behavior on timeout: `tokio::process::Child::kill().await` performs `start_kill()? + self.wait().await?`, so the child is reaped before the `Status` is returned. The `biased` keyword ensures completion races wait first, avoiding sleeping-past-completion. No user-triggerable panics in any branch: every conversion uses `try_from_secs_f64` / `map_err`, status assembly uses safe field assignment, and `code().unwrap_or(-1) as i64` is sound.
- Typed `Status` matches the public contract: `kind == "timeout"`, `timed_out == true`, `success == false`, `code == -1`. The redundant `__timeout_status.success = false;` is harmless (`Status::new(-1, ...)` already sets `success = false`) and mirrors the sync timeout helper for consistency.
- Type alignment: stdlib metadata registers 6 params (`Float` for `timeout_seconds`), the `.sifr` `def async_run_timeout(... seconds: float)` arity matches `if args.len() != 6` in `lower_process_async_run_timeout`, and the lowering clones the first four (`String` / `Vec<String>`) owned args, takes `has_cwd: bool` and `timeout_seconds: f64` by value (both `Copy`).

## Non-blocking notes

1. **Duplicated helpers between `process_runtime.rs` and `process_async_runtime.rs`** — `string_ty`, `string_vec_ty`, `process_error_expr`, and `process_map_err` exist as identical copies in both files. The split was a clean responsibility-based extraction, but a follow-up could lift these four helpers into a shared module (e.g. `preamble/process_common.rs`) to avoid drift. Not a blocker — both copies are private, used only inside their own module, and identical at the moment.
2. **Raw multi-line Rust string injection via `RustExpr::Ident(...)`** in `process_async_runtime.rs:256-275` — flagged in pass-1; established pattern (mirrors `preamble/task_runtime.rs:275, :888`), accepted by rustc, output is not user-facing. Unchanged here.
3. **Test coverage gap: NaN seconds** — flagged in pass-1; `1e30` trips the `try_from_secs_f64` branch rather than `!is_finite()`, so the explicit `!is_finite()` path remains uncovered by fixtures. Not a blocker because both rejection paths produce a `ProcessError`.

## Regressions from the merge

None observed. The merged sync pipe reader code (`process_child_pipes.rs`, `process_pipe_read_all` runtime, `Child.stdout()/stderr()` intrinsics) lives in its own files / sections of `process_runtime.rs` and is not exercised by the async timeout helper. The traceability doc and both validation lane manifests (`create_pr_e2e_manifest.json`, `merge_e2e_manifest.json`) list `process_async_run_timeout` alongside the pipe reader entry.

## Required follow-up validation

None. The reported `scripts/run_all_tests.sh --profile create-pr` run (wall 571.32s, e2e signature `42aaf1077a936d74`, 99/99 pass, file-size guardrail green over 2184 files) is sufficient. PR #2354 is ready to merge from a reviewer perspective.
