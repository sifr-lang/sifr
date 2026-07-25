# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 4)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (749 lines) against `verification/runner/sifr_verify/profile_runner.py`, `verification/profiles/release.json`, `verification/areas/developer_tooling/{manifest.json,runner.py}`, `verification/areas/coverage_matrix/checks/*`, `verification/schemas/area.schema.json`, `internal_docs/distribution_pipeline.md`, `.github/workflows/`, `docs/installation.mdx`, `editor_integrations/vscode/`, and both active rust-interop issues.

## Round-3 resolution audit

| R3 finding | Status | Evidence |
|---|---|---|
| 1a. `legacy_facade.tooling_suites` unnamed | **Addressed, but overshoots** | `40:270-271` now names both surfaces. However `editor-release` is already executed — see **Material 3**. |
| 1b. No `documentation_checks` step / no falsifiable DoD | **Resolved** | `40:266-269` scopes the executable step plus omission/no-result self-tests into 40_0; `40:296-298` replaces "schedules" with a printed `name=documentation_checks … status=pass` requirement. The format is real: `profile_runner.py:126` emits `[sifr-lane-step] name=… status=…`, and a vacuous pass is closed by the "selected-but-unrun … fail the runner self-test" clause. `40:272-274`'s "Confirm, rather than re-add" correctly demotes the `distribution_release` and structural rust-interop no-ops. |
| 1c. Owner registry | **Resolved** | `40:262-263` names `verification/owners.json`, satisfying `coverage_matrix.py:269-272`. Verified no repository check requires a new area's suites to be selected by any profile (`coverage_matrix.py:292-313`, `profile_assignment_matrix.py:145-155` validate matrix→profile only), so `documentation`/`structure` may land unassigned in 40_0. |
| 1d. `ga-release` vacuous at 40_0 | **Resolved** | 40_0 now registers `structure` (`40:263-265`); `ga-release` lands in 40_4 (`40:528-532`) and enters the per-PR contract only from 40_4 (`40:671-675`), with `structure` carrying 40_0–40_3 (`40:662`). |
| 2. `<N>` ambiguous / write-once retry deadlock | **Resolved** | `40:143-148` defines `<N>` as the proposed new generation, allocated as `max(current index, retained snapshots) + 1`, states publication reserves it, and states a burned generation is retained as attempt evidence with retry taking the next number and never overwriting. Restated in DoD `40:493-494` and exit gate `40:737-739`. Consistent with `40:116`/`40:145` (strictly increasing, non-contiguous) and with the immutable candidate plan, which binds only the *previous* generation (`40:178`), so burning `N` never invalidates an approved plan. |
| 2b. Post-replacement resume | **Resolved** | `40:495-496` + `40:738-739`: resume dispatcher/docs deployment, smoke, and sign-off without a second index mutation. |
| 3. Rollback docs/Marketplace unreconciled | **Partially resolved** | `40:475-480`, `40:497-499`, `40:742-744` add the reconciliation and correctly make Marketplace metadata a *range* (matching `40:539-542` and negative validation `40:585`). But the reconciliation now depends on artifacts that do not exist at 40_3 — see **Material 2**. |
| Marketplace step location | **Resolved in substance** | `40:600-602` + `40:613-614` leave only one legal reading (the `vsce publish` step lives in `release-publication.yml`). |
| Coordinated cross-repo PR pair | **Resolved** | `40:547-551` orders the upstream `editor_integrations` PR before the submodule pointer bump and requires the execution issue to record the exception. Confirmed the changes really are submodule-side: `editor_integrations/vscode/dist/` holds `0.0.0`–`0.1.3` while `package.json:5` is `0.1.7`. |

No new circularity, no fallback/migration/dual-metadata/legacy-reader language anywhere in 749 lines. `selftest.py:82`'s exact profile set and `profile_assignment_matrix.py:17`'s `PROFILE_NAMES` remain unviolated (no new profile).

---

## Material findings

### 1. Public-site deployment is required by the exit gate but owned by no milestone, and the plan never acknowledges it is a different repository

`40:83-84` requires `https://sifr.sh/install` to default to stable and `/install/stable` to exist; `40:615` activates stable by "publishing the next governed release-index generation **and deploying the generated stable-default dispatcher**"; `40:617-618` records "site deployment" as sign-off evidence; `40:630-632` makes public-surface agreement an exit-gate condition; `40:475-476` and `40:495-496` require docs/dispatcher deployment during rollback and resume.

Repository evidence contradicts the assumption that this is a main-repo action. `internal_docs/distribution_pipeline.md:13` — "Static site files live under **the site repository** at `<site-repo>/apps/sifr-site/public/install/`"; `:36-38` — the generator only writes into a caller-supplied `--install-root`; `:28` and `:314` describe rollout as a separate manual step ("before pushing/deploying the site repository"). The main repo has exactly two workflows (`local-first-validation.yml`, `preview-release.yml`) and zero deploy/pages automation; `grep -n "site\|deploy" preview-release.yml` and `generate_dispatchers.sh` return nothing.

So the one mutation that actually flips the public default from beta to stable is (a) in a repository Phase 40 never names, (b) outside the `sifr-release-index` concurrency group and the protected environment that `40:213-216` calls "exactly one enforcement path", and (c) unordered relative to index activation inside a single bullet (`40:615`) — if the dispatcher deploys before the generation is published, `/install` resolves a `stable` channel that does not yet exist.

**Required:** name the site repository and the deployment mechanism (cross-repo token/dispatch, or an explicit manual step recorded in the sign-off), assign its implementation to a milestone (naturally 40_2 alongside the dispatcher generator), and order it strictly after index activation.

### 2. `milestone_40_3`'s post-rollback reconciliation depends on `milestone_40_4`/`40_5` artifacts, so its DoD cannot be validated where it is asserted

`40:475-477` (40_3 scope) requires deploying "the active stable version and withdrawal notice to public docs from the new governed index" and verifying "that the **published** VS Code extension's compiler compatibility range contains the rollback target." `40:497-499` (40_3 DoD) asserts both hold after rollback.

Neither input exists at 40_3:

- The version-bearing GA docs surface is created in 40_4 (`40:526-527`), and its executable `release version` check lands in 40_4 (`40:528-532`), gated only from 40_4 (`40:671-675`). Today `docs/installation.mdx:204` states stable channels "are not yet available"; nothing renders an active stable version.
- The extension compatibility-range metadata is introduced in 40_4 (`40:539-542`) and only *published* to the Marketplace in 40_5 (`40:613-614`). At 40_3 there is no published extension range to verify against.
- 40_3's own demo (`40:511-514`) exercises only the mock index and two mock installations; it has no docs or Marketplace vehicle.

This also leaves the docs-version surface with two owners (40_3 deploys a version into it; 40_4 creates and checks it) — the same defect class as round-2 finding 4.

**Required:** either scope the reconciliation *mechanism* into 40_3 with fixture-backed validation and move the "public docs name the rolled-back active version / Marketplace metadata remains truthful" assertions to 40_4 (docs+range) and 40_5 (published state, where `40:513-514` already reruns the drill), or move the whole reconciliation item to 40_4.

### 3. Round-3's `tooling_suites` fix overshoots: `editor-release` already executes in the `release` lane, so `40:270-271` adds a duplicate npm-driven run

`40:270-271` requires adding `editor-release` to both the `developer_tooling` selection and `legacy_facade.tooling_suites`. But `release.json` `legacy_facade.tooling_suites` is `["full"]`, `profile_runner.py:438-445` forwards it to the area runner, and `developer_tooling/runner.py:142-151` defines `FULL_SUITES` as including `"editor-release"`, expanded at `:235-236`. `editor-release` therefore already runs today in `--profile release` (emitting `case=full/editor-release:*` timings), and its commands are the npm-driven `check_vscode_extension.py` plus two rules checks (`runner.py:111-117`).

Selecting both `full` and `editor-release` makes `select_suites` (`runner.py:220-230`) return both, so the entire `editor-release` command set — including `npm ci`/package — runs **twice** per release-lane invocation. The plan itself applies the correct treatment two lines later ("**Confirm, rather than re-add**, the existing `distribution_release` `full` execution", `40:272-274`) but not here.

**Required:** replace `40:270-271` with a confirmation that `full` already covers `editor-release`, and restate `40:296` as requiring the existing `full/editor-release` evidence in the release report. If a distinct `name=editor-release` result line is genuinely wanted, say so and remove the duplication (e.g. don't also add it to `tooling_suites`).

---

## Optional polish (non-blocking)

- **Cross-doc drift, still unfixed.** `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:158-159` still says "Update Phase 40 `milestone_40_1` … to execute the stable-candidate check," while registration is now 40_0 scope (`40:260-261`); `:137` says "Phase 40 `milestone_40_1` is downstream of both `hardening_1` and this item," while `40:53` requires both merged **before** `milestone_40_0`. The Phase 40 side is internally coherent; the issue file needs the edit.
- **One-PR rule wording.** `40:230-231` permits deviation only via "an approved smaller split"; the cross-repo pair at `40:547-551` is an *additional* upstream PR, not a split. One clause covering "cross-repository sequences" would close it.
- **Index drift persists.** `plans/phases/index.md:50` still lists Phase 40 status `unspecified` against `40:3` `implementation-ready`.
- **`distribution_release` suite pin.** `40:274` and `40:659` pin `full`; `representative` and `full` resolve to the same adapter case set. Harmless.
- **File-size headroom.** `profile_runner.py` is 739 lines and receives `rust_interop_checks` (hardening_1) plus `documentation_checks` (`40:266-269`). Still under the 900-line cap, but worth naming in 40_0 so the second step doesn't push a decomposition into an unrelated PR.

## What remains strong

- `40:64-77` / `40:711-712`: canonical-cutover policy stated once and never violated — no shim, dual metadata, legacy reader, fallback URL/installer/path across 749 lines.
- `40:143-148`: generation reservation, burned-generation retention, and next-number retry are now unambiguous and match the write-once/no-pruning rules at `40:202-211` and `40:468-474` without deadlock.
- `40:207-226`: naming GitHub asset storage's lack of compare-and-swap, deriving the single-enforcement-path design from it, and refusing to claim signing or notarization.
- `40:169-196`: candidate plan immutable in a work directory, sign-off a separate schema referencing the plan digest; no post-approval evidence rewrites an approved plan.
- `40:383-389`: `rc` removal remains falsifiable — `self_update_install_receipt.schema.json` and `preview-release.yml:59` still contain it and every surface is named; both live `--clobber` sites (`preview-release.yml:269,309`) are in scope at `40:405-409`.
