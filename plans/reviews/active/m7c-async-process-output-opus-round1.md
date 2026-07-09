status: TOOL_TIMEOUT
reviewer: claude-opus-4-7
effort: xhigh
scope: M7c async process run/output/shell migration

Please review the current working tree for the M7c process-family slice.

Scope:
- Migrates `_sifr.process` async run/output/run-timeout/output-timeout/shell run/shell output/shell output-timeout leaves from compiler-retained intrinsics to private native declarations backed by `sifr_stdlib::process`.
- Adds `crates/sifr_stdlib/src/process/async_ops.rs` with boxed-future native async process operations using owned cloned inputs so sysroot Rust interop probes accept the async boundary.
- Keeps async child lifecycle and async pipe leaves retained for the next M7 slice.
- Removes migrated async run/output/shell leaves from `crates/sifr_codegen/src/intrinsics/registry.rs`, `process_async.rs`, retained catalog, and `sifr_retained_intrinsics`.
- Shrinks `crates/sifr_codegen/src/preamble/process_async_runtime.rs` to retained async child/pipe support and the generated `__sifr_process_status_from_exit` helper only.
- Fixes stdlib bootstrap top-level async function exports so async stdlib functions export coroutine types, matching existing method behavior.

Validation already run:
- `cargo fmt --check`
- `cargo test -p sifr_stdlib --features process process::async_ops::tests -- --nocapture`
- `cargo test -p sifr_driver function_type_from_hir_exports_async_functions_as_coroutines -- --nocapture`
- `cargo test -p sifr_driver process_sync_private_declarations_codegen_through_sifr_stdlib -- --nocapture`
- `cargo test -p sifr_codegen process -- --nocapture`
- `cargo test -p sifr_codegen registry_extended_tests::process_sync_timeout_intrinsics_lower_through_private_stdlib -- --nocapture`
- `cargo test -p sifr_retained_intrinsics -- --nocapture`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_shell_exec.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_output.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_output_timeout.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_timeout_group_cleanup.sifr`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_stdlib_migration_closure.py`
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`
- `scripts/run_all_tests.sh --profile create-pr`

Please focus on:
- Whether the boxed-future native stdlib boundary is sound for async Rust interop and does not accidentally retain borrowed data across the bridge.
- Whether `process_output_close` runs on all successful public async output paths and whether errors leave any concerning handles.
- Whether timeout status semantics match the previous generated preamble behavior.
- Whether the bootstrap async export typing change has any broader compatibility risk.
- Whether any migrated async run/output/shell leaf remains in compiler intrinsic dispatch, retained manifest/catalog, or generated preambles.

Do not edit files. Report blocking findings first, then non-blocking notes.

Reviewer execution note:
- Normal Claude Opus invocations with `claude --model claude-opus-4-7 --effort xhigh --print ...` hung without output across multiple attempts and were interrupted.
- A Python-capped normal invocation timed out after 180 seconds with no output.
- A second Python-capped normal invocation timed out after 240 seconds with no output after the create-pr validation passed.
- A `--bare` invocation exited immediately with `Not logged in · Please run /login`, so it cannot be used in this environment.
- No Opus findings were produced for this round.

Create-pr validation result:
- `scripts/run_all_tests.sh --profile create-pr` passed.
- E2E pass suite: 129 passed, 0 failed.
- Profile report: `target/validation_lane_reports/create-pr.latest.json`.

Local fallback review:
- No blocking findings found in the local review pass.
- Migrated async run/output/shell symbols are present in private declarations, the stdlib implementation, tests, and docs only; retained compiler dispatch/catalog paths keep only async child/pipe lifecycle support.
- The boxed-future Rust entrypoints clone borrowed arguments before constructing the future, so the native async boundary does not retain borrowed caller data across awaits.
- `process_output_close` runs on successful public async output paths after stdout/stderr/status extraction.
- Timeout status and stdin-mode rejection match the previous generated async preamble behavior.
