# /create-new-version

Use this command to plan or execute preview releases.

The implementation is `scripts/distribution/create_new_version.sh`.

## Required Inputs

- `--channel alpha|beta`
- `--version <semver-prerelease>`
- exactly one of `--dry-run` or `--real-run`

Optional input:

- `--base-ref <sha-or-branch>`

## Examples

Dry-run:

```bash
scripts/distribution/create_new_version.sh \
  --channel alpha \
  --version 0.1.0-alpha.2 \
  --dry-run
```

Real-run using local validation artifacts:

```bash
scripts/distribution/create_new_version.sh \
  --channel beta \
  --version 0.1.0-beta.2 \
  --real-run \
  --artifact-dir target/preview-artifacts/0.1.0-beta.2
```

Production real-runs must keep `--mutation-mode github` once release assets are ready to publish. Local validation uses `--mutation-mode local` to exercise the same plan and file mutations without publishing GitHub Release assets.

## Safety Rules

- Stable-looking versions are rejected before mutation.
- The version prerelease label must match the selected channel.
- Dry-run has no side effects.
- Real-run writes a plan, release checklist, and recovery note under `target/preview-release/<version>/`.
- Real-run regenerates immutable version installers and channel dispatchers from the dry-run plan.
- `stable` entrypoints and stable release metadata must remain unchanged by preview-release work.
