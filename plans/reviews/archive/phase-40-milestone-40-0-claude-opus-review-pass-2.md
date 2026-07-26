# Re-review: `milestone_40_0` — Architecture and Gate Lock (pass 2)

Scope inspected: the complete working-tree diff (39 modified, 22 untracked additions), the Phase 40 plan, the execution issue, and the pass-1 review. I re-ran `governance.selftest` (14/14), `governance.schema_epoch`, `governance.evidence_custody`, `documentation/check_structure.py`, `demos/milestone_40_0_demo.sh`, and the three new runner self-tests (`precondition`, `production`, `documentation`) — all pass.

## Pass-1 findings: re-audit

| # | Pass-1 finding | Status |
|---|---|---|
| 1 | `release` profile omits `rust_interop:stable-candidate` | **External blocker — still open** (see below) |
| 2 | 899 lines via blank-line stripping | **Fixed.** `profile_runner.py` is 868 lines; step execution extracted to `profile_area_steps.py` (39) + `profile_results.py` (75); PEP8 blank lines at `:1-4` and `:26-29` restored |
| 3 | Tautological index CAS, no re-verify before replacement | **Fixed.** `preview-release.yml:298-316` re-fetches the live index, re-validates canonically, and compares generation+digest against `EXPECTED_INDEX_*` immediately before the `channels.json` upload |
| 4 | Moving `base_ref` recorded as provenance | **Fixed.** `validate` resolves `source_sha` once (`:80`); build (`:101`), publish checkout (`:146`), and `--source-commit` (`:212`) all consume it |
| 5 | Evidence custody fails open | **Fixed.** `evidence_custody.py:111-116` `require_comparison_base` + negative self-test (`governance/selftest.py:718-721`) |
| 6 | Channel-downgrade guard deleted | **Fixed.** `release_index.py:138-146` monotonicity check, mutation-covered at `governance/selftest.py:427-436` |
| 7 | No end-to-end release-report coverage | **Mostly fixed.** `build_release_profile_payload` factored and driven end-to-end by `_release_report_production_self_test` (`selftest.py:517`) against synthesized lane logs and area results |
| 8 | Real-run preferred site shadow / added `curl` | **Partly fixed** — see local finding 4 |
| 9 | `rc` scope creep into 40.2 surfaces | **Fixed.** `generate_version_installer.sh`, `build_preview_artifacts.sh`, `trigger_preview_release.sh` untouched; `preview-release.yml:66` and `create_new_version.sh:124` retain `rc` |
| — | editor-release evidence twice | **Fixed.** `release_evidence.py:243-251` splits non-editor/editor cases, so `editor-release:*` appears exactly once |
| — | Docs self-test overwriting release evidence | **Partly fixed** — see local finding 10 |
| — | Zero-digest schema alignment / `--release-report-out` missing value | **Fixed** |
| — | `target_milestone` inconsistency | Field removed from the inventory — but see local finding 9 |

## External integration blocker (do not implement here)

`verification/areas/rust_interop/manifest.json` still exposes only `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`, and `verification/areas/rust_interop/data/` contains no `stable_support_claims.json`. Consequently `verification/profiles/release.json:selected_areas[rust_interop]` cannot list `stable-candidate`, while `release_report.py:38` mandates it in `REQUIRED_SUITES` — so a real `--release-report-out` run on the release lane still fails `validate_profile`/`validate_steps`. Integration once the prerequisite lands: append `"stable-candidate"` to that one array; it flows through `run_rust_interop_checks` → `run_selected_area` → `validate_area_result` and satisfies both required-suite checks. No other local code change is needed.

## Local findings

**1. MEDIUM — `verification/areas/distribution_release/governance/release_plan.py:56,96`: conditionally-required field raises `KeyError` instead of a governed rejection.**
`incident_request_sha256` is declared *optional* for `incident-roll-forward`, then indexed unconditionally at `:96`. An `incident-roll-forward` plan that omits it crashes:
```
$ python3 -c "... validate_release_plan(plan_without_digest)"
KeyError 'incident_request_sha256'
```
The DoD explicitly requires rejecting "an `incident-roll-forward` plan without … a matching approved incident-request digest". `release_governance.py:146` catches only `GovernanceError`, and `evidence_custody.run_evidence_custody_checks` likewise, so this surfaces as a traceback, not a governed diagnostic. `stable_release_plan.schema.json` also declares `incident_request_sha256` as a plain optional property with no `if/then` requirement, so neither layer catches the omission. Make it required when `transition == "incident-roll-forward"` in both the validator and the schema, and add the omission mutation.

**2. MEDIUM — `governance/incident.py:48,81`: same defect for `rollback_target`.**
```
$ python3 -c "... validate_incident_request({**req, 'operation':'rollback'} minus rollback_target)"
KeyError 'rollback_target'
```
`stable_incident_request.schema.json:36` *does* conditionally require it, but the Python validator is the fail-closed path used by evidence custody and `release_governance.py`, and it crashes. DoD: "a rollback request with an inactive or mismatched target/plan digest" must be rejected. No mutation coverage for the omission.

**3. MEDIUM — the ten checked-in governance JSON Schemas are never applied to any artifact.**
`sifr_verify/schemas.py:41` (`validate_all_committed_schemas`) walks `verification/schemas/`, not `verification/areas/distribution_release/schemas/`; nothing in `governance/`, `release_governance.py`, or the case scripts validates a payload against them (only `artifact_self_update_receipt_rules.sh` does, for the receipt). The sole coverage is `governance/selftest.py:327-335` checking epoch shape. This is exactly how finding 1's schema/validator divergence stayed invisible. Add a self-test that validates each governance fixture against its checked-in schema, and lint these schemas with the runner's schema checker.

**4. MEDIUM — `scripts/distribution/create_new_version.sh:153-171` still sources channel state from the site shadow or the network, and it can now disagree with `--release-index`.**
`read_current_channel_versions()` prefers `${INSTALL_ROOT}/channels.json` and otherwise `curl`s the canonical asset; it runs in both modes via `build_plan`. Its `CURRENT_ALPHA`/`CURRENT_BETA` feed `NEW_ALPHA`/`NEW_BETA` (`:212-218`), the plan text and `PLAN_SHA` (`:249-259`), and the checklist (`:301`) — while the index actually produced comes from `--release-index` (`:369-390`). Before this change the two were consistent by construction, because `generate_channel_metadata.sh` consumed `NEW_ALPHA`/`NEW_BETA`; nothing now cross-checks them, so a real run can emit a plan/checklist naming channel versions the generated index does not contain. Every real-run case (`create_new_version_attribution_checklist.sh:28`, `create_new_version_missing_artifact_rejected.sh:25`, `create_new_version_real_run_plan_reuse.sh`) passes the site shadow as `--release-index`, so divergence is untested. Derive the plan's channel state from `--release-index`, or assert agreement.

**5. LOW-MEDIUM — `crates/sifr/src/self_update_metadata.rs:719-724`: `rejects_stable_metadata` no longer tests what it names.**
The fixture sets `"ga_status":"active"`, which fails the new `ga_status != "preview"` gate before the channel loop is reached, so the reader's stable-channel-key rejection (`:205-210`) is untested. The DoD's "preview metadata with `stable`" case is covered on the Python side only. Use `"ga_status":"preview"` with valid alpha/beta release records plus a `stable` channel key.

**6. LOW-MEDIUM — no producer↔consumer parity test for the two new CLI JSON surfaces.**
`self-version` and `self-update-plan` validators exist (`surface_contracts.py:95,121`) and schemas are checked in, but no case feeds real `sifr self version --format json` / `sifr self update --dry-run --format json` bytes to `release_governance.py validate`, unlike the receipt path. The atomic-cutover DoD names these surfaces explicitly. Note also `validate_self_version:103-104` requires `sysroot_sifr_version == receipt_version`, which contradicts that surface's own purpose of reporting `matches_receipt: false` — worth resolving while wiring the parity case.

**7. LOW-MEDIUM — `governance/schema_epoch.py:12-18` implements the DoD's "repository search checks" as a 5-file allowlist.**
`GOVERNED_SOURCES` covers only `self_update_cli.rs`, `self_update_metadata.rs`, `generate_channel_metadata.sh`, `release_governance.py`, and `preview-release.yml`. `generate_version_installer.sh` (the receipt producer), `self_update_receipt.rs` (the consumer), `generate_dispatchers.sh`, `tools/validate_self_update_metadata.sh`, and the case fixtures are never scanned, so v1 residue there would pass. Walk the governed surface set instead of enumerating it.

**8. LOW — `internal_docs/distribution_pipeline.md:320-329`: documented real-run invocation is now invalid.**
The example omits the newly required `--release-index`, so it fails with `--release-index is required with --real-run`; the flag appears nowhere in the document.

**9. LOW — planning/tracking drift.**
`plans/issues/active/phase-40-stable-channel-ga-execution.md:33` still asserts every inventory entry has a "target milestone", but `target_milestone` was removed from `plans/releases/stable_gate_inventory.json`. All ten `milestone_40_0` checkboxes (`:41-57`) remain unchecked with no commands, evidence, review rounds, or PR link, while the plan's Validation Contract requires them per milestone; `plans/roadmap.md:85` still lists Phase 40 as `planned`. `plans/reviews/active/phase-40-milestone-40-0-claude-opus-review-pass-2.md` is a zero-byte placeholder.

**10. LOW — `verification/runner/sifr_verify/selftest.py:440` still executes the real `documentation` area in every lane.**
The evidence-overwrite half is fixed (`runner.profile_name = "documentation-self-test"` gives it a distinct result file), but `_documentation_profile_self_test` runs inside `verification_hardening_self_tests`, which executes on every profile — so `create-pr`, `merge`, and `nightly` now run an area their manifests do not select, diverging from `--emit-plan`/`compare-plans` output and adding wall time. Assert on the release lane's already-produced result file and keep the negative case pure-unit.

**11. LOW — `verification/areas/distribution_release/cases/preview_release_workflow_yaml_parses.sh:9-34` checks fragments, not structure.**
It never asserts that the live-index recheck step *precedes* `gh release upload channels`, nor that the workflow rejects stable. Moving the recheck after the upload would still pass. Parse the YAML and assert step ordering within `publish-release`.

**12. LOW — documentation inventory registration is partially self-enforcing.**
`verification/areas/documentation/check_structure.py:16-19` never verifies that an `active` check's `suite` exists in `documentation/manifest.json` (it holds only because `ga-release` is `reserved`), and `docs_inventory.json` declares an `unsupported-rust-claim` mutation case that `check_ga_release_docs.run_self_test()` never exercises.

**13. LOW — `scripts/distribution/generate_dispatchers.sh:125-134` (carried from pass 1).**
`validate_preview_release_index` still greps raw metadata text, so a `"generation"` or `"ga_status":"preview"` occurring anywhere satisfies it and the resolved version's release record is never checked for `status: active`. Additionally, the hard `ga_status == "preview"` requirement will fail *alpha and beta* dispatchers once GA activates, until they are regenerated — acceptable while 40.2 owns dispatcher parsing, but it should be recorded there.

## Verdict

**CHANGES REQUESTED — 13 local findings** (2 medium correctness defects with identical root cause in `release_plan.py:56,96` and `incident.py:48,81`, 2 medium coverage/consistency gaps in unapplied governance schemas and the `create_new_version.sh` plan-vs-index split, and 9 lower-severity test, guard, docs, and tracking items). The `rust_interop:stable-candidate` suite and `stable_support_claims.json` remain absent and are correctly scoped as an external integration blocker owned by another worktree.
