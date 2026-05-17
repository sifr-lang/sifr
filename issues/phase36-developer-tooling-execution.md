# Phase 36 Developer Tooling Execution Checklist

Status: in_progress
Source phase: `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`

This issue tracks the sequential implementation loop for Phase 36. Each milestone requires implementation, local validation, Claude Opus review, PR review, merge, and tracker updates before the next milestone starts.

## Milestones

- [x] `milestone_36_1`: Production Tooling Contract Lock
- [ ] `milestone_36_2`: Diagnostics, Rules, Suppressions, Exclusions, And Formatting Foundation
- [ ] `milestone_36_3`: AnalysisHost, Symbol Index, And Session Model
- [ ] `milestone_36_4`: Full Editor Query Layer
- [ ] `milestone_36_5`: Production Native LSP Server
- [ ] `milestone_36_6`: Multi-Editor Syntax And Integration Assets
- [ ] `milestone_36_7`: VS Code Extension
- [ ] `milestone_36_8`: Production Verification And Performance Closeout

## Active Milestone: milestone_36_1

Branch: `phase36-m36-1-tooling-contract-lock`

Scope:

- [x] Lock final crate/module names: `sifr_analysis`, `sifr_format`, `sifr_lint`, `sifr_lsp`.
- [x] Lock VS Code extension repository boundary to separate `sifr-lang/sifr-vscode`.
- [x] Add support docs: `tooling_analysis`, `lsp_server`, `vscode_extension`, `editor_integrations`, and `tooling_verification`.
- [x] Add LSP protocol matrix with required methods, commands, settings, diagnostics modes, semantic token legend, code-action kinds, and unsupported surfaces.
- [x] Add VS Code extension contract JSON.
- [x] Add m36.1 guardrails and negative self-tests for tooling contracts, dependency boundaries, LSP split-brain, and VS Code extension contract drift.
- [x] Wire m36.1 developer tooling checks into `scripts/run_all_tests.sh`.
- [x] Run local validation.
- [x] Run Claude Opus review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2129>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2129>

## m36.1 Validation Evidence

- `python3 verification/tooling/check_tooling_contract_lock.py && python3 verification/tooling/check_tooling_contract_lock.py --self-test` -> PASS.
- `python3 verification/tooling/check_tooling_dependency_boundaries.py && python3 verification/tooling/check_tooling_dependency_boundaries.py --self-test` -> PASS.
- `python3 verification/tooling/check_lsp_split_brain.py && python3 verification/tooling/check_lsp_split_brain.py --self-test` -> PASS.
- `python3 verification/tooling/check_vscode_extension_contract.py && python3 verification/tooling/check_vscode_extension_contract.py --self-test` -> PASS.
- `python3 -m json.tool verification/tooling/lsp_protocol_matrix.json >/dev/null && python3 -m json.tool verification/tooling/vscode_extension_contract.json >/dev/null` -> PASS.
- `python3 -m py_compile verification/tooling/check_tooling_contract_lock.py verification/tooling/check_tooling_dependency_boundaries.py verification/tooling/check_lsp_split_brain.py verification/tooling/check_vscode_extension_contract.py verification/tooling/check_vscode_extension.py` -> PASS.
- `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `scripts/run_all_tests.sh --profile quick` -> PASS on rerun. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=939.02s`; `max_rss=555.4MiB`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.

Note: the first quick-lane attempt failed in the performance smoke step with `building frontend query benchmark helper timed out`. The same performance smoke command passed when rerun directly, and the full quick lane then passed.

## m36.1 Review Evidence

- `reviews/phase36-m36-1-review-pass-1.md` -> SATISFIED with no findings. Reviewer explicitly approved proceeding to PR.

## m36.1 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2129>
- Merge commit: `82eaf50fea0ebbf7dba7a46749ee549fa11f4d73`

## Active Milestone: milestone_36_2

Branch: `phase36-m36-2-format-lint-foundation`

Scope:

- [x] Add concrete `sifr_format` and `sifr_lint` workspace crates.
- [x] Add `sifr fmt [--check] <path-or-project>` CLI.
- [x] Add `sifr lint <path-or-project>` CLI.
- [x] Add Sifr-owned `FMT` and `LINT` diagnostic families and generated docs.
- [x] Implement conservative syntax-validated formatter foundation over `sifr_syntax`.
- [x] Implement policy metadata, explicit suppressions, unknown suppression, unused suppression, and blanket suppression diagnostics.
- [x] Add `check_formatter_contract.py` and `check_rule_suppression_contract.py` with negative self-tests.
- [x] Wire m36.2 checks into `scripts/run_all_tests.sh`.
- [x] Run local validation.
- [x] Run Claude Opus review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2130>
- [ ] Merge PR.

## m36.2 Validation Evidence

- `cargo fmt --check && git diff --check` -> PASS.
- `cargo check -p sifr_format -p sifr_lint -p sifr_driver -p sifr` -> PASS.
- `cargo clippy -p sifr_format -p sifr_lint -p sifr_driver -p sifr -- -D warnings` -> PASS.
- `cargo test -p sifr_format -p sifr_lint` -> PASS.
- `cargo test -p sifr_driver test_diagnostic_labels_are_derived_from_diagnostic_codes` -> PASS.
- `python3 verification/tooling/check_formatter_contract.py && python3 verification/tooling/check_formatter_contract.py --self-test` -> PASS.
- `python3 verification/tooling/check_rule_suppression_contract.py && python3 verification/tooling/check_rule_suppression_contract.py --self-test` -> PASS.
- `python3 scripts/check_diagnostic_code_coverage.py && python3 scripts/check_diagnostic_docs_sync.py && python3 scripts/check_diagnostic_schema_sync.py` -> PASS.
- Full developer tooling guardrail block, including m36.1 and m36.2 checks plus negative self-tests -> PASS.
- `scripts/run_all_tests.sh --profile quick` -> PASS. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=944.96s`; `max_rss=756.4MiB`; `e2e cache_hits=12/12`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.

## m36.2 Review Evidence

- `reviews/phase36-m36-2-review-pass-1.md` -> SATISFIED with no blocking findings. Reviewer explicitly approved proceeding to PR.

## m36.2 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2130>
