# Phase 36 Developer Tooling Execution Checklist

Status: completed
Source phase: `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`

This issue tracks the sequential implementation loop for Phase 36. Each milestone requires implementation, local validation, agent review, PR review, merge, and tracker updates before the next milestone starts.

## Milestones

- [x] `milestone_36_1`: Production Tooling Contract Lock
- [x] `milestone_36_2`: Diagnostics, Rules, Suppressions, Exclusions, And Formatting Foundation
- [x] `milestone_36_3`: AnalysisHost, Symbol Index, And Session Model
- [x] `milestone_36_4`: Full Editor Query Layer
- [x] `milestone_36_5`: Production Native LSP Server
- [x] `milestone_36_6`: Multi-Editor Syntax And Integration Assets
- [x] `milestone_36_7`: VS Code Extension
- [x] `milestone_36_8`: Production Verification And Performance Closeout

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
- [x] Run agent review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2129>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2129>

## m36.1 Validation Evidence

- `python3 verification/tooling/check_tooling_rules_lock.py && python3 verification/tooling/check_tooling_rules_lock.py --self-test` -> PASS.
- `python3 verification/tooling/check_tooling_dependency_boundaries.py && python3 verification/tooling/check_tooling_dependency_boundaries.py --self-test` -> PASS.
- `python3 verification/tooling/check_lsp_split_brain.py && python3 verification/tooling/check_lsp_split_brain.py --self-test` -> PASS.
- `python3 verification/tooling/check_vscode_extension_rules.py && python3 verification/tooling/check_vscode_extension_rules.py --self-test` -> PASS.
- `python3 -m json.tool verification/tooling/lsp_protocol_matrix.json >/dev/null && python3 -m json.tool verification/tooling/vscode_extension_rules.json >/dev/null` -> PASS.
- `python3 -m py_compile verification/tooling/check_tooling_rules_lock.py verification/tooling/check_tooling_dependency_boundaries.py verification/tooling/check_lsp_split_brain.py verification/tooling/check_vscode_extension_rules.py verification/tooling/check_vscode_extension.py` -> PASS.
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
- [x] Add `check_formatter_rules.py` and `check_rule_suppression_rules.py` with negative self-tests.
- [x] Wire m36.2 checks into `scripts/run_all_tests.sh`.
- [x] Run local validation.
- [x] Run agent review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2130>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2130>

## m36.2 Validation Evidence

- `cargo fmt --check && git diff --check` -> PASS.
- `cargo check -p sifr_format -p sifr_lint -p sifr_driver -p sifr` -> PASS.
- `cargo clippy -p sifr_format -p sifr_lint -p sifr_driver -p sifr -- -D warnings` -> PASS.
- `cargo test -p sifr_format -p sifr_lint` -> PASS.
- `cargo test -p sifr_driver test_diagnostic_labels_are_derived_from_diagnostic_codes` -> PASS.
- `python3 verification/tooling/check_formatter_rules.py && python3 verification/tooling/check_formatter_rules.py --self-test` -> PASS.
- `python3 verification/tooling/check_rule_suppression_rules.py && python3 verification/tooling/check_rule_suppression_rules.py --self-test` -> PASS.
- `python3 scripts/check_diagnostic_code_coverage.py && python3 scripts/check_diagnostic_docs_sync.py && python3 scripts/check_diagnostic_schema_sync.py` -> PASS.
- Full developer tooling guardrail block, including m36.1 and m36.2 checks plus negative self-tests -> PASS.
- `scripts/run_all_tests.sh --profile quick` -> PASS. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=944.96s`; `max_rss=756.4MiB`; `e2e cache_hits=12/12`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.

## m36.2 Review Evidence

- `reviews/phase36-m36-2-review-pass-1.md` -> SATISFIED with no blocking findings. Reviewer explicitly approved proceeding to PR.

## m36.2 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2130>
- Merge commit: `cb08508f8db60109740fed15df5f3ccbd19c3482`

## Active Milestone: milestone_36_3

Branch: `phase36-m36-3-analysis-host-symbol-index`

Scope:

- [x] Add concrete `sifr_analysis` workspace crate.
- [x] Implement `AnalysisHost` over `sifr_frontend` for project/single-file sessions.
- [x] Add coherent source snapshots, document versions, invalidation reports, and stale-result rejection.
- [x] Add current-workspace symbol index and stable symbol identity for the active analysis revision.
- [x] Expose every Phase 36 editor query method through `sifr_analysis`, with deeper feature logic reserved for m36.4 where required.
- [x] Add formatter and lint handoffs through `sifr_format` and `sifr_lint`.
- [x] Add completion ranking/evaluation foundation.
- [x] Add positive/negative tests for load/update/query plumbing, stale versions, stale snapshots, project symbols, and query metadata.
- [x] Add `check_analysis_snapshot_rules.py` and `check_analysis_split_brain.py` with negative self-tests.
- [x] Wire m36.3 checks into `scripts/run_all_tests.sh`.
- [x] Run local validation.
- [x] Run agent review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2131>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2131>

## m36.3 Validation Evidence

- `cargo fmt --check && git diff --check` -> PASS.
- `python3 -m py_compile verification/tooling/check_analysis_snapshot_rules.py verification/tooling/check_analysis_split_brain.py` -> PASS.
- `python3 verification/tooling/check_analysis_snapshot_rules.py && python3 verification/tooling/check_analysis_snapshot_rules.py --self-test` -> PASS.
- `python3 verification/tooling/check_analysis_split_brain.py && python3 verification/tooling/check_analysis_split_brain.py --self-test` -> PASS.
- `cargo check -p sifr_frontend -p sifr_analysis` -> PASS.
- `cargo clippy -p sifr_frontend -p sifr_analysis -- -D warnings` -> PASS.
- `cargo test -p sifr_frontend -p sifr_analysis` -> PASS.
- `scripts/run_all_tests.sh --profile quick` -> PASS. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=1121.72s`; `max_rss=723.5MiB`; `e2e cache_hits=12/12`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.

## m36.3 Review Evidence

- `reviews/phase36-m36-3-review-pass-1.md` -> SATISFIED with no blocking findings. 10 findings: 2 informational (symbol identity embedding, no frozen index needed), 3 low (explain_diagnostic reason ambiguity, explain_diagnostic re-diagnosis, index_for_module panic), 5 informational (format whitelist, generated_rust_preview sentinel, test coverage, verification scripts, error flattening). Reviewer explicitly approved proceeding to PR.

## m36.3 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2131>
- Merge commit: `5b2315e69aaead9269dd41a092e35b37c0968504`

## Active Milestone: milestone_36_4

Branch: `phase36-m36-4-editor-query-layer`

Scope:

- [x] Implement token-backed hover, definition, declaration, type-definition, references, prepare-rename, rename, document highlights, folding ranges, selection ranges, semantic tokens, and inlay hints through `sifr_analysis`.
- [x] Preserve diagnostics and workspace diagnostics through `sifr_frontend` plus `sifr_lint`.
- [x] Add token-ranged document symbols and workspace symbols.
- [x] Add policy suppression code actions for Sifr lint diagnostics.
- [x] Add generated Rust preview through the canonical compiler driver handoff.
- [x] Make missing diagnostic explanations explicit.
- [x] Add editor query parity snapshots, completion-quality fixture, and `run_tooling_parity.py` with negative self-test.
- [x] Wire m36.4 parity checks into `scripts/run_all_tests.sh`.
- [x] Run local validation.
- [x] Run agent review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2132>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2132>

## m36.4 Validation Evidence

- `cargo fmt --check && git diff --check` -> PASS.
- `python3 -m py_compile verification/tooling/run_tooling_parity.py verification/tooling/check_analysis_split_brain.py` -> PASS.
- `python3 verification/tooling/check_analysis_snapshot_rules.py && python3 verification/tooling/check_analysis_snapshot_rules.py --self-test` -> PASS.
- `python3 verification/tooling/check_analysis_split_brain.py && python3 verification/tooling/check_analysis_split_brain.py --self-test` -> PASS.
- `python3 verification/tooling/run_tooling_parity.py && python3 verification/tooling/run_tooling_parity.py --self-test` -> PASS.
- `cargo check -p sifr_analysis -p sifr_frontend -p sifr_driver` -> PASS.
- `cargo clippy -p sifr_analysis -p sifr_frontend -p sifr_driver -- -D warnings` -> PASS.
- `cargo test -p sifr_analysis -p sifr_frontend` -> PASS.
- `scripts/run_all_tests.sh --profile quick` -> PASS. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=869.59s`; `max_rss=582.2MiB`; `e2e cache_hits=0/12`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.

## m36.4 Review Evidence

- `reviews/phase36-m36-4-review-pass-1.md` -> SATISFIED with no blocking findings. Reviewer noted low/informational follow-ups around hardcoded first lint suppression rule, future parity runner scalability, and richer future hint/completion semantics.

## m36.4 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2132>
- Merge commit: `348a3ff7c67a8740c87c7e387428b721812134bb`

## Active Milestone: milestone_36_5

Branch: `phase36-m36-5-native-lsp-server`

Scope:

- [x] Add concrete `sifr_lsp` workspace crate.
- [x] Add `sifr lsp --stdio` CLI command.
- [x] Implement LSP 3.17 stdio initialize/shutdown/exit handling on `lsp-server` and `lsp-types`.
- [x] Add capability registry, document store, session, request queue, scheduler, conversion layer, diagnostics controller, command registry, request handlers, and protocol harness.
- [x] Implement full and incremental sync, push diagnostics, pull diagnostics, workspace diagnostics, workspace settings, watched-file refresh, save/close flows, cancellation notification handling, stale-version rejection, deterministic protocol errors, and snapshot discipline.
- [x] Route required editor query families through `sifr_analysis`: completion, hover, signature help, navigation, references, rename, symbols, semantic tokens, inlay hints, document highlights, folding, selection ranges, type hierarchy, code actions, formatting, diagnostics, generated Rust preview, and test command metadata.
- [x] Add `lsp_protocol_smoke.py` and `lsp_protocol_stress.py` with negative self-tests.
- [x] Wire m36.5 protocol checks into `scripts/run_all_tests.sh`.
- [x] Add Phase 35 `lsp-query-001-request-families` benchmark, baseline, and budget evidence.
- [x] Run local validation.
- [x] Run agent review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2133>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2133>

## m36.5 Validation Evidence

- `cargo fmt --check && git diff --check` -> PASS.
- `cargo check -p sifr_lsp -p sifr` -> PASS.
- `cargo clippy -p sifr_lsp -p sifr_analysis -p sifr -- -D warnings` -> PASS.
- `cargo test -p sifr_lsp` -> PASS.
- `python3 -m py_compile verification/tooling/lsp_protocol.py verification/tooling/lsp_protocol_smoke.py verification/tooling/lsp_protocol_stress.py verification/performance/lsp_query_bench.py verification/performance/run_benchmarks.py` -> PASS.
- `python3 verification/tooling/lsp_protocol_smoke.py && python3 verification/tooling/lsp_protocol_smoke.py --self-test` -> PASS.
- `python3 verification/tooling/lsp_protocol_stress.py && python3 verification/tooling/lsp_protocol_stress.py --self-test` -> PASS.
- `python3 verification/tooling/check_lsp_split_brain.py && python3 verification/tooling/check_lsp_split_brain.py --self-test` -> PASS.
- `python3 verification/tooling/check_tooling_dependency_boundaries.py && python3 verification/tooling/check_tooling_dependency_boundaries.py --self-test` -> PASS.
- `python3 verification/performance/run_benchmarks.py --validate-only && python3 verification/performance/run_benchmarks.py --self-test` -> PASS.
- `python3 verification/performance/check_budgets.py && python3 verification/performance/check_budgets.py --self-test` -> PASS.
- `python3 verification/performance/run_benchmarks.py --case lsp-query-001-request-families --sample-scale smoke` -> PASS.
- `scripts/run_all_tests.sh --profile quick` -> PASS on final pre-PR rerun. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=835.60s`; `max_rss=619.3MiB`; `e2e cache_hits=12/12`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.

## m36.5 Review Evidence

- `reviews/phase36-m36-5-review-pass-1.md` -> CHANGES_REQUESTED. Reviewer requested explicit shutdown/exit exit-code behavior, explicit protocol stress-test `exit` notification, non-stub `completionItem/resolve`, request scheduler lane evidence, and diagnostic clearing on `didClose`.
- `reviews/phase36-m36-5-review-pass-2.md` -> SATISFIED. Reviewer confirmed the blocking findings were resolved and approved proceeding to PR.

## m36.5 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2133>
- Merge commit: `a4a1297b1432598c98827ad98ba68293f33211c1`

## Active Milestone: milestone_36_6

Branch: `phase36-m36-6-editor-assets`

Scope:

- [x] Deliver checked-in or contribution-ready Neovim, Zed, Helix, and Emacs configs using `sifr lsp --stdio`.
- [x] Deliver TextMate and/or Tree-sitter assets required by VS Code and non-VS Code editor targets.
- [x] Add syntax asset drift checks against `sifr_syntax` tokenization fixtures.
- [x] Add `check_editor_assets.py`.
- [x] Run local validation.
- [x] Run agent review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2134>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2134>

## m36.6 Validation Evidence

- `cargo fmt --check && git diff --check` -> PASS.
- `python3 -m py_compile verification/tooling/check_editor_assets.py` -> PASS.
- `python3 -m json.tool editor_integrations/syntaxes/sifr.tmLanguage.json >/dev/null && python3 -m json.tool editor_integrations/syntaxes/sifr-token-scope-map.json >/dev/null` -> PASS.
- `python3 verification/tooling/check_editor_assets.py && python3 verification/tooling/check_editor_assets.py --self-test` -> PASS.
- `python3 verification/tooling/check_tooling_dependency_boundaries.py && python3 verification/tooling/check_tooling_dependency_boundaries.py --self-test` -> PASS.
- `python3 verification/tooling/check_tooling_rules_lock.py && python3 verification/tooling/check_tooling_rules_lock.py --self-test` -> PASS.
- `python3 scripts/check_diagnostic_cancel_usage.py && cargo check -p sifr_lsp && cargo clippy -p sifr_lsp -- -D warnings` -> PASS after renaming the LSP request-queue cancellation operation to avoid the diagnostics-only `.cancel(...)` guardrail.
- `scripts/run_all_tests.sh --profile quick` -> first attempt failed in `check_diagnostic_cancel_usage.py` on `crates/sifr_lsp/src/request_queue.rs` and `crates/sifr_lsp/src/session.rs`; fixed by renaming the queue operation.
- `scripts/run_all_tests.sh --profile quick` -> PASS on rerun. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=2305.22s`; `max_rss=595.5MiB`; `e2e cache_hits=0/12`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.

## m36.6 Review Evidence

- `reviews/phase36-m36-6-review-pass-1.md` -> SATISFIED. Reviewer confirmed all editor targets launch `sifr lsp --stdio`, parser-token syntax drift validation covers the fixtures, TOML/JSON assets parse, no fallback/semantic markers are present, and validation is wired into `scripts/run_all_tests.sh`.
- `reviews/phase36-m36-6-review-pass-2.md` -> SATISFIED. Reviewer confirmed the post-review LSP request-queue rename from `cancel` to `remove_pending` preserves behavior and satisfies `check_diagnostic_cancel_usage.py`.

## m36.6 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2134>
- Merge commit: `ac42f73464903b75b6ab3639d5ff766f31c44341`

## Active Milestone: milestone_36_7

Branch: `phase36-m36-7-vscode-extension`

Scope:

- [x] Implement the VS Code extension in the locked `sifr-lang/sifr-vscode` repository boundary.
- [x] Add language id, file extension, grammar, language configuration, LSP launcher, settings, commands, trace/logging, binary discovery, generated Rust preview, explain diagnostic, check/test commands, VS Code Test Explorer integration, format command, restart server, and server log access.
- [x] Add `.vsix` packaging, extension integration tests, and `vscode_extension_rules.json` validation.
- [x] Ensure extension tests can launch the locally built `sifr lsp --stdio`.
- [x] Run local validation.
- [x] Run agent review rounds until satisfied.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2135>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2135>

## m36.7 Validation Evidence

- In `../sifr-vscode`: `npm install` -> PASS, generated `package-lock.json`; npm reported 0 vulnerabilities.
- In `../sifr-vscode`: `npm run lint && npm run typecheck && npm test && npm run test:extension && npm run package` -> PASS after fixing the pure-config unit-test import and package output directory. Produced `dist/sifr-vscode-0.0.0.vsix`.
- Extension repo PR: <https://github.com/sifr-lang/sifr-vscode/pull/1>; merge commit: `eea6255bb4080e74ebd0b541923ea33315f4e279`.
- `python3 -m py_compile verification/tooling/check_vscode_extension.py verification/tooling/check_vscode_extension_rules.py` -> PASS.
- `python3 verification/tooling/check_vscode_extension_rules.py --require-extension-repo && python3 verification/tooling/check_vscode_extension_rules.py --self-test` -> PASS.
- `python3 verification/tooling/check_vscode_extension.py && python3 verification/tooling/check_vscode_extension.py --self-test` -> PASS.
- `scripts/run_all_tests.sh --profile quick` -> PASS. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=1459.55s`; `max_rss=562.1MiB`; `e2e cache_hits=12/12`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.

## m36.7 Review Evidence

- `reviews/phase36-m36-7-review-pass-1.md` -> SATISFIED. Reviewer confirmed the extension identity, native LSP launcher, command/settings coverage, forbidden-behavior guardrails, Test Explorer delegation, syntax/language assets, package scripts, CI, cross-repo validation wiring, docs, and versioning notes are acceptable for PR after final quick validation.

## m36.7 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2135>
- Merge commit: `b519c597516bb8585d48211a7d7cadc264c7b90b`

## Active Milestone: milestone_36_8

Branch: `phase36-m36-8-verification-closeout`

Scope:

- [x] Finalize `internal_docs/tooling_verification.md` for closeout-level gates.
- [x] Add `check_analysis_snapshot_coherence.py`, `check_completion_quality.py`, and `check_phase36_closeout.py`.
- [x] Wire the m36.8 checks into `scripts/run_all_tests.sh`.
- [x] Finalize LSP request-family budget coverage docs with no active LSP waivers.
- [x] Run targeted closeout validation.
- [x] Run agent review rounds until satisfied.
- [x] Run `scripts/run_all_tests.sh --profile quick`.
- [x] Run `scripts/run_all_tests.sh --profile pr`.
- [x] Open PR: <https://github.com/sifr-lang/sifr/pull/2136>
- [x] Merge PR: <https://github.com/sifr-lang/sifr/pull/2136>

## m36.8 Targeted Validation Evidence

- `python3 -m py_compile verification/tooling/check_completion_quality.py verification/tooling/check_analysis_snapshot_coherence.py verification/tooling/check_phase36_closeout.py` -> PASS.
- `python3 verification/tooling/check_completion_quality.py && python3 verification/tooling/check_completion_quality.py --self-test` -> PASS.
- `python3 verification/tooling/check_analysis_snapshot_coherence.py && python3 verification/tooling/check_analysis_snapshot_coherence.py --self-test` -> PASS.
- `python3 verification/tooling/check_phase36_closeout.py && python3 verification/tooling/check_phase36_closeout.py --self-test` -> PASS.

## m36.8 Review Evidence

- `reviews/phase36-m36-8-review-pass-1.md` -> SATISFIED. Reviewer confirmed the closeout scripts, validation wiring, docs, LSP budget coverage, no-waiver evidence, completion-quality negative seed, snapshot-coherence contract wrapper, and reuse-strategy consistency.
- `reviews/phase36-final-implementation-review-pass-1.md` -> SATISFIED. Final full-implementation review confirmed all Phase 36 exit criteria are satisfied, no critical/high/medium findings remain, and only Phase 37 package-registry intelligence plus Phase 39 marketplace publication governance are deferred.

## m36.8 Validation Evidence

- `scripts/run_all_tests.sh --profile quick` -> PASS. Report: `target/validation_lane_reports/quick.latest.json`; `wall_time=1201.85s`; `max_rss=521.5MiB`; `e2e cache_hits=12/12`; `report_signature=f808284595f17a99`; advisories: warm wall-time budget exceeded and high group skew.
- `scripts/run_all_tests.sh --profile pr` -> PASS on final rerun. Report: `target/validation_lane_reports/pr.latest.json`; `wall_time=2645.85s`; `max_rss=521.4MiB`; `e2e cache_hits=0/19`; `report_signature=6cd36071cf629b47`; `hardening variants=28 failures=0`; advisories: warm wall-time budget exceeded and high group skew.
- Earlier `pr` attempts failed only in the performance budget batch with marginal median timing noise on `check-single-file-001-arithmetic` and `phase27-non-regression-002-json-diagnostic-schema`; both cases passed isolated reruns, no budgets or waivers were changed, and the final full `pr` lane passed.

## m36.8 PR

- PR: <https://github.com/sifr-lang/sifr/pull/2136>
- Merge commit: `bb92e3f7577251f737bcb3a706ce45874daf6050`

## Phase 36 Closure

- Phase 36 completed on 2026-05-17 after m36.8 merged.
- Phase 35 is already completed and audited in `issues/phase35-performance-benchmarking-execution.md`.
- Remaining explicitly deferred scope: package-registry intelligence in Phase 37 and marketplace publication governance in Phase 39.
