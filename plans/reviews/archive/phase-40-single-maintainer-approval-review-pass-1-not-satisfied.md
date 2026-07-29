## Review — PR #3060 @ `7e7081fef` (`sifr-lang/sifr`)

Verified state: head `7e7081fefa8344422da920494300df6b14c7d6eb`, PR open, base `main`, **`mergeable: CONFLICTING` / `mergeStateStatus: DIRTY`**. Diff base used: `origin/main` = `cad0e8aaf`.

I re-executed the runtime gates rather than trusting the validation claim. The core policy shape is right: `resolve-publication-approvers` still reads GitHub's own `/approvals`, and I confirmed by direct execution that it fails closed for every unwaived case — no waiver (`normal`): `requires an authorized stable-release approval`; waiver + `--operation normal|rollback|""`: `$.allowed_operations: does not authorize …`; wrong repo/env: rejected. `run_incident_publication.sh:299-303` never receives a waiver, so rollback and incident-roll-forward remain distinct-reviewer-only. Waiver mode accepts self-approval only for `ga-activation`/`bootstrap-*`. Schema↔runtime parity for `approval_policy` is real (`oneOf` mode/digest pairing in both schemas mirrors `validate_approval_policy`), the runner inventory count was bumped 18→19, and the checked-in waiver validates against both its JSON Schema and the runtime validator with `--require-canonical`.

Five actionable findings remain.

### 1. Blocker: head is based on stale `main` and reverts #3058's path fixes → broken doc links after merge
`plans/phases/index.md:52-54` and `plans/roadmap.md:87` on this head point GENC-NAN / PKG-RUST / PERF-HOST at `./adhoc_*.md` (i.e. `plans/phases/…`) and `./phases/adhoc_generated_nan_constant_clippy_quality.md`. On `main` those files live only at `plans/issues/active/adhoc_*.md` (`git ls-tree origin/main` confirms). `git merge-tree origin/main 7e7081fef` produces real content conflicts in exactly those two files. The local docs-links pass is an artifact of the stale base — post-merge these are dangling links, and it silently reverts an unrelated merged change.
**Remediation:** rebase onto `cad0e8aaf`, keep `main`'s `../issues/active/…` paths, add only the new `REVIEWER-RESTORE` rows, re-run the gates.

### 2. Bootstrap reads the waiver from an unpinned, caller-chosen tree, with no expected digest
`.github/workflows/release-publication.yml:150` uses a repo-relative path; for `bootstrap-*` the workspace root is checked out at `inputs.source_commit` (`:152-157`), which flows from `preview-release.yml`'s free-form `base_ref` input. So the bytes hashed at `:243-246` and validated at `:250-258` come from whatever ref was dispatched — not from protected `main`. I confirmed a waiver copy with `expires_at: 2099-01-01T00:00:00Z` is accepted (`rc=0 → ["yaseralnajjar"]`); its digest is then recorded into evidence as authoritative. This defeats the expiry — the only mechanism making the exception temporary — and it's the one place this workflow abandons its own pinning convention (`SITE_WORKFLOW_SHA256`, `PREPARE_SUMMARY_SHA256`, `--expected-plan-sha256`). Note the `ga-activation` path is correct here: `STABLE_MUTATION_OPERATION` checks out protected `main` (`:158-162`), so only bootstrap is exposed.
**Remediation:** add `SINGLE_MAINTAINER_APPROVAL_WAIVER_SHA256` as a pinned env constant and verify the computed digest against it before `resolve-publication-approvers` (or resolve the waiver from a protected-main checkout), and pin that literal in `schema_epoch_bootstrap_workflow_contract.sh`.

### 3. `approval_mode` is asserted from the operation, not from the approval that happened — resolver and validator disagree
`release-publication.yml:242,260-266` sets `approval_mode=single-maintainer-waiver` for *every* `bootstrap-*` run, and `run_stable_publication.sh:227-236` does the same for every `ga-activation`. But the resolver in waiver mode deliberately accepts extra approvers (`schema_bootstrap.py:118-123`), while `_validate_approvers` (`:267-279`) rejects anything other than exactly `[initiator]`. Reproduced:

| approvals | resolver | evidence validator (mode=waiver) |
|---|---|---|
| owner only | `["yaseralnajjar"]` | accepted |
| owner + second reviewer | `["second-maintainer","yaseralnajjar"]` | **rejected** |
| second reviewer only | `["second-maintainer"]` | **rejected** |

For `bootstrap-alpha` the failure lands at `:565-580`, *after* `gh release create` and asset verification at `:512-563` — so one extra legitimate `stable-release` approval aborts the run with the public release already irreversibly published and no retained protected evidence. Before this PR extra approvers were harmless. It also means the evidence states a policy the run didn't necessarily follow.
**Remediation:** have `resolve-publication-approvers` emit the selected mode alongside the approver list (waiver only when the sole authorized approver is the owner; otherwise `distinct-reviewer`), and reject the mismatch in the resolver — before any publish step — instead of only at materialization.

### 4. Stable sign-off's `approval_policy` is unbound to who approved
`stable_release_signoff.schema.json` required keys are `[schema_version, version, plan_sha256, approval_policy, attempts, published_assets, marketplace, channel_generation, site_publication, site_facts_sha256, post_publication_smoke]` — no `initiator`. So `validate_release_signoff` (`release_plan.py:342`) cannot cross-check `approval_policy.mode` against `attempts[].approver`, and a sign-off asserting `distinct-reviewer` with a self-approval (or `single-maintainer-waiver` with a distinct approver) validates cleanly. Bootstrap evidence gets this right; GA sign-off — the artifact that will actually record the waived first GA — does not.
**Remediation:** add `initiator` to the sign-off schema/materializer and enforce mode↔approver: waiver ⇒ `approver == initiator`, distinct ⇒ `approver != initiator`.

### 5. Insufficient direct negative coverage of the operation boundary and of the canonical waiver
- No case validates `plans/releases/single-maintainer-approval-waiver.json`. `stable_orchestrator_selftest.py:78-82` only passes its path; that run aborts earlier on `candidate source commit must be merged into protected main`, so the file is exercised by nothing but a `-f` test. The mandated expiry `2026-08-27T00:00:00Z` and the exact three-operation scope are asserted only in prose and in a synthetic fixture (`approval_waiver_selftest.py`) that isn't the real file.
- The `validate --kind single-maintainer-approval-waiver` wiring (`release_governance.py:313-320`) is dead — no case invokes it, and it passes `operation=None` with `require_unexpired` defaulted off, so it would not catch expiry or scope anyway. Nothing fails or warns when the waiver expires; discovery mode is a failed release run.
- The contract cases assert only positive presence of `--single-maintainer-waiver` / `--approval-mode` / `--approval-waiver-sha256` (`stable_publication_workflow_contract.sh:83-85`, `schema_epoch_bootstrap_workflow_contract.sh:49-51`). Nothing pins the `[[ "${operation}" == "ga-activation" ]]` guard, the `approval_mode=distinct-reviewer` else branch, or that `run_incident_publication.sh` never receives the flag — so widening the waiver to `normal` or to the incident orchestrator would pass every gate.

**Remediation:** add a case that loads the real waiver, asserts canonical bytes, schema conformance, and `validate_single_maintainer_waiver(operation=<each of the three>, require_unexpired=True)` against the mandated expiry/owner/repo/env; and add the three negative contract assertions above.

### Non-blocking notes
- `verification/areas/distribution_release/governance/schema_contracts.py` is 893 lines and `.github/workflows/release-publication.yml` is 890 — under the 900 cap but with almost no headroom for the fixes above.
- `run_stable_publication.sh:229` / `release-publication.yml:825-843` expand possibly-empty arrays under `set -u`; fine on bash ≥ 4.4 (runners and this host are 5.x), and the empty (`normal`) branch is untested. Cosmetic today.
- Operator/public/internal docs (`docs/releases/stable.mdx`, `internal_docs/distribution_pipeline.md:688-695,734-739`, `internal_docs/architecture.md:1462-1466`) and the follow-up issue accurately describe the narrow exception and do not claim distinct review is unconditional; the archived pre-exception audit is correctly framed as historical.

Findings 1–3 are hard blockers; 4–5 are required to make the checked-in code truthfully support the `prevent_self_review=false` / `can_admins_bypass=false` configuration you plan to apply.

VERDICT: NOT SATISFIED
