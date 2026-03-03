# Phase Contracts (15-35)

Last updated: 2026-03-03  
Owner: Phase 15 (`milestone_15_2`)  
Status: active

## Contract Intent
This document defines explicit phase entry criteria, exit criteria, and mandatory local validation commands for Phases `15` through `35`.

Gate mapping rule:
- Every entry gate maps to at least one concrete command.
- Every exit gate maps to at least one concrete command.
- A phase is complete only when its exit validations pass and roadmap status is updated to `completed`.
- Milestone quality/coverage checks must be satisfied before phase closure (see milestone registry validation command).

## Command Baseline
- `ENTRY-CHECK`: `python scripts/phase_contract_gate_check.py --phase <N> --check entry`
- `EXIT-CHECK`: `python scripts/phase_contract_gate_check.py --phase <N> --check exit`
- `MILESTONE-REGISTRY-CHECK`: `python scripts/validate_milestone_registry_15_35.py`
- `FULL-SUITE`: `./scripts/run_all_tests.sh`

## Phase 15 Contract
- Entry criteria: Phase 14 is completed and phase-review findings are available.
- Exit criteria: Canonical source of truth is approved and locked for execution.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 15 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 15 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 16 Contract
- Entry criteria: Phase 15 is completed and canonical backlog/contracts are finalized.
- Exit criteria: Local parallel validation is trusted as primary, with CI parity confirmed.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 16 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 16 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 17 Contract
- Entry criteria: Phase 16 is completed and deterministic local profiles are in place.
- Exit criteria: Import semantics are correct and consistent in all execution modes.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 17 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 17 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 18 Contract
- Entry criteria: Phase 17 is completed and import/external behavior is stable.
- Exit criteria: CLI project semantics are stable, documented, and test-covered.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 18 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 18 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 19 Contract
- Entry criteria: Phase 18 is completed and project-mode semantics are stable.
- Exit criteria: Multi-module builds are deterministic, cycle-safe, and faster in local iteration.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 19 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 19 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 20 Contract
- Entry criteria: Phase 19 is completed and module graph determinism is enforced.
- Exit criteria: HIR layer is materially more maintainable with regression-safe modular structure.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 20 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 20 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 21 Contract
- Entry criteria: Phase 20 is completed and HIR decomposition guardrails are active.
- Exit criteria: Control-flow lowering/analysis is complete for supported syntax and semantics.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 21 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 21 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 22 Contract
- Entry criteria: Phase 21 is completed and traversal/control-flow behavior is stable.
- Exit criteria: Critical type-system soundness issues are resolved and regression-covered.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 22 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 22 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 23 Contract
- Entry criteria: Phase 22 is completed and type-system soundness baseline is met.
- Exit criteria: Generated code semantics are safe-by-default for supported language behavior.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 23 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 23 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 24 Contract
- Entry criteria: Phase 23 is completed and runtime-safe codegen invariants are active.
- Exit criteria: Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 24 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 24 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 25 Contract
- Entry criteria: Phase 24 is completed and diagnostic stability contract is in place.
- Exit criteria: Reliability hardening is broad, deterministic, and locally enforceable.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 25 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 25 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 26 Contract
- Entry criteria: Phase 25 is completed and verification hardening is active.
- Exit criteria: Reliability claims are backed by stdlib parity evidence with explicit parity governance.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 26 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 26 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 27 Contract
- Entry criteria: Phase 26 is completed and codegen architecture from Phase 14 remains intact.
- Exit criteria: Async runtime core, typed serialization core, sync primitives, and advanced async features are all delivered with regression coverage.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 27 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 27 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 28 Contract
- Entry criteria: Phase 27 is completed and async/runtime ecosystem primitives are stable.
- Exit criteria: Preview release lifecycle works reliably without enabling stable GA promotion.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 28 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 28 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 29 Contract
- Entry criteria: Phase 28 is completed and preview artifacts are reproducible.
- Exit criteria: Performance regressions are systematically detected and controlled.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 29 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 29 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 30 Contract
- Entry criteria: Phase 29 is completed and performance budgets are enforced.
- Exit criteria: Tooling hooks are coherent, stable, and aligned with current phase contracts.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 30 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 30 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 31 Contract
- Entry criteria: Phase 30 is completed and tooling contracts are stable.
- Exit criteria: Package management workflows are stable enough for broader ecosystem usage.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 31 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 31 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 32 Contract
- Entry criteria: Phase 31 is completed and package workflows are deterministic.
- Exit criteria: Core documentation is canonical, navigable, and quality-gated for ongoing phase execution and release usage.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 32 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 32 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 33 Contract
- Entry criteria: Phase 32 is completed and release-facing documentation is canonical.
- Exit criteria: Stable GA promotion is policy-driven, auditable, and reversible.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 33 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 33 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 34 Contract
- Entry criteria: Phase 33 is completed and release governance is active.
- Exit criteria: Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 34 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 34 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Phase 35 Contract
- Entry criteria: Phase 34 is completed and typed data model contracts are stable.
- Exit criteria: Web, data/ML, and interoperability MVP tracks are delivered without violating reliability/stability contracts.
- Entry validation: `python scripts/phase_contract_gate_check.py --phase 35 --check entry`
- Exit validation:
  - `python scripts/phase_contract_gate_check.py --phase 35 --check exit`
  - `python scripts/validate_milestone_registry_15_35.py`
  - `./scripts/run_all_tests.sh`

## Deferred Scope Note
- Phase 36 is intentionally excluded from this Phase 15 contract baseline.
