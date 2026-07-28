# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 7)

I read the full diff, all 20 untracked files, the phase/issue ledger, and archived passes 1–6, then ran the requested single-guard deletion sweep **across the entire semantic bootstrap validator** rather than the seven guards pass 5 happened to name.

## Method

I generated 83 single-edit mutants of `schema_bootstrap.py` mechanically from its AST — one per boolean clause of every `if …: fail(…)`, one per dropped `require_*` statement, one per bypassed `x = require_*(y, …)` → `x = y` — ran `governance.schema_bootstrap_selftest` against each, and for every mutant the suite failed to kill, re-ran a 29-case bad-input battery against both entry paths (`validate_bootstrap_evidence`, and `materialize_bootstrap_evidence` at *both* the `alpha-assets` and `preview-index` stages) to separate genuinely load-bearing guards from ones masked by a sibling surface.

**Result: 42 of 83 mutants survive the gate; 10 of those are load-bearing.** Every guard is correct today — all 29 bad inputs are rejected on the current code. The gap is entirely in what the merge-gated suite pins.

## Pass-6 re-audit

| # | Pass-6 finding | Status |
|---|---|---|
| 1 | Exactly-four `public_smoke` length guard unpinned | **Resolved.** `schema_bootstrap_selftest.py:198` adds the short-list `pop()`; deleting `schema_bootstrap.py:209-210` now kills the suite. |
| 2 | `release["status"] != "active"` unpinned; withdrawn case rejected upstream | **Not resolved** — see finding 1. |
| 3 | Ledger asserts an isolation the suite does not support | **Resolved for pass 5's entry, recurs in pass 6's** — see finding 4. |
| 4 | Five further guards unpinned (`approvers: []`, `run_attempt`, index digest, alpha-stage prepare digest, `release_record_sha256`) | **Resolved.** All five mutants are now killed (`:160,158,167,175-177,181-183`). |

All pass-1/2/3/4/5 remediations remain: `--clobber` pinned at 1; two `verify_site_workflow_identity.sh` call sites; `release-publication.yml` at 795/900; `epoch-bootstrap` in merge/nightly/release + matrix + `release_report`; no schema-v1 parser/fixture/migration/fallback; the four-surface legacy-identity gate; smoke-filename freeze.

**Gates re-run:** `schema_bootstrap_selftest` PASS; `epoch-bootstrap` suite PASS (variants=1); merge-profile distribution selection PASS (**variants=55, failures=0**); `governance.selftest` PASS; all three contract cases PASS; file-size gate PASS (2898 files) and self-test PASS; `git diff --check` clean; `bash -n` over all `scripts/distribution/*.sh` clean. Two I could not reproduce: `ruff` is not resolvable under the active pyenv shim, and `coverage_matrix` needs the `uv` project env (`ModuleNotFoundError: sifr_verify`) — I verified `profile_assignment_matrix.json`'s three added rows by inspection instead. The working tree is unmodified; every mutant was reverted.

## Findings

### 1. The `release.status != "active"` guard is still unpinned — pass-6 finding 2 recurs, and the guard *is* load-bearing — Low-Medium

`schema_bootstrap.py:469-470`. Measured: deleting both lines leaves `governance.schema_bootstrap_selftest` at **rc=0**.

Pass 6 diagnosed the missing `incident_id`; that was added (`schema_bootstrap_selftest.py:100`), but the case still runs through `build_preview_epoch`, and with the guard deleted the input is caught one layer down by `validate_release_index`:

```
$.channels.alpha: must point at an active matching release
```

So the retained mutation exercises `release_index.py`, not the bootstrap-local guard — the same masking shape as the length/dedup pair pass 6 called out, one surface over. Because `validate_release_index` always re-checks channel heads, **no input routed through `build_preview_epoch` can ever isolate this guard.**

The guard is not redundant, though. `_validate_release_wrapper` is also reached from `_materialize_release_evidence`, where no index validation follows. With `:469-470` deleted I confirmed:

```
alpha-assets producer, withdrawn alpha record + incident_id "inc-2026-001"
  baseline → rejected: alpha release.release.status: must be active
  guard deleted → ACCEPTED, durable evidence written
```

i.e. `bootstrap-alpha` would materialize write-once generation-1 alpha evidence attesting a withdrawn release. The isolating case must call `materialize_bootstrap_evidence(stage="alpha-assets", …)` with a withdrawn-but-valid alpha record, not `build_preview_epoch`.

### 2. The producer's source-commit binding is unpinned — Low-Medium

`schema_bootstrap.py:437-438`. Deleting leaves the suite green, and the alpha-assets producer then **accepts** a release record whose `source_commit` contradicts the workflow-supplied `--alpha-source-commit`, recording the supplied value in durable evidence. Nothing downstream catches it: `require_commit` at `:429` and `:489` only check the 40-hex *format*, and `_validate_release_evidence` never sees the record. This is the field that makes generation-1 evidence attributable to a reviewed commit.

### 3. Eight more load-bearing guards in the same function set are unpinned — Low

Each of these single-edit deletions leaves `governance.schema_bootstrap_selftest` green, and each admits a concrete bad input I confirmed is rejected today:

| `schema_bootstrap.py` | guard | admitted once removed |
|---|---|---|
| `:435-436` | `record_version != version` | producer accepts a record whose `version` differs from the evidenced one |
| `:487-488` | `version_channel(version) != channel` | `$.alpha` carrying a beta version and `$.beta` carrying an alpha version both accepted |
| `:460` | `require_exact_keys(wrapper, {"version","release"})` | release wrapper with an arbitrary extra top-level key accepted |
| `:42` (size clause) | `size_bytes != LEGACY_INDEX_SIZE_BYTES` | 104-byte legacy index with the correct digest accepted (the digest clause *is* pinned) |
| `:227` | `require_array(payload, location)` in `_validate_approvers` | `"approvers": "x"` accepted |
| `:234` | `require_nonempty_string(raw, …)` | `"approvers": [""]` accepted |
| `:92` | `require_nonempty_string(initiator, …)` | `resolve_distinct_approvers(…, initiator="")` accepted — self-approval can no longer be excluded |
| `:111` | `require_nonempty_string(user.get("login"), …)` | an approval whose `user.login` is `""` counted as a distinct approver |

The last two matter most: they are what makes "a protected, non-initiating human approved this" a checkable claim rather than a formatting convention. Eight mutations of the same shape as the tuple entries already at `:153-211` close all of these; findings 1 and 2 need producer-path cases instead.

### 4. The pass-6 ledger entry again asserts an isolation the suite does not support — Low

`plans/issues/active/phase-40-stable-channel-ga-execution.md:408-415` records pass 6 as "remediated by isolating the short-smoke and valid-withdrawn-release cases." The short-smoke half is true (verified). The withdrawn half is not, per finding 1. Pass 6 raised exactly this against pass 5's entry and it was corrected; the correction re-introduced the same overclaim one entry down. This is the durable record future rounds re-audit against.

## Not findings

- **32 surviving mutants whose removal I could not make observable.** `:268` (producer legacy-identity custody — masked by `$.legacy_index.sha256` in the final `validate_bootstrap_evidence`), `:310` (producer generation-1 — masked by `$.index.generation`), `:388`/`:397` clauses (masked by the pinned `index["releases"]` equality and by `validate_release_index`), `:429` (`require_commit` — re-checked at `:489`), `:462` (masked by `validate_release_record`'s own channel check), and the seven `:294` `is None` clauses (each masked by its six siblings in the all-omitted test). Defense-in-depth, correctly ordered; not worth mutation cases.
- The 14 `require_object`/`require_array` type guards fail closed when removed, but via an uncaught `TypeError` rather than a governed diagnostic. Structural typing is covered by the JSON Schema, which the protected path never consults — worth knowing, not worth a finding.
- Schema-side: `duplicate_smoke`, `extra_asset` (validly named tenth asset), and `alpha_with_beta` remain the three load-bearing negatives pass 6 measured; `schema_contracts.py:66-95` is unchanged.
- The four cross-field semantics 2020-12 cannot express remain validator-stricter and semantically covered.
- Post-`gh release create` failures remain unrecoverable by re-run; the `bootstrap-index` evidence upload is necessarily last. Fail-loud stays correct.
- The fifth smoke output `${out}/stable-dispatcher.sh` is correctly outside `SMOKE_IDS`.

## Commit mechanics and execution requirements (not implementation defects)

- `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-7.md` is a 0-byte untracked file. It must receive this report, be archived, and be ledgered (passes 1–6 are at `:361-415`). I did not modify it, per instruction.
- Pass-1's external requirements stand: `stable-release` environment with ≥1 `release/distribution` reviewer and "prevent self-review" enabled; no reviewers on the auto-created `preview-release` environment; live `channels.json` still exactly 105 bytes / `71b3243925…4bf9ef` at `bootstrap-index` time; `sifr.sh` serving new dispatcher bytes inside the 180-second budget.
- Still not locally reproducible: `actions/download-artifact@v4` resolving an attempt-1 artifact during "Re-run failed jobs". Validate on the first live re-run.
- `ruff` and the `coverage_matrix` area could not be executed in this environment (see above); both were green in pass 6 and the relevant files are unchanged, but I am not vouching for them independently this round.

VERDICT: NOT SATISFIED
