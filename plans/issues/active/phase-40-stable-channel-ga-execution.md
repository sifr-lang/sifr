# Phase 40 Stable Channel GA Execution

## Status

In progress. This is the execution checklist for
[`plans/phases/40_stable_channel_ga_promotion_and_release_governance.md`](../../phases/40_stable_channel_ga_promotion_and_release_governance.md).
Milestones execute in order and each closes through a reviewed, locally
validated, merged PR.

## Frozen GA Decisions

- First stable version: `0.1.0`.
- Release owner: `release/distribution`.
- Incident owner: `release/distribution`.
- Protected GitHub environment: `stable-release`.
- Protected review policy: at least one non-initiating
  `release/distribution` reviewer; self-review is forbidden. Initial and resume
  attempts each require a fresh approval recorded in sign-off.
- Supported standalone targets:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-unknown-linux-gnu`
- Builder-derived minimum floors:
  - macOS 15.0, from the `macos-15` and `macos-15-intel` release builders.
  - glibc 2.39 on Linux, from the Ubuntu 24.04 release builders.
- Canonical site repository: `sifr-lang/sifr-blog-website`.
- Canonical release-index governance tag: `channels`.
- Canonical schema epoch: integer `2` only. No v1 reader, writer, fixture,
  migration, negotiation, or fallback survives the milestone-40.0 cutover.
- Stable-gate inventory:
  [`plans/releases/stable_gate_inventory.json`](../../releases/stable_gate_inventory.json).
  Every entry has an owner, current behavior, activation boundary, and final
  disposition; the distribution governance self-test rejects unowned or stale
  paths.

## Milestone Checklist

### milestone_40_0: Architecture and Gate Lock

- [x] Check in all Phase 40 JSON Schemas under
  `verification/areas/distribution_release/schemas/`.
- [x] Add schema-v2 generators and fail-closed validators for release index,
  stable plan/sign-off, qualification index, site facts, incident
  request/sign-off, and release-profile report.
- [x] Atomically replace channel metadata, install receipt, CLI version JSON,
  and self-update-plan JSON with schema-v2 alpha/beta/stable contracts while
  keeping stable selection unavailable.
- [x] Add fresh external release-report output and validation.
- [x] Add candidate/incident evidence-custody validation and release-profile
  selection.
- [x] Add documentation area ownership, structure suite, profile step, and
  selected-but-unrun/result self-tests.
- [x] Reserve Phase 40 ownership of the separately delivered Rust-interop
  stable-candidate registration without making it a milestone-40.0 blocker.
- [x] Check in and validate the owned stable-gate inventory.
- [x] Record positive/negative evidence, commands, review rounds, PR, and merge.

Artifacts mapped to the Phase 40 exit gate:

| Artifact | Milestone | Exit-gate role |
| --- | --- | --- |
| `channels.json` schema/generation history | 40.0, 40.5 | Governed channel and immutable generation history |
| `release-profile-report.json` | 40.0, 40.1 | Canonical local qualification evidence |
| `stable-release-plan.json` | 40.0, 40.1, 40.4 | Immutable candidate provenance and artifact binding |
| `qualification-artifact-index.json` | 40.0, 40.1 | Workflow transport and expiry binding |
| `stable-site-release-facts.json` | 40.0, 40.5 | Derived pinned site deployment facts |
| `stable-release-signoff.json` | 40.0, 40.5 | Protected attempts, mutations, Marketplace, and smoke |
| incident request/sign-off | 40.0, 40.3, 40.5 | Withdrawal, rollback, roll-forward, and closure |
| install receipt / CLI version / update plan | 40.0, 40.2 | Stable installer and self-update agreement |
| documentation suite reports | 40.0, 40.4 | Stable public claims and support floors |
| Rust stable-candidate report | certification_0, 40.1 | Advertised Rust-interop claim boundary |
| VSIX qualification report | 40.4 | Qualified editor artifact consumed without rebuild |

### milestone_40_1: Canonical Release Plan and Qualification

- [x] Implement the non-mutating stable planner and canonical digest binding.
- [x] Consume and register the separately delivered Rust-interop
  stable-candidate suite and claims artifact before qualification.
- [ ] Qualify all compiler, sysroot, installer, documentation, Rust-claim, site,
  and VSIX artifacts.
- [x] Validate first-GA and normal predecessor/rollback semantics.
- [ ] Record review rounds, PR, validation, and merge.

### milestone_40_2: Stable Distribution and Self-Update

- [ ] Enable stable resolution in dispatchers, immutable installer, exact pins,
  receipts, and `sifr self update`.
- [ ] Keep publication mutation disabled.
- [ ] Validate all four targets and negative checksum/withdrawal/channel cases.
- [ ] Record review rounds, PR, validation, and merge.

### milestone_40_3: Rollback and Incident Governance

- [ ] Implement local fail-closed incident planning, rollback, withdrawal,
  roll-forward, generation burning, and recovery evidence.
- [ ] Keep production mutation adapters disabled.
- [ ] Validate first-GA and later incident recovery.
- [ ] Record review rounds, PR, validation, and merge.

### milestone_40_4: Stable Documentation and VS Code Release

- [ ] Add GA documentation checks and stable public documentation.
- [ ] Qualify the exact VSIX and Marketplace identity without publication.
- [ ] Materialize the reviewed `0.1.0` candidate evidence.
- [ ] Record review rounds, PR, validation, and merge.

### milestone_40_5: Protected Sign-off and GA Activation

- [ ] Add the single protected publication workflow and production site adapter.
- [ ] Publish or verify write-once assets, Marketplace version, governed index
  activation, site facts, and post-publication smoke.
- [ ] Exercise resume, stale generation, burned generation, rollback, and
  incident roll-forward.
- [ ] Record review rounds, PR, validation, and merge.

## Final Phase Closure

- [ ] Every milestone PR is merged and linked below.
- [ ] Full implementation receives repeated Claude Opus review until approved.
- [ ] Release profile and all Phase 40 suites pass on the final source commit.
- [ ] Phase, roadmap, architecture, distribution, and execution docs record the
  final state.
- [ ] This issue is archived only after the Phase 40 exit gate is satisfied.

## Merged PRs and Review Evidence

### milestone_40_0

Status: complete in
[PR #3025](https://github.com/sifr-lang/sifr/pull/3025). Rust-interop
`stable-candidate` registration and `stable_support_claims.json` consumption
occur at milestone 40.1 before qualification and did not block this
architecture/gate lock PR.

- Review pass 1:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-1.md`
  requested corrections to CAS, provenance, custody, downgrade protection,
  release-report coverage, source decomposition, and exact scope.
- Review pass 2:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-2.md`
  requested conditional-field, schema parity, producer/consumer, documentation,
  and tracking corrections.
- Review pass 3:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-3.md`
  confirmed every pass-2 finding was resolved, then requested a central
  fail-closed enum primitive, governed timestamp errors, stricter rejection
  assertions, and removal of the duplicate documentation-suite authority. A
  fourth pass after remediation.
- Review pass 4:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-4.md`
  verified the fail-closed fixes with exhaustive corruption and differential
  schema/validator sweeps, then found one remaining plan-identity divergence
  plus three mutation/dead-branch cleanup items. A fifth pass is required after
  remediation.
- Review pass 5:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-5.md`
  confirmed plan identity and expiry parity, then found the final
  `sysroot_schema_version` schema/validator mismatch and one stale workflow
  diagnostic. A sixth pass is required after remediation.
- Review pass 6:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-6.md`
  confirmed receipt/CLI schema parity and found two remaining weak-validator
  fields: release sign-off version class and qualification artifact identifier
  shape. A seventh pass is required after remediation.
- Review pass 7:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-7.md`
  found no local defects after broadened raw-exception and
  schema/validator-differential sweeps. Milestone 40.0 was locally approved
  before its final upstream rebase and capability-based demo rename.
- Review pass 8:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-8.md`
  independently re-ran 17,682 adversarial schema/validator cases with no unsafe
  divergence, then requested two naming/ownership consistency corrections.
- Review pass 9:
  `plans/reviews/archive/phase-40-milestone-40-0-claude-opus-review-pass-9.md`
  verified both corrections, the capability-based demo convention, and the
  fail-closed 40.1 qualification boundary, then returned `APPROVED`.
- Passing local evidence:
  - `demos/stable_release_governance_demo.sh`
  - `scripts/run_all_tests.sh --profile create-pr`
  - `scripts/run_all_tests.sh --profile merge` (all enforced lanes passed:
    674/674 E2E fixtures and 261 hardening variants; zero blocking failures)
  - `uv run --project verification --locked python -m sifr_verify areas run
    --area distribution_release --suite full --suite evidence-custody`
  - `uv run --project verification --locked python -m sifr_verify areas run
    --area developer_tooling --suite editor-release`
  - `uv run --project verification --locked python -m sifr_verify areas run
    --area rust_interop --suite matrix --suite tiers
    --suite compatibility-matrix --suite stale-drafts`
  - `uv run --project verification --locked python -m sifr_verify areas run
    --area documentation --suite structure`
  - governance/schema/runner self-tests, Rust self-update unit tests, Clippy,
    formatting, taxonomy, and file-size guardrails
- Deferred dispatcher parser replacement: the generated shell dispatcher still
  performs its preview-only structural checks with text matching. Its mandatory
  strict schema-v2 active/preview parser and stable behavior remain assigned to
  `milestone_40_2`; 40.0 keeps stable resolution unavailable.

### milestone_40_1

Status: implementation and local qualification in progress. The separately
owned Rust certification input merged through
[PR #3026](https://github.com/sifr-lang/sifr/pull/3026) and is consumed without
modifying its Rust-interop implementation.

- Review pass 1:
  `plans/reviews/active/phase-40-milestone-40-1-claude-opus-review-pass-1.md`
  requested exact artifact-id and path custody, end-to-end planner
  materialization and drift tests, immutable-installer identity binding,
  deterministic VSIX evidence, the governed locked build path, verified
  workflow repository identity, exact Rust-claim consumption, and the missing
  capability demo. Those findings are remediated and await the next review
  pass.
- Review pass 2:
  `plans/reviews/active/phase-40-milestone-40-1-claude-opus-review-pass-2.md`
  verified all pass-1 product-path corrections, then found a confounded
  digest-sensitivity fixture, a symlinked-container custody escape, incomplete
  artifact-id-to-target/container binding, and a test-order flaw in the
  outside-checkout assertion. Remediation now uses a same-source no-op control,
  rejects container symlinks and resolved paths outside the artifact root,
  binds every governed artifact id to its exact kind/target/upload/name,
  exercises the output guard with valid evidence, and derives the 30-day
  retention interval from API timestamps. A third review pass is required.
- Review pass 3:
  `plans/reviews/active/phase-40-milestone-40-1-claude-opus-review-pass-3.md`
  confirmed the pass-2 corrections, then found raw non-UTF-8 evidence
  tracebacks, an unbound dispatch-workflow commit, loose editor/documentation
  report shapes, and missing mismatched-ref coverage. Remediation now converts
  text-decoding failures into governed errors, requires the dispatch head SHA
  to equal the candidate source commit, validates exact schema-v2 report
  shapes, adds binary-evidence and mismatched-ref negatives, and makes fresh
  fixture commit identities deterministic. A fourth review pass is required.
- Review pass 4:
  `plans/reviews/active/phase-40-milestone-40-1-claude-opus-review-pass-4.md`
  verified the pass-3 corrections, then found sibling non-UTF-8 read sites,
  alternate shell assignments that could evade installer identity parsing,
  and a collector-side symlink-container gap. Remediation now governs every
  release-profile/checksum/sysroot text decode, rejects any direct alternate or
  duplicate installer identity assignment, mirrors resolved-path custody in
  the collector, and permanently covers those cases. A fifth review pass is
  required.
- Review pass 5:
  `plans/reviews/active/phase-40-milestone-40-1-claude-opus-review-pass-5.md`
  verified the pass-4 collector and most text-custody corrections, then found
  the archive verifier's earlier non-UTF-8 sysroot decode, further shell forms
  that evade assignment parsing, and a one-claim fixture that could not test
  claim order. Remediation now governs the verifier decode without a traceback,
  regenerates the installer with the pinned governed producer and requires
  byte-for-byte equality instead of parsing shell, and exercises a two-claim
  fixture plus an order-reversal negative. A sixth review pass is required.
- Passing evidence so far:
  - `demos/stable_candidate_qualification_demo.sh` with a real
    `aarch64-apple-darwin` host artifact, isolated install, `sifr --version`,
    `sifr check`, `sifr self version`, and canonical planner output
  - fixture-backed repeated materialization plus source, submodule, lockfile,
    target artifact, sysroot, installer, Rust-claim, and VSIX digest sensitivity
  - negative missing-artifact, expired-artifact, cross-target, stale-report,
    source/version drift, floating/mismatched refs, in-checkout output,
    symlink-container, binary installer/checksum/profile, alternate installer
    assignments, byte-divergent installer regeneration, Rust-claim ordering,
    and exact editor/documentation shape cases
  - Rust-interop `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`, and
    `stable-candidate` suites, including the stable-claim adversarial self-test
  - qualification workflow contract, stable artifact generation, governance
    contracts, schema epoch, runner self-tests, formatting, HIR, and file-size
    guardrails
