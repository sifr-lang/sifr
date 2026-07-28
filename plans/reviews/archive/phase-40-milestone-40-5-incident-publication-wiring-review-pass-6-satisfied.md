# Satisfied review result

## Review pass 6 — Phase 40 / milestone 40.5 incident publication production wiring

No files modified. I re-derived every pass‑1→pass‑5 finding against the working tree (`git diff origin/main` + all untracked files) and independently reproduced the validations.

### Pass‑5 findings — all closed

**1. Rollback dispatcher-provenance dead end → fixed, fail-closed before any output.**
`incident_publish.py:75-82` now rejects rollback when the target and affected plans disagree on `site.dispatcher_sha256`:

```python
if (summary["operation"] == "rollback"
    and successor_plan["site"]["dispatcher_sha256"]
    != site_plan["site"]["dispatcher_sha256"]):
    fail("successor_plan_path",
         "rollback target and affected site dispatcher digests disagree")
```

Placement is correct on both axes I checked. It sits **before** `_validated_dispatchers` and before `output_root.mkdir()` (`:83-84`), so no partial output. And it sits before every mutation in the rollback orchestration path: `run_incident_publication.sh:409` (`stage`) precedes the request-asset upload (`:417`), the snapshot upload (`:428`), and the sole `--clobber` (`:438`); lines 364‑406 (retained-release verification, gallery download, VSIX verification) are read-only. The contract case pins that ordering (`incident_publication_workflow_contract.sh:62-77`). For roll-forward the equivalent check already happened pre-mutation at `materialize_stable_publication.py stage` (`:339-344`, before `publish_stable_release.py` at `:345`), matching the stable sibling's ordering — so no analogous hole exists on that branch.

Because digest equality of the four generated dispatchers is exactly what the site re-derives and byte-compares, the three-provenance inconsistency pass 5 described (main's generator / affected's digests / target's `source_commit`) is now proven consistent at stage time rather than discovered after the index is clobbered.

**2. The negative test binds an otherwise valid mismatch and reaches that precise gate.**
`incident_publication_selftest.py:190-231` drifts only `site.dispatcher_sha256.stable` on the target plan, then re-binds `successor.plan_sha256`, `mutation.successor_plan_sha256`, and `mutation.plan_sha256` to the new digest so the summary still passes `validate_incident_prepare_summary` and clears the two earlier gates (`sha256_file(successor_plan_path) != summary[...]` at `:53`, and `expected_site_plan_sha256` at `:71-76`). It asserts the exact diagnostic string and `assert not mismatched_output.exists()`. Executed: 5/5 pass.

**3. Rendered labels canonicalized and cross-checked.** `verify_public_stable_docs.py:21-24` hoists the two literals into `RENDERED_LABELS`; `site_release_workflow_contract.sh:101-111` AST-parses the verifier and asserts `list(rendered_labels) == fixture["stable_documentation"]["rendered_labels"]`. Fixture and verifier can no longer drift silently.

**4. Focused negative schema module present and wired.** `schema_negative_contracts.py` (52 lines) is invoked from `schema_contracts.py:160`; it rejects all three `incident_publication_prepare` conditionals (`activated`+`initial`, `rollback` with an object `release_prepare`, roll-forward with `"none"`) and the `incident_index_mutation_evidence` zero predecessor generation. The mutation fixture is derived from the prepare fixture's own `mutation` (`schema_contracts.py:184-187`), so the two can't diverge.

**5. Passes 1‑4 remain closed.** Spot-verified the ones with production consequence: the `install.json` receipt path matches both the installer (`generate_version_installer.sh:581-590`) and the client (`self_update_runner.rs:157-170`) for `SIFR_INSTALL_DIR=$root/bin`; the recovery fake enforces real `sort -V` ordering with `affected/successor` reversed per operation (`incident_public_recovery_selftest.py:112-115,135-139`); both drift negatives assert `recovery.json` was never written; the demo runs all four adapter suites; `verify_site_workflow_identity.sh` still runs twice per publication (once pre-mutation at `run_incident_publication.sh:309` / `run_stable_publication.sh:228`, once inside `dispatch_stable_site_publication.sh:102`), so `distribution_pipeline.md`'s "before release mutation and again immediately before dispatch" stays true; the drill sign-off is a schema-validated asset (`fixture_release_signoff.py:81`); `alpha` is gone from the dispatcher's accepted set.

### Independent review

Nothing actionable surfaced. What I checked beyond the above:

- **Cross-boundary byte parity.** Both shell→Python canonical-JSON handoffs are genuinely byte-identical, not just structurally equal — I verified `jq -cnS` output for `incident-recovery.json` equals `canonical_json_bytes` and passes `load_json_strict(require_canonical=True)`, and `poll_site_release_run.sh:86-96` emits exactly the four keys `_site_run` requires with `run_id` as an integer.
- **Ordering / failure atomicity.** All three governance uploads (site facts `:540`, release sign-off `:544`, incident sign-off `:546`) follow smoke and both recovery paths. The roll-forward release sign-off is materialized locally at `:511` but not uploaded until `:544`.
- **Write-once / resume.** Snapshot upload uses `allow_existing=false` (`:428`) — safe because `allocate_next_generation` skips burned generations, so a resumed pending attempt gets `N+1` and never collides. `activated` resume skips the snapshot and the clobber entirely, then verifies live bytes equal `proposed_sha256` (`:447`). `--clobber` count asserted == 1.
- **Credentials.** `SITE_TOKEN`/`VSCE_PAT` unset at `:116`; installer runs, smoke, recovery, and the Marketplace publish all scrub `GH_TOKEN`/`SITE_TOKEN`/`VSCE_PAT` at the call site; the prepare workflow is asserted free of `contents: write`, `secrets.`, `gh release upload`, and `vsce publish`.
- **Exact-byte custody.** `verify_retained_stable_release.py` requires full inventory-set equality plus digest equality, plus `tagName`/`targetCommitish`/tag object SHA == `plan.source_commit` and `isDraft`/`isPrerelease` both `false`. Rollback's Marketplace identity chains protected-main plan → qualification index digest → editor report digest → publisher/extension, and `--compiler-version "${successor_version}"` proves the retained VSIX's own `sifrCompilerCompatibility` covers the rollback target.
- **Ancestry.** Incident commit ancestry is enforced in prepare (`governance-source` HEAD) and again in publish against a freshly fetched `refs/remotes/origin/main`, with `rev-parse HEAD == workflow_commit == origin/main` and `workflow_ref == refs/heads/main`.
- **Workflow semantics.** `env.STABLE_MUTATION_OPERATION` / `STABLE_CANDIDATE_OPERATION` / `STABLE_PUBLICATION_OPERATION` / `INCIDENT_OPERATION` are job-level `env` consumed in step `if:` (valid context) and partition the modes correctly; `environment` resolves to `stable-release` for both incident modes; `default: ""` is present on all six dispatch inputs; `VSCE_BIN` is only required for roll-forward, so rollback doesn't need the absent `stable-source` checkout.
- **Schema/runtime parity.** Runtime is equal-or-stricter everywhere I compared (`stable_publication_prepare`'s `incident` required-iff-roll-forward matches `validate_stable_prepare_summary`'s `optional=` gate; `stable_incident_signoff`'s `release_signoff_sha256` conditional and expanded `site_reconciliation` match `incident.py:131-192`).
- **Registration / docs.** 18-schema lint count updated; new suites registered in both the dedicated and combined selection paths; the case script is auto-discovered by glob; `distribution_release` is selected by merge/nightly/release profiles. Architecture, pipeline, and ledger text now match the code, including the site tag/ruleset/digest rotation to `sifr-release-site-stable-facts` / `19899766` / `a9360c82…`.

**Validations reproduced locally:** full distribution area **125 variants / 0 failures**; `--suite full` 67/0; incident publication 5/5 including the precise no-output dispatcher-provenance negative; capability demo exit 0 (filename remains capability-based, no phase/milestone numbering); `bash -n` clean on all 40 `scripts/distribution/*.sh`; `compileall` clean; file-size guardrails **PASS (2936 files, limit 900)**; `git diff --check` clean.

### Non-blocking

1. **The rollback gate lives only in the publish job.** `materialize_incident_prepare` already loads and validates both plans (`incident_prepare.py:112-146`), so the same assertion there would surface the disagreement in the read-only, reviewer-visible prepare summary rather than after protected approval. Both are pre-mutation and fail-closed, so this is ergonomics, not correctness — and pass 5 explicitly asked for the stage-time placement.
2. **Pass‑5 #2's expensive half remains open.** `renderer_sha256` is still pinned only in the fixture and never verified live the way `verify_site_workflow_identity.sh` verifies `workflow_sha256`. Cross-repository; the cheap half (label cross-check) is now done.
3. **The new gate makes rollback across a dispatcher-generator change fail closed** rather than reconciling it — e.g. a target plan whose digests were generated with `--default-channel beta` (pre-GA) can never be a rollback target. That is the right default; worth one comment at `incident_publish.py:75` so a future reader doesn't read it as an oversight.
4. **`plans/reviews/active/phase-40-milestone-40-5-incident-publication-wiring-review-pass-6.md` is 0 bytes.** This is this pass's own artifact slot and I was instructed not to write files, so it needs populating from this report before the PR. Pass 5's artifact was properly archived, closing the pass‑1 #7 / pass‑3 #4 / pass‑4 #4 / pass‑5 #4 recurrence.

VERDICT: SATISFIED
