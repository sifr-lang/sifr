# Review: Phase 40 — Stable Channel GA Promotion and Release Governance

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (577 lines) against the actual scripts, crates, schemas, manifests, profiles, workflows, and submodule state.

The plan is well above average: the canonical-cutover policy is stated cleanly and honored (no shims/fallbacks/dual metadata anywhere), the release-plan → sign-off split at lines 164–169 correctly avoids post-publication evidence rewriting an approved plan, provenance-by-resolved-commit (lines 171–174) is right, and the integrity-boundary honesty at lines 184–188 (no signing claims) is exactly correct. The Phase 36 and Phase 38 handoffs are accurate.

But there are eight material gaps where a gate is not falsifiable, not owned by any milestone, or contradicts the repository surface it names.

---

## Blocking / material findings

### 1. `rc` is declared rejected but no milestone removes it, and line 133 preserves it

Line 82–84: "`rc` is not a public channel… Public dispatchers, release metadata, and self-update reject `rc` channel selection."

`rc` is live in four producer/validator surfaces today:
- `verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json:25-32` — version pattern `-(alpha|beta|rc)\.[0-9]+` and channel enum `["alpha","beta","rc"]`
- `scripts/distribution/generate_version_installer.sh:131-132` — `APP_CHANNEL` derived from any prerelease label
- `scripts/distribution/generate_dispatchers.sh:78-90` — rc version pins pass `preview_channel_for_version` (then dead-end in `normalize_channel:92-107`)
- `.github/workflows/preview-release.yml:59` — accepts rc in input validation

Line 133–134 then says receipts "remain `schema_version: 2`; their version patterns and channel enum are **extended** to accept stable releases." Extending without removing leaves `rc` in the receipt enum, directly contradicting line 83. No milestone scope item names any of these four files for rc removal. Milestone 40_2's negative-validation list (line 331) tests that "`rc` requests fail" — which already passes in Rust (`crates/sifr/src/self_update_metadata.rs:243-245`) and would pass without touching the receipt schema. The gate is not falsifiable against the real defect.

**Required:** state explicitly that `rc` is deleted from the receipt schema enum/pattern, the installer `APP_CHANNEL` derivation, the dispatcher pin resolver, and `preview-release.yml`, and scope it to milestone 40_2.

### 2. The one suite Phase 40 actually depends on is the one not wired into any profile

`grep rust_interop verification/profiles/*.json` → zero hits. The `rust_interop` area (`matrix`, `tiers`, `compatibility-matrix`, `stale-drafts` — all four exist, `verification/areas/rust_interop/manifest.json`) is executed by no profile lane.

Line 273 makes `run_all_tests.sh --profile release` mandatory for the release commit, and lines 543 / 573 assert "Public Rust interop claims never exceed the compatibility matrix." The release profile cannot enforce that. Wiring it is `hardening_1` in `plans/issues/active/rust-interop-verification-matrix-hardening.md:129`, which is unmerged — and Phase 40's Upstream Handoffs (lines 43–53) never lists either active rust-interop issue as a dependency, despite `rust-interop-runtime-ecosystem-certification.md:39-45` gating Track A behind `hardening_1`–`hardening_4`.

### 3. The stable-claim gate is prose, not a mechanism — and ignores the artifact its upstream issue mandates

`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:132-163` (`certification_0`) requires creating `verification/areas/rust_interop/data/stable_support_claims.json` plus a stable-candidate check, and states verbatim: *"Update Phase 40 `milestone_40_1`, its promotion checklist, and `milestone_40_4` activation gate to execute the stable-candidate check."* `:316-326` repeats it as a hard Stable Release Constraint.

`grep -c stable_support_claims` over the Phase 40 file → **0**. Instead, line 271 says stable claim validation "fails if docs advertise a `future-owned-by-separate-phase` surface" and line 406 says "Derive Rust interop support wording from the compatibility matrix." Neither is implementable: nothing maps a docs sentence to a matrix row. There are currently **11 of 34** `future-owned-by-separate-phase` rows in `verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`, so this is not a hypothetical edge.

**Required:** name `stable_support_claims.json` and the stable-candidate check as the enforcement mechanism, in 40_0 (artifact) / 40_1 (gate) / 40_4 (docs derivation).

### 4. VS Code repository identity is wrong, and extension provenance is duplicated

Line 34 names "the separate `sifr-lang/sifr-vscode` repository checked out at `editor_integrations/vscode`". `.gitmodules` registers one submodule `editor_integrations` → `https://github.com/sifr-lang/editor-integrations.git`; `vscode/` is a *directory inside* it, not a repository. (`package.json` `name` is `sifr-vscode`, which is likely the source of the confusion.)

Consequently line 158's "VS Code extension repository commit" is the `editor-integrations` submodule SHA — already bound by line 145's "recursive submodule SHAs." Two plan fields for one value, with the risk they disagree.

### 5. Marketplace publication ownership is contradictory and overstates the existing pipeline

Line 418: "Add protected Marketplace publication to **the existing extension pipeline**." There is no such pipeline. `editor_integrations/` contains no `.github/`; `editor_integrations/vscode/package.json` scripts are `compile, lint, typecheck, test, test:extension, package` — no `publish`, no `vsce publish` anywhere in the tree. The only automation is `verification/areas/developer_tooling/check_vscode_extension.py:137-150` driving those npm scripts locally.

Meanwhile line 463 puts publication in the main-repo protected workflow: "Publish and verify the recorded VS Code extension." These are two different owners in two different repositories for one mutation, and the main repo's protected environment cannot govern a workflow living in the submodule's upstream repo. Line 429 ("Marketplace failure leaves the stable channel unactivated") is only satisfiable under the line 463 reading.

**Required:** pick one owner. Given line 465 orders channel activation last, Marketplace publication must be a step in the main-repo protected workflow consuming the recorded VSIX.

Related: `editor_integrations/vscode/dist/` holds committed VSIXs for `0.0.0`–`0.1.3` while `package.json` version is `0.1.7`. Line 417's "Produce a VSIX from the recorded extension commit and bind its version and SHA-256" must also state that stale committed VSIXs are removed, or digest binding will race against checked-in artifacts.

### 6. Write-once is asserted for stable but nothing removes `--clobber` from the shared publication path

Lines 176–182 declare stable assets write-once and "stable publication never uses `--clobber`." Line 68 and line 181 make alpha/beta share the same canonical index and the same concurrency group.

`.github/workflows/preview-release.yml:267-269` uses `--clobber` on **version-release assets**, and `:307-309` on `channels.json`. Under a shared release index whose `releases` map binds installer and artifact SHA-256 (lines 105–109), a clobbered alpha version asset silently invalidates a recorded digest. No milestone scope item names `preview-release.yml` at all. Same for the concurrency-group edit line 181 requires (current group `preview-release-channels`, `:27-29`) and for adding the protected environment (the workflow has no `environment:` key).

### 7. The compare-and-swap invariant is asserted against storage that cannot provide it

Line 127: "A metadata update must present the expected previous generation and digest; stale release plans fail before mutation."

`channels.json` is a GitHub release asset uploaded with `gh release upload --clobber` (`preview-release.yml:287-309`). Asset upload offers no atomic compare-and-swap; read-then-write is racy by construction. The only real serialization is the workflow concurrency group — and `scripts/distribution/create_new_version.sh:322-348` can mutate metadata and dispatchers **locally** via `--real-run`, outside that group.

**Required:** state that after cutover all release-index mutation flows exclusively through the single protected workflow, and that the local path is dry-run/plan-only. Otherwise the expected-generation check (lines 127, 366, 385) is advisory, not fail-closed.

### 8. Where `stable-release-plan.json` lives is unspecified → circular-evidence risk

Line 143 binds the plan to an "immutable source commit SHA"; line 273 records the release-profile report identifier and digest *in the plan*; line 281 requires "a passing plan references a passing release-profile report for the same source commit."

If the plan is checked in, writing the report digest into it changes the commit the report was taken at — a genuine circularity. The plan never says. Precedent is work-dir-only (`scripts/distribution/create_new_version.sh:207` → `${WORK_DIR}/plan.txt`), which resolves it, but line 152 also lists a "release-plan asset" among write-once published assets, implying it is published rather than committed. Make it explicit: generated into a work directory at the release commit, published as a release asset, never committed.

### 9. Net-new docs checks are added but not invoked by the phase's own validation contract

Line 404–405 adds executable docs checks for GA sections, internal/public drift, target claims, command examples, release version, and schema references. Correct that this is net-new: there is no `documentation` verification area, and the only docs gate in the repo is error-code link sync (`scripts/check_docs_error_code_links.py` → `verification/areas/diagnostics/checks/docs_sync.py`, registered at `verification/areas/diagnostics/manifest.json:24-25`).

The Validation Contract (lines 502–522) names no docs suite, and 40_4's DoD (line 424) is prose ("Docs, release plan, compiler behavior… agree"). Name the area/suite the new checks register into and add it to line 518–521. Note the docs that must change are concrete and currently preview-only: `docs/installation.mdx:34-38,54,65,110-131` and `:204` ("Stable channels… are not yet available"), plus `docs/self_update.md`.

### 10. `--profile create-pr` gives zero coverage of everything Phase 40 changes

Line 502–504 makes `create-pr` the per-PR gate. `verification/profiles/create-pr.json` has **no `distribution_release` entry** (only `merge.json:223`, `release.json:201`, `nightly.json:202` do), and no `developer_tooling --suite editor-release` selection exists in any profile. So milestone 40_2's dispatcher/installer/receipt/self-update work and 40_4's extension work are unexercised by the mandatory per-PR command. Either require the area suites per-PR for Phase 40 milestones, or add the selections to `create-pr.json` as scoped work.

---

## Optional polish

- **Redundant validation commands.** `release.json` already runs `diagnostics` (:95), `developer_tooling` (:174), and `distribution_release` (:201). Lines 518, 519, 521 are therefore subsumed by line 517. The only non-redundant line is 520 (`rust_interop`) — which is finding #2. Collapsing this makes the contract self-documenting.
- **Duplicated evidence fields.** Line 156 ("Phase 30 parity, Phase 34 generated-code, and Phase 35 performance evidence") is already implied by line 155's release-profile report digest, since `release.json` runs `stdlib_parity` (:242), `generated_code_quality` (:183), and `performance` (:192). Prefer the single report digest.
- **`distribution_release` suites are not distinct.** Line 518 pins `--suite full`, but `representative` and `full` map to the identical adapter case over the same case directory. Harmless, but the pin implies a scope difference that doesn't exist.
- **Index drift.** `plans/phases/index.md:50` lists Phase 40 status as `unspecified`; the phase file line 3 says `implementation-ready`.
- **Sequencing wrinkle.** 40_3's rollback drill (line 389) is authored before the publication workflow exists (created in 40_5), then re-validated at line 481. Correct but worth stating that the 40_3 drill targets the local metadata path and 40_5 re-runs it against the workflow.
- **Dispatcher cleanup gap.** `scripts/distribution/generate_dispatchers.sh:226` clears `versions/` and `metadata/` but not a stray top-level `channels.json`, which `create_new_version.sh:138-148` prefers over GitHub. Worth folding into 40_2's cutover so schema-v1 residue cannot shadow the v2 index.

---

## What is genuinely good

- Lines 55–68 and 544–548: the canonical-cutover policy is stated once and never violated — I found no shim, fallback URL, dual metadata, or migration path anywhere in the 577 lines.
- Lines 164–169: candidate plan immutable, sign-off separate with its own schema referencing the plan digest. No circular signoff.
- Lines 184–188: refusing to claim signing/notarization and naming the actual integrity boundary is the right call and rare in release plans.
- Lines 47–53 match the real four-value vocabulary in `rust_interop_compatibility_matrix.json` exactly, and lines 43–46 correctly declare Phase 38 (draft, no `status:` line, `38:3`) a non-prerequisite — Phase 39 gets this wrong at `39:47`.
- Lines 100–128 keeping executable URLs out of metadata and deriving them from repository constants is the correct hardening of today's `self_update_metadata.rs:10-11` model.
