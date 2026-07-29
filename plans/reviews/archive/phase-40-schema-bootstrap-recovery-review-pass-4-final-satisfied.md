> Archived as the final satisfied exact-PR-head review.

Unfiltered complete area confirmed at **125 variants, 0 failures** — the corrected ledger wording and the recorded pass-3 measurement are both exact.

## Head identity — confirmed

| Ref | SHA |
|---|---|
| local `HEAD` (`codex/phase40-bootstrap-index-recovery`) | `cd92c820611d0f7c3fcd657ccdd46b05283057b7` |
| `origin/refs/heads/codex/phase40-bootstrap-index-recovery` | `cd92c820611d0f7c3fcd657ccdd46b05283057b7` |
| PR #3061 `headRefOid` (OPEN, base `main`, MERGEABLE) | `cd92c820611d0f7c3fcd657ccdd46b05283057b7` |

Base `origin/main` = `94a5fec67b7bef51cae0034c84386c57d9ff1785`. Full PR: 17 files, +1723/−39. Dirty `third_party/ruff`, `editor_integrations`, `leetcode` and the untracked `plans/reviews/active/phase-40-...pass-4-final.md` are working-tree only, not in the PR. I modified no files.

## Delta verification

**Since `df5a5f5f5` (pass-3 head):** exactly two markdown files, +64/−2. One ledger bullet reworded, one new archive file. Zero diff in any `.py`/`.sh`/`.yml`/`.json`/schema file — tracking-only confirmed.

**Since implementation head `ddcd7e3d6`:** three markdown files, +103/−0 (the two pass-2 tracking additions plus this commit). No implementation change across either delta.

**Factual exactness of the new archive file** — every checkable claim verified:
- Diffstats: `main..df5a5f5f5` = 16 files/+1661/−39 ✔; `main..ddcd7e3d6` = 15 files/+1620/−39 ✔; `ddcd..df5a` = 2 files/+41/−0 ✔.
- Its F1 premise: the pass-2 archive at line 22 *does* record the filter (`runner.py --suite full` → 67), so calling the archive itself accurate and localizing the defect to the ledger paraphrase is correct.
- Citations: `_resolve_legacy_identity` at `schema_bootstrap.py:515` ✔, `materialize_bootstrap_evidence` at `:364` (gates at 390–404) ✔, `fetch_schema_bootstrap_beta.sh:83` is the `tr -d '[:space:]'` compare ✔, `build_release_artifacts.sh:126` is `sha256_file` ✔.
- Its measurements reproduce: `--suite full` → 67/0, unfiltered → 125/0.

## Pass-3 sole finding — closed

`plans/issues/active/phase-40-stable-channel-ga-execution.md:792` now reads "67-variant distribution `full` suite" instead of "complete 67-variant distribution area." Pass 3 is archived at `plans/reviews/archive/phase-40-schema-bootstrap-recovery-review-pass-3-not-satisfied.md` (correct `-not-satisfied` naming, matching the pass-1 convention), and the ledger records both that the wording was corrected and that the reviewer independently ran the unfiltered complete area at 125/125. The complete-area convention elsewhere in the ledger (125/125 at lines 752, 704, 712, 1064) is no longer contradicted.

## Complete-PR recheck — no remaining actionable issue

- **Legacy identity boundary.** `_resolve_legacy_identity:515-541` forbids exact-bytes and attested identity together, requires both attested fields, and rejects `bool`/zero/negative sizes. The resolved pair still flows into `require_opaque_legacy_identity`, whose pins are the module constants `LEGACY_INDEX_SHA256`/`LEGACY_INDEX_SIZE_BYTES` (`schema_bootstrap.py:32-33`) — byte-identical to the workflow's inline `71b3243…`/`105`, so those inline values are not trust-bearing; the schema independently const-pins `size_bytes: 105`.
- **Fail-closed both directions.** `materialize_bootstrap_evidence:390-404`: attested path requires `stage == preview-index` *and* `recovery`; exact-bytes path forbids `recovery`.
- **Recovery provenance.** `_validate_recovery:317-360` enforces the exact seven keys, positive ints, non-empty initiator, `validate_approval_policy`, approvers bound to the recovery initiator, and `site_run_id != failed_site_run_id`. Schema `$defs/recovery` mirrors it with `additionalProperties: false`, and the top-level `else` branch adds `{"not": {"required": ["recovery"]}}`.
- **Workflow.** Top-level `contents: read`; `contents: write` only on `recover`, behind `environment: stable-release`; `concurrency: cancel-in-progress: false`. The single `download-artifact` is intra-run and digest-gated by `EXPECTED_RECOVERY_SUMMARY_SHA256`. All five site-workflow pins byte-identical to `release-publication.yml:145-149`.
- **Waiver.** `plans/releases/single-maintainer-approval-waiver.json` hashes `b9630cc0…`, matching `WAIVER_SHA256` in both jobs; `allowed_operations` includes `bootstrap-index`; `expires_at` `2026-08-27T00:00:00Z` as documented.
- **Live state.** Original run `30443929353` = `preview-release`/`workflow_dispatch`/`head_sha 94a5fec6…`/attempt 1/`completed`+`failure`. Failed site run `30445065348` = `release-site`/`ff472f2a…`/`"Sifr site release 30443929353-1"`/`completed`+`failure`. `channels` assets remain only `channels.json`, `channels-generation-1.json`, `schema-v2-bootstrap-alpha-0.1.0-alpha.2.json` — final evidence absent, write-once path open. Durable summary hashes `f45c012c…` as documented.
- **Checks rerun (all pass).** `schema_bootstrap_selftest` → PASS; `cases/schema_epoch_bootstrap_workflow_contract.sh` → exit 0; distribution area `--suite full` → 67/0 and unfiltered → **125/0**; `check_file_size_guardrails.py` → PASS (2959 files, limit 900); YAML parse of `schema-bootstrap-recovery.yml` clean; `bash -n` clean on the prepare script, fetch script, and contract case.

## VERDICT: SATISFIED

## Non-blocking observations

1. `expect_failure` in the self-test asserts only that a `GovernanceError` was raised, not which one, so the recovery/partial-identity/boundary mutation cases could in principle pass for an incidental reason. Passes 2 and 3 instrumented this by hand; a message assertion would make it durable in-suite.
2. The `recovery_evidence()` fixture uses `distinct-reviewer`/`none`, while the live run will emit `single-maintainer-waiver` with a real digest. That mode is covered only by out-of-suite simulation.
3. On read-back, `recovery` is merely optional for `preview-index` — `legacy_index` is const-pinned identically on both paths, so a standalone validator cannot re-prove that an attested-path payload carried provenance. Enforced only at materialization time; correct by construction.
4. The input re-grep depends on the failed site run's Actions log, whose retention (~2026-10-26) isn't listed alongside the two recorded deadlines. Later than the binding 2026-08-27 waiver expiry, so not a live risk.
5. The waiver has no `bootstrap-index-recovery` operation, so recovery reuses `--operation bootstrap-index` for both approval resolutions. Defensible, but the waiver's scope wording doesn't distinguish them.
6. `schema_bootstrap.py` and `schema_bootstrap_selftest.py` carry cosmetic line reflows unrelated to the remediation; no lint lane in `run_all_tests.sh` requires them.
