# Phase 29: Verification Hardening

status: completed

## Objective
Establish a production-grade compiler verification system that is deterministic, locally enforceable, reviewable, and issue-traceable through explicit suite taxonomy, baseline governance, regression corpus discipline, fuzz/property operations, curated real-world validation, and flake control.

## Depends on
- Phase 28

## Non-goals
- Stdlib behavioral and complexity parity closure.
- Broad compatibility remediation across large external corpora.
- Generated Rust lint/format/static-analysis closure.
- Compiler performance budget governance.
- Tooling/LSP/editor parity.
- Release governance and stable-channel promotion policy.

## Scope
This phase owns the compiler verification foundation:
- compiler-facing suite taxonomy
- baseline/snapshot governance for compiler outputs
- issue-linked regression corpus
- known-failure sentinel policy
- fuzz/property operating model
- curated real-world blocking gate
- broader ecosystem non-blocking lane definition
- deterministic sharding, rerun, and flake policy
- machine-readable validation artifacts

This phase does not own:
- stdlib parity governance (Phase 30)
- broad compatibility taxonomy, scorecards, and remediation waves (Phase 31)
- generated-code quality closure (Phase 34)
- incremental cache/query correctness, invalidation contracts, and local-loop cache architecture (Phase 35)
- performance thresholds and benchmark governance (Phase 35)
- tooling parity (Phase 36)
- release governance (Phase 39)

## Verification Taxonomy
The phase must define explicit suite kinds with contracts, ownership, and required artifacts.

Minimum suite kinds:
- `diagnostics`
  - compiler diagnostics, exit codes, renderer behavior, and structured suggestions where applicable
- `project`
  - multi-file and multi-module compiler behavior
- `fixedbugs`
  - permanent issue-linked regressions for resolved compiler bugs
- `crashes`
  - known unresolved compiler crashes or sentinel failures kept visible until fixed
- `property`
  - invariant-based generator tests for compiler internals
- `fuzz-smoke`
  - deterministic local fuzz smoke gate over curated corpora
- `oss-curated`
  - small pinned representative real-world project gate that blocks merges
- `ecosystem-broader`
  - larger non-blocking compatibility lane for signal only

## Test Case Format Conventions
- The phase must define canonical fixture and baseline conventions for every suite kind it introduces.
- At minimum, the conventions must cover:
  - `diagnostics` fixture layout and expected-output representation
  - `fixedbugs` issue-link format and required metadata
  - `crashes` sentinel format and promotion-to-regression rules
  - `oss-curated` corpus manifest format, including pinned revisions and command metadata
  - `property` and `fuzz-smoke` manifest/corpus metadata
  - baseline artifact naming and storage conventions
- Diagnostic suites must define one canonical way to represent expected codes, messages, spans, and renderer output.
- If diagnostics include structured suggestions, the conventions must distinguish between suggestion rendering validation and suggestion-application validation.
- Baseline-backed suites must define where expected artifacts live and how they are associated with source fixtures.
- The chosen conventions must favor reviewability, deterministic diffs, and low ambiguity over author convenience.

## Baseline Governance Contract
- Compiler-facing outputs that are part of the user contract must support checked-in baselines.
- Baseline-backed artifacts in this phase include:
  - diagnostic renderer output
  - exit-code behavior
  - selected multi-file/project outputs
  - machine-readable result summaries
- Critical generated-output baselines may be maintained here as verification artifacts; Phase 34 owns generated-code quality closure and acceptance thresholds.
- The phase must define one canonical bless/accept workflow for updating baselines.
- Normalization rules must be explicit for:
  - absolute paths
  - temporary directories
  - machine-specific noise
  - nondeterministic ordering where unavoidable
- Baseline updates are review artifacts and must never be treated as incidental side effects.

## Regression Corpus Contract
- Every resolved compiler bug in scope must land with permanent regression coverage.
- Each regression artifact must be linked to:
  - issue or finding identifier
  - root-cause category
  - suite location
  - brief note when context is not obvious
- Known unresolved compiler crashes or sentinel failures must remain visible in `crashes`.
- When a crash or sentinel case is fixed, it must be promoted into `fixedbugs` or another normal suite with issue linkage preserved.

## Fuzz and Property Contract
- Fuzz targets and property suites must be explicit and version-controlled.
- Seed corpora must be:
  - reproducible
  - reviewable
  - deduplicated against already-known failures where possible
- Every fuzz-found issue must follow a defined workflow:
  - reproduce
  - minimize
  - classify
  - link issue
  - promote to regression after fix
- This phase distinguishes:
  - local deterministic smoke fuzz/property gates
  - longer-running sustained fuzzing that may run outside the default blocking local flow

## Curated Real-World Validation Contract
- The blocking real-world gate must use a small pinned representative corpus.
- Each corpus entry must define:
  - pinned revision
  - rationale
  - owner
  - required commands (`check`, `build`, `run`, `test` as applicable)
  - timeout policy
  - expected result classification
- The phase must also define a broader non-blocking ecosystem lane separately from the curated gate.
- The curated gate is a hard verification gate.
- The broader lane is for compatibility signal and backlog generation, not merge blocking.

## Incremental Boundary Note
- This phase may verify repeatability and deterministic behavior of the canonical local validation entrypoints, including edit-run workflows exercised by owned suites.
- This phase does not define compiler cache/query invalidation correctness, dirty-rebuild guarantees, or incremental architecture contracts.
- Phase 35 owns deterministic invalidation rules, cache-consistency guarantees, and shared analysis/query architecture.

## Determinism and Flake Contract
- Repeat runs with identical inputs must produce identical canonical results.
- Sequential and parallel runs must agree on pass/fail outcomes and canonical artifacts for the suites this phase owns.
- The phase must define:
  - deterministic sharding strategy
  - rerun protocol
  - quarantine policy
  - re-enable criteria
  - reporting requirements for flaky tests
- A test that fails and then passes on rerun is tracked explicitly and is not silently treated as clean.

## Milestones

### milestone_29_1: Suite Taxonomy and Baseline Governance
- Scope:
  - Define canonical suite taxonomy and per-suite contracts.
  - Add baseline-backed verification for diagnostics and project behavior.
  - Define canonical checked-in artifacts and normalization rules.
  - Define one canonical bless/accept workflow.
- Definition of done:
  - Compiler-facing suites are explicitly categorized and documented.
  - Baseline-backed outputs are deterministic and reviewable.
  - Diagnostics and project behavior are governed by checked-in verification artifacts.

### milestone_29_2: Fixedbugs and Crashes Corpus
- Scope:
  - Require every resolved compiler bug in scope to land in `fixedbugs`.
  - Add issue-linked metadata and root-cause traceability.
  - Define `crashes` sentinel policy for unresolved failures.
  - Define promotion rule from `crashes` to normal regression suites.
- Definition of done:
  - Issue -> root cause -> test mapping exists for resolved bugs in scope.
  - Known unresolved crashes remain visible and intentionally tracked.
  - Regression corpus acts as institutional memory rather than ad hoc coverage.

### milestone_29_3: Fuzz and Property Operationalization
- Scope:
  - Define fuzz targets, property suites, and seed corpora for highest-value compiler surfaces.
  - Define reproducibility, deduplication, minimization, and triage rules.
  - Separate local smoke fuzz/property gates from longer-running sustained lanes.
- Definition of done:
  - Fuzz/property checks are part of standard hardening gates.
  - Every fuzz-found issue follows a documented triage path.
  - Minimized reproducible cases are required before closure.
  - Local fuzz/property runs are reproducible enough for routine engineering use.

### milestone_29_4: Curated OSS Gate and Broader Ecosystem Lane
- Scope:
  - Build a small pinned curated real-world/project corpus that blocks merges.
  - Define a separate broader non-blocking ecosystem lane.
  - Require structured result classification and reproducible execution.
- Definition of done:
  - Curated gate is version-controlled, pinned, and locally reproducible.
  - Broader ecosystem lane is explicitly non-blocking and separately classified.
  - Results are structured and comparable over time.

### milestone_29_5: Deterministic Scale, Flake Control, and Structured Evidence
- Scope:
  - Define deterministic sharding and per-suite runtime expectations.
  - Add repeat-run and sequential-vs-parallel equivalence checks where applicable.
  - Define rerun/quarantine policy for flakes.
  - Require machine-readable artifacts from all hardening gates.
  - Make the suggestion contract explicit:
    - if suggestions are part of the stable diagnostic contract, suggestion rendering must be baseline-validated
    - if automated suggestion application is part of the stable contract, add application validation
    - if automated application is not part of the stable contract, record that explicitly and do not require autofix execution in this phase
  - If structured suggestions or autofixes are already part of the stable compiler contract, add suggestion-application validation:
    - emit suggestion
    - apply suggestion
    - compare against expected transformed source
    - require recompile success
- Definition of done:
  - Hardening gates scale deterministically.
  - Flaky tests are never silently accepted as green.
  - Gate results can be triaged mechanically from structured artifacts.
  - Suggestion rendering is baseline-validated whenever suggestions are part of the stable contract.
  - Suggestion/autofix application validation exists if automated application is part of the stable contract; otherwise the non-application boundary is documented explicitly.

## Quality Contract

### Entry criteria
- Phase 28 exit gate is satisfied.
- Phase 27 non-regression baseline is green at phase start.
- Phase 16 local-first validation platform remains the authoritative execution foundation.

### Phase-wide invariants
- No user-triggerable panic paths.
- No data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths.
- Stable diagnostic contract:
  - codes
  - severity
  - spans
  - URLs
  - suggestions
  - schema
- Canonical and lossless `json` diagnostics remain authoritative.
- `human` and `compact` remain renderer views over the same diagnostic model.
- Recovery ordering remains deterministic.
- Exit-code and CLI contract remain stable.

### Milestone quality checks
- No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
- No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
- Every suite introduced in this phase has:
  - explicit purpose
  - deterministic contract
  - owner
  - machine-readable output
- Every fixed bug in scope lands with permanent regression coverage.
- Every unresolved crash tracked by this phase has issue linkage.
- Every corpus introduced in this phase is version-controlled and reproducible.
- Validation evidence is recorded in the phase execution checklist issue before merge.
- Every milestone includes at least one positive-path and one negative-path validation case.
- No milestone is complete if its outputs are not reviewable and reproducible locally.

### Validation planning goals
- `milestone_29_1` (Suite Taxonomy and Baseline Governance): validation goals cover: Define canonical suite taxonomy and per-suite contracts; Add baseline-backed verification for diagnostics and project behavior; Define canonical checked-in artifacts, normalization rules, and a bless/accept workflow. Include negative-path goals that catch regressions against these guarantees.
- `milestone_29_2` (Fixedbugs and Crashes Corpus): validation goals cover: Require every resolved compiler bug in scope to land in `fixedbugs`; Add issue-linked metadata and root-cause traceability; Define `crashes` sentinel policy and promotion rules. Include negative-path goals that catch regressions against these guarantees.
- `milestone_29_3` (Fuzz and Property Operationalization): validation goals cover: Define fuzz targets, property suites, and seed corpora for highest-value compiler surfaces; Define reproducibility, deduplication, minimization, and triage rules; Separate local smoke fuzz/property gates from longer-running sustained lanes. Include negative-path goals that catch regressions against these guarantees.
- `milestone_29_4` (Curated OSS Gate and Broader Ecosystem Lane): validation goals cover: Build a small pinned curated real-world/project corpus that blocks merges; Define a separate broader non-blocking ecosystem lane; Require structured result classification and reproducible execution. Include negative-path goals that catch regressions against these guarantees.
- `milestone_29_5` (Deterministic Scale, Flake Control, and Structured Evidence): validation goals cover: Define deterministic sharding and per-suite runtime expectations; Add repeat-run and sequential-vs-parallel equivalence checks; Define rerun/quarantine policy for flakes; Require machine-readable artifacts from all hardening gates; Add suggestion/autofix validation if suggestions are part of the stable contract. Include negative-path goals that catch regressions against these guarantees.
- Exit-gate evidence explicitly demonstrates: Verification hardening is a concrete compiler verification system rather than an informal collection of tests.

## Local Validation Commands
- Full local suite:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Quick hardening gate:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile create-pr`
- Full hardening gate:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile merge`
- Stress/determinism gate:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile release`
- Determinism recheck:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/check_e2e_report_determinism.sh`
- Fuzz/property smoke runner:
  - `/Users/yaseralnajjar/work/sifr/codebase/verification/areas/fuzz_property/`
- Milestone demos:
  - `cargo run -q -p sifr -- run demos/<milestone_demo>.sifr`
- Baseline bless/accept command:
  - `python3 /Users/yaseralnajjar/work/sifr/codebase/scripts/run_verification_hardening.py --profile merge --bless`
- Curated OSS gate runner:
  - `python3 /Users/yaseralnajjar/work/sifr/codebase/scripts/run_verification_hardening.py --profile merge --suite oss-curated`
- Sequential-vs-parallel equivalence check:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/check_e2e_sequential_parallel_equivalence.sh --profile merge`

All suite kinds introduced in this phase must be invocable through the canonical local validation entrypoints and must emit structured machine-readable results.

## Required Policies
The phase must define and keep current:
- suite taxonomy policy
- baseline/bless policy
- normalization policy
- fixedbugs policy
- crashes/sentinel policy
- fuzz triage and minimization policy
- curated OSS gate policy
- broader ecosystem lane policy
- deterministic sharding policy
- rerun/quarantine/flake policy
- machine-readable artifact schema and retention policy

## Required Artifacts
- suite taxonomy document
- baseline governance document
- normalization rules
- fixedbugs index
- crashes/sentinel index
- fuzz target and corpus manifest
- curated OSS corpus manifest with pinned revisions
- broader ecosystem lane definition
- deterministic sharding and flake policy
- structured gate result schema
- exit-gate evidence summary

## Exit Criteria
- Compiler verification suites are explicit, deterministic, and locally enforceable.
- Baseline-backed compiler outputs are governed by a bless/accept workflow.
- Resolved compiler bugs in scope are preserved in a permanent regression corpus.
- Known unresolved crashes are visible and intentionally tracked.
- Fuzz/property operations are active and documented.
- Curated OSS gate is reproducible and blocking.
- Broader ecosystem lane is defined separately and non-blocking.
- Sharding, rerun, and flake rules are active and enforced.
- Hardening gates emit structured machine-readable evidence.

## Exit Gate
Verification hardening is a concrete compiler verification system rather than an informal collection of tests: suites are explicit, baselines are governed, regressions are issue-linked, curated real-world validation is active, fuzz/property operations are defined, deterministic scaling and flake control are enforced, and results are mechanically triageable.
Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
