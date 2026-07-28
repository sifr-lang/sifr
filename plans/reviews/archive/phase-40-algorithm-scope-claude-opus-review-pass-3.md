## Review — Phase 40 algorithmic release-scope, pass 3

**Verdict: NOT APPROVED** (1 medium, 2 low). Every pass‑2 finding is closed, including the demonstrated deletion exploit. What remains is one narrower residual of the same drift class, demonstrated empirically, plus two mechanical items.

### Pass‑2 findings — verified closed (empirically, in a `/tmp` copy of `verification/` + `plans/`; no repo file modified)

| # | Claim | Verification |
|---|---|---|
| 1 HIGH | Deletable divergence declaration | **Closed for the demonstrated exploit.** Deleting `release_suite`/`release_divergence_record`/`release_divergence_expiry` from `compiler_surface_matrix.json` while release stays reduced now fails: `algorithmic_compatibility_profile: release assignment diverges from nightly without release_suite` (rc=1). Also confirmed rc=1 for content drift (`advertised=[…] assigned=[…]`), bogus suite token (`references unknown suite`), and redundant declaration (`release_suite is declared without a profile assignment divergence`). Present `release_suite` is set-equal to the release assignment, not a subset. |
| 2 MED | Performance step mislabeled | Both places now say `performance_budget_checks` in `full` mode (`ad-hoc-…-failures.md:75-76`, `phase-40-…-execution.md:112-113`); `release.json:376` is `"performance_budget": "full"` (`representative` is merge-only). No PERF-HOST contradiction — that ledger entry is the *merge* lane's representative budget. The narrative order (developer tooling → documentation → performance → algorithmic) matches `profile_runner.py:83-89`, so "then reported the 20 failures" is executable-order-consistent. |
| 3 LOW | `release.json` description | Now "…full breadth except where an expiry-bound release policy applies." |
| 4 LOW | Rejection inventory | `profile_policy.md:50-52` drops "ownerless"; names undeclared drift, expired divergence, missing/unindexed records — all implemented and all self-tested. |
| 5 LOW | Brittle record lookup | Confirmed robust: padded cells pass, a link in the Title cell does not hijack resolution (column 4 only), nonexistent target → `record target does not exist`, deleted `plans/phases/index.md` → `record is not indexed` with no traceback (guarded `OSError`). |
| 6 LOW | Id convention | Documented at `profile_policy.md:141-143`, and enforced CSM→PAM by the `release_suite has no profile assignment row` check. |

Also verified: readiness suite green — `variants=4 failures=0`, `surfaces=34 temporary_rows=0 strict=yes`, self-tests `cases=21` (the five new negative cases all delegate to production code); `representative-subset` = **12 variants, 0 failures** (+ `taxonomy-smoke` = 13), still calling `load_profile_manifest` (`runner.py:327`), so the 411-fixture pin and per-category coverage remain enforced in the release lane; all **20 slugs untouched** by the diff; `nightly.json` untouched and still `leetcode-full` + `taxonomy-smoke`; row `status: blocking`, no advisory flag; `matrix_referenced_areas` gains no new strict area; restoring `leetcode-full` is an ad hoc closeout gate *and* acceptance criterion; all nine pass‑1 findings closed; no release-evidence coupling (`distribution_release`, `docs/`, `plans/releases/` have zero references); README suite list matches `release.json` exactly; no demos added; `file-size guardrails: PASS`, `git diff --check` clean, all touched JSON parses; no unrelated scope.

---

### Findings

**1 — MEDIUM · Divergence is detected from the hand-maintained mirror, not from the profiles, so a two-file deletion still reduces release with no record and no clock.**
`profile_assignment_matrix.py:65-72` derives divergence from `profile_assignment_matrix.json`'s `nightly`/`release` lists. Those lists are only *subset*-checked against the profile JSONs (`validate_row_membership:196-198`), and nothing requires a CSM row to have a PAM row at all (34 surface rows vs 17 PAM rows). Two demonstrated bypasses, both rc=0:

- **Delete the PAM row + the three CSM fields** (`release.json` needs no edit — it is already reduced at HEAD): `profile assignment matrix ok: rows=16`, self-tests `cases=21` pass. `coverage_matrix.py` cannot catch it — `PROFILE_ASSIGNMENT_MATRIX_PATH` (`coverage_matrix.py:26`) is a dead constant, never read.
- **Keep the row, under-declare PAM `nightly`** to match the reduced `release` (subset check permits it): `profile assignment matrix ok: rows=17`.

Net effect is the pass‑2 outcome — release permanently reduced, expiry clock and `ALG-CORPUS` owner record gone — reached by deletion rather than by mutation. The first variant requires no false statement anywhere, only removal of the governance row.
Fix (self-contained, PAM-independent): in `coverage_matrix.py`, for each surface row compare `nightly_release_suite`'s area-suite tokens intersected with `nightly.json`'s selections against the same tokens intersected with `release.json`'s selections, and require `release_suite` whenever they differ. I checked all 34 rows against both profiles with that predicate: **only `algorithmic_compatibility_profile` diverges today, and it declares `release_suite` — zero false positives.** Add a negative self-test for the profile-derived form so the rule, not its bookkeeping, is what fails.

**2 — LOW · Two implemented CSM rules still carry no negative self-test.**
`release_suite must differ from nightly_release_suite` (`coverage_matrix.py:272-273`) and `release divergence metadata requires release_suite` (`:229-230`) have no case in `coverage_matrix_readiness_self_test.py`, while the other five release-divergence rules do. I confirmed both fire correctly by direct call, so this is a coverage gap against the area's one-case-per-enforcement-claim contract, not a defect.

**3 — LOW · `load_release_surface_suites` registers an empty `surface_id`.**
`profile_assignment_matrix.py:144-150`: when a row has `release_suite` but a missing or blank `surface_id`, `require_string` reports the error yet execution continues and stores `release_suites[""]`, so `:83` emits a second, confusing `": release_suite has no profile assignment row"`. `continue` when `surface_id` is falsy.

---

### On the decision

Unchanged and reconfirmed: the scope split is honest and fail-closed as a decision — nightly keeps the full corpus blocking, both release suites stay blocking, corpus size and per-category coverage remain enforced in the release lane, and the carve-out now carries a machine-read record and a `2026-10-31` clock. Finding 1 is the only substantive item: the contract now protects the divergence's *content* and its *mutation*, but its *existence* is still guarded by an artifact that can be deleted alongside it.

One note: `plans/reviews/active/phase-40-algorithm-scope-claude-opus-review-pass-3.md` exists as a zero-byte placeholder; per your instruction I did not write to it.

**NOT APPROVED**
