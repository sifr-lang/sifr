status: TOOL_TIMEOUT
reviewer: claude-opus-4-7
effort: xhigh
scope: M7d async process child and pipe migration

Please review the current working tree for the M7d process-family slice.

Scope:
- Migrates `_sifr.process` async spawn/wait/kill/terminate, async child pipe
  accessors, async pipe read/write/close leaves, and `process_handle_wait`
  from retained compiler intrinsics to private native declarations backed by
  `sifr_stdlib::process`.
- Adds `crates/sifr_stdlib/src/process/async_child.rs` with stdlib-owned async
  child and pipe tables.
- Keeps async native Rust interop entrypoints as boxed futures that own data
  across awaits.
- Updates public `sifr.process` wrappers to build `AsyncChild` and `Status`
  values from native handles/status parts.
- Updates scoped process runtime glue to register scoped children in the
  stdlib async child table so `ProcessHandle.wait()` and scope observation
  semantics share one table.
- Removes the async process intrinsic registry file, process retained fallback
  catalog, async process preamble files, and the obsolete process status
  preamble.

Validation already run:
- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p sifr_stdlib --features process process::async_child::tests -- --nocapture`
- `cargo test -p sifr_stdlib --features process process -- --nocapture`
- `cargo test -p sifr_driver process_sync_private_declarations_codegen_through_sifr_stdlib -- --nocapture`
- `cargo test -p sifr_codegen registry_extended_tests::process_sync_timeout_intrinsics_lower_through_private_stdlib -- --nocapture`
- `cargo test -p sifr_codegen process -- --nocapture`
- `cargo test -p sifr_retained_intrinsics -- --nocapture`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_spawn_pipes.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_scoped_spawn_handle.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_wait_cancel_safe.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_child_kill_terminate.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_scoped_parent_cancel.sifr`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_stdlib_migration_closure.py`
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`
- `python3 scripts/check_stdlib_manifest_schema.py`
- `python3 scripts/check_sysroot_stdlib_resource_certification_gate.py`
- `scripts/run_all_tests.sh --profile create-pr`

Please focus on:
- Whether the stdlib async child/pipe tables preserve prior wait cancellation
  safety, pipe read/write handle behavior, and close semantics.
- Whether scoped process observation remains correct after moving the child
  table into `sifr_stdlib::process`.
- Whether `ProcessHandle.wait()` correctly marks scoped children observed and
  shares state with the scope observer.
- Whether the boxed-future native boundary owns all borrowed data across awaits.
- Whether any migrated async child/pipe leaf remains in retained compiler
  dispatch, retained fallback signatures, or preamble injection.

Do not edit files. Report blocking findings first, then non-blocking notes.

Reviewer execution note:
- `claude --model claude-opus-4-7 --effort xhigh --print ...` was invoked
  through a Python wrapper with a 240-second timeout.
- The command produced no output and timed out with
  `CLAUDE_REVIEW_TIMEOUT_AFTER_240S`.
- No Opus findings were produced for this round.

Local fallback review:
- No blocking findings found in the local review pass.
- A retained-symbol search found no old async process preamble/helper names in
  codegen, retained fallback signatures, or the retained ledger.
- Scoped process E2E fixtures passed after moving scoped child registration to
  `sifr_stdlib::process`, covering `ProcessHandle.wait()` observation and
  parent-cancel behavior.
- Async child wait cancellation and async pipe fixtures passed with the new
  stdlib-owned async child/pipe tables.
- Local fallback review tightened async pipe `read()` to preserve cancellation
  reinsertion while still dropping the reader on actual read errors, matching
  the previous retained helper behavior.

Create-pr validation result:
- `scripts/run_all_tests.sh --profile create-pr` passed.
- E2E pass suite: 129 passed, 0 failed.
- Profile report: `target/validation_lane_reports/create-pr.latest.json`.
