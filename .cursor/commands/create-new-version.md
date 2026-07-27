# /create-new-version

Use this command to render and validate preview release plans. Local execution
has no mutation or artifact mode.

The implementation is `scripts/distribution/create_new_version.sh`.

## Required Inputs

- `--channel alpha|beta`
- `--version <semver-prerelease>`
- `--dry-run`
- `--site-repo <clean-sifr-website-checkout>`
- `--release-index <canonical-schema-v2-index>`

Optional input:

- `--base-ref <sha-or-branch>`

## Examples

Trigger the CI-native preview release workflow:

```bash
scripts/distribution/trigger_preview_release.sh \
  --channel beta \
  --version 0.1.0-beta.5
```

Dry-run:

```bash
scripts/distribution/create_new_version.sh \
  --channel alpha \
  --version 0.1.0-alpha.2 \
  --dry-run \
  --site-repo ../sifr-website \
  --release-index /path/to/channels.json
```

Use `trigger_preview_release.sh` for production preview releases when the release
artifacts should be built on native GitHub-hosted runners. The script dispatches
`.github/workflows/preview-release.yml`, which validates the preview inputs,
builds each target on the matching runner family, and delegates every mutation
to `.github/workflows/release-publication.yml`.

## Safety Rules

- Stable-looking versions are rejected before mutation.
- The version prerelease label must match the selected channel.
- Dry-run has no side effects.
- `--real-run`, `--mutation-mode`, `--artifact-dir`, and `--work-dir` are
  rejected.
- The plan pins exact Sifr, site, and governed-index identities.
- Preview publication accepts only alpha or beta; protected stable publication
  remains separate.
- Immutable version assets are write-once, generation snapshots reserve their
  number before index replacement, and only `channels.json` is replaceable.
