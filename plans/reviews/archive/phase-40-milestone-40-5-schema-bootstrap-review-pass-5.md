# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 5)

I read the full diff, all 16 untracked files, the phase spec (`plans/phases/40_stable_channel_ga_promotion_and_release_governance.md:938-1010`), the issue ledger, and archived passes 1–4, then independently re-ran the gates plus mutation probes against both the JSON Schema and the semantic validator.

## Pass-4 re-audit

| # | Pass-4 finding | Status |
|---|---|---|
| 1 | `epoch-bootstrap` assigned to no profile; merge gate never runs the bootstrap validator/producer | **Resolved.** Present in `merge.json:247`, `nightly.json:226`, `release.json:225`, `profile_assignment_matrix.json:99-113`, `release_report.py:46`, `selftest.py:244,301`, `qualification_fixture.py:123`, `schema_contracts.py:407,447`, `architecture.md:1433`, `distribution_pipeline.md:614-616`. Measured with the merge profile's exact selection: **variants=55, failures=0**, `case=schema-v2-preview-epoch-bootstrap` present. Nightly selection (`full`+`epoch-bootstrap`) yields exactly **1** occurrence — pass-3's dedup still holds. Release selection: **58 variants, 0 failures**. `coverage_matrix` area PASS (`profile assignment matrix ok: rows=17`). |
| 2 | `duplicate_smoke` / `extra_asset` negatives not load-bearing; no `alpha-assets`-else negative | **Resolved** for both remediations. Measured against weakened schema copies: baseline rejects all three negatives; `delete properties.public_smoke.allOf` → `duplicate_smoke=ACCEPT` (now load-bearing via the distinct `SHA_A` digest at `schema_contracts.py:78`); `delete top-level allOf` → `alpha_with_beta=ACCEPT` (else branch now load-bearing). |

All pass-1/2/3 remediations remain. Re-verified: no schema-v1 parser/fixture/migration/fallback (`schema_epoch_bootstrap_workflow_contract.sh:145-151`); digest chain prepare→publish→live at `release-publication.yml:113-117,267-272,285-291,568-573`; `--clobber` pinned at 1; `bootstrap-alpha` never reaches the snapshot/replace/site steps; both `verify_site_workflow_identity.sh` call sites gated by count and ordering; `verify_release_publication_assets.sh:85-96` and `release-publication-prepare.yml:166-174` use matching prefix-stripped digest maps.

Gates re-run: three contract cases PASS; file-size gate PASS (2898 files, `release-publication.yml` 795/900) and self-test PASS; `git diff --check` PASS; `stable_gate_inventory_selftest` exit 0; `ruff check` clean on every changed Python file (the 8 diagnostics are all in untouched files — reproduced under pyenv 3.10.12). Pass-4's cache-busting preflight item is closed by the ledger entry.

## Actionable findings

### 1. The bootstrap validator's guards are not pinned by the suite that now gates them — Low

Pass 4 moved `epoch-bootstrap` into `merge.json` so the merge gate would cover `validate_bootstrap_evidence`/`materialize_bootstrap_evidence`. It now runs, but it does not *pin* them. Applying pass-4's own methodology to the validator, these single-line deletions each leave `governance.schema_bootstrap_selftest` green (rc=0):

| `schema_bootstrap.py` | guard removed |
|---|---|
| `:217` | `or smoke_id in seen` (duplicate smoke id) |
| `:217` | `smoke_id not in SMOKE_IDS` (unknown smoke id) |
| `:209-210` | exactly-four `public_smoke` length check |
| `:179` | `require_sha256(alpha_evidence["sha256"], …)` |
| `:149-152` | `require_sha256(evidence["prepare_summary_sha256"], …)` |
| `:238-239` | approver `normalized in seen` uniqueness |
| `:469-470` | `release["status"] != "active"` |

For contrast, these *are* detected: index generation, legacy `size_bytes`, both exact-asset-set checks, approver≠initiator, the staged-alpha stage/identity binding, and index↔record equality. The three `public_smoke` guards mask each other — `schema_bootstrap_selftest.py:142-144` appends a 5th duplicate, so removing both the length and dedup checks *is* caught; only a single-guard regression is invisible.

This is the semantic half of the parity pair passes 2/#1, 3/#5 and 4/#2 spent three rounds closing, and it is the half that matters at publication time: the protected path calls only `release_governance.py validate --kind schema-bootstrap-evidence` and `materialize_bootstrap_evidence`; the JSON Schema is never consulted. A regression here is schema-stricter/validator-looser — the dangerous direction. Add to the `mutations` tuple at `schema_bootstrap_selftest.py:129-146`: a four-item `public_smoke` with a duplicated id and distinct digest; a four-item list with an unknown id; non-digest `alpha_evidence.sha256`; non-digest `prepare_summary_sha256`; `approvers: ["r", "R"]`; and a wrapper whose `release.status` is not `active`.

### 2. `$defs/alpha_assets`'s `not: {minProperties: 10}` still has no isolating negative — Low

`schema_contracts.py:80` adds the key `"unexpected"`, which `propertyNames` (`schema_epoch_bootstrap_evidence.schema.json:144-146`) rejects on its own. Measured: `delete $defs.alpha_assets.not` leaves `validate_schema_contracts` green, and the weakened schema then **accepts** an alpha block carrying a tenth *validly named* asset (`sifr-0.9.9-alpha.1-x86_64-apple-darwin.tar.gz`) that `validate_bootstrap_evidence:493-496` rejects. Exactly the shape of pass-4 finding 2, one constraint over. Use a validly named tenth asset for that negative, or add one beside it.

## Not findings

- `ruff format --check` would reformat `schema_bootstrap.py`, `schema_bootstrap_selftest.py`, `schema_contracts.py`, and `runner.py` — but it would also reformat 19 of 29 files in that package and is wired into no repo gate. Pre-existing style baseline, not this slice.
- `iter_source_files` (`check_file_size_guardrails.py:133-134`) now walks `.yml`/`.yaml` under `crates`/`scripts`/`verification`/`demos` too, but `category_for_path` returns `None` for them — extra traversal only, no behavior change.
- `distribution_release` is absent from `create-pr` entirely (`profile_assignment_matrix.json:98`), so `--profile create-pr` not running `epoch-bootstrap` matches `incident-governance` and is pre-existing.
- `extra_asset`'s redundancy aside, the four cross-field semantics 2020-12 cannot express remain validator-stricter and semantically covered.
- The fifth smoke output `${out}/stable-dispatcher.sh` is correctly outside `SMOKE_IDS`.
- Post-`gh release create` failures remain unrecoverable by re-run; for `bootstrap-index` the evidence upload is necessarily last. Fail-loud stays correct.
- `A && B || C` at `release-publication.yml:175` and `release-publication-prepare.yml:72` is correct for both false branches.

## Commit mechanics and execution requirements (not implementation defects)

- `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-5.md` is a 0-byte untracked file. It must receive this report, be archived, and be ledgered in `plans/issues/active/phase-40-stable-channel-ga-execution.md` (passes 1–4 are at `:361-399`; pass 5 is absent). I did not modify it, per instruction.
- Pass-1's external requirements stand: `stable-release` environment with ≥1 `release/distribution` reviewer and GitHub "prevent self-review" enabled; confirmation that no reviewers are attached to the auto-created `preview-release` environment; live `channels.json` still exactly 105 bytes / `71b3243925…4bf9ef` at `bootstrap-index` time; `sifr.sh` serving new dispatcher bytes inside the 180-second budget.
- Still not locally reproducible: whether `actions/download-artifact@v4` resolves an attempt-1 artifact during "Re-run failed jobs". Validate on the first live re-run.

VERDICT: NOT SATISFIED
