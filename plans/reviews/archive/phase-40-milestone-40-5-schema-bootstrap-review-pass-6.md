# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 6)

I read the full `git diff`, all 16 untracked files, the phase/issue ledger, and archived passes 1–5, then independently re-ran the gates and re-ran pass-5's own mutation methodology against both the JSON Schema and the semantic validator. I did not modify any file.

## Pass-5 re-audit

| # | Pass-5 finding | Status |
|---|---|---|
| 1 | Bootstrap validator guards not pinned by the suite that gates them | **Partially resolved — 5 of 7 pinned.** See findings 1 and 2. |
| 2 | `$defs/alpha_assets`'s `not: {minProperties: 10}` had no isolating negative | **Resolved.** `schema_contracts.py:79-81` now adds the validly named `sifr-0.9.9-alpha.1-x86_64-apple-darwin.tar.gz`, so `propertyNames` no longer masks the bound. Measured: `delete $defs.alpha_assets.not` now makes `validate_schema_contracts` **fail** (it was green in pass 5). `delete properties.public_smoke.allOf` and `delete top-level allOf` also still fail, so pass-4's two negatives remain load-bearing. |

Re-verified as still in place: all pass-1/2/3/4 remediations; profile assignment across `merge`/`nightly`/`release` + matrix + `release_report.REQUIRED_SUITES` + `selftest.py` + `qualification_fixture.py`; the `full`-suite dedup; no schema-v1 parser/fixture/migration/fallback; digest chain prepare→publish→live; `--clobber` pinned at 1; both `verify_site_workflow_identity.sh` call sites gated by count and ordering; smoke-output filename freeze; the four-surface legacy-identity equality gate; pass-5 report archived and ledgered at `:401-407`.

Gates re-run here: `governance.schema_bootstrap_selftest` PASS; `epoch-bootstrap` suite PASS (variants=1); merge-profile distribution selection PASS (**variants=55, failures=0**); the three contract cases PASS; `profile_assignment_matrix` PASS (rows=17); file-size gate PASS (2898 files, `release-publication.yml` 795/900); `git diff --check` clean; `ruff check` rc=0 on all changed Python (under pyenv 3.10.12).

## Actionable findings

### 1. The exactly-four `public_smoke` length guard — pass-5 item 3 — is still not pinned; the retained mutation is masked by the guard the same round made load-bearing — Low

Measured: deleting `schema_bootstrap.py:209-210` leaves `governance.schema_bootstrap_selftest` at **rc=0**.

The mutation retained for this (`schema_bootstrap_selftest.py:156-158`) appends a copy of `public_smoke[0]`, producing a *five*-item list with a duplicate id. Pass 5 already diagnosed the masking; this round pinned the dedup branch (`:159-166`) but kept the same length mutation, so the append is now caught by `smoke_id in seen` whether or not `:209-210` exists. Because `SMOKE_IDS` has exactly four members, no over-length list can isolate the guard — only a *short* one can. With `:209-210` deleted I confirmed a three-record `public_smoke` (all unique, all known ids) is **accepted** by `validate_bootstrap_evidence`, i.e. `release_governance.py validate --kind schema-bootstrap-evidence` would admit generation-1 evidence recording only three of the four public smokes. Add `lambda value: value["public_smoke"].pop()` to the `mutations` tuple.

### 2. The `release["status"] != "active"` guard — pass-5 item 7 — is still not pinned; the new withdrawn case is rejected upstream — Low

Measured: deleting `schema_bootstrap.py:469-470` leaves the self-test at **rc=0**.

The added case (`schema_bootstrap_selftest.py:86-96`) sets `status = "withdrawn"` but leaves `incident_id` absent, so `validate_release_record` (`release_index.py:88-91`) rejects it first — I reproduced the actual diagnostic: `release[0.1.0-alpha.2].incident_id: must be a lowercase incident identifier`. The bootstrap-local guard is never reached. The isolating input is a withdrawn wrapper carrying a *valid* `incident_id`; with that, and only that, the guard fires (`alpha release.release.status: must be active`). Without it, `build_preview_epoch` would seat a withdrawn alpha or beta release as a generation-1 preview channel head. Set `withdrawn_alpha["release"]["incident_id"] = "inc-2026-001"` alongside the status change.

### 3. The pass-5 ledger entry asserts both of the above are remediated — Low

`plans/issues/active/phase-40-stable-channel-ga-execution.md:403-406` records "independent semantic mutations for smoke length … and active release status". Neither mutation is independent, as measured above. This is the durable record future rounds re-audit against; correct it with the remediation rather than leaving a claim the suite does not support.

### 4. Five further guards in the same load-bearing function are unpinned — Low

Applying pass-5's methodology beyond its own list, these single-edit deletions also leave `governance.schema_bootstrap_selftest` green:

| `schema_bootstrap.py` | guard | behavior once removed (confirmed) |
|---|---|---|
| `:228-229` | `_validate_approvers` empty-list rejection | `"approvers": []` **accepted** |
| `:146` | `require_positive_int(run_attempt)` | `run_attempt: 0` accepted |
| `:207` | `require_sha256(index["sha256"])` | non-digest index digest accepted |
| `:194-197` | `require_sha256(alpha_evidence["prepare_summary_sha256"])` | non-digest accepted |
| `:490` | `require_sha256(release_record_sha256)` | non-digest accepted |

The first is the consequential one: it is what makes the durable evidence prove a protected, non-initiating approval happened. Every guard is correct today — I verified each rejects on the current code — but the merge gate would not notice their removal, and the protected path consults only this validator, never the JSON Schema. Six mutations of the same shape as the ones already in the tuple close all five.

## Not findings

- Schema-side constraints `public_smoke.uniqueItems`/`minItems`/`maxItems`, `$defs/alpha_assets.propertyNames`, `$defs/approvers.{minItems,uniqueItems}`, and `$defs/beta_assets.not` are individually deletable without failing `validate_schema_contracts` — but each is redundantly covered by a sibling constraint that *is* pinned, and the schema is the secondary surface. Not worth chasing past finding 4.
- The four cross-field semantics 2020-12 cannot express remain validator-stricter and semantically covered.
- `profile_assignment_matrix.json` omitting `distribution_release:qualification` under `merge` is correct: `validate_row_membership` treats rows as a declared subset of the profile's selection, and the omission predates this slice.
- `A && B || C` at `release-publication.yml:175` and `release-publication-prepare.yml:74` is correct for both false branches.
- Post-`gh release create` failures remain unrecoverable by re-run; for `bootstrap-index` the evidence upload is necessarily last. Fail-loud stays correct.
- The fifth smoke output `${out}/stable-dispatcher.sh` is correctly outside `SMOKE_IDS`.
- `release-publication.yml` at 795/900 with the rest of 40.5 still to land; extraction discipline needs to continue, but no new evidence to reopen.

## Commit mechanics and execution requirements (not implementation defects)

- `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-6.md` is a 0-byte untracked file. It must receive this report, be archived, and be ledgered (passes 1–5 are at `:361-407`). I did not modify it, per instruction.
- Pass-1's external requirements stand: `stable-release` environment with ≥1 `release/distribution` reviewer and GitHub "prevent self-review" enabled; no reviewers attached to the auto-created `preview-release` environment; live `channels.json` still exactly 105 bytes / `71b3243925…4bf9ef` at `bootstrap-index` time; `sifr.sh` serving new dispatcher bytes inside the 180-second budget.
- Still not locally reproducible: whether `actions/download-artifact@v4` resolves an attempt-1 artifact during "Re-run failed jobs". Validate on the first live re-run.

VERDICT: NOT SATISFIED
