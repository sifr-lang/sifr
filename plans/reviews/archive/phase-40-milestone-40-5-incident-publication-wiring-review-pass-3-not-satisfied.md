## Review pass 3 — Phase 40 / milestone_40_5 incident publication + roll-forward production wiring

Scope: full working tree vs `origin/main` including all untracked files. No files modified. Independently re-ran: `distribution_release` combined selection (68 variants, 0 failures), `incident_publication_selftest` 5/5, `stable_public_smoke_selftest` 2/2, all five publication/site workflow contracts, file-size guardrail (2932 files, `release-publication.yml` 851 lines).

### Every pass-1 and pass-2 finding verified closed

| Pass | Finding | Verified |
|---|---|---|
| 1.1 / 2.1 | Sibling-checkout false dirtiness | `incident_prepare.py:579` `-uno` + `_require_head_file` at `:93,94,565`, commented at `:576-578` (closes 2.11) |
| 1.2 / 2.2 | Roll-forward end-to-end | `incident_publication_selftest.py:208-355` drives prepare → stage → release-signoff cross-bind → signoff, with 5 negatives; `stable_prepare_selftest.py:99` adds the stable-side prepare |
| 1.3 / 2.3 | Drill/adapter surface assertions | `test_protected_production_adapter_surface:413-453`; the one dropped assertion is covered by `protected_release_drill_workflow_contract.sh:38` |
| 1.4 | jq indexing `"none"` | `release-publication-prepare.yml:573,574,578` use `(.release_prepare \| objects)` |
| 1.5 / 2.5 | Cap by decomposition | 851 / 589 / 778 lines via real extraction; `contents: write` count == 1 asserted |
| 1.6 / 2.2 | Demo | `demos/stable_release_governance_demo.sh:75` now invokes `stable_public_smoke_selftest` |
| 1.8 / 2.3 | Site rationale + facts-driven docs | `incident_prepare.py:162-164`; `run_stable_public_smoke.sh:120-139` → `verify_public_stable_docs.py`; positive with non-empty `withdrawals` + missing-`incident_id` negative at `stable_public_smoke_selftest.py:133-164`; site facts pinned by tag/ruleset/digest and the 13-input cross-repo fixture |
| 1.9 | Hardcoded rollback Marketplace identity | `run_incident_publication.sh:369-406`, derived + charset- + digest-verified; `--compiler-version` positive/negative at `stable_publish_selftest.py:171,189` |
| 1.10 / 2.4 | Revalidation diagnostics | `revalidate_incident_publication.py:79,110` matches the stable sibling |
| 1.11 / 2.10 | Shared mutation schema | standalone `incident_index_mutation_evidence.schema.json` `$ref`ed; lint count 16 → 18 |
| 2.1 | Durable docs / ledger | `architecture.md:1449-1464`, `distribution_pipeline.md:584-680`, issue ledger `:715-753` all corrected and honest (checkboxes still open) |
| 2.4 | Worktree-drift negatives | present at `incident_publication_selftest.py:123-132` (but see finding 3) |
| 2.5 | Orchestrator guards executed | `test_incident_orchestrator_rejects_unprotected_and_unmerged:456-572` covers the unprotected ref and unmerged-incident ancestry with fake `git`/`gh` and asserts `gh` was never reached |
| 2.6 | Dispatcher-digest drift negative | `incident_publication_selftest.py:287-299` |
| 2.7 | Dead `affected_vsix` | removed |
| 2.8 | Collapsed diagnostics | `validate_preview_publication_inputs.sh:18-21,55-79` restores per-check messages |
| 2.9 | Recovery token scrubbing | `run_incident_publication.sh:521` scrubs at the call site |
| 2.10 | Cosmetic whitespace churn | reverted (16 insertions / 130 deletions, all real) |

---

### Blocker

**1. `run_incident_public_recovery.sh` reads a receipt path that no Sifr installer ever writes — every rollback and roll-forward fails after the index has already been clobbered.**

`scripts/distribution/run_incident_public_recovery.sh:80` and `:85` read `"${root}/install-receipt.json"`. The installer writes the receipt as **`install.json`**:

- `scripts/distribution/generate_version_installer.sh:590` — `manifest_path="${manifest_dir}/install.json"`
- `crates/sifr/src/self_update_receipt.rs:89,102,108,117` — every discovery path is `install.json`
- `verification/areas/distribution_release/cases/artifact_self_update_receipt_rules.sh:32` — `receipt_path="${install_root}/install.json"`

Repo-wide, the string `install-receipt.json` appears **only** in this script (the `--kind install-receipt` validator name is unrelated). I reproduced it end-to-end: built a real `sifr-installer-0.1.0`, installed with exactly the orchestrator's env (`HOME=$root SIFR_INSTALL_DIR=$root/bin SIFR_SYSROOT_INSTALL_DIR=$root SIFR_NO_MODIFY_PATH=1`), and the install root contains `install.json` and no `install-receipt.json` (`manifest_dir` resolves to `$root` via `generate_version_installer.sh:574-589`).

There is a **second** defect in the same two lines: `:78-81` passes `--require-canonical`. The installer writes pretty-printed JSON, so even with the corrected filename the validator rejects it. Verified against the real receipt:

```
validate --kind install-receipt --input .../install-receipt.json --require-canonical
  → invalid JSON: No such file or directory                      exit 2
validate --kind install-receipt --input .../install.json --require-canonical
  → must use canonical JSON bytes                                exit 2
validate --kind install-receipt --input .../install.json
  → validation ok                                                exit 0
```

`artifact_self_update_receipt_rules.sh:35-38` calls the same validator *without* `--require-canonical`, which is the correct precedent.

Failure trace: the loop at `:73-89` runs under `set -euo pipefail`, so both roots fail → `run_incident_public_recovery.sh` exits 2 → `run_incident_publication.sh:521` fails. That call site is **after** the sole `channels.json --clobber` (`:438`), after the activated-digest check, after the site dispatch/poll and public smoke, and after `materialize_stable_publication.py signoff` for roll-forward — but **before** `materialize_incident_publication.py signoff` (`:530`) and before any sign-off asset is retained (`:540-546`).

So on the first real rollback or incident-roll-forward: the governed index is withdrawn/activated, the site is deployed, the release and Marketplace version are published, and the run then fails with no incident sign-off. It is not convergent — protected `resume` re-enters `activated` state, skips the mutation, and replays the identical failure at the same line. Recovery requires a code change, not an operator action. Per the milestone, `incident-roll-forward` is the *only* usable incident operation at GA, so this blocks the primary path.

Fix: `--input "${root}/install.json"` and drop `--require-canonical` on lines 78-81 and 85.

**2. `run_incident_public_recovery.sh` is the only production script in this wave with no executed coverage — which is exactly why finding 1 survived two review rounds.**

`incident_publication_workflow_contract.sh:73` asserts only that the string `run_incident_public_recovery.sh` appears in the orchestrator. Nothing executes the script. Its sibling `run_stable_public_smoke.sh` *is* executed against a fake `curl` and a fake dispatcher (`stable_public_smoke_selftest.py:36-130`), and `run_incident_publication.sh` now has an executed guard test — this 102-line post-mutation script has neither. Entirely unexercised: the rollback downgrade-refusal branch and its `grep -F -- "--force"` on the diagnostic (`:56-63`), `run_working --force` (`:64`), the broken-client `rm` + out-of-band dispatcher path (`:65-66,69-70`), the roll-forward branch (`:68`), the `sifr --version` equality check (`:74`), and both receipt checks (`:78-88`).

The pattern from `stable_public_smoke_selftest.py` applies directly: a fake `sifr` that emits the downgrade diagnostic then a successor version, a fake stable dispatcher that installs it, and a receipt fixture — plus at least one negative (version drift or receipt-channel drift). Without executed coverage the fix for finding 1 is itself unverified.

---

### Minor / non-blocking

**3. The `_require_head_file` untracked-forgery negative is still missing.** The two negatives added at `incident_publication_selftest.py:123-132` append bytes to *tracked* files, which `_require_clean_checkout` (`incident_prepare.py:579`, `-uno`) already rejects on its own. `_require_head_file` exists precisely because `-uno` ignores untracked files, and that case — an untracked `plans/releases/candidates/<version>/stable-release-plan.json` or incident request planted in an otherwise clean root — is never exercised. It is a one-line addition to the existing rollback fixture (`git rm --cached` the plan, or write a plan for a version with no tracked blob).

**4. `plans/reviews/active/phase-40-milestone-40-5-incident-publication-wiring-review-pass-3.md` is 0 bytes.** Same as pass-1 finding 7; populate or remove before the PR.

**5. `stable_public_smoke_selftest.py:22` imports the private `_run`, `_stage` from `stable_publish_selftest`.** The finding-5 extraction left private cross-module coupling; promote them to a shared helper module.

**6. Residual (pass-2 minor 12):** `release-publication.yml` still repeats the same four-way mode-exclusion `if:` at 12 sites (142, 148, 153, 161, 168, 175, 185, 191, 197, 246, 266, 275…). The cap is met by genuine decomposition; this is readability only.

---

### Verified clean (fresh review)

- **Ordering / fail-closed:** revalidate → approvers → pre-mutation site identity → retained-release verification → affected-version client installs → dispatchers → (roll-forward: release + Marketplace publish) → stage → write-once request asset → re-fetch + revalidate → write-once generation snapshot → live `cmp` → single `--clobber` → activated-digest check → site dispatch/poll → public smoke → recovery → sign-offs. `--clobber` count == 1; ordering locked by `incident_publication_workflow_contract.sh:62-96`.
- **Exact custody:** request/withdrawal/plans bound to `HEAD` bytes and to the request's own digests; the affected qualification index is transitively bound via `plan["qualification_artifact_index"]["sha256"]` (`verify_retained_stable_release.py:46-52`); `revalidate` requires byte-exact reproduction of the reviewer-visible summary (`revalidate_incident_publication.py:72`).
- **Protected-main ancestry:** `workflow_ref = refs/heads/main`, `HEAD == GITHUB_SHA == origin/main`, merge-base ancestry for incident evidence and (roll-forward) candidate evidence + candidate source (`run_incident_publication.sh:81-160`); prepare enforces the same via `governance-source` (`:327,353`); rollback rejects successor inputs at `:108` and `:501`.
- **Generation burn / resume / races:** `allocate_next_generation` skips burned generations; a pending re-attempt gets a fresh generation so the write-once snapshot upload cannot collide; `activated` skips reservation and re-verifies the live digest; `upload_or_verify_governance` requires explicit `resume` and byte-compares; sign-off names are attempt-scoped, facts are generation-scoped with `${allow_existing}`; both `revalidate` calls re-fetch the index and snapshot set, so any concurrent generation fails closed.
- **Sole index mutation:** one `publish` job, `contents: write` count == 1, `sifr-release-index` concurrency with `cancel-in-progress: false` shared with preview/bootstrap, drills on a separate group/environment, `stable-release` environment for all six protected modes.
- **Rollback/roll-forward release + Marketplace:** rollback publishes no release (`verify_retained_version` on both affected and target instead), fetches the affected Gallery VSIX and proves `sifrCompilerCompatibility` contains the rollback target (`verify_marketplace_vsix.py:90-97`, `--compiler-version`); roll-forward publishes write-once and reuses-or-publishes the Marketplace version idempotently, both strictly before the clobber.
- **Site / public truth:** `generate_site_release_facts` derives `stable_version` and every `withdrawals[]` entry from the proposed index only (`release_plan.py:386-421`), never from site prose; `stage_incident_publication:68-74` pins the operation's approved site plan by digest; the dispatch passes the exact canonical facts digest, `none` for preview, and the cross-repo fixture asserts all 13 inputs, producer/renderer/route, preview absence, and the GA facts negative.
- **Schema/runtime parity:** `incident_publication_prepare` ⇄ `validate_incident_prepare_summary` (`release_prepare` const/`$ref` branches match); `stable_publication_prepare` `then/else` on `incident` matches `optional={"incident"}`; `stable_incident_signoff` `release_signoff_sha256` and the expanded `site_reconciliation` object agree with `incident.py:131-190`; lint count 18 correct.
- **Secrets / credentials:** `SITE_TOKEN`/`VSCE_PAT` captured then `unset` (`run_incident_publication.sh:116`, `dispatch_stable_site_publication.sh:100`); every installer/smoke/recovery invocation scrubs `GH_TOKEN`/`SITE_TOKEN`/`VSCE_PAT`; prepare asserted free of `secrets.`, `contents: write`, `gh release upload`, `vsce publish`; drill workflow asserted free of secrets, write scope, releases, `vsce publish`, and dispatches.
- **Smoke inventories:** exactly the 7 `stable_publish.SMOKE_FILES` (now including `stable-release-docs.html`) and the 8 incident files; both consumers assert set equality, so the shared `cp` at `:520` cannot drift either sign-off.
- Every `scripts/distribution/*.sh` individually `bash -n` parsed (`preview_release_workflow_yaml_parses.sh:7-9`); `incident_publication_workflow_contract.sh` auto-registered by directory discovery.
- No Rust-interop work; all changes stay within Phase 40 release governance.

VERDICT: CHANGES_REQUIRED
