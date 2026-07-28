# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 4)

I read the full `git diff`, all 16 untracked files, the phase doc (`plans/phases/40_stable_channel_ga_promotion_and_release_governance.md:938-1010`), the issue ledger, and the archived pass-1/2/3 reports, and independently re-ran the gates plus schema-weakening probes.

### Pass-3 re-audit

| # | Pass-3 finding | Status |
|---|---|---|
| 1 | Pre-dispatch site-workflow re-verification ungated | **Resolved.** `site_release_workflow_contract.sh:96-104` asserts `count == 2`, `first < "Publish write-once version release and verify assets"`, and `"Dispatch exact site workflow" < second < "Poll exact site run"`. I confirmed the two call sites at `release-publication.yml:212-220,678-686`, and that the extracted helper's own diagnostics are now asserted against `verify_site_workflow_identity.sh` directly (`:77-84`) rather than the caller. |
| 2 | Bootstrap self-test ran twice per default full-area run | **Resolved.** `runner.py:51-57,113-118,153-159` mirrors the incident dedup exactly. Measured: default run = **110 variants, 0 failures**, `case=schema-v2-preview-epoch-bootstrap` appears **1** time (same as `case=incident-recovery`). |
| 3 | `bootstrap_evidence_bytes` dead code | **Resolved.** Both it and the `canonical_json_bytes` import are gone; `sha256_bytes`/`sha256_file` are the only serializers used. |
| 4 | Smoke set unread `SIFR_SYSROOT_DIR` | **Resolved.** `run_schema_bootstrap_public_smoke.sh:80,94` use `SIFR_SYSROOT_INSTALL_DIR`, and `schema_epoch_bootstrap_workflow_contract.sh:114,125` both require the correct spelling and forbid the dead one. |
| 5 | `alpha-assets` stage never validated against the JSON Schema | **Resolved** for the positive case: `schema_contracts.py:66-73` deletes the four conditional keys and validates the alpha-stage instance. See finding 2 for the negative half. |
| 6 | Duplicate `from .common import` | **Resolved.** Single import block, `schema_bootstrap.py:8-25`. |

All pass-1 and pass-2 remediations remain in place. I re-verified: no schema-v1 parser/fixture/migration/fallback exists (the `forbidden` sweep at `:145-151` plus `build_preview_epoch` taking only digest+size); the digest chain prepare→publish→live holds at `release-publication.yml:80-84,285-290,303-309,568-573`; `--clobber` count is pinned at 1; `bootstrap-alpha` never touches `channels.json`; site publication facts carry no `operation` field, so the `schema-epoch-bootstrap` plan operation cannot fail that step post-mutation.

Gates re-run here: default distribution area PASS (110 variants, 0 failures); `governance.schema_bootstrap_selftest` PASS; the three touched/new contract cases PASS; `bash -n` over all `scripts/distribution/*.sh` PASS; file-size gate PASS (`release-publication.yml` at 795/900); `git diff --check` PASS. Ruff is not installed under the active interpreter, so I could not reproduce that one.

## Actionable findings

### 1. The new `epoch-bootstrap` suite is assigned to no validation profile, so the merge gate never executes the bootstrap validator or producer — Low-Medium

`verification/areas/distribution_release/manifest.json:79-88` registers the suite and `runner.py:194-215` validates its command/entry, but the name appears in **none** of `verification/profiles/{create-pr,merge,nightly,release}.json` nor `verification/areas/coverage_matrix/profile_assignment_matrix.json`. Consequently the module runs only via the `full` suite's unconditional append (`runner.py:153-159`), and `full` is selected by `nightly` and `release` only.

Measured, using the merge profile's exact selection:

```
runner.py --suite representative --suite qualification --suite incident-governance
  → variants=54, failures=0   # no schema-v2-preview-epoch-bootstrap, no governance-contracts
```

`scripts/run_all_tests.sh` with no arguments is the merge gate and AGENTS.md's authoritative pre-PR check. Under it, the three new shell contract cases *do* run (they live in `cases/`, so `representative` picks them up — I confirmed `schema_epoch_bootstrap_workflow_contract`, `site_release_workflow_contract`, and `preview_release_workflow_yaml_parses` all execute there). What does **not** run is `governance.schema_bootstrap_selftest` — the only coverage for `validate_bootstrap_evidence`, `resolve_distinct_approvers`, `build_preview_epoch`, and `materialize_bootstrap_evidence`, i.e. the code that authorizes the irreversible one-time mutation. The two comparable named governance suites, `qualification` and `incident-governance`, are both in `merge.json:distribution_release`.

The self-reported "default distribution run PASS: 110 variants" is a bare `runner.py` invocation, which selects every manifest suite including `full`; it is not what the merge gate or CI runs. Add `epoch-bootstrap` to `merge.json` (and, following `incident-governance`, to `nightly.json`/`release.json` plus `profile_assignment_matrix.json` and `release_report.py:REQUIRED_SUITES`), or drop the named suite and document the module as `full`-only like `governance.selftest`. As it stands the suite is selectable by no profile and is not listed among the suite commands in `internal_docs/architecture.md:1432` or the new `distribution_pipeline.md` section either.

(`governance-contracts` being merge-absent is pre-existing and not new to this slice; it is noted only because it means `validate_schema_contracts` — including finding 2's coverage — is also nightly/release-only.)

### 2. The pass-2 `public_smoke` `contains`/`maxContains` blocks and the stage `if/then/else` have no negative coverage — Low

`schema_contracts.py:74-88` registers two bootstrap negatives. I probed which schema constraints they are actually load-bearing for:

- `duplicate_smoke` sets `public_smoke[1].id = public_smoke[0].id`. Since the fixture's four records differ *only* by `id`, the mutation makes two records byte-identical, so `uniqueItems: true` (`schema_epoch_bootstrap_evidence.schema.json:72`) rejects it on its own.
- `extra_asset` is rejected by `$defs/alpha_assets`'s `not: {minProperties: 10}` (`:143`).

Measured against weakened copies of the schema:

```
delete properties.public_smoke.allOf   → duplicate_smoke=REJECT, extra_asset=REJECT   (gate stays green)
delete top-level allOf                 → duplicate_smoke=REJECT, extra_asset=REJECT,
                                          alpha-assets instance retaining "beta"=ACCEPT (gate stays green)
```

So deleting either pass-2 remediation leaves `validate_schema_contracts` passing. With `properties.public_smoke.allOf` gone, `[{id:A,sha:1},{id:A,sha:2},{id:B,…},{id:C,…}]` validates against the JSON Schema while `validate_bootstrap_evidence:217-219` rejects it (`smoke_id in seen`) — exactly the schema/validator parity break this area has spent passes 40.0/4–6, 2/#1 and 3/#5 closing. The schema is correct today (I verified all four cases reject); only the gate is blind. Add a duplicate-id-with-distinct-digest negative and an `alpha-assets`-instance-retaining-`beta` negative.

## Not findings

- The four cross-field semantics 2020-12 cannot express (asset map vs sibling `version`, exact asset-set membership vs the 9-property bound, approver vs sibling `initiator`, case-folded approver uniqueness) remain validator-stricter and semantically covered at `schema_bootstrap_selftest.py:132,139-141`.
- `release-publication.yml` at 795/900 with the remaining 40.5 scope (`ga-activation`, `normal`, `rollback`, `incident-roll-forward`, `drill`, Marketplace) to land in the same file leaves 105 lines. Pass 1 adjudicated this resolved once the guardrail was extended; I have no new evidence to reopen it, but the extraction discipline established here needs to continue.
- Post-`gh release create` failures are unrecoverable by re-run in every mode; for `bootstrap-index` the evidence upload is necessarily last. Fail-loud over recording an unverified pass remains correct.
- `poll_site_release_run.sh` preserves the sticky-`poll_error`-then-cancel ordering, gated at `preview_release_workflow_yaml_parses.sh:110-114`.
- `verify_release_publication_assets.sh:81-99` and `release-publication-prepare.yml:166-175` compute basename-keyed digest maps from the same `sha256sum`/`capture` idiom, so the prepare↔publish `.assets` equality is sound.
- `materialize_schema_bootstrap_evidence.py:74` catches `GovernanceError`, `OSError`, and `json.JSONDecodeError`; every failure path in `schema_bootstrap.py` routes through `fail()`.
- The fifth smoke output `${out}/stable-dispatcher.sh` is not in `SMOKE_IDS` and is correctly not exact-set-checked.

## Commit mechanics and execution requirements (not implementation defects)

- `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-4.md` is a 0-byte untracked file. It must receive this report, be archived, and be ledgered in `plans/issues/active/phase-40-stable-channel-ga-execution.md` (passes 1–3 are at `:361-386`; pass 4 is absent). I did not modify it, per instruction.
- **New pre-flight item:** `run_schema_bootstrap_public_smoke.sh:45` appends `?sifr_publication_smoke=${RANDOM}-${SECONDS}` to every polled URL, including `https://github.com/${repository}/releases/download/channels/channels.json`. There is no precedent for this pattern anywhere else in `scripts/`. If GitHub's release-download redirect or Cloudflare rejects the unknown query parameter, the `governance-index` check can never converge and the run fails *after* `channels.json` has been replaced and the site deployed. Verify one `curl` against the live endpoint with the parameter before the bootstrap run, not during it.
- Pass-1's external requirements stand: `stable-release` environment with ≥1 `release/distribution` reviewer and GitHub "prevent self-review" enabled; confirmation that no reviewers are attached to the auto-created `preview-release` environment; live `channels.json` still exactly 105 bytes / `71b3243925…4bf9ef` at `bootstrap-index` time; `sifr.sh` serving new dispatcher bytes inside the 180-second budget.
- Still not locally reproducible: whether `actions/download-artifact@v4` resolves an attempt-1 artifact during "Re-run failed jobs". Validate on the first live re-run.

VERDICT: NOT SATISFIED
