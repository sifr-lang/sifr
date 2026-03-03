# Milestone Registry (Phases 15-35)

Last updated: 2026-03-03
Owner: Phase 15 (`milestone_15_2` detail closure)
Status: active

## Purpose
This registry guarantees that every milestone across phases 15-35 is explicitly tracked with scope and definition-of-done snapshots.

Mandatory local validation contract per milestone:
- `python scripts/phase_contract_gate_check.py --phase <N> --check entry`
- `python scripts/validate_milestone_registry_15_35.py`
- `./scripts/run_all_tests.sh`

## Phase 15: Baseline Reconciliation
Source: `.cursor/plans/main/phases/15_baseline_reconciliation.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_15_1` (Canonical Backlog Reconciliation) | Merge reviewer findings into one backlog.; Deduplicate overlaps and normalize severity (P0-P3).; Tag each item to owning future phase. | One canonical backlog file exists and is current.; No duplicate finding IDs remain. |
| `milestone_15_2` (Phase Contract Definition) | Define entry/exit criteria for Phases 15-35.; Define mandatory local validation expectations for each phase. | Every phase has explicit completion gates.; Every gate maps to at least one concrete validation step. |
| `milestone_15_3` (Stakeholder Sign-off Snapshot) | Review reconciled backlog + phase contracts.; Record explicit sign-off decision and open risks. | Sign-off recorded in plan docs.; Any deferred risks are linked to backlog issues. |

## Phase 16: Local-First Test Platform Foundation
Source: `.cursor/plans/main/phases/16_local_first_test_platform_foundation.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_16_1` (Parallel Test Profiles) | Define local profiles: quick, full, stress.; Make profile execution parallel-safe and reproducible. | Profiles run reliably on developer machines.; Profile purpose and runtime envelope are documented. |
| `milestone_16_2` (Deterministic Reporting) | Stabilize output ordering, summary format, and failure grouping.; Ensure reruns produce equivalent reports. | Identical inputs produce deterministic pass/fail summaries.; Failure reports are actionable and not order-noisy. |
| `milestone_16_3` (CI-Parity and Smoke Hardening) | Wire CI to run exact local scripts and flags.; Add always-on smoke fuzz/property jobs. | CI and local commands are 1:1.; Smoke fuzz/property checks run in default validation flow. |

## Phase 17: Import and Externals Correctness
Source: `.cursor/plans/main/phases/17_import_and_externals_correctness.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_17_1` (Frontend-Only Check Path) | Ensure check stops after frontend/type phases.; Remove codegen/runtime coupling from check flow. | check no longer triggers full code generation. |
| `milestone_17_2` (Non-Main Externals Resolution) | Resolve stdlib/local externals in non-main modules.; Ensure multi-file projects type-check consistently. | Non-main modules can import stdlib/local modules correctly. |
| `milestone_17_3` (Test and Constant Import Parity) | Align sifr test import behavior with regular compilation.; Support local-module constant imports in externals model. | Test runner imports behave like compile pipeline.; Local constants import successfully across modules. |

## Phase 18: Project and CLI Semantics Correctness
Source: `.cursor/plans/main/phases/18_project_and_cli_semantics_correctness.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_18_1` (Run/Build Semantics Alignment) | Align project detection and compilation scope between run and build. | Equivalent project inputs yield equivalent resolution behavior. |
| `milestone_18_2` (Auto-Detection Rule Tightening) | Replace over-aggressive auto project mode with explicit, documented rules. | Nearby scratch files do not unexpectedly break single-file runs. |
| `milestone_18_3` (CLI Contract and Regression Suite) | Document stable CLI semantics and edge cases.; Add regression tests for command-mode behavior. | CLI behavior contract exists and is regression-protected. |

## Phase 19: Module Graph Safety, Determinism, and Cache
Source: `.cursor/plans/main/phases/19_module_graph_safety_determinism_and_cache.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_19_1` (Dependency-Safe Module Ordering) | Introduce topological ordering for module compilation.; Add cycle diagnostics with actionable context. | Module compile order is dependency-correct and cycle-safe. |
| `milestone_19_2` (Deterministic Assembly) | Remove nondeterministic HashMap-order behavior from module assembly/output. | Repeated builds produce stable module output order. |
| `milestone_19_3` (Stdlib Cache for Local Loops) | Cache stdlib compilation artifacts for repeated check/test cycles. | Repeated local runs avoid redundant stdlib recompilation. |

## Phase 20: HIR Decomposition and Maintainability Hardening
Source: `.cursor/plans/main/phases/20_hir_decomposition_and_maintainability_hardening.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_20_1` (Split `lower.rs`) | Extract lowering concerns into coherent submodules (imports, statements, expressions, typing hooks, diagnostics).; Preserve current semantics and test outcomes. | lower.rs is split into maintainable units with no behavior drift. |
| `milestone_20_2` (Split `stdlib.rs`) | Partition stdlib metadata/registration logic into focused modules. | stdlib.rs is modularized with equivalent behavior. |
| `milestone_20_3` (Anti-Regrowth Guardrails) | Add file-size and module-boundary conventions.; Add review checklist items for new lowering additions. | Guardrails are documented and enforced in local/CI checks where practical. |

## Phase 21: Traversal Completeness and Control-Flow Correctness
Source: `.cursor/plans/main/phases/21_traversal_completeness_and_control_flow_correctness.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_21_1` (Canonical Walker Coverage) | Standardize recursive traversal across statement/expression variants.; Remove partial traversal blind spots. | Traversal completeness matrix is satisfied for supported nodes. |
| `milestone_21_2` (`while ... else` End-to-End Support) | Implement intended Python-like while ... else semantics through HIR and codegen. | while ... else behavior matches language intent with regression tests. |
| `milestone_21_3` (Yield and Exception-Path Coverage) | Fix generator/yield detection across nested constructs.; Ensure try/except analysis includes loop-else and other missed paths. | No known missed traversal paths in generator/error analysis. |

## Phase 22: Type-System Soundness
Source: `.cursor/plans/main/phases/22_type_system_soundness.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_22_1` (TypeVar Constraint Enforcement) | Replace permissive TypeVar assignability with bound/constraint validation. | Generic code is type-checked against declared constraints. |
| `milestone_22_2` (Inheritance and Variance Corrections) | Implement multi-level inheritance assignability.; Remove special-case inheritance hacks.; Enforce invariance on mutable collections. | Subtyping and mutable variance behavior are sound. |
| `milestone_22_3` (Optional Arithmetic Soundness) | Eliminate unsound optional arithmetic acceptance in type checking. | Optional arithmetic requires explicit safe handling. |

## Phase 23: Runtime-Safe Codegen Semantics
Source: `.cursor/plans/main/phases/23_runtime_safe_codegen_semantics.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_23_1` (Remove Data-Dependent `unwrap/expect`) | Replace generated data-dependent unwrap/expect with explicit safe propagation. | User-facing generated paths avoid data-dependent unwrap/expect panics. |
| `milestone_23_2` (Indexing and Semantics Parity Fixes) | Correct negative indexing and related parity semantics. | Indexing behavior matches language intent and tests. |
| `milestone_23_3` (Defaults and Panic-to-Diagnostic Conversion) | Preserve non-literal default argument semantics.; Replace remaining user-triggerable panic/assert codegen paths with diagnostics. | Runtime/codegen semantics are safe and diagnostic-driven. |

## Phase 24: Diagnostics, Error Recovery, Stability Contract, Panic-to-Diagnostic
Source: `.cursor/plans/main/phases/24_diagnostics_error_recovery_and_stability_contract.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_24_1` (Span and Diagnostic Schema Quality) | Thread precise spans through frontend/codegen errors.; Standardize stable diagnostic codes/categories. | Diagnostics include accurate source locations and stable codes. |
| `milestone_24_2` (Bounded Multi-Error Recovery) | Add parser/type-check recovery to report multiple actionable errors.; Control error cascades with bounded recovery policy. | Compiler reports multiple useful errors without crash storms. |
| `milestone_24_3` (Stability Contract Finalization) | Define documented exit codes, CLI flag stability/versioning, and diagnostic-text policy.; Convert remaining user-triggerable panics to diagnostics. | Stability policy is explicit and enforced by tests/docs. |

## Phase 25: Verification Hardening
Source: `.cursor/plans/main/phases/25_verification_hardening.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_25_1` (Regression Matrix Expansion) | Ensure each fixed bug has dedicated regression coverage.; Expand cross-phase regression suites. | Regression matrix maps directly to resolved findings. |
| `milestone_25_2` (Fuzz and Property Scale-Out) | Move from smoke fuzz/property checks to sustained coverage.; Track and triage fuzz findings systematically. | Fuzz/property suite is part of standard hardening gates. |
| `milestone_25_3` (Real-World E2E Parallel Gate) | Validate representative multi-module real-world projects end-to-end (check/build/run/test). | E2E suites pass deterministically in local parallel mode. |

## Phase 26: Reliability Parity (Stdlib)
Source: `.cursor/plans/main/phases/26_reliability_parity_and_performance_budgets.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_26_1` (Stdlib Behavioral Parity) | Port and maintain module-by-module parity tests against Python behavior.; Classify outcomes as parity, intentional-diff, or unsupported with rationale. | Targeted stdlib modules have parity suites and an up-to-date parity matrix. |
| `milestone_26_2` (Complexity and Resource Parity) | Run scaling benchmarks (time and memory) for exposed stdlib APIs.; Validate asymptotic class parity against CPython and track constant-factor deltas. | Asymptotic parity is verified; constant-factor regressions are budgeted or waived explicitly. |
| `milestone_26_3` (Parity Governance and Waiver Discipline) | Enforce parity classification discipline (parity, intentional-diff, unsupported) with linked rationale.; Require explicit waiver records for unresolved parity gaps. | No unresolved parity gaps exist without documented waiver and owner. |

## Phase 27: Async and Ecosystem Foundation
Source: `.cursor/plans/main/phases/27_async_ecosystem.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_async_core` (Async Runtime Core) | Add the minimum viable async language support: async def/await syntax, Tokio runtime auto-bundling, and basic task spawning. This is the foundational compiler feature that all other async milestones build on. | async def compiles to Rust async fn; await compiles to .await; Tokio runtime is automatically bundled when async is used; try/except auto-unwrap works across .await points (compiler-internal ? in HIR, not user-facing); sifr.task.spawn works for concurrent tasks; sifr.task.sleep works for async delays; sifr.task.timeout wraps an async call with a deadline; All existing E2E tests still pass (no regressions); cargo test passes, cargo clippy -- -D warnings passes, no new unsafe without justification; E2E pass tests: async_basic, await_chain, task_spawn, async_error_propagation, task_sleep, task_timeout; E2E fail tests: spawn_non_send (clear Sifr diagnostic when spawning a non-Send type); Milestone demo in ./demos/milestone_async_core_demo.sifr |
| `milestone_typed_serde_core` (Typed Serialization (Core)) | Web-independent typed serialization. This does NOT include web extractors — those are delivered in a later web phase. Typed serde is kept in the async phase to make typed payload handling available early. | Classes auto-derive Serialize/Deserialize — no manual annotation needed; dumps(obj) serializes any class to JSON string; loads(s, T) deserializes JSON string to typed class, returns Result[T, JSONDecodeError]; Nested classes, lists, dicts, optionals, unions serialize correctly; All existing E2E tests still pass (no regressions); cargo test passes, cargo clippy -- -D warnings passes, no new unsafe without justification; E2E pass tests: typed_json_roundtrip, nested_class_serde, union_serde, optional_serde; E2E fail tests: json_parse_wrong_type, missing_required_field |
| `milestone_async_sync` (Async Synchronization Primitives) | Add cross-task synchronization primitives and Send/Sync checking at spawn boundaries. These are needed for production async code but are not required for basic async functionality. | sifr.sync.Lock works for shared mutable state across tasks; sifr.sync.Channel works for typed message passing between tasks; sifr.sync.Semaphore works for concurrency limiting; Send/Sync checking at spawn boundaries produces clear diagnostics; Async closures are checked for Send + 'static; All existing E2E tests still pass (no regressions); cargo test passes, cargo clippy -- -D warnings passes, no new unsafe without justification; E2E pass tests: lock_basic, channel_basic, semaphore_basic, send_sync_check; E2E fail tests: non_send_spawn (clear error for non-Send type in spawn) |
| `milestone_async_advanced` (Advanced Async Features) | Add advanced async features that build on the core runtime and sync primitives. These are powerful but not needed for basic async applications. | async with works for async context managers; Async generators (yield in async def) produce async iterators; Async comprehensions compile correctly; All existing E2E tests still pass (no regressions); cargo test passes, cargo clippy -- -D warnings passes, no new unsafe without justification; E2E pass tests: async_with_basic, async_generator_basic, async_comprehension; Milestone demo in ./demos/milestone_async_advanced_demo.sifr |

## Phase 28: Preview Distribution and Release Automation (`alpha`/`beta`)
Source: `.cursor/plans/main/phases/28_preview_distribution_and_release_automation.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_28_1` (Installer and Channel Resolution) | Implement install entrypoint (curl -fsSL https://sifr.sh/install | bash).; Support SIFR_CHANNEL/--channel and explicit --version pinning. | Installer resolves alpha/beta/stable channel metadata correctly. |
| `milestone_28_2` (Artifact and Manifest Pipeline) | Publish multi-platform artifacts with checksums/signatures.; Maintain channel manifest pointers. | Installer validates checksums and installs matching artifacts. |
| `milestone_28_3` (Agentic Preview Release Command) | Add /create-new-version workflow for preview release automation.; Support dry-run and real-run paths. | Preview release flow is repeatable end-to-end for alpha/beta. |

## Phase 29: Performance Benchmarking and Budgets
Source: `.cursor/plans/main/phases/29_performance_benchmarking_and_budgets.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_29_1` (Baseline Benchmark Suite) | Define compiler benchmark suites for check, build, and incremental local loops. | Baselines are versioned and reproducible locally. |
| `milestone_29_2` (Budget and Threshold Policy) | Set compiler regression thresholds and waiver process. | Performance budget policy is documented and testable. |
| `milestone_29_3` (Enforcement Integration) | Add local and CI gates for benchmark regressions. | Regressions fail gates unless approved waiver exists. |

## Phase 30: Developer Tooling and Ecosystem Hooks
Source: `.cursor/plans/main/phases/30_developer_tooling_and_ecosystem_hooks.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_30_1` (Developer Tooling and Ecosystem Hooks) | LSP/formatter/linter/doc hooks aligned with new phase contracts. | Tooling integrates with language/runtime capabilities added in prior phases. |

## Phase 31: Package Management
Source: `.cursor/plans/main/phases/31_package_management.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_31_1` (Package Management) | Dependency declaration, lockfile semantics, resolution workflow. | Package workflows are deterministic and reproducible. |

## Phase 32: Docs and Documentation
Source: `.cursor/plans/main/phases/32_docs_and_documentation.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_32_1` (Documentation Information Architecture) | Define canonical docs structure for language, compiler internals, stdlib, CLI, packaging, and operations.; Remove duplicated/contradictory guidance and centralize source-of-truth ownership. | Documentation map is approved and all core sections have canonical owners. |
| `milestone_32_2` (Reference and Contract Documentation) | Publish versioned references for CLI behavior, diagnostics, package workflows, and phase contracts.; Document expected compatibility/stability guarantees for users and contributors. | Contract docs are complete, versioned, and linked from roadmap/architecture entry points. |
| `milestone_32_3` (Documentation Quality Gates) | Add local docs validation for link integrity, required sections, and drift checks against phase files.; Ensure docs checks are runnable in local quick/full/stress workflows. | Documentation quality gates pass locally and are mirrored in CI. |

## Phase 33: Stable Channel GA Promotion and Release Governance
Source: `.cursor/plans/main/phases/33_stable_channel_ga_promotion_and_release_governance.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_33_1` (Stable Promotion Policy) | Define hard preconditions for stable promotion from preview channels. | Promotion checklist is documented and mandatory. |
| `milestone_33_2` (Rollback and Incident Governance) | Define rollback triggers, owner responsibilities, and communication protocol. | Rollback path is tested and documented. |
| `milestone_33_3` (Release Sign-off Workflow) | Enforce formal release sign-off and artifact provenance checks. | Stable releases require auditable approvals and pass governance gates. |

## Phase 34: Typed Data Model and Validation (Pydantic-Parity Track)
Source: `.cursor/plans/main/phases/34_typed_data_model_and_validation.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_34_1` (Typed Model Core) | Class-to-model mapping with field metadata and defaults.; Optional/union/list/dict model handling.; Baseline serialization/deserialization (dumps/loads). | Typed model core is usable independent of async/web runtime concerns. |
| `milestone_34_2` (Validation Engine) | Strict vs coercion modes.; Nested model validation and collection constraints.; Field/model validator hooks with deterministic order. | Validation behavior is deterministic, testable, and documented. |
| `milestone_34_3` (Error Model and Diagnostics Contract) | Structured validation errors (path, code, message, context).; Stable parse/validation error-code contract. | Validation failures produce stable, structured, and actionable errors. |
| `milestone_34_4` (Parity and Compatibility Matrix) | Feature matrix per capability: parity, intentional-diff, unsupported.; Port representative pydantic behavior tests. | Target pydantic subset is explicit and regression-locked. |

## Phase 35: Web Framework and Platform Expansion
Source: `.cursor/plans/main/phases/35_web_framework_and_platform_expansion.md`

| Milestone | Scope Snapshot | Definition-of-Done Snapshot |
|---|---|---|
| `milestone_35_1` (Web Framework Core) | Routing, middleware, lifecycle/shutdown, and base request/response pipeline. | Core web scaffolding is stable and test-covered. |
| `milestone_35_2` (Typed Extractors and Request Validation) | Json/Path/Query/Form extractor behavior.; Validation and error mapping via Phase 33 model contract. | Extractors enforce typed validation with consistent error responses. |
| `milestone_35_3` (Production Web Baseline) | Logging/tracing, config, and operational hooks for production readiness. | Web stack production baseline is documented and smoke-covered. |
| `milestone_35_4` (Data/ML Track (Scoped)) | Initial data processing and ML inference workflows on top of web/model foundations. | Data/ML MVP workflows are validated with regression coverage. |
| `milestone_35_5` (Interoperability Track (Scoped)) | Initial FFI/interoperability boundary model and safety constraints. | Interop MVP workflows are documented, test-covered, and quality-gated. |

## Coverage Summary
- Total milestones tracked: `63`
- Validation command: `python scripts/validate_milestone_registry_15_35.py`

