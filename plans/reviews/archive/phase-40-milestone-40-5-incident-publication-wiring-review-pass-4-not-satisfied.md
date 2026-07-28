# Review result

I re-derived every prior finding and independently re-ran the validations. Here is the pass-4 result.

## Verified closed

**Pass-3 blocker 1 (receipt path + canonical requirement)** — `run_incident_public_recovery.sh:79-88` now reads `"${root}/install.json"` with no `--require-canonical`. This matches the real installer: `generate_version_installer.sh:574-590` resolves `manifest_dir` to `dirname(SIFR_INSTALL_DIR)` and writes `install.json` there with the exact 14 pretty-printed keys of `RECEIPT_FIELDS` (`surface_contracts.py:22-37`), and `self_update_receipt.rs:99-107` discovers the same path from `bin/sifr`. The selftest pins the regression (`install.json` present, `install-receipt.json` absent).

**Pass-3 blocker 2 (no executed coverage)** — `incident_public_recovery_selftest.py` executes the real script for both operations: downgrade rejection plus `grep -F -- "--force"`, forced working-client update, broken-client `rm` + out-of-band dispatcher reinstall, `sifr --version` convergence, pretty-receipt validation, exact evidence bytes, and token scrubbing. I confirmed the surrounding contracts hold: the real dispatcher forwards unknown args to the installer (`generate_dispatchers.sh:201-205, 291-299`), the installer accepts `--force` (`:174, 621-644`), and the real downgrade diagnostic contains the literal `--force` (`self_update_metadata.rs:390-395`).

**Pass-3 minor 3** — untracked-forgery negative now present (`incident_publication_selftest.py:133-161`): the plan is `git rm`'d from HEAD, replanted untracked, tracked-only status asserted clean, then rejected by `_require_head_file`.
**Pass-3 minor 5** — closed via public `stable_publish_fixture.py` (`run_command`, `stage_fixture`).
**Pass-3 minor 6** — largely closed: three job-level booleans (`STABLE_MUTATION_OPERATION`, `STABLE_CANDIDATE_OPERATION`, `INCIDENT_OPERATION`) replace the repeated expressions; each converted `if:` preserves or correctly widens the original predicate, and `env` is valid in step-level `if`.
**All pass-1/pass-2 findings** re-verified closed, including ordering, `--clobber` count == 1, `contents: write` count == 1, custody/`HEAD` binding, generation burn/resume, secret scrubbing, docs/ledger truth (`architecture.md:1449-1464`, `distribution_pipeline.md:584-680`, issue ledger `:715-763`).

I also confirmed governance forbids a backwards roll-forward (`release_index.py:348-352`) and that rollback targets are pinned to the affected plan's `expected_stable_predecessor` (`incident_planner.py:154-160`), so the post-mutation recovery branches cannot be entered with an inverted version relationship in production.

**Validation reproduced:** full `distribution_release` area 125 variants / 0 failures; incident-governance 8/8 + 5/5 + 2/2; stable-publication 8/8 + 2/2; file-size guardrail PASS (2934 files, limit 900); every `scripts/distribution/*.sh` `bash -n`; all governance/distribution Python compiles; all workflow YAML parses; capability demo exit 0; `git diff --check` clean.

## Major (actionable)

**1. The executed roll-forward recovery test models a version relationship production forbids, so the branch's precondition is unverified.**
`incident_public_recovery_selftest.py:28-37` reuses one fixture for both operations with `affected=0.1.1, successor=0.1.0` (`_run_recovery:203-207`, `INSTALLED_VERSION=0.1.0`). Production roll-forward always activates a *newer* successor (`release_index.py:348-352`), and `run_incident_public_recovery.sh:69` runs `sifr self update` **without** `--force` — which the real client permits only for `Ordering::Greater` (`self_update_metadata.rs:388-395`). The fake `sifr` (`:110-120`) ignores ordering entirely except the `RECOVERY_OPERATION == rollback` gate, so it accepts a downgrade the real client would reject and the test asserts convergence to a lower version than the affected one. The roll-forward branch is exercised but its actual precondition is not. Fix: use `affected=0.1.0 / successor=0.1.1` for the roll-forward iteration and make the fake compare versions, rejecting a non-forced downgrade regardless of operation.

**2. The two convergence assertions the pass-3 blocker was about still have no executed negative.**
The sole negative, `test_recovery_rejects_receipt_drift`, does not reach the script's own checks. I reproduced it: with `RECEIPT_CHANNEL=beta` the run aborts inside `release_governance.py validate` with `release-governance: $.version: does not match receipt channel`, because `validate_install_receipt` cross-checks `version_channel(version)` against `channel` (`surface_contracts.py:78-80`). So `run_incident_public_recovery.sh:82-88` (receipt version/channel equals the successor) and `:75-78` (binary `--version` equals `sifr ${successor}`) — the two assertions that actually prove post-mutation client convergence — are never exercised negatively. Add a receipt whose `version` drifts while `channel` stays `stable` (passes the schema, must fail the `jq`), and a `version.txt` drift for the binary check.

**3. The capability demo omits the wave's new executed recovery suite.**
`demos/stable_release_governance_demo.sh:69-78` prints "Stable install/update, Marketplace, and incident publication adapters" and runs `stable_publish_selftest`, `stable_public_smoke_selftest`, `incident_publication_selftest` — but not `governance.incident_public_recovery_selftest`, the only executed production adapter in this wave and the one that covers the rollback client install/update/recovery flow named in the milestone demo requirement (`plans/phases/40_...md:1084-1088`). This is the same one-line omission that pass-2 finding 2 fixed for the extracted public-smoke suite. Add it to the same subshell.

## Minor / non-blocking

**4.** `plans/reviews/active/phase-40-milestone-40-5-incident-publication-wiring-review-pass-4.md` is 0 bytes — third recurrence (pass-1 #7, pass-3 #4). Populate before the PR.

**5.** `release-publication.yml:16-21` collapses six `workflow_dispatch` inputs to bare `{}`, dropping the explicit `default: ""`. It is unprecedented in `.github/workflows/`, saves 12 lines in a file at 853/900, and cannot be schema-validated locally (no actionlint; the YAML case only `yaml.safe_load`s). Behaviour-neutral if GitHub accepts it, but it is line-count cosmetics on the dispatch surface of the sole production mutation workflow — restore the explicit form.

**6.** `verify_public_stable_docs.py:29-31` requires the literal strings `Active stable version` and `Withdrawn stable versions` in the deployed page, but `site_release_contract.json`'s `stable_documentation` pins only the renderer *path*, route, and field names — not the rendered labels or the renderer's digest (unlike `workflow_sha256`). An external renderer edit fails the docs smoke 180 s after the clobber. Residual of pass-2 #3.

**7.** `stable_prepare.py:588` validates `$.incident.incident_id` with `require_nonempty_string` while `stable_publication_prepare.schema.json:152-155` requires the incident-id pattern; the runtime is looser than the schema (transitively constrained via `validate_incident_request`, but the asymmetry invites drift).

**8.** `dispatch_stable_site_publication.sh:12,87` advertises and regex-accepts `--default-channel alpha`, but `channel_facts_valid` (`:67-74`) can never be true for `alpha`, so every `alpha` call exits via `usage()` with no diagnostic. Fail-closed, but the accepted-value set is self-contradictory.

**9.** `incident_fixture.py:638-642` sets the drill's `release_signoff_sha256` to the digest of the *proposed index* for roll-forward. The field means "digest of the stable release sign-off asset"; the drill core now carries a semantically fabricated value that only passes because `validate_incident_signoff` requires any sha256.

**10.** Residual of pass-2 #12 / pass-3 #6: `release-publication.yml:787` still spells the stable-only subset out longhand while the other twelve sites now use the named booleans.

VERDICT: CHANGES_REQUIRED
