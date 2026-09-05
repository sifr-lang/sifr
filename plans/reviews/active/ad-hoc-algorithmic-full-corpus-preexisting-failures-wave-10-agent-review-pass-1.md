# Wave 10 Milestone Review — PR #3093 (read-only)

Working tree unchanged; no files were modified.

## Methodology

- Full `origin/main...HEAD` diff (11 files, 135+/72-), commit graph, and submodule pointer diff.
- Live policy/checker code read and executed (read-only, lightweight): `coverage_matrix.py` (`validate_release_divergence`, `validate_release_surface_profile_policy`), `profile_assignment_matrix.py`, `coverage_matrix_readiness_self_test.py`, `verification_taxonomy.py`, `check_file_size_guardrails.py`, `check_submodule_ownership.py`, `sifr_verify profiles plan --profile release` / `profiles check`.
- Artifact inspection: `target/algorithmic-native-closeout.4ZmiZw` (summary + all 411 result rows + fixture set), `target/verification/areas/algorithmic-compatibility-results.json`, `target/verification/areas/coverage-matrix-results.json`, `target/validation_lane_reports/{nightly,create-pr}.latest.*`, `target/algorithmic-capability-demo-post-main/sifr_output/src/main.rs`.
- Capability demo verified at current head: `target/debug/sifr check demos/algorithmic_collections_and_recursive_models/main.sifr` → "no errors found" (binary newer than every `crates/**/*.rs`); prebuilt post-merge native binary re-run → exit 0.
- Consumer sweep for stale release-scope claims (docs, workflows, `plans/releases/candidates/0.1.0`, distribution-governance digest checks), corpus-baseline/annotation inspection, and PR #3093 body via `gh`.

## Verification results (1–7)

**1. Capability demo — verified.** `demos/algorithmic_collections_and_recursive_models/main.sifr` is capability-named, phase-free (`verification_taxonomy.py` passes), and natively exercises every listed surface. Generated Rust confirms each is real, not incidental: nested `Vec<Vec<i64>>` `sort()`/`==` (wave 1–2), `__sifr_empty_list_literal` specialization for `[[1], []]` (wave 2), `seen = {}` → `HashMap<i64,i64>` from later use sites (wave 3), `entry().or_insert()` augassign counting and order-independent `defaultdict(set)` inference (waves 4–5), `let Some(mut node) … node.next.take().map(|v| *v)` for the owned recursive option (wave 7), and exactly one box layer in `ListNode::new(10, (node).clone().map(|v| Box::new(v)))` for borrowed-option constructor forwarding (wave 8). Demo check/build/run evidence is at the *integrated* head (artifact dir timestamped after the `origin/main` merge). Observation only: wave 9's nested captured-container surface is not represented in the demo — outside the enumerated criterion.

**2. release.json — verified exact.** `release` selects exactly `["leetcode-full","taxonomy-smoke"]` under `["external-corpus","long-running"]`, identical to nightly and within the profile's declared top-level resource classes; merge keeps `representative-subset`/`default-local`; create-pr keeps `profile-manifest`. Emitted plan (`profiles plan --profile release`) matches byte-for-byte.

**3. Matrices — verified.** Only the ALG-CORPUS divergence was removed (`release_suite`, `release_divergence_record`, `release_divergence_expiry` removed together — a partial removal would have tripped the `elif` guard at `coverage_matrix.py:230`), reproduction command repointed to `leetcode-full`, and `profile_assignment_matrix` release row now equals nightly, which is required now that no `release_suite` exists (`validate_release_divergence_declaration`). All three GENC-NAN rows remain intact with entries and 2026-10-31 expiry. Live checks pass: coverage matrix (13 guarantees / 34 surfaces), profile assignment (19 rows), readiness self-tests (24 cases).

**4/5. Docs and ledger — one inaccuracy (finding 2), otherwise sound.** Historical evidence is preserved and marked as such ("historical clean exact-source release profile", "the pinned corpus *was* not a Phase 40 prerequisite"); the 0.1.0 candidate evidence under `plans/releases/` is correctly untouched and is not re-validated against the live profile digest by any suite (self-tests recompute from synthetic fixture sources), so the changed release manifest breaks nothing. Roadmap/phase index/issue all say "closeout in progress"; four acceptance boxes correctly remain unchecked (nightly lane, release restore + release lane, create-PR/merge gates, reviews/merges).

**6. Scope — clean.** No baseline, suppression, exclusion, fixture annotation, resource downgrade, or fallback. Corpus submodule gitlink is unchanged (`9d71595347`); `leetcode_full_baseline_*.json` are the untouched all-pass taxonomy-generator inputs, not failure baselines; submodule-ownership and file-size guardrails pass. Nothing unrelated entered the diff.

**7. Evidence consistency — reproduced, with one provenance gap (finding 1).** Native audit: 411 fixtures, 411 `PASS`, 0 check/build/run failures, covering all 411 `corpora/leetcode/src/*.sifr` (including `0022_generate_parentheses`, which proves the binary carried the wave-9 fix). Canonical lane: `leetcode-full`, 411 variants, 0 blocking/non-blocking failures. Coverage readiness 5/5 pass — and that artifact is timestamped *after* the `origin/main` merge, so it is current-head evidence.

**Explicitly still pending (not treated as satisfied):** canonical `leetcode-full` and the 411-fixture native audit have **not** been rerun on the current integrated head (both predate the PR-#3092 merge, which changed `sifr_hir` `method_receiver_places` footprint analysis — a compiler-behavior change over a surface the corpus exercises heavily). The complete nightly, release, create-pr, and merge gates are unsatisfied; the only nightly attempt on this tree (14:00) **failed** at `performance_budget_checks` (three host-sensitive median regressions, `waiver_status=no_waiver`) before reaching the algorithmic lane. The Wave 10 review is in progress and no review artifact is committed.

## Findings (ranked)

**1 — Non-blocking, actionable (medium): the issue ledger and two acceptance boxes present pre-integration evidence as closeout-complete.**
`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:337` describes "the merged compiler's complete isolated native audit … under `target/algorithmic-native-closeout.4ZmiZw`" and "the independent canonical `leetcode-full` lane passed 411/411" with no provenance qualifier, and lines 368/376 check `[x] Every listed fixture passes the canonical full-corpus algorithmic suite` and `[x] All 411 pinned corpus fixtures pass a complete native build/run audit **at closeout**`. Both runs finished at 12:30–13:14 local, before PR #3092 merged into `main` (13:04 +02:00 = 14:04 local) and before this branch integrated it (14:11). PR #3093's own body contradicts the ledger, listing "rerun canonical and native 411-fixture audits on the current integrated head" under *Remaining before ready* and labelling the audit "pre-integration".
*Fix:* in the Wave 10 row, state the audit/canonical evidence base commit (`eee55b9f94`, pre-#3092) and that both must be rerun on the integrated head; and either uncheck the two boxes until the reruns land or annotate them as satisfied only at that base.

**2 — Non-blocking, actionable (low): Phase 40 documents call the follow-up "completed"/"remediated" while its own file says closeout is in progress.**
`plans/phases/40_stable_channel_ga_promotion_and_release_governance.md:932` ("The completed `ALG-CORPUS` remediation restores…"), `plans/issues/active/phase-40-stable-channel-ga-execution.md:91` ("is remediated") and `:120` ("The completed follow-up now keeps…") read as a closed issue, whereas the issue status line, the phase index, and the roadmap all say "closeout in progress" with four open acceptance criteria and no release-lane run.
*Fix:* distinguish the two states, e.g. "the follow-up's 20 preserved failures are remediated; its closeout (nightly/release/merge gates and final review) is in progress", and keep the configuration statement ("release now blocks on the full corpus plus taxonomy self-test") separate from the completion claim.

## Stale active references (archive-time, not now)

- Issue-path links that break on archive: `plans/roadmap.md:86`, `plans/phases/index.md:51`, `plans/phases/40_…md:931`, `plans/issues/active/phase-40-stable-channel-ga-execution.md:{93,121}`, plus the issue's own `../../reviews/active/…` links for every wave (waves 1–9 review artifacts must archive with it).
- `plans/phases/index.md` ALG-CORPUS row status/target must move to the archived path; this is now safe because no surface row references the `ALG-CORPUS` record any more — but the GENC-NAN row must stay indexed and resolvable or `validate_release_divergence` fails closed for all three codegen rows.
- `verification/policy/profile_policy.md:143` ("tracked from the phase index") stays valid only while that index row resolves to the archived file.
- Housekeeping: `plans/reviews/active/…-wave-10-agent-review-pass-1.md` exists untracked and **empty** (0 lines); fill or delete before it is committed.

## Verdict

**NOT SATISFIED** — two actionable, non-blocking findings remain (ledger/acceptance provenance overclaim; Phase 40 "completed" wording). The implementation itself — demo, release.json, both matrices, policy, and scope discipline — is correct and independently reproduced, and the outstanding gates are legitimately disclosed as pending in the PR body. Correcting the two documentation statements above (no code or profile change required) would clear this pass for the current milestone state.
