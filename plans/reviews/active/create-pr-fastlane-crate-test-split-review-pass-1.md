# create-pr fastlane crate-test split review pass 1

Please review the current working tree for the create-pr fastlane crate-test split milestone.

Context:
- Previous merged PRs already fixed generated-code-quality smoke selection/cache behavior and replaced repeated validation-internal `cargo run -p sifr` compiler launches with a resolved/prebuilt Sifr binary.
- This milestone addresses the `crate_tests` regression by moving slow generated artifact/build success-path tests out of smoke crate suites while keeping that coverage in full/merge/nightly/release profiles.
- Do not modify files. Review only.

Current changes to review:
- Marked generated artifact/build success-path tests as `#[ignore]` in:
  - `crates/sifr_driver/src/build/entrypoint.rs`
  - `crates/sifr_driver/src/tests/project_build_check.rs`
  - `crates/sifr_driver/src/tests/package_project_build_check.rs`
  - `crates/sifr/src/diagnostics_and_packages_tests.rs`
  - `crates/sifr/src/mode_resolution_tests.rs`
- Added full-only profile suites in `create-pr`, `merge`, `nightly`, and `release`:
  - `sifr_cli_generated_builds`: `cargo test -p sifr --bin sifr -- --ignored --test-threads=1`
  - `sifr_driver_generated_builds`: `cargo test -p sifr_driver --lib -- --ignored --test-threads=1`
- Added a runner self-test guardrail that these suites exist in full profiles and do not run in smoke.
- Changed profile runner crate-test execution to pass `self.env`, so crate tests inherit the lane's offline/probe-cache/resolved-binary environment.

Validation already run:
- `cargo fmt --check`: pass
- `python3 -m py_compile verification/runner/sifr_verify/profile_runner.py verification/runner/sifr_verify/selftest.py`: pass
- `uv run --project verification --locked python -m sifr_verify --self-test`: pass
- `python3 scripts/check_file_size_guardrails.py`: pass
- `SIFR_RUST_BRIDGE_PROBE_CACHE_DIR=target/sifr_rust_bridge_probe_cache/create-pr /usr/bin/time -p cargo test -p sifr --bin sifr --locked`: pass, `98 passed; 4 ignored`, ~8.5s
- `SIFR_RUST_BRIDGE_PROBE_CACHE_DIR=target/sifr_rust_bridge_probe_cache/create-pr /usr/bin/time -p cargo test -p sifr_driver --lib --locked`: pass, `285 passed; 18 ignored`, ~31.3s

Validation attempted but intentionally stopped:
- Full ignored CLI suite was started twice. The first parallel run spent ~6m without finishing because all generated build/probe cases contended. After adding `--test-threads=1`, a cold serial run still spent ~9m in the first generated probe/build path, confirming this is full-only generated-build coverage and not practical as milestone smoke validation. No failures were observed before interruption; orphaned temp subprocesses from the interrupted run were cleaned up.

Review questions:
1. Is any meaningful coverage lost from create-pr smoke rather than moved to full/merge/nightly/release?
2. Are any ignored tests too broad or misplaced?
3. Are the profile additions correct and sufficient to retain displaced coverage?
4. Is passing `self.env` into crate-test commands correct, or does it have unintended side effects?
5. Are there any blockers before opening this milestone PR?
