# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 8)

I read the full `git diff`, all 20 untracked files, the phase/issue ledger, and archived passes 1–7, then re-ran pass-7's full-validator mutant methodology from scratch. I generated **83** single-edit mutants of `schema_bootstrap.py` mechanically from its AST (one per `or`-clause of every `if …: fail(…)`, one per dropped `require_*` statement, one per bypassed `x = require_*(y, …)`) and ran `governance.schema_bootstrap_selftest` against each, then probed every survivor at its real entry path (`validate_bootstrap_evidence`, `build_preview_epoch`, `resolve_distinct_approvers`, and `materialize_bootstrap_evidence` at both stages).

**Result: 33 of 83 survive (was 42); exactly 1 is load-bearing (was 10).**

### Pass-7 re-audit — 9 of 10 load-bearing guards now killed

| `schema_bootstrap.py` | pass-7 item | mutant | status |
|---|---|---|---|
| `:469-470` `status != "active"` | F1 | 73 | **killed** — `schema_bootstrap_selftest.py:424-439` now calls the `alpha-assets` producer with a withdrawn-but-valid record, the entry path pass 7 identified |
| `:437-438` `source_commit` binding | F2 | 67 | **killed** (`:440-446`) |
| `:435-436` `record_version != version` | F3 | 66 | **killed** (`:487-493`) |
| `:487-488` `version_channel != channel` | F3 | 77 | **killed** (`:218-220`) |
| `:460` `require_exact_keys(wrapper)` | F3 | 70 | **killed** (`:107-117`) |
| `:42` size clause | F3 | 0 | **killed** (`:89-97`) |
| `:234` `require_nonempty_string(raw)` | F3 | 46 | **killed** (`:198`) |
| `:92` `require_nonempty_string(initiator)` | F3 | 3 | **killed** (`:143-149`) |
| `:111` `user.login` | F3 | 9 | **killed** (`:150-158`) |
| `:227` `require_array` in `_validate_approvers` | F3 | 44 | **not killed** — finding 1 |

Pass-7 F4 (ledger) is addressed for pass 6's entry; pass 7's own entry carries a narrower version of the same overclaim — finding 2.

The 32 other survivors are all in pass-7's declared not-findings set and I re-confirmed they fail closed via a pinned sibling: `:268`, `:310`, the `:388`/`:397` clauses, `:429`, `:462`, the seven `:294` `is None` clauses, and the type guards. I additionally probed the four `require_nonempty_string` bypasses pass 7 did not enumerate individually (mutants 4, 39, 71, 76 at `:93`, `:216`, `:461`, `:486`) — all still reject their bad input via a sibling (`smoke_id not in SMOKE_IDS`, `version_channel`, the `environment` default constant). Not findings.

**Gates re-run:** `schema_bootstrap_selftest` PASS; `epoch-bootstrap` suite PASS (variants=1); merge-profile distribution selection PASS (**variants=56, failures=0**); `governance.selftest` PASS (tests=14); `validate_schema_contracts` PASS; all three contract cases PASS; file-size gate PASS (2898 files, `release-publication.yml` 795/900); `ruff check` **rc=0** on all 11 changed/new Python files (resolved under pyenv 3.10.12 — pass 7 could not run this); `bash -n` clean over `scripts/distribution/*.sh`; `git diff --check` PASS. Pass-6's three schema-side negatives re-measured as still load-bearing (deleting `properties.public_smoke.allOf`, the top-level `allOf`, or `$defs.alpha_assets.not` each fails `validate_schema_contracts`). Structural remediations intact: `--clobber` at 1, two `verify_site_workflow_identity.sh` call sites, no schema-v1 surface, `epoch-bootstrap` in merge/nightly/release + matrix rows 102/107/112 + `release_report.py:46`.

Mutation testing required temporarily rewriting `schema_bootstrap.py` and the evidence JSON Schema. Both were restored and verified byte-identical (`schema_bootstrap.py` sha `90cb386a…`; schema `diff` clean). The working tree is unmodified; I wrote nothing to `plans/`.

## Actionable findings

### 1. `_validate_approvers`' array-container guard (`:227`) is still unpinned — pass-7 finding 3 recurs — Low

`schema_bootstrap.py:227`. Deleting it (`values = payload` instead of `require_array(payload, location)`) leaves `governance.schema_bootstrap_selftest` at **rc=0**.

The mutation added for it (`schema_bootstrap_selftest.py:197`, `"approvers": "release-reviewer"`) does not isolate the guard. Measured with `:227` bypassed:

```
"approvers": "release-reviewer"  → rejected: $.approvers[3]: must be a unique GitHub login
"approvers": "abc"               → ACCEPTED   (approvers a, b, c)
"approvers": "x"                 → ACCEPTED
"approvers": {"a": 1}            → ACCEPTED
```

The chosen string happens to repeat the character `e`, so the per-character walk trips the `normalized in seen` dedup at `:238-239` — the guard passes 5, 6 and 7 each spent a round making load-bearing. Any string of distinct characters, or a mapping, slips through. Baseline rejects all four with `$.approvers: must be an array`, so the guard is correct today and genuinely load-bearing: with it gone, `release_governance.py validate --kind schema-bootstrap-evidence` would admit durable generation-1 evidence whose `approvers` is a bare scalar rather than the list of protected-environment logins it is supposed to attest — and the protected path never consults the JSON Schema that would otherwise catch the type. Replace or supplement `:197` with `"approvers": "abc"` (or `{"a": 1}`).

### 2. The pass-7 ledger entry asserts an isolation the suite does not support — Low

`plans/issues/active/phase-40-stable-channel-ga-execution.md:418-424` records pass 7 as remediated with "direct guards for … approver container/value types". The *value* half is true (`:234`, mutant 46, killed). The *container* half is not, per finding 1. This is the third consecutive round in which the ledger entry claims an isolation one guard broader than the suite delivers (pass-6 F3 against pass 5, pass-7 F4 against pass 6). Correct it alongside the remediation rather than leaving the durable record ahead of the gate.

## Not findings

- 32 surviving mutants re-confirmed masked by a pinned sibling, as enumerated above. Correctly ordered defense-in-depth.
- Merge-profile selection is 56 variants this round vs pass-7's 55; the delta is an added case in the area manifest, and failures remain 0 with `schema-v2-preview-epoch-bootstrap` appearing exactly once (the `full`+`epoch-bootstrap` dedup from pass 3 still holds).
- `ruff format --check` would still reformat several files in the package; wired into no repo gate, pre-existing baseline.
- The four cross-field semantics 2020-12 cannot express remain validator-stricter and semantically covered.
- Post-`gh release create` failures remain unrecoverable by re-run; the `bootstrap-index` evidence upload is necessarily last. Fail-loud stays correct.
- The fifth smoke output `${out}/stable-dispatcher.sh` is correctly outside `SMOKE_IDS`.
- `A && B || C` at `release-publication.yml:175` and `release-publication-prepare.yml:74` is correct for both false branches.

## Commit mechanics and downstream execution requirements (not implementation defects)

- `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-8.md` is a 0-byte untracked file. It must receive this report, be archived, and be ledgered (passes 1–7 are at `:361-424`). I did not modify it, per instruction.
- Downstream, not implementation prerequisites: `stable-release` environment with ≥1 `release/distribution` reviewer and "prevent self-review" enabled; no reviewers on the auto-created `preview-release` environment; live `channels.json` still exactly 105 bytes / `71b3243925…4bf9ef` at `bootstrap-index` time; `sifr.sh` serving new dispatcher bytes inside the 180-second budget.
- Still not locally reproducible: whether `actions/download-artifact@v4` resolves an attempt-1 artifact during "Re-run failed jobs". Validate on the first live re-run.

## Whole-wave verdict

The wave is one mutation case from done. Every structural, workflow, profile, schema, and producer-path finding from passes 1–7 is closed and re-measured green here, and the pass-7 sweep's load-bearing gap has gone from 10 guards to 1. The remaining defect is a test-coverage gap in the merge-gated suite, not a behavior defect — all guards are correct on the current code — but it is the same masking pattern that has now survived four consecutive rounds, and it sits on the guard that makes the approver attestation structurally checkable.

VERDICT: NOT SATISFIED
