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
- Canonical site repository: `sifr-lang/sifr-website` (the former
  `sifr-lang/sifr-blog-website` remote redirects here).
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
| `site-publication-facts.json` | 40.2 | Schema-v2 preview publication/site-dispatch binding |
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
- [x] Qualify all compiler, sysroot, installer, documentation, Rust-claim, site,
  and VSIX artifacts.
- [x] Validate first-GA and normal predecessor/rollback semantics.
- [x] Record review rounds, PR, validation, and merge.

### milestone_40_2: Stable Distribution and Self-Update

- [x] Enable stable resolution in dispatchers, immutable installer, exact pins,
  receipts, and `sifr self update`.
- [x] Keep stable-changing publication mutation disabled.
- [x] Validate all four targets and negative checksum/withdrawal/channel cases.
- [x] Record review rounds, PR, validation, and merge.

### milestone_40_3: Rollback and Incident Governance

- [x] Implement local fail-closed incident planning, rollback, withdrawal,
  roll-forward, generation burning, and recovery evidence.
- [x] Keep production mutation adapters disabled.
- [x] Validate first-GA and later incident recovery.
- [x] Record review rounds, PR, validation, and merge.

### milestone_40_4: Stable Documentation and VS Code Release

- Packaged-candidate generated-Rust preview is intentionally not a Phase 40
  prerequisite. The real `0.1.0` candidate serves initialization, diagnostics,
  and formatting, but the cold first-run generated-Rust qualification exceeded
  its deterministic bound through both `sifr emit` and
  `sifr.server.showGeneratedRust`. That independently repairable compiler
  startup/performance defect is recorded in
  [`adhoc_packaged_candidate_generated_rust.md`](../../phases/adhoc_packaged_candidate_generated_rust.md);
  stable public docs explicitly record the affected capability as outside the
  packaged `0.1.0` GA-qualified surface.
- The editor report records a Marketplace publication plan with status
  `planned`, not a synthetic dry-run pass. The credentialed dry run and exact
  no-rebuild workflow binding are realized in `milestone_40_5`.
- [ ] Use the coordinated upstream-first exception for editor release:
  merge the `sifr-vscode` package PR, merge the `editor-integrations` pointer
  PR, then update the main-repository submodule pointer and matching consumer
  rules in the same main PR.
- [ ] Add GA documentation checks and stable public documentation.
- [ ] Qualify the exact VSIX and Marketplace identity without publication.
- [ ] Materialize the reviewed `0.1.0` candidate evidence.
- [ ] Record review rounds, PR, validation, and merge.

Review and upstream coordination ledger:

- `sifr-lang/sifr-vscode`
  [PR #12](https://github.com/sifr-lang/sifr-vscode/pull/12) merged as
  `273fd5d3ebc958124c3151647e2b61136a3ddb06`. Package review pass 1 requested
  package cleanup and metadata corrections; pass 2 approved the exact package
  head. Both reports are archived as
  `plans/reviews/archive/phase-40-milestone-40-4-vscode-package-claude-opus-review-pass-{1,2}.md`.
- `sifr-lang/editor-integrations`
  [PR #10](https://github.com/sifr-lang/editor-integrations/pull/10) merged as
  `d7577d49274b97fdf508b7fa16b6d9bdb51b4acd`. Pointer review pass 1 requested
  exact consumer binding corrections; pass 2 approved the paired pointer and
  main-repository consumer head. Both reports are archived as
  `plans/reviews/archive/phase-40-milestone-40-4-editor-pointer-claude-opus-review-pass-{1,2}.md`.
- Main-repository Claude Opus review pass 1 is archived at
  `plans/reviews/archive/phase-40-milestone-40-4-claude-opus-review-pass-1.md`.
  Its eight findings are remediated: the GA sweep covers every public doc;
  Marketplace evidence is a truthful protected-workflow plan; rollback is a
  governed qualification input; the packaged generated-Rust limitation is
  public and evidence-based; qualification fixtures have headroom;
  unsupported target additions fail; and exact candidate commands preserve
  paths containing spaces.
- Main-repository Claude Opus review pass 2 is archived at
  `plans/reviews/archive/phase-40-milestone-40-4-claude-opus-review-pass-2.md`.
  It verified the substantive pass-1 closures and withheld approval only for
  fixture headroom, semantic preview-claim patterns, import ordering, and this
  review ledger. All four observations are remediated before pass 3.
- Main-repository Claude Opus review pass 3 is archived at
  `plans/reviews/archive/phase-40-milestone-40-4-claude-opus-review-pass-3.md`.
  Its three findings are remediated before pass 4: the rollback input is bound
  to the editor workflow step and protected by a structural contract; the LSP
  guide installs the qualified VSIX until protected Marketplace activation;
  and the capability demo runs the candidate compiler's test command. The
  review observations are also closed with positive target allowlisting,
  an operator command for documentation qualification, truthful VSIX package
  smoke naming, and governance self-test headroom.
- Main-repository Claude Opus review pass 4 is archived at
  `plans/reviews/archive/phase-40-milestone-40-4-claude-opus-review-pass-4.md`.
  Its three cross-surface findings are remediated before pass 5: the LSP guide
  uses the contributed `sifr.lsp.path` setting and the protected Marketplace
  acquisition path; the GA docs contract binds those exact facts; and
  self-update help names `alpha|beta|stable` plus an immutable governed
  version. The target detector also covers non-`aarch64`/`x86_64`
  architectures.
- Main-repository Claude Opus review pass 5 is archived at
  `plans/reviews/archive/phase-40-milestone-40-4-claude-opus-review-pass-5.md`.
  It re-ran the milestone gates, verified every finding from passes 1–4, found
  no remaining actionable issue, and approved the implementation.
- The rebased create-PR lane passed its coverage-matrix, core-guardrail, and
  diagnostic-rule steps before 18 of 19 selected Python-interop variants
  passed. Its sole failure was the repeated 120-second
  `readonly-check-doctor` host timeout. The separately invoked documentation
  structure/GA-release, editor-release, and distribution qualification/full
  gates all passed. Exact create-PR evidence is appended to
  `plans/phases/adhoc_performance_budget_host_variance.md` and is not a Phase
  40 prerequisite. No timeout or validation waiver was added.
- Post-merge qualification run
  [#30270476093](https://github.com/sifr-lang/sifr/actions/runs/30270476093)
  bound source `3ebe27bc4095134137a5b47df7ea372aff936011` and passed all
  four governed targets, editor qualification, and aggregate assembly. Its
  collector exposed two real GitHub API semantics missing from the fixtures:
  `name` contains the dynamic `run-name`, while `path` identifies the workflow;
  and `created_at` can trail the expiry anchor by several upload seconds. The
  repair binds the exact workflow path and permits at most 60 seconds of
  one-sided timestamp skew while the workflow contract still requires
  `retention-days: 30`; the real six-container replay produces and validates
  the complete canonical 20-row index.
- Claude Opus collector-repair review pass 1 is archived at
  `plans/reviews/archive/phase-40-milestone-40-4-collector-repair-claude-opus-review-pass-1.md`.
  It verified the API-path binding, one-sided timestamp bound, and real
  six-container replay, then requested an explicit over-retention mutation.
  The repair now rejects both a 61-second shortfall and one second beyond 30
  days; its error also records the observed timestamps and interval.
- Claude Opus collector-repair review pass 2 is archived at
  `plans/reviews/archive/phase-40-milestone-40-4-collector-repair-claude-opus-review-pass-2.md`.
  It independently reproduced the live API semantics and 509 MB artifact
  replay, mutation-tested both retention bounds and the workflow-path binding,
  found no remaining actionable issue, and approved the repair.

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

Status: complete. [PR #3028](https://github.com/sifr-lang/sifr/pull/3028)
merged as `56f8c41eec`. The separately owned Rust certification input merged
through
[PR #3026](https://github.com/sifr-lang/sifr/pull/3026) and is consumed without
modifying its Rust-interop implementation.

- Review pass 1:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-1.md`
  requested exact artifact-id and path custody, end-to-end planner
  materialization and drift tests, immutable-installer identity binding,
  deterministic VSIX evidence, the governed locked build path, verified
  workflow repository identity, exact Rust-claim consumption, and the missing
  capability demo. Those findings are remediated and await the next review
  pass.
- Review pass 2:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-2.md`
  verified all pass-1 product-path corrections, then found a confounded
  digest-sensitivity fixture, a symlinked-container custody escape, incomplete
  artifact-id-to-target/container binding, and a test-order flaw in the
  outside-checkout assertion. Remediation now uses a same-source no-op control,
  rejects container symlinks and resolved paths outside the artifact root,
  binds every governed artifact id to its exact kind/target/upload/name,
  exercises the output guard with valid evidence, and derives the 30-day
  retention interval from API timestamps. A third review pass is required.
- Review pass 3:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-3.md`
  confirmed the pass-2 corrections, then found raw non-UTF-8 evidence
  tracebacks, an unbound dispatch-workflow commit, loose editor/documentation
  report shapes, and missing mismatched-ref coverage. Remediation now converts
  text-decoding failures into governed errors, requires the dispatch head SHA
  to equal the candidate source commit, validates exact schema-v2 report
  shapes, adds binary-evidence and mismatched-ref negatives, and makes fresh
  fixture commit identities deterministic. A fourth review pass is required.
- Review pass 4:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-4.md`
  verified the pass-3 corrections, then found sibling non-UTF-8 read sites,
  alternate shell assignments that could evade installer identity parsing,
  and a collector-side symlink-container gap. Remediation now governs every
  release-profile/checksum/sysroot text decode, rejects any direct alternate or
  duplicate installer identity assignment, mirrors resolved-path custody in
  the collector, and permanently covers those cases. A fifth review pass is
  required.
- Review pass 5:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-5.md`
  verified the pass-4 collector and most text-custody corrections, then found
  the archive verifier's earlier non-UTF-8 sysroot decode, further shell forms
  that evade assignment parsing, and a one-claim fixture that could not test
  claim order. Remediation now governs the verifier decode without a traceback,
  regenerates the installer with the pinned governed producer and requires
  byte-for-byte equality instead of parsing shell, and exercises a two-claim
  fixture plus an order-reversal negative. A sixth review pass is required.
- Review pass 6:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-6.md`
  independently closed every pass-5 correctness finding and found one durable
  architecture-documentation omission. Remediation now documents the pinned
  governed-producer regeneration and byte-equality binding, and the workflow
  contract pins the exact production invocation that must remain identical to
  planner regeneration. A seventh review pass is required.
- Review pass 7:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-7.md`
  verified the architecture documentation but found that the new workflow
  contract literal ended before the closing output-path quote. Remediation
  anchors the full four-line invocation, including its closing quote and
  newline, so appended arguments or an altered output path cannot satisfy the
  contract. An eighth review pass is required.
- Review pass 8:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-8.md`
  independently verified the full invocation anchor, all earlier correctness
  closures, the capability-named demo and surfaces, durable documentation, and
  the milestone tracker. The reviewer returned `APPROVED` with no actionable
  findings.
- Final PR-head review pass 9:
  `plans/reviews/archive/phase-40-milestone-40-1-claude-opus-review-pass-9.md`
  verified ready PR #3028 at exact head `aeff4d07a`, all eight earlier review
  closures, the three tracker-only follow-up commits, the authoritative
  create-PR evidence, and the complete milestone diff. The reviewer returned
  `APPROVED` with no actionable findings and declared the PR ready to merge.
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
  - authoritative `scripts/run_all_tests.sh --profile create-pr`: pass,
    including 131/131 E2E fixtures and zero blocking failures; the cold-cache
    21.8-minute wall time exceeded only the advisory warm target, with 1.5 GiB
    peak RSS and no swap
  - post-merge `scripts/run_all_tests.sh`: every completed merge-only area
    passed, including CPython differential, 25 Python-interop variants, all
    Rust-interop suites, and 32 developer-tooling variants; representative
    performance then stopped on small median-budget variance
  - the representative retry failed a changed benchmark set, and the identical
    suite also failed on immediate parent `082988df1f`; the unrelated
    host/budget-stability work is deferred to
    `plans/phases/adhoc_performance_budget_host_variance.md`

### milestone_40_2

Status: in progress. The required site workflow landed first through
[sifr-website PR #14](https://github.com/sifr-lang/sifr-website/pull/14), merged
as `721bceca795a79a03af74ccb707d117a6f031f38`. Its GA-aware default-channel
binding landed through
[sifr-website PR #15](https://github.com/sifr-lang/sifr-website/pull/15), merged
as `07d88cc3c24707e386c5ad73fb0875c06ffd598f`. The main-repository caller pins
that exact protected-main commit through
`sifr-release-site-stable-distribution`; active ruleset `19791667` prohibits
updating or deleting that exact tag with no bypass actors.

- Site review passes 1–5 found and closed immutable-action pinning, credential
  persistence, metadata-shadow output, deployed-byte custody, stale-CDN index
  checks, site-base attribution, terminal failure recording, protected-main
  ancestry, mutation-boundary parsing, live-byte verification, bounded retry,
  Cloudflare attribution, and PR-time CI gaps.
- Site review pass 6 returned `APPROVED` with no actionable findings after the
  paired Sifr generator contract, unique deployment tag, ignored-env boundary,
  attempt length, and routing-asset checks were added.
- Exact site PR-head review pass 7 independently verified clean/pushed head
  `85de564`, the complete six-file diff, green `build website` check, all prior
  closures, local build/routing/workflow validation, and returned `APPROVED`
  ready to merge.
- GA-aware site review pass 1 found and closed contradictory operator
  documentation, a missing input inventory item, and a vacuous beta-only
  dispatcher distinctness check. Pass 2 verified all closures at exact head
  `b33c8e7` and returned `APPROVED`.
- Main implementation enables schema-v2 active stable fresh install, exact
  stable pins, forced preview-to-stable and ordinary stable-to-stable
  self-update, installer SHA-256 verification, `rc` removal, canonical
  max-generation snapshots, write-once version assets, the sole reusable
  mutation workflow, read-only local planning, and the pinned paired site
  handoff.
- Capability demo: `demos/stable_self_update_demo.sh`.
- Cross-repository fixture:
  `verification/areas/distribution_release/fixtures/site_release_contract.json`.
- Full main implementation review pass 3 verified every earlier main/site
  closure and independently passed 102 distribution variants, 48 self-update
  tests, Clippy, formatting, guardrails, and the capability demo. It found four
  follow-ups now under remediation: stale preview-lifecycle demo prose,
  canonical site-repository schema parity, redundant exact-pin lookup, and
  explicit release-tag source targeting.
- Full main implementation review pass 4 verified all pass-3 closures and the
  live site tag/ruleset/workflow identities. Its remaining findings are now
  remediated by pinning the attested no-bypass ruleset revision, enforcing a
  wall-clock site deadline with cancellation headroom, rejecting exact stable
  pins under preview metadata in Rust, explicitly passing only the site
  Actions secret, and removing an early-exit snapshot-name pipeline.
- Full main implementation review pass 5 verified every substantive pass-4
  closure, all earlier findings, the live ruleset/tag/workflow identities, and
  102 distribution variants with zero failures. Its sole actionable finding
  was two trailing blank lines at EOF; those are removed. The informational
  polling note is also hardened with three bounded query attempts.
- Full main implementation review pass 6 rechecked the complete 71-file
  milestone diff, all earlier closures, 49 self-update tests, and 27 targeted
  distribution cases at exact head `f28b9d8fa`; it returned `APPROVED` with no
  actionable findings.
- Main-repository [PR #3030](https://github.com/sifr-lang/sifr/pull/3030)
  opened at exact head `2c282f1c7`. The authoritative
  `scripts/run_all_tests.sh --profile create-pr` gate passed at that head,
  including all blocking lanes and 131/131 E2E fixtures. A prior cold,
  contended Python doctor timeout passed in isolation and then passed inside
  the complete authoritative rerun.
- Exact PR-head review pass 7 found six issues: preview-era site planner and
  validator defaults, an unschematized publication binding, zero-digest
  acceptance, missing current evidence, and phase-index/EOF drift. The
  remediation makes preview/active site source validation GA-aware, registers
  canonical `site-publication-facts.json` schema-v2 producer/schema/validator
  coverage with zero-digest rejection, records current PR evidence, and repairs
  the ad hoc phase index and whitespace. The complete distribution area passes
  52/52 variants after remediation; another exact-head review is required.
- The authoritative remediation gate
  `scripts/run_all_tests.sh --profile create-pr` passed at exact implementation
  head `e29722dfe46bc4f091eb66e7be47744a4c14b24b`, including the registered
  12-schema runner-foundation inventory, every blocking lane, and 131/131 E2E
  fixtures (`report_signature=7c39b8c1dd4fec7c`). The reported warm-wall-time
  advisory is non-blocking and covered by the indexed `PERF-HOST` follow-up.
  PR #3030 now requires an independent full-diff review at its pushed
  documentation-inclusive head.
- Exact PR-head review pass 8:
  `plans/reviews/archive/phase-40-milestone-40-2-claude-opus-review-pass-8.md`
  independently reproduced the preview planner against the real pinned site
  checkout, reverified the live protected tag/ruleset/workflow identities,
  closed all six pass-7 findings, ran the focused milestone checks, and
  returned `APPROVED` at pushed head `28fe8527f` with no actionable finding.
  The archived review and this ledger entry are the only subsequent
  documentation changes; a final exact-head review follows their push.
- The required merge-profile run at documentation-inclusive head `27d2cea83`
  passed every executed lane through developer tooling, including CPython
  differential, 25/25 Python-interop variants, 10/10 consumed Rust-interop
  variants, and all core/diagnostic/frontend/tooling checks. It stopped only at
  the representative performance budget on two host-sensitive medians. An
  immediate isolated retry reduced the overruns to 0.55% and 0.69% but
  reproduced the same variance. This is the independently parent-reproduced
  condition recorded in indexed, non-prerequisite follow-up `PERF-HOST`; no
  baseline or waiver was changed.
- The milestone-specific closure commands then passed independently:
  distribution `full` plus `evidence-custody` 53/53, developer-tooling
  `editor-release` 6/6, consumed Rust-interop matrix/tiers/compatibility/stale
  drafts 8/8, Rust stable-candidate 2/2, and documentation structure 1/1.
  These commands made no tracked-file changes.
- Exact PR-head review pass 9:
  `plans/reviews/archive/phase-40-milestone-40-2-claude-opus-review-pass-9.md`
  reverified the complete 86-file PR diff, reproduced the milestone contract
  counts, independently accepted the indexed `PERF-HOST` result as
  non-prerequisite host variance, and returned `APPROVED` with no finding at
  exact remote head `939e69083`.
- Main-repository [PR #3030](https://github.com/sifr-lang/sifr/pull/3030)
  merged as `db80dd35e056b9dcc9a2ac64475a198f5c36bfaa`.

### milestone_40_3

- Pure incident planning in
  `verification/areas/distribution_release/governance/incident_planner.py`
  binds canonical request, affected/target/successor plan, and expected live
  generation/digest bytes. Rollback requires the affected `normal` plan's exact
  active predecessor; incident roll-forward requires a request-bound qualified
  successor with `rollback_target: none`.
- The release-index core now preserves every unaffected channel and retained
  release byte, withdraws only the affected stable, and either reuses exactly
  one active retained rollback target or adds exactly one qualified successor.
- The credential-free fixture harness takes explicit temporary index,
  governance-asset, immutable-asset, Marketplace-stub, extension-metadata, and
  non-deploying-site paths. It refuses production credentials and site
  repositories without the local-only marker and has no network, GitHub
  release, Marketplace publication, or repository-dispatch adapter.
- Local mutation order is request retention, write-once proposed-generation
  snapshot, atomic index replacement, exact generation/digest site recheck,
  bounded 20-minute attempt model, site reconciliation, and schema-v2 incident
  sign-off. Reservation failure burns the generation; post-index timeout
  records cancellation and resumes site work without a second index mutation.
- Fresh install, working-client self-update, and broken-client out-of-band
  recovery all resolve the active stable and delegate to its digest-verified
  immutable installer. Downgrades refuse without explicit `--force`.
- Evidence-only commit validation accepts exactly the canonical incident
  request and digest-bound withdrawal evidence under
  `plans/releases/incidents/<incident-id>/`; source changes or unrelated
  evidence fail.
- The stable incident runbook records owner, non-initiating approval authority,
  30-minute acknowledgement target, triggers, communication locations, retry
  matrix, retention, first-GA roll-forward, and closure requirements.
- Focused evidence:
  `uv run --project verification --locked python -m sifr_verify areas run
  --area distribution_release --suite incident-governance` passes the
  nine-scenario recovery module; `demos/stable_incident_recovery_demo.sh`
  demonstrates burned-generation resume, forced rollback recovery through both
  client paths, immutable-installer execution, and first-GA roll-forward.
- Claude Opus review pass 1 was not approved and is archived at
  `plans/reviews/archive/phase-40-milestone-40-3-claude-opus-review-pass-1.md`.
  Its five findings are remediated: canonical incident custody now uses
  `plans/releases/incidents/` and is called by the evidence-custody repository
  check; the sole-first-GA rollback rejection has a direct acceptance test;
  incident index-transition cases live in the dedicated incident module rather
  than consuming the shared governance file-size boundary; the sign-off schema
  and validator require exactly one completed terminal attempt; and merge,
  nightly, and release profiles select the named `incident-governance` suite,
  with the release report requiring it and the full-suite runner de-duplicating
  the module.
- The authoritative create-PR profile completed all functional validation,
  including 19/19 Python-interop variants, but exited on the host timing budget
  after that passing step took 788.45 seconds against 600 seconds. This
  unrelated host variance is recorded in
  `plans/phases/adhoc_performance_budget_host_variance.md` and is not a Phase
  40 prerequisite; no baseline or waiver changed.
- Claude Opus review pass 2 was not approved and is archived at
  `plans/reviews/archive/phase-40-milestone-40-3-claude-opus-review-pass-2.md`.
  Its three findings are remediated: a focused incident-index mutation module
  now directly tests atomic withdrawal, channel isolation, rollback/roll-forward
  version sets, and retained-release byte preservation; the shared self-test's
  top-level separator is restored; and custody explicitly allows the release
  README only for candidate evidence while incident evidence remains an exact
  request-plus-withdrawal-evidence commit.
- Claude Opus review pass 3 was not approved and is archived at
  `plans/reviews/archive/phase-40-milestone-40-3-claude-opus-review-pass-3.md`.
  It cleared every implementation and prior-review finding; its sole new
  documentation finding is remediated by making the adjacent release-evidence
  README require the digest-bound `withdrawal-evidence.txt` beside every
  incident request.
- Claude Opus review pass 4 is approved with no actionable findings and is
  archived at
  `plans/reviews/archive/phase-40-milestone-40-3-claude-opus-review-pass-4.md`.
  It re-ran the nine incident scenarios, combined 55-variant distribution
  selection, coverage/profile assignment, runner self-tests, the capability
  demo, file-size guardrails, custody-layout reproduction, stale-path sweep,
  and a fresh full definition-of-done review against exact implementation head
  `cc87f1e79a2d11c2f2cd1fba8b99d470741c82da`.
- Exact PR-head Claude Opus review pass 5 is approved with no actionable
  findings and is archived at
  `plans/reviews/archive/phase-40-milestone-40-3-claude-opus-review-pass-5-pr-head.md`.
  It reverified the complete merge candidate and the pass-4 tracking-only
  delta at exact remote head
  `e42bb9a3d4fb48ae3ba50fc9209aa2e8cd5c10d7`.
- Main-repository [PR #3032](https://github.com/sifr-lang/sifr/pull/3032)
  merged as `97df4acb4656ee55754ec87d4c2d982b13df740e`.
