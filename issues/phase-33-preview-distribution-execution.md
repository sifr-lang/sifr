# Phase 33 Preview Distribution Execution Checklist

Status: in progress

Phase source: `internal_docs/phases/33_preview_distribution_and_release_automation.md`

## Milestone 33.1: Generated Installer and Channel Dispatchers

- [x] Sifr-owned generated dispatcher path selected.
- [x] uv-derived code status recorded: no copied or adapted uv code in milestone 33.1.
- [x] Public site dispatcher files generated under `apps/sifr-site/public/install/`.
- [x] Default beta, alpha, beta, and explicit preview version dispatch are deterministic.
- [x] Stable channel and stable-looking versions are rejected before download.
- [x] Invalid channel, conflicting inputs, missing immutable installer, and malformed dispatcher config are rejected.
- [x] Reviewer satisfied.
- [x] PR merged.

PRs:

- Compiler repo: https://github.com/sifr-lang/sifr/pull/2105
- Site repo: https://github.com/sifr-lang/sifr-website/pull/3

Validation evidence:

- `verification/distribution/install_default_beta_dispatcher.sh`
- `verification/distribution/install_alpha_dispatcher.sh`
- `verification/distribution/install_version_pin_dispatcher.sh`
- `verification/distribution/install_stable_channel_gated.sh`
- `verification/distribution/install_invalid_channel_rejected.sh`
- `verification/distribution/install_conflicting_channel_and_version_rejected.sh`
- `verification/distribution/install_stable_version_pin_rejected.sh`
- `verification/distribution/install_missing_generated_installer_rejected.sh`
- `verification/distribution/install_dispatcher_malformed_config_rejected.sh`

## Milestone 33.2: Artifact and Generated Installer Pipeline

- [x] Artifact build automation.
- [x] SHA-256 checksum generation and validation.
- [x] Immutable generated version installer.
- [x] Channel dispatchers point to generated installers.
- [x] Demo evidence recorded.
- [x] Reviewer satisfied.
- [ ] PR merged.

Validation evidence:

- `verification/distribution/artifact_generated_installer_all_preview_targets.sh`
- `verification/distribution/artifact_sha256_validated.sh`
- `verification/distribution/install_matching_target_artifact.sh`
- `verification/distribution/channel_dispatcher_points_to_generated_installer.sh`
- `verification/distribution/artifact_missing_target_rejected.sh`
- `verification/distribution/artifact_checksum_mismatch_rejected.sh`
- `verification/distribution/artifact_target_mismatch_rejected.sh`
- `verification/distribution/stable_entrypoints_unchanged_by_preview_release.sh`
- `scripts/run_distribution_validation.sh`

## Milestone 33.3: Agentic Preview Release Command

- [ ] `.cursor/commands/create-new-version.md` added.
- [ ] Dry-run planner.
- [ ] Authorized real-run workflow.
- [ ] Release checklist and recovery note generation.
- [ ] Stable release attempts rejected before mutation.
- [ ] Reviewer satisfied.
- [ ] PR merged.

## Phase Exit Gate

- [ ] Public beta installer resolves through `/install`.
- [ ] Public alpha installer resolves through `/install/alpha`.
- [ ] Explicit preview version pin installs exact immutable version.
- [ ] Artifact downloads are SHA-256 validated before install.
- [ ] `/create-new-version` dry-run and real-run flows are repeatable.
- [ ] Stable channel and stable-looking version pins remain impossible.
- [ ] Site deployment path has been exercised.
- [ ] Phase 27 non-regression contract remains green.
