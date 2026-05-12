# Sifr Preview Distribution Pipeline

Status: Phase 33 implementation contract.

## Installer Model

Sifr preview distribution uses generated shell installer entrypoints with the same high-level shape as Astral's `uv` installer: a channel dispatcher selects an immutable version installer, and the immutable installer owns platform/artifact selection and checksum verification.

Phase 33 does not copy or adapt code from `astral-sh/uv`. The current implementation is Sifr-owned shell generation in `scripts/distribution/`, so the uv MIT attribution checklist is recorded as not applicable until copied or adapted uv code is introduced.

## Site Layout

Static site files live in `/Users/yaseralnajjar/work/sifr/sifr-blog-website/apps/sifr-site/public/install/`.

The filesystem layout is:

```text
public/install/
  index
  alpha
  beta
  versions/
    <version>
```

`index` is the default beta dispatcher. The deployment must serve it at `https://sifr.sh/install`; `alpha` and `beta` are served at `https://sifr.sh/install/alpha` and `https://sifr.sh/install/beta`. Immutable generated installers are served from `https://sifr.sh/install/versions/<version>`.

This directory layout avoids the impossible static-file shape where `public/install` is both an executable file and a directory for nested channel paths.

## Channel Dispatcher Contract

Generate dispatchers with:

```bash
scripts/distribution/generate_dispatchers.sh \
  --install-root /Users/yaseralnajjar/work/sifr/sifr-blog-website/apps/sifr-site/public/install \
  --alpha-version 0.1.0-alpha.1 \
  --beta-version 0.1.0-beta.1
```

Dispatcher behavior:

- `/install` defaults to the configured beta preview.
- `/install/alpha` and `/install/beta` select the corresponding preview channel.
- `--channel alpha|beta` and `SIFR_CHANNEL=alpha|beta` select a moving preview channel.
- `--version <semver-prerelease>` selects `versions/<version>` directly.
- `stable` and stable-looking versions such as `1.0.0` or `0.1.0` are rejected before download.
- Conflicting `SIFR_CHANNEL`, `--channel`, and `--version` inputs are rejected.
- The dispatcher downloads exactly one immutable generated installer and preserves its exit status.

The dispatcher never resolves artifacts itself and never compiles from source.

## Phase 33.1 Validation

Milestone 33.1 uses mocked immutable generated installers until milestone 33.2 adds checksum-verified artifact installers.

Run dispatcher validation with:

```bash
verification/distribution/install_default_beta_dispatcher.sh
verification/distribution/install_alpha_dispatcher.sh
verification/distribution/install_version_pin_dispatcher.sh
verification/distribution/install_stable_channel_gated.sh
verification/distribution/install_invalid_channel_rejected.sh
verification/distribution/install_conflicting_channel_and_version_rejected.sh
verification/distribution/install_stable_version_pin_rejected.sh
verification/distribution/install_missing_generated_installer_rejected.sh
verification/distribution/install_dispatcher_malformed_config_rejected.sh
```

## Artifact Format

Preview artifacts are gzip-compressed tar archives published as GitHub Release assets in `sifr-lang/sifr`.

Each target has:

```text
sifr-<version>-<target>.tar.gz
sifr-<version>-<target>.tar.gz.sha256
```

The archive contains exactly one executable at the archive root: `sifr`.

The Phase 33 target set is:

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

The production path runs `cargo build --release -p sifr --target <target>` for every Phase 33 target and fails if any target cannot be built. It does not fall back to another binary or another target.

## Immutable Version Installer Generation

Generate the immutable installer from the verified artifact directory:

```bash
scripts/distribution/generate_version_installer.sh \
  --version 0.1.0-beta.1 \
  --artifact-dir target/preview-artifacts/0.1.0-beta.1 \
  --out /Users/yaseralnajjar/work/sifr/sifr-blog-website/apps/sifr-site/public/install/versions/0.1.0-beta.1
```

The generated installer embeds:

- the exact app version,
- target-to-archive mapping,
- SHA-256 checksums for every target archive,
- the default GitHub Release asset base URL,
- platform detection for the Phase 33 targets,
- checksum validation before extraction or replacement,
- atomic replacement of the installed `sifr` binary after validation.

The generated installer honors `SIFR_ARTIFACT_BASE_URL`, `SIFR_TARGET`, and `SIFR_INSTALL_DIR` for local validation.

## Phase 33.2 Validation

Run artifact and installer validation with:

```bash
scripts/run_distribution_validation.sh
```

The milestone 33.2-specific checks are:

```bash
verification/distribution/artifact_generated_installer_all_preview_targets.sh
verification/distribution/artifact_sha256_validated.sh
verification/distribution/install_matching_target_artifact.sh
verification/distribution/channel_dispatcher_points_to_generated_installer.sh
verification/distribution/artifact_missing_target_rejected.sh
verification/distribution/artifact_checksum_mismatch_rejected.sh
verification/distribution/artifact_target_mismatch_rejected.sh
verification/distribution/stable_entrypoints_unchanged_by_preview_release.sh
```

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
  --mutation-mode github
```

Dry-run validates inputs, resolves the base commit, computes every target artifact name, detects site dispatcher drift, confirms stable entrypoints remain absent, and prints the exact GitHub Release and site mutations.

Real-run reuses the same plan SHA-256, verifies or builds all target artifacts, generates the immutable version installer, regenerates channel dispatchers, writes a release checklist, writes a recovery note, and publishes GitHub Release assets when `--mutation-mode github` is selected. Validation uses `--mutation-mode local` to exercise the same file mutations without publishing assets.

The Cursor command wrapper lives at `.cursor/commands/create-new-version.md`.

## Phase 33.3 Validation

The milestone 33.3-specific checks are:

```bash
verification/distribution/create_new_version_alpha_dry_run.sh
verification/distribution/create_new_version_beta_dry_run.sh
verification/distribution/create_new_version_real_run_plan_reuse.sh
verification/distribution/create_new_version_release_checklist.sh
verification/distribution/create_new_version_attribution_checklist.sh
verification/distribution/create_new_version_stable_rejected.sh
verification/distribution/create_new_version_bad_semver_rejected.sh
verification/distribution/create_new_version_missing_artifact_rejected.sh
verification/distribution/create_new_version_site_dispatcher_drift_rejected.sh
```
