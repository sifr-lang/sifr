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
- Protected review policy: each initial and resume attempt requires a fresh
  GitHub-recorded `stable-release` approval retained in sign-off. The default
  requires a non-initiating `release/distribution` reviewer and forbids
  self-review. The user-directed temporary single-maintainer exception in
  `plans/releases/single-maintainer-approval-waiver.json` authorizes only
  `bootstrap-alpha`, `bootstrap-index`, and `ga-activation` through
  2026-08-27. It still requires the named owner to approve the protected
  environment, disables admin bypass, binds the waiver digest into retained
  evidence, and cannot authorize normal or incident publication. Distinct
  reviewer restoration is tracked by
  [`ad-hoc-distinct-release-reviewer-restoration.md`](./ad-hoc-distinct-release-reviewer-restoration.md).
- The `0.1.0` qualification expires at `2026-08-28T02:17:30Z`, but protected
  GA prepare requires seven full days of remaining lifetime. Its effective
  start deadline is therefore `2026-08-21T02:17:30Z`; the waiver's later
  expiry does not extend the candidate.
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

- The separately owned pinned algorithmic full-corpus follow-up linked under
  `milestone_40_4` has remediated its 20 preserved failures; its closeout
  validation and review remain in progress. Release qualification now blocks
  on the full corpus and taxonomy self-test, matching nightly.
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

- The pinned algorithmic full corpus was not a Phase 40 prerequisite.
  Exact-source release validation reproduced the same 20 pre-existing failures
  preserved in
  [`ad-hoc-algorithmic-full-corpus-preexisting-failures.md`](./ad-hoc-algorithmic-full-corpus-preexisting-failures.md)
  after every preceding gate—including `performance_budget_checks` in `full`
  mode—passed. The follow-up has remediated those 20 failures and now keeps the
  full corpus and taxonomy self-test blocking in both nightly and release
  qualification; its closeout validation and review remain in progress.
- The independently scoped packaged-candidate generated-Rust follow-up is
  complete. [PR #3102](https://github.com/sifr-lang/sifr/pull/3102) qualified
  cold and warm installed `sifr emit` and `sifr.server.showGeneratedRust` on
  all four supported targets, including cancellation and bounded shutdown.
  Stable public docs now include the qualified editor action. The completed
  record is archived in
  [`adhoc_packaged_candidate_generated_rust.md`](../archive/adhoc_packaged_candidate_generated_rust.md).
- The editor report records a Marketplace publication plan with status
  `planned`, not a synthetic dry-run pass. The credentialed dry run and exact
  no-rebuild workflow binding are realized in `milestone_40_5`.
- [x] Use the coordinated upstream-first exception for editor release:
  merge the `sifr-vscode` package PR, merge the `editor-integrations` pointer
  PR, then update the main-repository submodule pointer and matching consumer
  rules in the same main PR.
- [x] Add GA documentation checks and stable public documentation.
- [x] Qualify the exact VSIX and Marketplace identity without publication.
- [x] Materialize the reviewed `0.1.0` candidate evidence.
- [x] Record review rounds, PR, validation, and merge.

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
  `plans/issues/active/adhoc_performance_budget_host_variance.md` and is not a Phase
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
- The historical clean exact-source release profile on
  `c17f3c7d1ea1ed97ca125eb7a43344b30cf9413b` passed every lane through
  `performance_budget_checks` in `full` mode before reproducing exactly the 20
  previously preserved failures among 412 pinned algorithm variants. The
  follow-up remediated those failures without a baseline or exclusion and
  removed the temporary release divergence by restoring `leetcode-full` plus
  taxonomy smoke to release qualification.
- Claude Opus algorithm-scope review passes 1 through 3 are archived at
  `plans/reviews/archive/phase-40-algorithm-scope-claude-opus-review-pass-{1,2,3}.md`.
  Their findings are closed by exact release-suite/profile agreement,
  profile-derived divergence detection independent of the assignment matrix,
  indexed-record and expiry validation, mutation coverage for deletion and
  under-declaration paths, truthful policy/docs attribution, and restoration
  criteria in the ad hoc issue.
- Claude Opus algorithm-scope review pass 4 is archived at
  `plans/reviews/archive/phase-40-algorithm-scope-claude-opus-review-pass-4.md`.
  It independently re-ran the focused gates and 24 negative cases, found no
  remaining actionable issue, and approved the correction.
- The authoritative `create-pr` profile passed every blocking step, including
  131 of 131 selected e2e fixtures. The exact-state documentation
  `structure`/`ga-release` suites, coverage-matrix readiness with all 24
  negative cases, formatting, diff hygiene, and the 900-line file-size
  guardrail also passed. The only lane advisory was the unchanged warm
  wall-time budget while isolated e2e caches rebuilt.
- Exact pushed-head Claude Opus review pass 5 is archived at
  `plans/reviews/archive/phase-40-algorithm-scope-claude-opus-review-pass-5-pr-head.md`.
  It independently reviewed commit `99c847705`, mutation-tested every
  deletion and under-declaration path raised in earlier passes, verified the
  nightly corpus and release subsets remain blocking, and returned `APPROVED`
  with no actionable findings.
- Final PR-head Claude Opus review pass 6 is archived at
  `plans/reviews/archive/phase-40-algorithm-scope-claude-opus-review-pass-6-final-pr-head.md`.
  It found one evidence-attribution error: the assignment-matrix checker, not
  both checkers, owns the PAM under-declaration mutation. The underlying
  readiness gate remains fail-closed because both checks are blocking; the
  archived pass-5 table and conclusion now state that division accurately.
- Final PR-head Claude Opus review pass 7 is archived at
  `plans/reviews/archive/phase-40-algorithm-scope-claude-opus-review-pass-7-final-pr-head.md`.
  It independently reproduced the corrected checker attribution, re-ran the
  mutation matrix, found no remaining actionable issue, and approved exact PR
  head `95d5e2bbb`.
- The governed algorithm-scope correction merged through
  [PR #3037](https://github.com/sifr-lang/sifr/pull/3037) as
  `7242e4737b1ee89f9f02a3b4793d5cdb13d372ea`.
- Exact-source qualification run
  [#30297288986](https://github.com/sifr-lang/sifr/actions/runs/30297288986)
  passed all four governed targets, aggregate installer/checksum assembly,
  VS Code package qualification, and immutable-index collection. External
  replay verified all 20 indexed files (533,743,470 bytes) by exact size and
  SHA-256; the workflow and every transported artifact remain unexpired.
- Documentation qualification on the same source passed as
  `docs-7242e4737b1e-038b0eabc1c1` in a local canonical report. Canonical
  release-profile report attempts passed every preceding gate but reproduced
  the indexed `PERF-HOST` condition; the same-host old-source control timed
  out, while the unchanged standalone full suite passed after disposable cache
  cleanup. No performance baseline, threshold, waiver, or release-profile
  selection changed. Commands, result paths, digests, and replay totals are
  archived in
  `plans/reviews/archive/phase-40-milestone-40-4-exact-source-evidence.md`.
- A later unchanged canonical run passed all eight performance variants and all
  earlier release lanes. It then failed only the installed sysroot smoke because
  the public governance release still serves schema-v1 `channels.json`; the
  exact schema-v2 candidate correctly rejected that preview state because the
  required `generation`, `ga_status`, and `releases` fields are absent. This is
  the truthful one-time GA epoch-bootstrap boundary for `milestone_40_5`, not a
  compiler, sysroot, or performance failure. The installed heavy-stdlib sysroot
  variant passed on the same run. Milestone 40.5 also owns the missing
  test-only endpoint override so installed-sysroot qualification uses an
  isolated schema-v2 fixture instead of mutable public network state.
- On exact source `8a23f90869a68438a7b4ae3b8f9623531d1ce68f`, a
  low-noise canonical release-profile run passed all eight performance
  variants, all 69 distribution-release variants, all 25 Python-interop
  variants, all 10 consumed Rust-interop variants, all 48 developer-tooling
  variants, and both GA documentation variants. The blocking release profile
  later reached its first generated-code Clippy failure at
  `e2e-018-cpython-math-semantic-corrections`, whose emitted
  `const NAN: f64 = (0.0_f64) / (0.0_f64);` is rejected by Rust 1.94
  `clippy::zero_divided_by_zero`; later ordered corpus entries were not
  Clippy-checked by that run. This generated stdlib constant defect is recorded
  in indexed, non-prerequisite follow-up `GENC-NAN`. Before remediation, the
  release profile selected the expiry-bound
  `generated_code_quality:release-full` suite. Every full gate and Clippy entry
  stayed blocking. The three affected entries had to reproduce only
  `clippy::zero_divided_by_zero` as expected-failure evidence. The complete
  91-entry corpus was materialized before that set was frozen. Nightly kept
  unmodified `generated_code_quality:full`. No Clippy allow, threshold,
  generated source, or stable-governance contract changed.
- PR [#3103](https://github.com/sifr-lang/sifr/pull/3103) completed
  `GENC-NAN` on 2026-08-11. The compiler now emits canonical Rust constants for
  NaN and both infinity signs. Release and nightly now select blocking
  `generated_code_quality:full`, and the temporary divergence no longer exists.
- Generated-code release-divergence review pass 1 is archived at
  `plans/reviews/archive/phase-40-generated-code-release-divergence-review-pass-1-not-satisfied.md`.
  Its seven findings are closed by whole-plan structural pinning, mandatory
  governed-entry execution, exact three-entry policy and matrix binding,
  machine-readable expected-failure disclosure, 15 fail-closed mutations, and
  per-entry failure collection. Review pass 2 is archived at
  `plans/reviews/archive/phase-40-generated-code-release-divergence-review-pass-2-satisfied.md`;
  it independently mutation-tested the repaired paths, found no actionable
  issue, and returned `VERDICT: SATISFIED`. The complete release-only Clippy
  evidence records 88 normal passes plus three exact expected failures among
  all 91 entries; `generated_code_quality:release-full` passes all seven gates
  with zero blocking failures and explicit `expected_failures=3` disclosure.
  The first create-PR attempt reproduced the indexed
  `readonly-check-doctor` per-command 120-second host timeout while all 18
  later Python-interop cases passed; the exact case then passed in isolation,
  and the warm authoritative rerun passed every blocking lane, all 19
  Python-interop variants, and 131 of 131 E2E fixtures with
  `report_signature=7c39b8c1dd4fec7c`. Its only advisory was the unchanged
  warm wall-time budget; no timeout, threshold, or waiver changed.
  Exact PR-head review pass 3 is archived at
  `plans/reviews/archive/phase-40-generated-code-release-divergence-review-pass-3-exact-pr-head-satisfied.md`.
  It matched local, remote, and PR head
  `eebc715f412be91e7751a0ac56a80d0e3ca4271b`, independently re-ran the
  fail-closed guards, found no actionable issue, and returned
  `VERDICT: SATISFIED`. Final frozen-head review pass 4 is archived at
  `plans/reviews/archive/phase-40-generated-code-release-divergence-review-pass-4-final-exact-head-satisfied.md`.
  It verified exact final head
  `a93330231735a83f78e7d0e8762a9d56d15022ed`, found no actionable issue,
  and returned `VERDICT: SATISFIED`. The release-divergence slice merged
  through [PR #3049](https://github.com/sifr-lang/sifr/pull/3049) as
  `bae42ba47d4c1324b2d34dc654effaef2d39576e`.
- Fresh exact-source qualification run
  [#30406842210](https://github.com/sifr-lang/sifr/actions/runs/30406842210)
  passed at source `53cc9c4bf36762d39a0b372402d202589f920c2e`: all four
  targets, aggregate installer/checksums, editor/VSIX qualification, and the
  collector succeeded. Independent custody replay verified the canonical index,
  all six unexpired uploads, and all 20 transported payloads by exact size and
  SHA-256.
- Final qualification-evidence review pass 1 is archived at
  `plans/reviews/archive/phase-40-final-qualification-evidence-review-pass-1-not-satisfied.md`.
  It returned `VERDICT: NOT SATISFIED` after proving a real supporting-evidence
  incompatibility: the release report hashed the Rust runner's pretty-printed
  result while candidate custody required canonical JSON, the certified stable
  claims source also needed deterministic canonical staging, and the staged
  Rust report had come from a standalone rerun instead of the release-profile
  invocation. The qualification transport itself passed the full independent
  audit.
- The remediation stays within Phase 40 release custody: the passing release
  evidence writer canonicalizes the exact consumed Rust result before binding
  its digest; a governed staging command derives canonical stable-claims bytes
  from the exact source file; and the planner requires those staged bytes to
  match that source-derived representation. No Rust-interop capability,
  compatibility claim, suite selection, or implementation changes.
- Canonical candidate-evidence review pass 2 is archived at
  `plans/reviews/archive/phase-40-canonical-candidate-evidence-review-pass-2-satisfied.md`.
  It independently reproduced all three blocker closures, ran the complete
  distribution area at 125/125, and returned `VERDICT: SATISFIED` with no
  blocking finding. Its recommended hardening is included before the next
  review: Rust-claim digest sensitivity now asserts both changed claim digest
  and ids despite the necessarily changed source commit; mutation tests cover
  noncanonical, drifted, in-checkout, missing, duplicate-key, wrong-area, and
  symlink evidence; the real CLI wiring is exercised; and the planner requires
  canonical Rust report bytes at its earliest load.
- Hardened-tree review pass 3 is archived at
  `plans/reviews/archive/phase-40-canonical-candidate-evidence-review-pass-3-satisfied.md`.
  It mutation-tested every new guard, independently reran the complete
  distribution area at 125/125 plus runner, qualification, custody, formatting,
  diff, and file-size gates, found no blocking issue, and returned
  `VERDICT: SATISFIED`. The reviewed diff touches no Rust-interop implementation
  or demo.
- The first authoritative create-PR attempt passed every case around the
  pre-existing `readonly-check-doctor` command, which hit its fixed 120-second
  subprocess limit under host contention. Its exact isolated suite immediately
  passed with one variant and zero failures. The warmed authoritative rerun then
  passed every blocking lane: Python interop 19/19, consumed Rust interop 10/10,
  generated-code and performance gates, all selected crates and runtime checks,
  and E2E 131/131 with `report_signature=7c39b8c1dd4fec7c`. Every enforced
  per-step budget passed; only the indexed nonblocking warm wall-time advisory
  remained. No timeout, threshold, baseline, waiver, or profile selection
  changed.
- Milestone evidence-closure Claude Opus review passes 1 through 4 are archived
  at
  `plans/reviews/archive/phase-40-milestone-40-4-evidence-closure-review-pass-{1,2,3,4}.md`.
  The first three rounds found and closed bootstrap ownership, public-network
  isolation, wording, command, artifact, metric-count, and digest-custody gaps.
  Pass 4 recomputed every preserved digest and measurement, found no remaining
  actionable issue, and returned `VERDICT: SATISFIED`.
- Exact pushed-head review pass 5 is archived at
  `plans/reviews/archive/phase-40-milestone-40-4-evidence-closure-review-pass-5-final-pr-head.md`.
  It verified PR head `b09845a86`, re-ran the 20-file custody replay, found no
  actionable issue, and returned `VERDICT: SATISFIED`. The evidence slice
  merged through [PR #3038](https://github.com/sifr-lang/sifr/pull/3038) as
  `21bd64d7c4cd83a45da274519ed0fdd3ac8d63f7`.
- The exact-head create-PR profile passed every selected case except the
  already indexed `readonly-check-doctor` 120-second host timeout. All later
  Python-interop cases passed, including the CPython 3.11 buffer, Arrow, and
  DLPack suites; no Phase 40 source change was present in that evidence PR.

### milestone_40_5: Protected Sign-off and GA Activation

- [ ] Publish a protected, truthful one-time schema-v2 preview epoch with fresh
  qualified alpha/beta records and no stable mapping; retain no v1 migration
  or fallback path.
- [x] Isolate installed-sysroot self-update qualification from mutable public
  network state while retaining separate protected public-endpoint smoke.
- Qualification-isolation wave validation passed the complete installed
  release smoke from a schema-v2 fixture, including self-update dry run,
  doctor, emit, LSP, and path-leakage checks. Claude Opus review pass 1 is
  archived at
  `plans/reviews/archive/phase-40-milestone-40-5-qualification-isolation-review-pass-1.md`.
  Its findings are remediated before pass 2: the override is dry-run-only,
  direct runner execution is restored, the release trust boundary is
  inventoried, symlinks and malformed fixture inputs fail closed, and this
  wave is tracked.
  Review pass 2 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-qualification-isolation-review-pass-2.md`;
  it reproduced the release-binary dry-run and real-update rejection, found no
  remaining actionable issue, and returned `VERDICT: SATISFIED`.
- Exact pushed-head review pass 3 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-qualification-isolation-review-pass-3-final-pr-head.md`.
  It re-ran the installed release smoke, full self-update unit surface,
  workspace clippy, and fail-closed fixture matrix at exact remote head
  `d78cfb756`, and returned `VERDICT: SATISFIED`. That approval was superseded
  when the later authoritative create-PR profile found the inventory omission
  described below.
- The first authoritative create-PR profile found that the new regular-file
  probe and fixture read were absent from the existing TypeScript-Go
  direct-read/probe inventory. The source-provider guardrail now classifies
  both sites as a non-semantic, dry-run-only release-qualification command
  surface; the compiler source-provider boundary is unchanged.
- Review pass 4 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-qualification-isolation-review-pass-4.md`.
  It independently reproduced the guardrail mutation, 53 self-update tests,
  workspace clippy, formatting, file-size and inventory checks, and a
  nine-case release-binary fail-closed matrix. Its sole bookkeeping finding,
  the missing pass-3 artifact and ledger link, is remediated above.
- Review pass 5 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-qualification-isolation-review-pass-5.md`.
  Its full authoritative create-PR profile passed 131 E2E fixtures and every
  blocking lane on the remediated working tree. Its remaining findings were
  to commit and push that remediation and refresh PR #3039's review summary;
  both are release-mechanics requirements completed before the final exact-head
  round.
- Exact pushed-head review pass 6 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-qualification-isolation-review-pass-6-final-pr-head.md`.
  It matched local, remote branch, and PR head
  `36e20eeb166a7a241b9f71c5b2080dd9b01e8703`, re-ran every gate affected by
  the final documentation commit, found no actionable issue, and returned
  `VERDICT: SATISFIED`.
- Qualification isolation merged through
  [PR #3039](https://github.com/sifr-lang/sifr/pull/3039) as
  `d8dd28a8013447365e3b1fab5a7422de5509ac3b`.
- The schema-epoch bootstrap wave now has a read-only prepare workflow, a
  `stable-release` environment boundary, immutable workflow-approval-history
  validation that rejects the initiator, exact opaque pre-epoch digest/size
  custody, fresh alpha staging, fresh beta plus generation-1 preview
  activation, write-once bootstrap evidence, site reconciliation, and real
  public install/self-update smoke. No pre-epoch payload fixture, parser,
  migration, compatibility reader, or fallback is present.
- Bootstrap review pass 1 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-1.md`.
  Its `VERDICT: NOT SATISFIED` findings were remediated by making prepare
  artifact selection rerun-safe, retaining prepare and alpha-stage
  correlations in durable evidence, accepting and recording all distinct
  non-initiating approvers, adding producer and schema mutation coverage,
  failing on any public-smoke override, preserving site-run cancellation after
  query failure, enforcing numeric ruleset identity, reducing the publication
  workflow to 795 lines, and extending the 900-line guardrail to workflow YAML.
- Bootstrap review pass 2 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-2.md`.
  Its `VERDICT: NOT SATISFIED` findings were remediated by freezing all four
  public-smoke output filenames and their shared workflow directory in the
  contract test, directly testing wrong-stage and wrong-alpha staged evidence,
  and checking the opaque pre-epoch digest and byte size agree across both
  workflows, the semantic validator, and the JSON Schema.
- Bootstrap review pass 3 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-3.md`.
  Its `VERDICT: NOT SATISFIED` findings were remediated by proving both
  site-workflow identity checks surround publication and dispatch, deduplicating
  the named bootstrap self-test from the default full-area run, removing an
  unused evidence serializer, using the installer's actual sysroot-isolation
  variable, validating a positive alpha-stage instance against the JSON Schema,
  and consolidating the bootstrap module's common imports.
- Bootstrap review pass 4 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-4.md`.
  Its `VERDICT: NOT SATISFIED` findings were remediated by assigning
  `epoch-bootstrap` to merge, nightly, and release profiles and release-report
  custody, documenting the named suite, and adding load-bearing negatives for
  duplicate smoke IDs with distinct digests and forbidden beta data in an
  alpha-stage record.
- The review's live query-string preflight succeeded against the public
  governance endpoint: the cache-busted URL returned the exact opaque
  `channels.json` identity
  `71b3243925670f56dc510b8f45b6614a622f58097a0fea9492f61d20dc4bf9ef`
  at 105 bytes.
- Bootstrap review pass 5 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-5.md`.
  Its `VERDICT: NOT SATISFIED` findings were remediated with independent
  semantic mutations for smoke length, duplicate and unknown smoke IDs,
  evidence and prepare digests, case-folded approver uniqueness, and active
  release status, plus a validly named tenth asset that makes the JSON Schema's
  exact-nine bound load-bearing.
- Bootstrap review pass 6 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-6.md`.
  Its `VERDICT: NOT SATISFIED` findings were remediated by isolating the
  short-smoke and valid-withdrawn-release cases and expanding the semantic
  mutation matrix across run identity, approval presence/uniqueness, prepare,
  alpha-stage and index digests, release-record custody, smoke membership and
  exact object shapes.
- Bootstrap review pass 7 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-7.md`.
  Its `VERDICT: NOT SATISFIED` full-validator mutation sweep was remediated with
  producer-path isolation for withdrawn records, source-commit disagreement,
  and record-version disagreement, plus direct guards for opaque byte size,
  wrapper shape, channel/version evidence, approver container/value types, and
  empty approval initiator/login identities.
- Bootstrap review pass 8 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-8.md`.
  Its `VERDICT: NOT SATISFIED` full-validator sweep found one remaining masked
  case: the scalar approver test repeated characters and hit uniqueness before
  the array-container guard. The scalar is now `abc`, whose distinct characters
  isolate and pin the container requirement; this also makes pass 7's ledgered
  container/value claim accurate.
- Bootstrap review pass 9 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-9.md`.
  Its `VERDICT: NOT SATISFIED` widened whole-call mutation sweep found the beta
  evidence validator was not independently pinned. The complete alpha mutation
  set is now mirrored for beta: object shape, channel/version, source commit,
  release-record digest, exact asset membership, and individual asset digests.
- Bootstrap review pass 10 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-10-satisfied.md`.
  Its widened 88-mutant whole-wave sweep independently re-ran every guard and
  validator deletion, confirmed all 33 survivors are structurally masked by
  pinned sibling guards, found no fail-open path, and returned
  `VERDICT: SATISFIED` with no actionable findings.
- The first authoritative create-PR run reached one unrelated transient LSP
  transcript timeout; the exact replay passed immediately in 7.6 seconds and
  the second full run passed it in 6 seconds. That second run then exposed two
  real bootstrap-registration gaps: the verification runner still expected 12
  governed release schemas and its production release-report fixture omitted
  the newly required `epoch-bootstrap` suite. Both fixtures now include the
  schema and suite, and the complete runner self-test passes.
- Bootstrap review pass 11 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-11.md`.
  Its `VERDICT: NOT SATISFIED` whole-wave registration sweep confirmed both
  runner remediations, then found the new read-only prepare workflow was
  omitted from the no-v1-residue contract. The forbidden reader, migration,
  and fallback sweep now covers prepare, publication, and bootstrap workflows.
- Bootstrap review pass 12 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-12-satisfied.md`.
  It re-ran the full distribution area (110/110, with the bootstrap exactly
  once), found no remaining actionable issue, and returned
  `VERDICT: SATISFIED`.
- The final create-PR profile completed all 19 Python-interop variants with
  zero failures but exited on the host timing budget after that passing step
  took 690.10 seconds against 600 seconds. Two preceding wave runs completed
  the same functional step within budget at 456.79 and 455.79 seconds. This
  unrelated host variance is recorded in
  `plans/issues/active/adhoc_performance_budget_host_variance.md`; no performance
  baseline or waiver changed, and it is not a Phase 40 prerequisite.
- Schema-v2 preview epoch bootstrap implementation is under review in
  [PR #3040](https://github.com/sifr-lang/sifr/pull/3040).
- Exact PR-head bootstrap review pass 13 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-13-exact-pr-head-satisfied.md`.
  It independently matched local, remote-branch, and PR head at
  `e51491338e396e6b8f2d19345c9df68242e2b029`, re-ran the complete focused
  gate set, found no actionable issue, and returned `VERDICT: SATISFIED`.
- Frozen-head bootstrap review pass 14 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-schema-bootstrap-review-pass-14-final-pr-head-satisfied.md`.
  It matched local, remote-branch, and PR head at
  `7236ce5f773979ec6d56c8942785f25be04a60d9`, verified the pass-13 archive
  delta was documentation-only and truthful, re-ran the focused gates, and
  returned `VERDICT: SATISFIED` with no actionable finding.
- [PR #3040](https://github.com/sifr-lang/sifr/pull/3040) merged the schema-v2
  preview epoch bootstrap implementation as
  `e22a8cfbf058f9657b285370d7d075f9ff0209b3`.
- [x] Add the single protected publication workflow and production site adapter.
- The protected-drill wave adds the deterministic `ga-activation` and `normal`
  index-planning core, a named `protected-drill` suite, and a
  `stable-release-drill` job whose dispatch selects exactly publication,
  rollback, or first-GA coverage through temporary local adapters inside a
  network namespace. The local suite runs all scenarios, including normal,
  site-timeout resume, and stale/credential boundaries. The reusable drill
  accepts no secrets, grants read-only repository permission, and retains
  write-once schema-v2 evidence for 30 days.
- Protected-drill review pass 1 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-protected-drill-review-pass-1-not-satisfied.md`.
  Its six findings are remediated: drill concurrency is isolated, unknown
  modes fail closed, the production site secret remains required, credential
  names share one Python contract checked against the workflow, transition
  defenses have direct tests, and emitted mutation evidence binds the exact
  plan, previous index identity, and proposed index bytes.
- Protected-drill review pass 2 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-protected-drill-review-pass-2-not-satisfied.md`.
  Both findings are remediated: mutation evidence now accepts intentional
  burned-generation gaps while requiring strict monotonicity and validating
  before write, and the durable distribution reference documents the exact
  drill dispatch, concurrency, credential, network, and retention boundaries.
- Protected-drill review pass 3 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-protected-drill-review-pass-3-satisfied.md`.
  It re-ran the focused and combined 60-variant gates, confirmed every prior
  finding remains closed, found no actionable issue, and returned
  `SATISFIED`.
- The authoritative create-PR profile passed with zero blocking failures:
  Python interop 19/19 in 414.69 seconds, consumed Rust interop 10/10, e2e
  131/131, and every crate, runtime, tooling, performance, and guardrail step.
  Its 1058.85-second cold-cache wall time produced only the declared warm-target
  advisory; every enforced per-step budget passed.
- Protected credential-free drill and stable index planning are under review
  in [PR #3041](https://github.com/sifr-lang/sifr/pull/3041).
- Exact PR-head review pass 4 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-protected-drill-review-pass-4-exact-pr-head-satisfied.md`.
  It matched local, remote branch, and PR head at
  `774592acd140747c068bfe6f4752b34006e9664a`, rechecked the complete wave,
  found no actionable issue, and returned `SATISFIED`.
- Frozen-head review pass 5 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-protected-drill-review-pass-5-frozen-pr-head-satisfied.md`.
  It matched local, remote branch, and PR head at
  `a5ffe3704bbdf71616f5edee6f08c9de34c3ac76`, confirmed the pass-4 archive
  delta was documentation-only and truthful, and returned `SATISFIED`.
- [PR #3041](https://github.com/sifr-lang/sifr/pull/3041) merged the protected
  credential-free drill and stable index planning wave as
  `f9837adb105f048ed56624c148ee83ecbd2a3d03`.
- The real main-branch protected drill workflow passed all three
  credential-free modes at exact source
  `476a2983003f9fec74ac15584a576f79495f7482`:
  [publication #30427276373](https://github.com/sifr-lang/sifr/actions/runs/30427276373),
  [first-GA #30427280203](https://github.com/sifr-lang/sifr/actions/runs/30427280203),
  and [rollback #30427342590](https://github.com/sifr-lang/sifr/actions/runs/30427342590).
  Each retained canonical schema-v2 evidence with status `pass`, environment
  `stable-release-drill`, external network `blocked`, and production
  credentials `absent`. The rollback evidence exercised burned-generation and
  site-timeout resume; first-GA exercised incident roll-forward; publication
  exercised GA activation, normal successor, identity, transition, CLI
  producer, and evidence contracts. The first queued rollback dispatch was
  cancelled before execution when a third run replaced the pending
  concurrency slot; the standalone redispatch above passed.
- A fresh current-main replay at exact source
  `1a90170dbe878b60cf644c63d28d3076f31e6320` passed the same three protected
  credential-free modes:
  [publication #30496849280](https://github.com/sifr-lang/sifr/actions/runs/30496849280),
  [first-GA #30496852409](https://github.com/sifr-lang/sifr/actions/runs/30496852409),
  and [rollback #30496911507](https://github.com/sifr-lang/sifr/actions/runs/30496911507).
  Their retained canonical evidence SHA-256 values are respectively
  `3450ca33248e3d846c3dadc092a02e1e6bd5100b8777f095a484f157a79e9c9f`,
  `2e3d6f520279da697becad2a561aed9159c80946c46a3774db81c204298d03ae`,
  and
  `be8b24b4a16c3da0e0ff3288327b9b573274e6adf0478e3a6eb8d31d5ce91822`.
  Each validated with status `pass`, environment `stable-release-drill`,
  external network `blocked`, and production credentials `absent`. The
  initially queued rollback
  [#30496850896](https://github.com/sifr-lang/sifr/actions/runs/30496850896)
  was cancelled when the newer pending dispatch replaced GitHub's single
  concurrency slot; the standalone rollback redispatch above succeeded.
- [PR #3056](https://github.com/sifr-lang/sifr/pull/3056) merged the protected
  drill evidence as
  `edb7d302a7b145787b1762180654671637de0123`. Its exact-head Opus review is
  archived at
  `plans/reviews/archive/phase-40-protected-drill-evidence-review-pass-1-satisfied.md`;
  it independently reconciled all four workflow runs, the three retained
  canonical evidence artifacts, the cancelled pending rollback dispatch, and
  the successful standalone redispatch, then returned `VERDICT: SATISFIED`
  with no actionable finding.
- The documentation-only drill closeout
  [PR #3057](https://github.com/sifr-lang/sifr/pull/3057) merged as
  `649334330ce4f9c682b5aa8453ddad6ada737d40`. Its exact-head review is
  archived at
  `plans/reviews/archive/phase-40-protected-drill-closeout-review-pass-1-satisfied.md`;
  it returned `SATISFIED` with no actionable finding.
- The pre-exception approval-boundary audit is archived at
  `plans/reviews/archive/phase-40-protected-approval-boundary-audit-pass-1-external-reviewer-required.md`.
  It correctly proved that no compliant single-maintainer path existed under
  the then-frozen distinct-reviewer policy. The later user direction explicitly
  authorized the narrow, expiring exception recorded below; the audit remains
  historical evidence rather than a current blocker.
- The user directed Phase 40 to proceed without a second human reviewer while
  the repository has one maintainer. The in-review single-maintainer approval
  exception keeps a real `stable-release` environment approval mandatory,
  rejects admin bypass, expires on 2026-08-27, and is limited to the two
  bootstrap stages plus first GA. Canonical bootstrap and stable sign-off
  evidence record the approval mode and exact waiver digest; normal and
  incident operations remain ineligible. Restoration of a distinct reviewer is
  isolated in the non-blocking ad hoc follow-up rather than weakening future
  stable releases.
- The exception is under review in
  [PR #3060](https://github.com/sifr-lang/sifr/pull/3060). Claude Opus review
  pass 1 is archived at
  `plans/reviews/archive/phase-40-single-maintainer-approval-review-pass-1-not-satisfied.md`.
  Its five findings are remediated before pass 2: the branch is rebased on
  current `main` without reverting corrected issue links; bootstrap and GA pin
  the canonical waiver digest; approval resolution prefers a real distinct
  reviewer and derives the retained mode from the actual approval set before
  publication; stable sign-off binds the initiator and rejects mode/approver
  mismatch; and direct tests validate the real canonical waiver, its expiry,
  all three allowed operations, all forbidden stable/incident operations, and
  the incident workflow's absence of waiver arguments.
- Post-remediation validation passed the complete distribution area 125/125,
  all focused governance and workflow contracts, the file-size guardrail, and
  every authoritative create-PR blocking lane before Python interop. The
  unchanged interop lane reproduced the separately indexed
  `readonly-check-doctor` 120-second timeout in both an isolated replay and the
  profile, then hit a 180-second `binding-authoring` timeout under concurrent
  worktree compilation while later interop cases passed. This host-variance
  evidence is recorded in
  [`adhoc_performance_budget_host_variance.md`](./adhoc_performance_budget_host_variance.md);
  no timeout, threshold, waiver, baseline, or Phase 40 source was changed.
- Exact PR-head Opus review pass 2 is archived at
  `plans/reviews/archive/phase-40-single-maintainer-approval-review-pass-2-satisfied.md`.
  At remote head `2b2f613fd522184c65ce1cc4bce755406ac8b360`,
  it independently reran the complete distribution area 125/125, the release
  runner self-test, and the file-size guardrail; verified all five pass-1
  findings closed; found the PR cleanly based on current `main`; and returned
  `SATISFIED` with no actionable finding.
- Final exact-head Opus review pass 3 is archived at
  `plans/reviews/archive/phase-40-single-maintainer-approval-review-pass-3-final-satisfied.md`.
  It reviewed remote head `36a71dc467ae1bc2a82c7bce33348edec5d7dbc5`,
  proved the pass-2 archive/ledger delta was tracking-only, independently
  reran distribution 125/125, the runner self-test, and the file-size
  guardrail, spot-reverified the full waiver boundary, and returned
  `SATISFIED` with zero actionable finding.
- [PR #3060](https://github.com/sifr-lang/sifr/pull/3060) merged the temporary
  single-maintainer approval boundary as
  `94a5fec67b7bef51cae0034c84386c57d9ff1785`. The live `stable-release`
  environment now requires reviewer `yaseralnajjar`, allows the reviewed
  expiring self-approval path, and has admin bypass disabled.
- Protected bootstrap-alpha run
  [`30442990238`](https://github.com/sifr-lang/sifr/actions/runs/30442990238)
  succeeded for `0.1.0-alpha.2`. Its write-once evidence SHA-256 is
  `e6ee4f9ac7808799838ec2653b81c5b8533b8bde094f1fcb3df82306bef2cd8e`
  and retains source `94a5fec67b7b`, approver/initiator `yaseralnajjar`,
  waiver SHA-256 `b9630cc060ca281946da76a9cb9bc67564759c8d5446b6a33157a7d138080008`,
  prepare SHA-256
  `bfca99484db957557f2c569db4a28bf395149c64714d371f156d689444bd5477`,
  and all nine published asset identities.
- Protected bootstrap-index run
  [`30443929353`](https://github.com/sifr-lang/sifr/actions/runs/30443929353)
  published `0.1.0-beta.15`, reserved immutable
  `channels-generation-1.json`, and activated canonical preview generation 1
  at SHA-256
  `04edacb8ef64706e2285ec241fc23f7d5f2b80199bb1c2bac5889c48e8485964`.
  The exact correlated website run
  [`30445065348`](https://github.com/sifr-lang/sifr-website/actions/runs/30445065348)
  passed all identity, generation, build, dispatcher, and pre-deploy checks,
  then failed because the `sifr.sh-production` environment had neither
  `CLOUDFLARE_API_TOKEN` nor `CLOUDFLARE_ACCOUNT_ID`. The final bootstrap
  evidence was correctly not uploaded.
- The live failure also exposed that the one-time bootstrap path lacked the
  phase-required post-index resume. The focused recovery milestone adds a
  credential-free exact-state prepare and a protected site-only completion
  workflow. It binds the original failed mutation/site runs, approvals,
  prepare/plan/site facts, public alpha/beta releases, and exact generation-1
  bytes; prohibits every release/index mutation; reruns public smoke; and
  retains final evidence from the original mutation identities. The external
  credential prerequisite is tracked in
  [`ad-hoc-sifr-site-production-credentials.md`](./ad-hoc-sifr-site-production-credentials.md).
- The recovery implementation reproduced the live failed-run prepare, release
  records, generation-1 bytes, plan, dispatchers, and site facts from public
  custody, then passed the complete distribution area 125/125, workflow/YAML
  contracts, schema mutations, file-size and diff guardrails. Its authoritative
  create-PR profile passed every pre-interop blocking lane and then reproduced
  only the already indexed `readonly-check-doctor` 120-second host timeout; no
  interop source, timeout, threshold, or waiver changed.
- The exact recovery dispatch ledger is publication attempt
  `30443929353-1`, failed site run `30445065348`, source
  `94a5fec67b7bef51cae0034c84386c57d9ff1785`, plan
  `979d469cb21675e4df6943220deb0f6453d4d1f8c3fb2056c108b8b7ec98f43f`,
  generation/index `1` /
  `04edacb8ef64706e2285ec241fc23f7d5f2b80199bb1c2bac5889c48e8485964`,
  site base `ff472f2af59255c8031b1a6f9b9b294c4b820496`, dispatchers index
  `93a40ff1224a038402ed4952d968404ee503368d368b43166809db86ec562cc4`,
  stable
  `4dc2fde3dcc5deb8aa390900c3e8ef606e9ef46f6c1c3b2471a1caa3c29a73ae`,
  alpha
  `afbe013b87273e8b7aa0f676ff658ad82159434cfe5339369b1ae9ad63a69bac`,
  beta
  `5885601276c1aa157146b5262ea505ba57c3081513dbe4338b09df2477d35481`,
  default `beta`, publication facts
  `f3f03dd9366d61269d83f06d43c7d29b89edbe756207a40af0895ddb9ccf8dc1`,
  and stable site facts `none`. The original summary digest is
  `f45c012c17d2908bc2ef227f202e1037343c63d1f1881ca7913f22628f62a086`.
  Its exact canonical bytes are now durable under
  `plans/releases/schema-bootstrap-recovery/`; recovery no longer depends on
  the source artifact that expires `2026-08-28T10:46:13Z`. The approval waiver
  expires `2026-08-27T00:00:00Z`, which remains the recovery completion
  deadline unless a distinct reviewer is configured. Recovery must complete
  and retain generation-1 bootstrap evidence before `ga-activation` is
  dispatched; activation would advance the live index to generation 2 and
  intentionally make the one-time recovery precondition fail. The qualified
  GA prepare has the earlier `2026-08-21T02:17:30Z` start deadline recorded
  above.
- Recovery review pass 1 is archived at
  `plans/reviews/archive/phase-40-schema-bootstrap-recovery-review-pass-1-not-satisfied.md`.
  Its `NOT SATISFIED` findings were remediated by embedding validated recovery
  run/approval/site provenance in final evidence, retaining the exact original
  prepare summary canonically in-repository, recording every dispatch identity
  and deadline, covering all partial attested-identity combinations, and
  pinning the original exact-bytes versus recovery-only attestation boundary
  in the workflow contract.
- Exact implementation-head recovery review pass 2 is archived at
  `plans/reviews/archive/phase-40-schema-bootstrap-recovery-review-pass-2-satisfied.md`.
  At pushed PR head
  `ddcd7e3d656e39a2b00727a7ce6ac775fa823f1e`, Opus independently reran
  the bootstrap self-test, workflow contract, 67-variant distribution `full`
  suite, YAML/shell and file-size checks; compared the durable summary
  byte-for-byte with the original artifact; checked the live failed run log
  and generation-1 assets; found every pass-1 finding closed; and returned
  `SATISFIED` with zero actionable finding.
- Final exact-head review pass 3 is archived at
  `plans/reviews/archive/phase-40-schema-bootstrap-recovery-review-pass-3-not-satisfied.md`.
  It confirmed the post-pass-2 commit was tracking-only and the full recovery
  implementation remained clean, then found that the new ledger entry
  inaccurately called the filtered 67-variant `full` suite the complete area.
  The wording above now names the suite exactly; the reviewer also independently
  ran the unfiltered complete distribution area at 125/125.
- Final exact-PR-head review pass 4 is archived at
  `plans/reviews/archive/phase-40-schema-bootstrap-recovery-review-pass-4-final-satisfied.md`.
  At local, remote, and PR head
  `cd92c820611d0f7c3fcd657ccdd46b05283057b7`, Opus verified pass 3's sole
  ledger finding closed, independently reran both the filtered 67-variant
  `full` suite and complete unfiltered distribution area at 125/125, rechecked
  the full recovery boundary and live generation-1 state, and returned
  `SATISFIED` with zero actionable finding.
- [PR #3061](https://github.com/sifr-lang/sifr/pull/3061) merged the reviewed
  schema-bootstrap recovery implementation as
  `3ce906c8445569039ebd762de0f346587464742a`.
- The tracking-only [PR #3062](https://github.com/sifr-lang/sifr/pull/3062)
  merged as `637dd0c0b06ecb7d5e5d7e2fa26cbb7c094128b1`. Exact-head Opus review is
  archived at
  `plans/reviews/archive/phase-40-schema-bootstrap-recovery-tracking-review-pass-1-satisfied.md`;
  it verified remote head
  `14b66c82f49ad58c4aaa79df5a79f9b78c800b59`, the tracking-only two-file
  diff, spot-checked recovery citations, digests, and live run state, the
  complete distribution area at 125/125, and returned `SATISFIED` with no
  actionable finding.
- The recovery-tracking closeout
  [PR #3063](https://github.com/sifr-lang/sifr/pull/3063) merged as
  `cef1c55bdd63215704d8564e764fe876508b4b8b`. Its exact-head Opus reviews
  are archived at
  `plans/reviews/archive/phase-40-bootstrap-recovery-closeout-review-pass-1-not-satisfied.md`
  and
  `plans/reviews/archive/phase-40-bootstrap-recovery-closeout-review-pass-2-satisfied.md`.
  Pass 1 found one over-broad review-attribution sentence; pass 2 verified the
  precise tracking-only correction at head
  `483a0c563c1ea451446d6acb06a4bcfa53b928f9` and returned `SATISFIED`.
- The next protected-publication wave adds a credential-free stable prepare
  path for `ga-activation` and `normal`. It binds an exact evidence commit,
  canonical candidate directory and plan digest, a separate clean exact source
  checkout, at least seven full days of qualification lifetime, all 20
  transported artifact identities from six write-once qualification uploads,
  Rust/documentation/release-note evidence, Marketplace VSIX bytes, live index
  identity, site base commit, and a deterministic proposed stable mutation.
  The reusable workflow remains read-only and secret-free and retains the
  reviewer-visible summary for 30 days. The named `stable-prepare` suite is
  selected by merge, nightly, and release profiles and required by the release
  report.
- Stable-prepare review pass 1 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-prepare-review-pass-1-satisfied.md`.
  Opus re-read every file after the in-review hardening, reproduced the
  stable-prepare, governance, runner, combined 60-variant distribution,
  coverage, workflow-contract, diff, and file-size gates, and found no
  actionable correctness, provenance, extraction, schema, permission, or
  compatibility issue. Its verdict binds the final reviewed workflow,
  extractor, artifact-index, and prepare-core byte identities recorded in the
  archive.
- The authoritative `scripts/run_all_tests.sh --profile create-pr` gate passed
  for the stable-prepare implementation, including every blocking lane and
  131/131 E2E fixtures (`report_signature=7c39b8c1dd4fec7c`). The only
  advisory was the already-indexed non-blocking warm wall-time variance; every
  enforced step budget passed.
- Exact PR-head stable-prepare review pass 2 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-prepare-review-pass-2-exact-pr-head-satisfied.md`.
  It independently matched local, remote, and [PR #3043](https://github.com/sifr-lang/sifr/pull/3043)
  at `a671c913116e6fa30073d6220abe639154b51e72`, reproduced the focused
  gates, found no actionable issue, and returned `SATISFIED`.
- Its non-blocking observations were hardened before the frozen-head review:
  each governed upload now has one canonical expiry shared by all transported
  entries and matched exactly to the authoritative GitHub artifact API,
  the stable-prepare schema fixture passes the semantic validator, and the
  extraction self-test pins pre-write rejection of uncompressed byte-count
  drift. The focused 60-variant distribution run, stable-prepare 6/6,
  governance 14/14, runner foundation, coverage readiness, schema/workflow
  contracts, Ruff, diff, and file-size guardrails pass after these changes.
- Frozen PR-head stable-prepare review pass 3 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-prepare-review-pass-3-frozen-pr-head-satisfied.md`.
  It matched local, remote, and PR #3043 at
  `55c6d960c4ea29b7b945df88d72573a6008c9651`, independently rechecked the
  complete wave and all post-pass-2 hardening, reproduced the focused gates,
  found no actionable issue, and returned `SATISFIED`.
- Final exact-head review pass 4 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-prepare-review-pass-4-final-pr-head-satisfied.md`.
  It matched local, remote, and PR #3043 at
  `8d81040e597cb3d64f90788fb2ce2e822eb236f1`, verified the pass-3 archive
  delta was documentation-only and truthful, reproduced 60 distribution
  variants plus the focused gates, and returned `SATISFIED`.
- [PR #3043](https://github.com/sifr-lang/sifr/pull/3043) merged the protected
  stable-prepare wave as `da7c38fb15dbebe11b1e9be943f4d080b8e7bafc`.
- The protected publish-primitives wave removes the operator-selected
  generation from stable prepare, allocates after every canonical retained
  snapshot, and requires the live index to equal its retained snapshot. It
  centralizes exact-ID qualification artifact refetch with authoritative
  run/attempt/source/name/expiry verification and safe bounded extraction, then
  adds a protected revalidation command that recomputes and byte-compares the
  complete reviewer-visible prepare summary before any production mutation.
  The named `stable-publish-primitives` suite is selected by merge, nightly,
  and release and is required by release-report custody.
- Publish-primitives review pass 1 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publish-primitives-review-pass-1-not-satisfied.md`.
  Its `NOT SATISFIED` findings are remediated: durable docs now describe the
  unwired command rather than a nonexistent publish integration; mutation
  coverage pins summary byte inequality, generation names/payloads/live bytes,
  run/attempt/repository/source and transported content; the protected-input
  gate remains an early defense-in-depth check subsumed by the final byte
  equality check. Preview still requires alpha/beta, and unreadable summaries
  produce governed diagnostics without a hash/read race. The review's hardening
  suggestions are also applied: revalidation rejects a generation burned after
  prepare, history enumeration uses paginated release assets, ZIP download is
  streamed to an authoritative API-size boundary, and the new scripts use one
  canonical governance package identity. The unchanged legacy distribution
  scripts still use their earlier top-level import convention.
- Publish-primitives review pass 2 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publish-primitives-review-pass-2-not-satisfied.md`.
  Its `NOT SATISFIED` findings are remediated: the ledger no longer overstates
  independent coverage of the defense-in-depth protected-input check; the
  artifact refetch self-test now rejects both truncated and overlong downloads,
  expired artifacts, wrong artifact-run custody, and symlinked output parents;
  and streamed downloads direct stderr to a file so a full stderr pipe cannot
  deadlock the protected release path. The six-upload grouping check is retained
  as defense in depth after the semantic qualification-index validator.
- Publish-primitives review pass 3 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publish-primitives-review-pass-3-satisfied.md`.
  It re-ran the focused suites and mutation-tested the wave, verified every
  prior finding was closed, found no actionable correctness, security, test,
  workflow, or documentation issue, and returned `SATISFIED`.
- The authoritative `scripts/run_all_tests.sh --profile create-pr` gate passes
  for the publish-primitives wave: all blocking steps pass, the e2e suite is
  131/131, and the only advisory is the nonblocking warm wall-time target
  (`1033.83s`).
- Frozen PR-head publish-primitives review pass 4 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publish-primitives-review-pass-4-frozen-pr-head-satisfied.md`.
  It matched local, remote, and [PR #3044](https://github.com/sifr-lang/sifr/pull/3044)
  at `f355a2b0a40a4ab644f711d0e6fd6d2aa63bf19a`, independently reproduced
  the full distribution gate, verified the complete wave and earlier review
  closures, found no actionable issue, and returned `SATISFIED`.
- Final exact-head publish-primitives review pass 5 is satisfied and archived
  at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publish-primitives-review-pass-5-final-pr-head-satisfied.md`.
  It matched local, remote, and PR #3044 at
  `338f318d47b3b6b2459a7fcfac9a05886273e459`, verified that the final review
  archive was the only delta after the frozen review, and returned
  `SATISFIED`.
- [PR #3044](https://github.com/sifr-lang/sifr/pull/3044) merged the protected
  stable publication-primitives wave as
  `47c837a4b7f9d4a06322b5fbb0e6b65255dda8c0`.
- The stable production-wiring wave enables `ga-activation` and `normal` in the
  existing single protected `publish` job. It makes protected resume
  distinguish a pending proposal from an already-activated exact release,
  re-fetches exact qualification uploads, reproduces the reviewer-visible
  summary before any publication and again immediately before index
  reservation, and stages the exact 20 qualified artifacts plus approved plan.
  Its adapters create or byte-verify the write-once GitHub release, publish the
  recorded VSIX only when the exact Marketplace version is absent, verify the
  raw Gallery VSIX, reserve and activate the governed generation, dispatch and
  poll the pinned site workflow, run public install/update/asset smoke, and
  retain generation-specific site facts and release sign-off without
  overwriting them. Sign-off now records the correlated site run and deployed
  commit. Each protected run retains a distinct immutable sign-off asset, so a
  later resume adds attempt evidence without rewriting an earlier sign-off.
  The named `stable-publication` suite is selected by merge, nightly,
  and release and is required by release-report custody.
- Production-wiring review pass 1 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publication-wiring-review-pass-1-not-satisfied.md`.
  Its `NOT SATISFIED` findings are remediated: prepare and publish now execute
  the same governance code from the exact workflow commit and stable mutation
  refuses any ref except protected main HEAD; Node 22 plus the candidate's
  exact extension lockfile provision the local `vsce` binary without lifecycle
  scripts or publication secrets; sign-off is immutable per run/attempt;
  Marketplace raw-byte drift and public-smoke execution have negative/positive
  coverage; and governance asset inventory is paginated.
- Production-wiring review pass 2 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publication-wiring-review-pass-2-not-satisfied.md`.
  It verified every pass-1 finding was closed, then identified the missing
  protected-main ancestry proof for the stable candidate source and evidence
  commits. The orchestrator now checks both commits with
  `git merge-base --is-ancestor` against the freshly fetched protected main
  head before any network mutation, and an executed fake-git test rejects an
  unmerged candidate.
- Production-wiring review pass 3 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publication-wiring-review-pass-3-not-satisfied.md`.
  It verified both earlier review rounds were closed, then requested tighter
  process-secret scoping and direct negative schema parity coverage. The
  orchestrator now unexports site/Marketplace secrets and exposes them only to
  their intended commands, clears all publication tokens for Marketplace's
  unrelated credentials and for dispatcher/installed-binary smoke, and schema
  self-tests reject activated-initial prepare plus a noncanonical site
  repository in stable sign-off.
- Production-wiring review pass 4 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publication-wiring-review-pass-4-satisfied.md`.
  It independently reproduced distribution `full` at 63/63 and the whole area
  at 120/120, verified every prior review finding and the complete mutation
  ordering/resume/secret/schema/profile surface, found no actionable issue, and
  returned `SATISFIED`.
- Exact [PR #3045](https://github.com/sifr-lang/sifr/pull/3045) head review
  pass 5 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publication-wiring-review-pass-5-exact-pr-head-satisfied.md`.
  It matched local, remote, and PR head at
  `a5c9a2ce873b6a3f65b142c803bca61b191abbbf`, independently reproduced
  the 63-variant focused distribution run and all workflow/schema/coverage,
  parsing, diff, and file-size gates, reverified every earlier closure and the
  complete production mutation ordering, found no actionable issue, and
  returned `SATISFIED`.
- Final exact-head review pass 6 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-stable-publication-wiring-review-pass-6-final-exact-head-satisfied.md`.
  It matched local, remote, and PR head at
  `d2b919eb8a5a32ede375ab7f9e03f18431a4e506`, verified the only
  post-pass-5 delta was the truthful review archive and ledger entry,
  independently reran the focused suites and guardrails, found no actionable
  issue, and returned `SATISFIED`.
- Focused production-wiring validation passes: stable prepare 7/7, stable
  publication primitives 4/4, stable publication 9/9, governance 14/14,
  runner self-tests, workflow/schema contracts, coverage readiness, and the
  combined distribution run at 63 variants with zero failures. File-size
  guardrails pass with the single publication workflow at 899 lines.
- The production-wiring create-PR profile passed coverage-matrix checks, core
  guardrails, diagnostic rules, and CPython differential checks, then passed
  18 of 19 Python interop variants. Its sole failure was a repeated
  120-second timeout in the pre-existing `readonly-check-doctor` capability;
  the exact isolated suite reproduced the same timeout. This unrelated
  follow-up is closed in
  [`python-interop-readonly-inspection-timeout.md`](../archive/python-interop-readonly-inspection-timeout.md)
  through [PR #3110](https://github.com/sifr-lang/sifr/pull/3110), without a
  timeout change, waiver, suppression, or Phase 40 code change.
- The incident production-wiring wave extends that same protected job and
  read-only prepare boundary to `rollback` and `incident-roll-forward`. It
  binds exact incident request, withdrawal evidence, affected and
  successor/target plan bytes, protected-main ancestry, live index and retained
  generations, and—only for roll-forward—the complete stable candidate
  prepare. The protected path retains request/generation evidence write-once,
  performs the sole atomic `channels.json` replacement, reconciles the pinned
  site, verifies stable install/update/Marketplace/withdrawal documentation
  and both recovery paths, and emits exact stable/incident sign-offs.
- Incident production-wiring review passes 1 and 2 are retained at
  `plans/reviews/archive/phase-40-milestone-40-5-incident-publication-wiring-review-pass-{1,2}-not-satisfied.md`.
  Both returned `CHANGES_REQUIRED`. Their findings are remediated: tracked
  checkout bytes are `HEAD`-bound without false sibling-checkout dirtiness;
  rollback and roll-forward have executed end-to-end and drift/ancestry
  negatives; schema/runtime and release-signoff cross-bindings are direct;
  public smoke includes a non-empty withdrawal; the demo invokes the extracted
  public-smoke suite; secret scrubbing, diagnostics, and workflow decomposition
  are restored; and durable architecture/pipeline/ledger truth is updated here.
- The facts-driven public stable page landed through
  [sifr-website PR #16](https://github.com/sifr-lang/sifr-website/pull/16) as
  `ff472f2af59255c8031b1a6f9b9b294c4b820496`. Site review passes 1 and 2
  found preview breakage, unpinned runtime, missing facts custody, shell
  precedence, post-GA preview persistence, and contract/test gaps. Pass 3
  closed every actionable finding and returned `SATISFIED`; exact PR-head pass
  4 independently matched `03a407933ad054309cef0d8408043012970af710`,
  reproduced the renderer/build/provenance/contracts, and returned
  `SATISFIED`.
- Immutable site tag `sifr-release-site-stable-facts` resolves to that merge
  commit. Active no-bypass tag ruleset `19899766` forbids update and deletion
  at attested revision `2026-07-28T13:22:41.496Z`; the pinned workflow digest
  is `a9360c82395f6e9d9822f201e56cc0f2eabab1bacda01c31e4e9f22d0202b3af`.
  The live identity helper and cross-repository fixture both verify the exact
  tag, commit, workflow bytes, ruleset, 13 dispatch inputs, renderer, route,
  and preview-absence/active-facts behavior.
- Focused incident-publication validation passes 5/5, public stable smoke 2/2,
  executed working-client/out-of-band rollback and roll-forward recovery 2/2,
  the site renderer/build and active-only sitemap paths, every publication/site
  workflow contract, individual distribution shell parsing, Python/YAML
  parsing, and live immutable site identity verification. Incident review pass
  3 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-incident-publication-wiring-review-pass-3-not-satisfied.md`.
  It found the production recovery adapter used the nonexistent
  `install-receipt.json` path and incorrectly required canonical installer
  receipt bytes. The adapter now validates the real pretty-printed
  `install.json`, scrubs production credentials internally as well as at its
  call site, and is executed for rollback, roll-forward, downgrade consent,
  out-of-band installation, version/receipt convergence, and receipt drift.
  The same remediation adds an untracked-plan forgery negative and moves
  stable publication fixture helpers out of private self-test coupling.
  The complete focused distribution selection passes 68 variants with zero
  failures after that remediation.
- Incident review pass 4 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-incident-publication-wiring-review-pass-4-not-satisfied.md`.
  Its three actionable findings are remediated: executed roll-forward recovery
  now uses a strictly newer stable successor while the fake client rejects
  every non-forced downgrade; schema-valid receipt-version drift and binary
  version drift reach and fail the production adapter's own convergence
  assertions; and the capability demo invokes the recovery adapter suite.
  The smaller findings are closed as well: explicit dispatch defaults and one
  named stable-publication predicate are restored, the site renderer digest
  and exact public labels are pinned, incident IDs use the schema-equivalent
  runtime validator, the site dispatcher advertises only its two supported
  defaults, and the local roll-forward fixture retains a validated stable
  release sign-off asset whose exact digest is bound by the incident sign-off.
  The new fixture sign-off responsibility is isolated in its own focused module
  so all hand-maintained files remain below 900 lines.
- Incident review pass 5 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-incident-publication-wiring-review-pass-5-not-satisfied.md`.
  It verified every pass-4 finding closed and independently reproduced the
  full 125-variant distribution area, then found one pre-mutation rollback
  provenance gap. Staging now requires the rollback target and affected plan
  to agree on all four site dispatcher digests before creating the staged
  output; an executed negative binds a deliberately mismatched target plan into
  an otherwise valid prepare summary, verifies the precise rejection, and
  proves no output was created. The site contract also cross-checks its public
  rendered labels against the verifier's single canonical label tuple, and
  focused negative schema contracts exercise all incident-prepare conditionals
  plus the incident-mutation generation floor. Those negative contracts are
  decomposed into a focused module to preserve the file-size guardrail.
- Incident review pass 6 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-incident-publication-wiring-review-pass-6-satisfied.md`.
  It independently reproduced the full distribution area at 125 variants with
  zero failures, the capability demo, all parsing and file-size guardrails,
  and every pass-1 through pass-5 closure. It verified the new rollback
  dispatcher-provenance gate runs before staging output and every mutation,
  verified the negative reaches that precise diagnostic with no output, found
  no actionable issue, and returned `VERDICT: SATISFIED`.
- The authoritative `scripts/run_all_tests.sh --profile create-pr` gate passed
  every blocking step: coverage and maintainability guardrails, 19/19
  Python-interop variants, the required existing Rust-consumption suites at
  10/10, frontend/tooling/performance checks, generated-code quality, crate
  tests, 28 runtime-platform variants, and 131/131 E2E fixtures with
  `report_signature=7c39b8c1dd4fec7c`. Both Python-interop and E2E completed
  inside their 600-second blocking budgets. The 1116-second cold-cache wall
  time exceeded only the advisory warm target; no budget, waiver, or
  validation policy changed.
- Exact [PR #3047](https://github.com/sifr-lang/sifr/pull/3047) head review
  pass 7 is archived at
  `plans/reviews/archive/phase-40-milestone-40-5-incident-publication-wiring-review-pass-7-exact-pr-head-not-satisfied.md`.
  It matched local, remote, and PR head at
  `8776b4dbbec3d4b342c937dd1a6b4effaabca5aa`, reverified all earlier
  closures and full validation, then found one malformed-input diagnostic
  defect: an incident roll-forward stable-prepare summary without its required
  `incident` binding raised raw `KeyError`. Runtime required-key construction
  now mirrors the schema, both conditional directions have runtime and schema
  negatives, and the operator-facing CLI negative proves exit 2 with the
  governed missing-field diagnostic and no traceback.
- Exact PR-head review pass 8 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-incident-publication-wiring-review-pass-8-exact-pr-head-satisfied.md`.
  It matched local, remote, and PR head at
  `341b312f50de61c549f1bde01a6676f248231d02`, independently reproduced
  the missing/forbidden incident-binding schema and runtime cases plus both
  operator-facing validator kinds, reran the focused publication/recovery
  suites and guardrails, found no actionable finding, and returned
  `VERDICT: SATISFIED`.
- Final exact PR-head review pass 9 is satisfied and archived at
  `plans/reviews/archive/phase-40-milestone-40-5-incident-publication-wiring-review-pass-9-final-exact-pr-head-satisfied.md`.
  It matched local, remote, and PR head at
  `dabdfec856b1e9a31ea5f95201de84c7cb70402c`, verified that the pass-8
  archive delta was documentation-only and faithful, re-audited the full
  incident-publication implementation, independently reproduced 68 focused
  variants with zero failures plus the malformed-input CLI matrix, found no
  actionable issue, and returned `VERDICT: SATISFIED`.
- [x] Merge the protected rollback and incident roll-forward production wiring.
- [PR #3047](https://github.com/sifr-lang/sifr/pull/3047) merged the protected
  rollback and incident roll-forward production wiring as
  `8a23f90869a68438a7b4ae3b8f9623531d1ce68f`.
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
    `plans/issues/active/adhoc_performance_budget_host_variance.md`

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

Coordination record (2026-08-19): Phase 40 releases the shared verification
profiles, runner, workflows, and release-governance paths from base
`9caed42242d017ed4ebff84332df0c201f6b403b` to compatibility-removal Item 13.
Item 13 owns their canonical profile-v2 migration and must preserve the Phase
40 release suites, evidence custody, and publication contracts. Phase 40
resumes from the merged Item 13 result.

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
  `plans/issues/active/adhoc_performance_budget_host_variance.md` and is not a Phase
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

### canonical_candidate_evidence_remediation

- Final qualification-evidence review pass 1 is archived at
  `plans/reviews/archive/phase-40-final-qualification-evidence-review-pass-1-not-satisfied.md`.
  The reviewer confirmed the transport and retention evidence, then found that
  pretty-printed Rust results and stable-support claims could not satisfy the
  candidate custody contract's canonical-JSON requirement. It also found that
  the staged Rust report had to be the exact release-profile result rather than
  a standalone rerun.
- Release-report generation now canonicalizes that exact release-run Rust
  result before hashing it. Stable-support claims are staged deterministically
  from the exact source checkout into custody; the staged bytes must be
  canonical JSON and exactly equal the source claims' canonical
  representation. The release evidence writer rejects a wrong-area Rust
  result, while the writer and planner reject applicable noncanonical,
  drifted, symlinked, and duplicate-key inputs.
- Reviewer passes 2 and 3 are archived at
  `plans/reviews/archive/phase-40-canonical-candidate-evidence-review-pass-2-satisfied.md`
  and
  `plans/reviews/archive/phase-40-canonical-candidate-evidence-review-pass-3-satisfied.md`.
  Both returned `SATISFIED`; pass 3 independently reran the complete
  distribution area (125/125) and found no blocking issue.
- The authoritative create-PR profile passed at implementation commit
  `8048de4343`, including Python interop 19/19, consumed Rust interop 10/10,
  and E2E 131/131 (`report_signature=7c39b8c1dd4fec7c`). A preceding cold
  attempt hit only the already indexed `PERF-HOST` timeout in
  `readonly-check-doctor`; an isolated replay passed 1/1 in 160.114 seconds.
- Main-repository [PR #3051](https://github.com/sifr-lang/sifr/pull/3051)
  carries the focused Phase 40 remediation from implementation commit
  `8048de4343`. Its exact pushed-head review follows after this tracking update.
- Exact PR-head reviewer pass 4 is archived at
  `plans/reviews/archive/phase-40-canonical-candidate-evidence-review-pass-4-pr-head.md`.
  It independently verified remote head `1841576ce`, reran the complete
  distribution area 125/125 and the isolated `readonly-check-doctor` case 1/1,
  closed blockers A-C, and returned `SATISFIED`. Its four low-severity
  observations are also resolved: release-result and source-claims symlinks
  now fail closed, operator documentation names the exact Rust result copy,
  and this ledger precisely distinguishes source JSON from canonical staged
  bytes. The focused qualification/custody selection, runner self-tests, both
  guardrails, and diff checks pass after that hardening.
- Final exact-head reviewer pass 5 is archived at
  `plans/reviews/archive/phase-40-canonical-candidate-evidence-review-pass-5-final-pr-head.md`.
  It independently reran the complete distribution area 125/125 and runner
  self-tests at remote head `90cda61b9`, verified all four pass-4 observations
  closed with discriminating mutation coverage, and returned `SATISFIED` with
  no actionable finding.
- Main-repository [PR #3051](https://github.com/sifr-lang/sifr/pull/3051)
  merged as `7034c4c69bf3fa7e2c36ddc002f6389d6f3511a9`.
- The documentation-only closeout
  [PR #3052](https://github.com/sifr-lang/sifr/pull/3052) merged as
  `c9d611fb7c7c5d05421d784d53a2b78c1a7dcae9`. Its independent Opus review is
  archived at
  `plans/reviews/archive/phase-40-canonical-evidence-closeout-review-pass-1-satisfied.md`;
  it returned `SATISFIED` with no actionable finding.
- Fresh exact-source qualification run
  [#30416219284](https://github.com/sifr-lang/sifr/actions/runs/30416219284)
  passed at source `c9d611fb7c7c5d05421d784d53a2b78c1a7dcae9`.
  Exact-ID custody replay verified six candidate uploads and all 20 indexed
  payloads (533,998,429 bytes) by name, size, and SHA-256. The canonical index
  SHA-256 is
  `503f4fcc0dcf4843e0476fbbd1aaa02994c431fac3e7aebb89fb5565bba04703`.
- Documentation qualification and canonical source-derived support-claims
  staging passed on that same clean source. The cold local release attempt
  exposed only Python-environment initialization order; the unchanged warm
  attempt passed Python interop 25/25, consumed Rust interop 10/10, developer
  tooling 48/48, and documentation 2/2 before reproducing indexed
  `PERF-HOST` median variance under concurrent load from another checkout.
  No threshold, baseline, waiver, profile selection, or interop implementation
  changed. Exact commands and custody paths are archived in
  `plans/reviews/archive/phase-40-final-exact-source-qualification-evidence.md`.
- After the competing checkout completed, the unchanged isolated performance
  suite passed 8/8. An earlier fresh-parent authoritative release profile then
  passed all 24 lane steps in 7,610.91 seconds: generated-code release-full,
  all crate suites, Python interop 25/25, consumed Rust interop 10/10, 674 E2E
  cases, and 290 hardening variants all had zero blocking failures. That
  earlier run's report `release-c9d611fb7c7c-fa3d95c04f8a` has SHA-256
  `faa6844410de98cb6ebe40d740ab6b1edc9aeb176ee0301e4ec181937eeb6e03`;
  its exact canonical Rust result has SHA-256
  `be24b69a7afc0f2f7061657258d9c367946496bf745b3cc17b1cd15e00bba87a`.
  Those report bytes were superseded by the committed candidate bytes recorded
  below.
  The lane report recorded two nonblocking advisories: the already indexed
  warm wall-time target and group skew (largest fixture group 16, median 1).
  Every blocking functional gate passed.
- Exact PR-head qualification-evidence review pass 1 is archived at
  `plans/reviews/archive/phase-40-final-exact-source-qualification-review-pass-1-not-satisfied.md`.
  It independently reconciled the workflow, custody, source, report, Rust
  result, checklist, and public bootstrap boundary, then found the ledger
  incorrectly described the warm wall-time advisory as the only advisory. The
  evidence now records both nonblocking lane advisories: warm wall time and
  fixture-group skew.
- Exact PR-head review pass 2 is satisfied and archived at
  `plans/reviews/archive/phase-40-final-exact-source-qualification-review-pass-2-satisfied.md`.
  It revalidated every preserved digest and count at head `340a40b10`,
  confirmed the pass-1 wording correction, found no actionable issue at any
  severity, and returned `VERDICT: SATISFIED`.
- Frozen-head review pass 3 is satisfied and archived at
  `plans/reviews/archive/phase-40-final-exact-source-qualification-review-pass-3-frozen-head-satisfied.md`.
  It matched local, remote, and PR head
  `677b37dffe525128067f85ff575a26e2a28c399f`, independently rechecked the
  final review archives and exact evidence, found no actionable issue, and
  returned `VERDICT: SATISFIED`.
- [PR #3054](https://github.com/sifr-lang/sifr/pull/3054) merged the final
  exact-source qualification-evidence wave as
  `15c384d958340d7545370f9249d58ac46e202797`.
- The documentation-only final-qualification closeout
  [PR #3055](https://github.com/sifr-lang/sifr/pull/3055) merged as
  `476a2983003f9fec74ac15584a576f79495f7482`. Its exact-head Opus review is
  archived at
  `plans/reviews/archive/phase-40-final-qualification-closeout-review-pass-1-satisfied.md`;
  it returned `VERDICT: SATISFIED` with no actionable finding.
- The final candidate release pass used the same clean exact source and
  unchanged release profile after coordinating an uncontended performance
  window. All 24 blocking lane steps passed, including the full performance
  budget, 674/674 E2E fixtures, the complete crate surface, generated-code
  release checks, and 290 hardening variants. Canonical report
  `release-c9d611fb7c7c-fa3d95c04f8a` has SHA-256
  `e5200229dfdacb2503190d4c3784cdfb3085088f7ad687e49659f54f3a11de98`;
  its exact canonical release-run Rust result has SHA-256
  `95176b5937b4ed0e1c9843ef6c3896969f6336431bc8a0d08350cc2db9b9555e`.
  The 7,976.92-second wall time produced the same two nonblocking advisories:
  the warm wall-time target and fixture-group skew.
- Release-note Opus review pass 1 found seven missing or over-broad public
  claims covering receipt integrity, platform floors, install-time
  verification, install/update commands, generated-Rust scope, Rust-interop
  evidence scope, and preview-channel context. The corrected notes closed all
  findings, and pass 2 returned `SATISFIED`. Both reviews are archived at
  `plans/reviews/archive/phase-40-candidate-release-notes-review-pass-1-not-satisfied.md`
  and
  `plans/reviews/archive/phase-40-candidate-release-notes-review-pass-2-satisfied.md`.
- The canonical candidate plan has SHA-256
  `3e4c7b7c50691eb360b031cebec734ae89bdef253f1a138e706038962b7ded27`.
  Focused qualification plus evidence custody passed 2/2; the unfiltered
  distribution area passed 125/125; canonical plan, release-report, and
  qualification-index validation passed; and the source checkout remained
  clean at `c9d611fb7c7c5d05421d784d53a2b78c1a7dcae9`.
- Exact-head Opus review of candidate
  [PR #3070](https://github.com/sifr-lang/sifr/pull/3070) is archived at
  `plans/reviews/archive/phase-40-canonical-candidate-evidence-pr-3070-review-pass-1-satisfied.md`.
  At head `74c5dd02f1ca692c0fb1f9c8b50004827028cdfb`, it independently
  reproduced all seven artifact digests, custody, source/submodule/toolchain
  provenance, qualification expiry, Rust claims, release-note truthfulness,
  and first-GA semantics, found no blocking issue, and returned `SATISFIED`.
  [PR #3070](https://github.com/sifr-lang/sifr/pull/3070) merged the immutable
  candidate evidence as
  `2e203136f864f132499095d7d68884c3ecc1ec2e`.
- Candidate-evidence tracking
  [PR #3072](https://github.com/sifr-lang/sifr/pull/3072) merged as
  `b5f4d0673e8c77ae9fcebe47f377f9d45ae3c842`. Its two exact-head Opus reviews
  are archived at
  `plans/reviews/archive/phase-40-candidate-evidence-closeout-pr-3072-review-pass-1-satisfied.md`
  and
  `plans/reviews/archive/phase-40-candidate-evidence-closeout-pr-3072-review-pass-2-final-satisfied.md`;
  the second confirmed the pass-1 wording observation was closed and returned
  no actionable finding.
- The pre-GA full-implementation Opus audit is archived at
  `plans/reviews/archive/phase-40-pre-ga-full-implementation-review-pass-1-satisfied.md`.
  It independently reproduced distribution release 125/125, documentation
  2/2, evidence custody, workflow shell parsing, file-size guardrails, all
  pinned custody digests, and the complete recovery/publication boundary at
  source `1a90170dbe878b60cf644c63d28d3076f31e6320`; it returned
  `VERDICT: SATISFIED` with zero actionable implementation findings.
- Pre-GA documentation-closure review pass 2 is archived at
  `plans/reviews/archive/phase-40-pre-ga-full-implementation-review-pass-2-doc-closure-not-satisfied.md`.
  It verified the deadline, recovery ordering, drill evidence, and review
  archives, then found that the first exit-gate rewrite incorrectly attributed
  fresh install and self-update to the four-target qualification matrix.
- Pre-GA documentation-closure review pass 3 is archived at
  `plans/reviews/archive/phase-40-pre-ga-full-implementation-review-pass-3-doc-closure-satisfied.md`.
  It independently mapped every corrected proof clause to the native-target
  qualification, isolated installed-sysroot certification, all-target
  post-publication digest verification, and protected-runner live smoke;
  confirmed all four pass-1 observations closed; and returned
  `VERDICT: SATISFIED` with zero actionable findings.
- Pre-GA audit closeout
  [PR #3073](https://github.com/sifr-lang/sifr/pull/3073) merged exact reviewed
  head `1d4a5c59f5cd15f898f9057edf3e94a9707d2611` as
  `16cc34eb9eccebc183554fc7aa471024eaef7636`. Its exact-head Opus review is
  archived at
  `plans/reviews/archive/phase-40-pre-ga-audit-closeout-pr-3073-review-pass-1-satisfied.md`.
  The reviewer independently reran distribution release 125/125,
  documentation 2/2, file-size and diff guardrails; reverified deadline math,
  recovery ordering, four-way qualification/smoke semantics, all fresh drill
  run/digest identities, and PR #3072 archive fidelity; and returned
  `VERDICT: SATISFIED` with zero actionable findings.
