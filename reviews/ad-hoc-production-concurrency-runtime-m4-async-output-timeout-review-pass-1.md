# M4 Async Process Output Timeout — Review Pass 1

Branch: `codex/concurrency-runtime-m4-async-output-timeout-next`
Scope: working-tree review of the uncommitted M4 async output timeout wave.

## Verdict: PASS

The wave is internally consistent, types and gating line up, and the targeted local validation already executed is sufficient for opening the PR. No blocking correctness issues found.

## Verification by checklist

### 1. 8-argument order alignment

The argument order `program, args, env, cwd, has_cwd, stdin_mode, has_stdin, timeout_seconds` agrees across all four surfaces:

- Public wrapper — `lib/sifr/process.sifr:406-415` forwards `command.program, command.arguments, command.env_vars, command.working_dir, command.has_working_dir, command.stdin_mode, command.has_stdin_data, seconds`.
- Stdlib metadata — `crates/sifr_stdlib/src/process.rs:244-259` registers the same tuple with matching types (`Str`, `List[Str]`, `List[Str]`, `Str`, `Bool`, `Str`, `Bool`, `Float`).
- Intrinsic lowerer — `crates/sifr_codegen/src/intrinsics/registry/process_async.rs:67-78` enforces `args.len() == 8`, clones positions 0-3 + 5 via `async_process_owned_args`, and passes positions 4, 6, 7 by value, producing 8 helper args.
- Generated helper params — `crates/sifr_codegen/src/preamble/process_async_runtime.rs:209-216` builds the same 8 named params via `process_async_params(true)` + appended `timeout_seconds: f64`.

### 2. Generated `__sifr_process_async_output_timeout` body

`process_async_runtime.rs:370-473`:
- `process_async_stdin_mode_guard` rejects any non-`inherit` stdin mode with a typed `ProcessError` ("…mode requires owned pipe support"), satisfying the non-inherit rejection requirement.
- `has_stdin` guard returns typed `ProcessError` "stdin bytes require owned pipe support" before any spawn.
- Finite/non-negative validation: `!timeout_seconds.is_finite() || timeout_seconds < 0.0` returns typed `ProcessError` via `format!` with the offending value; otherwise `Duration::try_from_secs_f64` is propagated via `process_map_err` so out-of-range floats also become typed errors rather than panics.
- Normal-completion arm pipes stdout/stderr, spawns via Tokio, takes the owned pipe handles, and drains both via `read_to_end` inside `tokio::try_join!(__child.wait(), __stdout_read, __stderr_read)` — i.e., async draining, not blocking, and concurrent with `wait()`. The status is mapped through the shared `__sifr_process_status_from_exit` helper with the same `status.code().unwrap_or(-1) as i64` shape used everywhere else.
- No `.unwrap()`/`.expect()` on data-dependent values in the user runtime path. The only `unwrap_or(-1)` (on `ExitStatus::code()`) is the existing safe sentinel pattern already used by `__sifr_process_status_from_exit` and adjacent helpers.

### 3. Timeout arm: kill + reap + typed evidence

Timeout branch (`_ = tokio::time::sleep(__duration)`) does, in order:
1. `__child.kill().await` (typed map_err).
2. `__child.wait().await` reaping (typed map_err, result discarded).
3. Constructs `Status::new(-1, "timeout")`, sets `success=false` and `timed_out=true`.
4. Returns `Output::new(Vec::new(), Vec::new(), __timeout_status)` — empty stdout/stderr on timeout as required.

`Status::new` already sets `success = (code == 0)`, so the explicit `success = false` is redundant with the `code = -1` case but matches the run_timeout pattern and is harmless.

### 4. Preamble gating

- `SharedPreludeProcessAsyncNeeds` adds `needs_output_timeout` as an independent flag — `stdlib_filter/implementation.rs:53-58`.
- Text scan disambiguates needs_output vs needs_output_timeout: the existing `needs_output` matcher was tightened to `__sifr_process_async_output(` (with paren) so the new identifier no longer false-positives needs_output (`implementation.rs:335-337`). This is the correct fix — otherwise the plain output helper would be force-emitted alongside output_timeout.
- `SharedNeedsCollector` visit and `is_shared_prelude_item` match the new ident exactly (`implementation.rs:384-388, 422-426`), so syn-AST classification is precise.
- `build_process_async_items` takes the four bools and only pushes the matching helper item; `__sifr_process_async_output_timeout` only emits when `needs_output_timeout` is set. Existing run/run_timeout/output bodies are unchanged in this diff except for the gate function signature extension.
- Top-level `needs_process_async` correctly ORs in `needs_output_timeout` in `lib_modules_and_codegen.rs:421-426`; per-helper flags propagate through to the build call at `:595-601`.

Emission checks already executed confirm this: `async_run_timeout` does not emit the new ident nor the plain output helper, and the new ident shows up only for output_timeout fixtures.

### 5. Tokio io-util propagation

Required because the new helper imports `tokio::io::AsyncReadExt` and calls `read_to_end`. Propagated in both places that render Cargo.toml:
- Generated project — `crates/sifr_stdlib/src/features.rs:189` (`TOKIO_DEPS` now lists `"io-util", "macros", "process", "rt", "sync", "time"` in alphabetical order).
- Grouped e2e harness — `crates/sifr/tests/e2e_support/fixture_compilation.rs:481` (`tokio_dependency_spec`).

Contract tests updated to match:
- `crates/sifr/tests/e2e_support/harness_contract_tests.rs:521` (`test_generate_cargo_toml_required_tokio_uses_runtime_features`).
- `crates/sifr_codegen/src/lib_codegen_tests/async_runtime_codegen_tests.rs:164` (`test_generate_project_emits_tokio_dependency_when_required`).

Feature ordering is consistent across all four sites.

### 6. Fixture / manifest / doc honesty

- Fixture `crates/sifr/tests/e2e/pass/process_async_output_timeout.sifr` exercises only the surfaces the wave actually delivers: successful capture with status kind/success, timeout → empty bytes + `kind="timeout"` + `timed_out=True` + not-success, invalid (negative) timeout → typed `ProcessError` with "finite" substring, `stdin_bytes` → typed "owned pipe" rejection, non-inherit `stdin(Stdio("pipe"))` → typed "owned pipe" rejection. No claims of async spawn/wait/communicate, public async pipes, scoped supervision, or Windows behavior.
- Traceability (`concurrency_runtime_m4_process_traceability.md`) extends Output/Async-API rows with output-timeout, removes "async output timeout" from the follow-up boundaries list, and keeps async spawn/wait/communicate, owned pipes, cancellation, and scoped supervision explicitly pending. Honest.
- Host matrix adds a single `Async subprocess output timeout` row with macOS/Linux supported and Windows `host-limited`, matching the existing row pattern.
- Both create_pr and merge e2e manifests get the new fixture name added in the matching position. JSON valid (json.tool ran).
- Ledger entry under `issues/…-substrate-execution.md` records the wave as "in progress", the implementation summary, validation steps, and the initial create-pr failure + fix narrative. No overclaim of scope.

### 7. Initial create-pr failure fix

The wave introduced a real new dependency (`tokio io-util`), and the harness fixture-compilation path renders its own Cargo.toml for grouped e2e crates separately from generated projects. The initial create-pr lane caught that drift, the harness renderer plus its contract test were updated symmetrically with the runtime features list, and the lane was re-run to `101 passed / 0 failed`, `cache_hits=25/26`, `report_signature=9212e77abfa82acc`. Recorded validation is sufficient.

## Minor observations (non-blocking)

- The output-timeout body, like the existing run-timeout body, splices a multi-line raw Rust snippet through `RustStmt::Expr(RustExpr::Ident("…"))`. This is the established escape hatch for tokio::select! blocks in this preamble module; the new body matches the existing style. A future refactor could lower the select arms via structured `RustExpr` nodes, but that is out of scope for this wave.
- Explicit `__timeout_status.success = false;` is redundant given `Status::new(-1, ...)` already sets `success = (code == 0) == false`. Matches the run_timeout helper convention; leave for consistency.
- Outer fixture-level `try/except ProcessError as e: assert e.message == ""` is a redundant safety net (any unexpected ProcessError would already fail the inner asserts). Matches the pattern used by other process fixtures; harmless.

## Recommendation

Proceed to open the PR with the validation already recorded. No code or doc changes requested.
