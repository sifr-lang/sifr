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
