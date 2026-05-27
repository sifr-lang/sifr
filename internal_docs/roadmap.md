# Sifr Compiler Roadmap (Execution Baseline)

This roadmap is the authoritative execution plan for the current hardening and expansion cycle.

## Scope Note
- Historical language-building phases (1-14) are preserved in existing phase docs.
- Per review alignment, **Phase 12, 13, and 14 are completed**.
- Active sequencing is split into:
  - **Reliability and Architecture Track:** Phase 15 through Phase 31
  - **Expansion and Distribution Track:** Phase 32 through Phase 41

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
- Parse/lower/type-check/semantic-diagnostic logic must flow through one canonical frontend API; tool-specific semantic reimplementation is forbidden.
- Every top-level user-facing compiler diagnostic must carry a stable code and deterministic documentation URL of the form `https://sifr.sh/docs/errors/<CODE>`.
- No user-triggerable compiler panics.
- No data-dependent emitted `.unwrap()`/`.expect()` in generated user runtime paths.
- Scoped fix-back (`N+1` discovering defect in `N`) is allowed only if minimal, documented, regression-tested, and revalidated.
- Phase entry/exit gates, milestone quality checks, and mandatory local validation commands are embedded in each phase file (`15`-`41`) under `## Quality Contract`.

## Reliability and Architecture Track (Phase 15-31)

| # | Phase | Status | Phase File(s) | Unlocks |
|---|---|---|---|---|
| 15 | Baseline Reconciliation | completed | [15_baseline_reconciliation.md](./phases/15_baseline_reconciliation.md) | One canonical source of truth and signed execution contract |
| 16 | Local-First Test Platform Foundation | completed | [16_local_first_test_platform_foundation.md](./phases/16_local_first_test_platform_foundation.md) | Deterministic local parallel validation as primary gate |
| 17 | Import and Externals Correctness | completed | [17_import_and_externals_correctness.md](./phases/17_import_and_externals_correctness.md) | Correct import behavior across modes |
| 18 | Project and CLI Semantics Correctness | completed | [18_project_and_cli_semantics_correctness.md](./phases/18_project_and_cli_semantics_correctness.md) | Predictable single-file and project-mode behavior |
| 19 | Module Graph Safety, Determinism, and Cache | completed | [19_module_graph_safety_determinism_and_cache.md](./phases/19_module_graph_safety_determinism_and_cache.md) | Deterministic multi-module builds and faster local loops |
| 20 | HIR Decomposition and Maintainability Hardening | completed | [20_hir_decomposition_and_maintainability_hardening.md](./phases/20_hir_decomposition_and_maintainability_hardening.md) | Modular HIR architecture and anti-regrowth guardrails |
| 21 | Traversal Completeness and Control-Flow Correctness | completed | [21_traversal_completeness_and_control_flow_correctness.md](./phases/21_traversal_completeness_and_control_flow_correctness.md) | Correct walkers and intended control-flow semantics |
| 22 | Frontend Mode Parity Hardening | completed | [22_frontend_mode_parity_hardening.md](./phases/22_frontend_mode_parity_hardening.md) | Canonical frontend parity across `check/build/run/test` |
| 23 | Project Graph and Isolation Correctness | completed | [23_project_graph_and_isolation_correctness.md](./phases/23_project_graph_and_isolation_correctness.md) | Import-closure graph correctness and invocation isolation |
| 24 | HIR Analysis Consolidation | completed | [24_hir_analysis_consolidation.md](./phases/24_hir_analysis_consolidation.md) | Canonical traversal/query architecture |
| 25 | CFG/Flow Analysis Activation | completed | [25_cfg_flow_analysis_activation.md](./phases/25_cfg_flow_analysis_activation.md) | CFG-backed control-flow truths for correctness-critical analyses |
| 26 | Type-System Soundness Closure | completed | [26_type_system_soundness.md](./phases/26_type_system_soundness.md) | Sound generic/subtyping/variance and strict protocol bounds |
| 27 | Runtime Safety and Diagnostics Contract | completed, amended | [27_runtime_safe_codegen_semantics.md](./phases/27_runtime_safe_codegen_semantics.md), [27_diagnostics_error_recovery_and_stability_contract.md](./phases/27_diagnostics_error_recovery_and_stability_contract.md) | Panic-safe generation and stable diagnostic/recovery guarantees; diagnostic-code taxonomy and structured HIR diagnostic closure is amended by the ad-hoc semantic diagnostic phase |
| 28 | Decimal Semantics | completed | [28_decimal_type_and_exact_numeric_semantics.md](./phases/28_decimal_type_and_exact_numeric_semantics.md) | First-class exact decimal semantics |
| 29 | Verification Hardening | completed | [29_verification_hardening.md](./phases/29_verification_hardening.md) | Regression/fuzz/property/e2e hardening |
| 30 | Stdlib Parity (Behavior + Complexity) | completed | [30_reliability_parity_and_performance_budgets.md](./phases/30_reliability_parity_and_performance_budgets.md) | CPython parity governance for stdlib behavior and complexity; milestone_30_4 structural closure completed (2026-03-10) |
| 31 | Algorithmic Compatibility (LeetCode) | complete | [31_algorithmic_compatibility_and_leetcode_coverage.md](./phases/31_algorithmic_compatibility_and_leetcode_coverage.md) | Corpus-driven compatibility signal, remediation backlog, remediation wave 1, and scorecard/handoff landed on 2026-03-11 with external review sign-off recorded |
| 31.5 | Ad Hoc Python Source Parity and Builtin Stdlib Surface Closure | in_progress | [issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md](../issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md), [issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md](../issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md), [issues/ad-hoc-python-source-parity-extension-waiver-reduction.md](../issues/ad-hoc-python-source-parity-extension-waiver-reduction.md), [issues/ad-hoc-python-source-parity-extension-waiver-reduction-execution.md](../issues/ad-hoc-python-source-parity-extension-waiver-reduction-execution.md), [issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md](../issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md), [issues/ad-hoc-structured-data-and-class-surface-parity-expansion-execution.md](../issues/ad-hoc-structured-data-and-class-surface-parity-expansion-execution.md), [issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md](../issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md), [issues/ad-hoc-first-class-bytes-and-binary-surface-foundation-execution.md](../issues/ad-hoc-first-class-bytes-and-binary-surface-foundation-execution.md), [issues/ad-hoc-runtime-and-file-object-parity-expansion.md](../issues/ad-hoc-runtime-and-file-object-parity-expansion.md), [issues/ad-hoc-runtime-and-file-object-parity-expansion-execution.md](../issues/ad-hoc-runtime-and-file-object-parity-expansion-execution.md), [issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md](../issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md), [issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure-execution.md](../issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure-execution.md), [issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md](../issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md), [issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-execution.md](../issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-execution.md), [issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md](../issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md), [issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision-execution.md](../issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision-execution.md), [verification/stdlib/milestone_psp_7_parity_governance_inventory.md](../verification/stdlib/milestone_psp_7_parity_governance_inventory.md) | Root-cause-first closure of builtin/object-model/stdlib parity before Phase 32 continues through sequenced continuation phases. Closed continuations: parity-extension waiver reduction (`wave_psp_ext_1`-`wave_psp_ext_4`, 2026-03-18), structured/class parity (`wave_psp_struct_0`-`wave_psp_struct_4`), bytes foundation (`wave_psp_bytes_0`-`wave_psp_bytes_5`), runtime/file parity (`wave_psp_runtime_0`-`wave_psp_runtime_4`), canonical iteration closure (`wave_psp_iter_fix_0`-`wave_psp_iter_fix_8`), and stateful RNG/crypto closure (`wave_psp_rng_0`-`wave_psp_rng_3`) with phase-level production-grade approval on 2026-03-21. Closed corrective continuation: ownership-aware collection lowering and clone elision (`wave_clone_0` baseline/architecture lock, `wave_clone_1` iterator/comprehension ownership correction, `wave_clone_2` index/slice/star-unpack ownership correction, and `wave_clone_3` generic hardening/regression lock completed on 2026-03-21; wave-closure pass-1/pass-2, milestone-closure pass-1/pass-2, and phase-closure pass-1/pass-2 production-grade reviews approved on 2026-03-21). |
| 31.6 | Ad Hoc Sifr Workspace Resolution Via `sifr.toml` | closed | [issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md](../issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md), [issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md](../issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md) | Shipped native `sifr.toml` workspace discovery, workspace-rooted user imports for non-`main.sifr` entries, dotted module materialization, validation fixtures, and the LeetCode helper pilot without stdlib pollution. |
| 31.7 | Ad Hoc Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics | completed | [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) | Corrective Phase 27 addendum completed on 2026-05-03: phase-derived diagnostic buckets replaced with `SIFR-<FAMILY>-dddd`, `sifr_diagnostics` owns the shared model/schema/docs, HIR raw diagnostic transport is removed, and guardrails enforce code coverage, docs/schema sync, baseline hygiene, cancel usage, and transport cleanup. |
| 32.1 | Ad Hoc Async Effect And Offload Diagnostics | completed | [issues/ad-hoc-async-effect-and-offload-diagnostics.md](../issues/ad-hoc-async-effect-and-offload-diagnostics.md), [issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md](../issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md) | Corrective Phase 32 seal completed on 2026-05-12: async functions and awaits must have real suspension effects, direct annotated blocking/CPU-heavy calls in async code are errors, and blocking offload requires classified `@blocking_io`, `@cpu_heavy`, or known stdlib work. |

## Expansion and Distribution Track (Phase 32-41)

| # | Phase | Status | Phase File | Unlocks |
|---|---|---|---|---|
| 32 | Async and Ecosystem Foundation | completed | [32_async_ecosystem.md](./phases/32_async_ecosystem.md) | Async runtime and ecosystem primitives completed on 2026-05-11: typed async/await, structured task scopes, cancellation/timeout semantics, synchronization/channels, explicit blocking offload, async context/iteration/generator/comprehension support, and limited runtime-neutral compatibility veneers are implemented with deferred APIs documented by negative fixtures. |
| 33 | Preview Distribution and Release Automation (`alpha`/`beta`) | completed | [33_preview_distribution_and_release_automation.md](./phases/33_preview_distribution_and_release_automation.md) | Preview installer dispatchers, checksum-verified immutable installers, artifact packaging, `/create-new-version` automation, and public `0.1.0-alpha.1`/`0.1.0-beta.1` releases completed on 2026-05-12 |
| 34 | Generated Code Quality and Production Readiness | completed, audited | [34_generated_code_quality_and_production_readiness.md](./phases/34_generated_code_quality_and_production_readiness.md) | Deterministic, lint-clean, production-safe generated Rust; post-closure emitted-code audit covered all demos and LeetCode fixtures on 2026-05-14 |
| 35 | Performance Benchmarking and Shared Analysis Query Architecture | completed, audited | [35_performance_benchmarking_and_budgets.md](./phases/35_performance_benchmarking_and_budgets.md) | Performance budgets plus the canonical syntax/frontend/query foundation required by production tooling completed on 2026-05-17 |
| 36 | Production Developer Tooling and Editor Ecosystem | completed, audited | [36_developer_tooling_and_ecosystem_hooks.md](./phases/36_developer_tooling_and_ecosystem_hooks.md) | Native LSP, formatter/linter policy surfaces, packageable VS Code extension, multi-editor assets, completion-quality gates, and anti-split-brain tooling validation completed on 2026-05-17 |
| 36.1 | Ad Hoc Production-Grade Sifr Formatter | completed, audited | [ad-hoc-production-grade-sifr-formatter.md](../issues/ad-hoc-production-grade-sifr-formatter.md), [ad-hoc-production-grade-sifr-formatter-execution.md](../issues/ad-hoc-production-grade-sifr-formatter-execution.md) | Ruff-backed formatter parity for Sifr syntax, configuration, CLI, analysis, LSP, editor integrations, formatter corpus guardrails, performance budgets, and public/internal docs completed with final production-readiness review on 2026-05-26 |
| 36.2 | Ad Hoc Production-Grade Sifr Linter | in progress | [ad-hoc-production-grade-sifr-linter.md](../issues/ad-hoc-production-grade-sifr-linter.md), [ad-hoc-production-grade-sifr-linter-execution.md](../issues/ad-hoc-production-grade-sifr-linter-execution.md) | Ruff-informed but Sifr-owned lint config, rule registry, suppressions, file discovery, phase-gated engine, fixes, LSP diagnostics, and editor code actions; M1-M3 are merged and M4 phase-gated runner work is under review |
| 37 | Package Management | completed, audited | [37_package_management.md](./phases/37_package_management.md) | Cargo-backed package graph, workspace selection, package archive validation, publishing/vendoring delegation, and package guardrails completed on 2026-05-19 |
| 38 | Docs and Documentation | draft | [38_docs_and_documentation.md](./phases/38_docs_and_documentation.md) | Canonical versioned docs and local docs quality gates |
| 39 | Stable Channel GA Promotion and Release Governance | planned | [39_stable_channel_ga_promotion_and_release_governance.md](./phases/39_stable_channel_ga_promotion_and_release_governance.md) | Governed stable release promotion and rollback policy |
| 40 | Typed Data Model and Validation (Pydantic-Parity Track) | planned | [40_typed_data_model_and_validation.md](./phases/40_typed_data_model_and_validation.md) | Typed model/validation layer with stable error contracts |
| 41 | Web Framework and Platform Expansion | planned | [41_web_framework_and_platform_expansion.md](./phases/41_web_framework_and_platform_expansion.md) | Web stack with typed extractors and platform expansion baseline |

## Dependency Chain

```mermaid
flowchart LR
    p15["Phase 15\nBaseline Reconciliation"] --> p16["Phase 16\nLocal-First Test Foundation"]
    p16 --> p17["Phase 17\nImport and Externals"]
    p17 --> p18["Phase 18\nProject and CLI Semantics"]
    p18 --> p19["Phase 19\nModule Graph + Cache"]
    p19 --> p20["Phase 20\nHIR Decomposition"]
    p20 --> p21["Phase 21\nTraversal + Control Flow"]
    p21 --> p22["Phase 22\nFrontend Mode Parity"]
    p22 --> p23["Phase 23\nProject Graph + Isolation"]
    p23 --> p24["Phase 24\nHIR Analysis Consolidation"]
    p24 --> p25["Phase 25\nCFG/Flow Activation"]
    p25 --> p26["Phase 26\nType-System Soundness"]
    p26 --> p27["Phase 27\nRuntime Safety + Diagnostics"]
    p27 --> p28["Phase 28\nDecimal Semantics"]
    p28 --> p29["Phase 29\nVerification Hardening"]
    p29 --> p30["Phase 30\nStdlib Parity"]
    p30 --> p31["Phase 31\nAlgorithmic Compatibility"]
    p31 --> p32["Phase 32\nAsync and Ecosystem"]
    p32 --> p33["Phase 33\nPreview Distribution"]
    p33 --> p34["Phase 34\nGenerated Code Quality"]
    p34 --> p35["Phase 35\nPerformance + Shared Analysis Query"]
    p35 --> p36["Phase 36\nProduction Developer Tooling"]
    p36 --> p37["Phase 37\nPackage Management"]
    p37 --> p38["Phase 38\nDocs and Documentation"]
    p38 --> p39["Phase 39\nStable GA Governance"]
    p39 --> p40["Phase 40\nTyped Data Model + Validation"]
    p40 --> p41["Phase 41\nWeb + Platform Expansion"]
```

## Execution Discipline
- A phase is complete only when its exit gate is met.
- Any scoped fix-back must be recorded in both affected phase docs.
- Merge decisions are based on local gate evidence first, CI second.

## Deferred Planning Drafts (Need Alignment)
- Phase 42 and Phase 43 are intentionally excluded from the main execution table until post-Phase-41 planning lock.
- [42_data_science_ml.md](./phases/42_data_science_ml.md) (needs more planning)
- [43_interoperability.md](./phases/43_interoperability.md) (needs more planning)
