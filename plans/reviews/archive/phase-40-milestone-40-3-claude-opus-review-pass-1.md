I verified `HEAD = cb7567f2daa8cf721e1785844e307db4e514aebf`, merge base `db80dd35e056b9dcc9a2ac64475a198f5c36bfaa`, reviewed all 21 changed files, and ran read-only checks.

## Checks run

| Check | Result |
|---|---|
| `areas run --area distribution_release --suite incident-governance` | pass, 8/8 scenarios |
| `areas run --area distribution_release --suite full --suite evidence-custody` | pass, 54 variants, 0 failures |
| `demos/stable_incident_recovery_demo.sh` | pass, exit 0, all four scenes |
| `scripts/check_file_size_guardrails.py` | PASS (2866 files, limit 900) |
| Diff scope | no `crates/**` change; no Rust-interop implementation touched |
| Demo filename | `stable_incident_recovery_demo.sh` — capability-based, no phase/milestone identifier ✅ |
| Production surface | `.github/workflows/release-publication.yml` has no rollback/roll-forward input, no drill env; harness has no network/`gh`/`vsce`/dispatch adapter ✅ |

What is genuinely solid: the pure planner (`incident_planner.py`), the index state machine (`release_index.py:171-313`) with byte-preservation of unaffected releases, atomic withdraw+activate, generation burning/allocation over all retained snapshots, race rejection, exact-resume without a second index mutation, site-timeout terminal/lease-releasing semantics, write-once evidence via `O_EXCL`, temp-root confinement, symlink rejection, credential refusal, downgrade consent with installer delegation, and first-GA roll-forward.

## Actionable issues

**1. HIGH — Incident evidence custody contract is self-contradictory and unsatisfiable; the new validator is wired into nothing.**

- `verification/areas/distribution_release/governance/incident_evidence.py:38` requires `plans/incidents/<incident-id>/`.
- The phase contract (`plans/phases/40_...md`, "New incident requests enter custody through evidence-only PRs at `plans/releases/incidents/<incident-id>/stable-incident-request.json`") and the pre-existing gate `evidence_custody.py:23-26` require `plans/releases/incidents/<incident-id>/`.
- `evidence_custody.py:189` allows only `stable-incident-request.json` and `stable-incident-signoff.json` in an incident directory; the new validator *requires* a co-located `withdrawal-evidence.txt`.

Reproduced directly:

```
changed-path-set REJECTED: invalid evidence path(s):
  plans/releases/incidents/inc-2026-001/withdrawal-evidence.txt
incident-directory REJECTED: .../inc-2026-001: unsupported incident evidence: withdrawal-evidence.txt
```

Failure scenario: a real incident PR at the canonical `plans/releases/incidents/<id>/` is rejected by all three checks; a PR at `plans/incidents/<id>/` passes the new validator but is silently invisible to `evidence_custody` (`validate_changed_path_set` early-returns, `validate_existing_evidence` only walks `RELEASES_ROOT/incidents`), so the canonical custody registry never validates it. Neither location works. Compounding this, `validate_incident_evidence_commit` has **no caller** outside its own selftest and the CLI subcommand — no suite, profile, or repository check invokes it, so the milestone requirement "Repository checks reject source changes, schema/digest drift, or unrelated incident files in that evidence commit" is not actually enforced by a repository check. The wrong path is propagated into `internal_docs/stable_incident_response.md:52`, `internal_docs/distribution_pipeline.md` ("Stable Incident Recovery" section), and `plans/issues/active/phase-40-stable-channel-ga-execution.md` (milestone_40_3 bullet), so the docs are inaccurate against the phase contract.

**2. MEDIUM — Missing acceptance coverage explicitly named in the milestone 40.3 definition of done.**

"A fixture-backed first-GA incident proves **the sole stable version cannot be withdrawn alone**." `build_roll_forward_fixture` (`incident_recovery_selftest.py:539-581`) builds the only single-stable-version index and is used solely in the happy path at line 151. No test attempts a `rollback` against that one-version first-GA index. The nearest case, `incident_recovery_selftest.py:209-213`, uses `build_rollback_fixture(..., affected_transition="ga-activation")`, which still contains **two** retained stable releases and fails on plan transition (`"rollback requires an affected normal release plan"`), not on target unavailability. The distinct guard at `release_index.py:244-251` ("must name a distinct retained active stable release") is unexercised. The DoD clause is therefore not proven.

**3. MEDIUM — The 900-line file-size guardrail was satisfied by deleting PEP 8 blank separators rather than by decomposition.**

`selftest.py` went 898 → 899 lines. The diff adds 3 lines and removes exactly two top-level blank separators, leaving single blank lines before `def test_release_plan_mutations` (`selftest.py:518`) and `def test_incident_mutations` (`selftest.py:563`) — the only two such spots in the file. With correct spacing the file is 901 lines, over the cap. AGENTS.md: "If a touched file exceeds the cap, refactor it by responsibility rather than adding more code to an oversized module." The guardrail passes only because of the style regression, and any further addition to this file is now blocked.

**4. LOW — Schema/validator parity gap on sign-off attempt terminal state.**

`incident.py:131-137` rejects duplicate/non-increasing `run_id`, any `started` attempt, and a non-`completed` final attempt. `schemas/stable_incident_signoff.schema.json:24-28,75-86` encodes none of these: `status` still enumerates `started`, and there is no uniqueness or terminal-state constraint. A consumer validating only against the checked-in schema accepts a sign-off with a dangling pending attempt. (The `operation` field and the `X.Y.Z` version patterns were correctly added to both sides.)

**5. LOW — The registered `incident-governance` suite is selected by no profile.**

`manifest.json:64-76` registers the suite and `runner.py:109-115,167-181` supports it, but no entry in `verification/profiles/*.json` selects it; `release.json:220-228` selects `full`/`qualification`/`evidence-custody`, and `release_report.py:41` `REQUIRED_SUITES["distribution_release"]` omits it. Coverage happens only transitively because `runner.py:128-134` appends the module to `full`, so the named suite itself is dead registration and merge/nightly/create-pr never run incident recovery.

## Not issues (checked and cleared)

- Sign-off completeness — request digest, per-attempt run/mode/approver/status/mutations, realized generation/digest, site reconciliation, validation, communications, closure are all recorded and cross-validated against the request (`incident_fixture.py:611-668`, `incident.py:171-176`).
- Retention — `tree_digest` asset invariance plus retained gen-21 and gen-22 snapshots asserted (`incident_recovery_selftest.py:60,73-75`); every write path is `O_EXCL`.
- Concurrency — shared `state/metadata-concurrency.lock` and `check_release_submission_allowed`; lease release on terminal site timeout matches the phase's GitHub-concurrency semantics.
- Extension/Marketplace reconciliation — range check applied for `rollback`, correctly skipped for `incident-roll-forward` per the milestone (`incident_fixture.py:475-488`).
- `generate-incident-request` correctly enforces an outside-repo clean work directory and exact affected/target plan and evidence digest binding (`release_governance.py:358-403`).

## Verdict

**NOT APPROVED** — issues 1 and 2 are blocking (a mutually unsatisfiable, unenforced evidence-custody contract that also contradicts the phase doc, and a missing DoD-named first-GA acceptance case); 3, 4, and 5 must also be resolved before approval.
