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

- Frozen qualification scope treats the pinned algorithmic full corpus as the
  separately owned, expiry-bound follow-up linked under `milestone_40_4`;
  release retains its blocking representative subset and taxonomy self-test.
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

- The pinned algorithmic full corpus is intentionally not a Phase 40
  prerequisite. Exact-source release validation reproduced the same 20
  pre-existing failures already preserved in
  [`ad-hoc-algorithmic-full-corpus-preexisting-failures.md`](./ad-hoc-algorithmic-full-corpus-preexisting-failures.md)
  after every preceding gate—including `performance_budget_checks` in `full`
  mode—passed.
  Nightly keeps the full corpus blocking; release qualification keeps the
  representative subset and taxonomy self-test blocking until the ad hoc
  issue restores the full corpus.
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
- The clean exact-source release profile on
  `c17f3c7d1ea1ed97ca125eb7a43344b30cf9413b` passed every lane through
  `performance_budget_checks` in `full` mode before reproducing exactly the 20
  previously preserved failures among 412 pinned algorithm variants. The
  governed correction keeps `leetcode-full` blocking in nightly and selects
  the already blocking representative subset plus taxonomy self-test for
  release qualification. Its indexed divergence record expires on 2026-10-31
  and fails readiness closed if not restored or separately renewed.
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
- [ ] Isolate installed-sysroot self-update qualification from mutable public
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
  `plans/phases/adhoc_performance_budget_host_variance.md`; no performance
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
