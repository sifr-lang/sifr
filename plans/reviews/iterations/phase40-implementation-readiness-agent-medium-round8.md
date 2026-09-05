# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 8)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (903 lines) against all seven artifacts in `plans/reviews/iterations/` and live repository evidence: `.github/workflows/`, `verification/{owners.json,profiles/release.json,runner/sifr_verify/profile_runner.py}`, `verification/areas/{distribution_release/schemas,rust_interop,developer_tooling}/`, `scripts/distribution/*.sh`, `crates/sifr/src/self_update_metadata.rs`, `editor_integrations/vscode/`, `plans/phases/index.md`, and both active Rust-interop issue plans.

## Round-7 resolution audit

| R7 item | Status | Evidence |
|---|---|---|
| **M1.** First GA irreversible, yet plan binds a rollback target and M40.5 requires a *public* rollback drill | **Partially resolved** | Objective reworded to "incident-recoverable" (`40:8`); `40:212-213` grants `rollback_target: none` for first GA; `40:348` validator rejects a non-`none` first-GA target; `40:552-559` makes first-GA incidents roll-forward; `40:592-593` skips the range check for `none`; `40:666-668`/`40:713` scope the range to *non-`none`* targets; the post-activation "public rollback drill" is gone, replaced by `40:776-777` protected non-production drills that explicitly "never mutate the live GA index"; exit gate adds `40:884-885`. **But the roll-forward path it introduces is itself unspecified — see Material 1 and 2.** |
| Polish 1 — "changed stable predecessor" unscoped | Resolved | `40:787` now reads "changed stable predecessor **before index mutation**". |
| Polish 2 — sign-off singular vs. plural | Resolved (contract), residue in one restatement | `40:216-219` `attempts` list with run/mode/approver/status/mutations; `40:350-351` adds sign-off negative cases to M40.0 DoD. `40:753` still singular — polish 2 below. |
| Polish 3 — Marketplace re-download identity | Resolved | `40:246-248` names the Gallery `Microsoft.VisualStudio.Services.VSIXPackage` raw asset. |
| Polish 4 — slug attribution | Resolved | `40:32-33` "Phase 40 establishes the verified current remote identity". Confirmed the slug exists nowhere else in the repo. |
| Polish 6 — `profile_runner.py` headroom | Resolved | `40:320-322` names the responsibility split before adding the Rust/docs steps (file is 739 lines today; no `documentation_checks`/`rust_interop_checks` step yet). |
| Polish 7 — snapshot publication milestone | Resolved | `40:463-466` "From its first alpha/beta mutation … snapshot history is not deferred to the rollback milestone." |
| Polish 5 — cross-doc drift | **Unaddressed** (carried since R3) | See polish 1 below. |

Rounds 1–6 material findings all remain closed, re-verified against live evidence: every `rc` site (`self_update_install_receipt.schema.json:29-31`, `preview-release.yml:59-60`, `generate_dispatchers.sh:83-88`, `self_update_metadata.rs:40,68,195,243`, `build_preview_artifacts.sh:82-83`, `generate_version_installer.sh:70-71,227`, `docs/self_update.md:53`) is covered by `40:443-450` + falsifiable DoD `40:513-515`; both `--clobber` sites (`preview-release.yml:269,309`) are in scope (`40:496-499,515`); `release-publication.yml` does not exist (`.github/workflows/` holds only `local-first-validation.yml`, `preview-release.yml`); `rust_interop/manifest.json` declares exactly the four structural suites and `data/stable_support_claims.json` is absent — the entry state `40:52-61` assumes; `release.json:340-342` `legacy_facade.tooling_suites:["full"]` expands to `editor-release`, so `40:324-326` correctly *confirms* rather than duplicates; no `documentation` area or owner exists yet, matching M40.0 scope; `editor_integrations/vscode/dist` is ignored with no committed VSIX.

---

## Material findings

### 1. The roll-forward remedy has no approved-artifact binding, and the successor plan's `rollback_target` is a value the governance layer simultaneously mandates and forbids

`40:555-558` defines the first-GA remedy: "The workflow qualifies and publishes a new stable version, then atomically activates it and withdraws the affected first version in one new index generation." That single mutation touches **two** release records and (per `40:126`) may carry an incident identifier. Nothing in the phase binds either.

- The candidate plan binds "the expected stable predecessor version/status … and **the desired stable release record**" (`40:194-196`) — one record. There is no field for the affected version to withdraw and no incident identifier anywhere in the plan-binding list (`40:190-213`). `40:738` requires revalidating "the release-plan digest, source SHA, … live stable predecessor" before mutation, so the workflow would withdraw a version that the approved, digest-pinned artifact never names. That is an unapproved index mutation under the phase's own governance rule (`40:860-863`: "Only the canonical … workflow can initiate release state mutation" against an approved plan).
- The M40.0 validator DoD (`40:346-351`) enumerates rejections for first-GA/later-plan predecessor targets, withdrawn channel targets, and site-facts disagreement — but no case for a roll-forward request, so the schema author has no falsifiable target.
- The successor plan is not first GA, so `40:212-213` forces `rollback_target` = "the exact active stable predecessor" = the affected v1. That same mutation withdraws v1 (`40:556-557`). Consequences, none of which the doc resolves:
  - `40:590-592` "the extension-range validator requires the rollback target to remain covered … a rollback target outside that range is ineligible", and the exemption at `40:592-593` fires "only when first GA records `rollback_target: none`" — so the Marketplace range must advertise a **withdrawn** compiler version, while `40:668` forbids `none` here.
  - `40:552` permits rollback "only when the plan names an active stable predecessor", so v2's recorded target is non-active from the moment it is recorded: the second stable release is *also* roll-forward-only. `40:776-777` asserts the playbook is roll-forward-only "until a stable predecessor exists", which reads as satisfied at v2 but is not.
  - `40:884-885` (exit gate) requires "later stable plans cannot omit their active-predecessor rollback target" — unsatisfiable for the roll-forward successor if the intent is `none`, and misleading if the intent is a withdrawn v1.

**Required:** state explicitly (a) how the affected-version withdrawal and incident identifier enter the approved artifact set for a roll-forward publication (new plan field, or a separate approved incident record referenced by the sign-off), (b) whether the roll-forward successor's `rollback_target` is `none` (and if so, extend the `40:348`/`40:592-593` exemptions and the `40:884-885` exit-gate clause beyond "first GA") or the withdrawn predecessor (and if so, state that the range check intentionally covers a withdrawn version and that rollback stays fail-closed for that release), and (c) the corresponding negative validator cases in M40.0's DoD.

### 2. No milestone enables the roll-forward operation in `release-publication.yml`

`40:461-462` (M40.2): the workflow "initially accepts only alpha and beta operations; rollback and stable inputs remain fail-closed until their owning milestones." `40:560-562` (M40.3) enables "the rollback operation". `40:729-730` (M40.5) enables "stable input". Roll-forward is neither: it publishes and activates a new **stable** version *and* withdraws another in one generation. Its scope bullet sits in M40.3 (`40:552-559`) with a fixture-backed DoD (`40:601-603`), yet stable publication is fail-closed until M40.5, and M40.5's scope never mentions the composite operation — only `40:776-777` reruns the drill. So the workflow input/mode, its permission and protected-environment attachment, and its enabling milestone are unowned, violating M40.0's own "no unowned entry" standard (`40:353`).

**Required:** name the roll-forward operation as an explicit `release-publication.yml` mode, assign its enablement to a milestone (M40.5 alongside stable input, with M40.3 owning only the fixture-backed proof), and state that it runs under the protected environment and the `sifr-release-index` group with the same generation recheck and snapshot publication.

---

## Optional polish (non-blocking)

1. **Cross-doc drift — carried since round 3, and the touched files were not retargeted.** `plans/issues/active/rust-interop-verification-matrix-hardening.md:12` and `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:137,158-160` still name `milestone_40_1` (and the "`milestone_40_4` activation gate") as the stable-candidate consumer and instruct edits to those milestones, while `40:56-61` requires the artifacts *before* `milestone_40_0` and `40:309-310` registers the validator there. Both issue files are modified in the working tree, so this was an available fix. `plans/phases/index.md:50` still lists Phase 40 as `unspecified` against `40:3` `implementation-ready`.
2. **Sign-off restatement still singular.** `40:753-757` "Record workflow run, approver, …" versus the canonical `attempts` list at `40:216-219` and the per-resume fresh approval at `40:239-240`. Say "record a new attempt (run, mode, approver, status, mutations) plus …".
3. **Exit-gate rollback-target phrasing is unscoped.** `40:895-896` requires Marketplace metadata to cover "both the active version and its governed rollback target"; at Phase 40 close the target is `none` (`40:212-213`). Add "any non-`none`", matching `40:666-668`/`40:713`.
4. **`40:796` names "the first-GA roll-forward command"** but no command is defined; the only named out-of-band entrypoint is `/install/stable --force` (`40:564-566`). Either name the workflow dispatch or drop "command".

## What remains strong

- `40:212-213` + `40:348` + `40:592-593` + `40:666-668` + `40:713`: the `none`/non-`none` distinction is now threaded consistently through plan binding, index validator, range validator, and M40.4 negative validation — the round-7 unimplementable-field problem is gone.
- `40:776-777`: both drills are explicitly fixture/protected and "neither drill mutates the live GA index", removing the post-activation public-drill contradiction.
- `40:236-254` + `40:246-248`: `initial`/`resume` with exact-digest reuse, no clobber, fresh protected approval, and a concretely named Gallery asset for VSIX identity.
- `40:463-466` + `40:151-159`: snapshot allocation and write-once history exercised from the first alpha/beta mutation, so the max-generation allocator and burned-generation semantics are proven long before GA.
- `40:471-491` + `40:567-570` + `40:611-620`: bounded lease, terminal timeout, downstream generation/digest recheck, and rollback supersession close the stale-deployment class at three layers.
- `40:67-80` / `40:857-858`: the no-compatibility policy holds across all 903 lines — no shim, migration, dual metadata, legacy reader, fallback URL/installer, or alternate path anywhere.
