# Satisfied Independent Review — Phase 40 / Milestone 40.5 Incident-Publication Wiring (PR #3047, pass 9, final exact-PR-head)

## 1. Identity verification

| Check | Value | Result |
|---|---|---|
| Local `HEAD` | `dabdfec856b1e9a31ea5f95201de84c7cb70402c` | ✓ |
| `origin/codex/phase-40-milestone-40-5-incident-publication-wiring` | `dabdfec856b1e9a31ea5f95201de84c7cb70402c` | ✓ |
| `gh pr view 3047 --json headRefOid` | `dabdfec856b1e9a31ea5f95201de84c7cb70402c` | ✓ identical |
| Branch / base / state | expected branch → `main`, `OPEN`, `MERGEABLE` | ✓ |
| Merge base vs `main` | `0f59a48b30160691c6cf047d987d1aeb978724dd` | ✓ |
| `HEAD` ancestor of `origin/main` tip (`401c53971`) | no — `main` advanced by unrelated Rust-interop merges | non-blocking (obs. 1) |
| Worktree | clean except this pass's own untracked 0-byte slots (`…pass-9-final-exact-pr-head.md`, `.claude.log`); no active review `.md` tracked | ✓ no implementation drift |
| Diff scope | 64 files, +6343 / −620 vs `0f59a48b3`; `git diff --check` clean | ✓ |
| Commits | `8776b4dbb`, `341b312f5`, `dabdfec85` | ✓ |

**Reviewed SHA: `dabdfec856b1e9a31ea5f95201de84c7cb70402c`.**

## 2. Delta since pass 8 — documentation-only, faithful

`git show dabdfec85` is the entire delta over pass 8's head `341b312f5`. It touches exactly two files, both Markdown, +105/−0, no deletions:

- `plans/issues/active/phase-40-stable-channel-ga-execution.md:822-829` — new ledger bullet.
- `plans/reviews/archive/…-review-pass-8-exact-pr-head-satisfied.md` — new, 97 lines.

No workflow, script, schema, Python, or fixture byte changed. Verified against the archived report itself:

| Ledger assertion (`:822-829`) | Archived pass-8 report | Verdict |
|---|---|---|
| "satisfied and archived at `plans/reviews/archive/…pass-8-exact-pr-head-satisfied.md`" | file exists at that exact path, ends `VERDICT: SATISFIED` | accurate |
| "matched local, remote, and PR head at `341b312f50de61c549f1bde01a6676f248231d02`" | identity table records that SHA in all three rows; it is this PR's 2nd commit, i.e. the then-head | accurate |
| "reproduced the missing/forbidden incident-binding schema and runtime cases plus both operator-facing validator kinds" | report's parity table + CLI transcript for both `stable-publication-prepare` and `incident-publication-prepare` | accurate; independently re-reproduced below |
| "reran the focused publication/recovery suites and guardrails" | validation table (68 variants, 8/8, 14/14, 5/5, 2/2, 8/8, 2/2, rc=0, guardrail PASS) | accurate; re-measured 68/0 below |
| "found no actionable finding" | "## Actionable findings — None." | accurate |
| "returned `VERDICT: SATISFIED`" | final line | accurate |

Stale-assertion sweep on the delta: the report's self-referential observation #8 ("populate this pass's own artifact slot and archive before merge") is **satisfied by this very commit** — the active slot `…pass-8-exact-pr-head.md` is gone from `plans/reviews/active/` (only its `.claude.log` remains) and the content landed under `archive/`. Its observation #1 (focused selection is 68, not 69) matches my own measurement. Its reference to the "`governance-contracts` variant" is precise: that is the variant label at `verification/areas/distribution_release/runner.py:217`, not a suite filter. The unchecked ledger item `- [ ] Merge the protected rollback and incident roll-forward production wiring.` correctly remains open while the PR is open. **No stale or false assertion found in the delta.**

## 3. Re-evaluation of the full Phase-40 incident-publication implementation

### Correctness & schema/runtime parity
- `validate_incident_prepare_summary` (`incident_prepare.py:240-367`) calls `require_exact_keys` with every subsequently-indexed key before any indexing, so no malformed-input `KeyError` path of the pass-7 class exists on the new surfaces. Same for `validate_incident_mutation_evidence` (`:482-541`) and `validate_incident_signoff` (`incident.py:33-190`, with `release_signoff_sha256` added to `SIGNOFF_REQUIRED`).
- `incident_publication_prepare.schema.json` and `stable_incident_signoff.schema.json` mirror the runtime in both conditional directions (`rollback ⇒ release_prepare == "none"` / `else ⇒ $ref stable prepare`; `rollback ⇒ release_signoff_sha256 == "none"` / `else ⇒ sha256`). Runtime is equal-or-stricter (e.g. `affected.version != successor.version` at `:306`, `plan_sha256 == successor_plan_sha256` at `:516`, generation floor at `:537`).
- Pass-7's fix at `stable_prepare.py:366-388` is present and both directions re-reproduced live at this SHA:

```
--kind stable-publication-prepare, roll-forward summary minus incident
  → release-governance: $: missing required field(s): incident        rc=2, no traceback
--kind stable-publication-prepare, normal summary plus incident
  → release-governance: $: unknown field(s): incident                 rc=2, no traceback
```

- All three new/extended operator validator kinds probed with three malformed payloads each (9 cases): every one exits **2** with a governed `$…: missing required field(s)` diagnostic and **zero** tracebacks.

### Fail-closed behavior & security
- `.github/workflows/release-publication.yml`: top-level `contents: read`; exactly one `contents: write`, on the `publish` job (`:118-120`); prepare and drill jobs read-only. Environment gate `stable-release` for all non-preview modes (`:117`). Mutation lease `sifr-release-index`, `cancel-in-progress: false` (`:87-89`). All actions SHA-pinned; every checkout `persist-credentials: false`.
- Credential hygiene in `run_incident_publication.sh`: `SITE_TOKEN`/`VSCE_PAT` captured then `unset` at `:116`; every installer, smoke, and recovery invocation is scrubbed (`GH_TOKEN="" SITE_TOKEN="" VSCE_PAT=""` at `:328,498,521`); Marketplace publish narrows to `VSCE_PAT` only (`:355`); `run_incident_public_recovery.sh:37` re-`unset`s internally as defense in depth.
- Injection safety: retained Marketplace publisher/extension are regex-gated (`:381-385`) before URL interpolation; all identity inputs are anchored-regex validated at `:81-109`; every consumed path is checked non-symlink and inside its checkout.
- Protected-main binding: `HEAD == workflow_commit == origin/main` (`:127-136`), plus `merge-base --is-ancestor` for incident evidence and, for roll-forward, candidate evidence and candidate source (`:137-160`). Prepare enforces the same ancestry independently (`release-publication-prepare.yml:353-357`, `:315-319`).
- Mode cross-exclusion is enforced in both directions at both layers: `rollback` must not receive candidate inputs (script `:108`; prepare `:497-500`), non-incident stable prepare must not receive incident inputs (`:347-352`), preview/bootstrap prepare must not receive any stable/incident input (`:130`).

### Exact evidence custody & atomic mutation
- `revalidate_incident_publication.py:26-76` re-derives the entire prepare summary from source evidence and requires **byte equality** with the digest-pinned bytes — run twice, before staging and again immediately before index reservation (`run_incident_publication.sh:297,421`).
- Exactly one `--clobber` in the incident path (`:439`), on `channels.json` only, immediately preceded by a `cmp` proving the live index is unchanged since reservation (`:429-437`) and followed by a re-download digest assertion (`:444-451`).
- Write-once discipline: `upload_or_verify_governance` refuses pre-existing assets unless explicitly resumed, and on resume `cmp`s bytes (`:208-234`). The generation snapshot is uploaded with `allow_existing=false` (`:428`) — safe because `allocate_next_generation` accounts for orphaned snapshots, so a crashed prior attempt yields a fresh generation rather than a name collision. Signoff asset names embed `run_id-run_attempt`; the site-facts asset name embeds the generation and is resume-tolerant, which is exactly what the `activated` state requires.
- `_smoke_evidence` (`incident_publish.py:288-318`) requires the exact 8-file set, rejects symlinks, byte-compares the public governed index against the prepared proposed index, and requires `incident-recovery.json` to equal the prepare-derived tuple field-for-field.

### Rollback and roll-forward recovery
- `_materialize_or_recover` / `_recover_realized_mutation` (`incident_prepare.py:370-436`) only accept a predecessor snapshot whose re-materialized proposed index is **byte-identical** to the live index, and only under `mode=resume`; `publication_state == "activated"` is rejected outside resume (`:277-278`).
- `run_incident_public_recovery.sh` drives real clients: rollback proves the unforced downgrade is refused, that the refusal names `--force`, then that `--force` and an out-of-band dispatcher install both converge; roll-forward proves plain `self update` and out-of-band install converge. Both paths then assert `sifr --version` and a schema-validated `install.json` bind the successor version and `stable` channel.
- Marketplace semantics for rollback are correct and non-destructive: the retained extension is fetched and required to match the affected plan's digest/version, with the new `--compiler-version` check requiring its `sifrCompilerCompatibility` range to contain the rollback target (`verify_marketplace_vsix.py:90-97`).

### Public site recovery
- `stage_incident_publication` regenerates site facts from the proposed index only, never from prose; validates dispatcher digests against the operation's approved site plan; and for rollback additionally requires the target and affected plans to agree on all four dispatcher digests before any output is created (`incident_publish.py:68-86`).
- The public docs smoke polls `/releases/stable` for up to 180 s and requires the rendered page to carry the active stable version plus every withdrawal's version and incident id (`run_stable_public_smoke.sh:120-140`, `verify_public_stable_docs.py`). Site facts are also revalidated against the **downloaded public** index (`--kind site-facts --live-index`).
- Cross-repository pins are consistent across all three places: ruleset `19899766` @ `2026-07-28T13:22:41.496Z`, workflow digest `a9360c82…02b3af`, tag `sifr-release-site-stable-facts`, commit `ff472f2a…` — identical in `release-publication.yml:146-149`, `preview-release.yml:73`, and `fixtures/site_release_contract.json`.

### Scope discipline
Zero compiler-crate, `demos/*.sifr`, or Rust files touched. The diff is confined to `.github/workflows/`, `scripts/distribution/`, `verification/areas/distribution_release/`, `verification/runner/sifr_verify/selftest.py` (schema count 16 → 18), `demos/stable_release_governance_demo.sh`, `internal_docs/`, and `plans/`. No Rust-interop implementation reviewed or requested.

### Validation executed on this exact head

| Check | Result |
|---|---|
| `--suite full --suite evidence-custody` | **68 variants / 0 failures** |
| `--suite incident-governance --suite stable-publication --suite stable-prepare` | 6 variants / 0 failures |
| `incident_publication_selftest` / `incident_public_recovery_selftest` | 5/5 · 2/2 |
| `stable_prepare_selftest` / `stable_publish_selftest` / `stable_public_smoke_selftest` | 8/8 · 8/8 · 2/2 |
| `governance.selftest` (incl. schema contracts) | 14/14 |
| `incident_publication_workflow_contract.sh` / `site_release_workflow_contract.sh` / `stable_publication_workflow_contract.sh` | rc=0 · rc=0 (`PASS`) · rc=0 |
| `sifr_verify` runner self-test (`run_all`) | 11 checks pass; 18 governance schemas on disk, matching `selftest.py:87` |
| `scripts/check_file_size_guardrails.py` | PASS (2936 files, limit 900) |
| `demos/stable_release_governance_demo.sh` | rc=0, capability-named |
| 9 malformed-input probes across the 3 new/extended validator kinds | all rc=2, governed diagnostic, no traceback |

Existing full local validation (`run_all_tests.sh --profile create-pr`) is recorded in the ledger and was treated as consumed evidence.

## 4. Actionable findings

**None.** No blocking or non-blocking *actionable* defect survived verification at this SHA.

## 5. Non-blocking observations

1. **Branch is behind `main`.** `HEAD` is not an ancestor of `origin/main` (`401c53971`); `main` advanced by unrelated Rust-interop merges after the merge base. GitHub reports `MERGEABLE`; a merge commit or rebase is coming. No governance impact — the workflow's `HEAD == origin/main` assertions apply to production runs on `main`, not to this PR. *(Recurrence of pass-8 #2.)*
2. **Docs smoke is substring containment.** `verify_public_stable_docs.py:31-41` checks `value not in document` over raw HTML, so a version like `1.0.0` could be satisfied by unrelated page text; the check proves presence, not placement. Mitigated by the site-side pinned renderer digest and the AST-parsed `RENDERED_LABELS` equality in `site_release_workflow_contract.sh`. Cross-repository; not introduced by this wave's Python.
3. **Retained-VSIX fetch has no retry.** `run_incident_publication.sh:397-399` is a single `curl` (120 s cap) while adjacent public probes poll to convergence. A transient Marketplace CDN blip aborts a rollback — but strictly *before* any mutation, so it is fail-closed and simply re-runnable. Robustness only.
4. **No negative for the rollback-specific recovery guards.** `incident_public_recovery_selftest.py:71-93` iterates `incident-roll-forward` only. The rollback *positive* path does exercise `run_incident_public_recovery.sh:57-64`; what has no executed negative is a client that wrongly *permits* an unforced downgrade, or a refusal message lacking `--force`. *(Recurrence of pass-8 #6.)*
5. **Pass-6 #1 residual.** The rollback dispatcher-provenance gate remains only in `stage_incident_publication` (`incident_publish.py:77-85`); `materialize_incident_prepare` validates both plans and could surface the disagreement in the reviewer-visible prepare summary. Both placements are pre-mutation and fail-closed.
6. **Pass-5 #2 residual.** `site_release_contract.json` pins `renderer_sha256` without a live verification analogous to `workflow_sha256`. Cross-repository.
7. **Pre-existing looseness, not introduced here.** `validate_stable_prepare_summary` still accepts arbitrary strings for `release_prepare.marketplace.{publisher,extension,version}`, `artifacts.*.name`, and `release_report.id` — identical for `normal`, so out of scope.
8. **Cosmetic.** `run_incident_publication.sh:137` is a single-element `for ancestry in …` loop, retained for symmetry with the three-element roll-forward loop at `:148-159`.
9. **This pass's own artifact slot** (`plans/reviews/active/…pass-9-final-exact-pr-head.md`) is 0 bytes. Per instruction I modified no repository file; populate it from this report and archive alongside pass 8 before merge.

## 6. Conclusion

Local, remote, and PR head are identical at `dabdfec856b1e9a31ea5f95201de84c7cb70402c`. The delta over the previously satisfied pass 8 is strictly documentation — one ledger bullet and the archived pass-8 report — and every claim in it checks out against the report it records, with no stale or false assertion. The full Phase-40 incident-publication implementation re-verifies clean on correctness, security, fail-closed behavior, schema/runtime parity, rollback and roll-forward recovery, exact evidence custody, public site recovery, tests, workflow permissions, and scope discipline. Focused validation reproduces 68 variants with zero failures at this exact head, matching the recorded figure. No actionable finding.

VERDICT: SATISFIED
