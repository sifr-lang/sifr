# Phase 33 Preview Distribution Execution Checklist

Status: completed

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
- [x] PR merged.

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

PRs:

- Compiler repo: https://github.com/sifr-lang/sifr/pull/2106
- Site repo: https://github.com/sifr-lang/sifr-website/pull/4

## Milestone 33.3: Agentic Preview Release Command

- [x] `.cursor/commands/create-new-version.md` added.
- [x] Dry-run planner.
- [x] Authorized real-run workflow.
- [x] Release checklist and recovery note generation.
- [x] Stable release attempts rejected before mutation.
- [x] Reviewer satisfied.
- [x] PR merged.

Validation evidence:

- `verification/distribution/create_new_version_alpha_dry_run.sh`
- `verification/distribution/create_new_version_beta_dry_run.sh`
- `verification/distribution/create_new_version_real_run_plan_reuse.sh`
- `verification/distribution/create_new_version_release_checklist.sh`
- `verification/distribution/create_new_version_attribution_checklist.sh`
- `verification/distribution/create_new_version_stable_rejected.sh`
- `verification/distribution/create_new_version_bad_semver_rejected.sh`
- `verification/distribution/create_new_version_missing_artifact_rejected.sh`
- `verification/distribution/create_new_version_site_dispatcher_drift_rejected.sh`
- `scripts/run_distribution_validation.sh`

PR:

- Compiler repo: https://github.com/sifr-lang/sifr/pull/2107

## Phase Exit Gate

- [x] Public beta installer resolves through `/install`.
- [x] Public alpha installer resolves through `/install/alpha`.
- [x] Explicit preview version pin installs exact immutable version.
- [x] Artifact downloads are SHA-256 validated before install.
- [x] `/create-new-version` dry-run and real-run flows are repeatable.
- [x] Stable channel and stable-looking version pins remain impossible.
- [x] Site deployment path has been exercised.
- [x] Phase 27 non-regression contract remains green.

Closure evidence:

- Merged milestone PRs: https://github.com/sifr-lang/sifr/pull/2105, https://github.com/sifr-lang/sifr/pull/2106, https://github.com/sifr-lang/sifr/pull/2107
- Merged site PRs: https://github.com/sifr-lang/sifr-website/pull/3, https://github.com/sifr-lang/sifr-website/pull/4
- Closure PR: https://github.com/sifr-lang/sifr/pull/2108
- Corrective release publication workflow PR: https://github.com/sifr-lang/sifr/pull/2109
- Corrective submodule checkout PR: https://github.com/sifr-lang/sifr/pull/2110
- Published alpha release: https://github.com/sifr-lang/sifr/releases/tag/0.1.0-alpha.1
- Published beta release: https://github.com/sifr-lang/sifr/releases/tag/0.1.0-beta.1
- Release workflow runs: https://github.com/sifr-lang/sifr/actions/runs/25767509795, https://github.com/sifr-lang/sifr/actions/runs/25767509841
- Public release assets verified: each preview release has eight assets, covering four target archives and four `.sha256` checksum files.
- Milestone reviews: `reviews/phase-33-m33-1-generated-dispatchers-review-pass-2.md`, `reviews/phase-33-m33-2-artifact-installer-review-pass-1.md`, `reviews/phase-33-m33-3-create-new-version-review-pass-1.md`
- Final closure review: `reviews/phase-33-full-implementation-closure-review-pass-1.md`
- Corrective workflow review: `reviews/phase33-release-submodules-review.md`
- Corrective release publication closure review: `reviews/phase-33-release-publication-closure-review-pass-1.md`
- Distribution validation: `scripts/run_distribution_validation.sh`
- Site build: `npm run build:site` in `/Users/yaseralnajjar/work/sifr/sifr-blog-website`
- Site preview smoke: local Astro preview served `/install` and `/install/alpha` with HTTP 200 and generated shell scripts.
