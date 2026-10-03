# Ad Hoc Validation Contracts and Resource-Aware Execution

status: active
registered: 2026-10-03
current_stage: M0 delivered; M1 implementation in progress

## Objective and authority

Deliver fast PR feedback, authoritative merge safety, continuous hardening, and
qualification of the exact artifacts shipped, with resource scheduling, evidence
reuse, and enforcement built in from the start. Every supported guarantee must
map to executable coverage, a cadence, and an enforcement boundary.

The user requested registration of the entire supplied implementation brief as
an active plan, including simplifying `AGENTS.md` so an agent can identify the
minimum commands needed for the work at hand. The subsequent instruction authorizes work through the plan to completion.
Implementation progress is recorded below; workflow/protection, qualification,
and publication claims still require their own acceptance evidence.

The full supplied brief is retained below as the scope contract. Its observations
about another VM, fixture counts, branch protection, unmerged work, and previous
timings are inherited evidence to verify at implementation entry, not current
measurements of this workspace. Existing assertions, pending failures, reviews,
and release obligations retain their status until prospectively changed through
the applicable policy and enforcement process.

## Existing authorities and dependencies

- [Verification runner and current commands](../../../verification/README.md),
  `verification/profiles/`, `verification/policy/`, and area manifests own current
  executable selection. Extend these mechanisms rather than introduce parallel
  inventories or a competing cloud performance implementation.
- [Roadmap](../../roadmap.md) owns sequencing. This plan extends the historical
  [PR gate rebalancing work](../archive/ad-hoc-pr-gate-speed-and-validation-lane-rebalancing.md);
  it does not reopen its merged changes or treat its old measurements as current.
- [Phase closure loop](../../../.cursor/skills/phase-closure-loop/SKILL.md)
  governs bounded implementation and review. The existing
  [agent instructions](../../../AGENTS.md) govern work until M0 updates them.
- The SQL cloud candidate in [PR #4259](https://github.com/sifr-lang/sifr/pull/4259)
  and storage prerequisite [#4278](https://github.com/sifr-lang/sifr/issues/4278)
  remain external dependencies to reconcile before consuming their results.
  This registration does not continue or approve that PR.
- Preserve the pending SQL cloud correctness, performance, and review requirements.
  The roadmap's completed original SQL platform entry describes earlier delivery;
  it does not establish acceptance of this later cloud qualification work.
- The transfer paths in the supplied handoff are from the previous environment.
  `/workspace/sql-cloud-transfer` is absent here at registration. Verify and restore
  the bundle and its provenance before relying on its contents; restore pinned
  runtime tools and caches separately, without copying authentication material.

## Execution milestones

Execute one bounded item at a time. Each implementation item records its owned
paths, dependencies, named checks, prospective budgets, final candidate, review,
and disposition. Split large milestones into independently reviewable changes.

| ID | Deliverable | Brief coverage | Status |
|---|---|---|---|
| M0 | Current-state inventory and simpler `AGENTS.md` with minimum commands by change type | User addition; implementation entry | implemented; delivered in #4279 |
| M1 | Canonical inventories, validation contracts, compatibility/support ownership, and evidence schemas | 1, 12–14, 17–19; reuse identity | in progress |
| M2 | Shared-VM admission, staged preparation, cache retirement, durable recovery, and evidence reuse | 7, 16, 20; all cloud additions; reuse/recovery | pending |
| M3 | Fast change-aware PR validation, enforced merge aggregate, main-push reuse, and scheduled hardening | 2–6, 15, 18 | pending |
| M4 | Compiler performance levels and separate generated-program benchmarks | 7–9, 14, 20 | pending |
| M5 | Artifact custody, published-predecessor upgrades, and compatibility/platform qualification | 10–12, 15, 19 | pending |
| M6 | End-to-end constrained-runner acceptance and verified protected enforcement | All scope and final acceptance | pending |

### M0 — inventory and minimal agent instructions

1. Record the actual base commit, initialized submodules, current profiles and
   selected IDs, SQL preparation costs, runner policies, workflow triggers,
   supported platforms, and available evidence. Verify repository protection
   through authenticated read access; YAML alone cannot establish enforcement.
2. Identify existing implementations for each requirement and assign ownership
   and gaps. Recount fixtures rather than freeze the inherited 143-fixture claim.
3. Simplify `AGENTS.md`: retain a short project description, essential safety and
   ownership constraints, links to canonical architecture/verification/phase
   guidance, and the small command decision table below. Remove the long command
   catalog and historical narrative from the agent's default reading path;
   preserve their authoritative records and any applicable approved exceptions.
4. Keep the compiler panic-safety rules, tracked-lockfile discipline, 900-line
   first-party source limit, scoped work, and protection of other owners' inputs.
   Do not turn simplification into a waiver of merge or release guarantees.

Command guidance implemented in `AGENTS.md` during M0:

| Work | Minimum local validation |
|---|---|
| Planning or review records only | `git diff --check` and verify changed local links, status, and scope; no build or broad gate |
| Rust/compiler implementation | Focused affected crate/test, e.g. `cargo test -p <affected-crate> <test-name>`; `cargo fmt --check`; `python3 scripts/check_file_size_guardrails.py`; named area checks for affected guarantees |
| Verification runner, policy, profiles, or workflow changes | Affected runner/area self-tests and workflow contracts; `uv run --project verification --locked python -m sifr_verify profiles check` when profile contracts change; compare emitted plans when selection changes |
| Implementation PR candidate | Existing `scripts/run_all_tests.sh --profile create-pr` until a reviewed stage contract replaces its selection; honor an explicitly applicable canonical phase exception |
| Final merge candidate | `scripts/run_all_tests.sh --profile merge` once for the final implementation candidate under current policy; rerun affected checks after relevant changes |
| Actual release candidate or explicit live integration | `release`, artifact qualification, or `python-interop-live` only when that stage is required by the task and its canonical contract |

The table guides selection; named contracts can require additional affected
checks. It must distinguish local targeted work from acceptance gates and explain
where the canonical command list lives. Do not run every listed command for every
edit. Reuse qualifying input-bound evidence only under the declared protocol;
mere compilation, a manifest check, or an old green job cannot replace execution.

Acceptance: the agent instructions are shorter, each change type has an
unambiguous minimum, existing required assertions and exceptions remain intact,
and documentation-only work does not trigger cold builds or broad qualification.

### M1 — contracts and evidence

Bind each supported guarantee to its owner, selected cases, stage, environments,
resources, and qualifying outcomes. Mechanically reconcile the canonical merge
inventory with cloud execution. Version evidence schemas, selection policies,
compatibility promises, support combinations, and the reuse dependency closure.
Separate report determinism, compiler determinism, cache equivalence, and release
reproducibility. Inventory security tests before creating new verification areas.

Acceptance: omissions, required skips, unassigned guarantees, incomplete runtime
execution, and drift fail contract checks. Smoke/representative/full reports show
the actual selection and total inventory. Unknown reuse dependencies invalidate
reuse conservatively. All policy changes apply prospectively.

### M2 — resource scheduling, recovery, and reuse

Discover affinity, cgroup CPU/memory limits, tmpfs accounting, storage, and
pressure. Admit each preparation/assertion stage using its additional allocation,
retained artifacts, and reserve; declare cold-preparation deadlines separately.
Schedule build graphs by consumer lifetime, preserve durable evidence, and retire
only eligible owned caches after protected consumers finish. Measure net recovery.
Bootstrap pinned tools, exact submodules, locked dependencies, and designated
offline stages through the environment's configured network/proxy/CA policy.

Use declared input-bound correctness checkpoints with complete inventory
accounting. Preserve timeout, cancellation, OOM, and ENOSPC classifications and
reap owned descendants. Separate functional, performance, and infrastructure
outcomes; performance admission cannot prevent functional execution. Protect
mutable Cargo targets from hardlink deduplication and preserve ownership/leases.

Acceptance: matching evidence is reused, source/runtime/artifact drift invalidates
it, cleanup cannot affect another owner or active inputs, and capacity failures
are explicit. Partial performance captures remain unusable for a paired pass.

### M3 — stage selection and protected delivery

Measure current preparation and assertion costs before rebalancing actual case
selection. Retain the brief's fast core, representative E2E, regression,
diagnostic, architecture/ABI/toolchain, LSP/generated-code/project/package/SQL,
performance-policy/smoke, and deterministic fuzz obligations. Classify shared
dependencies and every conservative trigger; unknown paths/diffs/errors broaden
coverage and selected jobs include their reasons.

Add a stable trusted required aggregate, validate the actual `merge_group`
candidate, and reconcile branch rules with the workflow. Replace duplicated
main-push `create-pr`/`merge`/`release` execution with verified evidence reuse and
required resulting-commit checks. Update `check_local_first_workflow.py`,
profile-selection tests, and workflow contracts together. Schedule broad nightly
coverage and bounded parser/HIR/codegen/diagnostic/project fuzz campaigns on
explicit commits, preserving seeds, reproductions, and owned regressions.

Acceptance: missing, skipped, cancelled, stale, failed, or untrusted mandatory
results cannot make the aggregate pass. Branch protection actually requires that
aggregate. Nightly never substitutes for required merge coverage, and release
qualification is tied to an actual release candidate.

### M4 — performance contracts

Declare separate smoke, representative, and full contracts and their permitted
claims. Preserve every obligation of the existing shared-cloud v2 protocol,
including the 65 cases, 5,120 prospectively fixed pairs, matched preparation,
independent compiler baseline, balanced AB/BA schedule, numeric budgets,
assumption screens, empirical p95 semantics, and real process/resource evidence.
Any cheaper protocol is separately registered and validated prospectively.

Add correctness-checked compiled-program throughput, startup, peak memory, binary
size, and reliable allocation metrics. Bind them to compiled hashes, optimization,
CPU target, and runtime dependencies; keep compiler time separate. Required
performance qualification blocks acceptance when inconclusive or unavailable.

Acceptance: interrupted captures, selected passing cases, adapted counts, or
unchanged-failure retries cannot produce a pass. Report process observations
separately from inner samples; do not claim unavailable PMU/retired instructions
or subtract CPU steal. Check freshness from completion and retain raw evidence.

### M5 — exact artifacts and support qualification

Audit source/submodule/version/target/toolchain/workflow identity through artifact
production, consumption, qualification, and publication. Verify consumed hashes
and promote qualified bytes without rebuilding. Add upgrades from verified actual
published predecessor bytes alongside existing synthetic transitions, including
reinstall/rollback where promised and explicit first-release inapplicability.

Exercise declared compatibility directions and supported environment combinations
and retain complete required release-platform qualification. Distinguish cache
invalidation/rebuild from compatibility promises, and component coverage from
native release packaging. Keep untrusted PR runs outside release credentials and
trusted qualification provenance.

Acceptance: source, runtime, producer, or artifact drift rejects evidence;
qualification and publication consume the expected bytes; every supported
transition and required platform has executable, owned coverage.

### M6 — complete acceptance and closure

Demonstrate all required workload on a declared constrained runner using measured
effective resources, not an assumed larger hardware minimum. Exercise selection,
reuse/invalidation, admission, recovery, cleanup ownership, aggregate negative
cases, artifact custody, predecessor transitions, and supported platforms.
Verify protected enforcement externally and retain candidate-bound raw evidence
and decision records. Preserve all historical failed/incomplete outcomes.

Close only when every acceptance condition in the supplied brief is demonstrated,
all required implementation items are delivered and reviewed, and the pending SQL
dependency has its own valid disposition. Record genuine capacity or external
blockers rather than manufacture a full pass or silently reduce required scope.

## Quality contract and current handoff

This registration requires documentation checks only: whitespace, changed local
links, roadmap registration, milestone coverage of all 20 brief sections and cloud
additions, and accurate active/pending statuses. No compiler, performance, cloud
receipt, branch-protection, or publication qualification is claimed here.

Implementation follows the current agent/phase rules until prospectively updated
in M0 and the relevant stage contracts. Targeted checks belong to the bounded
item; the authoritative gate belongs to the final candidate under applicable
policy. Required release checks remain conditional on a real release request.

Current state: M0 delivered through #4279; M1 implementation in progress;
M2–M6 remain pending.
The [entry inventory](../../../internal_docs/validation_execution_inventory_20261003.md)
records current selections, resources, verified protection, owners and gaps.
Next action: implement M1 canonical stage contracts and execution evidence. Subsequent items record candidate
SHA, changed paths, commands, outcomes, review, dependencies, and exact next action
here rather than treating the plan itself as execution evidence.

## M0 execution record — 2026-10-03

- Owned implementation candidate: `bbc9db098c96782cdcc4c72d4ec4bb09502bfb2b` on
  `codex/validation-contracts-resource-aware-20261003`, base
  `da57229746b0793577d257b29f1382baf60b37b0`.
- Draft implementation [PR #4279](https://github.com/sifr-lang/sifr/pull/4279).
- `AGENTS.md` reduced from 132 to 64 lines after review corrections. The complete proposal, current-state
  inventory and roadmap entry are delivered in the candidate; no executable
  selection, workflow, protection or performance policy has been changed.
- Passed: pinned-tool `profiles check`, file-size guardrails (4,338 source files),
  changed local-link checks, local-first workflow contract
  regressions, uv toolchain self-tests (46 checks), and the exact-pin invariant
  (6 projects/3 setup steps). Broad compiler gates are inapplicable to M0 prose.
- Required Opus review attempted three times through the repository review skill;
  each failed with `Not logged in · Please run /login`. No review verdict or merge
  acceptance is claimed. Atomic response paths were never published as passes.
- Failure logs and their hashes are retained outside the reviewed tree in
  `/workspace/validation-work/evidence/m0-review-blocker.json`, with the three
  separate `sifr-claude.*` request directories. No numbered review artifact was
  created for these failed requests.
- Blocking instruction: the [phase closure loop](../../../.cursor/skills/phase-closure-loop/SKILL.md)
  says, “If all three requests fail, record the blocker and stop.” M0 delivery and
  progression were paused for authenticated review or explicit user amendment.
  Claude CLI is installed session-locally; the missing prerequisite is account
  authentication, not the executable. No interactive login was started.
- The user completed official Claude sign-in. The authenticated review of
  `910d200c27faca079de640708e668c7f3b15dd93` returned NOT SATISFIED, retained in
  `/workspace/validation-work/evidence/sifr-claude.bnNg8w/response.md`. Its two
  blocking findings were missing unexpected-change/no-outside-mutation/external
  failure stop rules in `AGENTS.md` and an extra EOF blank line in the new inventory.
  Both are corrected together. The original workspace-only `git diff --check`
  missed the untracked inventory; that is withdrawn as whole-candidate evidence.
  Replacement validation uses `git diff --check` against the exact implementation
  base and includes all newly added files.
- SQL cold-preparation cost measurement is explicitly deferred from M0 to M3's
  lane measurement work; existing preparation commands and historical timings are
  inventoried without claiming a measurement in this VM. M0 completes the source,
  selection, resource, ownership and enforcement entry inventory.
- Empty guardrail labels now say `none`, and the verification command reference
  preserves CLI unit-test/E2E entrypoints.
- Final authenticated Opus review of `a02d8e1394045da8e2cbc67003741d23a6aa171d`
  returned SATISFIED with no blockers. Exact-candidate review is retained outside
  the reviewed tree at `/workspace/validation-work/evidence/candidates/a02d8e1394045da8e2cbc67003741d23a6aa171d/opus-m0.md`.
  The whole-candidate documentation checks pass. #4279 merged as
  `2bc2ebfc40db66619ab39134b12e7c9d9817ee7e`. M0 is delivered; historical failed
  requests and the initial NOT SATISFIED verdict retain their original status.
- The user authorizes working sequentially through the whole plan, superseding
  the skill's stop-after-one-delivered-item/start-new-session convention. Required
  checks and reviews still apply to each actual implementation candidate.
  M1–M6, protected enforcement, SQL acceptance and release qualification are open.

## M1 implementation checkpoint — 2026-10-03

The stage-policy/schema/CLI derive complete current selection from existing
profiles/manifests and preserve the live merge inventory for shared-cloud
correctness. The [contracts policy](../../../verification/policy/validation_contracts.md)
records stage boundaries, compatibility/support/security authorities and claims.
Input-bound correctness evidence validates source/runtime/command/selector/service/
artifact/producer bindings, complete selected-ID accounting, actual execution and
explicit runtime/compile/validation kinds; immutable output preserves old failures.
Cross-commit reuse is deliberately conservative pending M3's equivalence protocol.

Passed: 18 new contract/evidence negative tests, existing runner self-tests, strict
coverage matrix and assignment checks, and file-size guardrails. The actual
`create-pr` invocation failed before compiler setup at performance reference
admission because no dedicated `SIFR_PERFORMANCE_REFERENCE` is selected; the raw
failure is preserved at `/workspace/validation-work/evidence/m1-create-pr.log` and
is not acceptance evidence. This is the functional starvation explicitly owned
by the plan's cloud execution work, and is to be resolved through the existing
#4259 route with M2 scheduling. No implementation PR/merge acceptance is claimed
for M1 yet. Current implementation remains in the owned
`codex/validation-contracts-evidence-20261003` branch.

## Supplied implementation brief — complete scope contract

The following brief is preserved from the user's `Pasted text.txt` attachment.
Its historical observations and cross-environment paths carry the limitations
recorded above. Headings are nested for this document; transfer paths are displayed
as paths because the referenced files are not present in this workspace.

**Astra and I recommend the full proposal, with resource scheduling, evidence reuse and enforcement built into it from the start.** Those additions are essential for ordinary shared cloud VMs.

The following is a self-contained implementation brief for another cloud environment. It covers the entire proposal.

### Goal and governing rules

Deliver fast PR feedback, strong merge safety, continuous hardening and release qualification of the exact artifacts shipped.

Every supported guarantee must map to executable coverage, an appropriate cadence and an enforcement boundary. Preserve required assertions while reducing repeated work and unnecessary simultaneous storage.

Use existing Sifr capabilities where they already satisfy the requirement. Register policy changes prospectively, preserve historical failures and never reinterpret an incomplete run as passing.

### 1. Define explicit validation contracts

Keep these stages:

| Stage | Purpose |
|---|---|
| `create-pr` | Fast feedback and relevant specialist checks |
| `merge` | Authoritative pre-merge correctness |
| `nightly` | Broad hardening and exploratory coverage |
| `release` | Source-level release qualification |
| Artifact qualification | Validate the exact packages being shipped |
| `python-interop-live` | Explicit real-service integration |
| Cloud execution policy | Run correctness on shared VMs and report performance independently |

For each stage, declare required suites, selected cases, supported environments, resource requirements and qualifying outcomes.

Keep a canonical inventory so changes to merge coverage cannot silently disappear from cloud execution.

### 2. Make PR validation genuinely fast

Retain:

- Core unit and compiler checks.
- Representative E2E coverage—the existing corpus contains 143 fixtures.
- Known regressions and diagnostic rules.
- Architecture, ABI and toolchain guards.
- LSP, generated-code, project/package and SQL smoke.
- Performance-policy checks and performance smoke.
- Deterministic fuzz smoke.

Move expensive specialist qualification behind conservative change selection.

The current `create-pr` profile includes the same 19 SQL suites as merge, including clean-build qualification. Review actual selections and preparation costs; changing profile labels alone will not shorten feedback.

### 3. Implement conservative change-aware selection

Classify changes using subsystem dependencies, including shared compiler/runtime code.

Relevant triggers must include:

- Source changes and generated-code dependencies.
- Cargo manifests, lockfiles and feature definitions.
- Toolchains and dependency versions.
- Submodules and fixture inventories.
- Verification code, policies and workflows.
- Platform/process/filesystem/cache/ABI boundaries.

Unknown paths, unavailable diffs or classifier failures must select broader coverage.

Test the classifier against changes that affect multiple subsystems. Emit the selected jobs and the reasons for selecting them.

### 4. Enforce the actual merge gate

The inspected active main-branch ruleset has no required CI status checks. Fix that enforcement gap.

Create a stable required aggregate check that verifies every mandatory job and result. Missing, skipped, cancelled, stale or failed required work must prevent a passing aggregate.

For merge queues:

- Support `merge_group`.
- Check out and validate the actual merge-group commit.
- Bind results to that candidate.
- Configure branch rules to require the aggregate from the intended trusted workflow/application.

Workflow YAML and repository protection must agree. Do not declare enforcement complete merely because the workflow exists.

### 5. Remove duplicated main-push validation

The current workflow runs `create-pr`, `merge` and `release` on main pushes.

Replace that with reuse of applicable pre-merge evidence and any required checks for the resulting commit. Reserve release qualification for an actual release candidate.

Update the workflow, `check_local_first_workflow.py`, profile-selection tests and workflow-contract tests together.

Reuse must be based on verified inputs, not simply on an earlier green result.

### 6. Schedule nightly validation and sustained fuzzing

A nightly profile already exists; a scheduled nightly workflow was not found.

Run broader differential, sanitizer, generated-code, ecosystem, algorithmic, project/package, SQL and stdlib coverage on an explicit commit.

Run bounded fuzz campaigns separately for parser, HIR/type system, codegen, diagnostics and project/package parsing.

Preserve seeds, failures and minimized reproductions. Reproducible serious defects become owned regression tests and block affected delivery.

Nightly complements required merge coverage; it does not replace it.

### 7. Separate correctness from performance qualification

Report independent outcomes:

- Functional: PASS or FAIL.
- Performance: PASS, REGRESSION or INCONCLUSIVE.
- Infrastructure: an explicit failure classification.

Functional checks must execute even when performance admission fails.

Declare which changes and releases require performance qualification. When required, inconclusive or unavailable performance evidence blocks acceptance.

Reuse the cloud implementation already present rather than creating a second competing mechanism.

### 8. Define explicit performance levels

Maintain separate contracts for smoke, representative and full qualification.

Each must declare its workloads, sample counts, budgets, supported measurement conditions and permitted claims.

Preserve the current shared-cloud v2 contract for its existing obligations:

- 65 cases and 5,120 prospectively fixed pairs.
- Matched baseline/candidate preparation.
- Independently merged compiler baseline.
- Fixed balanced AB/BA schedules.
- Existing numeric budgets and deadlines.
- Process-level inference and mandatory assumption screens.
- Correct empirical p95 semantics.
- Real RSS, CPU, output, cache and cleanup evidence.

Do not combine interrupted captures, reuse selected passing cases, adapt counts after outcomes or retry unchanged failed performance runs until green.

Any cheaper future protocol needs its own prospective policy and validation.

### 9. Add generated-program performance coverage

Create a separate benchmark category for compiled Sifr programs.

Measure correctness-checked workloads for throughput, startup, peak memory and binary size. Add allocation metrics where instrumentation is reliable.

Bind results to compiled artifact hashes, optimization settings, target CPU and runtime dependencies.

Keep compilation time separate from program execution. Compiler speed and generated-code quality do not establish application performance.

### 10. Qualify and publish identical artifacts

Sifr already has substantial source identity, artifact-hash and publication controls. Audit and complete that chain.

Qualification reports must bind:

- Exact source and submodule revisions.
- Version, target and toolchain.
- Artifact hashes.
- Executed suites and outcomes.
- Producer and workflow provenance.

Consumers must verify hashes before execution. Publication must promote qualified bytes rather than rebuild equivalent-looking packages.

### 11. Test actual published predecessor upgrades

Keep existing synthetic transition coverage.

Additionally, install verified artifacts from the previous published release, create representative state, upgrade to the candidate and verify supported compatibility.

Exercise reinstall and rollback where promised.

The inspected transition job builds a lower-version fixture from current source; testing actual predecessor bytes provides different evidence.

For a first release, predecessor coverage is explicitly inapplicable.

### 12. Define compatibility promises

Consolidate existing contracts for:

- Source and project manifests.
- Lockfiles and caches.
- Generated artifacts.
- Installer and update state.
- Editor/compiler pairing.
- Python and Rust interop boundaries.

For each, specify its owner, supported version range, direction of compatibility and tests.

Distinguish compatibility guarantees from intentional cache invalidation and rebuild.

### 13. Standardize execution evidence

Distinguish:

`selected`, `validated`, `compiled`, `executed`, `passed`, `failed`, `skipped`, `blocked` and infrastructure failure.

Record source, selection, toolchain, features, platform, network mode, relevant dependency/service versions and consumed artifacts.

A manifest check or successful compilation cannot satisfy required runtime execution. A required skip cannot count as a pass.

### 14. Make coverage labels verifiable

For smoke, representative and full modes, report actual selected IDs and total inventory.

Validate completeness mechanically against manifests.

Document when “full” means full policy strength over a representative corpus.

Benchmark reports must distinguish process observations from averaged inner samples and avoid stronger latency claims than the measurements support.

### 15. Make platform coverage risk-aware

Trigger relevant platform qualification for changes to compiler components, sysroot, process execution, filesystem/cache storage, native loading, generated Cargo projects, ABI, SQL WASI and distribution.

Include shared dependencies and infrastructure changes.

Retain complete qualification of every required release platform. Introduce tiers without silently demoting existing support promises.

### 16. Treat cache correctness as correctness

Inventory existing coverage before adding gaps.

Cover cold/warm equivalence, restart, corruption, partial/stale entries, concurrency, crashes, GC, schema changes and relocation.

Preserve ownership and leases during cleanup.

Never deduplicate mutable Cargo targets through hardlinks. Deduplicate only demonstrably immutable artifacts with verified identity and protected consumers.

### 17. Keep determinism claims separate

Maintain distinct evidence for:

1. Test-report determinism.
2. Compiler semantic/output determinism.
3. Cache equivalence.
4. Release reproducibility.

Schedule repeated expensive checks according to relevant changes and qualification needs.

Normalization may remove declared volatile fields; it must not hide failures or coverage differences.

### 18. Give security boundaries explicit ownership

Map existing security-related tests before creating new verification areas.

Cover archive/path traversal, symlink escapes, malformed metadata, environment injection, host-tool confinement, process cleanup, resource exhaustion, temporary-file permissions and credential handling.

Use least-privilege workflows. Untrusted PR execution must not access release credentials or produce trusted qualification evidence.

### 19. Version the supported-environment matrix

Specify relevant OS/libc minimums, architectures, Python versions, editor versions and interop targets.

Use justified coverage combinations rather than the full Cartesian product.

Distinguish component support from promises to ship native release artifacts—for example, Windows component coverage and Windows release packaging are separate claims.

### 20. Measure test economics

Collect preparation and assertion time separately, along with cache effectiveness, executed counts, retries, infrastructure failures and peak resource use.

Use measurements to adjust future schedules and budgets prospectively.

Recent low defect discovery alone does not establish that a test is low-value.

### What the original proposal misses for this cloud environment

| Missing requirement | Recommended addition |
|---|---|
| Effective resource discovery | Read affinity, cgroup CPU quota and memory limits. This VM exposes five CPUs but has four CPU-equivalents and a 16 GiB memory limit. |
| Shared tmpfs memory accounting | `/tmp` and `/dev/shm` each advertise 8.8 GiB, but both consume the same memory allowance. They are not independent extra capacity. |
| Build-graph lifetime scheduling | Prepare a graph, execute all required consumers, retain evidence, then reclaim eligible owned caches. Make this normal runner behavior. |
| Per-stage resource admission | Account for additional disk, memory, temporary copies, retained artifacts and reserve before launching work. Measure actual reclaimed space. |
| Cold preparation budgets | Give named preparation stages prospective operational deadlines separate from assertion and performance budgets. |
| Reproducible bootstrap | Pin tools, initialize exact submodules, respect proxy/CA configuration, fetch locked dependencies, then run designated offline stages. |
| Runtime-context identity | Bind interpreter bytes, Cargo configuration, thread settings, fixture ancestry, local environment presence and artifact bytes. |
| Durable recovery | Record timeout, cancellation, OOM and ENOSPC separately. Reap owned descendants and preserve completed evidence without manufacturing a pass. |
| Cloud measurement limits | Record wall time, process CPU/RSS and pressure diagnostics. Do not subtract CPU steal or claim unavailable PMU measurements. |
| Evidence freshness and retention | Check qualification promptly after completion and retain durable raw evidence and decision records. |

These requirements come directly from this session:

- CPU steal was observed around **16–29%**. That does not establish the cause of every timing variation.
- Cold source and release compilation took **30m41s** and **33m29s**. Several cold builds cannot reliably share one 40-minute wrapper.
- The isolated sysroot targets occupy **3.41 GiB gross**. Net recovery must account for retained binaries and new outputs.
- The existing SQL clean-build guard requires **8 GiB free**. Preserve it until a reviewed prospective policy replaces it.
- This VM cannot provide retired-instruction evidence.
- The current cloud receipt expires **24 hours after completion**, so a 12–13-hour capture does not itself expire the receipt.

Do not solve these problems by imposing a larger hardware minimum without measurements. Demonstrate the resource-aware runner on a declared constrained environment and report genuine capacity blockers when a required stage cannot fit.

### Evidence reuse and recovery

Implement a concrete reuse key covering:

- Relevant source and dependency closure.
- Test, selector and policy versions.
- Selected cases, fixtures, manifests, locks and submodules.
- Toolchain, commands, features and configuration.
- Runtime inputs and consumed artifact hashes.
- Host/resource identity where the claim depends on them.
- Producer trust and provenance.

Record the exact observed commit. If another commit has equivalent relevant inputs, verify and record that equivalence explicitly. Unknown dependencies invalidate reuse conservatively.

Correctness checkpointing may reuse completed results under a declared, input-bound protocol. A final pass requires complete inventory accounting.

The current paired-performance protocol must continue to require one complete invocation; correctness checkpointing does not authorize combining partial benchmark captures.

### Implementation and acceptance

Deliver the full scope as independently reviewable changes:

1. Canonical inventories, policy contracts and evidence schemas.
2. Shared-VM resource discovery, staged preparation, cache retirement and recovery.
3. Merge enforcement, change selection, main-push deduplication and scheduled hardening.
4. Compiler and generated-program performance coverage.
5. Artifact custody, predecessor upgrades and support/compatibility qualification.
6. End-to-end validation and protected enforcement.

Acceptance must demonstrate that:

- Every required assertion remains covered.
- Missing or skipped required jobs cannot yield a pass.
- Cleanup cannot remove active or another owner’s inputs.
- Artifact, source and runtime drift invalidate evidence.
- Matching evidence is reused and relevant changes invalidate it.
- Resource and infrastructure failures remain distinguishable from product failures.
- Qualification and publication consume the expected artifact bytes.
- The complete required workload executes within the declared constrained-runner conditions.

### Handoff to the other cloud environment

Repository: [sifr-lang/sifr](https://github.com/sifr-lang/sifr).

Existing unmerged cloud work is in [PR #4259](https://github.com/sifr-lang/sifr/pull/4259):

- Candidate branch: `codex/sql-cloud-reference-qualification-20261002`
- Candidate SHA: `f88973102ac0b7b0b5c07969a57fcc4ed0e87098`
- Baseline tooling branch: `codex/sql-cloud-shared-performance-baseline-20261002`
- Baseline tooling SHA: `ef53c49e4762aa0e2813783d503f246c25cbfa46`
- Independent compiler reference: `5fbeee50c2f70abd47e5bfcb98d1f1cd0b978042`

The storage prerequisite is registered as [#4278](https://github.com/sifr-lang/sifr/issues/4278); its implementation is pending.

The resume instructions (`/workspace/sql-cloud-transfer/cloud-v2-resume.md`) and verified evidence bundle (`/workspace/sql-cloud-transfer/cloud-v2-evidence.tar.gz`) preserve the current work. The bundle excludes runtime tools, build caches and authentication. Restore those separately in the new environment.

**The existing SQL phase remains open. This proposal expands the work plan; it does not waive its pending correctness, performance or review requirements.**
