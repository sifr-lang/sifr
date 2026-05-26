# Sifr Tooling Verification

status: phase36-m36.8-closeout

## Verification Directory

Phase 36 owns `verification/tooling/`.

m36.1 adds contract checks and guardrail seeds. Later milestones add formatter, rule, analysis, LSP, editor asset, VS Code, parity, completion quality, stress, and performance checks.

## m36.1 Checks

Required m36.1 commands:

```bash
python3 verification/tooling/check_tooling_contract_lock.py
python3 verification/tooling/check_tooling_contract_lock.py --self-test
python3 verification/tooling/check_tooling_dependency_boundaries.py
python3 verification/tooling/check_tooling_dependency_boundaries.py --self-test
python3 verification/tooling/check_lsp_split_brain.py
python3 verification/tooling/check_lsp_split_brain.py --self-test
python3 verification/tooling/check_vscode_extension_contract.py
python3 verification/tooling/check_vscode_extension_contract.py --self-test
```

## m36.2 Checks

Required m36.2 commands:

```bash
python3 verification/tooling/check_formatter_contract.py
python3 verification/tooling/check_formatter_contract.py --self-test
python3 verification/tooling/check_rule_suppression_contract.py
python3 verification/tooling/check_rule_suppression_contract.py --self-test
```

## m36.3 Checks

Required m36.3 commands:

```bash
python3 verification/tooling/check_analysis_snapshot_contract.py
python3 verification/tooling/check_analysis_snapshot_contract.py --self-test
python3 verification/tooling/check_analysis_split_brain.py
python3 verification/tooling/check_analysis_split_brain.py --self-test
```

## m36.4 Checks

Required m36.4 commands:

```bash
python3 verification/tooling/run_tooling_parity.py
python3 verification/tooling/run_tooling_parity.py --self-test
```

The m36.1, m36.2, m36.3, and m36.4 checks are wired into `scripts/run_all_tests.sh` under "Developer Tooling Checks".

## m36.5 Checks

Required m36.5 commands:

```bash
python3 verification/tooling/lsp_protocol_smoke.py
python3 verification/tooling/lsp_protocol_smoke.py --self-test
python3 verification/tooling/lsp_protocol_stress.py
python3 verification/tooling/lsp_protocol_stress.py --self-test
python3 verification/tooling/check_lsp_split_brain.py
python3 verification/tooling/check_lsp_split_brain.py --self-test
python3 verification/tooling/check_tooling_dependency_boundaries.py
python3 verification/tooling/check_tooling_dependency_boundaries.py --self-test
```

`scripts/run_all_tests.sh` now runs the protocol smoke/stress checks under
"Developer Tooling Checks". The Phase 35 performance gate also includes
`lsp-query-001-request-families` and budget id `perf.lsp.request_families`.

## m36.6 Checks

Required m36.6 commands:

```bash
python3 verification/tooling/check_editor_assets.py
python3 verification/tooling/check_editor_assets.py --self-test
```

`check_editor_assets.py` verifies the checked-in Neovim, Zed, Helix, and Emacs
assets register Sifr files, launch `sifr lsp --stdio`, avoid Python/Ruff
fallbacks and semantics-bearing code, and keep the TextMate grammar scope map
covered by the `sifr_syntax` token fixtures.

The m36.6 checks are wired into `scripts/run_all_tests.sh` under "Developer
Tooling Checks".

## m36.7 Checks

Required m36.7 commands:

```bash
python3 verification/tooling/check_vscode_extension_contract.py --require-extension-repo
python3 verification/tooling/check_vscode_extension_contract.py --self-test
python3 verification/tooling/check_vscode_extension.py
python3 verification/tooling/check_vscode_extension.py --self-test
```

`check_vscode_extension.py` locates `editor_integrations/vscode`,
`SIFR_VSCODE_REPO`, or sibling `../sifr-vscode`, validates required package
metadata and syntax assets, runs the extension repo's lint, typecheck, unit
test, extension smoke test, and package scripts, and checks that
`dist/sifr-vscode-0.0.0.vsix` is produced.

The m36.7 package check is wired into `scripts/run_all_tests.sh` under
"Developer Tooling Checks".

## m36.8 Checks

Required m36.8 commands:

```bash
python3 verification/tooling/check_analysis_snapshot_coherence.py
python3 verification/tooling/check_analysis_snapshot_coherence.py --self-test
python3 verification/tooling/check_completion_quality.py
python3 verification/tooling/check_completion_quality.py --self-test
python3 verification/tooling/check_phase36_closeout.py
python3 verification/tooling/check_phase36_closeout.py --self-test
scripts/run_all_tests.sh --profile quick
scripts/run_all_tests.sh --profile pr
```

`check_analysis_snapshot_coherence.py` preserves the phase-contract script name
and delegates to the concrete `AnalysisHost` stale-version, stale-snapshot, and
revision-boundary evidence in `check_analysis_snapshot_contract.py`.

`check_completion_quality.py` validates the checked-in completion ranking
fixtures and thresholds from `verification/tooling/completion_quality/`, reruns
the required `sifr_analysis` cargo evidence, and fails on a seeded top-candidate
regression.

`check_phase36_closeout.py` verifies that all Phase 36 tooling and performance
checks are present, wired into `scripts/run_all_tests.sh`, documented, backed by
the LSP request-family budget, and free of active LSP performance waivers.

## Formatter Hardening Checks

The ad-hoc production-grade formatter phase extends the Phase 36 formatter
foundation with Ruff-backed formatting, coverage manifests, a checked formatter
corpus, and performance budgets.

Required formatter closeout commands:

```bash
python3 verification/tooling/check_formatter_contract.py
python3 verification/tooling/check_formatter_contract.py --self-test
python3 verification/tooling/check_formatter_phase_manifests.py
python3 verification/tooling/check_formatter_phase_manifests.py --self-test
python3 verification/tooling/check_formatter_ast_coverage.py
python3 verification/tooling/check_formatter_ast_coverage.py --self-test
python3 verification/performance/run_benchmarks.py --validate-only
python3 verification/performance/check_budgets.py
```

`check_formatter_ast_coverage.py` discovers Sifr-specific parser/AST extension
markers in the Ruff fork, requires concrete Ruff formatter fixtures and Sifr
wrapper corpus fixtures for each covered extension, runs corpus idempotence and
config matrix checks, and fails unresolved `pending:*` coverage rows. The quick
validation lane runs the formatter contract, formatter manifests, AST coverage
guardrail, editor asset guardrail, and formatter performance budget subset.

Each script must pass on positive fixtures and fail on seeded negative fixtures.

## Contract Lock

`check_tooling_contract_lock.py` verifies:

- required docs exist
- crate names are locked
- LSP protocol matrix covers every required method and command
- diagnostic modes, semantic token legend, settings, code-action kinds, and unsupported protocol surfaces are present
- VS Code extension contract exists and records the separate repository boundary

## Dependency Boundary

`check_tooling_dependency_boundaries.py` rejects forbidden production tooling dependencies and Python semantic authority:

- `ty_python_semantic`
- Python project semantics from `ty_project`
- `ruff_server` semantic behavior
- Pyright and Python language servers
- raw parser/lower/type-check entrypoints in tooling crates or editor adapters

## LSP Split-Brain Boundary

`check_lsp_split_brain.py` rejects direct semantic answers in `sifr_lsp`. LSP handlers must route through `sifr_analysis`; they must not parse, lower, type-check, traverse HIR, derive diagnostics, format, lint, or codegen independently.

## VS Code Boundary

`check_vscode_extension_contract.py` validates the main-repo extension contract in m36.1. Once m36.7 activates extension repository validation, it also fails if the extension checkout is missing or if the extension manifest/scripts/settings/commands drift from the contract.
