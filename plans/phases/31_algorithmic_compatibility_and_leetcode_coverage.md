# Phase 31: Algorithmic Compatibility and LeetCode Coverage

status: complete

> 2026-03-11 update: milestones `31_1` through `31_5` are implemented in the workspace with a canonical 50-problem seed corpus, a deterministic runner, a generated failure taxonomy, a ranked remediation backlog, a first remediation wave, and stable scorecard/handoff artifacts. External review sign-off is complete.

> 2026-04-27 update: the LeetCode sub-repository was cleaned to keep only the
> source corpus and root helper scripts. Historical Phase 31 generated artifacts,
> old audit reports, internal notes, and obsolete helper scripts were removed
> from the live tree and remain recoverable from `sifr-lang/leetcode` git
> history.

## Objective
Run a representative LeetCode corpus end-to-end on Sifr, identify failures, classify root causes, and define the language/compiler fixes required to improve algorithmic compatibility.

## Depends on
- Phase 30 (`reliability_parity_and_performance_budgets`) exit gate must be satisfied before Phase 31 execution begins.

## Non-goals
- Solving all LeetCode problems manually as product work.
- Adding unrelated new language features without failure-driven justification.
- Optimizing leaderboard performance before correctness/compatibility is established.
- Supporting problems that require external packages beyond current stdlib/runtime scope in this phase.

## Corpus Policy
- Start with a deterministic, version-controlled seed corpus.
- Coverage targets must be topic-balanced (arrays, strings, hash maps, DP, graphs, trees, backtracking, math, heap/priority queue, two pointers, sliding window).
- Include mixed difficulty (`easy`, `medium`, `hard`) with explicit counts.
- Include an initial minimum corpus size of `>= 50` problems.
- Mark each problem as one of:
  - `in_scope` (expected to run in current language/runtime scope)
  - `blocked_feature` (requires not-yet-implemented language/compiler feature)
  - `out_of_scope_external_dep` (requires non-stdlib package/runtime dependency)

## Milestones

### milestone_31_1: Corpus and Runner Baseline
status: complete

- Scope:
  - Define corpus size, selection criteria, and topic/difficulty distribution.
  - Build deterministic runner harness for `check/build/run` result capture.
  - Define timeout policy (per problem and global run budget) and deterministic retry rules.
  - Define output oracle strategy:
    - use problem-provided sample cases,
    - plus locally defined regression inputs,
    - plus reference output comparison baseline (Python implementation where available).
- Definition of done:
  - Corpus list is version-controlled and reproducible.
  - Runner is deterministic and emits structured results.
  - Timeout and oracle policy are documented and enforced.
  - Baseline pass/fail/timeout metrics are generated.
- Delivered artifacts:
  - historical artifact `phase31_seed_corpus.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_corpus_inventory.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_seed_summary.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical note artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
- Baseline metrics (2026-03-11 seed run):
  - `case_count=50`
  - `status_counts={CHECK_ERROR: 46, RUN_ERROR: 2, PASS: 2}`
  - topic balance satisfied at `5` cases per required topic
  - difficulty split: `easy=18`, `medium=27`, `hard=5`

---

### milestone_31_2: Failure Inventory and Root-Cause Taxonomy
status: complete

- Scope:
  - Classify failures by layer: parser, type system, lowering, codegen, stdlib/runtime, performance timeout, unsupported feature.
  - Require minimal reproducible case for each unique failure class.
  - Add false-positive handling policy for misclassified failures.
  - Add spot-audit workflow for classification accuracy.
- Definition of done:
  - Every failing case is tagged with root-cause category and reproducible evidence.
  - Taxonomy report includes frequency and impact ranking.
  - Classification spot-audit accuracy is `>= 90%`.
- Delivered artifacts:
  - historical artifact `phase31_failure_taxonomy.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_failure_repros.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_spot_audit.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_spot_audit_cases.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_failure_report.md` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_failure_report.md` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
- Baseline taxonomy metrics (2026-03-11 seed run):
  - classified failing seed cases: `48`
  - taxonomy buckets: `12`
  - largest buckets:
    - `type_system.optional_narrowing_and_union_ops` (`16`)
    - `lowering.destructuring_target_support` (`7`)
    - `frontend.nested_function_annotation_support` (`6`)
    - `stdlib.python_module_surface` (`6`)
  - spot-audit accuracy: `100%`

---

### milestone_31_3: Compatibility Fix Plan (Language + Compiler)
status: complete

- Scope:
  - Convert ranked blockers into concrete milestones/issues with acceptance criteria.
  - Tag each item as `bug`, `spec_gap`, or `intentional_divergence`.
  - Add rough effort sizing and dependency tags for sequencing.
  - Define approval process for roadmap insertion (owner + reviewer sign-off).
  - Define escalation policy for stale blockers.
- Definition of done:
  - Prioritized remediation backlog exists with owners, dependencies, and acceptance criteria.
  - Intentional divergences are explicitly documented.
  - Plan is approved and linked into roadmap phases.
  - Unresolved `P1` blockers older than 14 days are escalated with explicit owner reassignment or defer decision.
- Delivered artifacts:
  - historical artifact `phase31_remediation_backlog.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_remediation_backlog.md` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_remediation_backlog.md` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
- Backlog summary (2026-03-11):
  - backlog entries: `12` (one per taxonomy bucket)
  - `P1` items: `6`
  - wave candidates for milestone_31_4 selection: `5`
  - explicit intentional divergence recorded for `ownership.borrowed_return_surface`
  - stale blocker escalation threshold: `14` days for `P1`

---

### milestone_31_4: First Compatibility Remediation Wave
status: complete

- Scope:
  - Implement highest-leverage fixes using explicit selection criteria:
    - unblock count across corpus,
    - severity,
    - risk,
    - dependency readiness.
  - Add regression tests per fixed blocker.
  - Re-run corpus after each remediation batch.
- Definition of done:
  - First remediation batch lands with regression coverage.
  - Pass-rate improvement is measurable against baseline.
  - No regression in previously passing corpus slice.
- Delivered artifacts:
  - historical artifact `phase31_seed_results_wave1.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_wave1_delta.md` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_wave1_summary.md` (removed from the lean LeetCode subrepo; recoverable from git history)
  - `crates/sifr/tests/e2e/pass/function_shadowing_builtin_sum.sifr`
  - `crates/sifr/tests/e2e/pass/borrowed_param_shadowing.sifr`
  - `crates/sifr/tests/e2e/pass/tuple_unpack_reassignment.sifr`
  - `crates/sifr/tests/e2e/fail/builtin_sum_wrong_arity.sifr`
- Wave 1 metrics (2026-03-11):
  - seed status counts: `PASS=5`, `CHECK_ERROR=45`, `RUN_ERROR=0`
  - delta vs baseline: `PASS +3`, `CHECK_ERROR -1`, `RUN_ERROR -2`
  - fixed seed cases:
    - `0069_sqrtx`
    - `0151_reverse_words_in_a_string`
    - `2235_add_two_integers`
  - regression note: borrowed-parameter shadowing is limited to true rebinding sites, preserving in-place mutation semantics for existing stdlib and e2e coverage

---

### milestone_31_5: Compatibility Scorecard and Handoff
status: complete

- Scope:
  - Publish scorecard with:
    - total/pass/fail/timeout counts,
    - category breakdown,
    - `in_scope` vs `blocked_feature` vs `out_of_scope_external_dep`,
    - before/after remediation delta.
  - Record unresolved blockers with linked issues and target phases.
  - Run review/sign-off for phase closure.
- Definition of done:
  - Scorecard is published in a stable, repeatable format.
  - Open blockers are roadmap-mapped with owners.
  - Phase closure is approved with explicit handoff targets.
- Delivered artifacts:
  - historical artifact `phase31_scorecard.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_scorecard.md` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `phase31_scorecard.md` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical helper artifact (removed from the lean LeetCode subrepo; recoverable from git history)
- Scorecard summary (2026-03-11):
  - review status: `external_review_approved`
  - baseline status counts: `PASS=2`, `CHECK_ERROR=46`, `RUN_ERROR=2`
  - wave-1 status counts: `PASS=5`, `CHECK_ERROR=45`, `RUN_ERROR=0`
  - unresolved handoff entries: `9`
  - carried-forward targets:
    - unresolved former `phase31` proposals roll forward to `phase32`
    - `ownership.borrowed_return_surface` remains `deferred` as an intentional divergence
  - phase closure note: artifact publication and external review sign-off are complete

## Ad Hoc Follow-up Milestones

Phase 31 is closed, but its unresolved compatibility backlog now has a concrete carry-forward execution plan in `plans/issues/archive/phase31-ad-hoc-followup-milestones.md`.

Latest closure-track plan (`2026-04-05`):
- `plans/issues/archive/ad-hoc-codegen-runtime-build-gap-closure-phase-2026-04-05.md` is marked `ready_to_implement` for the live `codegen_runtime_build_gap=58` baseline.
- Source artifacts:
  - historical artifact `full_corpus_current_results_20260405_live_rerun1.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `full_corpus_failure_taxonomy_20260405_live_rerun1.json` (removed from the lean LeetCode subrepo; recoverable from git history)
  - historical artifact `codegen_runtime_build_gap_breakdown_20260405_v3.csv` (removed from the lean LeetCode subrepo; recoverable from git history)
- External review sign-off for this breakdown/phase basis reached `READY`.

Latest execution note (`2026-03-11`):
- `m31_c_stdlib_module_parity` slice 1 is locally validated and tracked in `plans/issues/archive/phase31-m31c-stdlib-module-parity-execution.md`.
- Targeted six-case rerun artifact: historical artifact `phase31_m31c_wave1_results.json` (removed from the lean LeetCode subrepo; recoverable from git history).
- Measured outcome for that slice: `PASS=1`, `CHECK_ERROR=5`, `RUN_ERROR=0`, with `0007_reverse_integer` now passing and `0502_ipo` advanced past the original missing-stdlib blocker.

Ownership/mutability boundary closure update (`2026-04-02`):
- Analysis and execution ledger: `plans/issues/archive/ownership-mutability-boundary-root-cause-2026-04-02.md`.
- Targeted post-fix artifact: historical artifact `ownership_mutability_boundary_targeted_results_20260402_post_fix.md` (removed from the lean LeetCode subrepo; recoverable from git history).
- Full-corpus rerun artifact: historical artifact `full_corpus_current_results_20260402_live_after_ownership_boundary_closure.json` (removed from the lean LeetCode subrepo; recoverable from git history).
- Post-closure breakdown artifact: historical artifact `ownership_mutability_boundary_breakdown_20260402_live_after_closure.json` (removed from the lean LeetCode subrepo; recoverable from git history).
- Outcome: ownership-category first-diagnostic count reduced to `0` in the corpus rerun; residual failures are secondary non-ownership defects.
- Secondary check-remediation wave 1 artifact: historical artifact `ownership_mutability_boundary_check_results_20260402_wave1.md` (removed from the lean LeetCode subrepo; recoverable from git history) (`24/47` check-pass).
- Secondary check-remediation wave 2 artifact: historical artifact `ownership_mutability_boundary_check_results_20260402_wave2.md` (removed from the lean LeetCode subrepo; recoverable from git history) (`47/47` check-pass).

- Planned follow-up milestones:
  - `m31_a_optional_narrowing_core`
  - `m31_b_destructuring_target_lowering`
  - `m31_c_stdlib_module_parity`
  - `m31_d_nested_function_pipeline`
  - `m31_e_tree_node_surface`
  - `m31_f_ownership_divergence_resolution`
- Remaining seed-corpus surface that this plan covers:
  - `44` supportable failing cases
  - `1` explicit intentional divergence case
- Sequencing rationale:
  - largest independent blockers first
  - then dependency-ordered enablement chains for nested helpers and recursive tree-node support

## Quality Contract

### Entry criteria
- Phase 30 exit gate is satisfied.
- Corpus seed and runner contract are approved.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.

### Milestone quality checks
- Local validation gates must pass before merge.
- Runner output must be deterministic and reproducible on repeated runs.
- Every fixed blocker must include regression coverage.
- No blocker may be closed without root-cause classification and reproducible evidence.
- Validation evidence must be recorded in the phase execution checklist issue before merge.

### Validation planning goals
- milestone_31_1:
  - Positive: corpus generation and full runner execution succeed on supported samples.
  - Negative: malformed inputs/timeouts are captured with expected diagnostics and status codes.
- milestone_31_2:
  - Positive: known seeded failures are classified into expected taxonomy buckets.
  - Negative: deliberately mis-tagged cases are detected by spot-audit checks.
- milestone_31_3:
  - Positive: prioritized backlog is dependency-sorted and approval-complete.
  - Negative: incomplete/ambiguous items are rejected by plan validation rules.
- milestone_31_4:
  - Positive: remediation batch improves pass rate versus baseline.
  - Negative: intentionally introduced regression is caught by corpus regression gates.
- milestone_31_5:
  - Positive: scorecard and roadmap handoff artifacts are complete and reproducible.
  - Negative: missing owner/phase mapping fails closure checklist.

### Local validation commands
- Full local suite:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Historical LeetCode Phase 31 runner artifacts:
  - removed from the live LeetCode sub-repository; recover from git history if
    auditing the completed phase.
- Repeat determinism check:
  - historical requirement for the removed Phase 31 runner artifacts.

### E2E test expectations
- Add/maintain E2E tests for:
  - runner determinism,
  - timeout classification,
  - root-cause classification integrity,
  - regression coverage for each resolved blocker.
- Each remediation PR must include:
  - at least one positive-path test,
  - at least one negative-path test,
  - updated corpus metrics snapshot.

### Exit criteria
- Baseline corpus execution is reproducible.
- Failure taxonomy and prioritized remediation backlog are complete and approved.
- First remediation wave is complete with measurable pass-rate improvement.
- Scorecard is published and unresolved blockers are mapped to future phases.
- Any waiver is explicit, time-bounded, owner-assigned, and issue-linked.

## Exit Gate
LeetCode compatibility is measurable, root causes are classified, top blockers are partially remediated, and the remaining plan is fully roadmap-integrated with approved ownership and sequencing.
Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
