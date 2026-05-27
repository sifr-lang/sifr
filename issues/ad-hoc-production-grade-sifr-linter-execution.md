# Ad Hoc Phase Execution: Production-Grade Sifr Linter

Status: completed on 2026-05-27

Phase contract: `issues/ad-hoc-production-grade-sifr-linter.md`

## Checklist

- [x] Phase plan reviewed and approved for implementation
- [x] Ruff linter reuse manifest created
- [x] Ruff rule-family and config-surface audit manifest created
- [x] Linter CLI parity manifest created
- [x] Forbidden Ruff/Python lint dependency guardrail completed
- [x] Lint config and file discovery completed
- [x] Parser-aware suppression engine completed
- [x] Phase-gated lint runner completed
- [x] Sifr policy rule families completed
- [x] Fix engine and LSP code actions completed
- [x] LSP/editor docs and contracts updated
- [x] Full local validation recorded
- [x] Final production-readiness review approved

## Planning Lock Addendum

This phase locks the lint/Ruff reuse decisions before implementation starts. Changing a reuse classification, config surface, suppression rule, rule namespace, or LSP action policy requires a reviewed planning update.

### Required Implementation Work

| ID | Work item | Required closeout |
| --- | --- | --- |
| W-1 | `sifr_lint` uses line-only suppression attachment. | Milestone 3 implements parser-aware statement/range suppression before syntax, HIR, or workspace rules ship. |
| W-2 | `sifr_lint` file discovery uses simple string/path matching. | Milestone 2 adapts Ruff-style glob/gitignore file discovery with Sifr defaults. |
| W-3 | Lint configuration is not loaded from `sifr.toml`. | Milestone 2 implements Sifr-owned lint config and override precedence. |
| W-4 | The lint runner is not phase-gated. | Milestone 4 implements Ruff-inspired phase gating behind Sifr APIs. |
| W-5 | LSP code-action gating can drift if it relies on diagnostic-code strings. | Milestone 6 adds typed hard-vs-policy diagnostic class and policy-only action gates. |
| W-6 | Fix-capable policy rules lack a production fix engine. | Milestone 6 adds applicability, edit isolation, conflict resolution, source-map tracking, and idempotence checks. |
| W-7 | The Ruff/Python lint reuse contract is not machine-enforced yet. | Milestone 1 adds `verification/tooling/check_linter_reuse_contract.py` with positive and negative self-tests. |
| W-8 | The parser-aware suppression gate is advisory until made mechanical. | Milestone 1 creates the gate manifest; Milestone 3 enables it; Milestone 5 cannot add non-physical-line rules unless the gate is closed. |
| W-9 | Ruff has many existing Python rule families and config keys; leaving their disposition to implementation would invite accidental porting. | Planning locks a full Ruff rule-family/config audit; Milestone 1 encodes it in `ruff_rule_config_audit.json`; Milestone 5 may only add rules/config from approved rows. |
| W-10 | `sifr lint` currently has a placeholder single-path CLI while Ruff's lint command has a production command surface. | Planning locks a Ruff-compatible `sifr lint [OPTIONS] [FILES]...` contract; Milestone 1 encodes it in `lint_cli_parity.json`; M2 and M6 implement the non-fix and fix portions. |
| W-11 | Pre-implementation discovery found minor M1/M2/M6 encoding details that should not be left implicit. | The phase now specifies rejected config-key manifest population, suppression-gate milestone identifier format, parser-aware API import checks, a lint CLI module split before file-size guardrail drift, and diagnostic-class code-action guardrails. |

### Locked Reuse Decisions

| Area | Locked decision |
| --- | --- |
| Ruff Python rules | Reject as production dependencies |
| Ruff rule registry contents | Reject; Sifr owns rule IDs/categories |
| Ruff config architecture | Adapt pattern, not Python options |
| Ruff file discovery | Reuse/adapt `ignore`, `globset`, path normalization, explicit-target behavior |
| Ruff linter orchestration | Adapt phase-gated structure |
| Ruff AST checker | Reference only; Sifr syntax/HIR checker is native |
| Ruff suppression engine | Adapt mapping/directive lookup concepts with Sifr syntax and rule IDs |
| Ruff fix engine | Adapt applicability/isolation/apply-fixes concepts |
| Ruff LSP code actions | Adapt deferred resolution, workspace edit tracking, settings patterns |
| Ruff Server diagnostics | Reference only; Sifr diagnostics remain canonical |
| Ruff rule families/config keys | Locked by planning audit; no implementation-time reinterpretation without reviewed phase update |
| Ruff `check` CLI | Adapt into `sifr lint`; `sifr check` remains hard compiler/type checking |

## Review Log

- `2026-05-26`: Claude review pass 1 found the high-level reuse boundary sound but identified parser-aware suppression as a blocker before adding non-line rules.
- `2026-05-26`: Claude review pass 2 confirmed the revised strategy is sound if parser-aware suppression is a documented gate before syntax/HIR/workspace rules.
- `2026-05-26`: Claude review pass 3 cross-checked current code and confirmed the reuse boundary is clean, with parser-aware suppression as the known prerequisite gate.
- `2026-05-26`: Claude subsystem reviews covered Ruff config, registry/rules, lint engine, suppression/fixes, file discovery/cache/path utilities, and LSP/editor integration. Findings are incorporated in the phase reuse matrix and milestones.
- `2026-05-26`: Claude phase review pass 1 found two planning blockers: the forbidden Ruff/Python lint dependency check needed a named enforceable guardrail, and the parser-aware suppression prerequisite needed a mechanical gate before syntax/HIR/workspace rules. The phase was updated to require `check_linter_reuse_contract.py`, a suppression-gate manifest, and rule-family enforcement.
- `2026-05-26`: Claude phase review pass 2 found the suppression-gate manifest and M3-to-M5 enforcement path were still underspecified. The phase was updated to define `verification/tooling/linter_manifests/suppression_gate.json`, its schema, the `physical_line_only` to `parser_aware` transition, and a single compile-time parser-aware suppression API dependency for non-physical-line rules.
- `2026-05-26`: Claude phase review pass 3 confirmed all pass-2 blockers are resolved and the phase is implementation-ready with no remaining blockers.
- `2026-05-26`: User review required Ruff rule/config decisions to be made during planning, not implementation. The phase was updated with a full Ruff rule-family audit, a config-surface audit, and a required `ruff_rule_config_audit.json` enforcement manifest.
- `2026-05-26`: Claude phase review pass 4 found the new rule-family audit complete and implementation-ready, with precision edits requested for the audit manifest schema and Ruff's deprecated `extend-ignore` surface. Both edits were applied before final review.
- `2026-05-26`: Claude phase review pass 5 verified the `extend-ignore` classification, audit manifest schema, rule/config audit completeness, and execution tracker update. The reviewer confirmed the phase is implementation-ready with no remaining blockers.
- `2026-05-26`: User review required explicit linter CLI decisions. The phase was updated with a Ruff-compatible `sifr lint [OPTIONS] [FILES]...` CLI parity contract, an exit-status contract, output-format decisions, stdin/discovery/fix/suppression decisions, and a required `lint_cli_parity.json` enforcement manifest.
- `2026-05-26`: Claude linter CLI review pass 1 found the CLI contract structurally sound and requested precision edits for hidden Ruff compatibility flags, `--extend-unfixable`, show-files/show-settings/statistics conflicts, manifest schema details, exit status fixtures, and suppression/statistics wording. The phase was updated with those decisions.
- `2026-05-26`: Claude linter CLI review pass 2 confirmed all pass-1 precision edits applied, manifest schema and validation obligations complete, all behavioral areas specified, and the plan is implementation-ready with no remaining blockers.
- `2026-05-26`: Claude linter CLI review pass 3 verified the final disposition spelling cleanup and confirmed the CLI plan is implementation-ready and elegant enough for implementation.
- `2026-05-27`: Claude pre-implementation discovery review found no blockers and confirmed M1 can start. It requested precision edits for manifest key population, suppression-gate milestone format, parser-aware API import checks, lint CLI file-size planning, and diagnostic-class code-action guardrails; the phase was updated accordingly.
- `2026-05-27`: Claude M1 reuse-contract review pass 1 found no blockers and returned `SATISFIED` for M1 closure. Review artifact: `reviews/sifr-linter-m1-reuse-contract-review-pass-1.md`.
- `2026-05-27`: Claude M2 config/discovery review pass 1 found no blockers and returned `SATISFIED` for M2 closure. Review artifact: `reviews/sifr-linter-m2-config-discovery-review-pass-1.md`.
- `2026-05-27`: Claude M3 parser-aware suppression review pass 1 found no blockers and returned `SATISFIED` for M3 closure. Review artifact: `reviews/sifr-linter-m3-parser-aware-suppression-review-pass-1.md`.
- `2026-05-27`: Claude M4 phase-gated runner review pass 1 found no blockers and returned `SATISFIED` for M4 closure. Review artifact: `reviews/sifr-linter-m4-phase-gated-runner-review-pass-1.md`.
- `2026-05-27`: Claude M5 policy rule families review pass 1 found no blockers and returned `SATISFIED` for M5 closure. Review artifact: `reviews/sifr-linter-m5-policy-rule-families-review-pass-1.md`.
- `2026-05-27`: M5 post-review quick validation exposed two closure blockers: direct `lower_module(` use from `sifr_lint` violated the split-brain guardrail, and the rule/suppression contract still expected lint diagnostics to exit 0. M5 was updated to route HIR access through `sifr_frontend::FrontendContext::hir_module_view` and to make the rule/suppression contract expect diagnostic exit code 1.
- `2026-05-27`: Claude M5 policy rule families review pass 2b rechecked the post-fix implementation, found no remaining blockers, and returned `SATISFIED` for M5 closure. Review artifact: `reviews/sifr-linter-m5-policy-rule-families-review-pass-2b.md`.
- `2026-05-27`: M6 pre-review implementation added safe fix application, fix-related lint CLI surfaces, typed policy/hard diagnostic-class LSP code-action gating, deferred fix-all resolution, stale-version rejection, and a diagnostic-class guardrail.
- `2026-05-27`: Claude M6 fix engine/code actions review pass 1 found no blockers and returned `SATISFIED` for M6 closure. Review artifact: `reviews/sifr-linter-m6-fix-engine-code-actions-review-pass-1.md`.
- `2026-05-27`: Claude M7 docs closeout review pass 1 found no blockers and returned `SATISFIED` for docs/contract closure, with final validation and final production-readiness review remaining as the only residual gates. Review artifact: `reviews/sifr-linter-m7-docs-closeout-review-pass-1.md`.
- `2026-05-27`: Claude final production-readiness review pass 1 found no blockers, returned `SATISFIED`, and approved closing the phase. Review artifact: `reviews/sifr-linter-final-production-readiness-review-pass-1.md`.

## Validation Log

- Validation evidence will be recorded per implementation milestone.
- Planning PR validation starts with `git diff --check` and docs/review artifact checks.
- `2026-05-27` M1 local checks:
  - `python3 verification/tooling/check_linter_reuse_contract.py` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py --self-test` passed.
  - `cargo test -p sifr_lint` passed: 3 unit tests and 0 doctests.
  - `git diff --check` passed.
  - `python3 scripts/check_file_size_guardrails.py` passed.
- `2026-05-27` M2 local checks:
  - `cargo check -p sifr` passed.
  - `cargo build -p sifr` passed.
  - `cargo clippy -p sifr_lint -- -D warnings` passed.
  - `cargo clippy -p sifr_lint -p sifr -- -D warnings` is blocked by pre-existing `clippy::too_many_arguments` in `crates/sifr/src/diagnostic_rendering_and_run.rs:219`, outside the M2 diff.
  - `cargo test -p sifr_lint` passed: 6 unit tests and 0 doctests.
  - `cargo test -p sifr -- --skip test_e2e_pass` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py --self-test` passed.
  - CLI smoke fixtures passed for concise diagnostics, stdin JSON diagnostics, config severity ignore, `--show-files`, `--show-settings`, `--exit-zero`, and rejected Ruff/Python flag handling.
  - `git diff --check` passed.
  - `python3 scripts/check_file_size_guardrails.py` passed.
- `2026-05-27` M3 local checks:
  - `cargo check -p sifr` passed.
  - `cargo build -p sifr` passed.
  - `cargo test -p sifr_lint` passed: 8 unit tests and 0 doctests.
  - `cargo test -p sifr -- --skip test_e2e_pass` passed.
  - `cargo clippy -p sifr_lint -- -D warnings` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py --self-test` passed.
  - CLI smoke fixture passed for `--ignore-suppressions`.
  - `python3 scripts/check_file_size_guardrails.py` passed.
- `2026-05-27` M4 pre-review local checks:
  - `cargo check -p sifr` passed.
  - `cargo build -p sifr` passed.
  - `cargo test -p sifr_lint` passed: 15 unit tests and 0 doctests.
  - `cargo test -p sifr -- --skip test_e2e_pass` passed.
  - `cargo clippy -p sifr_lint -- -D warnings` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py --self-test` passed.
  - `python3 scripts/check_file_size_guardrails.py` passed.
  - `git diff --check` passed.
- `2026-05-27` M5 pre-review local checks:
  - `cargo check -p sifr` passed.
  - `cargo build -p sifr` passed.
  - `cargo test -p sifr_lint` passed: 20 unit tests and 0 doctests.
  - `cargo test -p sifr_analysis` passed: 10 unit tests and 0 doctests.
  - `cargo test -p sifr -- --skip test_e2e_pass` passed.
  - `cargo clippy -p sifr_diagnostics -p sifr_lint -p sifr_analysis -- -D warnings` passed.
  - `cargo clippy -p sifr -- -D warnings` remains blocked only by the pre-existing `clippy::too_many_arguments` in `crates/sifr/src/diagnostic_rendering_and_run.rs:219`, outside the M5 diff.
  - `python3 verification/tooling/check_linter_reuse_contract.py` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py --self-test` passed.
  - `python3 scripts/check_file_size_guardrails.py` passed.
  - CLI smoke fixture passed for deterministic `--statistics` output and exit status.
  - `git diff --check` passed.
- `2026-05-27` M5 post-fix local checks:
  - `python3 verification/tooling/check_rule_suppression_contract.py` passed.
  - `python3 verification/tooling/check_rule_suppression_contract.py --self-test` passed.
  - `scripts/run_all_tests.sh --profile quick` passed after the split-brain and rule/suppression contract fixes. The lane reported wall-time budget advisories only, with no validation failures.
- `2026-05-27` M6 pre-review local checks:
  - `cargo check -p sifr_lint -p sifr_analysis -p sifr_lsp -p sifr` passed.
  - `cargo test -p sifr_lint` passed: 22 unit tests and 0 doctests.
  - `cargo test -p sifr_analysis` passed: 10 unit tests and 0 doctests.
  - `cargo test -p sifr_lsp` passed: 0 unit tests and 0 doctests.
  - `cargo test -p sifr -- --skip test_e2e_pass` passed.
  - `cargo clippy -p sifr_lint -p sifr_analysis -p sifr_lsp -- -D warnings` passed.
  - `cargo clippy -p sifr -- -D warnings` remains blocked only by the pre-existing `clippy::too_many_arguments` in `crates/sifr/src/diagnostic_rendering_and_run.rs:219`, outside the M6 diff.
  - `python3 verification/tooling/check_linter_reuse_contract.py` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py --self-test` passed.
  - `python3 verification/tooling/check_linter_diagnostic_class.py` passed.
  - `python3 verification/tooling/check_linter_diagnostic_class.py --self-test` passed.
  - `python3 verification/tooling/lsp_protocol_smoke.py` passed.
  - `python3 verification/tooling/lsp_protocol_smoke.py --self-test` passed.
  - `python3 verification/tooling/lsp_protocol_stress.py` passed.
  - `python3 verification/tooling/lsp_protocol_stress.py --self-test` passed.
  - CLI smoke fixtures passed for `--fix --show-fixes`, `--diff`, `--show-fixes`, and `--fix-only`.
  - `python3 scripts/check_file_size_guardrails.py` passed.
  - `git diff --check` passed.
  - `scripts/run_all_tests.sh --profile quick` passed. The lane reported warm wall-time and batching-skew advisories only, with no validation failures.
- `2026-05-27` M7 closeout local checks:
  - `python3 verification/tooling/check_tooling_contract_lock.py` passed.
  - `python3 verification/tooling/check_tooling_contract_lock.py --self-test` passed.
  - `python3 verification/tooling/check_vscode_extension_contract.py` passed.
  - `python3 verification/tooling/check_vscode_extension_contract.py --self-test` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py` passed.
  - `python3 verification/tooling/check_linter_reuse_contract.py --self-test` passed.
  - `python3 verification/tooling/check_linter_diagnostic_class.py` passed.
  - `python3 verification/tooling/check_linter_diagnostic_class.py --self-test` passed.
  - `python3 verification/tooling/check_rule_suppression_contract.py` passed.
  - `python3 verification/tooling/check_rule_suppression_contract.py --self-test` passed.
  - `python3 verification/tooling/check_editor_assets.py` passed.
  - `python3 verification/tooling/check_editor_assets.py --self-test` passed.
  - `python3 verification/tooling/lsp_protocol_smoke.py` passed.
  - `python3 verification/tooling/lsp_protocol_smoke.py --self-test` passed.
  - `python3 verification/tooling/lsp_protocol_stress.py` passed.
  - `python3 verification/tooling/lsp_protocol_stress.py --self-test` passed.
  - `python3 verification/tooling/check_phase36_closeout.py` passed.
  - `python3 verification/tooling/check_phase36_closeout.py --self-test` passed.
  - `cargo fmt --check` passed.
  - `python3 scripts/check_file_size_guardrails.py` passed.
  - `git diff --check` passed.
  - `scripts/run_all_tests.sh --profile quick` passed. The lane reported warm wall-time and batching-skew advisories only, with no validation failures.
  - `scripts/run_all_tests.sh` passed. The lane reported warm wall-time and batching-skew advisories only, with no validation failures.

## PR Log

Implementation PR links will be recorded here as each milestone closes.

- M1 `lint_reuse_contract_and_manifests`: https://github.com/sifr-lang/sifr/pull/2184
- M2 `lint_config_and_file_discovery`: https://github.com/sifr-lang/sifr/pull/2185
- M3 `parser_aware_suppression_engine`: https://github.com/sifr-lang/sifr/pull/2186
- M4 `phase_gated_lint_engine`: https://github.com/sifr-lang/sifr/pull/2187
- M5 `policy_rule_families`: https://github.com/sifr-lang/sifr/pull/2188
- M6 `lint_fixes_and_code_actions`: https://github.com/sifr-lang/sifr/pull/2189
- M7 `lsp_editor_docs_and_closeout`: https://github.com/sifr-lang/sifr/pull/2190
