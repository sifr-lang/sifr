> Archived after correcting its sole tracking-only accuracy finding.

Head resolution and delta verification complete; I found one actionable inaccuracy in the delta commit.

## Head identity — confirmed

| Ref | SHA |
|---|---|
| local `HEAD` (`codex/phase40-bootstrap-index-recovery`) | `df5a5f5f590fad1f7599a77f7c88e2648b248489` |
| `origin/refs/heads/codex/phase40-bootstrap-index-recovery` | `df5a5f5f590fad1f7599a77f7c88e2648b248489` |
| PR #3061 `headRefOid` (OPEN, base `main`, MERGEABLE) | `df5a5f5f590fad1f7599a77f7c88e2648b248489` |

Base `origin/main` = `94a5fec67b7bef51cae0034c84386c57d9ff1785`. Full PR diff: 16 files, +1661/−39. Dirty `third_party/ruff`, `editor_integrations`, `leetcode` are working-tree only, not in the PR. No files modified by this review.

## Delta since `ddcd7e3d656e39a2b00727a7ce6ac775fa823f1e`

`df5a5f5f5` touches exactly two markdown files (+41/−0): it adds `plans/reviews/archive/phase-40-schema-bootstrap-recovery-review-pass-2-satisfied.md` and one ledger bullet in `plans/issues/active/phase-40-stable-channel-ga-execution.md`. **No implementation change** — zero diff in any `.py`, `.sh`, `.yml`, `.json`, or schema file. Confirmed tracking-only. The archived review's own diffstat claim is exact: `94a5fec67...ddcd7e3d6` = 15 files, +1620/−39.

---

## VERDICT: NOT SATISFIED

### F1 — ledger entry misstates the review's distribution coverage as the complete area

`plans/issues/active/phase-40-stable-channel-ga-execution.md:792` (added by `df5a5f5f5`) records that pass 2 reran the "**complete 67-variant distribution** area". 67 is the `--suite full` subset, not the complete area. I measured both:

- `python3 verification/areas/distribution_release/runner.py --suite full` → `variants=67, failures=0`
- `python3 verification/areas/distribution_release/runner.py` (no filter, all ten manifest suites) → `variants=125, failures=0`

The archived review file itself is accurate — it writes the invocation with its filter (`runner.py --suite full` → `variants=67`). The defect is only in the ledger's paraphrase, which drops the filter and applies the word "complete." This also contradicts the established convention in the same ledger, where the complete area is consistently recorded as `125/125` — including the immediately preceding implementation entry at line 752 ("passed the complete distribution area 125/125", which I verified is correct) and the prior review entries at lines 704, 712, 1064.

Because this commit exists solely to record a factual review ledger entry, the entry needs to be exact. Fix: describe it as the 67-variant `full` suite, or rerun/record the complete 125-variant area.

---

## Everything else rechecked — clean

**Correctness / recovery.** `_resolve_legacy_identity` (`verification/areas/distribution_release/governance/schema_bootstrap.py:519-546`) forbids exact-bytes and attested identity together, requires both attested fields, rejects bool/zero/negative sizes, and the resolved identity still flows through `require_opaque_legacy_identity`, so the workflow's inline `71b3243…`/`105` are not trust-bearing — they must equal the pinned `LEGACY_INDEX_SHA256`/`LEGACY_INDEX_SIZE_BYTES` (verified identical) or it fails closed, and the evidence schema independently pins `"size_bytes": {"const": 105}`. `materialize_bootstrap_evidence:387-401` fails closed in both directions (attested requires `recovery`; exact bytes forbid it) and restricts the attested path to `preview-index`.

**Provenance / governance.** `_validate_recovery` enforces the exact seven fields, positive ints, non-empty initiator, `validate_approval_policy`, approvers bound to the recovery initiator, and `site_run_id != failed_site_run_id`. Schema `$defs/recovery` mirrors this with `additionalProperties: false`, and the top-level `else` branch adds `{"not": {"required": ["recovery"]}}` so non-final stages reject it. `publication/site-run.json`'s `.run_id` is written by `poll_site_release_run.sh:84-98` only on `conclusion == "success"`, so `site_run_id` is genuinely the recovered run. Waiver `plans/releases/single-maintainer-approval-waiver.json` digest `b9630cc0…` matches `WAIVER_SHA256` in both jobs, `allowed_operations` includes `bootstrap-index`, and `expires_at` `2026-08-27T00:00:00Z` matches all three documented deadlines. All five pinned site-workflow values (`SITE_WORKFLOW`, `_REF`, `_RULESET_ID`, `_RULESET_UPDATED_AT`, `_SHA256`) are byte-identical to `release-publication.yml:145-149`.

**Recovery workflow.** Top-level `contents: read`; only `recover` gets `contents: write` behind `environment: stable-release`; `concurrency: sifr-release-index` with `cancel-in-progress: false`. The single `download-artifact` is intra-run (prepare→recover) and digest-gated by `EXPECTED_RECOVERY_SUMMARY_SHA256`; it does not reach for the original run's expiring artifact. The input re-grep heredoc terminates correctly after YAML block-scalar dedent, and the loop's `exit 2` is not subshelled. `channels` asset listing gates on `schema-v2-bootstrap-generation-1.json` absence, preserving write-once. `.sha256` sidecars contain a bare digest (`build_release_artifacts.sh:126-133`), so `fetch_schema_bootstrap_beta.sh:83`'s `tr -d '[:space:]'` comparison is correct.

**Live state.** Original run `30443929353`: `workflow_dispatch`/`preview-release.yml`/`head_sha 94a5fec6…`/attempt 1/`completed`+`failure`. Failed site run `30445065348`: `release-site.yml`/`head_sha ff472f2a…`/`display_title "Sifr site release 30443929353-1"`/`completed`+`failure`. `channels` assets are still only `channels.json`, `channels-generation-1.json`, `schema-v2-bootstrap-alpha-0.1.0-alpha.2.json` — final evidence absent, write-once path open. Live `channels.json` and `channels-generation-1.json` both hash `04edacb8…`, matching every documented value. Durable summary hashes `f45c012c…` as documented, loads under `load_json_strict(require_canonical=True)`, has the 9 expected asset keys, and pins `current_index_sha256` to the legacy digest.

**Checks rerun (all pass).** `schema_bootstrap_selftest` → PASS; `cases/schema_epoch_bootstrap_workflow_contract.sh` → exit 0; distribution area `--suite full` → 67/0 and unfiltered → 125/0; `scripts/check_file_size_guardrails.py` → PASS (2959 files, limit 900); YAML parse of `schema-bootstrap-recovery.yml` clean.

## Non-blocking observations

1. `expect_failure` asserts only that a `GovernanceError` was raised, not which one, so the 10 recovery mutations + 6 partial-identity + 5 boundary cases could in principle pass for an incidental reason. Pass 2 instrumented this manually; a message assertion would make it durable in-suite.
2. `recovery_evidence()` fixture uses `distinct-reviewer`/`none`, while the live run will emit `single-maintainer-waiver` with a real waiver digest. That mode is only covered by out-of-suite simulation.
3. On read-back, `recovery` is merely optional for `preview-index` — `legacy_index` is const-pinned identically on both paths, so a standalone validator cannot re-prove that an attested-path payload carried provenance. The required-when-attested direction lives only in the materializer. Correct by construction, worth knowing.
4. The input re-grep depends on the failed site run's Actions **log**, whose retention (~2026-10-26) isn't listed alongside the two recorded deadlines. Later than the binding 2026-08-27 waiver expiry, so not a live risk.
5. The waiver has no `bootstrap-index-recovery` operation, so recovery reuses `--operation bootstrap-index` for both approval resolutions. Defensible — recovery completes that operation and mutates nothing — but the waiver's scope wording doesn't distinguish them.
6. `schema_bootstrap.py` and `schema_bootstrap_selftest.py` include cosmetic line reflows unrelated to the remediation; no lint lane in `run_all_tests.sh` requires them.
