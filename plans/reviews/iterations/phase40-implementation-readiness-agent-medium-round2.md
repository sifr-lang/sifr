# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 2)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (650 lines) against the verification runner, profiles, area manifests, coverage-matrix governance checks, distribution scripts, workflows, and the two active rust-interop issues.

## Round-1 resolution audit

| R1 finding | Status | Evidence |
|---|---|---|
| 1. `rc` retained / no owner | **Resolved** | `40:90-92`, `40:141-143` ("`rc` is removed, not retained"), and `40:346-355` names the receipt schema, `APP_CHANNEL` derivation, dispatcher pin parsing, `preview-release.yml`, and self-update fixtures. All five surfaces still contain `rc` today (`preview-release.yml:59`), so the gate is now falsifiable. |
| 2. `rust_interop` in no profile | **Resolved as inheritance** | `40:49-57` now names `hardening_1`–`hardening_4` / `certification_0` as hard prerequisites; `rust-interop-verification-matrix-hardening.md:133-152` genuinely wires all four suites into create-pr/merge/nightly/release. Still zero hits for `rust_interop` in `verification/profiles/*` today, so this is inherited, not done — which the doc states honestly. |
| 3. Prose stable-claim gate | **Resolved in substance** | `stable_support_claims.json` now named at `40:53-56`, `176-177`, `308-310`, `467-470`, `613-614`, with concrete failure conditions. (But see Blocking 1 for the suite name.) |
| 4. VS Code repo identity | **Resolved** | `40:33-35`, `40:176-178` now treat `editor_integrations` as the submodule and `vscode/` as a package inside the already-recorded recursive SHA. Matches `.gitmodules` (one `editor_integrations` submodule). Duplicate provenance field removed. |
| 5. Marketplace ownership | **Resolved** | `40:485-487` and `40:530-532` put publication solely in the main-repo protected workflow, consuming the recorded VSIX without rebuilding. Stale VSIX cleanup added at `40:479-481` (still needed: `dist/` holds `0.0.0`–`0.1.3`, `package.json` version is `0.1.7`). |
| 6. `--clobber` | **Resolved** | `40:194-196`, `40:370-373`. Both live sites (`preview-release.yml:269`, `:309`) are now in scope. |
| 7. Unachievable CAS | **Resolved** | `40:205-208` states the honest single-enforcement-path model and demotes local `--real-run` to plan/dry-run. |
| 8. Plan location circularity | **Resolved** | `40:158-160`: work directory at the resolved commit, never committed, published as a write-once asset. |
| 9. Docs checks not invoked | **Resolved** | `documentation` area + `ga-release` suite registered at `40:249-252`; `phase40` profile in the Validation Contract at `40:574`. |
| 10. `create-pr` coverage | **Resolved** | The `phase40` profile now carries the cross-surface lanes per-PR. |

All ten round-1 findings are materially closed. The blockers below are new, and are mostly consequences of the mechanisms the rewrite introduced.

---

## Blocking / material findings

### 1. `rust_interop` `stable-candidate` is a named suite that no owner creates — and it hard-fails profile loading

`40:251-252` has `milestone_40_0` register a `phase40` profile that selects `rust_interop` `stable-candidate`; `40:309` and `40:590` treat it as a selectable suite (`areas run --area rust_interop --suite stable-candidate`). `40:56-57` explicitly disclaims ownership: "Phase 40 consumes those artifacts; it does not substitute prose checks for them."

But the upstream charter does not create a suite by that name. `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:145-150` commits only to "a Rust-interop area check" plus "a **stable-candidate mode**". `verification/areas/rust_interop/manifest.json` declares exactly four suites: `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`.

This is not cosmetic. `verification/runner/sifr_verify/profiles.py:151-158` raises `ProfileError: profile <name> selects unknown suite <area>:<suite>` at load time, and `verification/areas/coverage_matrix/checks/profile_assignment_matrix.py:139-144` independently rejects unknown suite tokens. So `milestone_40_0` cannot land a loadable `phase40` profile, and `40:590` is an uninvokable command, unless either (a) `certification_0` is amended to declare a manifest **suite** named `stable-candidate`, or (b) Phase 40 takes ownership of declaring it. Pick one explicitly; today the surface is named by Phase 40 and owned by nobody.

### 2. Prerequisite gate is placed one milestone too late

`40:53` says the rust-interop prerequisites must be merged "Before `milestone_40_1` begins." Three things in `milestone_40_0` already require them:

- `40:258-260` — 40_0 must "Confirm the Rust-interop hardening prerequisites have merged and validate the initial `stable_support_claims.json`."
- `40:249-252` — 40_0 registers the `phase40` profile selecting `rust_interop` `stable-candidate` (needs the suite and claims file to exist; see Blocking 1).
- `40:271` — 40_0's DoD asserts "The `release` profile visibly executes the Rust-interop step and all four required suites," which is `hardening_1`'s deliverable.
- `40:569-575` — every milestone PR, including 40_0's, must pass `--profile phase40`.

`40:53` should read "before `milestone_40_0` begins." As written the entry condition contradicts the first milestone's own scope, DoD, and PR gate.

### 3. The canonical publication workflow has no creating owner, and 40_5 reads as a second workflow

`40:205-208` insists on "exactly one enforcement path: every release-index mutation runs in the canonical main-repository publication workflow." `milestone_40_2` (`40:366-369`) requires that "all channel mutations route through the canonical main-repository publication workflow and shared concurrency group," with "the protected stable approval branch remains disabled until `milestone_40_5`" — i.e. the workflow exists in 40_2 with a disabled branch.

But no scope item in 40_2 says *create* it, and `milestone_40_5` (`40:521-522`) says "**Add** a dedicated stable publication workflow under a protected GitHub environment." That is either a second workflow (violating `40:206`) or a re-description of the one 40_2 was supposed to route into. Today `.github/workflows/preview-release.yml` has no `environment:` key and group `preview-release-channels` (`:27-29`), so *something* must create the canonical workflow — and no milestone claims it.

Resolve by naming the workflow file, assigning its creation to 40_2 (stable job disabled), and reframing 40_5 as enabling the protected stable job and environment on that same workflow. The shared concurrency group's canonical name should also be stated once, since `40:200` requires alpha/beta/stable/rollback to share it.

### 4. `stable-release-plan.json` VS Code and docs fields are bound in 40_1 but produced in 40_4

`40:176-179` lists in the plan the `vscode/` package path, package version, VSIX digest, compatibility range, validation report, and the `documentation` `ga-release` suite report. `milestone_40_1` owns the planner (`40:296-297`) and its DoD at `40:320` asserts "Changing any commit, submodule, lockfile, version, target, artifact, sysroot, installer, claim, **docs, or VS Code input** changes the plan digest."

`milestone_40_4` then owns producing the VSIX and "bind[ing] its package version and SHA-256 into `stable-release-plan.json`" (`40:483-484`), and owns adding the docs checks (`40:467-470`). So 40_1's DoD is unfalsifiable at 40_1 for two of its enumerated inputs, and one plan field has two owners three milestones apart. Either scope minimal producers into 40_1 (the packaging script already exists — `editor_integrations/vscode/package.json` has a `package` script driven by `verification/areas/developer_tooling/check_vscode_extension.py:137-150`), or restrict 40_1's DoD to the inputs 40_1 owns and move the docs/VSIX digest-sensitivity assertion into 40_4's DoD.

### 5. Introducing a fifth profile has unnamed, gate-breaking prerequisites

`milestone_40_0` (`40:249-252`) registers a `phase40` profile, and its DoD (`40:272`) asserts "`phase40` visibly schedules every Phase 40 area named above." Two repository surfaces enumerate the profile set by hand and will fail the same PR:

- `verification/runner/sifr_verify/selftest.py:82` — `expected = {"create-pr", "merge", "nightly", "python-interop-live", "release"}`, an exact-set assertion.
- `verification/areas/coverage_matrix/checks/profile_assignment_matrix.py:17` — `PROFILE_NAMES = ("create-pr", "merge", "nightly", "release")`.

Neither is named in scope. The second has a further consequence: because the assignment matrix only inspects those four profiles, `documentation:ga-release`, `developer_tooling:editor-release`, and `rust_interop:stable-candidate` are invisible to the repository's own profile-assignment governance no matter what `phase40` selects. Note `verification/profiles/python-interop-live.json` is the correct precedent to follow (`execution_mode: "selected-areas-only"`, `selected_areas[]`) — worth naming it so the implementer doesn't invent a new shape.

### 6. "Release retention" is claimed as owned scope but appears in no milestone

`40:17` lists "rollback, withdrawal, incident recovery, and **release retention**" among the surfaces Phase 40 owns. `milestone_40_3` (`40:413-427`) defines rollback triggers, withdrawal, downgrade consent, out-of-band recovery, serialization, and retry — but no retention policy. The closest text is `40:417` ("preserves immutable version assets and evidence"), which is immutability, not retention. There is no retention clause in any DoD or in the Exit Gate (`40:629-649`). Either define retention (how long withdrawn versions, plans, and sign-off records are kept and where) in `milestone_40_3`, or delete it from `40:17`.

---

## Optional polish (non-blocking)

- **Phase-scoped profile leaves GA suites ungated after the phase closes.** `documentation:ga-release`, `developer_tooling:editor-release`, and `rust_interop:stable-candidate` live only in `phase40`. `40:586-591` correctly requires both `release` and `phase40` for every release candidate, so GA itself is covered — but once Phase 40 closes, nothing durable executes the GA docs or editor-release suites. Folding these selections into `release.json` would make `40:174`'s single release-profile report digest sufficient and keep the surfaces gated permanently.
- **Redundant scope item.** `40:305-306` ("Run the Rust interop `matrix`, `tiers`, `compatibility-matrix`, and `stale-drafts` suites through the release profile") is exactly `hardening_1`'s deliverable (`rust-interop-verification-matrix-hardening.md:136-140`), already declared a prerequisite at `40:49-57`. Restate it as a verification, not scope.
- **Index drift persists.** `plans/phases/index.md:50` still lists Phase 40 status `unspecified`; `40:3` says `implementation-ready`.
- **`distribution_release` suite pin implies a distinction that doesn't exist.** `40:250` pins `full`; `representative` and `full` resolve to the same adapter case set. Harmless.
- **Plan binds no `phase40` report anchor.** `40:174` records the `release` profile report id/digest; the cross-surface `phase40` report is not bound, only its constituent suite reports (`40:177-179`). Binding the `phase40` report id/digest too would give one anchor for all cross-surface evidence.
- **Verified implementable, no action needed:** `40:421-423`'s `/install/stable --force` recovery works — the generated dispatcher forwards unrecognized flags to the installer (`scripts/distribution/generate_dispatchers.sh:150-155`, `:213-219`) and the immutable installer implements `--force` downgrade (`generate_version_installer.sh:145,168,634-636`). `40:128`/`40:359` correctly catch that the current dispatcher's version-pin branch skips metadata fetch entirely (`generate_dispatchers.sh:174-181`), which is exactly the defect that must close.

## What remains genuinely strong

- `40:63-77` and `40:616-619`: the canonical-cutover policy is stated once and never violated — no shim, fallback URL, dual metadata, legacy reader, or migration path anywhere in 650 lines.
- `40:181-186`: candidate plan immutable, sign-off a separate schema referencing the plan digest. No post-approval evidence rewrites an approved candidate.
- `40:205-214`: naming GitHub asset storage's lack of compare-and-swap, then deriving the single-enforcement-path design from it, and refusing to claim signing or notarization. Both are unusually honest.
- `40:130-134`: metadata carries no executable URLs; trusted code derives them from repository constants — the right hardening of the current `INSTALLER_RELEASE_BASE_URL` model.
- `40:42-45` correctly keeps Phase 38 a non-prerequisite while taking ownership of the GA-specific docs checks, and `40:46-48` matches the real four-value vocabulary in `rust_interop_compatibility_matrix.json`.
