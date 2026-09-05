## Findings (Wave 7.2 pass 3 — budget disposition only)

Ordered by severity. Code review remains closed from pass 2; these are budget/disposition findings against the new merge-run evidence.

### Major

**M-budget-1 — Warm/cold budget advisory is non-blocking and does not block the PR.**
The advisory is emitted by `build_advisories` at `verification/runner/sifr_verify/reports.py:131-167`. It only appends strings to the `advisories` list; nothing in that function raises or alters exit status. The runner exited `0` on both runs, and `run_all_tests.sh --profile merge` does not consult `advisories` to compute an exit code. The two configured wall-time fields at `verification/profiles/merge.json:5-8` are budget *targets* paired with an advisory emitter, not a hard gate. So the warm-budget advisory is informational by design.

**M-budget-2 — Wave 7.2's contribution is not the cause of the overrun.**
- Warm run: 1235.88 s vs 900 s budget → 335.9 s over.
- `fuzz_property_checks` step elapsed 57.986 s.
- Removing the new step would still leave warm at ~1177.9 s, i.e. ~19.6 min, still over the 15 min budget by ~4.6 min.
- The slowest steps the report itself surfaces are `verification_hardening_suites` 472.315 s and `generated_code_quality_checks` 236.996 s — both pre-existing merge steps unchanged by this PR.
- Cold run: 1750.45 s vs 1500 s budget → 250.5 s over, fuzz step ~57.280 s. Same conclusion.
The added merge surface in Wave 7.2 (`fuzz_property:fuzz-smoke` at `verification/profiles/merge.json:206-214` and the new `run_fuzz_property_suites` step at `verification/runner/sifr_verify/profile_runner.py:144,321-337`) is exactly the "deterministic minimized smoke" the Wave 7 phase rule wants kept in merge (`plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1346`). Sustained/broad fuzz execution stays in nightly/release per `verification/areas/fuzz_property/sustained_lane.md:21-25`. The phase rule's prescribed mitigation is already in effect.

**M-budget-3 — No further agent review round is required before opening the PR.**
- Pass 2 closed B1/B2 with no remaining code-review blockers.
- The pass-2 PR-readiness condition was "merge profile run passes and records wall-time evidence." Both runs passed (exit 0, 651/651 e2e, `report_signature` identical at `ee5e5d44306f270c`, fuzz_property variants=25 / hardening variants=234 / blocking_failures=0 on both), and warm/cold wall times are recorded in the tracker.
- The advisory itself is non-blocking and not attributable to Wave 7.2 (M-budget-1, M-budget-2).
- Verdict: ready for PR.

### Minor

**m-budget-1 — PR description must call out the advisory explicitly so reviewers don't read silence as compliance.**
Required note wording (suggested verbatim block to lift into the PR body and/or the issue tracker's Wave 7.2 entry):

> Merge gate: `scripts/run_all_tests.sh --profile merge` exits 0 on both cold-cache and warm-cache runs (cache hits 0/182 and 182/182, full e2e 651/651, report signature `ee5e5d44306f270c`, `fuzz_property_checks` variants=25/failures=0/blocking_failures=0, hardening variants=234/failures=0/blocking_failures=0). Wall time was 1750.45 s cold and 1235.88 s warm. Both runs flagged the advisory `warm wall-time budget exceeded` plus `group skew is high`. The new `fuzz_property:fuzz-smoke` merge step contributes ~58 s — even removing it leaves warm at ~19.6 min, over the 15 min advisory target. The overrun is dominated by pre-existing `verification_hardening_suites` (472.3 s) and `generated_code_quality_checks` (237.0 s) steps unchanged by this PR. Per the Wave 7 phase rule in `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1346`, broad sanitizer/fuzz execution is already in nightly/release (see `verification/areas/fuzz_property/sustained_lane.md:21-25`), and merge holds only the deterministic minimized smoke. Budget trimming of the dominant pre-existing steps is tracked as a separate follow-up.

**m-budget-2 — Add a follow-up tracker entry for the merge budget overrun before merging Wave 7.2.**
Recommended placement: an explicit follow-up bullet at the end of the Wave 7.2 task list (`plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1346`) or a new Wave 7.2.x sibling row, with reproduction command `scripts/run_all_tests.sh --profile merge`, the two recorded wall times, and the two slow-step times. Suggested wording:

> Follow-up: merge profile warm wall-time advisory exceeded (1235.88 s vs 900 s warm; 1750.45 s vs 1500 s cold) on both warm- and cold-cache reference-host runs after Wave 7.2 landed. Wave 7.2's `fuzz_property:fuzz-smoke` step is ~58 s and is not the cause; dominant contributors are `verification_hardening_suites` (472.3 s) and `generated_code_quality_checks` (237.0 s). Triage and trim outside Wave 7.2; do not relax `merge.json` budgets without a new measurement on the declared reference host.

**m-budget-3 — Cold-budget overrun is not advised by the runner today; surface it manually in the PR note.**
`build_advisories` at `verification/runner/sifr_verify/reports.py:140` only checks `warm_target_minutes` and never emits an advisory for `cold_wall_time_minutes`. The cold run (1750.45 s) exceeds the `cold_wall_time_minutes: 25` field at `verification/profiles/merge.json:7` by 250.5 s but the report emits only the warm-budget advisory. Not a Wave 7.2 regression, but the PR note should record both warm and cold overruns so the follow-up in m-budget-2 is scoped correctly. Optional improvement (separate PR): extend `build_advisories` to emit a cold-budget string when the run is known cold (e2e cache hits 0/N), so future reports do not silently mask cold overruns.

### Verdict

- **Does the advisory block the PR?** No. The advisory is informational (`reports.py:131-167`), the runner exits 0, and the new merge surface contributes ~58 s out of a ~336 s warm overrun whose dominant contributors pre-date Wave 7.2. The phase-rule remediation (keep only deterministic minimized smoke in merge; broad fuzz in nightly/release) is already in place.
- **Minimal required mitigation in this PR:** none in code. Required is the PR-note + tracker wording in m-budget-1 and m-budget-2, so the overrun is owned by a follow-up and not silently absorbed into Wave 7.2.
- **Further agent review round before PR:** not required.
