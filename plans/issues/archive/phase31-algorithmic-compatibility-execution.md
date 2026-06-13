# Phase 31 Execution Checklist (Algorithmic Compatibility and LeetCode Coverage)

Status: complete (started 2026-03-11, closed 2026-03-11)
Owner: phase_31 execution loop
Reference phase docs:
- `internal_docs/phases/31_algorithmic_compatibility_and_leetcode_coverage.md`
- `internal_docs/roadmap.md`
- `internal_docs/architecture.md`

Historical note:
- `issues/phase31-sifr-driver-decomposition-and-boundary-hardening-execution.md` reflects an older phase numbering scheme. The active roadmap on 2026-03-11 assigns Phase 31 to algorithmic compatibility, so this file is the authoritative execution tracker for the current Phase 31 scope.

Loop per part: Plan -> Implement -> Validate -> Demo -> PR -> Review -> Merge -> Update docs -> Next part

Carry-forward planning:
- Remaining compatibility work is broken into execution-ready follow-up milestones in `issues/phase31-ad-hoc-followup-milestones.md`.

## Global Gates
- [ ] Scope remains constrained to the active phase-31 part
- [ ] Root cause addressed without fallback behavior or ad hoc compatibility shims
- [ ] Positive-path and negative-path validation recorded for the active part
- [ ] Deterministic runner output is regenerated from version-controlled inputs
- [ ] Demo runs successfully before opening the PR for the active part
- [ ] Full local suite passes before merge: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] PR is opened, reviewed, and merged before the next part starts
- [ ] Planning docs, roadmap status, and issue tracker state are updated before moving on
- [ ] Phase 27 non-regression invariants remain green throughout the phase

## Initial Investigation Summary

### What already exists
- `audits/leetcode/` already contains a large pinned corpus of converted LeetCode programs (`.sifr`) plus source references (`.py`).
- `audits/leetcode/run_audit.py` is a historical ad hoc harness, but it is not wired into the canonical workspace validation flow and hardcodes external paths.
- `audits/leetcode/audit_results.json`, `audits/leetcode/REPORT.md`, and `audits/leetcode/POST_HARDENING_REPORT.md` provide historical compatibility snapshots.

### Current inconsistencies that Phase 31 must resolve first
- The roadmap says Phase 31 is still `draft`, but the repo already contains a substantial LeetCode corpus.
- Historical baseline artifacts disagree:
  - `audits/leetcode/audit_results.json` currently records `376` problems with `4 PASS`, `1 WRONG_OUTPUT`, and `371 COMPILE_ERROR`.
  - `audits/leetcode/REPORT.md` documents `411` files and `39` end-to-end passes.
  - `audits/leetcode/POST_HARDENING_REPORT.md` documents a February 16, 2026 post-hardening view with `39/411` end-to-end passes.
- Because the corpus definition, runner contract, and scorecard format are not canonicalized, the current repo cannot answer the core phase-31 question deterministically: "what is the current compatibility rate, why do failures happen, and what changed after a fix?"

### Current blocker signal from `audits/leetcode/audit_results.json`
- Total corpus entries: `376`
- Status counts:
  - `COMPILE_ERROR`: `371`
  - `PASS`: `4`
  - `WRONG_OUTPUT`: `1`
- Category counts:
  - `algo`: `288`
  - `node`: `58`
  - `design`: `30`
- Test coverage in manifest:
  - `has_tests=true`: `172`
  - `has_tests=false`: `204`
- Top repeated failure strings in the current JSON artifact:
  - `unknown type: 'TreeNode'` (`29`)
  - `unsupported statement type` (`28`)
  - `unknown type: 'ListNode'` (`20`)
  - `undefined function: 'set'` (`18`)
  - `len() ... got 'list[int] | None'` (`17`)
  - `assignment target must be a simple name` (`17`)
  - `for loop target must be a simple name` (`14`)
  - `cannot iterate over type 'str'` (`12`)
  - `unsupported expression type` (`10`)
  - `cannot iterate over type 'range'` (`10`)

### Phase-31 execution stance
- Part 1 must canonicalize the corpus and runner before any remediation work is considered valid.
- Part 2 must make failure taxonomy reproducible from machine-readable runner output, not a one-off markdown report.
- Part 3 must turn the ranked failures into an approved remediation backlog.
- Part 4 must land at least one high-leverage remediation wave with measurable corpus improvement.
- Part 5 must publish a stable scorecard and close the phase with explicit handoff for unresolved blockers.

## Full Phase 31 To-Do Plan

### milestone_31_1: Corpus and Runner Baseline
1. [x] `m31_1a_corpus_manifest`
   - Define the canonical Phase 31 seed corpus from `audits/leetcode/`.
   - Add version-controlled metadata per problem: id, slug, source paths, topic tags, difficulty, scope classification, oracle type, timeout class.
   - Resolve the current `376` vs `411` mismatch explicitly in docs and generated outputs.
2. [x] `m31_1b_runner_contract`
   - Replace the ad hoc runner contract with a deterministic workspace-native runner.
   - Emit structured machine-readable output with phase-stable status values and failure stages.
   - Enforce per-case timeout and whole-run summary generation.
3. [x] `m31_1c_docs_demo_and_validation`
   - Add docs for corpus format, runner CLI, determinism requirements, and status taxonomy.
   - Add a demo that exercises the runner against a deterministic small sample corpus.
   - Add tests that lock the runner schema, determinism behavior, and timeout handling.

### milestone_31_2: Failure Inventory and Root-Cause Taxonomy
4. [x] `m31_2a_taxonomy_schema`
   - Define canonical failure buckets by compiler layer and problem scope.
   - Teach the runner/report pipeline to map raw failures into that taxonomy deterministically.
5. [x] `m31_2b_minimal_repro_inventory`
   - Attach minimal reproducible evidence for each unique high-frequency failure class.
   - Store the evidence in a stable location linked from the generated report.
6. [x] `m31_2c_spot_audit`
   - Add a spot-audit script/check to measure classification accuracy and reject stale or ambiguous tags.

### milestone_31_3: Compatibility Fix Plan
7. [x] `m31_3a_ranked_backlog`
   - Convert ranked taxonomy buckets into concrete remediation items with acceptance criteria and dependency notes.
   - Mark each item as `bug`, `spec_gap`, or `intentional_divergence`.
8. [x] `m31_3b_docs_and_roadmap_alignment`
   - Update roadmap/architecture/phase docs with approved backlog links and explicit deferred items.

### milestone_31_4: First Compatibility Remediation Wave
9. [x] `m31_4a_select_wave`
   - Choose the highest-leverage blockers that are root-cause-fixable within Phase 31.
   - Selection must be justified by corpus impact, implementation risk, and dependency readiness.
10. [x] `m31_4b_implement_wave`
   - Land the chosen compiler/language/runtime fixes with regression coverage and rerun evidence.
11. [x] `m31_4c_measure_delta`
   - Regenerate the corpus report and record before/after improvement in the tracker.

### milestone_31_5: Compatibility Scorecard and Handoff
12. [x] `m31_5a_publish_scorecard`
   - Publish a stable scorecard artifact with total/pass/fail/timeout counts and category breakdown.
13. [x] `m31_5b_phase_closeout`
   - Record unresolved blockers, owners, and future-phase mapping.
   - Update roadmap/phase status and close Phase 31 only after review sign-off.

## Part-by-Part Execution Notes

### Part 1 target (`m31_1a_corpus_manifest` + `m31_1b_runner_contract` + `m31_1c_docs_demo_and_validation`)
- Goal:
  - make Phase 31 measurable and reproducible from the current repo alone
- Expected deliverables:
  - canonical corpus manifest
  - deterministic runner integrated into the workspace
  - structured result JSON
  - sample demo
  - tests for runner determinism/schema
  - updated docs
- Validation target:
  - targeted runner tests
  - demo run
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Status:
  - complete (merged on 2026-03-11)
- Implementation PR:
  - https://github.com/sifr-lang/sifr/pull/1100
- Delivered artifacts:
  - `verification/leetcode/phase31_seed_corpus.json`
  - `verification/leetcode/phase31_corpus_inventory.json`
  - `verification/leetcode/phase31_seed_summary.json`
  - `verification/leetcode/phase31_seed_results.json`
  - `demos/m31_1_leetcode_runner_demo/corpus.json`
  - `demos/m31_1_leetcode_runner_demo/results.json`
  - `internal_docs/verification/phase31_leetcode_corpus_policy.md`
  - `scripts/phase31_leetcode_lib.py`
  - `scripts/build_phase31_leetcode_assets.py`
  - `scripts/run_phase31_leetcode.py`
  - `scripts/test_phase31_leetcode.py`
- Validation evidence:
  - positive path: `python3 scripts/build_phase31_leetcode_assets.py` -> regenerated the seed corpus, raw inventory, summary, and demo manifest successfully
  - positive path: `python3 scripts/test_phase31_leetcode.py` -> passed (`8` tests)
  - positive path: `python3 scripts/run_phase31_leetcode.py --manifest demos/m31_1_leetcode_runner_demo/corpus.json --output demos/m31_1_leetcode_runner_demo/results.json` -> passed and produced representative demo statuses (`PASS=2`, `RUN_ERROR=1`, `CHECK_ERROR=2`)
  - positive path: `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/phase31_seed_corpus.json --output verification/leetcode/phase31_seed_results.json` -> produced the seed baseline (`CHECK_ERROR=46`, `RUN_ERROR=2`, `PASS=2`)
  - negative path: timeout handling is covered by `python3 scripts/test_phase31_leetcode.py` via `test_runner_classifies_timeout`
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - merge evidence: PR #1100 merged into `main` on 2026-03-11

### Part 2 target (`m31_2a_taxonomy_schema` + `m31_2b_minimal_repro_inventory` + `m31_2c_spot_audit`)
- Goal:
  - turn the seed baseline into a stable, machine-readable failure taxonomy with reproducible evidence and a classifier accuracy gate
- Expected deliverables:
  - taxonomy rules with stable bucket ids
  - generated taxonomy JSON and markdown report
  - smallest-known repro inventory per bucket
  - version-controlled spot-audit dataset and accuracy report
  - demo report under `demos/`
  - classifier tests
- Validation target:
  - targeted taxonomy generator and tests
  - taxonomy demo report generation
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Status:
  - complete (merged on 2026-03-11)
- Implementation PR:
  - https://github.com/sifr-lang/sifr/pull/1101
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
- Validation evidence:
  - positive path: `python3 scripts/build_phase31_leetcode_taxonomy.py` -> regenerated taxonomy JSON, repro inventory, spot-audit outputs, and demo report successfully
  - positive path: `python3 scripts/test_phase31_leetcode_taxonomy.py` -> passed (`4` tests)
  - positive path: `verification/leetcode/phase31_failure_report.md` classifies all `48` failing seed cases into `12` stable buckets
  - positive path: `verification/leetcode/phase31_spot_audit.json` records `accuracy=1.0` against a `0.9` threshold
  - negative path: the spot-audit dataset would fail if a bucket mapping regressed below the threshold enforced by `python3 scripts/test_phase31_leetcode_taxonomy.py`
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - merge evidence: PR #1101 merged into `main` on 2026-03-11

### Part 3 target (`m31_3a_ranked_backlog` + `m31_3b_docs_and_roadmap_alignment`)
- Goal:
  - convert the taxonomy into an approved, dependency-aware remediation backlog with explicit ownership, divergence handling, and stale-blocker governance
- Expected deliverables:
  - machine-readable remediation backlog covering every taxonomy bucket
  - markdown remediation plan with approval process and stale-blocker policy
  - explicit `bug` / `spec_gap` / `intentional_divergence` tagging
  - demo report under `demos/`
  - backlog validator tests
- Validation target:
  - targeted remediation backlog generator and tests
  - remediation-plan demo generation
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Status:
  - complete (merged on 2026-03-11)
- Implementation PR:
  - https://github.com/sifr-lang/sifr/pull/1102
- Delivered artifacts:
  - `verification/leetcode/phase31_remediation_backlog.json`
  - `verification/leetcode/phase31_remediation_backlog.md`
  - `demos/m31_3_leetcode_remediation_plan_demo/report.md`
  - `scripts/phase31_leetcode_remediation.py`
  - `scripts/build_phase31_leetcode_remediation_backlog.py`
  - `scripts/test_phase31_leetcode_remediation_backlog.py`
- Validation evidence:
  - positive path: `python3 scripts/build_phase31_leetcode_remediation_backlog.py` -> regenerated the remediation backlog JSON, markdown plan, and demo report successfully
  - positive path: `python3 scripts/test_phase31_leetcode_remediation_backlog.py` -> passed (`4` tests)
  - positive path: `verification/leetcode/phase31_remediation_backlog.json` covers all `12` taxonomy buckets with explicit owner, priority, effort, dependencies, and acceptance criteria
  - positive path: `verification/leetcode/phase31_remediation_backlog.md` records the approval process and stale `P1` blocker escalation policy (`14` days)
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - merge evidence: PR #1102 merged into `main` on 2026-03-11

### Part 4 target (`m31_4a_select_wave` + `m31_4b_implement_wave` + `m31_4c_measure_delta`)
- Goal:
  - land a low-risk first remediation wave that removes the seed run-stage failures and fixes one builtin-shadowing check-stage failure without regressing the existing pass slice
- Selected blockers:
  - `codegen.mutable_binding_emission` (`0069_sqrtx`)
  - `codegen.generic_run_failure` (`0151_reverse_words_in_a_string`)
  - `stdlib.python_builtin_signature_surface` (`2235_add_two_integers`)
- Expected deliverables:
  - compiler fixes for the three selected repros
  - regression coverage in unit and e2e suites
  - wave-specific seed result snapshot and before/after delta report
  - demo report under `demos/`
- Validation target:
  - targeted unit tests for tuple destructuring and builtin shadowing
  - direct canonical release-binary runs for the fixed repros
  - seed corpus rerun plus normalized repeat/determinism check
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Status:
  - complete (implementation + local validation finished on 2026-03-11; PR/merge pending)
- Delivered artifacts:
  - `verification/leetcode/phase31_seed_results_wave1.json`
  - `verification/leetcode/phase31_wave1_delta.md`
  - `demos/m31_4_leetcode_remediation_wave1_demo/report.md`
  - `crates/sifr/tests/e2e/pass/phase31_builtin_shadow_sum.sifr`
  - `crates/sifr/tests/e2e/pass/phase31_mutated_borrowed_param_shadow.sifr`
  - `crates/sifr/tests/e2e/pass/phase31_tuple_unpack_mutability.sifr`
  - `crates/sifr/tests/e2e/fail/phase31_builtin_sum_wrong_arity.sifr`
- Validation evidence:
  - positive path: `cargo test -p sifr_hir test_user_defined_sum_shadows_builtin` -> passed
  - positive path: `cargo test -p sifr_codegen lowers_simple_tuple_unpack_stmt_with_mutated_bindings -- --nocapture` -> passed
  - positive path: `target/release/sifr run audits/leetcode/0069_sqrtx.sifr` -> passed
  - positive path: `target/release/sifr run audits/leetcode/0151_reverse_words_in_a_string.sifr` -> passed
  - positive path: `target/release/sifr run audits/leetcode/2235_add_two_integers.sifr` -> passed
  - positive path: `target/release/sifr run crates/sifr/tests/e2e/pass/cpython_heapq_subset.sifr` -> passed after narrowing borrowed-param shadowing to true rebindings only
  - positive path: `target/release/sifr run crates/sifr/tests/e2e/pass/non_literal_default_args.sifr` -> passed, confirming mutated `own` params still receive mutable local shadows
  - positive path: `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/phase31_seed_corpus.json --output verification/leetcode/phase31_seed_results_wave1.json` -> produced the wave-1 snapshot (`CHECK_ERROR=45`, `PASS=5`)
  - positive path: normalized repeat run of the same seed command matched after excluding `duration_ms` and run-specific temp/output paths
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase31_builtin_sum_wrong_arity.sifr` -> preserved the builtin `sum()` arity diagnostic when no user-defined shadow exists
  - regression note: an over-broad borrowed-param shadowing change initially broke heapq/in-place mutation paths; the final implementation narrows shadowing to borrowed-param rebindings while preserving mutable-local shadows for mutated `own` params
  - delta summary:
    - baseline: `PASS=2`, `CHECK_ERROR=46`, `RUN_ERROR=2`
    - wave 1: `PASS=5`, `CHECK_ERROR=45`, `RUN_ERROR=0`
    - changed cases:
      - `0069_sqrtx`: `RUN_ERROR -> PASS`
      - `0151_reverse_words_in_a_string`: `RUN_ERROR -> PASS`
      - `2235_add_two_integers`: `CHECK_ERROR -> PASS`
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)

### Part 5 target (`m31_5a_publish_scorecard` + `m31_5b_phase_closeout`)
- Goal:
  - publish a stable scorecard for the phase state after wave 1 and record the unresolved-blocker handoff needed for final phase closure
- Expected deliverables:
  - machine-readable scorecard JSON
  - markdown scorecard and demo report
  - unresolved-blocker handoff map with owner, priority, and carried-forward target phase
  - updated phase/tracker docs reflecting that external review sign-off is the remaining closeout step
- Validation target:
  - scorecard generator and tests
  - regenerated scorecard/demo artifacts
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Status:
  - complete (merged on 2026-03-11; external review sign-off recorded on 2026-03-11)
- Implementation PR:
  - https://github.com/sifr-lang/sifr/pull/1104
- Delivered artifacts:
  - `verification/leetcode/phase31_scorecard.json`
  - `verification/leetcode/phase31_scorecard.md`
  - `demos/m31_5_leetcode_scorecard_demo/report.md`
  - `scripts/phase31_leetcode_scorecard.py`
  - `scripts/build_phase31_leetcode_scorecard.py`
  - `scripts/test_phase31_leetcode_scorecard.py`
- Validation evidence:
  - positive path: `python3 scripts/build_phase31_leetcode_scorecard.py` -> regenerated the scorecard JSON, markdown report, and demo report successfully
  - positive path: `python3 scripts/test_phase31_leetcode_scorecard.py` -> passed (`4` tests)
  - positive path: `verification/leetcode/phase31_scorecard.json` records the stable before/after status counts, bucket breakdown, fixed cases, and unresolved handoff mapping
  - positive path: `verification/leetcode/phase31_scorecard.md` records the final phase state before external review (`PASS=5`, `CHECK_ERROR=45`, `RUN_ERROR=0`)
  - positive path: unresolved carry-forward targets are normalized to `phase32` for still-open phase-31 proposals, with the explicit intentional divergence carried as `deferred`
  - positive path: external review sign-off recorded in `reviews/phase-31-review-pass-3.md` and `reviews/phase-31-review-pass-4.md`
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - merge evidence: PR #1104 merged into `main` on 2026-03-11

## Baseline Validation Log
- 2026-03-11:
  - inspected `.cursor/skills/project-workflow/SKILL.md`
  - confirmed working tree clean before starting phase execution
  - confirmed active roadmap Phase 31 scope is algorithmic compatibility
  - inspected `audits/leetcode/` baseline assets and recorded artifact inconsistency (`376` vs `411`)

## Open Questions Resolved for Execution
- Phase numbering conflict:
  - resolved in favor of `internal_docs/roadmap.md` as the active execution authority
- Whether a corpus already exists:
  - yes; Phase 31 should formalize and operationalize the existing `audits/leetcode` corpus rather than invent a new one from scratch
- Whether the existing audit harness is sufficient:
  - no; it is useful as source material, but not acceptable as the canonical phase runner because it is ad hoc, path-hardcoded, and not tied to the workspace validation contract
