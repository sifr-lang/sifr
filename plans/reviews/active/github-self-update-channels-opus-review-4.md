I have enough material to write the review. Below are findings ordered by severity, with file:line references.

---

## Review: GitHub-hosted self-update channel metadata migration

### Merge-blocking issues

**1. Transition gap — first run of the new workflow will fail at `Verify published release assets` for any pre-existing release that lacks `sifr-installer-<version>`.**
- `.github/workflows/preview-release.yml:272-286` runs `verify_release_installer_asset.py` against **both** channels' current versions from the just-generated `channel-metadata/channels.json` before publishing it.
- The "other" channel's version typically points at a release that was created under the old pipeline and has only `sifr-${VERSION}-${target}.tar.gz` / `.sha256` assets, not `sifr-installer-<version>`. `scripts/distribution/verify_release_installer_asset.py:25` then raises "missing release asset sifr-installer-…" and the workflow aborts before publishing `channels.json`.
- In the bootstrap path (`.github/workflows/preview-release.yml:191-202`) the situation is the same: `bootstrap_channel_metadata.py` selects the latest public prerelease for the other channel, which won't have the new installer asset either.
- Neither `docs/self_update.md` nor `internal_docs/distribution_pipeline.md` documents an "upload `sifr-installer-<existing-version>` to the current alpha and beta releases first" prerequisite. Without that step the first migration run cannot succeed.

  Fix: either pre-upload the missing installer asset to the existing alpha/beta releases as part of the migration, OR have the workflow auto-regenerate and upload the missing installer for the unchanged channel before verifying (recommended, since the build matrix is checked out), OR explicitly document the manual backfill as a one-time migration step in `docs/self_update.md`/`internal_docs/distribution_pipeline.md`.

**2. `Verify published release assets` and `Publish channel metadata` order admits a partial-write window the workflow claims to prevent.**
- `.github/workflows/preview-release.yml:272` runs *after* `Publish GitHub prerelease` (line 235-270) already uploaded the new `sifr-installer-${VERSION}` to the version release. If the verify step fails (e.g., other-channel asset missing per finding #1), the workflow exits without publishing `channels.json` — good — but the new version release is already live with a new installer asset that does not match any channel pointer. Users running `--version` pins succeed; channel resolution still points at the old version. That's the safer half of the asymmetry, but it should be called out in the recovery note / docs (or move the prerelease upload to happen *after* the cross-channel verify by uploading a draft first and finalizing afterwards).
- More importantly: between `Publish GitHub prerelease` and `Publish channel metadata`, an unrelated workflow operator who manually edits the `channels` release will lose changes — but the new `concurrency.group: preview-release-channels` only serializes *this* workflow, not concurrent gh CLI usage or workstation `create_new_version.sh` runs. The doc note at `internal_docs/distribution_pipeline.md:294-296` is the only guardrail. Acceptable, but flag this in the recovery note (`scripts/distribution/create_new_version.sh:287-305`).

### High-priority non-blocking issues

**3. Bootstrap path silently downgrades to a `::warning` and continues even when curl fails for transient reasons.**
- `.github/workflows/preview-release.yml:188-202`: a network blip during `curl -fsSL .../channels/channels.json` triggers bootstrap mode. Bootstrap then re-derives `latest[other_channel]` from public prereleases — which may be older than what was actually in the live `channels.json`. The result: a real-but-transient fetch failure causes the OTHER channel pointer to silently regress.
- The `bootstrap_channel_metadata.py:36-39` downgrade guard only protects `args.channel`, not the other channel.
- Mitigation: retry curl (e.g., `for attempt in 1 2 3; do … sleep 5; done`) before falling to bootstrap, OR require an explicit `workflow_dispatch` input like `bootstrap: true` to take the bootstrap path. Today the bootstrap fall-through is automatic and silent.

**4. `read_channel_versions.py` provides no version-format validation; truncation of the fetched JSON yields empty strings that propagate.**
- `scripts/distribution/read_channel_versions.py:20-22` only verifies the values are strings.
- `.github/workflows/preview-release.yml:204`: `read -r current_alpha current_beta < <(scripts/...)`. Process substitution exit codes do **not** propagate under `set -euo pipefail` in bash. If `read_channel_versions.py` raises `SystemExit("…")`, the bash `read` succeeds with empty variables; `generate_channel_metadata.sh` then catches the bad version format and exits — but the failure mode is opaque. Either capture the inner exit code explicitly (`tmp=$(scripts/...); rc=$?; (( rc == 0 )) || exit 2`), or have `read_channel_versions.py` validate version format with the same regex used elsewhere.

**5. Workstation `create_new_version.sh` and CI `preview-release.yml` independently compute `channels.json` from different sources and can drift.**
- Workstation (`scripts/distribution/create_new_version.sh:177-183`, `validate_site_dispatchers` at line 145-169) derives `CURRENT_ALPHA/BETA` from the site repo's dispatcher files.
- CI (`.github/workflows/preview-release.yml:182-217`) derives them from the live GitHub `channels.json` (or bootstraps).
- The local `channels.json` produced by the workstation run is evidence-only (never uploaded), and validation at `verification/areas/distribution_release/tools/validate_self_update_metadata.sh:127-130` only checks intra-plan consistency. So drift is harmless functionally but the docs (`internal_docs/distribution_pipeline.md:287-296`) should make it explicit that the workstation `channels.json` is evidence-only and the workflow is the source of truth.

### Lower-priority / non-blocking

**6. `head -n 1 … | grep -q '^#!'` race with very short installer.** `.github/workflows/preview-release.yml:230-233` and the runner check at `crates/sifr/src/self_update_runner.rs:195` use the same shebang check. They differ in that the runner first reads up to a newline; the workflow uses `head -n 1` which reads the whole first line. Both behave correctly for valid installers. No issue.

**7. Channel release flags on first-create vs edit.** `.github/workflows/preview-release.yml:292-306`: `gh release edit channels --draft=false --latest=false` does not pass `--prerelease=false`. If an operator manually flips the channels release to prerelease via the UI, the workflow won't reset it. Minor.

**8. `mktemp` for `current_metadata` is never cleaned up.** `.github/workflows/preview-release.yml:187`. Runner-scoped tmpfs gets cleaned on job teardown so it's not a leak. Cosmetic.

**9. `rmdir` of `…/install/metadata` only runs in the test fixture, not in the live site repo.**
- `verification/areas/distribution_release/cases/common.sh:169-170` strips it from fixtures.
- The actual website cleanup is a doc note in `internal_docs/distribution_pipeline.md:28` ("Remove any stale `public/install/metadata/` directory on the next site rollout"). Acceptable since the site repo is separate, but worth flagging: until that site PR lands, old binaries continue to resolve `channels.json` from the old website URL with stale content. Concurrent operation of old (sifr.sh) and new (GitHub) channel metadata sources is the actual transition. The `docs/self_update.md:59-70` "Migrating Earlier Previews" guidance acknowledges this — fine.

**10. `verify_release_installer_asset.py` only checks `size >= 1024`, not checksum.** `scripts/distribution/verify_release_installer_asset.py:21-24`. This is intentional (matches the local pre-upload shebang/size check) and the immutable installer asset embeds verified checksums for the .tar.gz payloads, so further content verification is downstream. Note rather than concern.

**11. Bootstrap downgrade guard considers only the channel being published.** `scripts/distribution/bootstrap_channel_metadata.py:36-39`. If the other channel's "latest public prerelease" picked by bootstrap is older than what was in the (now-unfetchable) `channels.json`, this regresses that channel silently. Tied to finding #3.

**12. `cancel-in-progress: false`** (`.github/workflows/preview-release.yml:29`) correctly serializes channel publication. Two workflow dispatches in quick succession queue rather than race. Correct call.

---

### Summary

There is **one merge-blocker** (#1: missing transition plan / pre-existing releases lack the new installer asset → first migration run fails at `Verify published release assets`). The simplest remediation is to document an explicit one-time backfill in `docs/self_update.md` (or auto-regenerate the missing installer for the unchanged channel inside the workflow). Findings #2–#5 are worth addressing for robustness but don't have to block this PR; the rest are minor.

The validation coverage looks good: YAML parsing is now tested (`preview_release_workflow_yaml_parses.sh`), bootstrap behavior is covered (`channel_metadata_bootstrap_from_github_releases.sh`), and the missing-installer rejection path is exercised (`channel_metadata_missing_installer_rejected.sh`). The atomicity intent in the workflow (concurrency group, generate-then-verify-then-publish) is sound; the gap is purely at the first transition.
