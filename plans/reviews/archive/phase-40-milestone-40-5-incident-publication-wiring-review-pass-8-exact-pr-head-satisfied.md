# Satisfied exact-PR-head review result — pass 8

## Identity verification

| Check | Value | Result |
|---|---|---|
| Local `HEAD` | `341b312f50de61c549f1bde01a6676f248231d02` | ✓ |
| `origin/codex/phase-40-milestone-40-5-incident-publication-wiring` | `341b312f50de61c549f1bde01a6676f248231d02` | ✓ |
| `gh pr view 3047 --json headRefOid` | `341b312f50de61c549f1bde01a6676f248231d02` | ✓ |
| Branch / base | expected branch; base `main`, `OPEN`, `MERGEABLE` | ✓ |
| Merge-base vs `main` | `0f59a48b3` (a commit on `main`'s first-parent history); `main` has since advanced by unrelated Rust-interop merges, so HEAD is not a descendant of the current tip — GitHub still reports `MERGEABLE` | ✓ (non-blocking, below) |
| Worktree | clean except this pass's own untracked 0-byte artifact slots (`…pass-8-exact-pr-head.md`, `.agent.log`); no active review `.md` is tracked | ✓ no implementation drift |
| Diff scope | 63 files, +6238/−620 vs `0f59a48b3`; delta over pass 7 is exactly 5 files | ✓ |

## Pass-7 sole actionable — re-derived and closed

Pass 7's finding was that `validate_stable_prepare_summary` placed `incident` in the **optional** set while line 578 indexed it unconditionally, yielding a raw `KeyError` traceback for a roll-forward summary missing its `incident` binding, reachable through two operator-facing validator kinds.

Fixed at `stable_prepare.py:366-388` exactly as the schema shapes it:

```python
required = { "schema_version", …, "site" }
if operation == "incident-roll-forward":
    required.add("incident")
require_exact_keys(summary, required=required, location="$")
```

The `optional=` argument is gone, so `require_exact_keys` (`common.py:65-77`) now fails **both** directions — missing on roll-forward, and `unknown field(s)` for any other operation. Independently reproduced at this SHA:

| Input | JSON Schema (`stable_publication_prepare.schema.json:176-183`) | Runtime |
|---|---|---|
| roll-forward `release_prepare` (positive) | accepts | accepts |
| same, `incident` deleted | `$: missing required field incident` | `GovernanceError: $: missing required field(s): incident` |
| `ga-activation` summary + `incident` | `$: matches forbidden schema` (`else: {not:{required:["incident"]}}`) | `GovernanceError: $: unknown field(s): incident` |

Runtime is now equal-or-stricter on this axis in both directions, with no crash on any input the schema rejects.

Operator-facing CLI, both new kinds, verified directly (not just via the suite):

```
$ python3 scripts/distribution/release_governance.py validate --kind stable-publication-prepare  --input …
release-governance: $: missing required field(s): incident      rc=2
$ python3 scripts/distribution/release_governance.py validate --kind incident-publication-prepare --input …
release-governance: $: missing required field(s): incident      rc=2
```

Governed exit code 2, governed diagnostic, no traceback.

Coverage for both directions at both levels, all executed:
- Runtime missing — `stable_prepare_selftest.py:208-216` inside `test_materialized_incident_roll_forward_prepare`, asserting the exact `missing required field(s): incident` text.
- Runtime unexpected — `stable_prepare_selftest.py:337-347` in `test_summary_contract` via `expect_rejected`.
- Runtime through the CLI — `stable_prepare_selftest.py:418-445`, asserting `returncode == 2`, the diagnostic substring, **and** `"Traceback" not in stderr`.
- Schema, both directions — `schema_negative_contracts.py:54-72`, reached from `schema_contracts.py:160`, which runs inside the `governance-contracts` variant.

Producer parity holds: `summary["incident"]` is written only when `incident_request is not None` (`stable_prepare.py:345-352`), and `incident_request` is populated only for `incident-roll-forward` (`:126-135`), with non-roll-forward incident inputs rejected at `:136-140`. So the new forbid-direction cannot reject a summary the producer emits.

## Regression check on pass-1 → pass-6 closures

The delta over pass 7 touches only `stable_prepare.py`, `schema_negative_contracts.py`, `stable_prepare_selftest.py`, the archived pass-7 report, and the issue ledger — no workflow, script, schema, or publication-path change. Re-confirmed independently at this SHA: single `--clobber` in `run_incident_publication.sh:439`; exactly one `contents: write` in `release-publication.yml`; `unset SITE_TOKEN VSCE_PAT` at `:116` with `GH_TOKEN=""/SITE_TOKEN=""/VSCE_PAT=""` scrubbing at every installer/smoke/recovery call site (`:328,498,521`) and the Marketplace publish narrowing to `VSCE_PAT` only (`:355`); `incident_publication_workflow_contract.sh` rc=0; 18 governance schemas, matching the updated `selftest.py:87` count.

## Validation executed on this exact head

| Selection | Result |
|---|---|
| `--suite full --suite evidence-custody` | **68 variants / 0 failures** |
| `--suite stable-prepare --suite incident-governance --suite stable-publication` | 6 variants / 0 failures |
| `stable_prepare_selftest` | 8/8, incl. the new CLI missing-incident negative |
| `governance.selftest` | 14/14 |
| `incident_publication_selftest` | 5/5 |
| `incident_public_recovery_selftest` | 2/2 |
| `stable_publish_selftest` / `stable_public_smoke_selftest` | 8/8 / 2/2 |
| `incident_publication_workflow_contract.sh` | rc=0 |
| runner self-test (`run_all`) | 11 checks pass |
| `check_file_size_guardrails.py` | PASS (2936 files, limit 900) |
| `git diff --check 0f59a48b3..HEAD` | clean |
| `demos/stable_release_governance_demo.sh` | rc=0, capability-named, no phase/milestone numbering |

Profile registration re-verified: `incident-governance` and `stable-publication` appear in `merge.json`, `nightly.json`, and `release.json`, and both the dedicated (`runner.py:152-166,196-206`) and combined `full` (`:226-242,258+`) paths cover the three incident variants plus the public-smoke variant.

## Actionable findings

None.

## Non-blocking observations

1. **Focused-selection count is 68, not 69.** `--suite full --suite evidence-custody` measures 68 variants / 0 failures at this SHA — matching the issue ledger's own figure, not the 69 in the request summary. Zero failures either way. Recurrence of pass-7 #5.
2. **Branch is behind `main`.** `main` advanced by unrelated Rust-interop merges (`401c53971` tip) after this branch was cut. Mergeable per GitHub; the "linear on protected main" property pass 7 recorded no longer holds at the tip, so a merge commit or rebase is coming. No governance impact, but the publication workflow's own `HEAD == origin/main` assertions apply to production runs on `main`, not to this PR.
3. **New CLI negative fixture is internally inconsistent by construction.** `stable_prepare_selftest.py:419-421` flips `operation` to `incident-roll-forward` on a `ga-activation` summary without touching `mutation.transition`. The assertion is still exact and correct because `require_exact_keys` runs before any cross-field check, but the fixture proves the key check rather than a realistic operator input. Cosmetic.
4. **Pass-6 #1 residual.** The rollback dispatcher-provenance gate remains only in `stage_incident_publication`; `materialize_incident_prepare` already validates both plans and could surface the disagreement in the reviewer-visible prepare summary. Both placements are pre-mutation and fail-closed; pass 5 asked for stage-time.
5. **Pass-5 #2 / pass-6 #2 residual.** `site_release_contract.json` pins `renderer_sha256` but never verifies it live the way `workflow_sha256` is verified; drift is still caught by the AST-parsed `RENDERED_LABELS` equality check and, ultimately, by the post-clobber docs smoke. Cross-repository.
6. **Rollback's own recovery guards still have no executed negative.** `test_recovery_rejects_binary_and_receipt_drift` iterates only `incident-roll-forward`; `run_incident_public_recovery.sh:57-64` failure branches are never driven.
7. **Pre-existing looseness, not introduced here.** `validate_stable_prepare_summary` still accepts arbitrary strings for `release_prepare.marketplace.{publisher,extension,version}`, `artifacts.*.name`, and `release_report.id` — identical for `normal`, so out of scope.
8. **This pass's own artifact slot** (`plans/reviews/active/…pass-8-exact-pr-head.md`) is 0 bytes; populate from this report and archive before merge, as was done for pass 7.

Scope stayed inside Phase 40 stable-channel GA governance; no Rust-interop implementation requested, and existing Rust suites were treated as consumed evidence only.

VERDICT: SATISFIED
