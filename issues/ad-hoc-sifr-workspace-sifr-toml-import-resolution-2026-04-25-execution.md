# Ad-hoc Phase Execution: Sifr Workspace Resolution Via `sifr.toml`

Status: in_progress
Started: 2026-04-25
Phase plan: `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md`
Source issue: `issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md`

## Wave Checklist

- [x] WS0 workspace discovery and config validation
- [ ] WS1 workspace-aware compilation mode
- [x] WS2 module resolver refactor with no behavior change
- [x] WS3 workspace source resolution and diagnostics
- [ ] WS4 build/run/check/emit wiring and cache correctness
- [ ] WS5 verification-suite fixtures, design note, and LeetCode pilot
- [ ] WS6 final gate, review, and closure

## WS0 Workspace Discovery And Config Validation

Status: merged
Branch: ad-hoc/sifr-workspace-ws0
PR: https://github.com/yaseralnajjar/sifr/pull/1639
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

Status: implemented_pending_pr
Branch: ad-hoc/sifr-workspace-ws2
PR: https://github.com/yaseralnajjar/sifr/pull/1640
Merged: tbd

### Planned Scope

- Make workspace presence activate project mode for any entry filename.
- Preserve the legacy `main.sifr` plus sibling-import heuristic outside workspaces.
- Surface workspace parse/config diagnostics through CLI commands.

### Validation Evidence

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo test -p sifr -- resolve_compilation_mode -- --nocapture` or equivalent targeted selector
- [ ] CLI malformed-workspace diagnostic check recorded in PR notes

## WS2 Module Resolver Refactor With No Behavior Change

Status: implemented_pending_pr
Branch: ad-hoc/sifr-workspace-ws3
PR: https://github.com/yaseralnajjar/sifr/pull/1641
Merged: tbd

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

Status: not_started
Branch: tbd
PR: tbd
Merged: tbd

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

Status: not_started
Branch: tbd
PR: tbd
Merged: tbd

### Planned Scope

- Thread workspace context through rooted entrypoint planning and build APIs.
- Align `build`, `run`, `check`, and `emit`.
- Materialize dotted modules as nested Rust module trees.
- Add dotted cache regression for workspace helper content changes.

### Validation Evidence

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo test -p sifr_driver project_build_check -- --nocapture` or equivalent targeted selector
- [ ] Cache-key regression proving dotted helper content invalidates cached runs

## WS5 Verification-Suite Fixtures, Design Note, And LeetCode Pilot

Status: not_started
Branch: tbd
PR: tbd
Merged: tbd

### Planned Scope

- Add workspace verification-suite pass/fail fixtures under `crates/sifr/tests/verification/project/`.
- Add `internal_docs/sifr_workspace_design.md`.
- Document the native `sifr.toml` model and explicitly defer `pyproject.toml` / `[tool.sifr]` compatibility.
- Update `internal_docs/architecture.md`.
- Populate the existing `audits/leetcode/helpers/` directory with `list_node.sifr` and pilot it with `0021_merge_two_sorted_lists.sifr`.
- Regenerate pair scan and full corpus artifacts.

### Validation Evidence

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] Project verification-suite command with workspace cases
- [ ] Targeted LeetCode pilot `check` and `run`
- [ ] Pair scan regeneration
- [ ] Full corpus rerun summary

## WS6 Final Gate, Review, And Closure

Status: not_started
Branch: tbd
PR: tbd
Merged: tbd

### Planned Scope

- Run required local validation.
- Record final review result and links.
- Update phase and roadmap status.
- Update `internal_docs/roadmap.md` Phase 31.6 status and dependency notes for closure.

### Required Final Validation

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `python3 scripts/check_hir_maintainability_guardrails.py`
- [ ] `scripts/run_all_tests.sh --profile quick`
- [ ] `scripts/run_all_tests.sh`
- [ ] Full LeetCode corpus rerun with no new `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`

### Validation Evidence

- tbd

## External Reviews

- historical pass 1: `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass1.md` was unavailable because the review runner was invoked outside the `talk-to-claude` uv project.
- historical pass 2: `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass2.md` returned NOT READY; blockers were dotted Rust module materialization and incompatible flat e2e fixture placement.
- historical pass 3: `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass3.md` returned READY for the old pyproject-targeted plan. A fresh review is required after switching this phase to native `sifr.toml`.
- pass 4: `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass4.md` returned READY with no blocking findings for the native `sifr.toml` plan.
