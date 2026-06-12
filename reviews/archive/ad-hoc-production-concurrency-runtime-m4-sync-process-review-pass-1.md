# Ad Hoc Production Concurrency Runtime — M4 Sync Process Foundation Review (Pass 1)

Result: **PASS**

Reviewer scope: M4 sync process foundation wave. Reviewed branch `codex/concurrency-runtime-m4-process` (working tree, against `main`).

## Verdict Summary

The M4 sync process foundation implements an honest, small, reviewable slice of `sifr.process`:

- New native stdlib module `lib/sifr/process.sifr` exposes `Command`, `Status`, `Output`, `TextOutput`, `ProcessError`, `Stdio`, plus six top-level callables (`run`, `output`, `output_text`, `run_shell`, `output_shell`, `output_shell_text`).
- `_sifr.process` intrinsics (`crates/sifr_stdlib/src/process.rs`) and registry lowerers (`crates/sifr_codegen/src/intrinsics/registry/process.rs`) emit native `std::process::Command` calls for argv APIs and explicit `sh -c` for shell APIs.
- A new `shell_exec` workload kind is added (`crates/sifr_lowering/src/lower/workload_annotations.rs`) along with `SIFR-ASYNC-0007`.
- Imported workload metadata now flows through `ExternalDefs.function_workloads`, `imports::resolve_imports_early`, and stdlib/user-module import paths in `mod_impl.rs`, propagating `@blocking_io` and `@shell_exec` classifications across module boundaries.
- E2E coverage: three pass fixtures (`process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output`) and two fail fixtures (`process_blocking_direct_async_rejected` → `SIFR-ASYNC-0003`, `process_shell_exec_direct_async_rejected` → `SIFR-ASYNC-0007`). Added to both create-pr and merge manifests.
- Traceability doc (`verification/stdlib/concurrency_runtime_m4_process_traceability.md`) honestly distinguishes the implemented foundation from deferred M4 work (spawn/Child/wait/pipes, async process, timeout/terminate/kill, scoped process supervision, richer text-mode/encoding integration).

## Reviewer Checks

### 1. Ordinary argv-style APIs lower to `std::process::Command` without shell or legacy helper

Verified by reading `crates/sifr_codegen/src/intrinsics/registry/process.rs` and emitting `crates/sifr/tests/e2e/pass/process_sync_output_text.sifr`:

- `normal_command_setup` builds `let mut __cmd = std::process::Command::new(&command.program);`, then iterates `command.arguments` and `command.env_vars` (splitting at `=` via `split_once('=')`), and conditionally calls `current_dir(&command.working_dir)`.
- `lower_process_run` / `lower_process_output` / `lower_process_output_text` route through `normal_command_setup`, then `cmd.status()` or `spawn + wait_with_output` with piped stdio.
- Emitted ordinary path contains `std::process::Command`, `Stdio::piped`, and `split_once('=')`; no reference to `subprocess_run*`, `__sifr_sys_subprocess`, `getoutput`, or any legacy helper. Independently re-verified with `cargo run -- emit ... | grep`.

### 2. Shell APIs are opt-in and classified `@shell_exec`

Verified:

- `shell_command_setup` constructs `std::process::Command::new("sh")` and appends `arg("-c")` then `arg(script)`. There is no implicit shell path; ordinary `run/output/output_text` never invoke a shell.
- `lib/sifr/process.sifr` annotates the three shell APIs with both `@blocking_io` and `@shell_exec`. `annotation_with_range` keeps the last workload kind, so shell APIs are classified as `ShellExec`. `process_shell_exec_direct_async_rejected.sifr` triggers `SIFR-ASYNC-0007` (verified via `cargo run -- check`).
- Argv invocation of `Command("sh")` from user code (e.g. `process_sync_bytes_env_cwd_stdin.sifr`) is correctly treated as ordinary argv work, not the `output_shell` family. This matches the design: `@shell_exec` marks the helper APIs that auto-construct `sh -c`, not every program named `sh`.

### 3. Env/cwd/stdin/output/text behavior is typed and no data-dependent unwraps

Verified by reading lowerings and emitted Rust:

- Env entries are joined as `key=value` and split via `split_once('=')`. Cwd is gated by `has_cwd`. Stdin is gated by `has_stdin` and piped via `Stdio::piped()` + `std::io::Write::write_all`. All `io::Error`s map to `ProcessError { message }` via `.map_err(...).?`.
- Exit code is collected via `status.code().unwrap_or(-1) as i64` — a typed fallback, not a panicking `unwrap`. Signal-killed processes return `-1` and `success = false`; `Status.signal` remains `None` (foundation: signal/timeout/cancellation fields are placeholders, documented as deferred in the traceability doc).
- `output_text` and `output_shell_text` route through `utf8_encoding_guard`, which rejects non-UTF-8 encodings with `Err(ProcessError { ... })`. Bytes-to-text conversion uses `String::from_utf8(...).map_err(...)?`, surfacing decode failures as typed `ProcessError` rather than panicking.

No `.unwrap()`/`.expect()`/`.unwrap_or_else(panic)` on data-dependent paths.

### 4. Imported workload metadata propagates without regressing existing diagnostics

Verified by reading and locally exercising:

- `LoweringResult.function_workloads` is added (`crates/sifr_ir/src/lowering_result.rs`), populated in `mod_impl.rs` from `function_workload_annotations`.
- `ExternalDefs.function_workloads` is added (`crates/sifr_lowering/src/lower/external_defs.rs`) and:
  - Populated in the stdlib bootstrap loop for locally defined functions (`crates/sifr_driver/src/stdlib/bootstrap.rs:91-105, 344-348`).
  - Populated in `collect_module_exports` for both locally defined functions and re-exports through `external_defs.function_workloads` (`crates/sifr_frontend/src/query_diagnostics.rs:288, 307-310, 402-408, 475-479`).
- `import_callable_workload` (`crates/sifr_lowering/src/lower/imported_defaults.rs`) writes `WorkloadKind` into `ctx.function_workload_annotations`. It is invoked from three sites in `mod_impl.rs` (early import resolution, stdlib `sifr.*` imports, local-module imports) and once from `imports.rs::resolve_imports_early`.
- Independent fixture check: I built a temporary fail fixture importing `run` (not just `output_text`/`output_shell_text`) from `sifr.process` and confirmed `SIFR-ASYNC-0003` fires with the expected message. The existing fail fixtures cover the named symbols used in the M4 ledger.
- The `cargo test -p sifr test_e2e_fail` run (413 fail tests) and full `scripts/run_all_tests.sh --profile create-pr` (92 pass / 0 fail in the e2e pass suite) confirm no regressions in existing diagnostics.

### 5. M4 traceability honestly distinguishes implemented vs deferred work

`verification/stdlib/concurrency_runtime_m4_process_traceability.md` records:

- Sync `Command`/`Status`/`Output`/`TextOutput`/`Stdio` constants and the six sync top-level callables as implemented, with explicit notes that:
  - `Status.signal`, `timed_out`, `cancelled` are placeholder fields populated only by future waves.
  - `Stdio` constants reserve the namespace; pipe ownership APIs are not claimed by this wave.
  - Non-UTF-8 text-process policy remains open for the full M4 text-mode closeout.
- Follow-up boundaries explicitly list deferred work: production `spawn`/`Child`/`wait`/`PipeReader`/`PipeWriter`, owned pipe lifecycle and double-close diagnostics, async spawn/wait/communicate, cancellation-safe observation, timeout/terminate/kill/signal evidence, scoped `TaskGroup.spawn_process`, and full text-mode closeout consuming text/i18n M1 evidence.
- The execution ledger entry for "M4 sync process foundation" explicitly says "Status: In progress; sync process foundation wave implemented locally" and the milestone box in `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` remains `[ ] milestone_concurrency_runtime_4`. No premature closure.

### 6. Wave size and review boundary

Diff stat: 20 tracked files, 164 insertions / 4 deletions, plus the new fixtures, intrinsic/codegen modules, stdlib source, traceability artifact, and `SIFR-ASYNC-0007.md`. The work is decomposed by responsibility (intrinsic registry, codegen lowerer, lowering metadata plumbing, diagnostic registry, stdlib source, fixtures, manifests, docs). No oversized files: `python3 scripts/check_file_size_guardrails.py` reports PASS across 2163 files.

`scripts/run_all_tests.sh --profile create-pr` reports PASS with platform golden `pass=5, skip=2` and create-pr e2e `92 passed, 0 failed, cache_hits=23/25`. The advisory "warm wall-time budget exceeded" warning is consistent with prior M3 waves and is non-blocking.

## Non-blocking Follow-ups

Recorded for later M4 waves, not blockers for this foundation PR:

- `Stdio.PIPE/INHERIT/NULL` are reserved namespace placeholders. They are constructible but unused by the sync APIs. When pipe ownership APIs land, route them through these constants rather than introducing parallel selectors.
- `Command.stdin_bytes` concatenates on repeated calls (`self.stdin_data = self.stdin_data + data`). For pipe-payload semantics this is reasonable, but the public API name suggests "set". A later wave can either rename to `extend_stdin_bytes` or change semantics to overwrite once the spawn/pipe wave defines the canonical model.
- `Command.cwd` uses `path + ""` to create an owned `String`. Consider exposing an owned-string assignment idiom in Sifr to remove the workaround.
- `Status` constructor always sets `signal=None`, `timed_out=false`, `cancelled=false`, `success=(code==0)`, `kind="success"|"nonzero"`. Signal-killed processes therefore appear as `code=-1, kind="nonzero"`. The traceability doc already calls this out; the process lifecycle wave should populate `signal` and add a `kind="signaled"` (or equivalent) state.
- `_sifr.sys.subprocess_run*` legacy intrinsics still exist in `crates/sifr_stdlib/src/sys_fs.rs` and `crates/sifr_codegen/src/intrinsics/registry/subprocess.rs`. They are unreferenced from any `lib/sifr/*.sifr` source and not consumed by the new `sifr.process` lowering. A follow-up wave should delete them once nothing in tests, fixtures, or docs needs them, to remove dead legacy surface from the registry.
- `lib/sifr/process.sifr` `output_shell` / `output_shell_text` declare `raw:` twice (one inside each branch of `if stdin is None`). This is legal but stylistically noisy; consider hoisting the call into a helper or single binding once function defaults gain a uniform shape.
- The bootstrap loop only exports workload labels for locally defined functions; if a future stdlib module re-exports a `@blocking_io`/`@shell_exec` callable, the workload metadata would currently be lost on the bootstrap re-export hop. M4 does not need this, but the follow-up should mirror `collect_module_exports`'s re-export handling in `compile_stdlib_uncached_impl` before any stdlib module re-exports workload-annotated callables.

## Local Validation Re-verified by Reviewer

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_sync_output_text.sifr` → PASS
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_sync_bytes_env_cwd_stdin.sifr` → PASS
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_shell_exec_output.sifr` → PASS
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_blocking_direct_async_rejected.sifr` → expected FAIL with `SIFR-ASYNC-0003`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_shell_exec_direct_async_rejected.sifr` → expected FAIL with `SIFR-ASYNC-0007`
- Independent `run`-in-async fixture (scratch) → expected FAIL with `SIFR-ASYNC-0003` for `run`, confirming workload export covers all six sync callables, not just the two used in the fail fixtures.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_sync_output_text.sifr | grep` → confirms `std::process::Command`, `Stdio::piped`, `split_once('=')`, `String::from_utf8`, and no `subprocess` helper on the ordinary path; shell APIs route through `Command::new("sh")` + `arg("-c")` + `arg(script)`.

## Decision

PASS. The M4 sync process foundation wave is ready to PR.
