# Sifr Compiler Roadmap (Execution Baseline)

This roadmap is the authoritative execution plan for the current hardening cycle.

## Scope Note
- Historical language-building phases (1-14) are preserved in existing phase docs.
- Active execution sequencing starts at **Phase 15** and runs through **Phase 30**.
- Legacy phase files from prior numbering are retained for historical context but are non-authoritative for current execution.

## Global Rules (from Phase 16 onward)
- Sequential execution only (one phase at a time).
- Local-first validation is authoritative; CI mirrors local commands/gates exactly.
- Local testing runs in parallel profiles (`quick`, `full`, `stress`) with deterministic output.
- Every bug fix includes a regression test before phase closure.
- `check` remains hard-separated from codegen/runtime.
- No user-triggerable compiler panics.
- No data-dependent emitted `.unwrap()`/`.expect()` in generated user runtime paths.
- Scoped fix-back (`N+1` discovering defect in `N`) is allowed only if minimal, documented, regression-tested, and revalidated.

## Phase Summary

| # | Phase | Status | Phase File | Unlocks |
|---|-------|--------|------------|---------|
| 15 | Baseline Reconciliation | planned | [15_baseline_reconciliation.md](./phases/15_baseline_reconciliation.md) | One canonical source of truth and signed execution contract |
| 16 | Local-First Test Platform Foundation | planned | [16_local_first_test_platform_foundation.md](./phases/16_local_first_test_platform_foundation.md) | Deterministic local parallel validation as primary gate |
| 17 | Import and Externals Correctness | planned | [17_import_and_externals_correctness.md](./phases/17_import_and_externals_correctness.md) | Correct import behavior across `check/run/build/test` |
| 18 | Project and CLI Semantics Correctness | planned | [18_project_and_cli_semantics_correctness.md](./phases/18_project_and_cli_semantics_correctness.md) | Predictable project-mode CLI behavior |
| 19 | Preview Distribution and Release Automation (`alpha`/`beta`) | planned | [19_preview_distribution_and_release_automation.md](./phases/19_preview_distribution_and_release_automation.md) | Early adopter distribution with controlled preview channels |
| 20 | Module Graph Safety, Determinism, and Cache | planned | [20_module_graph_safety_determinism_and_cache.md](./phases/20_module_graph_safety_determinism_and_cache.md) | Deterministic multi-module builds and faster local loops |
| 21 | HIR Decomposition and Maintainability Hardening | planned | [21_hir_decomposition_and_maintainability_hardening.md](./phases/21_hir_decomposition_and_maintainability_hardening.md) | Modular HIR architecture and anti-regrowth guardrails |
| 22 | Traversal Completeness and Control-Flow Correctness | planned | [22_traversal_completeness_and_control_flow_correctness.md](./phases/22_traversal_completeness_and_control_flow_correctness.md) | Correct walkers and full intended control-flow semantics |
| 23 | Type-System Soundness | planned | [23_type_system_soundness.md](./phases/23_type_system_soundness.md) | Sound generic/subtyping/variance behavior |
| 24 | Runtime-Safe Codegen Semantics | planned | [24_runtime_safe_codegen_semantics.md](./phases/24_runtime_safe_codegen_semantics.md) | Panic-safe generated runtime paths |
| 25 | Diagnostics, Error Recovery, Stability Contract, Panic-to-Diagnostic | planned | [25_diagnostics_error_recovery_and_stability_contract.md](./phases/25_diagnostics_error_recovery_and_stability_contract.md) | Production-grade diagnostics and stability guarantees |
| 26 | Verification Hardening | planned | [26_verification_hardening.md](./phases/26_verification_hardening.md) | Broad reliability evidence via regressions/fuzz/E2E |
| 27 | Stdlib Parity (Behavior + Complexity) | planned | [27_stdlib_parity_behavior_and_complexity.md](./phases/27_stdlib_parity_behavior_and_complexity.md) | Module-level behavior + complexity parity vs CPython |
| 28 | Performance Benchmarking and Budgets | planned | [28_performance_benchmarking_and_budgets.md](./phases/28_performance_benchmarking_and_budgets.md) | Enforced compile/runtime performance budgets |
| 29 | Stable Channel GA Promotion and Release Governance | planned | [29_stable_channel_ga_promotion_and_release_governance.md](./phases/29_stable_channel_ga_promotion_and_release_governance.md) | Governed stable release promotion and rollback policy |
| 30 | Resume Async/Web/FFI/Package/Tooling Expansion | planned | [30_resume_async_web_ffi_package_tooling_expansion.md](./phases/30_resume_async_web_ffi_package_tooling_expansion.md) | Feature expansion on hardened foundation |

## Milestone Index (Clear Milestones)

### Phase 15
- `milestone_15_1` Canonical Backlog Reconciliation
- `milestone_15_2` Phase Contract Definition
- `milestone_15_3` Stakeholder Sign-off Snapshot

### Phase 16
- `milestone_16_1` Parallel Test Profiles
- `milestone_16_2` Deterministic Reporting
- `milestone_16_3` CI-Parity and Smoke Hardening

### Phase 17
- `milestone_17_1` Frontend-Only Check Path
- `milestone_17_2` Non-Main Externals Resolution
- `milestone_17_3` Test and Constant Import Parity

### Phase 18
- `milestone_18_1` Run/Build Semantics Alignment
- `milestone_18_2` Auto-Detection Rule Tightening
- `milestone_18_3` CLI Contract and Regression Suite

### Phase 19
- `milestone_19_1` Installer and Channel Resolution
- `milestone_19_2` Artifact and Manifest Pipeline
- `milestone_19_3` Agentic Preview Release Command

### Phase 20
- `milestone_20_1` Dependency-Safe Module Ordering
- `milestone_20_2` Deterministic Assembly
- `milestone_20_3` Stdlib Cache for Local Loops

### Phase 21
- `milestone_21_1` Split `lower.rs`
- `milestone_21_2` Split `stdlib.rs`
- `milestone_21_3` Anti-Regrowth Guardrails

### Phase 22
- `milestone_22_1` Canonical Walker Coverage
- `milestone_22_2` `while ... else` End-to-End Support
- `milestone_22_3` Yield and Exception-Path Coverage

### Phase 23
- `milestone_23_1` TypeVar Constraint Enforcement
- `milestone_23_2` Inheritance and Variance Corrections
- `milestone_23_3` Optional Arithmetic Soundness

### Phase 24
- `milestone_24_1` Remove Data-Dependent `unwrap/expect`
- `milestone_24_2` Indexing and Semantics Parity Fixes
- `milestone_24_3` Defaults and Panic-to-Diagnostic Conversion

### Phase 25
- `milestone_25_1` Span and Diagnostic Schema Quality
- `milestone_25_2` Bounded Multi-Error Recovery
- `milestone_25_3` Stability Contract Finalization

### Phase 26
- `milestone_26_1` Regression Matrix Expansion
- `milestone_26_2` Fuzz and Property Scale-Out
- `milestone_26_3` Real-World E2E Parallel Gate

### Phase 27
- `milestone_27_1` Python Test Porting by Module
- `milestone_27_2` Behavioral Parity Classification
- `milestone_27_3` Complexity and Resource Audit vs CPython

### Phase 28
- `milestone_28_1` Baseline Benchmark Suite
- `milestone_28_2` Budget and Threshold Policy
- `milestone_28_3` Enforcement Integration

### Phase 29
- `milestone_29_1` Stable Promotion Policy
- `milestone_29_2` Rollback and Incident Governance
- `milestone_29_3` Release Sign-off Workflow

### Phase 30
- `milestone_30_1` Post-Hardening Replan
- `milestone_30_2` Gated Feature Kickoff
- `milestone_30_3` Expansion Governance Audit

## Dependency Chain

```mermaid
flowchart LR
    p15["Phase 15\nBaseline Reconciliation"] --> p16["Phase 16\nLocal-First Test Foundation"]
    p16 --> p17["Phase 17\nImport/Externals Correctness"]
    p17 --> p18["Phase 18\nProject/CLI Semantics"]
    p18 --> p19["Phase 19\nPreview Distribution (alpha/beta)"]
    p18 --> p20["Phase 20\nModule Graph + Determinism + Cache"]
    p20 --> p21["Phase 21\nHIR Decomposition"]
    p21 --> p22["Phase 22\nTraversal + Control Flow"]
    p22 --> p23["Phase 23\nType-System Soundness"]
    p23 --> p24["Phase 24\nRuntime-Safe Codegen"]
    p24 --> p25["Phase 25\nDiagnostics + Recovery + Stability"]
    p25 --> p26["Phase 26\nVerification Hardening"]
    p26 --> p27["Phase 27\nStdlib Parity + Complexity"]
    p27 --> p28["Phase 28\nPerformance Budgets"]
    p28 --> p29["Phase 29\nStable GA Governance"]
    p29 --> p30["Phase 30\nResume Feature Expansion"]
```

## Execution Discipline
- A phase is not complete when time is up; it is complete when the phase exit gate is met.
- Any scoped fix-back must be recorded in both affected phase docs.
- Merge decisions are based on local gate evidence first, CI second.
