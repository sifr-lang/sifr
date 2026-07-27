# Sifr Distribution Pipeline

Status: canonical release-governance epoch; stable qualification, fresh
installation, and self-update behavior are implemented. Public stable
activation remains gated by protected GA publication.

## Installer Model

Sifr preview distribution uses generated shell installer entrypoints with the same high-level shape as Astral's `uv` installer: a channel dispatcher selects an immutable version installer, and the immutable installer owns platform/artifact selection and checksum verification.

preview-release pipeline does not copy or adapt code from `astral-sh/uv`. The current implementation is Sifr-owned shell generation in `scripts/distribution/`, so the uv MIT attribution checklist is recorded as not applicable until copied or adapted uv code is introduced.

## Site Layout

Static site files live under the site repository at `<site-repo>/apps/sifr-site/public/install/`.

The filesystem layout is:

```text
public/install/
  index
  stable
  alpha
  beta
```

`index` is generated with stable as its canonical default and is served at
`https://sifr.sh/install`; the other files are served at the matching nested
paths. Before GA activation, the paired preview deployment explicitly retains
beta as the live `index` default so stable is never exposed before the governed
index becomes active. All four scripts fetch GitHub `channels.json`, resolve
the requested channel or exact active version, and download the immutable
`sifr-installer-<version>` from the matching GitHub release.

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

- `/install` defaults to stable in the canonical generated dispatcher.
- `/install/stable`, `/install/alpha`, and `/install/beta` select the matching
  governed channel.
- `--channel alpha|beta|stable` and `SIFR_CHANNEL=alpha|beta|stable` select a
  moving channel.
- `--version` accepts stable SemVer or an alpha/beta prerelease only when the
  exact version is an active governed release.
- Stable resolution requires `ga_status: active`; preview metadata contains
  exactly alpha and beta.
- `rc`, legacy metadata, version-negotiated metadata, dual-format metadata,
  and withdrawn or unlisted versions fail before installer execution.
- Conflicting `SIFR_CHANNEL`, `--channel`, and `--version` inputs are rejected.
- Metadata and installer URLs are generator-owned constants, not runtime
  environment overrides.
- The dispatcher verifies the governed installer SHA-256 before execution and
  preserves the delegated installer's exit status.

The dispatcher never resolves artifacts itself and never compiles from source.

Generate self-update channel metadata with:

```bash
scripts/distribution/generate_channel_metadata.sh \
  --out <work-dir>/channels.json \
  --generation <next-generation> \
  --alpha-release <work-dir>/alpha-release-record.json \
  --beta-release <work-dir>/beta-release-record.json
```

The generated `channels.json` shape is:

```json
{
  "schema_version": 2,
  "generation": 1,
  "ga_status": "preview",
  "channels": {
    "alpha": "0.1.0-alpha.1",
    "beta": "0.1.0-beta.1"
  },
  "releases": {
    "0.1.0-alpha.1": {
      "channel": "alpha",
      "status": "active",
      "source_commit": "<40-character-commit>",
      "installer_sha256": "<sha256>",
      "targets": "<the exact four governed target digest records>"
    },
    "0.1.0-beta.1": {
      "channel": "beta",
      "status": "active",
      "source_commit": "<40-character-commit>",
      "installer_sha256": "<sha256>",
      "targets": "<the exact four governed target digest records>"
    }
  }
}
```

The governed index contains data only and never executable URLs. Trusted
dispatchers and the Rust CLI derive immutable GitHub URLs from repository
constants after validating the selected active release record. Schema v1 is
discarded state: there is no bootstrap converter, fallback reader, migration,
or dual publication. Preview publication requires an existing canonical v2
index and advances it only with the expected generation and digest.

## Release-index and dispatcher validation

Run dispatcher validation with:

```bash
verification/areas/distribution_release/cases/install_default_stable_dispatcher.sh
verification/areas/distribution_release/cases/install_alpha_dispatcher.sh
verification/areas/distribution_release/cases/install_version_pin_dispatcher.sh
verification/areas/distribution_release/cases/install_stable_channel.sh
verification/areas/distribution_release/cases/install_invalid_channel_rejected.sh
verification/areas/distribution_release/cases/install_conflicting_channel_and_version_rejected.sh
verification/areas/distribution_release/cases/install_stable_version_pin.sh
verification/areas/distribution_release/cases/install_stable_requires_active_metadata.sh
verification/areas/distribution_release/cases/install_withdrawn_stable_rejected.sh
verification/areas/distribution_release/cases/install_installer_checksum_mismatch_rejected.sh
verification/areas/distribution_release/cases/install_metadata_url_injection_ignored.sh
verification/areas/distribution_release/cases/install_legacy_metadata_shapes_rejected.sh
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

Stable builders establish these public compatibility floors:

- macOS 15.0 for `aarch64-apple-darwin` and `x86_64-apple-darwin`.
- glibc 2.39 for `aarch64-unknown-linux-gnu` and
  `x86_64-unknown-linux-gnu`.

## Stable Candidate Qualification

`.github/workflows/release-qualification.yml` is the build/upload-only stable
candidate workflow. It accepts only an exact lowercase 40-hex source commit and
an exact stable SemVer plus `rollback_version` (`none` for first GA, otherwise
one exact stable SemVer), checks out that commit plus recursive submodules, and
uses only `contents: read` and `actions: read`. It has no release, package,
Marketplace, site, protected-environment, or metadata-mutation authority.

The workflow uses the governed four-runner matrix, builds with locked
dependencies, packages through the same release artifact builder, installs and
smokes each matching-host toolchain, and packages the recorded
`editor_integrations/vscode` checkout without publishing it. Every upload uses
the name
`sifr-stable-candidate-<version>-<source-sha>-<target-or-kind>`,
`overwrite: false`, and a 30-day retention period.

Editor qualification records a `marketplace_publish_plan` with status
`planned`; it does not claim that a credentialed dry run occurred. The
protected stable-publication workflow introduced at activation consumes that
exact VSIX and realizes the dry-run/publication evidence without rebuilding.

The final collector reads the current workflow run's artifact API, verifies
source/run attribution, and writes canonical
`qualification-artifact-index.json`. The index binds recursive submodules,
every workflow artifact id and upload name, exact per-artifact expiry, file
name, size, SHA-256, complete target coverage, aggregate installer/checksums,
and VSIX evidence.

The local `plan-stable-release` command in
`scripts/distribution/release_governance.py` is non-mutating. It requires a
clean checkout at the exact source SHA, an unexpired qualification index, the
canonical release-profile report for that same checkout, transported files,
Rust stable-claim evidence, documentation evidence, release notes, and the
current preview or active governed index. Its output must be a fresh path
outside the checkout. It hashes and cross-checks those exact bytes before
writing one canonical `stable-release-plan.json`. The planner also regenerates
the immutable installer with
`scripts/distribution/generate_version_installer.sh` from the pinned
`source_commit` and the transported per-target archives and checksums, then
requires byte-for-byte equality with the transported installer. The installer
digest is therefore bound to the governed producer rather than to textual
self-attestation inside the shell script.

## Artifact Packaging

Package artifacts with an existing binary fixture:

```bash
scripts/distribution/build_release_artifacts.sh \
  --version 0.1.0-beta.1 \
  --output-dir target/preview-artifacts/0.1.0-beta.1 \
  --binary target/release/sifr
```

For production target builds, use:

```bash
scripts/distribution/build_release_artifacts.sh \
  --version 0.1.0-beta.1 \
  --output-dir target/preview-artifacts/0.1.0-beta.1 \
  --cargo-build
```

The production path runs
`cargo build --locked --release -p sifr --target <target>` for every governed
release target and fails if any target cannot be built. It
accepts stable SemVer for qualification without creating a separate builder
and does not fall back to another binary or target.
Production builds remap repository, sysroot, Cargo-home, and rustup-home path
prefixes before packaging so release binaries do not embed local checkout,
Cargo registry, or rustup source paths. Archive verification is required before
checksums are written.

Run the capability demo from a clean checkout:

```bash
demos/stable_candidate_qualification_demo.sh
```

It builds and qualifies the real host artifact, installs it in isolation, runs
`sifr --version`, `sifr check`, and `sifr self version`, combines that evidence
with fixture-backed remote-target, documentation, Rust-claim, site, and VSIX
inputs, and materializes a schema-complete unapproved plan outside the source
checkout.

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

`channel` is derived from the installer version: exact stable SemVer records
`stable`, while prereleases use their label. Read-only receipt discovery,
`sifr self version`, and self-update resolution accept stable receipts.
`modify_path` records the actual installer request, including
`SIFR_NO_MODIFY_PATH=1` and `--no-modify-path`. `binary_path` and
`sysroot_path` record canonical installed paths when the platform can resolve
them. `sifr self update` validates that `binary_path` is paired with either
`sysroot_path/bin/sifr` for toolchain-root installs or `sysroot_path/sifr` for
legacy flat custom installs, and delegates to the immutable installer with both
paths under the same install lock.

## Self-Update TLS And Delegation Policy

The Rust CLI self-update path resolves a target immutable installer and delegates installation to that installer. It must not download release archives directly, parse dispatcher scripts for versions, bypass checksum validation, or accept executable URLs from metadata or receipts.

The public command surface is:

```bash
sifr self version [--short] [--format text|json]
sifr self update [--channel alpha|beta|stable] [--version <release-version>] [--dry-run] [--format text|json] [--force]
```

The default update channel is the receipt channel. Receipt eligibility is
validated before network access. Every channel and exact-version request then
resolves through canonical schema-v2 metadata, and the downloaded installer
must match its governed SHA-256 before execution. `--dry-run` performs no
mutation and does not acquire the install lock. Reinstalls, downgrades, and
alpha/beta/stable channel switches require `--force`; ordinary newer-version
updates on the receipt channel do not.

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
verification/areas/distribution_release/cases/stable_entrypoints_generated.sh
```

The self-update metadata drift checks validate that the GitHub-bound `channels.json` is well formed and that website dispatchers are GitHub-backed bootstrap scripts without website-hosted metadata or version installers:

```bash
verification/areas/distribution_release/tools/validate_self_update_metadata.sh \
  --install-root <install-root> \
  --channels-file <work-dir>/channels.json
verification/areas/distribution_release/cases/channel_metadata_installer_agreement.sh
verification/areas/distribution_release/cases/channel_metadata_dispatcher_drift_rejected.sh
verification/areas/distribution_release/cases/channel_metadata_installer_drift_rejected.sh
verification/areas/distribution_release/cases/channel_metadata_stable_active.sh
```

Run the validator against publication fixtures and generated site dispatchers.
Local tooling has no real-run or mutation mode.

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
  --dry-run \
  --site-repo <clean-sifr-website-checkout> \
  --release-index <canonical-channels-v2.json>
```

The planner validates the exact Sifr commit, clean exact website base commit,
production dispatcher URLs, the GA-aware site source shape (beta-default
`index` with no required stable entrypoint for a preview index; stable-default
`index` plus the stable entrypoint for an active index), absence of website
metadata shadow state, and current canonical index identity. It prints the
proposed channel mutation, paired site default, and write-once/replace-only
policies without writing files. Artifact, real-run, work-directory, and
mutation options are rejected.

`.github/workflows/preview-release.yml` is a build-only caller.
`.github/workflows/release-publication.yml` is the sole reusable mutation
authority and holds the `sifr-release-index` concurrency lease. It accepts only
alpha and beta until protected GA activation. It rejects any existing version
release, uploads immutable version assets without `--clobber`, downloads and
byte-verifies them, allocates one greater than the maximum current/snapshot
generation, uploads `channels-generation-<N>.json` write-once, and uses
`--clobber` only for the canonical `channels.json` asset.

After index replacement, the same leased workflow dispatches the pinned
`sifr-lang/sifr-website` `release-site.yml` through the immutable
`sifr-release-site-stable-distribution` dispatch tag, which must resolve to the exact
protected-main commit pinned in the payload. Active site repository ruleset
`19791667` forbids updating or deleting that exact tag and grants no bypass;
the caller verifies the ruleset's active exact-name update/deletion guards and
the attested no-bypass ruleset revision, plus the tag target, before release
mutation and again immediately before dispatch. The stable-distribution caller
pins the governed
workflow introduced by
[sifr-website PR #14](https://github.com/sifr-lang/sifr-website/pull/14) and its
GA-aware default-channel binding from
[sifr-website PR #15](https://github.com/sifr-lang/sifr-website/pull/15), merged
at `07d88cc3c24707e386c5ad73fb0875c06ffd598f`.
The protected cross-repository token is limited to that repository's Actions
operations. The site run checks exact Sifr/site commits, regenerates the four
dispatchers, validates dispatcher and canonical schema-v2
`site-publication-facts.json` digests,
re-fetches the exact governed generation immediately before deploy, requires
the default dispatcher to be beta for a preview index and stable for an active
index, deploys through Wrangler, and verifies the public bytes. The main
workflow polls the exact attempt/head/title for at most 20 minutes and requests
cancellation on timeout. The site workflow never writes release metadata.

The Cursor command wrapper lives at `.cursor/commands/create-new-version.md`.

## Preview lifecycle validation Validation

The preview lifecycle-specific checks are:

```bash
verification/areas/distribution_release/cases/create_new_version_alpha_dry_run.sh
verification/areas/distribution_release/cases/create_new_version_beta_dry_run.sh
verification/areas/distribution_release/cases/create_new_version_plan_is_read_only.sh
verification/areas/distribution_release/cases/create_new_version_local_mutation_rejected.sh
verification/areas/distribution_release/cases/create_new_version_artifact_mode_rejected.sh
verification/areas/distribution_release/cases/create_new_version_dirty_site_rejected.sh
verification/areas/distribution_release/cases/create_new_version_stable_rejected.sh
verification/areas/distribution_release/cases/create_new_version_bad_semver_rejected.sh
verification/areas/distribution_release/cases/create_new_version_site_dispatcher_drift_rejected.sh
verification/areas/distribution_release/cases/preview_release_workflow_yaml_parses.sh
verification/areas/distribution_release/cases/site_publication_facts_generated.sh
verification/areas/distribution_release/cases/site_release_workflow_contract.sh
```

Run the capability demo with:

```bash
demos/stable_self_update_demo.sh
```

It performs a forced beta-to-stable handoff and an ordinary
stable-to-stable update through immutable mock installers, proves receipt and
sysroot version movement together, shows the stable no-op plan, and verifies
that the public preview workflow still has no stable mutation input.

## Stable Incident Recovery

Rollback and incident roll-forward planning extend the canonical schema-v2
release index; they do not introduce a second publication workflow.
`plan-incident-index` in `scripts/distribution/release_governance.py` consumes
canonical request, affected-plan, successor/target-plan, and live-index bytes,
plus the expected generation/digest and a fresh generation number. Rollback is
accepted only when the affected `normal` plan names that exact retained active
predecessor and plan digest. Incident roll-forward requires a qualified
successor plan that binds the request and affected-plan digests and records
`rollback_target: none`. Both operations withdraw the affected stable and move
the stable channel atomically in one validated generation.

Incident requests are prepared outside the repository with exact affected
plan, withdrawal-evidence, and—for rollback—target-plan digests. The
evidence-commit validator permits exactly
`plans/releases/incidents/<incident-id>/stable-incident-request.json` and
`withdrawal-evidence.txt` as added files. It rejects source edits, renames,
deletions, unrelated files, noncanonical request bytes, directory/id drift, and
evidence-digest drift.

The credential-free local harness is:

```bash
scripts/distribution/run_incident_fixture.py run \
  --fixture-root <dedicated-system-temp-directory> \
  --live-index <temporary-channels.json> \
  --governance-release <temporary-governance-assets> \
  --release-assets <temporary-immutable-assets> \
  --marketplace-stub <temporary-marketplace.json> \
  --extension-metadata <temporary-extension-metadata.json> \
  --site-repo <temporary-non-deploying-site-repository> \
  --request <fixture-request> \
  --affected-plan <fixture-affected-plan> \
  --successor-plan <fixture-target-or-successor-plan> \
  --mode initial \
  --approver <fixture-reviewer>
```

It accepts only explicit temporary filesystem fixtures, rejects production
credentials, contains no network or production adapter, and shares one
filesystem metadata lease with preview/stable submission preflight. It
publishes write-once request and generation evidence, burns a generation after
reservation failure, atomically replaces only the local index, reconciles a
non-deploying site fixture, verifies the extension/Marketplace range for
rollback, and emits a schema-v2 incident sign-off. Resume either allocates
after every retained snapshot or verifies the already-realized
generation/digest and retries site reconciliation without another index
mutation.

The canonical ownership, acknowledgement, communication, retry, retention, and
closure policy is in
[`stable_incident_response.md`](./stable_incident_response.md). Production
workflow inputs, write permissions, Marketplace calls, and site dispatch remain
absent until protected publication wiring.

Run the incident-specific suite and capability demo with:

```bash
uv run --project verification --locked python -m sifr_verify areas run \
  --area distribution_release --suite incident-governance
demos/stable_incident_recovery_demo.sh
```
