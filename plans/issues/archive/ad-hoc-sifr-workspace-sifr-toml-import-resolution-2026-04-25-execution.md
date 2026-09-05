# Ad-hoc Phase Execution: Sifr Workspace Resolution Via `sifr.toml`

Status: closed
Started: 2026-04-25
Phase plan: `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md`
Source issue: `issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md`

## Wave Checklist

- [x] WS0 workspace discovery and config validation
- [x] WS1 workspace-aware compilation mode
- [x] WS2 module resolver refactor with no behavior change
- [x] WS3 workspace source resolution and diagnostics
- [x] WS4 build/run/check/emit wiring and cache correctness
- [x] WS5 verification-suite fixtures, design note, and LeetCode pilot
- [x] WS6 final gate, review, and closure

## WS0 Workspace Discovery And Config Validation

Status: merged
Branch: ad-hoc/sifr-workspace-ws0
PR: https://github.com/sifr-lang/sifr/pull/1639
Merged: 2026-04-25

### Planned Scope

- Add workspace discovery/config module.
- Add TOML parsing dependency.
- Define the internal native `sifr.toml` manifest model.
- Validate `[source].roots` config and source roots.
- Add targeted unit tests for discovery and validation.

### Validation Evidence

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo test -p sifr_driver workspace -- --nocapture`
- [x] Negative-path config validation tests cover malformed TOML, wrong `package.name` type, wrong `source.roots` type, source escape, absolute path, empty path, missing directory, and file path.

## WS1 Workspace-Aware Compilation Mode

Status: merged
Branch: ad-hoc/sifr-workspace-ws1
PR: https://github.com/sifr-lang/sifr/pull/1642
Merged: 2026-04-25

### Planned Scope

- Make workspace presence activate project mode for any entry filename.
- Preserve the legacy `main.sifr` plus sibling-import heuristic outside workspaces.
- Surface workspace parse/config diagnostics through CLI commands.

### Validation Evidence

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo test -p sifr -- resolve_compilation_mode -- --nocapture`
- [x] CLI malformed-workspace diagnostic path covered by `test_resolve_compilation_mode_reports_malformed_workspace_manifest`.

## WS2 Module Resolver Refactor With No Behavior Change

Status: merged
Branch: ad-hoc/sifr-workspace-ws2
PR: https://github.com/sifr-lang/sifr/pull/1640
Merged: 2026-04-25

### Planned Scope

- Introduce `ModuleResolver`, `ResolvedModule`, and structured resolution errors.
- Keep the initial resolver entry-parent-only.
- Update discovery and test-runner call sites without changing behavior.
- Update `test_runner/orchestrator.rs` to pass an entry-parent-only resolver and preserve current scope.

### Validation Evidence

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo test -p sifr_driver discovery -- --nocapture`
- [x] `cargo test -p sifr_driver test_runner -- --nocapture`

## WS3 Workspace Source Resolution And Diagnostics

Status: merged
Branch: ad-hoc/sifr-workspace-ws3
PR: https://github.com/sifr-lang/sifr/pull/1641
Merged: 2026-04-25

### Planned Scope

- Add configured workspace source roots to module resolution.
- Implement dotted-module path conversion.
- Add ambiguity and unresolved diagnostics with deterministic path lists.
- Add shared dotted Rust module layout helper and namespace-conflict diagnostic.
- Add positive/negative resolver tests and snapshots.

### Validation Evidence

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo test -p sifr_driver workspace -- --nocapture`
- [x] `cargo test -p sifr_driver discovery -- --nocapture`
- [x] Diagnostic code and URL unit coverage for `SIFR-WORKSPACE-0101`, `SIFR-WORKSPACE-0102`, and `SIFR-WORKSPACE-0103`

## WS4 Build/Run/Check/Emit Wiring And Cache Correctness

Status: merged
Branch: ad-hoc/sifr-workspace-ws4
PR: https://github.com/sifr-lang/sifr/pull/1643
Merged: 2026-04-25

### Planned Scope

- Thread workspace context through rooted entrypoint planning and build APIs.
- Align `build`, `run`, `check`, and `emit`.
- Materialize dotted modules as nested Rust module trees.
- Add dotted cache regression for workspace helper content changes.

### Validation Evidence

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo test -p sifr_driver project_build_check -- --nocapture`
- [x] Cache-key regression covered by `test_cached_project_invalidates_when_workspace_helper_changes`.

## WS5 Verification-Suite Fixtures, Design Note, And LeetCode Pilot

Status: merged
Branch: ad-hoc/sifr-workspace-ws5
PR: https://github.com/sifr-lang/sifr/pull/1644
Merged: 2026-04-25

### Planned Scope

- Add workspace verification-suite pass/fail fixtures under `crates/sifr/tests/verification/project/`.
- Add `internal_docs/sifr_workspace_design.md`.
- Document the native `sifr.toml` model and explicitly defer `pyproject.toml` / `[tool.sifr]` compatibility.
- Update `internal_docs/architecture.md`.
- Populate the existing `audits/leetcode/helpers/` directory with `list_node.sifr` and pilot it with `0021_merge_two_sorted_lists.sifr`.
- Regenerate pair scan and full corpus artifacts.

### Validation Evidence

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `python3 scripts/run_verification_hardening.py --suite project --result-json target/verification/ws5-project-after-root.json`
- [x] Targeted LeetCode pilot `cargo run -q -p sifr -- check audits/leetcode/0021_merge_two_sorted_lists.sifr` and `cargo run -q -p sifr -- run audits/leetcode/0021_merge_two_sorted_lists.sifr`
- [x] Pair scan regenerated at `verification/leetcode/leetcode_pair_diff_scan_20260425.json`; `0021_merge_two_sorted_lists` now reports `sifr_lines = 9`.
- [x] Full corpus rerun regenerated at `verification/leetcode/full_corpus_current_results_20260425_workspace_pilot.json`; summary `PASS = 208`, `NO_ORACLE = 203`, with no `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`.

## WS6 Final Gate, Review, And Closure

Status: merged
Branch: ad-hoc/sifr-workspace-ws6
PR: https://github.com/sifr-lang/sifr/pull/1645
Merged: 2026-04-25

### Planned Scope

- Run required local validation.
- Split workspace unit tests out of `crates/sifr_driver/src/workspace/mod.rs` so the final merged tree satisfies the `sifr_driver` maintainability guardrail.
- Refresh stale validation-contract and diagnostic baselines for module-scoped `[main]` diagnostics.
- Record final review result and links.
- Update phase and roadmap status.
- Update `internal_docs/roadmap.md` Phase 31.6 status and dependency notes for closure.

### Required Final Validation

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `python3 scripts/check_hir_maintainability_guardrails.py`
- [x] `scripts/run_all_tests.sh --profile quick`
- [x] `scripts/run_all_tests.sh`
- [x] Full LeetCode corpus rerun with no new `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`

### Validation Evidence

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `python3 scripts/check_hir_maintainability_guardrails.py`
- [x] `python3 scripts/check_sifr_driver_maintainability_guardrails.py`
- [x] `cargo test -p sifr_driver workspace -- --nocapture`
- [x] `cargo test -p sifr test_emit_entrypoint_uses_project_mode_for_project_like_main -- --nocapture`
- [x] `bash scripts/run_validation_contract_matrix.sh`
- [x] `python3 scripts/run_verification_hardening.py --profile pr --suite diagnostics --result-json target/verification/ws6-diagnostics-after-baseline-fix.json`
- [x] `scripts/run_all_tests.sh --profile quick`; report `target/validation_lane_reports/quick.latest.json`, wall time 74.36s, 0 failures.
- [x] `scripts/run_all_tests.sh`; report `target/validation_lane_reports/pr.latest.json`, wall time 96.82s, hardening `variants = 28`, `failures = 0`, `blocking_failures = 0`.
- [x] `cargo build --release -p sifr`
- [x] `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/full_corpus_manifest_20260402_live.json --output verification/leetcode/full_corpus_current_results_20260425_workspace_closure.json --timeout-seconds 30 --no-build-release-if-missing`; summary `case_count = 411`, `PASS = 208`, `NO_ORACLE = 203`, no `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`.
- [x] Post-pass-5 blocker fix validation: `cargo test -p sifr_driver --lib`; 97 passed, 0 failed.
- [x] Post-pass-5 blocker fix validation: `cargo test -p sifr_driver test_run_tests_resolves_dotted_local_support_modules -- --nocapture`.
- [x] Post-pass-5 blocker fix validation: `cargo fmt --check`.
- [x] Post-pass-5 blocker fix validation: `cargo clippy --workspace -- -D warnings`.
- [x] Post-pass-5 blocker fix validation: `scripts/run_all_tests.sh --profile quick`; report `target/validation_lane_reports/quick.latest.json`, wall time 87.03s, 0 failures, includes `cargo test -p sifr_driver --lib`.
- [x] Post-pass-5 blocker fix validation: `scripts/run_all_tests.sh`; report `target/validation_lane_reports/pr.latest.json`, wall time 108.06s, hardening `variants = 28`, `failures = 0`, `blocking_failures = 0`, includes `cargo test -p sifr_driver --lib`.

## External Reviews

- historical pass 1: `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass1.md` was unavailable because the review runner was invoked outside the `agent review` uv project.
- historical pass 2: `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass2.md` returned NOT READY; blockers were dotted Rust module materialization and incompatible flat e2e fixture placement.
- historical pass 3: `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass3.md` returned READY for the old pyproject-targeted plan. A fresh review is required after switching this phase to native `sifr.toml`.
- pass 4: `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass4.md` returned READY with no blocking findings for the native `sifr.toml` plan.
- pass 5: `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass5.md` superseded the earlier pass-5 draft and returned NOT READY; blockers were the stale `sifr_driver` deterministic assembly test, the missing `sifr_driver` lib-test gate in `scripts/run_all_tests.sh`, and missing dotted support-module materialization in the test runner.
- pass 6: `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass6.md` reviewed the earlier pass-5 READY draft before the corrected NOT READY pass-5 artifact landed; treat it as superseded by pass 5 and the follow-up blocker-fix work.
- pass 7: `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass7.md` returned READY; B1, B2, and B3 from the corrected pass-5 review are resolved and no further blocker-fix review rounds are required.
- pass 8: `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass8.md` returned READY on merged `main`; no blockers remain and no further review rounds are required for this phase closure.
