## Review pass 2 — Phase 40 / milestone_40_5 incident publication production wiring

Scope: full working tree vs `origin/main` plus all untracked files. No files modified. I independently reran `incident_publication_selftest` (4/4), `stable_public_smoke_selftest` (1/1), `incident_publication_workflow_contract.sh`, and the three other publication workflow contracts — all pass.

### Every pass-1 finding is closed

| # | Pass-1 finding | Status |
|---|---|---|
| 1 | Sibling checkouts caused false dirtiness | **Closed.** `incident_prepare.py:576` is now `git status --porcelain --untracked-files=no`, so `stable-source/`, `stable-evidence/`, `incident-evidence/`, `protected-prepare/` at the workspace root no longer trip it, while tracked modifications/deletions still do. Byte-exactness is enforced independently by the new `_require_head_file` (`incident_prepare.py:582-591`) on the request (`:93`), withdrawal evidence (`:94`), and both approved plans (`:565`) — an untracked forgery has no `HEAD` blob and is rejected. Regression-covered by the deliberate untracked sibling at `incident_publication_selftest.py:84`. |
| 2 | No roll-forward end-to-end test | **Closed.** `test_protected_incident_roll_forward_prepare_publish_and_negatives` (`incident_publication_selftest.py:195-329`) drives prepare → stage → release-signoff cross-binding → incident signoff, exercising `_mutation_evidence_from_stable`, the `release_prepare` `$ref` branch, and `--release-signoff`. Negatives: rollback supplying release sign-off (`:162`), missing release sign-off (`:314`), wrong `site_plan` digest (`:265`), site-run mismatch (`:317`), smoke-set drift (`:324`). `stable_prepare_selftest.py:99` adds the stable-side roll-forward prepare. |
| 3 | Drill/production surface assertions deleted | **Closed.** `test_protected_production_adapter_surface` (`incident_publication_selftest.py:387-427`) restores the `incident_fixture.py` import ban and the `run_incident_fixture.py` `gh release`/`vsce publish`/`repository_dispatch` bans, plus the drill-workflow assertions. The one dropped assertion (`uses: ./…drill.yml`) is still covered by `protected_release_drill_workflow_contract.sh:38`. |
| 4 | jq indexing the string `none` | **Closed.** `release-publication-prepare.yml:573,574,578` use `(.release_prepare \| objects)`. Verified on jq 1.7.1: rollback yields `""` with rc 0, no error. This is load-bearing — `source_commit` from `.release_prepare.source.commit` is what drives the `stable-source` checkout at `release-publication.yml:155` for roll-forward. |
| 5 | Cap met by whitespace, not decomposition | **Closed.** `release-publication.yml` 830 lines via real extraction (`validate_preview_publication_inputs.sh`, `dispatch_stable_site_publication.sh`); `stable_publish_selftest.py` 775 via extraction of `stable_public_smoke_selftest.py`. `publish` remains the single mutation job (`contents: write` count == 1, asserted). |
| 6 | Demo requirement unmet | **Mostly closed** — see finding 2 below. Approval evidence (`demo:49`), credential-free drills (`:58`), Marketplace/incident adapters (`:70`), roll-forward dispatch shape (`:80`) all present and non-mutating. |
| 7 | Site rationale + facts-driven docs smoke | **Closed.** Rationale at `incident_prepare.py:162-164` and `run_incident_publication.sh:173-175`; facts-driven docs assertion at `run_stable_public_smoke.sh:120-153` reading `stable_version`/`withdrawals[].version`/`incident_id`, whose key names match `generate_site_release_facts` (`release_plan.py:406-419`). |
| 8 | Hardcoded rollback Marketplace identity | **Closed.** `run_incident_publication.sh:369-410` derives publisher/extension from the retained `editor-qualification-report` asset (inventory- and digest-verified by `verify_retained_stable_release.py:62-74`), charset-validates them, cross-checks `package_version`/`vsix_sha256` against the affected plan, and keeps `--compiler-version "${successor_version}"`. |
| 9 | Revalidation error handling | **Closed.** `revalidate_incident_publication.py:79,106-110` threads the real parser, matching `revalidate_stable_publication.py:98-132`. |
| 10 | Mutation schema not shared | **Closed.** `incident_index_mutation_evidence.schema.json` is a standalone file `$ref`ed at `incident_publication_prepare.schema.json:53`, fixture-registered (`schema_contracts.py:182`), CLI-validatable (`release_governance.py:79`), lint count 16 → 18 (`selftest.py:87`). Runtime `validate_incident_mutation_evidence` is exported independently. |

---

### Blocker

**1. Architecture and pipeline docs now state the opposite of what the code does; the execution ledger has no entry for this wave.**
- `internal_docs/distribution_pipeline.md:579-581`: "Incident rollback and roll-forward production adapters remain gated until their later protected-publication slice." This diff *is* that slice.
- `internal_docs/architecture.md:1452-1453`: "Stable and incident production adapters remain deliberately absent until their protected-publication slices." False for stable since PR #3045 and now false for incident.
- `plans/issues/active/phase-40-stable-channel-ga-execution.md` gains no wave paragraph and no review-round entries, and its milestone_40_5 checkboxes (`:721-725`) are untouched.

AGENTS.md requires architecture/phase/issue docs to be updated per item, and the milestone Validation Contract requires each milestone to record commands, artifacts, review sign-off, PR link, and checklist status. The immediately preceding comparable commit (`a5c9a2ce8`) updated `internal_docs/distribution_pipeline.md`, `plans/issues/active/…`, `plans/releases/README.md`, and the gate inventory in the same commit. Leaving a now-false "adapters remain gated" sentence in the durable architecture reference is a governance defect, not a cosmetic one — an operator reading it would conclude rollback is not wired.

---

### Non-blocking (actionable)

**2. The demo no longer records the public stable install/update flow it announces.** `demos/stable_release_governance_demo.sh:69` prints "Stable install/update, Marketplace, and incident publication adapters" and then runs only `stable_publish_selftest` and `incident_publication_selftest` (`:72-76`). The dispatcher-driven install + `sifr self update --dry-run` flow moved to `stable_public_smoke_selftest.py` when finding 5 was fixed, and the demo does not invoke it. milestone_40_5's demo requirement names "the public stable install/update flow" explicitly. Add `python3 -m …governance.stable_public_smoke_selftest` to the same subshell.

**3. The withdrawal-naming branch of the new docs smoke is untested and depends on an unverified external rendering contract.** `run_stable_public_smoke.sh:140-141` requires every `withdrawals[]` `version` **and** `incident_id` to appear in `https://sifr.sh/releases/stable`. The only test (`stable_public_smoke_selftest.py:73-77`) serves a page for an empty `withdrawals` list, and the in-repo source block (`docs/releases/stable.mdx:14-17`, checked against a fixture by `stable_editor_qualification_contract.sh:11-15`) is statically rendered as "Withdrawn stable versions: none." Nothing in this repo proves the site re-renders that block from the dispatched generation. If it does not, the first real rollback fails deterministically at `run_stable_public_smoke.sh:150` — 180 s after the index has already been clobbered (`run_incident_publication.sh:442`) and the site deployed. Recoverable only via protected resume, but it is a post-mutation failure on the primary incident path. Add a positive test with a non-empty `withdrawals` list, and pin the site's facts-rendering contract the way `SITE_WORKFLOW_SHA256` pins the workflow.

**4. The governance worktree-drift guard has no negative test.** `_require_head_file` (`incident_prepare.py:582-591`) is the security half of the finding-1 fix, and `_require_clean_checkout` was deliberately loosened to `-uno` (`:576`). The suite covers the untracked-sibling positive but never rejects (a) a governance-root plan/request whose worktree bytes differ from `HEAD`, or (b) a dirty tracked file. Both are one-line fixture mutations in `test_protected_rollback_prepare_publish_and_resume`.

**5. The incident orchestrator's fail-closed guards are never executed.** `run_incident_publication.sh` — 544 lines of production mutation — is only text-asserted (`incident_publication_workflow_contract.sh`). Its guards at `:81-136` (`workflow_ref = refs/heads/main`, mutation-from-protected-main-HEAD, merge-base ancestry for incident/candidate/candidate-source, rollback rejecting successor inputs at `:108`) have no executed coverage, while the stable sibling does (`stable_publish_selftest.py` `test_orchestrator_rejects_unprotected_ref` / `test_orchestrator_rejects_unmerged_candidate`). The two newly extracted scripts `dispatch_stable_site_publication.sh` and `validate_preview_publication_inputs.sh` are likewise text-only. Reuse the existing fake-`git`/fake-`gh` harness for at least the unprotected-ref and unmerged-incident negatives.

**6. Dispatcher-digest drift negative still missing.** `incident_publish.py:251` (`digests do not match the approved successor plan`) — flagged in pass 1, not added.

---

### Minor

**7.** `run_incident_publication.sh:369-372` — `affected_vsix` is assigned and never used; dead code in a protected mutation path.

**8.** `validate_preview_publication_inputs.sh:50-63` collapses previously distinct diagnostics into a generic `usage()` exit 2. `"version and preview channel disagree"` and `"site_base_commit must be an exact commit"` (old `release-publication.yml:237,252`) are gone; an operator now gets only a usage block. The `BASH_REMATCH[1]` channel/version cross-check itself is correct — I verified bash evaluates it lazily against the third `=~`, so `beta` + `1.2.3-alpha.4` is rejected.

**9.** `run_incident_publication.sh:519` invokes `run_incident_public_recovery.sh` without call-site token scrubbing, unlike `run_stable_public_smoke.sh:496`. The recovery script scrubs inside `run_working`/`run_out_of_band`, but `release_governance.py validate` (`run_incident_public_recovery.sh:78`) runs with the ambient `contents: write` `GH_TOKEN`.

**10.** `stable_publish_selftest.py` still carries the finding-5 cosmetic blank-line deletions inside the embedded fake-`gh` script (diff hunk at `~:631-643`) although the file is now 775 lines. Revert for readability.

**11.** `incident_prepare.py:576` uses `-uno` while `stable_prepare.py:721` uses `-uall`. The asymmetry is correct (workspace root vs nested checkout) but undocumented; one comment prevents a future "consistency" regression that would re-break production.

**12.** `release-publication.yml` repeats the identical four-way mode-exclusion `if:` at lines 142, 175, 197, 246, 266, 275, 371, 488, 571, 622, 641, 676. Residual only — the cap is now met by genuine decomposition.

---

### Verified clean

- Single `publish` job, `contents: write` count == 1, `sifr-release-index` concurrency, `stable-release` environment for all four stable modes, drills on a separate group/environment.
- Mutation ordering locked: revalidate → approvers → retained-release verification → stage → write-once request asset → revalidate → snapshot (no `--clobber`) → live `cmp` → single `--clobber` → activated-digest check → site dispatch/poll → public smoke → recovery → sign-offs. `--clobber` count == 1.
- Prepare/publish env expressions: `VERSION`/`SITE_BASE_COMMIT` include rollback, `SOURCE_COMMIT` correctly excludes it; `setup-node`/`npm ci`/`VSCE_BIN` are gated off for rollback and `VSCE_BIN` is required only for roll-forward (`run_incident_publication.sh:117-122`).
- Exact smoke inventories: `run_stable_public_smoke.sh` emits exactly the 7 `stable_publish.SMOKE_FILES`; `${work}/incident-smoke` adds `incident-recovery.json` for the 8 `incident_publish.SMOKE_FILES`; both consumers assert set equality, so the shared `cp` at `:518` cannot drift either signoff.
- Schema/runtime parity: `stable_publication_prepare.schema.json` `then/else` on `incident` matches `optional={"incident"}` + `require_exact_keys`; the loose `proposed_index: {"type":"object"}` matches existing precedent with the byte/generation checks enforced at runtime; `release_signoff_sha256` and the expanded `site_reconciliation` object agree between `incident.py:131-190` and `stable_incident_signoff.schema.json:25-79`.
- Marketplace coverage is not weakened for `normal`: `editor_qualification.py:74-79` already requires the compatibility range to contain a non-`none` `rollback_version` at plan level; the new `--compiler-version` proves it against the live Gallery bytes on the rollback path.
- Retained-release verification binds tag commit, target commitish, draft/prerelease, plan+qualification digests, and the exact asset inventory, with a drift negative (`incident_publication_selftest.py:369-384`).
- Resume/race: burned generations skipped via `allocate_next_generation`; attempt-scoped sign-off filenames; `activated` path skips reservation and re-verifies the live digest; `upload_or_verify_governance` requires explicit resume for pre-existing assets.

VERDICT: CHANGES_REQUIRED
