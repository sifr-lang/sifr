## Wave 10 Milestone Review — Pass 2, PR #3093 @ `ff04485b6` vs `origin/main` `9c99ef43b1`

**I made no modifications.** Read-only inspection only (git diff/history, committed artifacts, existing `target/**` evidence, static reads of the policy checkers). No build, no test, no corpus sweep, no profile run.

### Scope and diff verified
`9c99ef43b1...ff04485b6` = 12 files, +185/−71: one new demo (`demos/algorithmic_collections_and_recursive_models/main.sifr`, 79 lines), 6 plan/tracker docs, the new pass-1 review artifact, `verification/README.md`, `profile_policy.md`, both coverage matrices, `release.json`. `ff04485b6` itself is documentation-only (4 files) on top of `53fd964ea` — the approved non-documentation tree from pass 1 is untouched. Submodule gitlinks identical to main (`leetcode` `9d71595347`, `ruff` `e024f2a487`); `git submodule status` shows no pointer change. No baseline, suppression, exclusion, fixture annotation, or fallback anywhere in the diff.

### Configuration claims — confirmed
- `verification/profiles/release.json:181-190`: `algorithmic_compatibility` = `["leetcode-full","taxonomy-smoke"]` with `["external-corpus","long-running"]`, byte-identical to `nightly.json`; both are within each profile's declared `resource_policy.classes`. `merge.json` (`representative-subset`/`default-local`) and `create-pr.json` (`profile-manifest`/`default-local`) are unchanged — no weakening.
- `compiler_surface_matrix.json:350-359`: `release_suite`/`release_divergence_record`/`release_divergence_expiry` removed together (a partial removal trips `coverage_matrix.py:230`), reproduction command repointed to `leetcode-full`. All three `GENC-NAN` rows (`:123-127`, `:142-146`, `:161-165`) and their `2026-10-31` expiry survive intact.
- `profile_assignment_matrix.json:65`: release row now equals nightly — required once no `release_suite` exists (`profile_assignment_matrix.py:233-239`), and consistent with `coverage_matrix.py:396`.
- No stale "release runs the representative subset" claim remains in `verification/README.md`, `profile_policy.md`, `docs/`, or `internal_docs/`.
- Pass 1's claim that the committed `plans/releases/candidates/0.1.0` evidence is not re-validated against the live profile digest **holds**: `evidence_custody.py:219-222` calls `validate_release_profile_report` without `source_root`/`expected_profile_sha256`, and `stable_prepare.py:752-757` digests `release.json` from the pinned expected-commit checkout, not the current tree. The now-stale `manifest_sha256` `fa3d95c0…` binds only historical `c9d611fb7c7c` evidence.

### Evidence claims — reproduced
- `target/algorithmic-native-closeout.4ZmiZw/summary.txt`: `fixtures=411 records=411 pass=411 check_fail=0 build_fail=0 run_fail=0`, written 12:30–13:03, i.e. after Wave 9 merge `eee55b9f94` (11:29 +0200 = 12:29 local) and before PR #3092 landed — matching the new "pre-integration compiler at `eee55b9f94`" wording.
- Canonical lane is genuinely current-head: `leetcode-full-taxonomy.json` (`total_cases 411`, `failing_cases 0`), delta `PASS 411 -> 411 (+0)`, and `algorithmic-compatibility-results.json` all stamped 14:23, after the integration merge `53fd964ea0` (14:11) and before `ff04485b6` (14:24).
- The integrated-head native rerun is in fact live (`target/algorithmic-native-closeout.ljNFEM`, started 14:23, `status/` still filling), consistent with "remains in progress".

### Pass-1 finding resolution
1. **Partially resolved.** The Wave 10 ledger row (`…preexisting-failures.md:337`) now correctly attributes the native audit to `eee55b9f94`, states the integrated-head rerun is pending, and attributes 411/411 canonical to `53fd964ea0`; the native-closeout acceptance box (`:376`) is correctly left `[ ]`, while `:363` is legitimately `[x]` on the new post-merge canonical run. See finding 1 for the residue.
2. **Resolved.** `40_stable_channel_ga_promotion_and_release_governance.md:932-934`, `phase-40-stable-channel-ga-execution.md:90-93` and `:119-122` now separate "has remediated its 20 preserved failures" from "closeout validation and review remain in progress". Ledger entry `:230-234` correctly marks the prior release-profile run "historical". Phase index `:51` and roadmap `:86` status both read "closeout in progress".

Ledger link `→ …wave-10-claude-opus-review-pass-1.md` resolves and its summary of pass 1 ("found only … pre-integration evidence provenance and Phase 40's closeout wording were overstated") is accurate. Remaining gates are disclosed as pending, not claimed.

---

## Actionable findings (ranked)

**1 — Non-blocking, medium. The native-audit overclaim pass 1 flagged survives in the two summary surfaces.**
`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:13-15` — "all 411 pinned fixtures **now** pass both the canonical suite and a complete native build/run audit" — and `plans/roadmap.md:86` — "All 411 pinned fixtures pass the canonical suite and native build/run audit" — carry no provenance qualifier. Both directly contradict the same file's Wave 10 row at `:337` ("the required integrated-head native rerun remains in progress") and the deliberately unchecked criterion at `:376` ("All 411 pinned corpus fixtures pass a complete native build/run audit **at closeout**"). A reader of the Status paragraph or the roadmap concludes the closeout native audit is done at the current head; only the ledger row says otherwise. This is the substance of pass-1 finding 1, corrected in one place out of three.
*Fix:* qualify both to the audit's base commit, e.g. "…and a complete native build/run audit on the pre-integration compiler `eee55b9f94`, with the integrated-head rerun in progress".

**2 — Non-blocking, low. `verification/policy/profile_policy.md:145` calls the inventory "completed" while closeout is in progress.**
"The **completed** failure inventory and remediation evidence are tracked from the phase index" reads as a closed record in the authoritative policy document, whereas the indexed target (`plans/phases/index.md:51`) says "closeout in progress" with four open acceptance criteria. The normative content of the section (both broad profiles blocking, nothing removed/re-baselined/reclassified) is accurate; only this adjective overstates.
*Fix:* "The failure inventory and remediation evidence are tracked from the phase index."

**3 — Non-blocking, low. The committed pass-1 artifact contains two wrong line citations and a stale self-referential bullet.**
`plans/reviews/active/…-wave-10-claude-opus-review-pass-1.md:32` cites "lines 368/376" for the two acceptance boxes; at the reviewed state (`53fd964ea`) they were lines **363** and 376. Line `:36` cites `phase-40-stable-channel-ga-execution.md:120` for "The completed follow-up now keeps…"; the actual line was **125** (`:91` and `:932` are correct, and every quoted string is verbatim-accurate). Separately, `:44` states that this very artifact "exists untracked and **empty** (0 lines); fill or delete before it is committed" — false of the committed file, and confusing in a durable record.
*Fix:* correct the two line numbers and drop or rephrase the housekeeping bullet as resolved.

### Observations (not findings)
- The demo exercises waves 1–5, 7, and 8 surfaces; wave 9's nested captured-container fix has no demo representation. Outside the stated criterion, as pass 1 noted.
- Local working tree carries untracked junk inside both submodules (`.DS_Store`, `src/__pycache__/`, `src/sifr_output/`) and an untracked pass-2 artifact path. Pre-existing local state, absent from the PR; do not let it enter a commit, since publication-path checks require clean checkouts.

## Verdict

**NOT SATISFIED** — three actionable findings remain (all documentation-only; finding 1 is a partially unresolved carryover of pass-1 finding 1). The implementation, policy, matrices, profile change, and scope discipline are correct and independently reproduced against current artifacts.

I made no modifications to any file.
