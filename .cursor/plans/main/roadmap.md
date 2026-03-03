# Sifr Compiler Roadmap (Execution Baseline)

This roadmap is the authoritative execution plan for the current hardening and expansion cycle.

## Scope Note
- Historical language-building phases (1-14) are preserved in existing phase docs.
- Active execution sequencing starts at **Phase 15** and runs through **Phase 37**.

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
| 17 | Async and Ecosystem Foundation | planned | [17_async_ecosystem.md](./phases/17_async_ecosystem.md) | Async runtime core and typed serialization core |
| 18 | Import and Externals Correctness | planned | [18_import_and_externals_correctness.md](./phases/18_import_and_externals_correctness.md) | Correct import behavior across `check/run/build/test` |
| 19 | Project and CLI Semantics Correctness | planned | [19_project_and_cli_semantics_correctness.md](./phases/19_project_and_cli_semantics_correctness.md) | Predictable project-mode CLI behavior |
| 20 | Preview Distribution and Release Automation (`alpha`/`beta`) | planned | [20_preview_distribution_and_release_automation.md](./phases/20_preview_distribution_and_release_automation.md) | Early adopter distribution with controlled preview channels |
| 21 | Module Graph Safety, Determinism, and Cache | planned | [21_module_graph_safety_determinism_and_cache.md](./phases/21_module_graph_safety_determinism_and_cache.md) | Deterministic multi-module builds and faster local loops |
| 22 | HIR Decomposition and Maintainability Hardening | planned | [22_hir_decomposition_and_maintainability_hardening.md](./phases/22_hir_decomposition_and_maintainability_hardening.md) | Modular HIR architecture and anti-regrowth guardrails |
| 23 | Traversal Completeness and Control-Flow Correctness | planned | [23_traversal_completeness_and_control_flow_correctness.md](./phases/23_traversal_completeness_and_control_flow_correctness.md) | Correct walkers and intended control-flow semantics |
| 24 | Type-System Soundness | planned | [24_type_system_soundness.md](./phases/24_type_system_soundness.md) | Sound generic/subtyping/variance behavior |
| 25 | Runtime-Safe Codegen Semantics | planned | [25_runtime_safe_codegen_semantics.md](./phases/25_runtime_safe_codegen_semantics.md) | Panic-safe generated runtime paths |
| 26 | Diagnostics, Error Recovery, Stability Contract, Panic-to-Diagnostic | planned | [26_diagnostics_error_recovery_and_stability_contract.md](./phases/26_diagnostics_error_recovery_and_stability_contract.md) | Production-grade diagnostics and stability guarantees |
| 27 | Verification Hardening | planned | [27_verification_hardening.md](./phases/27_verification_hardening.md) | Broad reliability evidence via regressions/fuzz/E2E |
| 28 | Stdlib Parity (Behavior + Complexity) | planned | [28_stdlib_parity_behavior_and_complexity.md](./phases/28_stdlib_parity_behavior_and_complexity.md) | Module-level behavior + complexity parity vs CPython |
| 29 | Performance Benchmarking and Budgets | planned | [29_performance_benchmarking_and_budgets.md](./phases/29_performance_benchmarking_and_budgets.md) | Enforced compile/runtime performance budgets |
| 30 | Developer Tooling and Ecosystem Hooks | planned | [30_developer_tooling_and_ecosystem_hooks.md](./phases/30_developer_tooling_and_ecosystem_hooks.md) | Tooling hooks aligned with phase contracts |
| 31 | Package Management | planned | [31_package_management.md](./phases/31_package_management.md) | Deterministic dependency and lockfile workflows |
| 32 | Docs and Documentation | planned | [32_docs_and_documentation.md](./phases/32_docs_and_documentation.md) | Canonical versioned docs and local docs quality gates |
| 33 | Stable Channel GA Promotion and Release Governance | planned | [33_stable_channel_ga_promotion_and_release_governance.md](./phases/33_stable_channel_ga_promotion_and_release_governance.md) | Governed stable release promotion and rollback policy |
| 34 | Typed Data Model and Validation (Pydantic-Parity Track) | planned | [34_typed_data_model_and_validation.md](./phases/34_typed_data_model_and_validation.md) | Dedicated typed model/validation layer |
| 35 | Web Framework and Typed Extractors | planned | [35_web_framework_and_typed_extractors.md](./phases/35_web_framework_and_typed_extractors.md) | Web stack with model-driven request validation/extractors |
| 36 | Data Science and ML | planned | [36_data_science_ml.md](./phases/36_data_science_ml.md) | Data and ML workflows on top of core platform |
| 37 | Interoperability | planned | [37_interoperability.md](./phases/37_interoperability.md) | FFI capabilities on top of stabilized foundations |

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
- `milestone_async_core`
- `milestone_typed_serde_core`
- `milestone_async_sync`
- `milestone_async_advanced`

### Phase 18
- `milestone_18_1` Frontend-Only Check Path
- `milestone_18_2` Non-Main Externals Resolution
- `milestone_18_3` Test and Constant Import Parity

### Phase 19
- `milestone_19_1` Run/Build Semantics Alignment
- `milestone_19_2` Auto-Detection Rule Tightening
- `milestone_19_3` CLI Contract and Regression Suite

### Phase 20
- `milestone_20_1` Installer and Channel Resolution
- `milestone_20_2` Artifact and Manifest Pipeline
- `milestone_20_3` Agentic Preview Release Command

### Phase 21
- `milestone_21_1` Dependency-Safe Module Ordering
- `milestone_21_2` Deterministic Assembly
- `milestone_21_3` Stdlib Cache for Local Loops

### Phase 22
- `milestone_22_1` Split `lower.rs`
- `milestone_22_2` Split `stdlib.rs`
- `milestone_22_3` Anti-Regrowth Guardrails

### Phase 23
- `milestone_23_1` Canonical Walker Coverage
- `milestone_23_2` `while ... else` End-to-End Support
- `milestone_23_3` Yield and Exception-Path Coverage

### Phase 24
- `milestone_24_1` TypeVar Constraint Enforcement
- `milestone_24_2` Inheritance and Variance Corrections
- `milestone_24_3` Optional Arithmetic Soundness

### Phase 25
- `milestone_25_1` Remove Data-Dependent `unwrap/expect`
- `milestone_25_2` Indexing and Semantics Parity Fixes
- `milestone_25_3` Defaults and Panic-to-Diagnostic Conversion

### Phase 26
- `milestone_26_1` Span and Diagnostic Schema Quality
- `milestone_26_2` Bounded Multi-Error Recovery
- `milestone_26_3` Stability Contract Finalization

### Phase 27
- `milestone_27_1` Regression Matrix Expansion
- `milestone_27_2` Fuzz and Property Scale-Out
- `milestone_27_3` Real-World E2E Parallel Gate

### Phase 28
- `milestone_28_1` Python Test Porting by Module
- `milestone_28_2` Behavioral Parity Classification
- `milestone_28_3` Complexity and Resource Audit vs CPython

### Phase 29
- `milestone_29_1` Baseline Benchmark Suite
- `milestone_29_2` Budget and Threshold Policy
- `milestone_29_3` Enforcement Integration

### Phase 30
- `milestone_30_1` Developer Tooling and Ecosystem Hooks

### Phase 31
- `milestone_31_1` Package Management

### Phase 32
- `milestone_32_1` Documentation Information Architecture
- `milestone_32_2` Reference and Contract Documentation
- `milestone_32_3` Documentation Quality Gates

### Phase 33
- `milestone_33_1` Stable Promotion Policy
- `milestone_33_2` Rollback and Incident Governance
- `milestone_33_3` Release Sign-off Workflow

### Phase 34
- `milestone_34_1` Typed Model Core
- `milestone_34_2` Validation Engine
- `milestone_34_3` Error Model and Diagnostics Contract
- `milestone_34_4` Parity and Compatibility Matrix

### Phase 35
- `milestone_35_1` Web Framework Core
- `milestone_35_2` Typed Extractors and Request Validation
- `milestone_35_3` Production Web Baseline

### Phase 36
- `milestone_36_1` Data Processing
- `milestone_36_2` ML Inference

### Phase 37
- `milestone_37_1` Interoperability (FFI)

## Dependency Chain

```mermaid
flowchart LR
    p15["Phase 15\nBaseline Reconciliation"] --> p16["Phase 16\nLocal-First Test Foundation"]
    p16 --> p17["Phase 17\nAsync Foundation"]
    p17 --> p18["Phase 18\nImport/Externals Correctness"]
    p18 --> p19["Phase 19\nProject/CLI Semantics"]
    p19 --> p20["Phase 20\nPreview Distribution"]
    p20 --> p21["Phase 21\nModule Graph + Cache"]
    p21 --> p22["Phase 22\nHIR Decomposition"]
    p22 --> p23["Phase 23\nTraversal + Control Flow"]
    p23 --> p24["Phase 24\nType-System Soundness"]
    p24 --> p25["Phase 25\nRuntime-Safe Codegen"]
    p25 --> p26["Phase 26\nDiagnostics + Recovery"]
    p26 --> p27["Phase 27\nVerification Hardening"]
    p27 --> p28["Phase 28\nStdlib Parity + Complexity"]
    p28 --> p29["Phase 29\nPerformance Budgets"]
    p29 --> p30["Phase 30\nDeveloper Tooling"]
    p30 --> p31["Phase 31\nPackage Management"]
    p31 --> p32["Phase 32\nDocs and Documentation"]
    p32 --> p33["Phase 33\nStable GA Governance"]
    p33 --> p34["Phase 34\nTyped Data Model + Validation"]
    p34 --> p35["Phase 35\nWeb Framework + Typed Extractors"]
    p35 --> p36["Phase 36\nData Science + ML"]
    p36 --> p37["Phase 37\nInteroperability"]
```

## Execution Discipline
- A phase is not complete when time is up; it is complete when the phase exit gate is met.
- Any scoped fix-back must be recorded in both affected phase docs.
- Merge decisions are based on local gate evidence first, CI second.
