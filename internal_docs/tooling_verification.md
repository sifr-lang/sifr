# Sifr Tooling Verification

status: phase36-contract-locked

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

The m36.1, m36.2, and m36.3 checks are wired into `scripts/run_all_tests.sh` under "Developer Tooling Checks".

## Required Later Checks

- `run_tooling_parity.py`
- `lsp_protocol_smoke.py`
- `lsp_protocol_stress.py`
- `check_editor_assets.py`
- `check_vscode_extension.py`
- completion quality fixtures and thresholds

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
