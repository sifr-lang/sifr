## Review: Phase 40 · milestone_40_3 — Rollback and Incident Governance

**HEAD reviewed:** `cc87f1e79a2d11c2f2cd1fba8b99d470741c82da`
**Range:** `db80dd35e056b9dcc9a2ac64475a198f5c36bfaa..cc87f1e79` (4 commits, 35 files, +3413/−71)
**Prior artifacts read:** phase definition `plans/phases/40_...md:704-838`; archived passes 1, 2, 3.
Read-only throughout: no file written, no git state changed.

### Checks run

| Check | Result |
|---|---|
| `distribution_release --suite incident-governance` | pass, 9/9 scenarios |
| `--suite full --suite qualification --suite evidence-custody --suite incident-governance` | pass, 55 variants, 0 failures, incident module runs **exactly once** |
| `coverage_matrix` (advisory + readiness + profile-assignment) | pass, 5 variants |
| `sifr_verify` runner `selftest.run_all()` | 11/11 self-tests pass |
| `demos/stable_incident_recovery_demo.sh` | exit 0, all four scenes (gen 21 burned → 22 realized; forced downgrade + out-of-band both land `0.1.0`; first-GA roll-forward → stable `0.1.1`, `0.1.0` withdrawn) |
| `scripts/check_file_size_guardrails.py` | PASS (2867 files, limit 900; largest touched: `selftest.py` 881, `incident_recovery_selftest.py` 872, `incident_fixture.py` 809) |
| Repro of README-described incident layout | request + `withdrawal-evidence.txt` **accepted** by `validate_incident_directory` and `validate_changed_path_set`; request-only rejected |
| Stale-path sweep `plans/incidents/` outside review artifacts | 0 hits |
| Diff scope | no `crates/**`, no `.github/workflows/**`; no Rust-interop implementation touched |

### Pass-3 finding — re-check

**RESOLVED.** `plans/releases/README.md:9-11` now states the incident directory contains the request *and its digest-bound `withdrawal-evidence.txt`*, with the sign-off added later by the protected workflow. That is exactly what is enforced (`evidence_custody.py:224-242` requires both files and cross-checks `withdrawal.evidence_sha256`; `incident_evidence.py:31-72` permits exactly those two added paths) and matches `internal_docs/stable_incident_response.md:52-56`, `internal_docs/distribution_pipeline.md` ("Stable Incident Recovery"), and the issue plan. Verified by direct repro above — the layout the README describes now passes; the previously-described request-only layout is the one that fails.

### Pass-1 and pass-2 findings — re-check

All eight remain resolved at this HEAD: canonical `plans/releases/incidents/<id>/` custody wired into `run_evidence_custody_checks` via `validate_committed_incident_addition` (`evidence_custody.py:33,121-143`); sole-first-GA rollback rejection driven directly (`incident_recovery_selftest.py:159-169` → `release_index.py:245-251`); file-size cap met by decomposition into `incident_index_selftest.py`, not blank-line deletion; sign-off schema/runtime parity (`stable_incident_signoff.schema.json:24-34,90` with real `contains`/`minContains`/`maxContains` support at `json_schema_202012.py:208-218`, plus two negative schema-contract cases; runtime `incident.py:132-138` is strictly tighter); `incident-governance` selected by merge/nightly/release and mirrored in `profile_assignment_matrix.json`, `release_report.py:41-46`, `qualification_fixture.py`, `sifr_verify/selftest.py`, with de-duplication at `runner.py:48-54,119-123,138-144` confirmed empirically in both directions.

### Fresh full review

- **Definition of done.** Fresh install / working-client `--force` self-update / broken-client out-of-band all resolve the governed active stable and execute the digest-verified immutable installer (`incident_recovery_selftest.py:89-130`). Sole-stable withdrawal rejected; roll-forward adds the qualified successor and withdraws atomically in one generation. Withdrawn versions unselectable by channel (`release_index.py:57-58`) or exact pin (pre-existing `install_withdrawn_stable_rejected.sh`). Racing rollback fails closed as `stale-generation` with the newer generation intact (`:259-273`). Immutable request authorizes each mutation; sign-off records request digest, per-attempt run/mode/approver/status/mutations, previous+realized generation/digest, site reconciliation, validation, communications, closure, cross-validated against the request (`incident_fixture.py:611-668`, `incident.py:172-177`). Retention proven by `tree_digest` invariance plus retained gen-21 *and* gen-22; every write is `O_EXCL`.
- **Atomicity/immutability.** `validate_incident_index_mutation` (`release_index.py:171-227`) is the single guard for both planners and is exercised on 10 distinct rejection branches by the dedicated module.
- **Resume/race semantics.** Pre- and post-reservation staleness rechecks around the write-once snapshot (`incident_fixture.py:158-190`); post-index site timeout is terminal and lease-releasing (`metadata_lease` `finally`), resumes with byte-identical live index, **no** gen-22 snapshot, no second index mutation, and a new correlated site attempt (`incident_recovery_selftest.py:144-153`); generation allocation burns over all retained snapshots (`:422`).
- **Schema/runtime parity.** Request and sign-off schemas both pin `schema_version: 2`; `operation` present on both sides; the rollback-target `if/then/else` matches `require_exact_keys`.
- **Local-only boundary.** Temp-root confinement, symlink rejection, credential refusal, non-deploying site marker, no `socket`/`urllib`/`requests`/`subprocess` in the harness, no `gh release`/`vsce publish`/`repository_dispatch` in the CLI, and `release-publication.yml` exposes neither operation (unchanged in range).
- **Docs/demo/scope/evidence.** Runbook states owners, non-initiating approval authority, 30-minute acknowledgement target explicitly exceeding the 20-minute site wait, triggers, communication locations, retry matrix, retention, closure. Demo filename is capability-based. Issue plan and architecture/pipeline docs match the code. The create-PR host-timing shortfall is disclosed honestly in the pre-existing `plans/issues/active/adhoc_performance_budget_host_variance.md` with no baseline or waiver change.

### Actionable findings

None.

Two non-actionable observations, recorded only for completeness: `plans/releases/README.md:13-14` describes evidence changes as never mixing with documentation changes, while `evidence_custody.py:63-70` permits the README itself alongside *candidate* evidence — the doc is stricter than the check, so following it can never fail a gate. And `incident_fixture.py:353-364` closes the lease descriptor twice (guarded by `except OSError`); no fd is live at `finally` time in this single-threaded harness, so it is latent style only.

### Verdict

**APPROVED**
