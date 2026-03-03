# Phase 21: Traversal Completeness and Control-Flow Correctness

## Objective
Guarantee walkers and control-flow analyses cover all supported constructs correctly.

## Depends on
- Phase 20

## Milestones

### milestone_21_1: Canonical Walker Coverage
- Scope:
  - Standardize recursive traversal across statement/expression variants.
  - Remove partial traversal blind spots.
- Definition of done:
  - Traversal completeness matrix is satisfied for supported nodes.

### milestone_21_2: `while ... else` End-to-End Support
- Scope:
  - Implement intended Python-like `while ... else` semantics through HIR and codegen.
- Definition of done:
  - `while ... else` behavior matches language intent with regression tests.

### milestone_21_3: Yield and Exception-Path Coverage
- Scope:
  - Fix generator/yield detection across nested constructs.
  - Ensure try/except analysis includes loop-else and other missed paths.
- Definition of done:
  - No known missed traversal paths in generator/error analysis.

## Quality Contract
- Entry criteria: Phase 20 is completed and HIR decomposition guardrails are active.
- Exit criteria: Control-flow lowering/analysis is complete for supported syntax and semantics.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 21 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 21 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- Control-flow lowering/analysis is complete for supported syntax and semantics.
