# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 9)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (918 lines) against all eight artifacts in `plans/reviews/iterations/` and live repository evidence: `.github/workflows/` (only `local-first-validation.yml`, `preview-release.yml` — `release-publication.yml` absent), `verification/owners.json` (no `documentation` area), `verification/profiles/release.json:340` (`legacy_facade.tooling_suites`), `verification/runner/sifr_verify/profile_runner.py` (739 lines, no `documentation_checks` step), `verification/areas/rust_interop/manifest.json` (exactly `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`; no `data/stable_support_claims.json`), `verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json:31` (`rc` still in the enum), `verification/areas/distribution_release/manifest.json` (`representative` ≡ `full`), `editor_integrations/vscode/package.json:5`, `plans/phases/index.md:50`, and both active Rust-interop issue plans. Entry state is exactly what `40:52-61`, `40:317-335`, and `40:453-458` assume.

## Round-8 resolution audit

| R8 item | Status | Evidence |
|---|---|---|
| **M1.** Roll-forward remedy had no approved-artifact binding; successor `rollback_target` simultaneously mandated and forbidden | **Resolved** | Three named transition kinds at `40:212-218`; the `incident-roll-forward` plan now binds incident id, affected active version, withdrawal reason/evidence, successor record, and `rollback_target: none` (`40:216-218`, restated `40:561-565`); validator cases at `40:352-359`; range-check exemption extended past first GA (`40:604-605`); M40.4 range scoped to non-`none` (`40:677-679`, `40:725`); exit gate scoped to `normal` plans (`40:900-901`) and to non-`none` Marketplace coverage (`40:911-913`). |
| **M2.** No milestone enabled the roll-forward operation | **Partially resolved** | `40:741-745` enables `ga-activation`, `normal`, `rollback`, `incident-roll-forward` as named operations in the one workflow; `40:568-571` gives M40.3 the fixture-backed proof. **But the M40.3 half of that split is now itself unowned — Material 1.** |
| Rollback eligibility restoration | Resolved, non-circular | `40:560` permits rollback only from a `normal` plan naming an active predecessor; `40:565-567` restores eligibility via the next `normal` release naming the still-active successor; `40:789-792` states roll-forward-only until then. No cycle: `ga-activation`→`normal`(target=GA) and `incident-roll-forward`(none)→`normal`(target=successor) both terminate. |
| Polish 2 (sign-off singular) | Resolved | `40:766-767` "Append a sign-off attempt containing run, mode, approver, status, and mutations". |
| Polish 3 (exit-gate range phrasing) | Resolved | `40:911-913` "any non-`none` governed rollback target". |
| Polish 4 ("roll-forward command") | Resolved | `40:812` now `workflow_dispatch operation=incident-roll-forward`. |
| Polish 1 (cross-doc drift) | **Unaddressed** (carried since R3) | Polish 4 below. |

All 30 material findings from rounds 1–7 remain closed against live evidence.

---

## Material findings

### 1. M40.3 adds two production-capable mutation operations whose only safety boundary is the undefined term "fixture-gated", with no permission owner and no falsifiable DoD

`40:568-571`: "Add fixture-gated `rollback` and `incident-roll-forward` operations to `release-publication.yml`; both use the same schema validator, generation recheck, snapshot publication, and concurrency group as alpha and beta. Production enablement waits for `milestone_40_5`."

- **"fixture-gated" is defined nowhere.** It occurs exactly once in the repository (`grep`: only `40:568`). The bullet simultaneously says the operations live in the canonical production mutation workflow *and* use the same concurrency group, generation recheck, and snapshot publication as the live alpha/beta path — i.e. they are wired to the real governance release. What input, environment condition, or repository guard makes them incapable of reaching the live index is unspecified, so an implementer has no contract to build and a reviewer no criterion to check.
- **Protected-environment attachment and permission scoping arrive one milestone late.** `40:743-744` ("attach every stable-changing job to the protected GitHub environment") and `40:749` ("Grant write permissions only to the publication job") are M40.5 scope. Rollback and roll-forward are stable-changing by construction (`40:556-559`, `40:562-565`), so between M40.3 merge and M40.5 merge the canonical workflow carries index-mutating operations with no stated approval gate and no stated permission boundary — the exact condition `40:876-879` forbids.
- **No falsifiable DoD.** Every comparable milestone states a negative capability: `40:368` "No publication workflow can accept stable yet"; `40:516` "No public stable publication occurs"; `40:522` "no local command can publish". M40.3's DoD (`40:608-636`) and negative validation (`40:644-646`) contain no equivalent — nothing proves the newly added operations cannot mutate the live governed index, the live governance release, or public state. That is an unowned entry against M40.0's own standard (`40:360`).

**Required:** define the fixture-gating mechanism concretely (e.g. the operations accept only an explicit fixture index/governance-release target and fail closed on the production repository), assign the protected-environment attachment and write-permission scoping for these two operations to M40.3 rather than M40.5, and add an M40.3 DoD/negative case that fails if either operation can reach live release state before M40.5.

### 2. The M40.5 "protected non-production drill" boundary covers only the release index, not the three other production mutations the drilled operations perform

`40:788-792` requires a protected non-production rollback/site-reconciliation drill and a separate first-GA roll-forward drill, bounded by "neither drill mutates the live GA index". `40:652` calls this "the protected non-production path", also undefined.

But `incident-roll-forward` in production, per `40:562-565` + `40:750-765`, additionally: (a) publishes a **write-once** version release and assets, (b) runs `vsce publish` against the real Marketplace (`40:757-760`), and (c) dispatches the real `sifr-lang/sifr-blog-website` deployment that regenerates and deploys `https://sifr.sh/install` (`40:761-765`). None of these has a non-production form anywhere in the phase, and the drill constraint is silent on all three. A drill that exercises the operation end-to-end therefore burns a real version tag, publishes a real Marketplace version, and deploys the public site pre-GA; a drill that stops short of them proves nothing about the roll-forward path that `40:900` makes an exit-gate requirement. `40:809-812`'s demo lists only "`workflow_dispatch operation=incident-roll-forward`", leaving completion depth ambiguous.

**Required:** state exactly which mutations each drill performs and against what (fixture governance release, fixture index target, dry-run `vsce` / Marketplace publisher stub or explicit skip, dispatch to a non-deployed site ref), and add the corresponding M40.5 DoD/negative case that the drill cannot touch the live version tags, Marketplace listing, or `sifr.sh`.

### 3. The `rollback` operation mutates the governed index with no approved machine-readable artifact and no schema'd evidence record

Every other governance artifact has a checked-in schema and validator landed in M40.0 (`40:313-316`: release index, stable-release-plan, sign-off, site facts). The rollback operation has none:

- Rollback publishes no new version, so it has no `stable-release-plan.json`. Its authorization is derived prose — "Permit rollback only when a `normal` plan names an active stable predecessor" (`40:560`) plus "approval authority" defined in the execution issue (`40:552-553`) — but nothing states which artifact the workflow validates and digest-pins before mutating, in contrast to `40:750-752`'s revalidation list for stable publication. `40:645` requires "Unapproved rollback … rejected" with no definition of the approval token.
- Sign-off cannot hold the evidence: `stable-release-signoff.json` "references the candidate plan digest" (`40:222-226`) and is published "after their respective generation and post-publication smoke" (`40:595-597`) — a rollback has no candidate plan.
- `40:590` retains a "rollback record, and incident record" for the lifetime of the repository, and `40:619-620` requires incident evidence recording "trigger, approver, mutations, communication, validation, and closure". Neither artifact has a schema (absent from `40:313-316`), a publication location (absent from `40:592-597`'s governance-release list), a generator, or a validator case (absent from `40:347-359`). `40:902` then makes "incident evidence" an exit-gate retention subject.

**Required:** name the rollback/incident record as a checked-in schema + validator in M40.0, state what the rollback operation validates and pins before mutation (a rollback request record referencing the affected version's approved plan digest, incident id, and approver), state where the record is published write-once in the governance release, and add its negative validator cases.

### 4. `initial`-mode Marketplace publication has no defined behavior when the recorded extension version is already published

`40:757-760` publishes the recorded extension via `vsce publish` and handles reuse **only** in `resume` mode ("Resume verifies and reuses an exact matching Marketplace version"). But extension SemVer is independent of compiler SemVer (`40:675-676`) and the extension only needs a *range* covering the candidate (`40:677-679`), so a `normal` or `incident-roll-forward` release inside the already-advertised range legitimately records an unchanged package version (`editor_integrations/vscode/package.json:5` is `0.1.7` today). That is a fresh `initial` run against an already-published Marketplace version: `vsce publish` fails, and `40:715` makes "Marketplace failure leaves the stable channel unactivated" — so a governed incident remedy is blocked by a Marketplace collision unrelated to the incident. The doc never says whether each stable release must bump the extension, or whether an exact-matching already-published version is acceptable outside `resume`.

**Required:** either require a new extension version per stable release (and say so in `40:675-679` and the plan bindings) or extend the exact-match verify-and-reuse rule at `40:757-760` to `initial` mode, with the matching negative case (non-matching existing Marketplace version fails).

---

## Optional polish (non-blocking)

1. **`40:645` is unscoped against `40:564`.** M40.3 negative validation rejects "`rollback_target: none`", while `40:564` *mandates* `rollback_target: none` for `incident-roll-forward` and `40:355-356` rejects its absence. The surrounding list items are all rollback-operation attributes, so the intent is readable, but a validator author reading `40:645` in isolation gets the opposite rule. Scope it: "a `rollback` operation whose active stable release records `rollback_target: none`".
2. **Affected version vs. expected stable predecessor never equated.** The plan binds an "expected stable predecessor version/status" (`40:194-196`) and, for roll-forward, an "affected currently active version" (`40:562`). Prose implies they are the same record, but `40:271-275` validates only the predecessor and `40:352-359` has no case rejecting a roll-forward plan whose affected version is not the live active stable. Add the equality rule and its negative case.
3. **`40:126`'s conditional incident identifier has no negative case.** "an optional incident identifier only when status is `withdrawn`" — `40:347-359` never rejects an `active` release carrying an incident identifier.
4. **Cross-doc drift, carried since round 3, with both files modified in the working tree.** `plans/issues/active/rust-interop-verification-matrix-hardening.md:12` and `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:137,158-159` still name `milestone_40_1` / the "`milestone_40_4` activation gate" as the stable-candidate consumer, while `40:56-61` requires the artifacts *before* `milestone_40_0` and `40:317-318` registers the suite there. `plans/phases/index.md:50` still lists Phase 40 as `unspecified` against `40:3` `implementation-ready`.
5. **Cosmetic:** `--suite full` for `distribution_release` (`40:334`, `40:821`) implies a scope distinction that does not exist — `manifest.json` gives `representative` and `full` identical case sets.

## What remains strong

- `40:212-218` + `40:352-359` + `40:560-567` + `40:604-605` + `40:677-679` + `40:900-901`: the three transition kinds, the `none`/non-`none` split, and rollback-eligibility restoration are now threaded consistently through plan binding, index validator, rollback permission, range validator, and exit gate. Round 8's Material 1 is fully closed and the state machine is non-circular and terminating.
- `40:741-745`: one workflow, six named operations, "Do not add a second mutation workflow" — the production enablement point is unambiguous.
- `40:471-474` + `40:151-159` + `40:623-627`: max-generation allocation, write-once snapshot history, and burned-generation retry exercised from the first alpha/beta mutation.
- `40:489-499` + `40:572-573` + `40:628-632`: bounded lease, terminal timeout, downstream generation/digest recheck, and rollback supersession close the stale-deployment class at three layers.
- `40:67-80` / `40:874`: no compatibility shim, migration, dual metadata, legacy reader, fallback URL/installer, or alternate path anywhere in 918 lines.
