# milestone_e2e_compiletest_throughput - Full-Corpus Compiletest Runner

## 1. Product Requirements

### Title
Full-Corpus E2E Throughput Redesign (Compiletest-Style Runner)

### Objective / Problem Statement
The current E2E pass runner executes the real fixture corpus but takes several minutes. This slows every compiler iteration and encourages developers to skip full validation. We need a faster architecture that still runs the full real tests.

### Constraints (business-wise)

| Constraint | Rationale |
| --- | --- |
| Must keep full corpus coverage | Team explicitly does not want a smoke subset replacement |
| Must preserve existing fixture contract | Existing `.sifr` fixtures use `# expect-stdout` and diagnostics comments |
| Must keep deterministic, debuggable failures | CI and contributors need stable output and reproducible failures |
| Must allow gradual rollout | Prevent blocking current development while redesign lands |

### Business Goals & Success Criteria (KPIs)

| Metric | Baseline (Today) | Target (Post-launch) |
| --- | --- | --- |
| Full `test_e2e_pass` wall time (warm cache) | ~4 to 5+ minutes | <= 90 seconds initial, <= 45 seconds stretch |
| Developer confidence in full-suite runs | Low due to runtime | High enough for frequent local runs |
| Correctness equivalence vs legacy runner | N/A | 100% equivalent pass/fail outcomes |
| Failure debuggability | Medium | Equal or better than legacy |

### Scope

#### Features In
1. Throughput redesign for `test_e2e_pass` (full pass corpus) only.
2. Parallel Sifr compile stage for pass fixtures.
3. Dependency fingerprinting and fixture grouping.
4. Batch crate generation and per-group Rust build/run.
5. Persistent cache for unchanged fixture/group artifacts.
6. Legacy compatibility mode and controlled rollout.

#### Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| Reducing fixture set | Explicitly rejected by team direction |
| Rewriting stdlib semantics | Out of scope for performance redesign |
| Replacing fixture format | Too disruptive for this milestone |
| Distributed remote execution | Added complexity, not required for first win |
| Functional redesign of `test_e2e_fail` | Deferred; should remain behaviorally unchanged |
| Functional redesign of `test_e2e_runtime_fail` | Deferred; should remain behaviorally unchanged |
| Functional redesign of `test_codegen_corpus_subset_parity` | Deferred; should remain behaviorally unchanged |
| Functional redesign of `test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus` | Deferred; should remain behaviorally unchanged |

### Users / Stakeholders, Use-Cases & Dependencies

| Persona | Use-Case / Benefit | Dependencies | AC-ID |
| --- | --- | --- | --- |
| Compiler engineer | Run full real E2E locally faster | New runner architecture | AC-1 |
| CI maintainer | Keep reliable full-corpus regression gate | Deterministic results + fallback mode | AC-2 |
| Future AI/code agent | Implement features safely with fast verification | Clear tickets and rollout plan | AC-3 |

### Acceptance Criteria

| AC-ID | Persona | Criterion (Given / When / Then) |
| --- | --- | --- |
| AC-1 | Compiler engineer | Given the full fixture corpus on benchmark CI hardware, when running warm-cache benchmark protocol, then p50 <= 90s and p95 <= 110s and all fixtures run |
| AC-2 | CI maintainer | Given CI executes legacy and new runner modes, when results are compared, then pass/fail sets are equivalent |
| AC-3 | Compiler engineer | Given any failure, when reading output, then case name, generated artifact location, and mismatch details are clearly reported |
| AC-4 | Compiler engineer | Given unchanged fixtures and toolchain, when rerunning, then cached artifacts are reused safely |
| AC-5 | Repo owner | Given staged rollout, when issues occur, then legacy runner can be toggled immediately |

---

## 2. Solution Design

### 2.1 Functional Requirements
- Discover and parse all pass fixtures deterministically.
- Compile fixtures through Sifr in parallel.
- Group compiled cases by normalized dependency fingerprint.
- Generate one Rust crate per dependency group.
- Build and run groups, then dispatch and validate each case output.
- Aggregate all failures in a stable deterministic report.
- Reuse artifacts when content hashes and environment keys match.

### 2.2 Non-Functional Requirements

| ID | Requirement |
| --- | --- |
| NFR-1 | Deterministic ordering of discovery, grouping, execution, and reports |
| NFR-2 | Throughput improvement with bounded CPU/memory contention |
| NFR-3 | Safe cache invalidation keyed by all correctness-critical inputs |
| NFR-4 | Rollback safety through compatibility toggle |
| NFR-5 | Sufficient instrumentation to locate new bottlenecks |

### 2.3 High-Level Architecture
```
Fixture Discovery
    -> Parallel Sifr Compile Stage
    -> Dependency Fingerprinting + Group Planner
    -> Batch Crate Generation
    -> Parallel Group Build
    -> Group Execution + Output Verification
    -> Aggregate Report + Metrics + Cache Manifest
```

### 2.4 Detailed Component Design

Component A - Discovery and Expectation Parser
- Reads fixtures and expectation comments.
- Produces deterministic `FixtureCase` list.

Component B - Parallel Compiler Stage
- Uses bounded worker pool.
- Produces `CompiledCase` with metadata and diagnostics.

Component C - Group Planner
- Normalizes dependencies.
- Assigns each case to a `DependencyFingerprint` group.

Component D - Batch Builder
- Generates crate layout per group.
- Handles symbol namespacing and case dispatch.

Component E - Executor and Comparator
- Runs binaries.
- Compares outputs exactly and records diffs.

Component F - Cache Manager
- Persists manifest entries.
- Reuses valid artifacts and invalidates stale ones.

### 2.5 Data Model
```text
FixtureCase {
  name, path, source_hash,
  expected_stdout, expected_stderr, expected_errors
  // expected_errors is retained for later fail-test integration
}

CompiledCase {
  fixture, rust_source,
  used_stdlib_modules, required_crates,
  compile_diagnostics
}

BatchGroup {
  fingerprint,
  cases[],
  crate_dir,
  artifact,
  build_status
}

CacheManifest {
  version,
  toolchain_keys,
  entries[key] -> artifact metadata
}
```

### 2.6 API Integration
- No external API required.
- Integrates with existing `sifr_driver` compile APIs and local Rust toolchain commands.

### 2.7 Error Handling & Monitoring
- Continue collecting failures across cases.
- Emit phase-level timing metrics and top slow groups/cases.
- On cache errors, log and rebuild safely.
- If a grouped crate fails to build, attribute failure to every fixture in that group and print:
  - group fingerprint
  - generated crate path
  - build log path
  - deterministic fixture list in that group

### 2.8 Deployment Plan
- Phase 1: Introduce runner mode selector (`SIFR_E2E_RUNNER_MODE=legacy|new|compare`) with back-compat env mapping.
- Phase 2: CI runs legacy, new, and mandatory differential/equivalence mode.
- Phase 3: Make new runner default, keep legacy fallback toggle.
- Phase 4: Remove legacy path after stable release window.

### 2.9 Trade-offs & Alternatives

| Option Considered | Pros | Cons | Rationale for Final Choice |
| --- | --- | --- | --- |
| Keep per-fixture Cargo builds | Simple | Too slow at corpus scale | Rejected |
| Smoke subset only | Fast | Violates full-corpus requirement | Rejected |
| Compiletest-style grouped runner | Keeps full coverage, strong speedup | More engineering complexity | Chosen |
| External distributed build farm | Potentially very fast | Operationally heavy | Deferred |

### 2.10 Testing Strategy (mapped to ACs)

| AC-ID | Test Layer | Happy-Path Check | Edge Check | Tooling & Automation | Pass/Fail Gate |
| --- | --- | --- | --- | --- | --- |
| AC-1 | Integration/E2E | Full corpus passes with new runner | Mixed dependency groups | `cargo test` + timing capture | Runtime target and correctness |
| AC-2 | Differential | Legacy vs new pass/fail equivalence | Intentional failing fixture | CI matrix mode | Exact equivalence |
| AC-3 | Snapshot/Integration | Failure reports include case context | Multi-failure aggregation | Report assertions | Stable and actionable output |
| AC-4 | Integration | Warm rerun reuses cache | Cache invalidation on input changes | Hash-key checks | No stale result risk |
| AC-5 | Rollout | Toggle fallback works | Forced rollback drill | CI toggle test | Immediate recoverability |

Additional NFR Tests
- Performance trend tracking across phases.
- Stress tests on high fixture counts.
- Determinism checks for ordering.
