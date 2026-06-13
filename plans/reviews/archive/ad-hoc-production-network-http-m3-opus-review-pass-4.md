# Claude Opus M3 final branch-tip review pass 4

Scope: final branch-tip review of `codex/network-http-m3-url-header-cookie` at head `1ccb3b17c9799bf8dd6cd96f5d57a3c414800377` for PR #2497 (M3 URL, Header, And Cookie Primitives), focused on whether any blocking issue remains before merge.

Inputs verified:
- Phase ledger `issues/ad-hoc-production-network-http-platform-substrate-execution.md`.
- M3 traceability `verification/stdlib/network_http_m3_url_header_cookie_traceability.md`.
- Prior review artifacts pass 2 (FAIL) and pass 3 (PASS).
- Implementation under `crates/sifr_codegen/src/preamble/url_http_runtime.rs`, intrinsic registry under `crates/sifr_codegen/src/intrinsics/registry`, public surface under `lib/sifr/url.sifr` and `lib/sifr/http.sifr`, fixtures `crates/sifr/tests/e2e/pass/network_http_m3_url_query_percent.sifr` and `crates/sifr/tests/e2e/pass/network_http_m3_header_cookie.sifr`, dependency snapshot tests under `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs`.
- Local validation reports `target/validation_lane_reports/create-pr.latest.json` and `target/validation_lane_reports/merge.latest.json` (plus matching `.log` and `.time` files).

---

## Verdict: **FAIL**

The implementation, fixtures, traceability prose, and dependency snapshots all match the pass-3 expectations; the two commits added since pass 3 are doc-only and consistent with the rest of the ledger.

However, the **merge-gate evidence cited in the ledger and the M3 traceability does not match the actual report on disk**. The ledger and traceability both cite `target/validation_lane_reports/merge.latest.json` as PASS with "advisory: high e2e group skew only", but the file on disk shows that the latest `scripts/run_all_tests.sh` run **failed** and bailed out early. M1/M2 set the bar that the cited merge-gate report path must reflect the actual passing run for that milestone, and that contract is broken here.

---

## B1 — Merge-gate evidence on disk contradicts the ledger PASS claim

**Ledger claim (`issues/...execution.md:356`, traceability `:32`, both added in commit `368450b76`):**

> `scripts/run_all_tests.sh` | PASS | Full merge-gate validation passed; report `target/validation_lane_reports/merge.latest.json`; advisory: high e2e group skew only.

**Actual on-disk report `target/validation_lane_reports/merge.latest.json` (mtime `Jun 12 09:24`, i.e. written after the ledger commit at 09:22:13):**

- `lane_steps[4].name = "performance_budget_checks"`, `lane_steps[4].status = "fail"`.
- Only 5 of the expected merge-gate lane steps ran. `crate_tests`, `platform_golden`, `e2e_pass_suite`, `generated_code_quality_checks`, `validation_contract_matrix`, `verification_hardening_*`, `distribution_validation`, and `extra_e2e_checks` are absent from `lane_steps` (compare to the `create-pr.latest.json` peer report, which records all 14 lane steps PASS in its `lane_steps`).
- `e2e: null`, `hardening_summary: null`, `contract_suites: []`, `observations: {cache_hit_rate: null, group_skew_ratio: null, rebuild_groups: null}` — the merge gate never reached e2e, so the cited "advisory: high e2e group skew" cannot be from this run.
- `advisories: []` — zero advisories recorded, not the cited "high e2e group skew".
- `time.real_seconds = 130.58` — far below the ~800s a complete merge gate has historically taken on this branch family (M2 recorded `799.89s` for the same script).

**Failure detail from `target/validation_lane_reports/merge.latest.log`:**

```
performance budget error: budget check failed:
phase27-non-regression-002-json-diagnostic-schema (perf.phase27.json_diagnostic_schema)
  p95_ms regression: measured=1774.831 threshold=1487.2 waiver_status=no_waiver
[sifr-lane-step] name=performance_budget_checks elapsed_ms=75974 status=fail
```

This is a real budget regression (~19% over threshold) with `waiver_status=no_waiver`, and the script then aborted before running the remaining merge-gate lane steps.

So either (a) an earlier successful merge run was overwritten on disk by this aborted run, in which case the ledger's `report` path no longer points to the cited evidence, or (b) the PASS row was written before the merge gate finished, and the merge gate then failed. In either case, the ledger and traceability currently assert PASS evidence that does not exist at the cited path.

This matters because:

- M1 and M2 both cite specific head SHAs with their merge-gate report paths, and reviewers (and future M4 readers) treat the cited `merge.latest.json` path as the authoritative evidence file. M3 currently breaks that invariant.
- The M0 row already established the precedent for handling a transient performance p95 failure (run targeted representative performance + full merge-gate rerun, then record the rerun PASS). M3 does not record any such rerun pattern, so the current ledger reads as "merge gate passed cleanly" when the actual on-disk evidence says otherwise.
- `waiver_status=no_waiver` means this is not an accepted pre-existing regression — it needs either a passing rerun (if transient) or a documented investigation/waiver before merge.

**Remediation (one of):**

1. Rerun `scripts/run_all_tests.sh` to a clean PASS, confirm `merge.latest.json` on disk is the PASSing report, and update the M3 row in `issues/...execution.md` and `verification/stdlib/network_http_m3_url_header_cookie_traceability.md` to reflect the actual advisories from that run (the current "high e2e group skew" claim is not in the on-disk report; the create-pr report shows only "warm wall-time budget exceeded").
2. If the `phase27-non-regression-002-json-diagnostic-schema` p95 regression turns out to be a real, reproducible regression on this branch (rather than the same kind of transient outlier M0 recorded), it must be investigated and either fixed on-branch or explicitly waived with rationale before merge — silently shipping a `no_waiver` budget failure is not acceptable.
3. Once a clean rerun is recorded, mirror the M0 ledger pattern explicitly: "first merge-gate run failed on a transient performance p95 outlier in `phase27-non-regression-002-json-diagnostic-schema`, targeted representative performance rerun and full merge-gate rerun passed", with the rerun report path cited and the head SHA recorded.

---

## Implementation/contract findings: **none new**

The pass-3 verdict still holds for the code, fixtures, traceability prose, and dependency snapshots. I re-checked the items that were in scope for pass 4:

- IDNA guard at `crates/sifr_codegen/src/preamble/url_http_runtime.rs:20-46` percent-decodes the extracted authority host and rejects any decoded byte `>= 0x80` ahead of `url::Url::parse`; `__sifr_url_validate_ascii_host` at `:49-70` repeats the decoded-byte check defensively. Fixture `network_http_m3_url_query_percent.sifr:43-46,80-86` locks `%C3%A9` rejection, `%61` acceptance, and `%2F` path preservation.
- Path-normalization traceability at `verification/stdlib/network_http_m3_url_header_cookie_traceability.md:10` correctly records "WHATWG dot-segment removal for special schemes" and "`%2F` is preserved as a segment byte, not a separator". The fixture covers both behaviors.
- Inventory caps (`__SIFR_URL_MAX_BYTES`, `__SIFR_QUERY_MAX_BYTES`, `__SIFR_HEADER_NAME_MAX_BYTES`, `__SIFR_HEADER_VALUE_MAX_BYTES`, `__SIFR_HEADER_SECTION_MAX_BYTES`) match the M0 inventory.
- Cookie value classifier `__sifr_http_is_cookie_value_byte` admits `=` (0x3D) and rejects `;` (0x3B); fixture at `network_http_m3_header_cookie.sifr:40-42,73-79` locks both.
- `requirements.rs` pairs URL/HTTP intrinsics with the right preamble dependencies for both explicit names and the `url_`/`http_` prefix fallback.
- `url_http_runtime.rs` is 471 lines, well under the 900-line file-size cap. `python3 scripts/check_file_size_guardrails.py` rerun: PASS (2319 files, limit 900 lines).
- `git diff --check origin/main..HEAD`: PASS (no output).
- Doc-only commits since pass 3 (`368450b76`, `1ccb3b17c`) are consistent with the ledger and traceability; the only concern is that their PASS claims about the merge gate are not backed by the on-disk evidence — see B1.

---

## Non-blocking observations

1. The two doc-only commits since pass 3 also update the milestone status banner (`issues/...execution.md:5`) to "M3 ... PR is merge-validated" and the traceability status (`verification/...m3_url_header_cookie_traceability.md:3`) to "implementation candidate merge-validated". Both are tied to the same merge-gate evidence cited in B1; once B1 is remediated, these strings remain accurate. If the merge gate is rerun and passes, no edit is required to these two status lines.

2. The M3 PR link in `issues/...execution.md:260` is recorded as "ready for merge after local merge-gate validation". After remediation of B1, this line is the correct closure phrasing for the PR ready state.

3. The milestone checklist box at `issues/...execution.md:24` (`[ ] milestone_network_http_3`) is still unchecked. That is correct for the pre-merge state; it should be checked only after the PR merges (matching the M1/M2 pattern where the box flips together with the merge-commit ledger entry).

---

## Bottom line

The M3 implementation, fixtures, traceability prose, and dependency snapshots are all acceptable to merge — the pass-3 remediations hold and no new code/contract blockers were introduced by the two doc-only commits added since pass 3.

The blocking finding is purely evidentiary: the ledger and the M3 traceability cite `target/validation_lane_reports/merge.latest.json` as a PASS report, but that file on disk currently records a FAILED, prematurely-aborted merge gate (`performance_budget_checks` p95 regression, `waiver_status=no_waiver`, only 5 of ~14 lane steps executed, zero advisories, no e2e). PR #2497 should not merge until either a clean rerun of `scripts/run_all_tests.sh` is recorded with the on-disk `merge.latest.json` reflecting the cited PASS (and matching the cited advisory), or the failure is explicitly waived with rationale following the M0 transient-performance-outlier precedent.

Once B1 is closed, PR #2497 is acceptable to merge.
