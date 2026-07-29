# Review — PR #3070, head `74c5dd02f1ca692c0fb1f9c8b50004827028cdfb`

**Correction to the request:** the full SHA given (`74c5dd02f50ee2fb552573104186295b54c34a09`) does not exist. The real head — same `74c5dd02f` prefix, confirmed via `gh pr view 3070 --json headRefOid` — is `74c5dd02f1ca692c0fb1f9c8b50004827028cdfb`. All work below is against that commit, reviewed in a fresh clone checked out at that exact SHA (so the dirty submodule worktree state was excluded, not merely ignored).

## What I verified

**Scope and shape.** Exactly seven files, all additions, 84 insertions, 0 deletions, parent `afd25c392` == `origin/main` (PR is one commit ahead). `git diff --check` clean; file-size guardrail PASS (2978 files).

**Custody.** `governance.evidence_custody.run_evidence_custody_checks()` re-run at the head with the correct base → `evidence custody ok` (exit 0). (An initial run reported a scope violation; that was a clone artifact — the local clone's `origin/main` lagged — not a property of the PR.)

**Digest closure — all seven, recomputed:**

| File | SHA-256 | Bound by |
|---|---|---|
| `stable-release-plan.json` | `3e4c7b7c5069…` | matches request |
| `release-profile-report.json` | `e5200229dfda…` | plan `.release_profile_report.sha256` |
| `qualification-artifact-index.json` | `503f4fcc0dcf…` | plan `.qualification_artifact_index.sha256` |
| `stable-support-claims.json` | `b62f5b936be0…` | plan `.rust_interop.stable_support_claims_sha256` |
| `rust-validation-report.json` | `95176b5937b4…` | plan + report `result_artifacts[]` |
| `documentation-report.json` | `a7a13122d6e8…` | plan `.documentation_report.sha256` |
| `release-notes.md` | `2f90a78a90bd…` | plan `.release_notes_sha256` |

All canonical JSON, all newline-terminated. `report_id` = `release-c9d611fb7c7c-fa3d95c04f8a` reproduces exactly from `commit[:12]` + `canonical_profile_digest(release.json)[:12]`; `plan_id` = `stable-0.1.0-c9d611fb7c7c` ✓.

**Provenance against the qualified source `c9d611fb…`:** `cargo_lock_sha256` `602c5cc8…` ✓; `compatibility_matrix_sha256` `1855919f…` ✓; `facts_schema_sha256` `b563df39…` ✓; `facts_generator_sha256` `554349f0…` ✓ (= `governance/release_plan.py` at source); `profile_manifest_sha256` `fa3d95c0…` ✓; `expanded_selected_areas` reproduces the source profile exactly. All 10 submodule pointers match the source tree's gitlinks byte-for-byte and are identical across plan/report/index; nested `editor_integrations/vscode` = `273fd5d3…` confirmed inside the `editor_integrations` submodule.

**Cross-artifact binding (checked programmatically, beyond what custody enforces):** every per-target `archive_sha256`/`checksum_sha256` in the plan equals the corresponding qualification-index artifact; `installer_sha256`, `vsix_sha256`, and `vscode.validation_report_sha256` all match their index entries; `desired_release.targets[*]` agrees with `targets[*]`. The apparent oddity that `sysroot_sha256` is identical (`8e109a6b…`) across all four targets is correct — it is the target-independent `sysroot-content-sha256` from `sysroot.toml` (`qualify_stable_target.py:187`), distinct from the four per-target sysroot *archive* digests in the index.

**Qualification custody/expiry.** Index validates with `require_unexpired=True` as of today: expires `2026-08-28T02:17:30Z`, retention 30, `overwrite: false`, run `30416219284` attempt 1, exactly six workflow uploads, exact 20-artifact ID set, every upload name binding version + full source commit.

**Rust claims.** The 29 advertised IDs match `docs/rust-interop.mdx` at the source commit as an ordered list, with identical `category`/`execution_kind` per row. All 7 non-advertised matrix rows are `future-owned-by-separate-phase`. The notes' "a `contract-only` row does not claim runtime-observed support, and future-owned runtime rows are not advertised" is accurate.

**Release-note truthfulness.** macOS 15.0 / glibc 2.39 floors, `sifr.sifr-vscode` (publisher `sifr`, name `sifr-vscode`) 0.2.0 with `>=0.1.0,<0.2.0` (verified in the VSIX submodule's `package.json`), `#stable-support-claims` anchor (`### Stable Support Claims`), `--force` semantics, `schema_version: 2` / rc rejection, and the Windows/package-manager scope carve-outs all corroborate the source-commit docs. First-GA semantics are correct: `transition: ga-activation` with `expected_stable_predecessor` and `rollback_target` both `"none"`, which `validate_release_plan` requires, and the notes describe incident roll-forward rather than rollback.

**No post-approval facts embedded.** No `stable-release-signoff.json`; no `channel_generation`, site-facts digest, run URLs, or timestamps in the plan. `documentation-report.result_sha256` (`998179ec…`) intentionally differs from the release profile's `documentation-release-results.json` (`a7cd2e04…`) — it digests `documentation-stable-qualification-results.json` from the standalone gate (`qualify_stable_documentation.py:174-181`), and `report_id` derives from it correctly.

## Findings

No blocking findings. Four non-blocking items:

**1. Low — `site.facts_generator_sha256` does not pin the code that will run.** `stable_prepare._validate_source_contracts` (`verification/areas/distribution_release/governance/stable_prepare.py:770-780`) resolves this field against the *candidate source* checkout's `governance/release_plan.py`. The pinned `554349f0…` is that file at `c9d611fb`. But `release-publication.yml:159-163` checks out the workflow ref (protected default branch) for governance code on the mutation path, where `release_plan.py` is now `c2c74098…` (the single-maintainer approval-waiver change, which is absent at `c9d611fb`). The check still passes — it compares source-to-source — so nothing fails open in a dangerous way, but the field's name implies a guarantee it doesn't provide. Pre-existing validator design, not introduced by this PR.

**2. Informational — hard publication deadline.** Qualification uploads expire `2026-08-28T02:17:30Z`. Since prepare/publish refetch the six uploads by immutable artifact ID, the entire protected publication must complete before that instant or this candidate becomes unpublishable and requires full re-qualification.

**3. Informational — ephemeral workstation paths in immutable evidence.** `release-profile-report.json` `.command[4]` and every `variants[].argv[0..1]` in `rust-validation-report.json` embed `/private/tmp/sifr-phase40-candidate-rebuild.s49VOp/…/release-output-pass8/…`. Harmless — nothing re-derives these bytes; prepare only digest-checks the committed files — but it is non-reproducible provenance that also discloses the rebuild attempt count.

**4. Informational — follow-up ledger PR required.** `plans/issues/active/phase-40-stable-channel-ga-execution.md:143-144` are still unchecked. Custody correctly forbids mixing that update into this commit, so it must land separately.

**Context on source staleness (not a finding):** `c9d611fb` is now 68 commits behind `main`; Rust-interop certifications 8 and 9 landed after it, so `main`'s compatibility matrix (`48e0732a…`) has diverged from the plan-pinned `1855919f…`. This is correct behavior for an immutable candidate — it advertises only what its own source certified, and the site run pins the exact Sifr commit so published docs match the advertised claim set. Worth being a conscious decision rather than a surprise.

## Verdict

SATISFIED
