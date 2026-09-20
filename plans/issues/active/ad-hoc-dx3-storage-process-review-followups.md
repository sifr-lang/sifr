# DX.3 storage and process review follow-ups

status: open; separate future work, not DX.3 blockers
source: [merged DX.3](../archive/ad-hoc-compiler-dx-and-toolchain-reuse.md)
implementation: [PR #3844](https://github.com/sifr-lang/sifr/pull/3844)

These observations do not authorize implementation in the completed DX.3 session.
The final scoped reviewer classified the following as suggestions, with no blocking findings.

- Revisit strict cache-root ownership/symlink checks when Rust interop uses a
  user-configured CARGO_TARGET_DIR. Establish intended behavior for symlinked
  targets and pre-existing group-writable probe directories.
- Qualify signal delivery during waitpid/capture reads and preserve cancellation
  causes and partial streams across EINTR; consider SA_RESTART or explicit retry.
- Qualify functional-status reporting for failures outside ProfileRunner.execute_step,
  including budget preparation, summarization and release-report generation.
- Name the generic 2400-second safety-deadline constant and document its policy
  rationale independently of named performance contracts.
- Fix pre-existing optional all-targets Clippy warnings in
  crates/sifr_driver/src/build/stdlib_interop_startup_tests.rs:150 (wildcard for a
  single variant) and crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:250/260
  (items after statements). Production driver Clippy passes.

Final review: [Opus SATISFIED](https://github.com/sifr-lang/sifr/pull/3844#issuecomment-5711325637).
Raw evidence and the initial review remain under
yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx3-evidence/.
