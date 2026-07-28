# Exact-PR-head review result

## Identity verification

| Check | Value | Result |
|---|---|---|
| Local `HEAD` | `8776b4dbbec3d4b342c937dd1a6b4effaabca5aa` | ✓ |
| `origin/…-incident-publication-wiring` | `8776b4dbbec3d4b342c937dd1a6b4effaabca5aa` | ✓ |
| `gh pr view 3047 --json headRefOid` | `8776b4dbbec3d4b342c937dd1a6b4effaabca5aa` | ✓ |
| Base / PR base | `main` / `main`, `MERGEABLE`, `OPEN` | ✓ |
| Merge-base vs `origin/main` | `0f59a48b3` == `origin/main`; `origin/main` is an ancestor of HEAD | ✓ linear on protected main |
| Diff scope | 62 files, +6053/−603 | ✓ matches the stated 62 |
| Implementation drift | none — the only untracked path is this pass's own 0-byte review slot | ✓ |

Reproduced locally at this SHA: full `distribution_release` **125 variants / 0 failures**; `--suite full --suite evidence-custody` **68 / 0**; incident-publication 5/5 (including the pass-5 dispatcher-provenance negative); incident-public-recovery 2/2; stable-prepare 8/8; stable-publication 8/8; stable-public-smoke 2/2; governance selftest 14/14; all five publication/site workflow contracts PASS; capability demo exit 0 (filename remains capability-based, no phase/milestone identifier); file-size guardrails **PASS (2936 files, limit 900)**; `git diff --check` clean.

All pass-1→pass-6 closures re-derived and confirmed, including pass 5's finding: `incident_publish.py:77-85` now rejects rollback when the target and affected plans disagree on `site.dispatcher_sha256`, placed before `_validated_dispatchers` and before `output_root.mkdir()` (`:86-87`), and before every mutation in `run_incident_publication.sh` (stage `:409` → request asset `:417` → snapshot `:428` → sole `--clobber` `:438`). The rationale comment pass 6 asked for is present at `:75-76`.

## Actionable

**1. `validate_stable_prepare_summary` raises an unhandled `KeyError` instead of a governance diagnostic when an `incident-roll-forward` summary omits `incident` — reachable through two operator-facing validator kinds.**

`verification/areas/distribution_release/governance/stable_prepare.py:364` makes `incident` *optional* rather than required:

```python
optional = {"incident"} if operation == "incident-roll-forward" else set()
```

so `require_exact_keys` (`common.py:71-76`) permits its absence. Line **578** then indexes it unconditionally:

```python
if operation == "incident-roll-forward":
    incident = require_object(summary["incident"], "$.incident")   # KeyError
```

Both new call sites are exposed. Reproduced at this SHA:

```
$ scripts/distribution/release_governance.py validate \
    --kind incident-publication-prepare --input summary.json     # release_prepare.incident removed
rc=1 … incident_prepare.py:355 → stable_prepare.py:578 → KeyError: 'incident'

$ scripts/distribution/release_governance.py validate \
    --kind stable-publication-prepare --input s.json             # operation=incident-roll-forward, no incident
rc=1 … KeyError: 'incident'
```

I also fuzzed every leaf of both new fixtures (drop / null / list / str) through `validate_incident_prepare_summary` and `validate_incident_signoff`: this is the **only** crash — every other malformation fails cleanly, which is exactly why it stands out as a defect rather than a house style.

The JSON Schema handles this correctly (`stable_publication_prepare.schema.json:170-179`: `then: {"required": ["incident"]}` / `else: {"not": {"required": ["incident"]}}`), so the runtime is *looser than the schema on the required-key axis* and *crashes* on the input the schema rejects — the inverse of the equal-or-stricter parity the rest of this wave maintains.

Fail-closed in production (`set -euo pipefail` aborts on the non-zero exit, and `revalidate_incident_publication.py:198` / `materialize_incident_publication.py:76` catch only `GovernanceError`, so the traceback escapes but the run still stops before any mutation). The harm is diagnostics, not integrity — the same class as pass-1 #10, which was treated as actionable and fixed.

*Fix:* move `incident` into the required set instead of the optional set, mirroring the schema exactly:

```python
required = {
    "schema_version", …, "site",
} | ({"incident"} if operation == "incident-roll-forward" else set())
require_exact_keys(summary, required=required, location="$")
```

That yields `release-governance: $: missing required field(s): incident` and lets line 578 stay as-is. Add the paired negative — a roll-forward summary with `incident` deleted must be rejected with that diagnostic — next to the existing conditionals in `schema_negative_contracts.py`; there is currently no negative, at either schema or runtime level, for the new `stable_publication_prepare` incident conditional in either direction.

## Non-blocking observations

1. **Pass-6 #1 residual.** The rollback dispatcher-provenance gate lives only in `stage_incident_publication`. `materialize_incident_prepare` already loads and validates both plans (`incident_prepare.py:141-151`), so the same assertion there would surface the disagreement in the read-only, reviewer-visible prepare summary rather than after protected approval. Both are pre-mutation and fail-closed, and pass 5 explicitly asked for the stage-time placement — ergonomics only.
2. **Pass-5 #2 / pass-6 #2 residual.** `site_release_contract.json:494` pins `renderer_sha256`, but it is never verified live the way `verify_site_workflow_identity.sh` verifies `workflow_sha256`. The cheap half is done — `site_release_workflow_contract.sh:101-111` AST-parses `RENDERED_LABELS` out of `verify_public_stable_docs.py:364-367` and asserts equality with the fixture, so fixture and verifier cannot drift silently. An external renderer edit still fails the docs smoke 180 s after the clobber (`run_stable_public_smoke.sh:120-139`). Cross-repository.
3. **Rollback's own recovery guards have no executed negative.** `test_recovery_rejects_binary_and_receipt_drift` (`incident_public_recovery_selftest.py:71-104`) iterates only `incident-roll-forward`. `run_incident_public_recovery.sh:57-59` (`affected downgrade succeeded without --force`) and `:61-64` (the `grep -F -- "--force"` on the diagnostic) are proven to *pass* by the positive rollback iteration but are never driven to their failure branches. I independently confirmed the real client emits the required text — `self_update_metadata.rs:390-395` returns `"downgrading self-update from … requires --force"`, and `resolve_channel`/`resolve_exact` accept a `withdrawn` current release (`:471-483`), so the withdrawn affected version does not divert the diagnostic.
4. **Pre-existing looseness surfaced while fuzzing, not introduced here.** `validate_stable_prepare_summary` accepts arbitrary strings for `release_prepare.marketplace.{publisher,extension,version}`, `release_prepare.artifacts.*.name`, and `release_prepare.release_report.id`. Identical for `normal`, so out of this diff's scope; noting for awareness.
5. **Focused-selection count.** `--suite full --suite evidence-ербcustody` measures **68** variants / 0 failures at this SHA, matching the issue ledger's own "68 variants" figure rather than the 69 in the request summary. No failures either way.
6. `plans/reviews/active/phase-40-milestone-40-5-incident-publication-wiring-review-pass-7-exact-pr-head.md` is 0 bytes — this pass's own artifact slot, which I was instructed not to write. Populate from this report before merge. Pass 6's artifact was properly archived, so the pass-1 #7 / pass-3 #4 / pass-4 #4 / pass-5 #4 / pass-6 #4 recurrence is otherwise closed.

## Verified clean at this SHA

- **Exact-byte custody.** Request, withdrawal evidence, and both approved plans bound to tracked `HEAD` blobs via `_require_head_file` (`incident_prepare.py:93,94,565,585-594`), with `-uno` cleanliness commented at `:575-578`; `revalidate_incident_publication.py:161` requires byte-exact reproduction of the reviewer-visible summary; `verify_retained_stable_release.py:460-488` requires full inventory-set *and* digest equality plus `tagName`/`targetCommitish`/tag-object SHA == `plan.source_commit` and `isDraft`/`isPrerelease` both `False`; the affected qualification index is transitively bound through `plan["qualification_artifact_index"]["sha256"]`.
- **Protected-main ancestry.** `workflow_ref == refs/heads/main`, `HEAD == workflow_commit == freshly fetched refs/remotes/origin/main`, merge-base ancestry for incident evidence and (roll-forward) candidate evidence + candidate source (`run_incident_publication.sh:81-160`); prepare enforces the same against `governance-source` (`release-publication-prepare.yml:327-331,353-357`); rollback rejects successor inputs at `:108` and `incident_prepare.py:130-140`.
- **Mutation ordering / atomicity.** revalidate → approvers → pre-mutation site identity → retained-release verification → affected-client installs → dispatchers → (roll-forward: release + Marketplace) → stage → write-once request asset → re-fetch + revalidate → write-once snapshot → live `cmp` → **single `--clobber`** → activated-digest check → dispatch/poll → smoke → recovery → sign-offs. Ordering and `--clobber` count == 1 pinned by `incident_publication_workflow_contract.sh:68-102`; `contents: write` count == 1; one `publish:` job; `sifr-release-index` lease shared with preview/bootstrap, drills on a separate group and environment.
- **Write-once / resume.** `allocate_next_generation` skips burned generations, so a resumed *pending* attempt gets `N+1` and the `allow_existing=false` snapshot upload cannot collide; *activated* resume skips reservation and the clobber entirely and re-verifies live bytes against `proposed_sha256` (`:447`); sign-off names are attempt-scoped, site facts generation-scoped with `${allow_existing}`; the request asset is content-addressed so resume's `cmp` is exact; `initial` re-runs are rejected by `upload_or_verify_governance`.
- **Credentials.** `SITE_TOKEN`/`VSCE_PAT` captured then `unset` (`:110-116`, `dispatch_stable_site_publication.sh:95-100`); installer runs, smoke, recovery, and the Marketplace publish all scrub `GH_TOKEN`/`SITE_TOKEN`/`VSCE_PAT` at the call site, and the recovery script scrubs again internally (`:37,40,48`); the prepare workflow is `contents: read` with a read-only `github.token` and is asserted free of `contents: write`, `secrets.`, `gh release upload`, and `vsce publish`; `VSCE_BIN` required only for roll-forward, so rollback needs no `stable-source` checkout.
- **Workflow semantics.** `STABLE_MUTATION_OPERATION` / `STABLE_CANDIDATE_OPERATION` / `STABLE_PUBLICATION_OPERATION` / `INCIDENT_OPERATION` are job-level `env` consumed in step `if:` and partition the seven modes without overlap; `environment` resolves to `stable-release` for both incident modes; `default: ""` present on all six dispatch inputs (`:16-27`); `(.release_prepare | objects)` guards every jq extraction in `Bind prepare outputs`, and `version`/`plan_sha256` fall through to `.successor.*` for incident summaries; all four YAML files parse and all 40 `scripts/distribution/*.sh` `bash -n`.
- **Schema/runtime parity.** Runtime is equal-or-stricter everywhere else I compared — `plan_sha256 == successor_plan_sha256`, the generation floor, and the `pending`/`activated` lease bindings are runtime-only additions; both new incident conditionals and the expanded `stable_incident_signoff.site_reconciliation` + `release_signoff_sha256` conditional agree with `incident.py:131-192`; 18-schema lint count correct.
- **Cross-repo site identity / public recovery.** Site tag/ruleset/digest rotated coherently to `sifr-release-site-stable-facts` / `19899766` / `a9360c82…` / `ff472f2a…` across the workflow, the 13-input fixture, `preview-release.yml`, and both docs; identity is verified once pre-mutation (`run_incident_publication.sh:309`, `run_stable_publication.sh:228`) and again immediately before dispatch inside `dispatch_stable_site_publication.sh:102`, keeping `distribution_pipeline.md`'s wording true after the extraction; `generate_site_release_facts` derives `stable_version` and every withdrawal from the proposed index only.
- **Registration / docs.** New suites wired into both the dedicated and combined selection paths in `runner.py` and reached by the merge/nightly/release profiles' `incident-governance` and `stable-publication` suites; the new case script is auto-discovered; architecture, pipeline, and ledger text now match the code with no stale "adapters remain gated" claim; no Rust-interop work; everything stays inside Phase 40 stable-channel GA governance.

VERDICT: CHANGES_REQUIRED
