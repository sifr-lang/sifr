# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 7)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (875 lines) against all six artifacts in `plans/reviews/iterations/`, plus `plans/phases/33_*.md`, `plans/phases/index.md`, `internal_docs/distribution_pipeline.md`, `scripts/distribution/*.sh`, `.github/workflows/`, `verification/{owners.json,profiles/release.json,runner/sifr_verify/profile_runner.py}`, `verification/areas/{distribution_release,developer_tooling,rust_interop}/`, `editor_integrations/vscode/`, and both active Rust-interop issue plans.

## Round-6 resolution audit

| R6 item | Status | Evidence |
|---|---|---|
| **M1.** Pre-activation retry contradicts write-once (burned generation blocks GA) | **Resolved** | `40:236-251` introduces explicit `initial`/`resume` modes; `40:227` scopes the write-once rejection to *initial* publication; `40:239-247` defines exact-match reuse of version assets and Marketplace versions, missing-asset upload without overwrite, and skipped completed steps; `40:591-594` and `40:717-721` restate it per-milestone; `40:764-767` scopes acceptance of prior publication to protected `resume`. The `40:555-556` ↔ `40:223-224` contradiction is gone. |
| **M2.** Immutable plan pinned the whole-index previous generation | **Resolved** | `40:149-150` "Candidate stable plans do not pin this whole-index value"; the plan now binds only "the expected stable predecessor version/status (`none` for first GA)" (`40:194-196`); `40:262-267` moves generation/digest acquisition inside the workflow and states "Unrelated preview publication does not invalidate an approved stable plan; a changed stable predecessor does." `40:715` revalidates the *live stable predecessor*, not a generation. No `previous generation` residue in the plan-binding list (`40:194-212`). |
| R6 polish 2 — four retry windows | Resolved | `40:549-551` now enumerates all four, matching DoD `40:590-599`. |
| R6 polish 3 — rollback's own site deployment | Resolved | `40:545-547`. |
| R6 polish 4 — ack target vs. 20-min lease | Resolved | `40:535`. |
| R6 polish 5 — GitHub pending-run semantics | Resolved | `40:557-560`. |
| R6 polish 1, 6, 7 — slug attribution, cross-doc drift, `profile_runner.py` headroom | Unaddressed | See polish 3–5. |

Rounds 1–5 findings all remain closed. Re-verified against live evidence: `rc` sites (`self_update_install_receipt.schema.json:27,31,40`, `preview-release.yml:59-60`, `generate_dispatchers.sh:83-88`, `self_update_metadata.rs:40,68,195,243`) plus unenumerated ones (`build_preview_artifacts.sh:82-83`, `generate_version_installer.sh:70,227`, `docs/self_update.md:53`) are all captured by the catch-all at `40:440-441` and the falsifiable DoD at `40:501-504`; both `--clobber` sites (`preview-release.yml:269,309`) are in scope (`40:486,504`); `release-publication.yml` genuinely does not exist yet; `rust_interop/manifest.json` declares exactly the four structural suites with no `stable-candidate`, and `verification/areas/rust_interop/data/stable_support_claims.json` does not exist — exactly the entry state `40:52-61` assumes; `release.json` `legacy_facade.tooling_suites:["full"]` expands to `editor-release` via `developer_tooling/runner.py:142-151`, so `40:319-321` correctly confirms rather than duplicates; `editor_integrations/vscode/dist` is gitignored with no committed VSIX.

One new material contradiction, not raised in any prior round.

---

## Material findings

### 1. The first GA release is irreversible under the stated index invariants, yet the plan binds a rollback target and M40.5 requires a public rollback drill

Phase 40's objective is a "reversible release system" (`40:8`), and the exit gate requires "rollback, withdrawal, stale-generation rejection, and out-of-band recovery are tested" (`40:857`). But GA activation is by construction the *first* stable release — `40:710-711`: "Make GA activation the one-way `ga_status: preview` to `active` transition; the same mutation adds the first governed stable channel mapping."

Rolling that release back is unsatisfiable:

- `40:537-538`: rollback "marks the affected version `withdrawn`" and "points `stable` at the approved active rollback version";
- `40:137`: "Every present channel points to an `active` release of the matching version class" — the stable channel therefore needs an `active` `X.Y.Z`, and a beta/alpha cannot serve;
- `40:135`: "While `ga_status` is `active`, `stable` is required" — stable cannot be dropped;
- `40:133-134`: activation "never returns to `preview`".

At the first GA there is no prior active stable version (`40:194-196` correctly records the predecessor as `none`), so no legal post-rollback index state exists. Withdrawing the sole stable release violates `40:137`; leaving it violates `40:139` for consumers; removing `stable` violates `40:135`.

Two concrete downstream breakages:

- `40:212` binds "release notes and rollback target" into the candidate plan, and `40:642-644` requires the Marketplace compiler range to "contain both the candidate stable version and the release plan's rollback target", with `40:573-575` making a target outside the range ineligible. For the first GA the rollback target has no defined value; unlike the predecessor field, `40:212` grants no `none` form, so neither the plan schema (checked in at `40:306`) nor the extension-range validator (`40:573`) can be implemented as written.
- `40:751-752` (M40.5 DoD): "The **public** rollback drill proves the site names the rolled-back active version and incident while the Marketplace compiler range remains truthful." This DoD sits after real activation (`40:725-729`) and has no rolled-back active version to name. `40:616-617` says M40.5 "reruns the same drill against the protected workflow **before activation**", which is mock-based; `40:751-752` reads as post-activation and public. The two cannot both be met at the first GA.

**Required:** pick one and state it — (a) define the first GA's incident remedy as governed roll-*forward* (new qualified stable version becomes the channel target; withdrawal of the affected version is permitted only once a successor is active), and allow `rollback_target: none` in the plan schema with the extension-range check skipped in that case; or (b) require the first GA to be published as two ordered stable versions so a rollback target exists before the drill; or (c) explicitly scope `40:751-752` to a fixture/mock index and state that the first GA is roll-forward-only, adjusting `40:8` and `40:857` accordingly. As written, `40:8`/`40:857` and `40:135`/`40:137`/`40:538` cannot all hold.

---

## Optional polish (non-blocking)

1. **`40:762` is an unscoped restatement.** The negative-validation list makes "changed stable predecessor" prevent sign-off unconditionally. In post-activation `resume` the live stable predecessor has by definition changed to the just-published version; the resolving rules exist (`40:715` scopes revalidation to "before mutation"; `40:595-597` substitutes a generation/digest equality check and skips index mutation), but a literal reading of `40:762` reintroduces the round-6 deadlock class. Add "before index mutation" to that clause.
2. **Sign-off record shape is singular in two places, plural in one.** `40:246-247` requires the sign-off to record "every initial and resume workflow run", while `40:216-217` and `40:730-732` describe "the protected workflow run, approver" in the singular. Since `40:237` requires a *fresh* protected approval per resume, the schema (checked in at `40:307`) needs an attempt list carrying run + approver per attempt. M40.0's DoD (`40:338-344`) also enumerates validator rejections only for the index, plan, and site-facts schemas — no sign-off negative cases.
3. **Marketplace re-download identity is unspecified.** `40:244-245` reuses a published Marketplace version only when "its VSIX digest … match[es] the plan". Whether the hashed bytes are the gallery `VSIXPackage` asset or the client-facing signed download determines whether this check can ever pass; name the asset/API.
4. **Slug attribution (carried from R6).** `40:31-33` still attributes `sifr-lang/sifr-blog-website` to Phase 33's handoff. Verified: the slug appears nowhere outside Phase 40 — `plans/phases/33_preview_distribution_and_release_automation.md:28` records only a local path, `internal_docs/distribution_pipeline.md:13,28,38` uses `<site-repo>`, and `create_new_version.sh:26,40` takes `--site-repo` as a path. Only the *path* `apps/sifr-site/public/install/` is corroborated (`create_new_version.sh:109`, `33:29-31`, `distribution_pipeline.md:13,38`). Say Phase 40 establishes the slug, or add it to Phase 33.
5. **Cross-doc drift (carried from R3/R6).** `plans/issues/active/rust-interop-verification-matrix-hardening.md:12-16` and `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:137-139,157-159` still name `milestone_40_1` (and `milestone_40_4`) as the stable-candidate consumer and instruct edits to those milestones, while `40:56-61` requires the artifacts before `milestone_40_0` and `40:309-310` registers the validator in M40.0. `plans/phases/index.md:50` still lists Phase 40 as `unspecified` against `40:3` `implementation-ready`.
6. **`profile_runner.py` headroom.** 739 lines today; both `rust_interop_checks` (upstream `hardening_1`) and `documentation_checks` (`40:315-318`) land in it, against the 900-line cap. Naming the decomposition in M40.0 keeps it out of an unrelated PR.
7. **Snapshot publication is contracted globally (`40:151-159`) but its milestone landing reads as M40.3 (`40:563-566`), while alpha/beta index mutation begins in M40.2 (`40:452-455`).** One clause in M40.2 stating that generation snapshots are published from the first mutation removes the ambiguity (`40:155-157`'s max-over-index-and-snapshots allocation stays safe either way).

## What remains strong

- `40:236-251`: the `initial`/`resume` split is now the single clean resolution of write-once vs. retry — exact-digest reuse, no-clobber completion of missing assets, fresh protected approval, and an explicit "idempotent completion of one approved publication, not a second artifact path".
- `40:149-150` + `40:194-196` + `40:262-267`: candidate plans are generation-independent; the only cross-run coupling left is the semantically meaningful stable predecessor, resolved live inside the concurrency group.
- `40:471-480` + `40:557-560` + `40:595-599`: bounded lease, cancellation, downstream generation/digest recheck, terminal-failure semantics, and rollback supersession close the stale-deployment and starvation classes at three independent layers.
- `40:67-80` / `40:832-834`: the no-compatibility/no-fallback policy holds across all 875 lines — no shim, migration, dual metadata, legacy reader, fallback URL, or alternate installer anywhere.
- `40:269-273`: refusing to claim signing or notarization, and naming the actual integrity boundary instead.
