# Phase 40: Stable Channel GA Promotion and Release Governance

## Objective
Promote stable channel only after reliability/parity/performance evidence is complete and governed.

Phase 40 owns the point where stable distribution becomes user-facing, including:

- stable release artifact eligibility,
- stable installer entrypoints and metadata,
- rollback and incident governance,
- formal release sign-off,
- stable-channel support in `sifr self update`.

Before this phase, preview self-update must keep `stable` and stable-looking version pins gated. Phase 40 is the phase that can lift that gate, but only after stable metadata, installer behavior, self-update behavior, rollback, and release sign-off are validated as one governance surface.

## Depends on
- Phase 39
- Phase 38
- Phase 34 generated-code quality gates pass before stable artifacts are eligible for GA promotion.
- Ad Hoc Sifr Self Update (`../issues/archive/ad-hoc-sifr-self-update.md`) preview substrate, if it has landed before Phase 40 starts. If that ad hoc phase has not landed, Phase 40 must first implement the same receipt-checked, version-metadata-only, immutable-installer-delegating self-update substrate before enabling stable self-update.

## Milestones

### milestone_40_1: Stable Promotion Policy
- Scope:
  - Define hard preconditions for `stable` promotion from preview channels.
  - Define the exact criteria for lifting stable gating in installer dispatchers, release metadata, and `sifr self update`.
  - Require stable self-update eligibility to use the same receipt validation and immutable-installer delegation model as preview self-update.
- Definition of done:
  - Promotion checklist is documented and mandatory.
  - Stable self-update cannot be enabled without passing the stable promotion checklist.

### milestone_40_2: Rollback and Incident Governance
- Scope:
  - Define rollback triggers, owner responsibilities, and communication protocol.
  - Define stable self-update rollback behavior as a governed downgrade: when rollback is approved, stable metadata points to the approved rollback version, `sifr self update` refuses to install that older stable version without `--force`, and `sifr self update --force` delegates to the immutable installer for the rollback target.
  - Define how stable channel metadata and installer entrypoints are reverted without producing binary/receipt mismatches for already-installed users.
- Definition of done:
  - Rollback path is tested and documented.
  - Rollback validation covers both fresh stable installs and `sifr self update` from an affected stable release.

### milestone_40_3: Release Sign-off Workflow
- Scope:
  - Enforce formal release sign-off and artifact provenance checks.
  - Include stable self-update metadata, immutable installer scripts, and stable dispatcher changes in the sign-off artifact set.
  - Require sign-off to verify that the stable self-update target version, GitHub release tag, immutable installer `APP_VERSION`, checksums, and public docs all agree.
- Definition of done:
  - Stable releases require auditable approvals and pass governance gates.

### milestone_40_4: Stable Installer And Self-Update Activation
- Scope:
  - Enable the public `stable` installer channel only through the governed release plan.
  - Generate stable channel metadata from the same plan as dispatchers and immutable installers.
  - Update `sifr self update` to accept stable channel metadata and stable-looking version pins only after the stable promotion gate is satisfied.
  - Keep the ad hoc self-update receipt schema, `self version` JSON schema, and `channels.json` schema at `schema_version: 1`; stable activation changes the governed allowlist and accepted version classes, not field shapes.
  - Preserve preview safety rules: no installer URLs from metadata, no artifact extraction in Rust, receipt eligibility before network access, installer URLs derived from trusted constants, and immutable installer delegation for checksum/extraction/replacement.
  - Add stable self-update validation covering preview-to-stable, stable-to-stable, stable no-op, forced stable downgrade or rollback, stale metadata rejection, and mismatched receipt rejection.
  - Update public and internal docs to distinguish preview update behavior from stable update behavior.
- Definition of done:
  - `sifr self update` can update an official standalone stable install to the current governed stable release.
  - `sifr self update --channel stable` and `sifr self update --version <stable-version>` work only for governed stable releases.
  - Stable metadata, dispatcher, immutable installer, GitHub release, and docs drift checks are part of local validation.
  - Stable activation tests prove schema-version `1` metadata and receipts accept `stable` only after the governed allowlist is updated.
  - Pre-GA stable metadata and unsigned or unapproved stable versions remain rejected.

## Quality Contract
- Entry criteria: Phase 39 is completed, Phase 38 is completed, and release-facing documentation is canonical.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Stable GA promotion is policy-driven, auditable, and reversible.
- Stable GA exit criteria include stable `sifr self update`; stable is not promoted until the standalone CLI update path can safely consume governed stable metadata.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_40_1` (Stable Promotion Policy): validation goals cover: Define hard preconditions for `stable` promotion from preview channels. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_1` must include negative-path validation proving stable self-update remains gated until promotion policy is satisfied.
  - `milestone_40_2` (Rollback and Incident Governance): validation goals cover: Define rollback triggers, owner responsibilities, and communication protocol. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_2` must include rollback validation through `sifr self update`, not only fresh installer dispatchers.
  - `milestone_40_3` (Release Sign-off Workflow): validation goals cover: Enforce formal release sign-off and artifact provenance checks. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_3` must prove stable self-update metadata, immutable installer scripts, release assets, and docs are included in the sign-off evidence.
  - `milestone_40_4` (Stable Installer And Self-Update Activation): validation goals cover stable channel metadata, stable installer dispatch, stable self-update, rollback/downgrade controls, stale metadata rejection, and receipt mismatch rejection.
  - Exit-gate evidence explicitly demonstrates: Stable GA promotion is policy-driven, auditable, reversible, and reachable through both fresh install and `sifr self update`.

## Exit Gate
- Stable GA promotion is policy-driven, auditable, and reversible.
- Stable installer entrypoints and stable `sifr self update` consume the same governed stable release plan and pass drift checks.
- Stable self-update preserves the ad hoc self-update safety model: receipt eligibility before network, version-only metadata, trusted derived installer URLs, immutable installer delegation, no Rust-side artifact replacement, and no stable update without sign-off.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
