## Review: Phase 40 · milestone_40_3 — Rollback and Incident Governance (PR #3032, PR head)

**Exact HEAD reviewed:** `e42bb9a3d4fb48ae3ba50fc9209aa2e8cd5c10d7`
**Base:** `db80dd35e056b9dcc9a2ac64475a198f5c36bfaa`
**Range:** 6 commits, 36 files, +3468/−71. One commit in the range (`eba76bd72`, docs-only) is the carried-over milestone_40_2 closure (issue-plan entry + archived 40‑2 pass‑9 artifact); milestone_40_3's own delta is `eba76bd72..e42bb9a3d` = 35 files, +3413/−70.
**Artifacts read:** phase definition `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md:704-836`; archived passes 1–4.
Read-only throughout — no file written, no git state changed (`git status` shows only the pre-existing untracked pass-5 artifact).

### Checks run

| Check | Result |
|---|---|
| `distribution_release --suite incident-governance` | pass, 9/9 scenarios |
| `--suite full --suite qualification --suite evidence-custody --suite incident-governance` | pass, 55 variants, 0 failures, incident module runs **exactly once** |
| `coverage_matrix` (advisory + readiness + profile assignment) | pass, 5 variants, `profile assignment matrix ok: rows=17` |
| `sifr_verify` runner `selftest.run_all()` | 11/11 self-tests pass |
| `demos/stable_incident_recovery_demo.sh` | exit 0, all four scenes (gen 21 burned → 22 realized, stable `0.1.0`; forced downgrade and out-of-band both land `0.1.0`; first-GA roll-forward → stable `0.1.1`, `0.1.0` withdrawn under `inc-forward-001`) |
| `scripts/check_file_size_guardrails.py` | PASS (2867 files, limit 900; largest touched `selftest.py` 881, `incident_recovery_selftest.py` 872, `incident_fixture.py` 809) |
| Independent custody repro (fresh temp tree) | request-only **rejected**; request + `withdrawal-evidence.txt` **accepted** by both `validate_incident_directory` and `validate_changed_path_set`; `+README.md`, `+internal_docs/*`, second incident id each rejected with the right message |
| PEP 8 top-level separator scan (10 touched modules) | 0 violations |
| Stale-path sweep `plans/incidents/` outside review artifacts | 0 hits |
| Diff scope | 0 files under `crates/**` or `.github/**`; `release-publication.yml` contains 0 occurrences of `rollback`/`incident` |

### Pass-4 delta — verified

The only change after the pass-4-approved implementation head `cc87f1e79` is the archived pass-4 report plus the execution-tracker approval entry. Both are accurate: the report's `35 files / +3413/−71` matches `git diff --shortstat db80dd35e..cc87f1e79` exactly, its verdict is `APPROVED` with `### Actionable findings — None.`, and the tracker entry at `plans/issues/active/phase-40-stable-channel-ga-execution.md:489-496` reproduces the same head SHA and check list without overclaiming. No contradiction with passes 1–3 (each is recorded as not-approved with its findings mapped to the remediation that landed) and none with the code. The tracker correctly leaves milestone_40_3's "Record review rounds, PR, validation, and merge" unchecked. One cosmetic slip: the pass-4 header says "4 commits" where the range holds 5 (it excluded the carried 40‑2 docs commit); file/line counts and the reviewed SHA are correct.

### Independent re-verification of the implementation

- **Definition of done.** Fresh install, working-client `--force` self-update, and broken-client out-of-band recovery all resolve the governed active stable and execute the digest-verified immutable installer (`incident_recovery_selftest.py:89-130`). Sole-first-GA rollback is rejected directly against the one-stable-version index (`:159-169` → `release_index.py:245-251`); roll-forward adds the qualified successor and withdraws the affected version in one generation (`:176-182`). Withdrawn releases are unselectable by channel (`release_index.py:57-58`) and by exact pin (pre-existing `cases/install_withdrawn_stable_rejected.sh`). A racing rollback fails closed as `stale-generation` with the newer generation intact (`:259-273`). The immutable request authorizes every mutation, and the sign-off records request digest, per-attempt run/mode/approver/status/mutations, previous+realized generation/digest, site reconciliation, validation, communications, and closure, cross-validated against the request (`incident_fixture.py:611-668`, `incident.py:172-177`). Retention proven by `tree_digest` asset invariance plus retained gen‑21 *and* gen‑22; every evidence write is `O_EXCL`.
- **Atomicity/immutability.** `validate_incident_index_mutation` (`release_index.py:171-227`) is the single guard behind both planners; `incident_index_selftest.py` drives its rejection branches (affected removed / not withdrawn / bytes altered, successor inactive, stable not pointing at successor, rollback and roll-forward version-set drift both directions, retained-release byte drift, non-stable channel drift, affected not the live predecessor).
- **Race/resume.** Staleness is rechecked both before and after the write-once generation reservation (`incident_fixture.py:158-189`); post-index site timeout is terminal and lease-releasing (`metadata_lease` `finally`), resumes with byte-identical live index, **no** gen‑22 snapshot, no second index mutation, and a new correlated site attempt (`:144-153`); generation allocation burns over all retained snapshots (`:422`).
- **Schemas/profiles.** Sign-off schema pins `operation`, drops `started` from the attempt enum, and enforces exactly one completed attempt via `contains`/`minContains`/`maxContains` — backed by real validator support (`json_schema_202012.py:205-218`, keywords registered in `SCHEMA_KEYS` and linted) and two negative schema-contract cases. Runtime `incident.py:132-138` is strictly tighter (unique strictly-increasing `run_id`, final attempt completed). `incident-governance` is selected by merge/nightly/release and mirrored in `profile_assignment_matrix.json`, `release_report.py:41-46`, `qualification_fixture.py`, and `sifr_verify/selftest.py`; de-duplication at `runner.py:48-54,119-123,138-144` confirmed empirically in both directions.
- **Local-only safety.** Temp-root confinement (`validate_fixture_root`, `require_fixture_path`), symlink rejection, production-credential refusal, mandatory non-deploying site marker, no `socket`/`urllib`/`requests`/`subprocess` in `incident_fixture.py`, no `gh release`/`vsce publish`/`repository_dispatch` in `run_incident_fixture.py`, and no rollback or roll-forward surface anywhere in `release-publication.yml`. `generate-incident-request` requires a clean work directory outside the repository and binds the exact affected/target plan and evidence digests.
- **Docs/demo/scope/disclosure.** Runbook states owners, non-initiating approval authority, and a 30-minute acknowledgement target explicitly exceeding the 20-minute site wait, plus triggers, communication locations, retry matrix, retention, and closure. `plans/releases/README.md`, `internal_docs/stable_incident_response.md:52-56`, `distribution_pipeline.md`, `architecture.md`, and the issue plan all agree with the enforced contract. Demo filename `stable_incident_recovery_demo.sh` is capability-based. No Rust-interop implementation touched. The create-PR host-timing shortfall (19/19 Python-interop variants pass, aggregate step 788.45 s vs 600 s) is disclosed with specifics in the pre-existing `plans/phases/adhoc_performance_budget_host_variance.md` follow-up, with no baseline or waiver change.

### Actionable findings

None.

Two non-actionable observations, for completeness: `plans/releases/README.md:13-14` forbids mixing evidence with documentation changes while `evidence_custody.py:63-70` permits the README itself alongside *candidate* evidence — the doc is stricter than the check, so following it can never fail a gate. And `incident_fixture.py:356-364` closes the lease descriptor twice (guarded by `except OSError`); no descriptor is live at `finally` time in this single-threaded harness, so it remains latent style.

### Verdict

**APPROVED**
