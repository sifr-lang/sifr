# Phase 31: Algorithmic Compatibility and LeetCode Coverage

status: complete

> 2026-03-11 update: milestones `31_1` through `31_5` are implemented in the workspace with a canonical 50-problem seed corpus, a deterministic runner, a generated failure taxonomy, a ranked remediation backlog, a first remediation wave, and stable scorecard/handoff artifacts. External review sign-off is recorded in `reviews/phase-31-review-pass-3.md` and `reviews/phase-31-review-pass-4.md`.

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
  - `verification/leetcode/phase31_seed_corpus.json`
  - `verification/leetcode/phase31_corpus_inventory.json`
  - `verification/leetcode/phase31_seed_summary.json`
  - `docs/verification/phase31_leetcode_corpus_policy.md`
  - `scripts/build_phase31_leetcode_assets.py`
  - `scripts/run_phase31_leetcode.py`
  - `scripts/test_phase31_leetcode.py`
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
  - `verification/leetcode/phase31_failure_taxonomy.json`
  - `verification/leetcode/phase31_failure_repros.json`
  - `verification/leetcode/phase31_spot_audit.json`
  - `verification/leetcode/phase31_spot_audit_cases.json`
  - `verification/leetcode/phase31_failure_report.md`
  - `demos/m31_2_leetcode_taxonomy_demo/report.md`
  - `scripts/phase31_leetcode_taxonomy.py`
  - `scripts/build_phase31_leetcode_taxonomy.py`
  - `scripts/test_phase31_leetcode_taxonomy.py`
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
  - `verification/leetcode/phase31_remediation_backlog.json`
  - `verification/leetcode/phase31_remediation_backlog.md`
  - `demos/m31_3_leetcode_remediation_plan_demo/report.md`
  - `scripts/phase31_leetcode_remediation.py`
  - `scripts/build_phase31_leetcode_remediation_backlog.py`
  - `scripts/test_phase31_leetcode_remediation_backlog.py`
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
  - `verification/leetcode/phase31_seed_results_wave1.json`
  - `verification/leetcode/phase31_wave1_delta.md`
  - `demos/m31_4_leetcode_remediation_wave1_demo/report.md`
  - `crates/sifr/tests/e2e/pass/phase31_builtin_shadow_sum.sifr`
  - `crates/sifr/tests/e2e/pass/phase31_mutated_borrowed_param_shadow.sifr`
  - `crates/sifr/tests/e2e/pass/phase31_tuple_unpack_mutability.sifr`
  - `crates/sifr/tests/e2e/fail/phase31_builtin_sum_wrong_arity.sifr`
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
  - `verification/leetcode/phase31_scorecard.json`
  - `verification/leetcode/phase31_scorecard.md`
  - `demos/m31_5_leetcode_scorecard_demo/report.md`
  - `scripts/phase31_leetcode_scorecard.py`
  - `scripts/build_phase31_leetcode_scorecard.py`
  - `scripts/test_phase31_leetcode_scorecard.py`
- Scorecard summary (2026-03-11):
  - review status: `external_review_approved`
  - baseline status counts: `PASS=2`, `CHECK_ERROR=46`, `RUN_ERROR=2`
  - wave-1 status counts: `PASS=5`, `CHECK_ERROR=45`, `RUN_ERROR=0`
  - unresolved handoff entries: `9`
  - carried-forward targets:
    - unresolved former `phase31` proposals roll forward to `phase32`
    - `ownership.borrowed_return_surface` remains `deferred` as an intentional divergence
  - phase closure note: artifact publication and external review sign-off are complete

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
- Milestone demos:
  - `python3 scripts/run_phase31_leetcode.py --manifest demos/m31_1_leetcode_runner_demo/corpus.json --output demos/m31_1_leetcode_runner_demo/results.json`
- LeetCode corpus runner:
  - `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/phase31_seed_corpus.json --output verification/leetcode/phase31_seed_results.json`
- Repeat determinism check:
  - run the corpus command twice with identical config and diff outputs.

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
