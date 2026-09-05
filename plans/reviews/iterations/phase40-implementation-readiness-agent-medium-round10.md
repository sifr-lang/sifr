# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 10)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (975 lines) against the nine artifacts in `plans/reviews/iterations/` (`round10.md`/`.log` are zero-byte placeholders for this run) and live repository evidence: `.github/workflows/` (`local-first-validation.yml`, `preview-release.yml` only; `release-publication.yml` absent), `verification/owners.json` (16 owners, no `documentation`), `verification/areas/` (19 manifests, no `documentation`), `verification/profiles/release.json:331-373` (`legacy_facade`; `selected_areas` has no `rust_interop`/`documentation`), `verification/runner/sifr_verify/profile_runner.py` (739 lines; steps at 160-187, no `documentation_checks`, no Rust step; report writer at 702-736), `verification/runner/sifr_verify/reports.py`, `verification/areas/developer_tooling/runner.py:111-152` (`FULL_SUITES` includes `editor-release`), `verification/areas/rust_interop/manifest.json` (4 suites; `data/stable_support_claims.json` absent), `verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json:27,31,40` (`rc` present), `preview-release.yml:12-13,59-60,269,309`, `editor_integrations/vscode/package.json:5` (`0.1.7`, `dist/` ignored), `scripts/run_all_tests.sh` (109 lines). Entry state matches what `40:52-61`, `40:325-357`, and `40:466-474` assume.

## Round-9 resolution audit

| R9 item | Status | Evidence |
|---|---|---|
| **M1.** "fixture-gated" rollback/roll-forward operations in the production workflow, unowned permissions, no falsifiable DoD | **Resolved** | The term is gone; `40:587-593` makes M40.3 pure mutation planners/validators plus a network-disabled harness accepting only a temporary filesystem index, release-asset directory, Marketplace stub, and non-deploying site fixture, with "no GitHub/Marketplace/site credentials, no `gh`/real `vsce`/repository dispatch adapter, and no production repository input"; `40:594-596` forbids adding the workflow inputs or write permissions; DoD `40:663-667` and negative `40:678-680` are falsifiable. |
| **M2.** Drill boundary covered only the index, not version assets/Marketplace/site | **Resolved** | `40:781-790` names the exact test adapters, read-only permissions, no production credentials, blocked network, no production target, and "no drill calls `gh release`, real `vsce publish`, or repository dispatch"; DoD `40:840-845` and negative `40:863-864` extend the prohibition to live tags/assets, the Marketplace listing, `sifr-lang/sifr-blog-website`, `sifr.sh`, and the live GA index. |
| **M3.** Rollback mutated the index with no approved artifact and no schema'd evidence | **Resolved** | `stable-incident-request.json` / `stable-incident-signoff.json` contracts at `40:179-186`; schemas/generators/validators owned by M40.0 (`40:328-329`); validator negatives at `40:361-375`; approval at `40:585-587` and `40:796-800`; write-once publication before mutation at `40:617-619`; retention at `40:614-616`; cross-validation at `40:581-583`, `40:646-648`; exit gate `40:959-961`. |
| **M4.** `initial`-mode Marketplace collision blocked a governed remedy | **Resolved** | `40:260-264` reuses an already-published version in *either* mode via the Gallery raw `Microsoft.VisualStudio.Services.VSIXPackage` asset on exact digest/metadata match; `40:805-809` and `40:858-860` restate it symmetrically. |
| Polish 1-3 (`rollback_target: none` scoping, affected-version equality, incident id on `active`) | **Resolved** | `40:678-679`; `40:368-369` + `40:799-800`; `40:371`. |
| Polish 4 (cross-doc drift) | **Unaddressed** (carried since R3) | Polish 1 below. |

All 34 material findings from rounds 1-9 are closed against live evidence. The two findings below are new to this round; both are evidence-lifecycle gaps in artifacts the phase *consumes* rather than in the governance artifacts it defines.

---

## Material findings

### 1. The `release`-profile report identifier/digest that the plan binds, the planner requires, and the protected workflow revalidates is an artifact that does not exist and is assigned to no milestone

The plan binds "local `release` profile report identifier and digest" (`40:210`). M40.1 must "Make the planner require a passing `scripts/run_all_tests.sh --profile release` report for the same source commit represented by its inputs" (`40:426-428`), with DoD "A passing plan references a passing release-profile report for the same source commit" (`40:437-438`). M40.4 must "retain its report identifier and digest" (`40:726-727`). M40.5 must "Revalidate the release-plan digest, source SHA, release-profile report, …" before mutation (`40:793`). The exit gate depends on it (`40:947`).

Repository evidence shows no such contract exists:

- `verification/runner/sifr_verify/profile_runner.py:702-736` writes exactly `target/validation_lane_reports/<profile>.latest.{log,time,json}` — **fixed names, overwritten by every subsequent run of the same profile**, under gitignored `target/`.
- The summarized JSON payload keys (verified against `target/validation_lane_reports/create-pr.latest.json`) are `advisories, artifact_cache, budget, cache_footprint, case_timings, cpu_seconds, description, e2e, hardening_summary, lane, lane_step_budgets, lane_steps, log_path, observations, policy, profile, requested_profile, suite_filters, time, time_file, workers` — **no source commit, no overall pass/fail status, no identifier, no digest**.
- `scripts/run_all_tests.sh` (109 lines) has no report/output flag; it only forwards to `sifr_verify profiles run`. Contrast `areas.py:105-126`, which does expose `--result-json`, so per-suite reports (`40:213-214,220`) are feasible while the profile report is not.

Consequences an implementer hits immediately:

- **Unowned implementation surface.** Nothing in M40.0-M40.5 adds report identity, source-commit binding, an overall pass verdict, digest emission, or retention outside `target/`. Every comparable prerequisite is explicitly owned (e.g. the `documentation_checks` step at `40:336-341`, the max-generation allocator at `40:487-489`), so this omission is a genuine gap against M40.0's own "no unowned entry" standard (`40:376`).
- **Ambiguous lifecycle / destructible evidence.** Because the file is `<profile>.latest.json`, any later local run destroys the exact report the approved plan digests — while `40:959-961` and `40:614-616` require every other piece of release evidence to be retained for the lifetime of the repository with no pruning.
- **`40:793` is unsatisfiable as written.** A protected workflow cannot revalidate a report that only ever exists under `target/` on the release engineer's workstation, so "Revalidate … release-profile report" has no achievable meaning and the M40.5 negative case "stale reports"/"changed plan" (`40:449`, `40:854`) has no reference object.

**Required:** assign to M40.0 or M40.1 a checked-in report contract — stable report identifier, source commit, resolved profile, overall status, canonical serialization, and digest — emitted by the profile runner outside `target/`'s overwrite path, plus an explicit statement of what "revalidate the release-profile report" means for the protected workflow (re-derive the digest from a transported/published report, or drop the item from `40:793` and rely solely on the plan binding plus the approver's attestation).

### 2. Custody and transport of the approved candidate plan and the approved incident request into the protected workflow are undefined, so the object under protected approval is unspecified

`40:195-197`: "The planner generates it in a release work directory at the already resolved release commit. It is never committed to the source tree; after approval, the exact file is published as a write-once version-release asset." M40.4 materializes it locally (`40:727-731`). M40.5 publishes it (`40:801-802`, with `40:246-247` failing if the release-plan asset already exists). So in `initial` mode the only copy of the approved plan is on a workstation, and the phase never states how the workflow obtains those bytes — dispatch input, pre-staged upload, artifact from a prior run, or a draft asset fetch. `grep -n "input"` over the file returns only site-dispatch inputs (`40:492-499`) and operation/channel inputs (`40:471`, `40:485`, `40:594`); nothing about plan or request ingestion.

The same hole is worse for the incident artifacts, which have no source commit to hang on: `40:617-619` says "Before an incident mutation, the canonical workflow publishes the **approved** request as a uniquely named write-once governance-release asset", while `40:585-587` and `40:796-798` locate approval *inside* the protected run ("protected approval applies to its digest"). The request must therefore already be present, digest-identified, and displayed to the approver before the environment gate — by an unstated mechanism.

Why this is material rather than mechanical:

- **The approval object is undefined.** `40:791` requires "a recorded approval distinct from the workflow initiator" and `40:831` "No stable mutation occurs without protected approval and a fully passing release plan", but a protected-environment reviewer approves a *run*. Whether they are attesting to a digest string typed into a dispatch field or to bytes the workflow fetched is exactly what determines whether `40:854`'s "changed plan" negative case is enforceable.
- **It is the one custody link the phase leaves open.** The referenced *predecessor* plans for a rollback are verifiable precisely because they are already published version-release assets (`40:585-587` + `40:196-197`), and every other integrity property is pinned (`40:288-292`). The new candidate plan and the new incident request are the only governance inputs with no stated provenance path, while `40:501-504` insists "no workstation credential is a release mechanism".
- **It interacts with M40.5 ordering.** `40:801-802` publishes assets first in `initial` mode; if the plan asset is the transport vehicle it must exist *before* that step, which `40:246-247` forbids. The doc gives no resolution.

**Required:** state, in M40.5's scope (and mirrored in `40:195-197` / `40:179-186`), exactly how the approved `stable-release-plan.json` and `stable-incident-request.json` bytes reach the protected workflow, what the approver sees and attests to, and the ordering relative to write-once publication — with a negative case for a run whose supplied artifact does not match the approved digest.

---

## Optional polish (non-blocking)

1. **Cross-doc drift, carried since round 3, both files modified in the working tree.** `plans/issues/active/rust-interop-verification-matrix-hardening.md:12-16` and `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:137-139,158-159` still name `milestone_40_1` / "the `milestone_40_4` activation gate" as the stable-candidate consumer, while `40:56-61` requires those artifacts *before* `milestone_40_0` and `40:345-346` registers the suite there. `plans/phases/index.md:50` still lists Phase 40 status `unspecified` against `40:3` `implementation-ready`.
2. **Drill environment is named only as "protected" (`40:781`).** On GitHub, environment secrets are exposed to any job referencing that environment, so "receives no production publication/site credentials" (`40:787`) is only satisfiable if the drill uses a *distinct* credential-free environment. Given how mechanism-specific the rest of the doc is (`sifr-release-index`, `--clobber`, the Gallery asset name), name the separate drill environment.
3. **Job topology wording.** `40:777-778` attaches "every stable-changing job" (plural) to the protected environment while `40:792` grants write permissions "only to the publication job" (singular), leaving the rollback/roll-forward job's permission scope inferred.
4. **`40:597` reads as requiring what `40:592-593` forbids.** "Rollback also invokes the same post-index site dispatch, deadline, generation/digest recheck, cancellation, and resume contract" sits in M40.3 scope, whose harness explicitly has no repository dispatch adapter. Add "…exercised through the non-deploying site fixture" to remove the apparent conflict.
5. **`40:485-486` vs `40:594-595`.** "rollback and stable inputs remain fail-closed" implies the inputs exist and are rejected; M40.3 says do not add them. Pick one phrasing.
6. **`40:331`** says to register "a `documentation` verification area and its owner in `verification/owners.json`". `owners.json` is an owner registry only (16 owner ids, no `area` key); areas are directories with a `manifest.json` whose `owner` must resolve there (`coverage_matrix.py`'s owner-registry check). Say "create `verification/areas/documentation/manifest.json` and register its owner in `verification/owners.json`".
7. **Cosmetic, carried since round 1.** `distribution_release --suite full` (`40:334`, `40:878`) implies a scope distinction that does not exist — `manifest.json` binds `representative` and `full` to the same case.

## What remains strong

- `40:587-596` + `40:663-667` + `40:781-790` + `40:840-845` + `40:863-864`: the no-write/no-network boundary is now stated twice with the same four temporary adapters, and both milestones carry falsifiable repository checks. The adapter seam is owned (M40.3 builds the mutation core against index/asset-directory/Marketplace/site fixtures; M40.5's drill invokes "the exact shared orchestration core"), so no second artifact path is introduced.
- `40:179-186` + `40:328-329` + `40:361-375` + `40:581-587` + `40:614-624` + `40:796-800` + `40:821-827`: incident request and incident sign-off have schema, generator, validator, negative cases, approval authority, pre-mutation write-once publication, atomic withdraw-plus-activate cross-validation, no-pruning retention, and exit-gate coverage. Rollback authorization no longer depends on a release plan.
- `40:260-264` + `40:805-809` + `40:858-860`: Marketplace reuse is symmetric across `initial`/`resume` with one exact-match rule and a fail-closed mismatch — resolving the collision that would otherwise have blocked a governed incident remedy.
- `40:67-80` / `40:930-931`: across 975 lines there is no compatibility shim, migration path, legacy reader, dual metadata, fallback URL/installer/path, or alternate architecture; the pre-GA no-users constraint is honored throughout.
