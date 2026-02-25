## [Task] E2E Runner Baseline and Contract Lock

#### Current Situation
- The full pass corpus is executed by one large test path with high runtime.
- There is no locked machine-readable contract document for legacy behavior.
- Throughput work can regress behavior unless contract and baseline are explicit.

#### Desired Situation
- Legacy behavior is documented in a concrete contract before redesign changes.
- Baseline timing and result metrics are captured and reproducible.
- Future tickets can compare against stable expectations.

#### Suggested Solution
- Add a test-contract section in `crates/sifr/tests/e2e.rs` comments or adjacent doc notes.
- Capture baseline run metrics and slowest fixtures with profiling mode.
- Record exact benchmark command protocol in the ticket notes.
- Add automated assertions for core invariants at this phase (not only documentation).

#### Implementation Checklist
- Identify all externally visible behaviors to preserve:
  - discovery ordering
  - expectation parsing
  - failure aggregation format
  - exit code behavior
- Capture baseline metrics from current runner.
- Store benchmark protocol in a stable repo doc under `issues/` notes.
- Add contract tests/assertions for:
  - deterministic fixture discovery ordering
  - expectation parser behavior for stdout/stderr/error annotations
  - aggregate failure counting semantics

#### Acceptance Criteria
- A written contract exists for legacy runner behavior.
- Baseline timing numbers and run commands are recorded.
- Next tasks can reference this ticket without ambiguity.
- Core invariants are enforced by automated checks.

#### Dependencies
- None.
