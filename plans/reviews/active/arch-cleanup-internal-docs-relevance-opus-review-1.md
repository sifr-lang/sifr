I have enough to give a complete review. Let me write up the findings.

---

## Milestone N+5 Review: Internal Docs Relevance Cleanup

### Scope check
The plan's PR N+5 validation criteria are: (1) roadmap links resolve, (2) phase roadmap links resolve, (3) verification policy docs live under `verification/policy/`, (4) no active internal doc contains long validation transcript ledgers, (5) `git diff --check` passes.

The diff covers: 7 `git mv` moves into `verification/policy/`, 3 in-place edits to those policy files (vocab normalization + cross-link updates), 7 phase-doc retargeting edits, 1 internal_docs paragraph rewrite, 1 reference-path table update in the scripts boundary guardrail, and 1 reference rewrite in `verification/areas/fuzz_property/sustained_lane.md`.

I verified by direct path lookup that every new path quoted in the diff exists on disk:
- `verification/policy/{artifact_schema_and_retention,baseline_governance,deterministic_sharding_and_flake_policy,fuzz_property,profile_policy,regression_corpus,suite_taxonomy}.md` ✓
- `verification/areas/core_language/fixtures/audits/{type_system,python_basics,ownership}/...` (4-phase fixture refs) ✓
- `verification/areas/stdlib_parity/fixtures/audits/stdlib/...` (6 refs) ✓
- `verification/areas/stdlib_parity/{reports/,docs/cpython_parity_fixture_format.md}` ✓
- `verification/areas/distribution_release/cases/install_*.sh, artifact_*.sh, create_new_version_*.sh` (13 refs) ✓
- `verification/areas/package_management/{tools/check_package_manager_guardrails.py, corpora/demo_repositories/}` ✓
- `verification/areas/diagnostics/fixtures/diagnostics/decimal_invalid_literal/`, `verification/areas/project_workspace/fixtures/project/...`, `verification/areas/regression/fixtures/crashes/CR-000{1,2}_*.sifr` (diagnostic inventory table) ✓

Stale-reference scan (`internal_docs/verification`, `internal_docs/validation_lane_policy`) across `internal_docs/`, `plans/phases/`, `plans/roadmap.md`, `scripts/`, `.cursor/`, `.github/`, `AGENTS.md`, `README.md`, `CLAUDE.md`, `docs/`, `crates/`, `verification/`: zero hits in active non-archive surfaces. Remaining hits are confined to (a) the active issue plan's own disposition tables (intentional — they describe the move), (b) `plans/issues/archive/` and `plans/reviews/archive/` (historical, expected to stay).

`scripts/check_scripts_verification_boundary.py:78` REFERENCE_PATHS swap from three explicit `internal_docs/verification/*.md` entries to a directory-scoped `verification/policy` is the correct generalization — the guardrail keeps scanning the same content semantics at the new location and stays in sync with future additions.

`verification/policy/profile_policy.md:55` intentionally retains `[sifr-lane-step]` and `target/validation_lane_reports/<profile>.latest.json` — both still emitted/written by the runner (`verification/runner/sifr_verify/{reports.py:38, profile_runner.py:107,477}`), so the doc accurately describes current behavior. Renaming those runner symbols is out of scope here.

### 1. Blockers
None. The milestone's stated validation criteria are all met and every new path resolves.

### 2. Non-blocking follow-ups
- **Plan self-contradiction not yet swept.** The active issue plan itself contradicts what this milestone executes at three points: line 554 (audits cleanup table allows `internal_docs/verification/` as a convention home), 669 ("keep verification conventions in `internal_docs/verification/`"), 680 ("Verification convention docs may live in `internal_docs/verification/`"). The disposition table at 833–840 supersedes these in practice, but the rule text should be updated to "`verification/policy/` is the convention home" so the plan is internally consistent. Either fold into this PR or queue for N+6.
- **`suite_taxonomy.md` deviation from plan.** Line 839 of the issue plan classifies this file as **delete** after schemas replace it. The milestone instead **moved** it to `verification/policy/suite_taxonomy.md`. This is a reasonable interim choice (the schemas don't yet absorb its content), but the deviation should be called out in the PR description so it isn't lost and a later PR can finish the deletion.
- **"Validation lane" vocabulary still in active docs.** The plan's stale-reference cleanup table at line 850 includes this work; it's still present in `internal_docs/{architecture.md:1295, dependency_policy.md:11, frontend_cache_invalidation.md:82, tooling_verification.md:170,203, integer_model.md:272, frontend_query_architecture.md:107}` and `plans/phases/{32_async_ecosystem.md:536-537, 35_performance_benchmarking_and_budgets.md:576,618}`. Scoping this to N+6 ("Docs And Guardrails Closeout") is defensible given N+5's narrow relocation scope, but should be explicit in the PR description.
- **Empty leftover dir.** `internal_docs/verification/` is empty locally; not tracked by git so the commit is fine, just remove from your working tree.
- **Record execution in the plan.** The PR N+5 block at lines 1314–1325 needs the usual Status/Review/Validation-evidence subsections appended before opening the PR — matching the pattern at N+3 (lines 1245–1278) and N+4 (lines 1286–1312).

### 3. Verdict
**Satisfied** for this milestone PR. The diff is correctly scoped to a relocation-and-link-update slice, every quoted destination exists, the scripts-boundary guardrail follows the move, and your local-validation run is clean. The follow-ups above are housekeeping items, not gate failures.
