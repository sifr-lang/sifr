# E2E Throughput Runner - Execution Plan

## Workflow Alignment (project-workflow skill)
- Planning: Epic PRDS plus task tickets drafted in `issues/`.
- Refinement: Move highest-priority tasks to Ready in sequence.
- Development: Implement tasks in order with explicit dependency edges.
- Review: Validate each task with benchmark and correctness evidence.
- Done: Merge after equivalence and rollback checks pass.

## Master Sequence
1. Baseline and guardrails.
2. Shared data model and dependency fingerprinting.
3. Parallel Sifr compile stage.
4. Batch crate generation model.
5. Parallel group build and execution.
6. Cache and invalidation.
7. Equivalence, reporting, and fallback.
8. CI rollout and legacy deprecation.

## Ticket Map
- Epic: `issues/milestone-e2e-compiletest-throughput-epic.md`
- Task 208: Baseline and contract
- Task 209: Manifest and fingerprint planner
- Task 210: Parallel Sifr compile stage
- Task 211: Batch crate generation
- Task 212: Parallel group build/execution
- Task 213: Cache + invalidation
- Task 214: Result equivalence + report compatibility
- Task 215: CI rollout + legacy path deprecation plan

## Required Invariants
- Full fixture corpus still runs.
- Deterministic ordering in reports.
- Aggregate failure report behavior retained.
- Exact expectation parsing semantics retained.
- Safe fallback to legacy runner.
- `test_e2e_fail`, `test_e2e_runtime_fail`, and codegen gate tests remain behaviorally unchanged in this milestone.

## Suggested Branching / PR Strategy
- One ticket per PR, do not combine non-adjacent phases.
- Keep feature behind explicit mode toggle until Task 214 complete.
- After Task 215, switch default only with fallback env preserved.
- During transition, differential/equivalence mode is a required CI gate.

## Benchmark Protocol (every relevant task)
1. Record benchmark hardware and toolchain metadata:
   - runner class / CPU count
   - `rustc -Vv`
   - `cargo -V`
2. Use warm-cache policy:
   - run once as warm-up and discard
   - collect 7 measured runs
3. Run legacy and new runner on same revision (where applicable).
4. Capture:
   - p50 and p95 wall time
   - stage times
   - pass/fail counts
   - coefficient of variation
5. Pin concurrency env vars for comparability.
6. Attach raw run data in ticket notes.

## Success Definition
- New architecture is default for full E2E pass tests.
- Correctness is equivalent to legacy runner.
- Throughput is substantially improved and stable across reruns.
