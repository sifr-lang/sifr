# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 3)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (699 lines) against `verification/runner/sifr_verify/{profile_runner,profiles,selftest}.py`, `verification/profiles/*.json`, all 21 area manifests, `verification/areas/coverage_matrix/checks/*`, `.github/workflows/`, `scripts/distribution/*`, `editor_integrations/`, and both active rust-interop issues.

## Round-1 / Round-2 resolution audit

All ten round-1 findings remain closed. Round-2 blockers:

| R2 finding | Status | Evidence |
|---|---|---|
| 1. `stable-candidate` suite owned by nobody | **Resolved** | `40:56-58` ("Phase 40 … owns registering the validator as the `rust_interop` `stable-candidate` suite") + `40:250-251`. Confirmed non-overlapping with upstream: `rust-interop-runtime-ecosystem-certification.md:147-157` creates only the check and a *mode*, never a manifest suite. `rust_interop/manifest.json` still declares exactly four suites, so `profiles.py:135-157` / `profile_assignment_matrix.py:139-144` will now be satisfied by 40_0's own PR. |
| 2. Prerequisite one milestone too late | **Resolved** | `40:53` now reads "Before `milestone_40_0` begins". Consistent with `40:266-268`, `40:279-281`, and the per-PR gate at `40:617-623`. |
| 3. Publication workflow had no creating owner | **Resolved** | `40:380-383` creates `.github/workflows/release-publication.yml` in 40_2 with alpha/beta only; `40:437-439` enables rollback in 40_3; `40:560-562` enables stable on *the existing* workflow and states "Do not add a second mutation workflow." No second workflow. Confirmed the file does not exist today (`.github/workflows/` = `local-first-validation.yml`, `preview-release.yml`). |
| 4. `sifr-release-index` group unnamed | **Resolved** | Named once and used consistently at `40:204-205`, `40:209-211`, `40:382`, `40:438-439`, `40:444-445`. Current group is `preview-release-channels` (`preview-release.yml:27-29`), and `40:385-387` scopes the refactor. |
| 5. Plan bound in 40_1 / produced in 40_4 | **Resolved** | `40:297-298` defers materialization to 40_4; `40:331-333` narrows 40_1's digest-sensitivity DoD to "input owned by this milestone"; `40:517-520` and `40:532-534` put docs/VSIX digest sensitivity in 40_4. No remaining two-owner field. |
| 6. Fifth profile | **Resolved as intent, broken as mechanism** | `40:259-260` now uses the durable `release` profile ("Do not create a Phase-40-only profile"), so `selftest.py:82`'s exact-set assertion and `profile_assignment_matrix.py:17`'s `PROFILE_NAMES` are no longer violated. But the wiring is inert — see **Blocking 1**. |
| 7. Retention unowned | **Resolved** | `40:449-456` in 40_3 scope, DoD `40:467-468`, exit gate `40:689-690`. Explicit "no automated pruning." |

No new circularity: `40:158-160` + `40:297-298` + `40:515-520` keep the plan work-dir-only, materialized after its inputs exist, with the release-profile report bound rather than committed. The 40_4 sequence (docs → VSIX → release-profile run at the final commit → plan) is acyclic.

---

## Blocking / material findings

### 1. "Add to the `release` profile" is inert for two of the named suites; the DoD says "schedules", not "executes"

`40:259-260` requires adding `distribution_release full`, `developer_tooling editor-release`, the four structural `rust_interop` suites, `rust_interop stable-candidate`, and `documentation ga-release` to the `release` profile. The DoD (`40:279-281`) demands the profile "visibly **executes** the Rust-interop step and all four structural suites plus `stable-candidate`, and visibly **schedules** every other Phase 40 area named above."

In this runner, `selected_areas` membership does **not** cause execution. `release.json` uses the legacy-facade path (`profile_runner.py:155-196`), whose step list is fixed and hand-written:

- `developer_tooling` execution reads `legacy_facade.tooling_suites` (`profile_runner.py:216-217`, `:438-445`), which is `["full"]` in `release.json`. Adding `editor-release` to `selected_areas` alone runs nothing; `legacy_facade.tooling_suites` must also change. Not stated.
- `documentation` has **no step at all** in `profile_runner.py:160-187`. A selected-but-unstepped area is silently skipped. Nothing in 40_0's scope adds a `documentation_checks` step or a self-test asserting it ran. Not stated.
- `distribution_release full` is already selected and already driven by `legacy_facade.distribution: "full"` (`profile_runner.py:494-501`); this scope item is a no-op.
- The four `rust_interop` suites plus the executable step are `hardening_1`'s deliverable (`rust-interop-verification-matrix-hardening.md:135-158`), already a declared prerequisite at `40:49-58` — so `40:259` "Add" contradicts `40:314-316`'s "Verify that the inherited … suites run through the release profile."

This is exactly the defect class `hardening_1` exists to fix ("adds profile-runner self-tests that fail if a normal legacy-facade profile selects `rust_interop` but omits the executable step"). Worse, `profile_assignment_matrix.py:145-155` validates only `selected_areas` tokens, so repository governance would report `documentation:ga-release` as assigned while it never runs. The word "schedules" in `40:280-281` makes the DoD pass under precisely this failure.

**Required:** name `legacy_facade.tooling_suites` and a new `documentation_checks` step in `profile_runner.py` as 40_0 scope, and change `40:280-281` to require a printed `name=documentation_checks … status=pass` / `editor-release` suite line in the release-profile report — the same falsifiable form `hardening_1` uses.

### 2. Generation-snapshot ordering creates a write-once retry deadlock, and `<N>` is ambiguous

Three clauses interact:

- `40:138-139`: the mutation workflow "reacquires and verifies both after entering its … concurrency group and **immediately before replacement**."
- `40:452-456`: "The canonical workflow publishes each `channels-generation-<N>.json` snapshot as a uniquely named write-once asset in the governance release **before replacing** the current `channels.json`."
- `40:580-582`: "Immediately before activation, publish the immutable `channels-generation-<N>.json` snapshot **without `--clobber`**."

With `<N>` read as the *new* generation (the reading `40:568-571`'s "publishing the next governed release-index generation" forces), any failure after the snapshot upload but before `channels.json` replacement leaves an orphaned write-once snapshot for a generation that was never active. `channels.json` still holds `N-1`, so the retry derives the same `N`, and the no-`--clobber` write-once rule (`40:196-199`, `40:581`) plus the no-pruning rule (`40:451-452`) make that upload permanently fail. The channel is bricked with no legal escape. `40:445-447`'s retry-behavior clause names the three failure windows but resolves none of them for the snapshot asset.

With `<N>` read as the *outgoing* generation, the ordering is safe but the newest generation is never snapshotted until the next mutation — which breaks `40:689-690`'s "every release-index generation snapshot" retention claim and `40:466-468`'s check.

The schema only requires generation be "monotonically increasing" (`40:116`), so gaps are already legal; the fix is one sentence.

**Required:** state which generation `<N>` names, and either publish the snapshot after a successful `channels.json` replacement, or state explicitly that a failed attempt burns its generation number and the retry uses the next one.

### 3. Rollback leaves public docs and Marketplace metadata claiming a withdrawn release, with no owner

`40:16-19` puts both "rollback, withdrawal, incident recovery" and "Marketplace publication governance" in Phase 40's ownership, and the exit gate at `40:692-693` requires "public docs and VS Code Marketplace metadata match the release plan."

40_4 requires docs to state the release version (`40:497`) and the extension release notes / Marketplace metadata to "name the supported stable compiler range" (`40:505-507`). 40_5 publishes the extension (`40:566-567`) *before* channel activation (`40:568-571`), which is correct for `40:535`. But `milestone_40_3` (`40:428-456`) reconciles only the release index: withdraw, repoint, incident id, snapshots, retention. Nothing updates the docs release version or the Marketplace-visible compiler range after a withdrawal, and `40:465-466`'s "communication" is incident comms, not published metadata. So after any rollback the exit-gate invariant at `40:692-693` is false with no milestone owning the correction.

**Required:** add a post-rollback surface-reconciliation item to `milestone_40_3` (docs release version and extension compatibility-range metadata follow the active stable release), or scope `40:692-693` to the activation instant and say explicitly that withdrawal does not require Marketplace metadata changes.

---

## Optional polish (non-blocking)

- **New area needs a registry entry.** `coverage_matrix.py:269-272` (`validate_owner_registry_covers_area_manifests`) requires every `verification/areas/*/manifest.json` `owner` to exist in `verification/owners.json`. 40_0's "Register a `documentation` verification area" (`40:251-252`) should name that file.
- **`ga-release` suite content at 40_0 is unspecified.** `40:617-623` makes `--area documentation --suite ga-release` a gate on *every* milestone PR, but the actual checks land in 40_4 (`40:495-498`). The suite would pass vacuously through 40_1–40_3. `area.schema.json` requires a `cases` array; state the minimum check 40_0 registers so the lane is not green-by-emptiness.
- **Cross-doc staleness.** `rust-interop-runtime-ecosystem-certification.md:158-159` still instructs "Update Phase 40 `milestone_40_1` … and `milestone_40_4` activation gate to execute the stable-candidate check" — stale now that registration is 40_0 scope. The Phase 40 side is coherent; the issue file needs the edit.
- **Marketplace step location unstated.** `40:566-567` says "the main-repository protected workflow" and `40:560-562` says "Do not add a second mutation workflow," which together imply the `vsce publish` step lives inside `release-publication.yml`. Saying so removes the last ambiguity.
- **40_4 requires a submodule PR.** Removing `editor_integrations/vscode/dist/*.vsix` (`40:510-511`; `0.0.0`–`0.1.3` are committed while `package.json` version is `0.1.7`) and adding the compiler-range metadata (`40:505-507`) are changes inside the `editor_integrations` submodule, which must land before the SHA the plan records (`40:512-513`). `40:225-226`'s one-PR-per-milestone rule should acknowledge the cross-repo pair.
- **`distribution_release` suite pin still implies a distinction that doesn't exist.** `40:259` and `40:619` pin `full`; `representative` and `full` resolve to the same adapter case set. Harmless.
- **Index drift persists.** `plans/phases/index.md:50` still lists Phase 40 status `unspecified`; `40:3` says `implementation-ready`.

## What remains genuinely strong

- `40:63-77` / `40:663-666`: canonical-cutover policy stated once and never violated. Zero shims, fallback URLs, dual metadata, legacy readers, or migration paths across 699 lines.
- `40:181-190`: candidate plan immutable; sign-off is a separate schema referencing the plan digest. No post-approval evidence rewrites an approved candidate.
- `40:207-214` and `40:216-220`: naming GitHub asset storage's lack of compare-and-swap, deriving the single-enforcement-path design from it, and refusing to claim signing or notarization.
- `40:130-135`: metadata carries no executable URLs; trusted code derives them from repository constants.
- `40:361-368`: `rc` removal is now falsifiable — all four surfaces still contain it (`self_update_install_receipt.schema.json:27,31,40`; `preview-release.yml:59`) and every one is named.
- `40:194-199` + `40:385-391`: both live `--clobber` sites (`preview-release.yml:269`, `:309`) and the local `--real-run` path (`create_new_version.sh:62-64,321`) are in scope, with `--real-run` correctly demoted to plan-only.
