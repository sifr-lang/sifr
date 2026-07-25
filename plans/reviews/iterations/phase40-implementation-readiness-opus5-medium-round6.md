# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 6)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (829 lines) against all five artifacts in `plans/reviews/iterations/`, plus `plans/phases/33_preview_distribution_and_release_automation.md`, `internal_docs/distribution_pipeline.md`, `scripts/distribution/*.sh`, `.github/workflows/{preview-release,local-first-validation}.yml`, `verification/profiles/release.json`, `verification/owners.json`, `verification/runner/sifr_verify/profile_runner.py`, `verification/areas/{distribution_release,developer_tooling,rust_interop}/`, `editor_integrations/vscode/`, and both active Rust-interop issue plans.

## Round-5 resolution audit

| R5 finding | Status | Evidence |
|---|---|---|
| **M1.** Wrong site slug (`sifr-lang/sifr-website`) | **Resolved** | `40:32,175,203,429,437,441,686` now use `sifr-lang/sifr-blog-website`, which matches the real remote (`git -C ../sifr-blog-website remote -v` → `https://github.com/sifr-lang/sifr-blog-website.git`). Path `apps/sifr-site/public/install/` still matches `scripts/distribution/create_new_version.sh:109`. Minor attribution residue → polish 1. |
| **M2.** Plan ↔ site-facts digest cycle | **Resolved** | `40:171-177` makes the facts' "realized payload and digest … post-approval evidence recorded in sign-off, not candidate-plan inputs"; `40:203-205` binds only base commit + dispatcher digests + facts **schema/generator source** digests and states "the plan does not bind the generation-dependent realized facts payload". One-directional: facts → plan digest. |
| **M3.** Plan bound a generation not knowable at approval | **Resolved** | Same lines; realized generation now lands in sign-off (`40:692-693`), retention (`40:530-532`), and exit gate (`40:815`). |
| **M4.** Schema-v2 required `stable` while stable is fail-closed | **Resolved** | `40:120-121` "conditionally required `stable` mapping"; `40:131-135` states preview⇒absent / active⇒required / one-way; `40:313-317` validator rejects both violations and the active→preview transition; `40:130` narrowed to "Every **present** channel". M40.2 DoD `40:470` and demo `40:492-493` are now fixture/mock-index backed with `ga_status: active`. |
| **M5.** Unbounded lease held across foreign-repo wait | **Resolved** | `40:443-449` hard 20-minute deadline → cancellation request, terminal failed attempt, exit, lease release; `40:450-453` downstream re-fetch and generation/digest match immediately before commit/deploy; `40:558-562` resume semantics + "rollback may proceed and supersedes/cancels any outstanding site attempt". Stale-deployment path is closed in two independent places (cancel + downstream recheck). |
| R5 polish: site-facts retention | Resolved | `40:530-532`, `40:815`. |
| R5 polish: cross-doc drift, index status, `profile_runner.py` headroom | Unaddressed | See polish 5–7. |

Rounds 1–4 findings all remain closed: `rc` removal is still falsifiable (`40:410-416` vs. live `verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json:31`, `.github/workflows/preview-release.yml:59-60`, `scripts/distribution/generate_dispatchers.sh:83-88`, `crates/sifr/src/self_update_metadata.rs:40-72`); both `--clobber` sites (`preview-release.yml:269,309`) remain in scope (`40:459-461`); no fifth profile; `editor-release` executes exactly once (`release.json` `legacy_facade.tooling_suites:["full"]` + `developer_tooling/runner.py:148`); `editor_integrations/vscode/dist` is `.gitignore`d with no committed VSIX, so `40:611-613` is a real (local-only) cleanup; upstream ownership of `rust_interop_checks` / `stable_support_claims.json` is correctly deferred (`40:52-61`, `40:295-298`) and matches `rust-interop-verification-matrix-hardening.md:135-157` and `rust-interop-runtime-ecosystem-certification.md:147-158` — `rust_interop` is genuinely absent from every profile today and `stable_support_claims.json` does not yet exist, exactly as the plan's entry condition assumes.

Two mutation/retry windows remain broken.

---

## Material findings

### 1. The pre-activation retry contract contradicts write-once: a burned generation permanently blocks the GA version

`40:555-556` asserts recovery: "Failure after reserving generation `N` but before index replacement leaves `N` retained and inactive; **retry activates `N+1` or later**."

But by `40:682-686` the mutation order is version assets → verify → Marketplace → index generation. So any failure in the reserved-`N` window happens **after** version publication, and re-entering the workflow hits fail-closed preconditions:

- `40:223-224`: "publication fails if the version tag, archive, checksum, installer, or release-plan asset already exists";
- `40:721-723` makes it explicit as a negative case: "existing version assets … prevent sign-off";
- `40:627-628` requires the protected workflow to `vsce publish` "the recorded VSIX without rebuilding" — a second attempt republishes an already-published Marketplace version, which the registry rejects;
- `40:225` offers only "rebuilds use a new version", and `40:211`/`40:216` forbid amending the approved plan, while `40:638-639` requires any changed VSIX/package-version input to change the candidate-plan digest.

Failure scenario: version assets and the plan asset publish, Marketplace publish succeeds, the pre-replacement generation recheck (`40:148-149`) fails because an alpha release landed. Generation `N` is burned as designed, but every retry path is closed — re-run fails at `40:223-224`, and a new version invalidates the approved immutable candidate. GA at that version becomes unreachable. `40:526-527` enumerates "after version publication but before channel activation" as a window whose retry behavior must be *defined*, but nothing in the plan permits skipping already-completed write-once steps, and `40:557` grants resume-without-re-publication only for the **post**-index-replacement window.

**Required:** state that the pre-activation retry is an explicit idempotent resume that verifies and reuses already-published version assets, plan asset, and Marketplace publication rather than re-uploading them (and that presence of exactly the recorded digests is a pass, not the `40:223-224` rejection) — or state that pre-replacement failure is terminal for the version and that GA re-qualification under a new version is the accepted cost. Today `40:555-556` and `40:223-224`/`40:721-723` cannot both hold.

### 2. The immutable candidate plan pins the previous release-index generation, with no freeze, TTL, or re-approval path

`40:193` binds "governed release-index previous generation and digest" into the candidate plan; `40:146-149` requires the mutation workflow to reacquire and verify both immediately before replacement, and `40:677` revalidates "metadata generation" before mutation. `40:211` makes the plan immutable once approved.

Because alpha and beta publication (`40:229-231`) and rollback (`40:519`) mutate the same index through the same group, **any** intervening preview publication or rollback between candidate approval (`40:594-596`, M40.4) and GA activation (`40:669-698`, M40.5) advances the generation and permanently invalidates the approved plan — the same irrecoverable state as finding 1. The plan acknowledges only that "stale release plans fail before mutation" (`40:149`); it never bounds the exposure. Grep confirms no `freeze`, `expire`, or re-approval language anywhere in the file.

Note the asymmetry: rollback resolves its expected previous generation at run time, so only the stable path carries this expiry.

**Required:** pick one — (a) declare an explicit `sifr-release-index` publication freeze between candidate approval and GA activation (with the approval TTL and who may lift it), (b) drop the previous generation/digest from the plan-bound inputs and recheck the live index only inside the workflow (`40:148-149` already provides the anti-race property), or (c) define a bounded re-approval that re-resolves only the previous-generation binding without changing the version or any qualified artifact.

---

## Optional polish (non-blocking)

1. **Slug attribution.** `40:31-33` still reads the slug as "Phase 33 preview distribution and **its** current site-repository handoff". Phase 33 records only a local path (`33:28`), and no other doc, script, or workflow in the repo records the slug — `internal_docs/distribution_pipeline.md:13,28,38` uses `<site-repo>` and `create_new_version.sh:26,40` takes `--site-repo` as a path. Phase 40 is the first place the slug appears; say so, or add it to Phase 33.
2. **M40.3 retry enumeration is stale.** `40:526-527` still lists three windows; the fourth (post-activation site wait/timeout/resume) is now specified only in DoD `40:558-562`. Add it to scope so the scope and DoD agree.
3. **Rollback's own site deployment.** `40:510-512` says rollback reuses the validator, generation recheck, snapshot publication, and concurrency group, but not the post-index site dispatch, 20-minute deadline, and resume semantics — implied only by `40:561-562` and `40:711-712`. One clause fixes it.
4. **20-minute lease hold vs. incident acknowledgement.** `40:443-449` blocks rollback for up to 20 minutes; `40:507` defines an acknowledgement target in the execution issue. State that the target must exceed the site-wait deadline.
5. **GitHub concurrency-group semantics.** `40:235-240`/`40:519` treat the group as a queueing lease. GitHub retains at most one *pending* run per group and cancels older pending runs, so a queued emergency rollback can be cancelled by a later-queued preview publication. Correctness is preserved by the generation recheck (a cancelled run mutates nothing), but the availability claim deserves a sentence.
6. **Cross-doc drift, unfixed since round 3.** `plans/issues/active/rust-interop-verification-matrix-hardening.md:12-13` and `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:137-139,158-159` still assign the stable-candidate consumer to `milestone_40_1`, while `40:52-61` and `40:285-286` place it before/inside `milestone_40_0`. `plans/phases/index.md:50` still lists Phase 40 as `unspecified` against `40:3` `implementation-ready`.
7. **`profile_runner.py` headroom.** 739 lines today, with `rust_interop_checks` (hardening_1) and `documentation_checks` (`40:274-277`) both landing in it. Still under the 900-line cap; naming the decomposition in M40.0 avoids pushing it into an unrelated PR.

## What remains strong

- `40:67-80` / `40:788-791`: canonical-cutover policy stated once and never violated across 829 lines — no shim, migration, dual metadata, legacy reader, fallback URL, installer, or path.
- `40:120-135` + `40:313-317`: conditional `stable` semantics, one-way `ga_status`, and validator rejections are now fully falsifiable and consistent with M40.2's fixture-backed DoD/demo.
- `40:443-453` + `40:558-562`: bounded site wait, cancellation, lease release, downstream generation/digest recheck, resume-without-remutation, and rollback supersession together close the stale-deployment class in two independent layers.
- `40:171-177` + `40:203-205` + `40:692-693`: candidate inputs are generation-independent; realized facts are sign-off evidence. The round-5 cycle and unknowable-generation defects are cleanly gone.
- `40:235-246`: deriving the single-enforcement-path design from GitHub assets' lack of compare-and-swap, and explicitly refusing to claim signing or notarization.
