## Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 1)

I read the phase and issue docs, the full `git diff`, all nine untracked files, and the surrounding governance/runner/contract surfaces. The core design is sound: the epoch is genuinely opaque (no v1 parser, fixture, migration, or fallback anywhere — the `forbidden` sweep in `schema_epoch_bootstrap_workflow_contract.sh:104-110` plus `build_preview_epoch` accepting only a digest+size holds up), `prepare` is truly read-only and unprotected, the digest chain prepare→publish→live is checked at three points, `fetch_schema_bootstrap_alpha.sh` reproduces the alpha release record byte-for-byte rather than trusting it, and the public smoke exercises real endpoints. The `.sha256` files contain a bare digest (`build_release_artifacts.sh:275`), so the `tr -d '[:space:]'` comparisons are correct; `site_default_channel` resolves to `beta` under `preview`, so the smoke's `${version}:beta:false` expectation is consistent.

The following are actionable.

### 1. Prepare→publish artifact handoff breaks on any job re-run — High

`release-publication-prepare.yml:210` names the artifact `publication-prepare-${{ github.run_id }}-${{ github.run_attempt }}`; `release-publication.yml:169-172` downloads that exact name. "Re-run failed jobs" increments `run_attempt` for the run but does **not** re-execute the already-succeeded `prepare` job (its outputs are reused via `needs.prepare.outputs.summary_sha256`). So on attempt 2 the publish job looks for `publication-prepare-<id>-2`, which was never uploaded, and dies at `download-artifact` before emitting any governed diagnostic. Every transient failure in the protected job — approvals API hiccup, site poll timeout, upload flake — becomes unrecoverable-by-rerun. Drop `run_attempt` from the name (`overwrite: false` already gives write-once semantics within the run).

### 2. Two distinct approvers hard-fail the protected mutation — Medium-High

`verification/areas/distribution_release/governance/schema_bootstrap.py:121-122`:

```python
if len(approved) != 1:
    fail("approval history", "contains ambiguous distinct approvers")
```

The frozen policy is "at least one non-initiating `release/distribution` reviewer" and "initial and resume attempts each require a fresh approval." The `/approvals` endpoint returns the whole run's review history with no attempt discriminator, so a resume approved by a second reviewer yields two distinct logins → permanent hard fail. Same outcome if the `stable-release` environment is configured with two required reviewers. This should accept ≥1 distinct non-initiator and record all of them, not reject.

### 3. Generation-1 evidence does not bind the alpha stage or the prepare summary — Medium

`schema_bootstrap.py:223-292` records `alpha.{version,source_commit,release_record_sha256,published_assets}` but not the alpha stage's own evidence-asset digest, run id, or approver; and no evidence field carries `prepare_summary_sha256`. Those two correlations live only in `protected-prepare/summary.json`, which exists solely as a 30-day workflow artifact (`release-publication-prepare.yml:213`). The phase doc (`plans/phases/40_…md:952-955`) requires "Its prepare summary, protected approval, immutable snapshot, exact asset digests, and public smoke are retained as publication evidence." After retention expiry, nothing durable ties generation 1 back to the read-only prepare or to who approved the alpha stage. Add `prepare_summary_sha256` and `alpha_evidence_sha256` (plus the alpha run id/approver) to the `preview-index` payload and schema.

### 4. New JSON Schema is materially weaker than the validator, with no negative coverage — Medium

Given this repo's history (40.0 passes 4–6 were entirely schema/validator-parity findings):

- `schema_epoch_bootstrap_evidence.schema.json` `public_smoke` has `minItems`/`maxItems: 4` but no uniqueness constraint, so four identical `"dispatcher-default"` entries validate; `schema_bootstrap.py:185-187` rejects them.
- `$defs/assets` only requires `minProperties: 9` with arbitrary keys; `schema_bootstrap.py:419-424` requires exactly `expected_asset_names(version)`. The schema accepts an asset map for the wrong version.
- `schema_contracts.py:27-65` registers only a positive fixture for the new schema, unlike the qualification-index and incident-signoff schemas which each carry explicit negatives.

### 5. `materialize_bootstrap_evidence` — the actual producer — is untested — Medium

`schema_bootstrap_selftest.py` covers `build_preview_epoch`, `validate_bootstrap_evidence`, and one happy/one self-approval case of `resolve_distinct_approver`. Nothing exercises the producer: `_require_exact_bootstrap_membership` (`schema_bootstrap.py:298-338`, index/record disagreement), the exact-asset-set check at `:367-370`, a missing smoke file, `write_canonical_json(refuse_existing=True)`, or the preview-index required-argument guard at `:243-255`. Also untested in `resolve_distinct_approver`: empty history, non-`approved` state, wrong environment name, and — notably — the `len(approved) != 1` branch that carries finding 2.

### 6. Public-smoke override scrub is too late and covers only the last check — Medium-Low

`run_schema_bootstrap_public_smoke.sh:93` unsets `SIFR_TEST_CHANNEL_METADATA_PATH` *after* the stable-rejection run (`:74-86`) and the fresh public install (`:88-92`). `internal_docs/distribution_pipeline.md` and the new `schema-bootstrap-public-smoke` inventory entry both claim the smoke runs "without the qualification override," and `schema_epoch_bootstrap_workflow_contract.sh:97` asserts the `unset` literal is present — but the guarantee only holds for the final `self update` invocation. Move it above line 31, and prefer a hard `test -z "${SIFR_TEST_CHANNEL_METADATA_PATH:-}"` failure over a silent unset: a set override means the smoke was mis-invoked.

### 7. Poller extraction abandons a live site run on repeated query failure — Low-Medium

`poll_site_release_run.sh:51-56` now `exit 2`s from inside the loop on the third consecutive query failure. The replaced inline code set `poll_error` and `break`, reaching the cancellation call (now `:93-95`). An already-matched, still-running site deployment is now left in flight after the protected job fails. Use `break` with a sticky error flag so the cancel path still runs.

### 8. Empty review artifact — Low

`plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-1.md` is 0 bytes and is not referenced from the issue ledger. Populate or remove before commit.

### 9. `release-publication.yml` is at 851/900 lines with no automated cap — Low

`scripts/check_file_size_guardrails.py:112-126` categorizes only `crates/**/*.rs`, `scripts/**/*.py`, `verification/**/*.py`, and `demos/**/*.sifr`, so this file's 586-line growth is unenforced even though AGENTS.md's 900-line rule lists no YAML exclusion. The rest of 40.5 (`ga-activation`, `normal`, `rollback`, `incident-roll-forward`, the `drill` job, Marketplace) must land in this same file under the phase's "do not add a second mutation workflow" constraint. Extract the bootstrap stages into a called workflow or step scripts now, and/or extend the guardrail to `.github/workflows/*.yml`.

### 10. Unrelated reformatting inflates the reviewable diff — Low

`scripts/distribution/release_governance.py` carries ~20 pure line-wrapping hunks in untouched functions (`generate_release_index`, `plan_stable_release`, `generate_incident`, `_require_clean_external_incident_directory`); `runner.py` carries ~15 more; `schema_contracts.py:490-492` converts an unrelated lambda to a def. `validate_bootstrap_evidence` is also inserted out of alphabetical order at `release_governance.py:28` and `governance/__init__.py:15,34` — the same class of nit pass 2 of 40.4 asked to be fixed.

### 11. `--ruleset-id` is unvalidated before `jq --argjson` — Low

`verify_site_workflow_identity.sh:30-33` checks `--ruleset-id` only for non-emptiness, then feeds it to `jq --argjson id`, which aborts with a raw jq parse error instead of a governed message for any non-numeric value. Add `"${ruleset_id}" =~ ^[0-9]+$` to the guard.

---

### External execution requirements (not implementation prerequisites)

- `stable-release` environment must exist with ≥1 required reviewer from `release/distribution`; enable GitHub's "prevent self-review" so the in-workflow check at `release-publication.yml:208-218` is a second layer, not the only one. With no reviewers configured the approvals list is empty and the job fails closed — correct, but it fails late, after `prepare`.
- `release-publication.yml:67-68` now routes *ordinary* preview publication through a `preview-release` environment. GitHub auto-creates it rule-free on first use, so behavior is unchanged today; confirm no reviewers are attached unless routine previews are meant to require approval.
- The live `channels.json` must still be exactly 105 bytes / `71b3243925…4bf9ef` when `bootstrap-index` runs; any intervening mutation permanently disables this path (by design, and correctly so).
- `sifr.sh/install` and `/install/stable` must serve the new dispatcher bytes within the 180-second convergence budget at `run_schema_bootstrap_public_smoke.sh:37`.

VERDICT: NOT SATISFIED
