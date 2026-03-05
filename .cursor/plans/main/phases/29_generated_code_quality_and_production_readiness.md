# Phase 29: Generated Code Quality and Production Readiness

status: planned

## Objective
Guarantee that emitted Rust is production-grade in safety, determinism, tooling compliance, and maintainability.

## Depends on
- Phase 28 (`preview_distribution_and_release_automation`)

## Non-goals
- New language feature development.
- Runtime semantics redesign already covered by prior soundness phases.
- Package ecosystem expansion concerns.

## Milestones

### milestone_29_1: Emission Quality Baseline and Corpus
- Scope:
  - Define generated-code quality profile and acceptance thresholds.
  - Build representative corpus from stdlib flows, demos, and multi-module samples.
- Definition of done:
  - Corpus is version-controlled and reproducible.
  - Coverage targets are explicit and met.

### milestone_29_2: Panic/Unsafe Path Elimination in Generated User Paths
- Scope:
  - Remove data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths.
  - Remove emitted `todo!` / `unimplemented!` from production paths.
- Definition of done:
  - User-facing generated paths are panic-safe by this contract.
  - Violations are blocked by automated checks.

### milestone_29_3: Lint/Format/Static Analysis Compliance
- Scope:
  - Enforce compile with `-D warnings` on generated corpus.
  - Enforce `rustfmt --check` and agreed clippy profile on generated corpus.
- Definition of done:
  - Generated corpus passes compile/lint/format gates with zero critical violations.

### milestone_29_4: Deterministic and Reproducible Emission
- Scope:
  - Enforce byte-stable output for identical input/configuration.
  - Add repeat-run determinism checks.
- Definition of done:
  - Determinism checks pass with no unstable output regressions.

### milestone_29_5: Demo Quality Validation Contract
- Scope:
  - Make required `demos/` runs part of phase quality gates.
  - Require milestone-level positive/negative validation plus demo evidence.
- Definition of done:
  - Required demos pass generated-code quality checks.
  - Demo validation evidence is recorded per milestone.

## Quality Contract

### Entry criteria
- Phase 28 exit gate is satisfied.
- Initial generated-code corpus seed is defined.

### Milestone quality checks
- Local validation gates pass for each milestone before merge.
- Generated Rust compiles with `-D warnings` on defined corpus.
- No data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths.
- No emitted `todo!` / `unimplemented!` in production paths.
- `rustfmt --check` and agreed clippy profile pass for generated corpus.
- Validation evidence is recorded in the phase execution checklist issue before merge.

### Validation planning goals
- `milestone_29_1`:
  - Positive: corpus generation succeeds for representative projects.
  - Negative: malformed/unsupported inputs are surfaced with expected diagnostics.
- `milestone_29_2`:
  - Positive: safe generated paths handle fallible flows without panic.
  - Negative: known panic-prone patterns are rejected by checks/regressions.
- `milestone_29_3`:
  - Positive: clean corpus passes compile/lint/format gates.
  - Negative: seeded lint/format violations fail gates as expected.
- `milestone_29_4`:
  - Positive: repeated runs produce identical outputs.
  - Negative: induced nondeterministic ordering is detected and fails checks.
- `milestone_29_5`:
  - Positive: required demos pass end-to-end quality gates.
  - Negative: intentionally broken demo path fails with expected gate signal.

### Exit criteria
- All milestone DoDs are satisfied.
- All milestone quality checks pass with zero unresolved critical violations.
- Determinism is verified across repeated runs on required corpus.
- Required demos pass and have recorded validation evidence.
- Any waiver is explicit, time-bounded, owner-assigned, and issue-linked.

## Exit Gate
Generated Rust satisfies all Phase 29 quality guarantees with zero critical violations, determinism is verified across repeated runs, and required demos pass quality gates.
