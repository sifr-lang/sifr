## Archived review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 11)

Read: phase plan, `plans/issues/active/phase-40-stable-channel-ga-execution.md`, the full working-tree diff vs `d8dd28a80` (22 tracked files + 22 untracked), and archived bootstrap passes 1–10.

### Verification of the two-line remediation

| Item | Result |
|---|---|
| `verification/runner/sifr_verify/selftest.py:89` `len(governed) != 13` | Correct — `schemas/` holds exactly 13 `*.schema.json` files including `schema_epoch_bootstrap_evidence.schema.json` |
| `selftest.py:684` adds `epoch-bootstrap` to the release-report production fixture | Correct and load-bearing — `release_report.REQUIRED_SUITES["distribution_release"]` (`release_report.py:41-46`) now requires it and `validate_profile` (`:160-166`) enforces it; the fixture passes `validate_release_profile_report` without `source_root`, so `REQUIRED_SUITES` is the only gate it must satisfy |

### Whole-wave registration sweep (the class of gap that broke pass 10)

Every surface that must know about the new suite/schema is registered and consistent: `manifest.json` (epoch-bootstrap adapter), `runner.py:51,134,195,205,214` (execution + `validate_suite_case` command/entry maps + `full`-suite dedup), `merge/nightly/release.json`, `profile_assignment_matrix.json`, `release_report.REQUIRED_SUITES`, `governance/selftest.py:244,301`, `qualification_fixture.py:123`, `schema_contracts.py:106,409,449`, `sifr_verify/selftest.py:89,684`, `internal_docs/architecture.md:1433`, `distribution_pipeline.md`. `schema_contracts.validate_schema_contracts:29-31` and the assignment-matrix `validate_row_membership` are name-derived, so `epoch-bootstrap`/the new schema are pinned rather than merely listed. No stale exact-count or exact-list fixture remains anywhere in `verification/` or `scripts/`.

Independently cross-checked `REQUIRED_SUITES` against the **real** `verification/profiles/release.json` (not the fixture): all four areas satisfied, `validate_profile` passes.

### Gates re-run in this worktree

`sifr_verify --self-test` (all 11 sections) PASS · `coverage_matrix` 5/5 PASS (`rows=17`) · `distribution_release` full area **110 variants, 0 failures**, `schema-v2-preview-epoch-bootstrap` appearing exactly once · merge-selection simulation (`representative`+`qualification`+`incident-governance`+`epoch-bootstrap`) 55/55 PASS · named `epoch-bootstrap` suite PASS · `documentation --suite structure` PASS · file-size guardrail PASS (2898 files) and its self-test PASS (`release-publication.yml` 795/900) · `git diff --check` clean · `compileall` + `bash -n` clean on all new/changed sources · asset-set arity confirmed 9, matching the schema's `minProperties: 9` / `not minProperties: 10`.

*(Note: the full-area run initially aborted on host `ENOSPC`, not a test failure; it was re-run to completion after space freed.)*

## Actionable findings

**1. MEDIUM — the execution ledger stops at pass 10 and now overstates the wave's state.**
`plans/issues/active/phase-40-stable-channel-ga-execution.md:435-440` records pass 10 as `VERDICT: SATISFIED` "with no actionable findings" and nothing follows it. The authoritative create-PR gate subsequently found two real verification-runner integration defects (stale governed-schema count; release-report production fixture missing `epoch-bootstrap`), both now fixed in the tree, and neither the gate run nor the remediation is recorded. Every comparable event in this issue is ledgered — including this milestone's own qualification-isolation wave, where the same situation is written up at `:332-335` ("The first authoritative create-PR profile found that…"). As it stands, a reader of the issue file concludes the wave was clean at pass 10. Add the create-PR discovery, the two fixture repairs, and the pass-11 artifact/link, consistent with `:299-302` and `:332-341`.

**2. LOW — the new `release-publication-prepare.yml` is outside both v1-residue guards.**
`verification/areas/distribution_release/cases/schema_epoch_bootstrap_workflow_contract.sh:145-151` sweeps for the forbidden `bootstrap_channel_metadata.py` / `migrate` / `fallback` fragments over `publication` and `bootstrap` only — `prepare` is read at `:14-16` and asserted against a positive fragment list at `:90-99`, but is never subjected to the forbidden sweep. Independently, `verification/areas/distribution_release/governance/schema_epoch.py:12-16` lists only `preview-release.yml` among workflows in `GOVERNED_FILES`, so neither publication workflow is scanned for `V1_PATTERNS`. The prepare job is precisely the surface that downloads the pre-epoch `channels.json` (`release-publication-prepare.yml:123-142`), i.e. the most likely place for a v1 reader/fallback to reappear against the milestone's "no v1 reader, writer, fixture, migration, negotiation, or fallback survives" invariant. I confirmed the file is currently clean, so this is a guard-coverage gap rather than a live defect; adding `prepare` to the loop at `:150-151` closes it.

## Not findings

- `profile_assignment_matrix.json` omits `distribution_release:qualification` and `:evidence-custody` while `REQUIRED_SUITES` mandates them, so those two are not statically pinned to `release.json`. Pre-existing (predates this wave); `epoch-bootstrap` *is* pinned by the matrix, so the new suite is fully guarded.
- `_release_report_production_self_test` hardcodes the suite selection instead of deriving it from `release.json`. This is what made the defect visible rather than silent, and the matrix row supplies the missing direction of the constraint.
- `test_materializer` patches `sha256_bytes` module-wide (`schema_bootstrap_selftest.py:435-439`); that symbol is used only for the opaque legacy identity in this module, so the fixture is sound.
- `run_schema_bootstrap_public_smoke.sh` leaves `stable-dispatcher.sh` in the smoke directory; `materialize_bootstrap_evidence` reads the four `<id>.txt` files by name and does not require an exact directory listing.
- 0-byte `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-11.md`: commit mechanics for this report; not modified, per instruction.
- Downstream live setup (`stable-release` environment reviewers/prevent-self-review, live 105-byte pre-epoch asset at bootstrap time, `SIFR_WEBSITE_ACTIONS_TOKEN`) treated as non-prerequisite per scope.

VERDICT: NOT SATISFIED
