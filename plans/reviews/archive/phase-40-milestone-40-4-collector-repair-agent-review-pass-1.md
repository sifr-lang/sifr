## Review — Phase 40 milestone_40_4 qualification collector repair (`origin/main...HEAD`, commit `b9a06bda3`)

Read-only. Scope: 5 files, +57/−9.

### What I verified

**Root cause and fix are correct.** Both repairs address real GitHub REST semantics rather than symptoms:

- Binding `run_metadata["path"] == ".github/workflows/release-qualification.yml"` replaces the `name` check. `name` is the rendered `run-name` (`release-qualification.yml:2` interpolates `inputs.version`/`inputs.source_commit`), so the old equality could never hold. The repo already relied on this semantics elsewhere — `release-publication.yml:651` correlates runs on `display_title` — so the collector's `name` binding was the outlier. `path` is the immutable workflow identity, and it is a strictly stronger binding than the mutable `name:` key (which no contract test pinned anyway).
- The one-sided retention interval matches the stated anchor asymmetry (expiry anchored at upload start, `created_at` stamped at completion), which is why the delta lands 1–9 s under 30 d.

**Fail-closed behavior confirmed empirically.** I drove `collect_index` directly against the fixture with mutations:

| probe | result |
|---|---|
| baseline (30 d − 5 s) | accepted |
| `path` = other workflow / basename only / absent | rejected |
| shortfall 60 s / exactly 30 d | accepted |
| shortfall 61 s | rejected |
| 30 d + 1 s, 31 d, 29 d | rejected |

The bound is genuinely one-sided: no over-retention is tolerated, not even one second, and a missing `path` key fails closed via `.get()`. Retention inflation and truncation both abort collection.

**Provenance chain intact.** `head_sha == source_commit`, `event == workflow_dispatch`, `id == GITHUB_RUN_ID`, `run_attempt`, and `repository.full_name` are still all bound, and `validate` enforces `github.sha == source_commit` before the contract case runs on that exact tree, so `path` resolves to the contract-checked workflow definition. The workflow contract still enforces `retention-days: 30` and `overwrite: false` exactly four times (`release_qualification_workflow_contract.sh:56,119`), so the loosened API-delta check remains a *derivation* backed by an independent config assertion, not a replacement for it. Permissions stay `contents: read` / `actions: read`; no mutation capability introduced.

**Downstream unaffected.** `artifact_index.py:99` still requires `retention_days == 30`; per-artifact `expires_at ≥ workflow boundary` and `require_unexpired` are unchanged. The 60 s slack does not leak into any custody window, since `expires_at` (not `created_at + 30d`) is what is recorded and later compared.

**Tests/docs.** `governance.qualification_selftest` passes (9 tests), plus `governance.selftest` (14), `evidence_custody`, `schema_epoch`, and the workflow contract case. Fixtures now carry realistic skew and a realistic dynamic `name`. `internal_docs/distribution_pipeline.md:223-228` and the plan ledger describe the mechanism accurately; the 6-container / 20-row replay arithmetic checks out (4×4 + 2 + 2). Touched files are all under the 900-line cap.

### Actionable finding

**1 · Low · test-coverage — the over-retention direction of the new bound lost its probe**

- **Location:** `verification/areas/distribution_release/governance/qualification_selftest.py:210` (was `2098-12-01T00:00:00Z` = 31 d, now `2098-12-02T00:01:01Z` = 61 s shortfall); guard at `scripts/distribution/collect_qualification_artifacts.py:161-170`.
- **Impact:** The exact-equality check previously had one negative probe, and it exercised the *long* direction — the archived pass-3 ledger credits it as "Probes: 31d, 29d, …". The repair repurposed that single probe for the shortfall direction, so `observed_retention <= ARTIFACT_RETENTION` is now asserted nowhere. Deleting the upper comparison entirely (leaving `ARTIFACT_RETENTION - SKEW <= observed_retention`) keeps the whole suite green while the collector would silently accept 60- or 90-day retention — i.e. the governance property whose loss the milestone is auditing for regresses undetected. Present behavior is correct; only the regression barrier is gone, hence low.
- **Remediation:** Keep the 61 s case and add one more mutation in `test_artifact_collector_rejects_drift` asserting rejection of retention longer than 30 days (e.g. `created_at = "2098-12-01T23:59:59Z"`, restoring to `2098-12-02T00:00:05Z` afterward as the existing block already does). I confirmed that input rejects today, so the assertion passes as written. ~15 lines; the file lands at ~855 lines, still inside the cap.

### Non-blocking observations (no change required)

- `collect_qualification_artifacts.py:167-169` omits the observed interval from the error. Since this failure aborts a four-runner multi-hour qualification run, including `observed_retention` and the two timestamps would separate "GitHub upload was slow" from "retention was misconfigured" without a re-run. The 60 s ceiling is ~7× the worst observed 9 s, which I consider appropriately sized rather than over-permissive.
- `qualification_fixture.py:556` hardcodes `0.1.0` in the cosmetic `name` while the version is otherwise carried in `prefix`; the field is unvalidated, so this is realism-only.
- All six fixture containers share one `created_at`, so heterogeneous per-container skew (the real 1–9 s spread) is not exercised. Pre-existing fixture shape; the guard is per-artifact, so coverage is not affected.

**NOT APPROVED**
