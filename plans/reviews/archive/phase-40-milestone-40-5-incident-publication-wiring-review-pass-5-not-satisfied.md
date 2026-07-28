# Review result

I re-derived every pass-4 finding against the working tree and independently reproduced the validations.

## Verified closed (pass-4)

- **Major 1** — `incident_public_recovery_selftest.py:112-115` now uses `affected=0.1.1/successor=0.1.0` for rollback and `0.1.0/0.1.1` for roll-forward, and the fake client does a real `sort -V` ordering check (`:135-139`) that rejects any non-forced downgrade regardless of operation.
- **Major 2** — `test_recovery_rejects_binary_and_receipt_drift` (`:71-104`) drifts `BINARY_VERSION` and `RECEIPT_VERSION` to `0.1.2` while `channel` stays `stable`, so the receipt still passes `release_governance.py validate` and fails the script's own `jq` at `run_incident_public_recovery.sh:82-88`; the binary case fails `:75-78`. Both assert `recovery.json` was never written.
- **Major 3** — `demos/stable_release_governance_demo.sh` now runs `governance.incident_public_recovery_selftest` in the same subshell.
- **Minors 4,5,7,8,9,10** — pass-4 artifact populated (8010 B); `default: ""` restored on all six dispatch inputs (`release-publication.yml:16-28`); `require_incident_id` at `stable_prepare.py:589`; dispatcher usage/regex both `beta|stable` (`dispatch_stable_site_publication.sh:12,87`); drill sign-off now a validated asset via `fixture_release_signoff.py`; `STABLE_PUBLICATION_OPERATION` at `release-publication.yml:131,794`.

**Validation reproduced:** `distribution_release` 125 variants / 0 failures; capability demo exit 0; file-size guardrail PASS (2935 files, limit 900); `git diff --check` clean. I also confirmed the new `incident_publication_prepare` fixture passes the *runtime* validator, and that both new schema `if/then` conditionals actually reject (`activated+initial`, `rollback` with an object `release_prepare`, roll-forward with `"none"`).

## Actionable

**1. Rollback binds the dispatcher digests to the affected plan but dispatches the *target's* source commit, so a generator change between two adjacent stable releases strands the site permanently after the index is already clobbered.**

For rollback, `run_incident_publication.sh:176` sets `site_plan="${affected_plan}"`, and `incident_publish.py:75` → `_validated_dispatchers` (`:241-253`) requires the locally generated dispatchers to equal `affected_plan["site"]["dispatcher_sha256"]`. Those dispatchers are generated from the **protected-main** copy of the script (`run_incident_publication.sh:335-336`, relative path, CWD = workspace-root checkout). But the same digests are then dispatched alongside `sifr_source_commit` = the **rollback target's** commit (`:454` `jq -er '.source_commit' "${successor_plan}"`, consumed at `:461` and `:485`). Per the pinned contract, the site regenerates the four dispatchers from `scripts/distribution/generate_dispatchers.sh` — a Sifr-repo path it can only reach through that dispatched Sifr commit (`site_release_contract.json:19-31`; `distribution_pipeline.md:476-480`). Nothing anywhere requires `affected_plan.site.dispatcher_sha256 == successor_plan.site.dispatcher_sha256`: `_validate_rollback_plans` (`incident_planner.py:135-160`) compares only `rollback_target`, `expected_stable_predecessor`, version, and `desired_release`, and `stage_incident_publication` never reads `successor_plan["site"]`.

Failure scenario: `generate_dispatchers.sh` is byte-identical between the affected release and main, but changed between the target and the affected release. Staging passes. `channels.json` is clobbered at `:438`. The site run then regenerates at the target's commit, produces the target's dispatcher bytes, and rejects the caller's digests. `dispatch_stable_site_publication.sh` fails at `:479`. Because the digests are re-derived identically on every attempt, **resume fails deterministically** — the governed index is rolled back while sifr.sh keeps serving the withdrawn release's dispatchers and stale `/releases/stable/`, with no path forward through this workflow. There is no pre-mutation check for this and no test covers it (the executed selftests never exercise the site side).

Fix: assert plan agreement in `stage_incident_publication`, before any mutation — for `operation == "rollback"`, require `successor_plan["site"]["dispatcher_sha256"] == site_plan["site"]["dispatcher_sha256"]` (the same block already loads both plans at `incident_publish.py:56-61`) and fail with a diagnostic naming the two plans. That converts a post-clobber dead end into a fail-closed rejection at stage time.

*Caveat, stated plainly:* the site-side regeneration source is established from the pinned cross-repository contract and `distribution_pipeline.md`, not from reading `sifr-website`. The caller-side inconsistency — three different provenances (main's generator, the affected plan's digests, the target's source commit) for one digest set — is fully verifiable in this repo regardless.

## Non-blocking

**2.** Pass-4 #6 residual, third wave. `site_release_contract.json:36-40` now pins `renderer_sha256` and `rendered_labels`, but both appear *only* in the fixture and in the contract case's assertion of the fixture against itself (`site_release_workflow_contract.sh:59-65`). Nothing verifies the renderer digest live the way `verify_site_workflow_identity.sh` verifies `workflow_sha256`, and nothing cross-checks `rendered_labels` against the literals hard-coded at `verify_public_stable_docs.py:29-31`. An external renderer edit still fails the docs smoke 180 s after the clobber, and the fixture can silently drift from the verifier. The cheap half — asserting the two label lists are equal in the contract case — is a three-line addition.

**3.** Neither new schema has a negative fixture in `validate_schema_contracts`, unlike `stable_publication_prepare`, `stable_index_mutation_evidence`, `stable_incident_signoff`, the drill, and the bootstrap schemas (`schema_contracts.py:50-165`). I probed the three conditionals in `incident_publication_prepare.schema.json:70-91` directly and all reject correctly, so this is an untested-but-correct gap, not a defect.

**4.** `plans/reviews/active/phase-40-milestone-40-5-incident-publication-wiring-review-pass-5.md` is 0 bytes — fourth recurrence (pass-1 #7, pass-3 #4, pass-4 #4). Populate before the PR.

VERDICT: CHANGES_REQUIRED
