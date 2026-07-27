## Review: Phase 40 · milestone_40_3 — Rollback and Incident Governance

**HEAD reviewed:** `32852b72fcbec6c3a8cd107c708656c902de8770`
**Range:** `db80dd35e056b9dcc9a2ac64475a198f5c36bfaa..32852b72f` (3 commits, 33 files, +3344/−69)
**Prior artifacts inspected:** `plans/reviews/archive/phase-40-milestone-40-3-claude-opus-review-pass-1.md`, `...-pass-2.md`

### Checks run (read-only)

| Check | Result |
|---|---|
| `distribution_release --suite incident-governance` | pass, **9/9** scenarios |
| `distribution_release --suite full` | pass, 53 variants; incident module runs once (9 scenarios) |
| `--suite full --suite qualification --suite evidence-custody --suite incident-governance` | pass, 55 variants, 0 failures, **no duplicate** incident execution |
| `coverage_matrix` (advisory + readiness + profile-assignment) | pass, 5 variants, `profile assignment matrix ok: rows=17` |
| `sifr_verify` runner `selftest.run_all()` | all 11 self-tests pass (incl. release-report production) |
| `demos/stable_incident_recovery_demo.sh` | exit 0, all four scenes |
| `scripts/check_file_size_guardrails.py` | PASS (2867 files, limit 900) |
| Instrumented branch trace of `test_incident_index_mutation_contract` | 12 rejections, each on a **distinct intended** `validate_incident_index_mutation` branch |
| PEP 8 top-level separator scan (7 governance modules) | 0 violations |
| Repro: README-described incident layout | `REJECTED: withdrawal-evidence.txt is required` (finding 1) |
| Diff scope | no `crates/**`, no `.github/workflows/**`; workflow has **zero** occurrences of `rollback`/`incident` |

### Pass-2 findings — re-check

1. **`validate_incident_index_mutation` coverage — RESOLVED, genuinely.** New `incident_index_selftest.py` (168 lines) is registered in `incident_recovery_selftest.py:50` and owned by the incident module, so the issue-plan relocation claim is now true. I instrumented `assert_rejected` and confirmed the 12 cases land on 10 distinct `fail()` sites, not on the generic index validator: removed affected (`release_index.py:197`), affected not withdrawn (`:199`), affected bytes altered (`:208`), successor not active (`:212`), stable not pointing at successor (`:214`), rollback version-set drift (`:219`), roll-forward version-set drift both directions (`:222`), retained-release byte drift (`:227`), non-stable channel drift (`:194`), affected not the live stable predecessor (`:189`). Atomicity and immutability guards are exercised. The only uncovered branch is `:191` (`ga_status` regression), which is unreachable — `validate_release_index_transition:122` rejects active→preview first. Not actionable.
2. **PEP 8 separator — RESOLVED.** `selftest.py:496-497` restored; scan of all touched governance modules shows no remaining single-blank top-level separators. `selftest.py` is 881 lines, `incident_recovery_selftest.py` 872, `incident_fixture.py` 809 — all under the cap without style debt.
3. **README custody policy — RESOLVED in code, internally consistent, and tested.** `evidence_custody.py:63-70` now allows `plans/releases/README.md` only when every evidence path is a candidate path, so `validate_changed_path_set` and `validate_incident_evidence_commit` can no longer disagree. Positively tested at `selftest.py:831` (candidate + README accepted) and negatively at `incident_recovery_selftest.py:356-365` (incident + README rejected with `cannot mix`).

### Pass-1 findings — re-check

1. Custody contract/enforcement — **resolved** (canonical `plans/releases/incidents/<id>/`, `validate_incident_evidence_commit` wired through `validate_committed_incident_addition`, `evidence_custody.py:33,121-143`).
2. Sole-first-GA rollback rejection — **resolved**, direct case at `incident_recovery_selftest.py:159-169` drives `propose_rollback` against the one-stable-version index and asserts `release_index.py:245-251`.
3. File-size guardrail — **resolved by decomposition**, not deletion (see above).
4. Schema/runtime parity — **resolved**; `started` gone from the attempt enum, `contains`/`minContains`/`maxContains` (schema `:28-34`) backed by real validator support (`json_schema_202012.py:208-218`) and two negative schema-contract cases. Runtime residual (`run_id` monotonicity, last-attempt-completed at `incident.py:132-138`) is not expressible in JSON Schema 2020-12.
5. Profile selection/de-duplication — **resolved**; `incident-governance` selected by merge/nightly/release, mirrored in `profile_assignment_matrix.json`, `release_report.py:41-46`, `qualification_fixture.py`, and the runner selftest. De-dup verified empirically in both directions (`full` alone → 9 scenarios; `full` + named suite → still exactly one `incident-recovery` case).

### Independently verified this pass

- **Definition of done.** Fresh-install / working-client `--force` self-update / broken-client out-of-band recovery all execute the digest-verified immutable installer to `0.1.0` (`:127-130`). Sole-stable withdrawal rejected; roll-forward activates the qualified successor and withdraws atomically. Withdrawn versions unselectable by channel or pin (`validate_release_index:57-58` plus the pre-existing `install_withdrawn_stable_rejected.sh`). Racing rollback fails closed as `stale-generation` with the newer generation intact (`:259-273`). Immutable request authorizes every mutation; sign-off records digest/attempts/approvers/mutations/site/validation/communications/closure and is cross-validated against the request (`incident.py:172-177`). Retention proven by `tree_digest` invariance plus retained gen-21 **and** gen-22 (`:72-82`); every write is `O_EXCL`.
- **Exact resume/race.** Reservation-failure burns 21 and resumes at 22; post-index site timeout resumes with byte-identical live index, **no** gen-22 snapshot, and no second index mutation (`:144-153`); pre- and post-reservation staleness rechecks (`incident_fixture.py:158-189`); terminal lease-releasing timeout via the `metadata_lease` `finally`.
- **Local-only boundary.** Temp-root confinement, symlink rejection, credential refusal, non-deploying site marker, `run_incident_fixture.py` has no network/subprocess adapter, and `release-publication.yml` contains no incident surface at all.
- **Scope.** No `crates/**` change; no Rust-interop implementation touched. Demo filename `stable_incident_recovery_demo.sh` is capability-based with no phase/milestone identifier. The create-PR host-timing shortfall is disclosed in the pre-existing `adhoc_performance_budget_host_variance.md` follow-up with no baseline or waiver change.

### Actionable finding

**1. LOW — `plans/releases/README.md` still describes the pre-milestone incident-directory contract and now contradicts an enforced check.**

The README (`plans/releases/README.md:10-11`) states the incident directory "contains one incident request. A later protected incident workflow may add its matching sign-off." This milestone made `withdrawal-evidence.txt` a **required** co-located file (`evidence_custody.py:26,235-237`, `incident_evidence.py:33-36`), and the README was last modified at `dd58b9706`, before this range — so the divergence is introduced here. Both accurate descriptions live elsewhere (`internal_docs/stable_incident_response.md:52-53`, `internal_docs/distribution_pipeline.md`), but this is the file that sits next to the evidence tree and is itself the path specially allowed by `validate_changed_path_set`.

Reproduced against the exact layout the README describes:

```
REJECTED: .../inc-2026-001: withdrawal-evidence.txt is required
```

Failure scenario: an incident owner follows the README, opens an evidence-only PR containing only `stable-incident-request.json`, and the `distribution_release:evidence-custody` suite fails during an active stable incident. One sentence in the README closes it.

### Verdict

**NOT APPROVED** — the implementation, tests, docs elsewhere, and all eight prior findings are fully resolved and independently verified; the sole remaining item is finding 1, a one-line correction to `plans/releases/README.md` so the custody documentation matches the contract this milestone now enforces.
