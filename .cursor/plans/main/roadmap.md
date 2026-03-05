# Sifr Compiler Roadmap (Execution Baseline)

This roadmap is the authoritative execution plan for the current hardening and expansion cycle.

## Scope Note
- Historical language-building phases (1-14) are preserved in existing phase docs.
- Per review alignment, **Phase 12, 13, and 14 are completed**.
- Active sequencing is split into:
  - **Reliability Track:** Phase 15 through Phase 27
  - **Feature Track:** Phase 28 through Phase 37

## Historical Status Clarifications

| Phase | Status | Note |
|---|---|---|
| 5 | superseded | Superseded by Phase 10 (`10_borrow_by_default.md`); kept only as historical reference |
| 12 | completed | Stdlib remediation closeout |
| 13 | completed | Type system completion closeout |
| 14 | completed | Codegen architecture closeout |

## Global Rules (from Phase 15 onward)
- Sequential execution only (one phase at a time).
- Local-first validation is authoritative; CI mirrors local commands/gates exactly.
- Local testing runs in parallel profiles (`quick`, `full`, `stress`) with deterministic output.
- Every bug fix includes a regression test before phase closure.
- `check` remains hard-separated from codegen/runtime.
- No user-triggerable compiler panics.
- No data-dependent emitted `.unwrap()`/`.expect()` in generated user runtime paths.
- Scoped fix-back (`N+1` discovering defect in `N`) is allowed only if minimal, documented, regression-tested, and revalidated.
- Phase entry/exit gates, milestone quality checks, and mandatory local validation commands are embedded in each phase file (`15`-`37`) under `## Quality Contract`.

## Reliability Track (Phase 15-27)

| # | Phase | Status | Phase File | Unlocks |
|---|---|---|---|---|
| 15 | Baseline Reconciliation | completed | [15_baseline_reconciliation.md](./phases/15_baseline_reconciliation.md) | One canonical source of truth and signed execution contract |
| 16 | Local-First Test Platform Foundation | completed | [16_local_first_test_platform_foundation.md](./phases/16_local_first_test_platform_foundation.md) | Deterministic local parallel validation as primary gate |
| 17 | Import and Externals Correctness | completed | [17_import_and_externals_correctness.md](./phases/17_import_and_externals_correctness.md) | Correct import behavior across `check/run/build/test`, including explicit import-form semantics |
| 18 | Project and CLI Semantics Correctness | completed | [18_project_and_cli_semantics_correctness.md](./phases/18_project_and_cli_semantics_correctness.md) | Predictable project-mode CLI behavior with explicit resolver trigger matrix |
| 19 | Module Graph Safety, Determinism, and Cache | planned | [19_module_graph_safety_determinism_and_cache.md](./phases/19_module_graph_safety_determinism_and_cache.md) | Deterministic multi-module builds and faster local loops |
| 20 | HIR Decomposition and Maintainability Hardening | planned | [20_hir_decomposition_and_maintainability_hardening.md](./phases/20_hir_decomposition_and_maintainability_hardening.md) | Modular HIR architecture and anti-regrowth guardrails |
| 21 | Traversal Completeness and Control-Flow Correctness | planned | [21_traversal_completeness_and_control_flow_correctness.md](./phases/21_traversal_completeness_and_control_flow_correctness.md) | Correct walkers and intended control-flow semantics |
| 22 | Type-System Soundness | planned | [22_type_system_soundness.md](./phases/22_type_system_soundness.md) | Sound generic/subtyping/variance behavior |
| 23 | Runtime-Safe Codegen Semantics | planned | [23_runtime_safe_codegen_semantics.md](./phases/23_runtime_safe_codegen_semantics.md) | Panic-safe generated runtime paths |
| 24 | Diagnostics, Error Recovery, Stability Contract, Panic-to-Diagnostic | planned | [24_diagnostics_error_recovery_and_stability_contract.md](./phases/24_diagnostics_error_recovery_and_stability_contract.md) | Production-grade diagnostics and stability guarantees |
| 25 | Decimal Type and Exact Numeric Semantics | planned | [25_decimal_type_and_exact_numeric_semantics.md](./phases/25_decimal_type_and_exact_numeric_semantics.md) | First-class exact decimal semantics with deterministic behavior and explicit conversion policy |
| 26 | Verification Hardening | planned | [26_verification_hardening.md](./phases/26_verification_hardening.md) | Broad reliability evidence via regressions/fuzz/E2E |
| 27 | Reliability Parity (Stdlib) | planned | [27_reliability_parity_and_performance_budgets.md](./phases/27_reliability_parity_and_performance_budgets.md) | Stdlib parity evidence and governed parity waivers |

## Feature Track (Phase 28-37)

| # | Phase | Status | Phase File | Unlocks |
|---|---|---|---|---|
| 28 | Async and Ecosystem Foundation | planned | [28_async_ecosystem.md](./phases/28_async_ecosystem.md) | Async runtime and ecosystem foundation |
| 29 | Preview Distribution and Release Automation (`alpha`/`beta`) | planned | [29_preview_distribution_and_release_automation.md](./phases/29_preview_distribution_and_release_automation.md) | Early adopter distribution with controlled preview channels |
| 30 | Generated Code Quality and Production Readiness | planned | [30_generated_code_quality_and_production_readiness.md](./phases/30_generated_code_quality_and_production_readiness.md) | Production-grade emission safety, determinism, and tooling compliance |
| 31 | Performance Benchmarking and Budgets | planned | [31_performance_benchmarking_and_budgets.md](./phases/31_performance_benchmarking_and_budgets.md) | Compiler performance baselines, thresholds, and enforcement gates |
| 32 | Developer Tooling and Ecosystem Hooks | planned | [32_developer_tooling_and_ecosystem_hooks.md](./phases/32_developer_tooling_and_ecosystem_hooks.md) | Tooling hooks aligned with phase contracts |
| 33 | Package Management | draft | [33_package_management.md](./phases/33_package_management.md) | Deterministic dependency and lockfile workflows |
| 34 | Docs and Documentation | draft | [34_docs_and_documentation.md](./phases/34_docs_and_documentation.md) | Canonical versioned docs and local docs quality gates |
| 35 | Stable Channel GA Promotion and Release Governance | planned | [35_stable_channel_ga_promotion_and_release_governance.md](./phases/35_stable_channel_ga_promotion_and_release_governance.md) | Governed stable release promotion and rollback policy |
| 36 | Typed Data Model and Validation (Pydantic-Parity Track) | planned | [36_typed_data_model_and_validation.md](./phases/36_typed_data_model_and_validation.md) | Dedicated typed model/validation layer |
| 37 | Web Framework and Platform Expansion | planned | [37_web_framework_and_platform_expansion.md](./phases/37_web_framework_and_platform_expansion.md) | Web stack plus scoped data/ML and interoperability expansion |

## Dependency Chain

```mermaid
flowchart LR
    p15["Phase 15\nBaseline Reconciliation"] --> p16["Phase 16\nLocal-First Test Foundation"]
    p16 --> p17["Phase 17\nImport and Externals"]
    p17 --> p18["Phase 18\nProject and CLI Semantics"]
    p18 --> p19["Phase 19\nModule Graph + Cache"]
    p19 --> p20["Phase 20\nHIR Decomposition"]
    p20 --> p21["Phase 21\nTraversal + Control Flow"]
    p21 --> p22["Phase 22\nType-System Soundness"]
    p22 --> p23["Phase 23\nRuntime-Safe Codegen"]
    p23 --> p24["Phase 24\nDiagnostics + Recovery"]
    p24 --> p25["Phase 25\nDecimal Semantics"]
    p25 --> p26["Phase 26\nVerification Hardening"]
    p26 --> p27["Phase 27\nStdlib Parity Closeout"]
    p27 --> p28["Phase 28\nAsync and Ecosystem"]
    p28 --> p29["Phase 29\nPreview Distribution"]
    p29 --> p30["Phase 30\nGenerated Code Quality"]
    p30 --> p31["Phase 31\nPerformance Budgets"]
    p31 --> p32["Phase 32\nDeveloper Tooling"]
    p32 --> p33["Phase 33\nPackage Management"]
    p33 --> p34["Phase 34\nDocs and Documentation"]
    p34 --> p35["Phase 35\nStable GA Governance"]
    p35 --> p36["Phase 36\nTyped Data Model + Validation"]
    p36 --> p37["Phase 37\nWeb + Platform Expansion"]
```

## Execution Discipline
- A phase is complete only when its exit gate is met.
- Any scoped fix-back must be recorded in both affected phase docs.
- Merge decisions are based on local gate evidence first, CI second.

## Deferred Planning Drafts (Need Alignment)
- Phase 38 and Phase 39 are intentionally excluded from the main execution table until post-Phase-37 planning lock.
- [38_data_science_ml.md](./phases/38_data_science_ml.md) (needs more planning)
- [39_interoperability.md](./phases/39_interoperability.md) (needs more planning)
