# E2E Throughput - /add-ticket Ready Payloads

This document is optimized for quick ticket creation and board setup.
Use each block as the source payload for `/add-ticket`.

## Project Field Defaults
- Status: `Backlog`
- Type: `Epic` or `Task`
- Priority: `P0`, `P1`, `P2`
- Size: `XS`, `S`, `M`, `L`, `XL`

## Creation Order (Epic first, then tasks)
1. milestone-e2e-compiletest-throughput-epic
2. 208-e2e-runner-baseline-and-contract
3. 209-e2e-runner-manifest-and-fingerprint-planner
4. 210-e2e-runner-parallel-sifr-compile-stage
5. 211-e2e-runner-batch-crate-generation
6. 212-e2e-runner-parallel-group-build-and-execution
7. 213-e2e-runner-cache-and-invalidation
8. 214-e2e-runner-equivalence-and-reporting
9. 215-e2e-runner-rollout-ci-and-deprecation

## Payloads

### Payload 1
- Title: `milestone_e2e_compiletest_throughput - Full-Corpus Compiletest Runner`
- Type: `Epic`
- Status: `Backlog`
- Priority: `P0`
- Size: `XL`
- Source: `issues/milestone-e2e-compiletest-throughput-epic.md`
- Depends on: none

### Payload 2
- Title: `[Task] E2E Runner Baseline and Contract Lock`
- Type: `Task`
- Status: `Backlog`
- Priority: `P1`
- Size: `S`
- Source: `issues/208-e2e-runner-baseline-and-contract.md`
- Depends on: none

### Payload 3
- Title: `[Task] E2E Manifest and Dependency Fingerprint Planner`
- Type: `Task`
- Status: `Backlog`
- Priority: `P0`
- Size: `M`
- Source: `issues/209-e2e-runner-manifest-and-fingerprint-planner.md`
- Depends on: Task 208

### Payload 4
- Title: `[Task] Parallelize Sifr Compile Stage`
- Type: `Task`
- Status: `Backlog`
- Priority: `P1`
- Size: `M`
- Source: `issues/210-e2e-runner-parallel-sifr-compile-stage.md`
- Depends on: Task 209

### Payload 5
- Title: `[Task] Implement Batch Crate Generation per Dependency Group`
- Type: `Task`
- Status: `Backlog`
- Priority: `P1`
- Size: `M`
- Source: `issues/211-e2e-runner-batch-crate-generation.md`
- Depends on: Task 209

### Payload 6
- Title: `[Task] Parallel Group Build and Execution Pipeline`
- Type: `Task`
- Status: `Backlog`
- Priority: `P0`
- Size: `L`
- Source: `issues/212-e2e-runner-parallel-group-build-and-execution.md`
- Depends on: Task 210, Task 211

### Payload 7
- Title: `[Task] Add Persistent Cache with Safe Invalidation`
- Type: `Task`
- Status: `Backlog`
- Priority: `P1`
- Size: `M`
- Source: `issues/213-e2e-runner-cache-and-invalidation.md`
- Depends on: Task 212

### Payload 8
- Title: `[Task] Legacy Equivalence and Failure Reporting Compatibility`
- Type: `Task`
- Status: `Backlog`
- Priority: `P0`
- Size: `M`
- Source: `issues/214-e2e-runner-equivalence-and-reporting.md`
- Depends on: Task 213

### Payload 9
- Title: `[Task] CI Rollout, Default Switch, and Legacy Deprecation Plan`
- Type: `Task`
- Status: `Backlog`
- Priority: `P1`
- Size: `S`
- Source: `issues/215-e2e-runner-rollout-ci-and-deprecation.md`
- Depends on: Task 214

## Refinement / Ready Board Order
Use this when moving from `Backlog` to `Ready`.

1. Move Task 208 to `Ready`.
2. Move Task 209 to `Ready` after 208 is done.
3. Move Tasks 210 and 211 together to `Ready` (parallel workstream).
4. Move Task 212 to `Ready` after 210 and 211 are done.
5. Move Task 213 to `Ready` after 212 is done.
6. Move Task 214 to `Ready` after 213 is done.
7. Move Task 215 to `Ready` after 214 is done.

## Optional Fast-Track Priority View
- P0: Epic, 209, 212, 214
- P1: 208, 210, 211, 213, 215

## Notes for /add-ticket Operator
- Keep all tickets initially in `Backlog`.
- Set `Type`, `Priority`, and `Size` at creation time.
- Paste the full markdown from each source file as the issue body.
- After creation, link dependencies in issue descriptions or project notes if the tracker has no native dependency field.
