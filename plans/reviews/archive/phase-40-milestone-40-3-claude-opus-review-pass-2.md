## Review: Phase 40 · milestone_40_3 — Rollback and Incident Governance

**HEAD reviewed:** `ff8c2664b795221433c1ac387849e3ee6fc8b803`
**Range:** `db80dd35e056b9dcc9a2ac64475a198f5c36bfaa..ff8c2664b` (3 commits, 31 files, +3086/−69)
**Pass-1 artifact inspected:** `plans/reviews/archive/phase-40-milestone-40-3-claude-opus-review-pass-1.md`

### Checks run (read-only)

| Check | Result |
|---|---|
| `distribution_release --suite incident-governance` | pass, 8/8 scenarios |
| `distribution_release --suite full` | pass, 53 variants, incident module runs (8 scenarios) |
| `distribution_release --suite full --suite qualification --suite evidence-custody --suite incident-governance` | pass, 55 variants, 0 failures, **no duplicate** incident execution |
| `coverage_matrix/runner.py` (advisory + readiness + profile-assignment) | pass, 5 variants, `profile assignment matrix ok: rows=17` |
| `sifr_verify.selftest` | exit 0 |
| `schema_contracts.validate_schema_contracts()` | pass (new negative sign-off cases) |
| `demos/stable_incident_recovery_demo.sh` | exit 0, all four scenes |
| `scripts/check_file_size_guardrails.py` | PASS (2866 files, limit 900) |
| Diff scope | no `crates/**`, no `.github/workflows/**` change → no Rust-interop implementation, no production adapter |

### Prior findings — re-check

1. **Custody contract / repository enforcement — RESOLVED.** Canonical path is now `plans/releases/incidents/<id>/` on both sides (`incident_evidence.py:38-42`), `withdrawal-evidence.txt` is allowed and *required* with a digest cross-check (`evidence_custody.py:26,220-238`), and `validate_incident_evidence_commit` is wired into the suite via `validate_committed_incident_addition` (`evidence_custody.py:33,117-139`). Docs and the issue plan corrected. Verified live: suite passes and the self-test now exercises `validate_changed_path_set` + `validate_incident_directory` on a real fixture repo.
2. **Sole-first-GA rollback rejection — RESOLVED.** Direct acceptance case at `incident_recovery_selftest.py:157-167` drives `propose_rollback` against the one-stable-version index and asserts the previously unexercised `release_index.py:245-251` guard.
3. **900-line guardrail / decomposition — PARTIALLY RESOLVED** (see finding 1 below). `selftest.py` is 879 lines and the cap no longer depends on the style deletion, but the method was deletion, not decomposition, and one blank-separator regression remains.
4. **Sign-off schema/runtime parity — RESOLVED.** `started` removed from the enum and `contains`/`minContains`/`maxContains` added (`stable_incident_signoff.schema.json:27-34,90`), backed by real validator support (`json_schema_202012.py:208-218`) and two negative schema-contract cases. Validator tightened to "exactly one final completed attempt" (`incident.py:136-138`). Residual (attempt ordering, `run_id` strict monotonicity) is not expressible in JSON Schema 2020-12 — not actionable.
5. **Profile selection / no duplicate execution — RESOLVED.** `incident-governance` selected by merge, nightly, release; mirrored in `profile_assignment_matrix.json`, `release_report.py:41-46`, `qualification_fixture.py`, and the runner selftest. Duplicate execution suppressed by `runner.py:48-54,105-109,138-144`; confirmed empirically (one `incident-recovery` case in the combined run, still present when only `full` is selected).

### Actionable findings

**1. MEDIUM — Pass-1 finding 3 was closed by deleting the only tests of `validate_incident_index_mutation`, and the issue plan states they were relocated.**

`selftest.py` lost 26 lines covering `validate_incident_index_mutation` directly: the hand-built `incident-roll-forward` mutation happy path and the rejection of a mutation that leaves the affected release `active` without an `incident_id`. They were **not** moved anywhere:

```
$ grep -rn "incident mutation|retained release bytes|reuse one retained|add exactly the qualified" \
    verification/areas/distribution_release/governance/*selftest*.py
(no matches)
```

`release_index.py:171-227` is now reachable only through `propose_rollback`/`propose_incident_roll_forward`, which construct correct mutations by definition, so **all ten** of its `fail()` branches — affected release removed, not withdrawn, affected bytes altered beyond withdrawal, successor not active, `stable` not pointing at the successor, rollback/roll-forward version-set mismatch, retained-release byte drift, non-stable channel drift — have zero test coverage. That validator is the atomicity/immutability guard behind two DoD clauses ("withdrawing the affected version atomically"; "add evidence without deleting or overwriting any version asset or prior generation snapshot").

`plans/issues/active/phase-40-stable-channel-ga-execution.md:463` claims "incident index-transition cases live in the dedicated incident module rather than consuming the shared governance file-size boundary." No such cases exist in `incident_recovery_selftest.py`. Restoring them there (which fits its ownership boundary and keeps it at 860→~890 lines) makes both the coverage and the claim true.

**2. LOW — The PEP 8 blank-separator regression pass-1 flagged is still present.**

`verification/areas/distribution_release/governance/selftest.py:497` has a single blank line before `def test_release_plan_mutations` — the only such site in the file, and one of the two that pass-1 identified as introduced to buy header room. It is no longer load-bearing for the cap (879 lines; restoring the separator gives 880), so there is no reason to keep it.

**3. LOW — The evidence-custody suite now contains two checks that disagree about `plans/releases/README.md`.**

`evidence_custody.py:66` explicitly permits `plans/releases/README.md` to accompany an evidence change, but `validate_incident_evidence_commit` — invoked by the same `run_evidence_custody_checks()` since this commit — rejects any path in the range other than the request and withdrawal evidence (`incident_evidence.py:57-61`). An incident evidence PR that also updates that README passes `validate_changed_path_set` and then fails `validate_committed_incident_addition`. One of the two allowances should be made authoritative.

### Cleared on this pass

- Exact resume/race semantics: single index mutation on resume (`incident_fixture.py:154-156,195-200`), generation burning over all retained snapshots (`:422`), pre- and post-reservation staleness rechecks (`:158-189`), terminal lease-releasing site timeout (`:201-212` + `metadata_lease` finally), byte-identical realized-index assertion on resume (`incident_recovery_selftest.py:146-151`).
- Immutable retention: every evidence write is `O_EXCL` (`_write_once_bytes`), `_write_text_evidence` refuses drift, gen-21 and gen-22 snapshots both retained, `tree_digest` proves release assets unchanged.
- Local-only adapter boundary: temp-root confinement (`validate_fixture_root`, `require_fixture_path`), symlink rejection, credential refusal, non-deploying site marker, and `test_no_production_adapter_surface` asserting no `socket`/`urllib`/`requests`/`subprocess` in the harness and no rollback/roll-forward dispatch input in `release-publication.yml`.
- Withdrawn-version selection: exact pins excluded via `active_installers` (`crates/sifr/src/self_update_metadata.rs:259-276`), covered by the pre-existing `install_withdrawn_stable_rejected.sh`; `validate_release_index` forbids a channel pointing at a non-active release.
- Downgrade consent + out-of-band recovery, extension/Marketplace range check correctly skipped for `incident-roll-forward`, clean external work directory for `generate-incident-request`, demo filename is capability-based (`stable_incident_recovery_demo.sh`), no Rust-interop implementation touched.
- The create-PR host-timing shortfall is disclosed honestly in a pre-existing follow-up (`plans/issues/active/adhoc_performance_budget_host_variance.md:40-47`) with no baseline or waiver change.

### Verdict

**NOT APPROVED** — finding 1 must be resolved (restore the `validate_incident_index_mutation` cases so the validator's rejection branches are covered and the issue-plan statement is true); findings 2 and 3 are small and should be fixed in the same change.
