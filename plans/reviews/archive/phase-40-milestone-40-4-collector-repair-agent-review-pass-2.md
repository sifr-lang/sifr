## Review — Phase 40 milestone_40_4 collector repair, pass 2 (`origin/main...HEAD`, `aeb3b7144`)

Read-only; working tree untouched. Scope: 6 files, +127/−10.

### Pass-1 finding closed

**Low · test-coverage — upper retention bound lacked a mutation → resolved.** `qualification_selftest.py:210` now restores the long-direction probe (`created_at = 2098-12-01T23:59:59Z` = 30 d + 1 s) *in addition to* the 61 s shortfall probe (`2098-12-02T00:01:01Z`), then restores the realistic baseline `2098-12-02T00:00:05Z`.

I mutation-tested the guard by loading source-mutated collectors through `load_collector` — all four mutants are killed by `test_artifact_collector_rejects_drift`:

| mutant | result |
|---|---|
| drop `<= ARTIFACT_RETENTION` | KILLED — "collector accepted retention longer than 30 days" |
| drop `ARTIFACT_RETENTION - SKEW <=` | KILLED — "collector accepted excessive retention shortfall" |
| drop the `path` identity clause | KILLED — "collector accepted another workflow path" |
| widen skew 60 s → 1 day | KILLED — "collector accepted excessive retention shortfall" |

Boundary probes against the fixture: `exactly 30 d`, `30 d − 59 s`, `30 d − 60 s` accepted (20 rows each); `30 d − 61 s`, `30 d + 1 s`, `31 d`, `29 d`, `90 d` rejected. The bound is exactly as documented and strictly one-sided.

**Improved error confirmed:** `… artifact retention is outside the governed 30-day API timestamp bound (created_at=2098-12-01T23:59:59Z, expires_at=2099-01-01T00:00:00Z, observed=30 days, 0:00:01)` — pass-1 observation 1 closed.

### Root cause verified against the live GitHub API

Both semantics are real, not inferred. `gh api …/runs/30270476093` returns `name = "Qualify stable candidate 0.1.0 at 3ebe27bc…"`, `path = ".github/workflows/release-qualification.yml"` — the old `name == "release-qualification"` equality could never hold. Real per-artifact deltas are `−1 s, −1 s, −4 s, −4 s, −6 s, −9 s` (expiry anchored at upload start), all one-sided and ≤ 9 s against the 60 s ceiling. I ran `origin/main`'s collector against the real metadata: it fails at the `name` clause; patching only that clause, it then fails at `retention is not exactly 30 days`. Both repairs are necessary and sufficient.

### Real six-container / 20-row replay reproduced

I downloaded the run's 509 MB of artifacts plus raw `gh api` run/artifact JSON and ran the branch collector end to end: `artifacts=20` across exactly 6 containers (4 targets × 4 files, assemble 2, editor 2), and `validate_qualification_artifact_index` accepts the output. Rows carry the API `expires_at` verbatim (not `created_at + 30 d`), so the 60 s slack cannot widen any custody window. Ledger claim is truthful; job list confirms four targets, editor, and assemble succeeded and only the collect job failed.

### Contract, provenance, security

- Workflow contract still asserts `retention-days: 30` and `overwrite: false` exactly four times each (`release_qualification_workflow_contract.sh:56,119`), so the loosened API-delta check remains a derivation backed by an independent config assertion.
- `path` binding matches the real file; `run-name` interpolation at `release-qualification.yml:2` explains the dynamic `name`. The workflow writes the full raw `gh api` run object, so `path` is present in production input.
- Provenance clauses intact: `id`, `run_attempt`, `head_sha == source_commit`, `event == workflow_dispatch`, `repository.full_name`. Missing `path` fails closed via `.get()`. Permissions remain `contents: read` / `actions: read`; no mutation surface added. `artifact_index.py:99` still pins `retention_days == 30`.
- Docs (`distribution_pipeline.md:223-230`) and the ledger describe the mechanism accurately; the archived pass-1 report's stated scope (5 files, +57/−9) matches `b9a06bda3` exactly.

### Gates run

`distribution_release --suite full`: 54 variants, 0 failures. `--suite qualification`: 9 tests pass. `scripts/check_file_size_guardrails.py`: PASS (touched files 312 / 855 / 829 lines).

### Non-blocking observations (no change required)

- 60 s ceiling is ~7× the worst observed 9 s upload for 124 MB containers. A future much larger artifact could exceed it and abort a run — but that fails closed with a now-precise diagnostic, which is the intended governance posture.
- `qualification_fixture.py:544` still hardcodes `0.1.0` in the cosmetic `name`; the field is unvalidated, realism-only.
- All fixture containers still share one `created_at`, so heterogeneous per-container skew isn't exercised; the guard is per-artifact and the drift test mutates only `artifacts[0]`, so coverage is unaffected.

No actionable finding remains.

**APPROVED**
