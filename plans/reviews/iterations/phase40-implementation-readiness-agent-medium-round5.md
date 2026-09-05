# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 5)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (797 lines) against rounds 1–4, `plans/phases/33_preview_distribution_and_release_automation.md`, `internal_docs/distribution_pipeline.md`, `scripts/distribution/*.sh`, `verification/profiles/release.json`, `verification/areas/developer_tooling/runner.py`, `verification/runner/sifr_verify/profile_runner.py`, `verification/owners.json`, and `.github/workflows/`.

## Round-4 resolution audit

| R4 finding | Status | Evidence |
|---|---|---|
| **M1.** Site deployment unowned / repo unnamed / unordered | **Partially resolved** | `40:31-33` names a repo and path; `40:420-433` scopes a paired workflow to 40_2 with pinned inputs, correlated run verification, credential scoping (`40:427-429`), cross-repo PR ordering (`40:425-426`), strict post-index ordering plus lease retention (`40:430-433`), and 40_5 activation ordering (`40:654-658`) with sign-off evidence (`40:659-663`). But the repository slug is **wrong** (Material 1) and the lease-hold has no bounded-wait semantics (Material 4). |
| **M2.** 40_3 reconciliation depends on 40_4/40_5 artifacts | **Resolved** | `40:510-517` makes 40_3 fixture-backed and explicitly defers public assertions ("Public assertions wait for the real docs/range in `milestone_40_4` and publication in `milestone_40_5`"); DoD `40:536-537` is fixture-scoped; the real-docs assertion sits in 40_4 DoD `40:611-613`; the published drill sits in 40_5 `40:680-681`. No two-owner docs surface remains. |
| **M3.** Duplicate `editor-release` execution | **Resolved** | `40:279-281` now confirms rather than adds; DoD `40:310` requires `full/editor-release:*` evidence **exactly once**. Verified against `release.json` `legacy_facade.tooling_suites: ["full"]` and `developer_tooling/runner.py:142-151` (`FULL_SUITES` contains `editor-release`). Standalone `--suite editor-release` at `40:708` is a separate invocation, not a second run in one lane. |
| R4 polish: one-PR rule vs cross-repo pairs | **Resolved** | `40:240-242` now permits "an ordered cross-repository PR sequence". |
| R4 polish: `profile_runner.py` headroom | Unaddressed | Still 739 lines; still non-blocking. |

Rounds 1–3 findings all remain closed. `verification/owners.json` exists (`40:262-263`); no fifth profile; no fallback/migration/dual-metadata/legacy-reader language anywhere in 797 lines.

---

## Material findings

### 1. `sifr-lang/sifr-website` does not exist; the real site repository is `sifr-lang/sifr-blog-website`, and Phase 33 is mis-cited

`40:31-33`, `40:167`, `40:196`, `40:421`, `40:425`, `40:429` all pin the cross-repository workflow, dispatch target, credential scope, and PR sequence to `sifr-lang/sifr-website`.

Repository evidence:
- `plans/phases/33_preview_distribution_and_release_automation.md:28` records the site repo as `/Users/yaseralnajjar/work/sifr/sifr-blog-website/` — a local path only. Phase 33 never names a GitHub slug, so `40:31-33`'s "Phase 33 preview distribution, including the separate `sifr-lang/sifr-website` repository" attributes a fact Phase 33 does not contain.
- The actual remote is `https://github.com/sifr-lang/sifr-blog-website.git`.
- `scripts/distribution/create_new_version.sh:109` and `internal_docs/distribution_pipeline.md:13,38` confirm only the **path** `apps/sifr-site/public/install` — that half of the round-4 fix is correct.

A named-but-nonexistent repository is worse than an unnamed one: the fine-grained token scope (`40:427-429`), the pinned dispatch (`40:421-424`), and the ordered PR pair (`40:425-426`) all target it.

**Required:** replace every occurrence with `sifr-lang/sifr-blog-website`, and drop the claim that Phase 33 established the slug (or add it to Phase 33 first).

### 2. `stable-release-plan.json` and `stable-site-release-facts.json` bind each other's digests — a circular definition

- `40:194-196` (plan contents): the plan binds "the exact `sifr-lang/sifr-website` base commit plus generated dispatcher and **release-fact payload digests**."
- `40:164-168` (facts contents): the facts bind "the generation, active stable version, withdrawal/incident facts, **source plan digest**, and dispatcher digests."

Each artifact's digest is an input to the other. Neither can be materialized first. `40:200-206` makes this concrete: the plan is materialized in 40_4 (`40:594-596`) and immutable once approved, so the facts payload must exist and be hashed before approval — but the facts payload embeds the plan digest, which does not exist until the plan is complete.

**Required:** break the cycle in one direction — either the facts reference the plan digest and the plan binds only the *generator inputs* (site base commit + dispatcher digests), or the plan binds the facts digest and the facts drop `source plan digest`.

### 3. Site release facts bind a generation that is not knowable at plan-approval time, and the burned-generation retry rule invalidates the immutable plan

`40:164-166` requires the facts to bind "the generation". `40:143-151` allocates `<N>` at publication as `max(current index, retained snapshots) + 1`, and states that a failed attempt **burns** `N` so "a retry uses the next generation". `40:180` correctly binds only the *previous* generation into the plan — but `40:195-196` binds the facts payload digest, and that payload's generation is `N`.

Consequences:
- At 40_4 the planner cannot compute the facts digest; `N` is allocated only inside `release-publication.yml`.
- Even if `N` were guessed, a burned attempt shifts it to `N+1`, changing the facts payload and its digest — which the approved plan pins and `40:205-206` forbids rewriting. The retry then fails digest revalidation at `40:646-648`, permanently. This is the same class of deadlock as round-3 finding 2, reintroduced through the new facts artifact.

**Required:** either exclude the generation (and anything else allocated at publication) from the plan-bound facts payload — binding instead a generation-independent *template* digest and recording the realized facts digest in `stable-release-signoff.json` (`40:659-663`), which is the correct home for post-approval evidence — or state that the facts payload is generated inside the publication workflow and never bound by the candidate plan.

### 4. Schema-v2 requires a `stable` channel entry, but stable is fail-closed until M40.5 — every alpha/beta publication in the M40.2–M40.5 window is unsatisfiable

`40:116-118` says the index carries "`channels` mappings for `alpha`, `beta`, and `stable`", and `40:130` makes it an invariant that "Every channel points to an `active` release of the matching version class."

`40:396` moves every producer/consumer to schema v2 in 40_2, `40:415-416` keeps stable "fail-closed until [its] owning milestone", and `40:314` states "No publication workflow can accept stable yet." So between M40.2 and M40.5 the only legal index has no stable release to point at. Whether `channels.stable` may be absent is never stated, and `40:302-307` lists "channel/release mismatch" as a validator rejection.

This also contradicts the current contract at `internal_docs/distribution_pipeline.md:76` ("Stable metadata remains absent until stable-channel release architecture changes the stable-channel rules") and `:110` (schema-v1 always contains both alpha and beta).

Related and unresolved by the same wording: 40_2's DoD `40:448-449` ("Fresh stable install, stable-to-stable update, stable no-op, exact stable pin … work") and its demo `40:471-472` (beta-to-stable `--force`) require a stable release that cannot exist at 40_2. 40_3 solves the analogous problem explicitly with a mock harness (`40:548-551`); 40_2 does not say so.

**Required:** one sentence stating that `channels.stable` is absent until GA activation and the active-release invariant applies only to present channels; and one clause marking 40_2's stable install/update DoD and demo as fixture/mock-index backed.

### 5. The `sifr-release-index` lease is held across an unbounded wait on a foreign-repository run, with no timeout or abort path

`40:430-433`: "The main workflow **retains the `sifr-release-index` concurrency lease while it waits for and verifies the correlated site run**." `40:424` ends the site workflow at a "terminal run result that the main workflow polls".

Nothing bounds that wait. A site run that hangs (queued for a self-hosted runner, awaiting an environment reviewer, stuck in deployment) is never terminal, so the lease is never released. Because `40:498` serializes rollback through the same group, an emergency rollback — the one operation whose whole purpose is time-bounded recovery (`40:482-484` names an "acknowledgement target") — is blocked behind the stuck run. `40:496-497` defines retry behavior for three windows (before version publication, after publication before activation, during channel rollback); the post-activation site-wait window is not one of them, and `40:530-533` covers resumption after a *failed* run, not a non-terminating one.

**Required:** give the poll a bounded deadline, state that deadline expiry is a terminal failure that releases the lease and leaves the index activated (resumable per `40:532-533`), and state that rollback may proceed once the lease is released.

---

## Optional polish (non-blocking)

- **Cross-doc drift, unfixed since round 3.** `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:158-159` still assigns stable-candidate execution to `milestone_40_1`, and `:137` still calls `milestone_40_1` the downstream consumer, while `40:53` and `40:260-261` put both before/inside `milestone_40_0`.
- **Index drift, unfixed since round 3.** `plans/phases/index.md:50` still lists Phase 40 as `unspecified` against `40:3` `implementation-ready`.
- **`stable-site-release-facts.json` retention.** The retention list at `40:500-506` covers version assets, plans, sign-offs, snapshots, and incident records but not the deployed facts payload; only its digest survives, inside the sign-off (`40:661-662`). Cheap to add.
- **`distribution_release` suite pin.** `40:281-282` and `40:707` pin `full`; `representative` resolves to the same case set. Harmless.
- **`profile_runner.py` headroom.** 739 lines, gaining `rust_interop_checks` (hardening_1) and `documentation_checks` (`40:274-277`). Still under the 900-line cap; naming the decomposition in 40_0 avoids pushing it into an unrelated PR.

## What remains strong

- `40:67-80` / `40:756-760`: canonical-cutover policy stated once and never violated across 797 lines — no shim, dual metadata, legacy reader, fallback URL, installer, or path.
- `40:143-151` + `40:530-533` + `40:785-787`: generation reservation, burned-generation retention, next-number retry, and post-activation resume-without-remutation are unambiguous (the only defect is finding 3's new coupling, not the rule itself).
- `40:207-236`: deriving the single-enforcement-path design from GitHub assets' lack of compare-and-swap, and explicitly refusing to claim signing or notarization.
- `40:279-281` + `40:310`: the round-4 duplicate-`editor-release` regression is properly closed and matches `release.json` / `developer_tooling/runner.py:142-151`.
- `40:510-517` + `40:536-537` + `40:611-613` + `40:680-681`: rollback reconciliation is now staged fixture → real docs/range → published drill, with exactly one owner per assertion.
- `40:396-409`: `rc` removal remains falsifiable — `self_update_install_receipt.schema.json` and `preview-release.yml:59` still contain it; both live `--clobber` sites remain in scope.
