# Sifr Preview Distribution Pipeline

Status: preview-release pipeline implementation requirement.

## Installer Model

Sifr preview distribution uses generated shell installer entrypoints with the same high-level shape as Astral's `uv` installer: a channel dispatcher selects an immutable version installer, and the immutable installer owns platform/artifact selection and checksum verification.

preview-release pipeline does not copy or adapt code from `astral-sh/uv`. The current implementation is Sifr-owned shell generation in `scripts/distribution/`, so the uv MIT attribution checklist is recorded as not applicable until copied or adapted uv code is introduced.

## Site Layout

Static site files live under the site repository at `<site-repo>/apps/sifr-site/public/install/`.

The filesystem layout is:

```text
public/install/
  index
  alpha
  beta
```

`index` is the default beta bootstrap. The deployment must serve it at `https://sifr.sh/install`; `alpha` and `beta` are served at `https://sifr.sh/install/alpha` and `https://sifr.sh/install/beta`. These scripts are stable website entrypoints only: they fetch GitHub `channels.json`, resolve the requested channel or version pin, and download `sifr-installer-<version>` from the resolved version's GitHub release.

Channel resolution is not served from the website install tree. It is published as the `channels.json` asset on the `sifr-lang/sifr` GitHub release tag `channels`; both the website bootstrap scripts and `sifr self update` use that metadata and download immutable installer assets from version releases.

The website repository must not publish `public/install/metadata/channels.json` or `public/install/versions/`. Remove any stale `public/install/metadata/` or `public/install/versions/` directory on the next site rollout after this contract is adopted.

This directory layout avoids the impossible static-file shape where `public/install` is both an executable file and a directory for nested channel paths.

## Channel Dispatcher Rules

Generate dispatchers with:

```bash
scripts/distribution/generate_dispatchers.sh \
  --install-root <site-repo>/apps/sifr-site/public/install
```

Dispatcher behavior:

- `/install` defaults to the beta channel from GitHub `channels.json`.
- `/install/alpha` and `/install/beta` select the corresponding preview channel.
- `--channel alpha|beta` and `SIFR_CHANNEL=alpha|beta` select a moving preview channel.
- `--version <semver-prerelease>` selects the matching GitHub release installer directly.
- `stable` and stable-looking versions such as `1.0.0` or `0.1.0` are rejected before download.
- Conflicting `SIFR_CHANNEL`, `--channel`, and `--version` inputs are rejected.
- The dispatcher downloads exactly one immutable GitHub installer asset and preserves its exit status.

The dispatcher never resolves artifacts itself and never compiles from source.

Generate self-update channel metadata with:

```bash
scripts/distribution/generate_channel_metadata.sh \
  --out <work-dir>/channels.json \
  --alpha-version 0.1.0-alpha.1 \
  --beta-version 0.1.0-beta.1
```

The generated `channels.json` shape is:

```json
{
  "schema_version": 1,
  "channels": {
    "alpha": "0.1.0-alpha.1",
    "beta": "0.1.0-beta.1"
  }
}
```

The metadata file is resolution metadata only. It records channel-to-version mappings and must not contain executable URLs. The Rust CLI derives immutable installer URLs from the trusted GitHub release download base URL and the resolved version string. Stable metadata remains absent until stable-channel release architecture changes the stable-channel rules.

First-time GitHub setup for a repository without a `channels` release can be bootstrapped from existing public prereleases:

```bash
for version in <current-alpha-version> <current-beta-version>; do
  scripts/distribution/generate_version_installer.sh \
    --version "${version}" \
    --artifact-dir "target/preview-artifacts/${version}" \
    --out "target/preview-artifacts/${version}/sifr-installer-${version}"
  gh release upload "${version}" \
    "target/preview-artifacts/${version}/sifr-installer-${version}" \
    --repo sifr-lang/sifr \
    --clobber
done

gh release list --repo sifr-lang/sifr --limit 100 --json tagName,isDraft,isPrerelease > current-releases.json
scripts/distribution/bootstrap_channel_metadata.py \
  --releases-json current-releases.json \
  --channel beta \
  --version <current-beta-version> \
  --out channels.json
gh release create channels \
  --repo sifr-lang/sifr \
  --latest=false \
  --title "Sifr self-update channels" \
  --notes "Machine-readable Sifr self-update channel metadata."
gh release upload channels channels.json --repo sifr-lang/sifr --clobber
```

The installer-asset backfill is a one-time migration prerequisite when the
current channel releases were created before GitHub-hosted self-update
metadata. The preview-release workflow verifies that both channel versions in
`channels.json` already have `sifr-installer-<version>` assets before it
publishes or updates the shared `channels` release asset.

The bootstrap helper fills the current channel from the release being published and requires an existing public prerelease for the other channel so schema-version 1 metadata always contains both `alpha` and `beta`.
It refuses to move the current channel backward relative to public prereleases unless a future reviewed release-governance change adds an explicit downgrade flow.

## Preview metadata validation Validation

Preview metadata validation uses mocked immutable generated installers until installer artifact validation adds checksum-verified artifact installers.

Run dispatcher validation with:

```bash
verification/areas/distribution_release/cases/install_default_beta_dispatcher.sh
verification/areas/distribution_release/cases/install_alpha_dispatcher.sh
verification/areas/distribution_release/cases/install_version_pin_dispatcher.sh
verification/areas/distribution_release/cases/install_stable_channel_gated.sh
verification/areas/distribution_release/cases/install_invalid_channel_rejected.sh
verification/areas/distribution_release/cases/install_conflicting_channel_and_version_rejected.sh
verification/areas/distribution_release/cases/install_stable_version_pin_rejected.sh
verification/areas/distribution_release/cases/install_missing_generated_installer_rejected.sh
verification/areas/distribution_release/cases/install_dispatcher_malformed_config_rejected.sh
```

## Artifact Format

Preview artifacts are gzip-compressed tar archives published as GitHub Release assets in `sifr-lang/sifr`.

Each target has:

```text
sifr-<version>-<target>.tar.gz
sifr-<version>-<target>.tar.gz.sha256
```

The archive contains the full toolchain root:

```text
bin/sifr
Cargo.toml
Cargo.lock
sysroot.toml
.cargo/config.toml
lib/sifr/stdlib/sifr/*.sifr
lib/sifr/stdlib/_sifr/*.sifr
crates/sifr_runtime/**
crates/sifr_stdlib/**
vendor/**
```

Archive validation rejects absolute paths, traversal paths, links, special
files, and archives missing required sysroot assets before checksums or
immutable installers are published.

The preview-release pipeline target set is:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

## Artifact Packaging

Package artifacts with an existing binary fixture:

```bash
scripts/distribution/build_preview_artifacts.sh \
  --version 0.1.0-beta.1 \
  --output-dir target/preview-artifacts/0.1.0-beta.1 \
  --binary target/release/sifr
```

For production target builds, use:

```bash
scripts/distribution/build_preview_artifacts.sh \
  --version 0.1.0-beta.1 \
  --output-dir target/preview-artifacts/0.1.0-beta.1 \
  --cargo-build
```

The production path runs `cargo build --release -p sifr --target <target>` for every preview-release pipeline target and fails if any target cannot be built. It does not fall back to another binary or another target.
Production builds remap repository, sysroot, Cargo-home, and rustup-home path
prefixes before packaging so release binaries do not embed local checkout,
Cargo registry, or rustup source paths. Archive verification is required before
checksums are written.

Installed-toolchain certification lives in `verification/areas/sysroot_release`.
The merge-safe suite checks a real packaged archive from outside the repository,
installed sysroot JSON, installed migrated-stdlib emit, installed LSP lifecycle,
and release artifact path leakage. Nightly/release also run the long suite with
broad installed stdlib check/emit, a real installed `sifr build`, the built
binary, and offline/frozen Cargo checks for the generated project.

## Immutable Version Installer Generation

Generate the immutable installer from the verified artifact directory:

```bash
scripts/distribution/generate_version_installer.sh \
  --version 0.1.0-beta.1 \
  --artifact-dir target/preview-artifacts/0.1.0-beta.1 \
  --out target/preview-artifacts/0.1.0-beta.1/sifr-installer-0.1.0-beta.1
```

The generated installer embeds:

- the exact app version,
- target-to-archive mapping,
- SHA-256 checksums for every target archive,
- the default GitHub Release asset base URL,
- platform detection for the preview-release pipeline targets,
- checksum validation before extraction or replacement,
- archive path/link validation before extraction,
- staged binary and sysroot replacement under the install lock after validation,
- schema-versioned install receipt writing through a temporary file and atomic rename,
- update locking at `<install_dir>/.sifr-update.lock` before binary or receipt mutation,
- shell profile wiring through `~/.sifr/env`, unless `SIFR_NO_MODIFY_PATH=1`
  or `--no-modify-path` is used.

The generated installer honors `SIFR_ARTIFACT_BASE_URL`, `SIFR_TARGET`,
`SIFR_INSTALL_DIR`, `SIFR_SYSROOT_INSTALL_DIR`, and `SIFR_NO_MODIFY_PATH` for
local validation. `SIFR_INSTALL_DIR` remains the binary directory; when it ends
in `/bin`, the default sysroot root is its parent. Otherwise the binary
directory itself is the flat sysroot root for compatibility with older custom
install examples.

## Self-Update Receipt Rules

Official standalone installers write a schema-versioned `install.json` receipt:

```json
{
  "schema_version": 2,
  "name": "sifr",
  "version": "0.1.0-beta.2",
  "channel": "beta",
  "target": "aarch64-apple-darwin",
  "install_dir": "/Users/example/.sifr/bin",
  "binary_path": "/Users/example/.sifr/bin/sifr",
  "sysroot_path": "/Users/example/.sifr",
  "sysroot_schema_version": 1,
  "sysroot_sifr_version": "0.1.0-beta.2",
  "sysroot_target_triple": "aarch64-apple-darwin",
  "sysroot_content_sha256": "<sha256-tree>",
  "artifact": "sifr-0.1.0-beta.2-aarch64-apple-darwin.tar.gz",
  "modify_path": true
}
```

The authoritative field enumeration lives at `verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json`. Receipts must use `schema_version: 2`, include every listed field, and reject unknown fields. Pre-schema, partial, malformed, or mismatched receipts are treated as unmanaged installs by `sifr self update`; the diagnostic tells users to re-run `curl -LsSf https://sifr.sh/install | sh` if they want standalone self-update management.

`channel` is derived from the installer version prerelease label. `modify_path` records the actual installer request, including `SIFR_NO_MODIFY_PATH=1` and `--no-modify-path`. `binary_path` and `sysroot_path` record canonical installed paths when the platform can resolve them. `sifr self update` validates that `binary_path` is paired with either `sysroot_path/bin/sifr` for toolchain-root installs or `sysroot_path/sifr` for legacy flat custom installs, and delegates to the immutable installer with both paths under the same install lock.

## Self-Update TLS And Delegation Policy

The Rust CLI self-update path resolves a target immutable installer and delegates installation to that installer. It must not download release archives directly, parse dispatcher scripts for versions, bypass checksum validation, or accept executable URLs from metadata or receipts.

The public command surface is:

```bash
sifr self version [--short] [--format text|json]
sifr self update [--channel alpha|beta] [--version <preview-version>] [--dry-run] [--format text|json] [--force]
```

The default update channel is the receipt channel. `--dry-run` performs no
mutation and does not acquire the install lock. Reinstalls, downgrades, and
channel switches require `--force`; ordinary newer-version updates on the same
channel do not.

Production installer downloads use normal TLS certificate verification. Test-only install-base overrides may be compiled or configured for fixtures; production runtime environment variables must not replace the trusted installer URL base.

Before invoking an immutable installer, `sifr self update` acquires `<install_dir>/.sifr-update.lock`, passes receipt-derived install environment, and marks the internal handoff with `SIFR_INSTALL_LOCK_HELD=1`. Generated immutable installers still acquire the same lock for manual runs, but they do not reacquire or release it when that internal handoff marker is present.

## Installer artifact validation Validation

Run artifact and installer validation with:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area distribution_release --suite full
```

The installer artifact-specific checks are:

```bash
verification/areas/distribution_release/cases/artifact_generated_installer_all_preview_targets.sh
verification/areas/distribution_release/cases/artifact_sha256_validated.sh
verification/areas/distribution_release/cases/install_matching_target_artifact.sh
verification/areas/distribution_release/cases/channel_dispatcher_points_to_generated_installer.sh
verification/areas/distribution_release/cases/artifact_missing_target_rejected.sh
verification/areas/distribution_release/cases/artifact_checksum_mismatch_rejected.sh
verification/areas/distribution_release/cases/artifact_target_mismatch_rejected.sh
verification/areas/distribution_release/cases/stable_entrypoints_unchanged_by_preview_release.sh
```

The self-update metadata drift checks validate that the GitHub-bound `channels.json` is well formed and that website dispatchers are GitHub-backed bootstrap scripts without website-hosted metadata or version installers:

```bash
verification/areas/distribution_release/tools/validate_self_update_metadata.sh \
  --install-root <install-root> \
  --channels-file <work-dir>/channels.json
verification/areas/distribution_release/cases/channel_metadata_installer_agreement.sh
verification/areas/distribution_release/cases/channel_metadata_dispatcher_drift_rejected.sh
verification/areas/distribution_release/cases/channel_metadata_installer_drift_rejected.sh
verification/areas/distribution_release/cases/channel_metadata_stable_rejected.sh
```

Run the validator after every `create_new_version.sh --real-run` and before pushing/deploying the site repository. The real-run command invokes it automatically before GitHub publication.

## Preview Release Command

The `/create-new-version` workflow is backed by:

```bash
scripts/distribution/create_new_version.sh
```

Dry-run example:

```bash
scripts/distribution/create_new_version.sh \
  --channel beta \
  --version 0.1.0-beta.2 \
  --dry-run
```

Real-run example:

```bash
scripts/distribution/create_new_version.sh \
  --channel beta \
  --version 0.1.0-beta.2 \
  --real-run \
  --artifact-dir target/preview-artifacts/0.1.0-beta.2 \
  --mutation-mode local
```

Dry-run validates inputs, resolves the base commit, computes every target artifact name, detects site dispatcher drift, confirms stable entrypoints remain absent, and prints the exact GitHub Release, channel metadata, and site mutations.

Real-run reuses the same plan SHA-256, verifies or builds all target artifacts, generates the immutable GitHub installer asset, regenerates GitHub-backed website bootstrap scripts and evidence-only GitHub channel metadata from one plan, validates metadata/dispatcher agreement, writes a release checklist, and writes a recovery note. It does not publish GitHub assets; `preview-release.yml` is the only authoritative GitHub-publish path for version releases and the shared `channels` release asset.

The GitHub Actions preview-release workflow serializes runs with the `preview-release-channels` concurrency group because channel publication is a read-modify-write operation over the shared `channels` release asset.
Do not run workstation GitHub publication while `preview-release.yml` is publishing. Use `scripts/distribution/trigger_preview_release.sh` to dispatch GitHub releases.
The workflow publishes the version release before updating `channels.json`; if
the final channel update fails, retry the workflow after correcting the failed
verification. Version-pinned installs may see the already-published release
asset before the moving channel pointer advances.

The Cursor command wrapper lives at `.cursor/commands/create-new-version.md`.

## Preview lifecycle validation Validation

The preview lifecycle-specific checks are:

```bash
verification/areas/distribution_release/cases/create_new_version_alpha_dry_run.sh
verification/areas/distribution_release/cases/create_new_version_beta_dry_run.sh
verification/areas/distribution_release/cases/create_new_version_real_run_plan_reuse.sh
verification/areas/distribution_release/cases/create_new_version_release_checklist.sh
verification/areas/distribution_release/cases/create_new_version_attribution_checklist.sh
verification/areas/distribution_release/cases/create_new_version_stable_rejected.sh
verification/areas/distribution_release/cases/create_new_version_bad_semver_rejected.sh
verification/areas/distribution_release/cases/create_new_version_missing_artifact_rejected.sh
verification/areas/distribution_release/cases/create_new_version_site_dispatcher_drift_rejected.sh
```
