# Sifr Tooling Verification

status: tooling readiness locked

## Verification Directory

developer tooling surface owns `verification/areas/developer_tooling/`.

tooling lock adds rules checks and guardrail seeds. Later tooling work adds formatter, rule, analysis, LSP, editor asset, VS Code, parity, completion quality, stress, and performance checks.

## tooling lock Checks

Required tooling lock commands:

```bash
python3 verification/areas/developer_tooling/check_tooling_lock.py
python3 verification/areas/developer_tooling/check_tooling_lock.py --self-test
python3 verification/areas/developer_tooling/check_tooling_dependency_boundaries.py
python3 verification/areas/developer_tooling/check_tooling_dependency_boundaries.py --self-test
python3 verification/areas/developer_tooling/check_lsp_split_brain.py
python3 verification/areas/developer_tooling/check_lsp_split_brain.py --self-test
python3 verification/areas/developer_tooling/check_vscode_extension_rules.py
python3 verification/areas/developer_tooling/check_vscode_extension_rules.py --self-test
```

## formatter/linter foundation Checks

Required formatter/linter foundation commands:

```bash
python3 verification/areas/developer_tooling/check_formatter_rules.py
python3 verification/areas/developer_tooling/check_formatter_rules.py --self-test
python3 verification/areas/developer_tooling/check_rule_suppression_rules.py
python3 verification/areas/developer_tooling/check_rule_suppression_rules.py --self-test
```

## analysis-host foundation Checks

Required analysis-host foundation commands:

```bash
python3 verification/areas/developer_tooling/check_analysis_snapshot_rules.py
python3 verification/areas/developer_tooling/check_analysis_snapshot_rules.py --self-test
python3 verification/areas/developer_tooling/check_analysis_split_brain.py
python3 verification/areas/developer_tooling/check_analysis_split_brain.py --self-test
```

## editor-query layer Checks

Required editor-query layer commands:

```bash
python3 verification/areas/developer_tooling/run_tooling_parity.py
python3 verification/areas/developer_tooling/run_tooling_parity.py --self-test
```

The tooling lock, formatter/linter foundation, analysis-host foundation, and editor-query layer checks are wired into `scripts/run_all_tests.sh` under "Developer Tooling Checks".

## LSP protocol layer Checks

Required LSP protocol layer commands:

```bash
python3 verification/areas/developer_tooling/lsp_protocol_smoke.py
python3 verification/areas/developer_tooling/lsp_protocol_smoke.py --self-test
python3 verification/areas/developer_tooling/lsp_protocol_stress.py
python3 verification/areas/developer_tooling/lsp_protocol_stress.py --self-test
python3 verification/areas/developer_tooling/check_lsp_split_brain.py
python3 verification/areas/developer_tooling/check_lsp_split_brain.py --self-test
python3 verification/areas/developer_tooling/check_tooling_dependency_boundaries.py
python3 verification/areas/developer_tooling/check_tooling_dependency_boundaries.py --self-test
```

`scripts/run_all_tests.sh` now runs the protocol smoke/stress checks under
"Developer Tooling Checks". The frontend query architecture performance gate also includes
LSP latency budget per-request `lsp-query-*` cases for cold start, diagnostics, completion,
hover, signature help, navigation, references, rename, semantic tokens, inlay
hints, selection range, type hierarchy, code actions, formatting, workspace
diagnostics, and generated Rust preview. `lsp-query-001-request-families` with
budget id `perf.lsp.request_families` remains aggregate smoke coverage only.

## editor asset layer Checks

Required editor asset layer commands:

```bash
python3 verification/areas/developer_tooling/check_editor_assets.py
python3 verification/areas/developer_tooling/check_editor_assets.py --self-test
```

`check_editor_assets.py` verifies the checked-in Neovim, Zed, Helix, and Emacs
assets register Sifr files, launch `sifr lsp --stdio`, avoid Python/Ruff
fallbacks and semantics-bearing code, and keep the TextMate grammar scope map
covered by the `sifr_syntax` token fixtures.

The editor asset layer checks are wired into `scripts/run_all_tests.sh` under "Developer
Tooling Checks".

## VS Code extension layer Checks

Required VS Code extension layer commands:

```bash
python3 verification/areas/developer_tooling/check_vscode_extension_rules.py --require-extension-repo
python3 verification/areas/developer_tooling/check_vscode_extension_rules.py --self-test
python3 verification/areas/developer_tooling/check_vscode_extension.py
python3 verification/areas/developer_tooling/check_vscode_extension.py --self-test
```

`check_vscode_extension.py` locates `editor_integrations/vscode`,
`SIFR_VSCODE_REPO`, or sibling `../sifr-vscode`, validates required package
metadata and syntax assets, runs the extension repo's lint, typecheck, unit
test, extension smoke test, and package scripts, and checks that
`dist/sifr-vscode-0.0.0.vsix` is produced.

The VS Code extension layer package check is wired into `scripts/run_all_tests.sh` under
"Developer Tooling Checks".

## tooling readiness Checks

Required tooling readiness commands:

```bash
python3 verification/areas/developer_tooling/check_analysis_snapshot_coherence.py
python3 verification/areas/developer_tooling/check_analysis_snapshot_coherence.py --self-test
python3 verification/areas/developer_tooling/check_completion_quality.py
python3 verification/areas/developer_tooling/check_completion_quality.py --self-test
python3 verification/areas/developer_tooling/check_tooling_readiness.py
python3 verification/areas/developer_tooling/check_tooling_readiness.py --self-test
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh --profile merge
```

`check_analysis_snapshot_coherence.py` preserves the tooling-rules script name
and delegates to the concrete `AnalysisHost` stale-version, stale-snapshot, and
revision-boundary evidence in `check_analysis_snapshot_rules.py`.

`check_completion_quality.py` validates the checked-in completion ranking
fixtures and thresholds from `verification/areas/developer_tooling/completion_quality/`, reruns
the required `sifr_analysis` cargo evidence, and fails on a seeded top-candidate
regression.

`check_tooling_readiness.py` verifies that all developer tooling surface tooling and performance
checks are present, wired into `scripts/run_all_tests.sh`, documented, backed by
the LSP request-family budgets, and free of active LSP performance waivers.

## Formatter Hardening Checks

The production-grade formatter work extends the developer tooling surface formatter
foundation with Ruff-backed formatting, coverage manifests, a checked formatter
corpus, and performance budgets.

Required formatter readiness commands:

```bash
python3 verification/areas/developer_tooling/check_formatter_rules.py
python3 verification/areas/developer_tooling/check_formatter_rules.py --self-test
python3 verification/areas/developer_tooling/check_formatter_manifests.py
python3 verification/areas/developer_tooling/check_formatter_manifests.py --self-test
python3 verification/areas/developer_tooling/check_formatter_ast_coverage.py
python3 verification/areas/developer_tooling/check_formatter_ast_coverage.py --self-test
python3 verification/areas/performance/run_benchmarks.py --validate-only
python3 verification/areas/performance/check_budgets.py
```

`check_formatter_ast_coverage.py` discovers Sifr-specific parser/AST extension
markers in the Ruff fork, requires concrete Ruff formatter fixtures and Sifr
wrapper corpus fixtures for each covered extension, runs corpus idempotence and
config matrix checks, and fails unresolved `pending:*` coverage rows. The quick
validation profile runs the formatter rules, formatter manifests, AST coverage
guardrail, editor asset guardrail, and formatter performance budget subset.

Each script must pass on positive fixtures and fail on seeded negative fixtures.

## Linter Hardening Checks

The production-grade linter work extends the developer tooling surface lint foundation
with Ruff-informed but Sifr-owned config, discovery, parser-aware suppressions,
stage-gated orchestration, policy rule families, safe fixes, and editor code
actions.

Required linter readiness commands:

```bash
python3 verification/areas/developer_tooling/check_linter_reuse_rules.py
python3 verification/areas/developer_tooling/check_linter_reuse_rules.py --self-test
python3 verification/areas/developer_tooling/check_rule_suppression_rules.py
python3 verification/areas/developer_tooling/check_rule_suppression_rules.py --self-test
python3 verification/areas/developer_tooling/check_linter_diagnostic_class.py
python3 verification/areas/developer_tooling/check_linter_diagnostic_class.py --self-test
python3 verification/areas/developer_tooling/lsp_protocol_smoke.py
python3 verification/areas/developer_tooling/lsp_protocol_smoke.py --self-test
python3 verification/areas/developer_tooling/lsp_protocol_stress.py
python3 verification/areas/developer_tooling/lsp_protocol_stress.py --self-test
```

`check_linter_reuse_rules.py` validates the Ruff rule/config/CLI manifests,
forbidden Python/Ruff lint dependencies, parser-aware suppression gate, rule
metadata, and implemented `sifr lint` option coverage. `check_linter_diagnostic_class.py`
fails if analysis or LSP code-action handlers gate policy actions by
`SIFR-LINT-` string prefixes instead of typed `Hard`/`Policy` diagnostic data.

The create-pr validation profile runs the linter reuse rules, rule/suppression
rules, diagnostic-class guardrail, LSP smoke/stress checks, editor asset
guardrail, VS Code extension package checks, and readiness checks.

## Rules Lock

`check_tooling_lock.py` verifies:

- required docs exist
- crate names are locked
- LSP protocol matrix covers every required method and command
- diagnostic modes, semantic token legend, settings, code-action kinds, and unsupported protocol surfaces are present
- VS Code extension rules exists and records the separate repository boundary

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

`check_vscode_extension_rules.py` validates the main-repo extension rules in tooling lock. Once VS Code extension layer activates extension repository validation, it also fails if the extension checkout is missing or if the extension manifest/scripts/settings/commands drift from the rules.
