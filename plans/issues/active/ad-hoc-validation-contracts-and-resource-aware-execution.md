# Ad Hoc Validation Contracts and Resource-Aware Execution

## M5 single standard Intel qualification attempt — prospective scope

Paid larger runners are unavailable. Register one actual native qualification
attempt on standard `macos-15-intel` for the frozen final candidate, using the
existing live 9 GiB preparation admission (6 GiB process, 1 GiB temporary memory,
2 GiB reserve). The audited standard Intel observation at execution source
`e274dec214e5b45606b711fe6fd70c3449b16426` recorded 9,577,099,264 available
bytes, 89,577,152 bytes below admission. That snapshot cannot admit a later job
or establish its rejection. The actual attempt must stop on admission failure;
do not repeat diagnostics or retry the unchanged attempt in a loop. The runner
label establishes neither available capacity nor qualification.

Change only the native Darwin x64 row from `macos-15-large` to `macos-15-intel`,
its exact finite workflow expectation, and focused runner/checksum controls.
The existing uv mapping binds the standard Intel runner to the canonical Darwin
x64 archive. Reject paid Intel substitution, an ARM label in the Intel row, and
the Darwin ARM uv checksum on Intel. This prospectively supersedes the older
provisioned-Intel selection below; historical failures remain unchanged.
Preparation still permits two Cargo workers, and the published transition uses
one. All four native targets, eight runtime cases per target, host/storage
authority, producer identities, budgets and reserves remain required.

ARM remains an external prerequisite. Keep its current `macos-15-xlarge` row;
the recorded standard ARM host has only 2.97 GiB admitted available memory.
Qualification needs an available dedicated Apple Silicon host meeting the
unchanged 9 GiB preparation admission and physical-storage requirements. The
suggested 5.5 GiB monitored experiment is unimplemented and does not admit that
snapshot. Successful Intel execution cannot substitute for ARM assertions.

Own `/workspace/sifr-validation-final-main-integration`, branch
`codex/validation-final-main-integration-20261004`, based on
`7424f3934e91125b4bbe607c740c4f79dfef741e`. That clean full candidate integrates
the diagnostic ownership repair `98c248a7161b478a3bf9360cc54c8535fc2f227c`
with delivered main `0e8bdc47122d2b45fbe6d2b76be5657bda888d75`: 58 disjoint
tooling paths, including an intentional byte-exact CRLF fixture. Existing draft
[#4332](https://github.com/sifr-lang/sifr/pull/4332) carries combined delivery.
Named checks for this bounded item: local-first workflow controls, uv invariant
and negative controls, actual submodule ownership guard/self-test, file-size
guardrail, local links and diff check; exact-candidate read-only Opus review.
No remote dispatch or broad/compiler gate is part of this implementation step.
Full local create-PR and final shared-VM cloud/performance acceptance, complete
four-platform native qualification and protected enforcement remain pending.

Focused validation passed: workflow controls; the maintained uv invariant
(six exact pins, eight setup steps) and 58 uv controls; actual ownership guard
and self-test; the 4,452-file guardrail; added plan paths/source identities and
whitespace checks. Raw logs are outside Git at
`/workspace/validation-work/evidence/standard-intel-attempt-20261004/`.
Exact-candidate review is pending; no native runtime assertion was executed.

The latest local create-PR attempt on `98c248a71` passed source preparation,
metadata preparation and both metadata structural tests, then exited 2 at the
next disk admission: 4,294,967,296 bytes required, 3,835,068,416 available
(4 GiB versus 3.57 GiB). Preserve its failed/incomplete evidence in
`/workspace/validation-work/evidence/acceptance-98c248a71-20261004-v5/`;
this is not a full gate pass. The original `0b7b5b8d3` capture failed because
operator command scoping temporarily changed its checkout. Its restored source
and incomplete evidence remain preserved; its samples cannot qualify or be
combined. The frozen final candidate requires one complete fresh fixed paired
capture with valid matched endpoints before final performance acceptance.

## M3/M5 diagnostic checkout ownership repair — prospective rescope

Actual hosted merge validation and the local create-PR gate on `0d8d01e1b`
failed before compiler preparation: the new capacity diagnostic checkout did not
initialize recursive submodules and was not an existing classified non-source
checkout. This separate bounded repair adds recursive initialization and makes
the diagnostic workflow contract reject both missing and nonrecursive settings.
Keep the existing ownership guard and its classifications unchanged. Named
checks: actual ownership guard/self-test, local-first workflow controls, uv
invariant/controls, source guardrail and diff; exact-candidate read-only Opus
review; the still-required full candidate gates. Historical diagnostic artifacts
remain observations of their actual execution source, not current qualification.

While creating this repair worktree, an agent command-scoping error temporarily
enabled sparse checkout in the original frozen measurement worktree. Its worker
rejected a missing corpus path and stopped with exit 2. Original source is fully
restored and clean at its unchanged commit. Preserve the incomplete capture as
infrastructure failure; its samples cannot qualify or be combined. Freeze the
repaired final candidate, prepare valid matched endpoints and run one complete
fresh fixed capture. No numerical verdict was produced or retried unchanged.

## M2/M5 standard macOS capacity observation — prospective scope

The user cannot provision paid larger runners and requested Astra's sizing
consultation. Its read-only audit found that native preparation's 9 GiB
admission is an estimate (6 GiB process, 1 GiB temporary memory, 2 GiB reserve),
not a measured hardware minimum. Preparation currently permits two Cargo workers;
the published transition permits one. The earlier single-worker preparation
description was inaccurate and has been corrected in the draft PR record.

The retained standard ARM snapshot has 7 GiB installed but 3,221,258,240 bytes
available under the existing free/inactive/speculative-page accounting. A smaller
3.5 GiB process estimate plus the unchanged 2 GiB reserve would still fail
admission. Historical child RSS does not establish a cold Darwin process-tree
peak. Do not reduce reserves or classify an unstarted job as qualification.

This bounded item adds a read-only diagnostic workflow on standard macOS ARM and
Intel runners, with an exact source checkout, pinned Python/uv, bounded raw
memory/compression/swap/pressure and physical-storage observations, and independent
retained-byte checks. It adds no compiler build, native runtime assertion, resource
profile, memory allowance, or qualification waiver. Existing native qualification
and the frozen original paired capture remain unchanged.

Owned scope: the diagnostic module/tests, diagnostic workflow, its workflow
contract/control registration, and this plan record. Acceptance requires meaningful
controls for failed/truncated/missing observations, raw/hash/source drift and
false qualification claims; an exact-candidate read-only Opus review; actual
standard-runner diagnostic artifacts; and the still-required full candidate gates.
Fresh facts decide whether a separately reviewed monitored resource profile can
be attempted or a real capacity blocker remains.

The diagnostic implementation and six meaningful controls pass. The existing
native custody, backing-store, recovery and source-dependency controls also pass;
the actual maintained uv invariant covers six pins/eight setup steps, and all
56 uv negative controls pass. The workflow contract rejects a paid runner,
unbound source, skipped observation, write permissions, inserted compilation and
missing failure retention. The 4,432-file guard and whitespace check pass.
Exact-candidate review, actual standard-runner observations and full gates are
pending; no native or performance qualification follows from these controls.

status: active
registered: 2026-10-03
current_stage: M0 delivered; M1–M5 implementation in progress

## Current bounded item: provisioned Darwin x64 native runner — 2026-10-04

Run `37189948998` on reviewed `13a4d7572dacfb69ac0077b385860347b6349699`
passed x64 locked online/offline source preparation, then stopped before native
compilation at admission: 9098891264 bytes available against the unchanged
9663676416-byte requirement. Its log and failed state stay preserved. The prior
x64 native pass on `886464cad855dae47a26748f1d6528dd32fe1f92` remains specific
to that frozen candidate and host observation. No unchanged numerical capture
is retried and no native memory reserve is reduced.

Own branch `codex/validation-native-larger-intel-20261004` in the existing owned
full worktree `/workspace/sifr-validation-native-larger-arm`, based on `13a4d7572`.
The earlier ARM branch is retained. Select documented `macos-15-large` x64
(30 GB RAM), bind its existing canonical Darwin x64 uv checksum, and require
that exact runner in the native workflow contract. Negative controls reject
downgrades of both macOS rows. All four native targets, single-worker native
preparation, runtime assertions, resource estimates/reserves and other workflows
remain intact. Named checks: uv controls/invariant, local-first workflow controls,
source guardrail and diff check; scoped read-only Opus review. Larger-runner
account access, actual qualification and full candidate acceptance remain pending.

## Current bounded item: provisioned Darwin ARM native runner — 2026-10-04

The user authorized a larger GitHub macOS ARM runner and removal of this chat's
inactive default incremental cache. The incremental-only cleanup completed with
the retained compiler hash unchanged, leaving 14 GiB free on the workspace disk.
Its custody record is retained outside Git; no failure graph or active cache was
removed. Disk capacity no longer blocks the independent remaining work.

Own `/workspace/sifr-validation-native-larger-arm` and branch
`codex/validation-native-larger-arm-20261004`, based on
`356168cac9ad5ef2a70289d7693851a1630e4a88`. Select the documented
`macos-15-xlarge` ARM M2 runner with 14 GB RAM and retain the unchanged native
admission estimates and reserves. Bind its exact platform checksum through the
finite toolchain guard. GitHub's hosted-runner API currently returns 404,
"GitHub hosted runners are not supported for this organization"; runner-group
administration returns 403, "Resource not accessible by integration". The
organization's plan is not exposed to this integration. The documented setup
requires an organization owner and GitHub Team or Enterprise Cloud; selecting a
label does not establish account availability, admission or qualification.

Actual bounded backing-store diagnostics from failed run `37186236372` show
`disk0s2`, an internal writable Apple_APFS partition of `disk0`, with an empty
bus field and the AppleVirtIOStorageDevice PCI device-tree path. Register this
specific OS-identified guest block-storage authority. Require every identity,
parent, partition, content, size and driver field; reject missing, contradictory,
RAM/image and unknown authority. Existing physical-bus and HFS rules remain.
Named validation: capacity negative controls, native candidate/source dependency
controls, finite uv matrix and local-first workflow controls (including runner
downgrade rejection), source guardrail and diff check; scoped
read-only Opus review. Actual larger-host qualification and full candidate
acceptance remain pending. No seven-GiB host admission or platform pass is claimed.

## Current bounded item: explicit local performance receipt reuse — 2026-10-04

Own `/workspace/sifr-validation-cloud-receipt-reuse` and branch
`codex/validation-cloud-receipt-reuse-20261004`, based on
`5b53dda480d651a89851c5df4c2efb4d1345cc36`. Add an explicit local consumption
recipe for a preserved measurement worktree, with `SIFR_CLOUD_PERFORMANCE_SOURCE_WORKTREE`.
Its original full paired checker remains authoritative. No receipt is resealed,
no sample is regenerated/combined and no runtime assertion is manufactured.

Require a measured ancestor, the same owned Git repository, clean full trees,
an exact narrowly enumerated set of non-measurement changes, equal actual
compiler/runtime/corpus trees including ignored source inputs, matching ambient
Python inputs and identical current/measured compiler bytes/build identity.
Recheck proof and hashes after the original checker. Record immutable started,
failed or passed consumption attempts, retaining measured/current commits and
zero newly executed assertions. Unknown inputs, linked source inputs, sparse
trees, compiler drift and unqualified original receipts fail closed. Current
compiler byte changes always require fresh qualification; this recipe does not
promise reuse of the running capture for a later final candidate.

Named validation: negative source/lock/harness/ignored-input/foreign/sparse,
compiler/Python drift, original-checker regression and mid-check mutation
controls; cloud outcome controls; source guardrail; diff check; scoped read-only
Opus review. Full local PR/final gates and actual paired/platform qualification
remain pending. The integration worktree stays frozen on `0b7b5b8d3...` during
the original 65-case/5120-pair capture.

## M6 input: shared-VM final gate command — 2026-10-04

Document the existing canonical `cloud --require-performance` route as the final
merge acceptance command on a shared Linux VM. Live resolved profile comparison
found only `name` and `description` differ from `merge`: both select 106 suites,
the same guardrails, first-party crate tests, E2E corpus, toolchain steps and skip
rules. Four focused controls verified selection equality and blocking outcomes
for missing/inconclusive performance, regression and failed correctness. This
records selection/control evidence, with zero runtime qualification assertions;
it does not claim full candidate gate execution.

The command is `scripts/run_all_tests.sh --profile cloud --require-performance
--compact-resources`, with `SIFR_CLOUD_PERFORMANCE_RECEIPT` naming the complete
fresh candidate-bound independently checked receipt. It uses the existing
reviewed shared-cloud numerical contract and leaves the controlled-host `merge`
route intact. Implementation PR gates, native platform requirements, external
ownership and protected enforcement remain required. The current integration
source is still frozen for its original 65-case/5120-pair capture; no partial
capture or configuration-mismatch preparation qualifies performance.

## Current bounded item: native APFS backing-store authority — 2026-10-04

The hosted ARM Darwin volume reports APFS with an empty volume bus field and an
explicit backing store `disk0s2`. Classify APFS through every OS-declared backing
store rather than the synthetic volume's bus label, and retain bounded raw
`diskutil info -plist` observations for those stores in the existing diagnostic
upload. Reject missing, malformed, duplicate, mismatched, unknown or memory-backed
store authority. HFS keeps its device bus authority. Native memory/disk estimates,
reserves, host ownership, target matrix and runtime assertions are unchanged.

Owned branch/worktree: `codex/validation-native-apfs-stores-20261004` at
`/workspace/sifr-validation-native-apfs-stores`, based on
`0b7b5b8d3c3aed0c476826e59c646ba4f19f59fc`. The integration source is frozen for
its full independent paired capture. Named validation: capacity controls,
candidate/source-dependency custody controls, diagnostic-program controls, source
guardrail and diff checks; scoped read-only Opus review follows. Full candidate
PR/merge acceptance gates and actual native platform qualification remain pending.
This repair does not grant admission to the seven-GiB ARM host or establish a
native qualification pass; its unchanged nine-GiB preparation allocation still
requires a sufficiently provisioned dedicated runner.

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
| M2 | Shared-VM admission, staged preparation, cache retirement, durable recovery, and evidence reuse | 7, 16, 20; all cloud additions; reuse/recovery | in progress |
| M3 | Fast change-aware PR validation, enforced merge aggregate, main-push reuse, and scheduled hardening | 2–6, 15, 18 | in progress |
| M4 | Compiler performance levels and separate generated-program benchmarks | 7–9, 14, 20 | in progress |
| M5 | Artifact custody, published-predecessor upgrades, and compatibility/platform qualification | 10–12, 15, 19 | in progress |
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

Current state: M0 delivered through #4279; M1–M4 implementation in progress;
M5/M6 remain pending.
The [entry inventory](../../../internal_docs/validation_execution_inventory_20261003.md)
records current selections, resources, verified protection, owners and gaps.
Next action: finish trusted aggregate custody and main-push reuse, then implement
generated-program metrics and exact-artifact predecessor coverage before full
constrained acceptance and protected enforcement.
The shared debug cache remains preserved; its ownership is not a prerequisite for
normal Cargo preparation. The cold estimate is not the measured additional cost.
M1/M2 acceptance gates and M3–M6 delivery remain open. Execution records below
identify candidate SHA, checks, review, dependencies and failures; the plan itself
is not qualifying execution evidence.

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

## M2 scheduling checkpoint — 2026-10-03

### Correctness checkpoint continuation

- Candidate work owns `codex/validation-correctness-checkpoints-20261003` in
  `/workspace/sifr-validation-recovery`, based on the separately reviewed native
  SIGKILL correction `1786e75ca197976f055c50a6bc3407ba87589c70`.
  Its review is outside Git at
  `/workspace/validation-work/evidence/candidates/1786e75ca197976f055c50a6bc3407ba87589c70/opus-native-sigkill.md`;
  [PR #4282](https://github.com/sifr-lang/sifr/pull/4282) remains a draft.
- Input-bound correctness checkpoint consumption now preserves immutable attempts
  and independent retained copies, recomputes consumer keys, rejects unknown
  producers, drift, incomplete/duplicate inventory, expired/tampered artifacts,
  and performance receipts. Unknown dependency closure executes fresh. The
  complete-inventory reconciler rejects missing or overlapping required cases.
- The first integrated recipe is the HIR maintainability guard, pinned by audited
  script hash and explicit consumed-document/path-presence inputs. It binds the
  full source inventory, tool/interpreter/dependency bytes, all standard-library
  source files and actual loaded native libraries in an isolated probe. Optional
  dormant native extensions are not incorrectly treated as loaded dependencies.
  Site hooks, PYTHONPATH, and mutable bytecode caches do not control the guard or
  custody supervisor. Other recipes remain fresh because their closure is unknown.
- Targeted checkpoint/recipe tests: 23 passed, raw
  `/workspace/validation-work/evidence/m2-checkpoint-recipe-tests.log`. The recipe
  tests execute the actual guard and demonstrate one execution followed by reuse,
  consumed-document/untracked-path invalidation, failure preservation after exact
  restoration, and unaudited-script fresh execution. Their capacity and general
  runtime identity are fixtures, not constrained-runner qualification. All 44
  process/custody tests passed, raw
  `/workspace/validation-work/evidence/m2-checkpoint-control-plane-tests.log`.
- On the actual current VM, checkpoint capacity is unavailable because ordinary
  disk reserve is already below 8 GiB after the failed cloud preparation. The
  real CLI executes the read-only guard fresh and passes; reuse remains disabled.
  Raw `/workspace/validation-work/evidence/m2-checkpoint-live-capacity.log`.
  It publishes no checkpoint and does not suppress the required assertion.
- The parent constrained run `d5cedf1a3` finished unqualified: source preparation
  passed in 1,721,969 ms; release compiler preparation took 24m13s, corpus test
  preparation 25m02s, and metadata preparation 1m23s; the whole package stage
  passed in 3,046,330 ms. Sysroot assertion admission then failed ENOSPC:
  required 10,737,418,240 bytes, available 7,979,851,776. No sysroot assertion or
  normal graph retirement ran. Functional failed, performance inconclusive,
  qualified false. Raw report/time/log/journal are archived under
  `/workspace/validation-work/evidence/candidates/d5cedf1a3091e10adc93d33b85c32b769891eafb/`.
- The private source/package graphs measure roughly 5.4/1.8 GiB. The main target's
  incremental cache now measures about 11 GiB; its original owner remains unknown
  and it must not be cleaned. Follow-up resource scope must correct measured
  graph lifetime/footprint and preserve all failures and compiled artifacts.
  Admission estimates proved insufficient for the main corpus/metadata graph;
  no reserve, assertion selection, runtime obligation, or performance threshold
  is lowered or reclassified. Production reuse, implementation acceptance gates,
  scoped checkpoint review, the resource follow-up, and M3–M6 remain open.

### Early immutable-output lifetimes and owned recovery continuation

- The correctness-checkpoint candidate `79e2524e87b0e597100a442c78f3e80779acca53`
  received Opus **SATISFIED**, with no blocking findings. Review is outside Git at
  `/workspace/validation-work/evidence/candidates/79e2524e87b0e597100a442c78f3e80779acca53/opus-checkpoints.md`;
  draft [PR #4283](https://github.com/sifr-lang/sifr/pull/4283) remains gated.
- This bounded continuation owns `/workspace/sifr` and branch
  `codex/validation-compiler-graph-lifetimes-20261003`, based on that candidate.
  It separates compilation/packaging consumers from later runtime consumers.
  Successful source preparation retains a bounded, verified lossless copy before
  supported Cargo cleanup and separately admits executable restoration. Package
  preparation retains its complete verified archive and compiler before cleanup.
  Only then do the original library corpus/metadata preparation and every selected
  sysroot assertion run. No compilation receipt claims an assertion passed.
- Receipts bind current source/observed commit, isolated Python closure, actual
  tool/compiler bytes, registry sources, toolchain libraries, native build inputs,
  configuration, producer and retained artifact bytes. Consumers independently
  recompute the expected identity and reject drift, expiry, tampering or a runtime
  assertion claim. Unknown cache owners are never cleaned; independently copied
  outputs still require actual retention admission.
- Real dependency observation enumerated 50,631 external build input files plus
  715 stdlib files and 10 loaded native libraries. The first observation rejected
  directory links; the correction traverses and binds their targets and bytes,
  rejecting cycles. Both logs remain outside Git; these are dependency observations,
  not compiler/assertion acceptance evidence.
- Before retiring the now-obsolete d5 preparations, the old e9 compiler was
  losslessly compressed to 310,235,081 bytes and decoded/hash-verified. That
  bounded archival recovery restored the unchanged 8 GiB normal reserve from
  the already under-reserve failed state; it did not admit a build/assertion.
  The package graph retired first using exact measured copy allocation, then the
  source graph, through supported Cargo cleanup under their original exclusive
  owner leases. Both d5 compiler byte sequences were retained independently,
  then losslessly compressed and verified (276,879,402 / 31,473,300 bytes).
  Free storage increased from 7,967,952,896 to approximately 16,318,980,096 bytes.
  The shared main target remains untouched. Raw d5 assertions remain unexecuted,
  functional failed, performance inconclusive and qualified false.
- Custody and retirement records are preserved under each original candidate's
  external evidence directory. Raw operator logs:
  `/workspace/validation-work/evidence/m2-obsolete-owned-recovery.log` and
  `m2-d5-compiler-compression.log`. A gzip decode restores the exact original
  compiler bytes; earlier raw-file paths now have explicit lossless custody records.
- Targeted receipt/retention tests cover real immutable copying, compressed
  restoration, retained native package bytes, changed inputs/artifacts, bounded
  encode/decode, expiry, runtime-claim rejection, unknown ownership, failed builds
  and symlink cycles. All 29 foundation groups passed before the last focused
  additions; 13 final receipt/retention tests passed. Final scoped review, real
  production checkpoint reuse, fresh constrained execution, and required gates
  remain pending. M3–M6 are still pending; no phase or performance pass is claimed.

### Actual launch identity and reclaimable memory follow-up

- Opus approved the early-lifetime candidate
  `59f8ee918cd68334fea5adcdcfd9990025bff8a1`, with no blocking findings; draft
  [PR #4284](https://github.com/sifr-lang/sifr/pull/4284) remains unmerged.
  Review: `/workspace/validation-work/evidence/candidates/59f8ee918cd68334fea5adcdcfd9990025bff8a1/opus-graph-lifetimes.md`.
- A real production HIR guard executed and published complete evidence on that
  candidate, then reused it on the next invocation. Raw logs are
  `m2-checkpoint-production-fresh-59f8ee918.log` and
  `m2-checkpoint-production-reuse-59f8ee918.log` outside Git. This is one guard,
  not full-profile reuse or phase acceptance.
- The review called out untested real producer/consumer launch identities.
  Actual direct and nested `uv run` identity observations differed; raw
  `m2-lifetime-real-nested-key-comparison.log` preserves that failure before any
  expensive compiler build. This new bounded item pins the same physical Python
  executable and removes exact duplicate PATH entries while preserving first
  lookup order, empty cwd entries, and distinct directory aliases. The real
  direct/nested observations now match every input/runtime component and digest;
  raw `m2-preparation-real-key-canonicalized.log` retains the result.
- Provenance reads left roughly 8.1 GB of clean active file LRU pages charged to
  the cgroup. Counting only inactive file pages falsely reduced available memory
  below the source-stage requirement. Admission now accounts for clean active
  and inactive file LRU pages within the same limits and host availability bound,
  subtracting dirty/writeback/unevictable bytes. Tmpfs/shmem and anonymous LRU
  pages are not extra memory. The original 2 GiB memory reserve remains required.
  Actual estimated available memory is approximately 16.3 GB in the 17.18 GB
  cgroup, without changing capacity; source/package/assertion memory admission
  passes. Disk remains approximately 16.3 GB, so the existing 19.3 GB cold
  corpus/metadata and remaining-preparation admission requirements still block.
- Restoration/copy admission observations are retained in preparation receipts.
  Dangling dependency links now report infrastructure unavailability instead of
  an assertion failure. Thirty-one targeted receipt/resource checks pass,
  including real path lookup preservation, active clean cache accounting,
  dirty-page exclusion and no shmem/anonymous allowance. Final review/gates and
  constrained full execution remain pending. The shared main target still has no
  established owner and remains untouched; M3–M6 remain pending.

- Final memory/launch implementation candidate:
  `42b0f9d57da1589520422823b42b83062832f880`, draft
  [PR #4285](https://github.com/sifr-lang/sifr/pull/4285). Read-only Opus returned
  **SATISFIED**, no blockers; external review:
  `/workspace/validation-work/evidence/candidates/42b0f9d57da1589520422823b42b83062832f880/opus-memory-launch.md`.
  The committed-candidate direct/nested observation again matched all input and
  runtime components, raw `m2-preparation-real-key-42b0f9d57.log`. Final 29
  foundation groups and the file-size guard passed. These are mechanism checks;
  required implementation gates, native assertions and performance remain open.
- Current external blocker: all safely reclaimable obsolete session-owned graphs
  have been retired with compiler-byte custody intact, but cold preparation
  still requires 19,327,352,832 bytes against approximately 16,307,044,352 bytes
  available. No shared `target/debug` cleanup is authorized while its original
  ownership is unknown. The user has been asked for authoritative disposability
  information; no answer or elapsed time is treated as authorization.
  Read-only cleanup scope/resource observation and concrete proposal are outside
  Git at `/workspace/validation-work/evidence/m2-shared-debug-cache-disposability-observation.json`
  and `/workspace/validation-work/shared-debug-cache-recovery-proposal.md`.
  Next action: resolve that ownership boundary, recover only the authorized
  inactive cache if applicable, then run fresh constrained preparation/assertions
  with exact candidate-bound receipts. Preserve the unchanged reserve and all
  failed/unexecuted outcomes. The full plan remains active and incomplete.

### Owned descendant recovery continuation

- The constrained execution of `d5cedf1a3091e10adc93d33b85c32b769891eafb`
  continues in `/workspace/sifr`; its source and targets are unchanged. This
  continuation owns the sparse worktree `/workspace/sifr-validation-recovery`
  and branch `codex/validation-recovery-checkpoints-20261003`, based on that
  candidate. It does not claim that the running candidate has passed.
- Linux commands now run under a dedicated child subreaper with parent-death
  cancellation, process-identity checks, and PID file descriptors. All children
  adopted by this private supervisor belong to its command; unrelated direct
  children of the runner are neither signalled nor reaped. A private completion
  channel preserves native status and rejects unconfirmed cleanup as an
  infrastructure error. The host must support these kernel primitives; failure
  rejects command startup rather than silently weakening custody. Other hosts
  retain their existing group teardown.
- Real failure injection covers detached, SIGTERM-ignoring descendants after
  normal exit, deadline, and cancellation; unrelated-child isolation; missing
  executables; native signal status; and fatal supervisor failure. A fatal
  supervisor failure cannot prove escaped descendants were reaped and remains
  unqualified. The failure injection owns a separate subreaper for its test
  orphans. These are recovery assertions, not compiler/performance qualification.
- The initial prototype assumed `/proc/<pid>/task/<pid>/children`; this host
  does not expose it. Its failed test is preserved in the execution transcript.
  The implementation reads process relationships and start times from `/proc`
  stat records instead. The first existing deadline test failed because its
  0.2-second budget included new supervisor startup; the corrected test keeps
  total successive command duration greater than each independent deadline.
  Both failed and corrected raw outputs remain outside Git.
- Targeted process tests: 36 passed, raw
  `/workspace/validation-work/evidence/m2-recovery-custody-final.log`.
  Canonical profiles and the 900-line guardrail passed. The first full runner
  self-test encountered its pre-existing `target/` setup assumption in this fresh
  worktree; its raw failure is retained. The retry creates only this worktree's
  owned target directory; all 26 runner self-test groups then passed, raw
  `/workspace/validation-work/evidence/m2-recovery-runner-selftest-prepared.log`.
  Scoped review and implementation PR/merge acceptance still require final
  candidate evidence.
- Correctness-checkpoint consumption and M3–M6 remain unfinished. This bounded
  item changes process custody only and preserves selected assertions and all
  performance protocols. Parent startup is outside command-body timing; any
  protocol that measures the entire wrapper must bind the changed mechanism.
- Scoped review of `c08e8708082729d04590f475025adf63698637e5` returned
  `SATISFIED`, outside Git at
  `/workspace/validation-work/evidence/candidates/c08e8708082729d04590f475025adf63698637e5/opus-recovery.md`.
  [PR #4281](https://github.com/sifr-lang/sifr/pull/4281) is a draft stacked on
  #4280; neither implementation PR is accepted or merged yet.
- One follow-up batch closes review suggestions about startup signal races,
  process-group PID reuse after leader reaping, JSON expansion of non-BMP error
  text, and supervisor-generated core files. Signals are blocked across spawn
  until the supervisor installs its handlers. Linux `waitid(WNOWAIT)` reserves
  the leader's PID until group teardown finishes. Error metadata stays within
  its byte bound; only the supervisor disables its own core dump after the
  command has finished. New failure injection exercises these mechanisms,
  unavailable kernel custody before command spawn, and a live escaped pipe
  holder after supervisor death. Native status and selected work remain intact.
- Follow-up targeted tests: 42 passed, raw
  `/workspace/validation-work/evidence/m2-recovery-hardening-corrected.log`;
  all 26 runner self-test groups passed, raw
  `/workspace/validation-work/evidence/m2-recovery-hardening-runner-selftest.log`.
  The first expanded test caught non-BMP JSON expansion exceeding the pipe frame
  bound; that failure remains in `m2-recovery-hardening-expanded.log` and was
  corrected before acceptance. The changed mechanisms require scoped review
  on their new committed candidate; the earlier approval covers only `c08e87080`.

- The recovery hardening candidate
  `49fb3f47f6c1af1c556edb6cdb6ebfeb7434a10c` received `SATISFIED`, preserved at
  `/workspace/validation-work/evidence/candidates/49fb3f47f6c1af1c556edb6cdb6ebfeb7434a10c/opus-recovery-hardening.md`.
  That second review discovered an original supervisor mechanism defect:
  resetting a SIGKILL handler raises `EINVAL`, losing native `-9` and disrupting
  OOM classification. Per the closure loop, the prior item stops/rescopes rather
  than iterating again under its old approval. A separate bounded correction
  owns branch `codex/validation-native-sigkill-20261003`, based on `49fb3f47f`.
  It skips resetting unchangeable signal handlers and proves native `-9` and
  absence of a supervisor traceback with actual failure injection. It never
  infers OOM from SIGKILL alone; actual OOM counter evidence remains required.
  All 14 recovery tests passed; raw output is
  `/workspace/validation-work/evidence/m2-native-sigkill-tests.log`. The prior
  unchanged 29 process tests and 26 foundation groups remain recorded. The new
  candidate still needs scoped review and the final implementation gates.

- Mechanism remediation review of `e9b3eda6726c240f7d2741fd9bef454646212a21`
  returned **SATISFIED**, with no blockers. Evidence is outside Git at
  `/workspace/validation-work/evidence/candidates/e9b3eda6726c240f7d2741fd9bef454646212a21/opus-m1-m2.md`.
  This is a mechanism approval, not an acceptance-gate or full-phase pass.
- That candidate completed the source compiler preparation in 932.8 seconds
  (maximum observed child RSS 5.2 GiB), then failed package admission:
  required 19,327,352,832 bytes; available 16,573,632,512 bytes. The run exited 2,
  functional failed, performance inconclusive, qualified false. No sysroot
  assertion or graph retirement ran. Raw evidence:
  `/workspace/validation-work/evidence/m2-cloud-execution-e9b3eda67.log` and
  `/workspace/sifr/target/verification/execution-journals/bdc85050-cc3b-4e10-ac26-832ef5b3bff3/`.
- Measured private source graph: approximately 11 GiB total and 5 GiB of Rust
  incremental edit caches. The next bounded resource item disables incremental
  compilation for this immutable private source qualification graph only. Both
  its preparation and boundary assertions use the same producer configuration.
  Contributor/performance compilation policy, every selected assertion and the
  8 GiB reserve remain unchanged. This prospective input change needs its own
  targeted checks and review; no earlier failed run is reclassified.
- The terminated old build context is superseded by this new source build
  configuration. Before reclaiming its now-obsolete, session-owned source cache,
  archive its raw journal/report and independently retain the old prepared
  compiler bytes and hash. No process or lease may still consume the old context.
  The old assertions remain unexecuted and nonqualifying; all required consumers
  run afresh against the rebuilt compiler. Do not clean the shared main target.
- The first supported cleanup attempt safely refused the known owned graph:
  Cargo 1.98.1 requires a valid `CACHEDIR.TAG`, and the lease had created the
  target directory before Cargo could initialize that tag. No cache was deleted.
  The prepared compiler copy survived the failure. The correction initializes
  Cargo's standard cache tag only for an exactly matching owned graph; unknown
  caches are never tagged or reclaimed. A real pinned-Cargo dry-run regression
  covers the issue. The measured old debug compiler is 1,304,028,344 bytes;
  future retirement copy admission is increased from 1 GiB to 2 GiB accordingly.
- The superseded source cache is now retired through supported Cargo cleanup.
  Its independently retained compiler remains outside the Git tree at the
  candidate's evidence directory, SHA-256
  `7e39107672cb839fb43994e40f6850a895a082cd7360d06d18d24d63b5d378bd`.
  Free storage increased from the failed admission's 16,573,632,512 bytes to
  26,894,381,056 bytes (about 9.61 GiB net across the retention/cleanup lifecycle).
  The retry's cleanup alone observed 11,636,019,200 bytes recovery because the
  retained copy had already been allocated by the safely refused first attempt.
  Preserve this distinction when aggregating economics. Detailed custody and
  cleanup evidence:
  `/workspace/validation-work/evidence/candidates/e9b3eda6726c240f7d2741fd9bef454646212a21/superseded-source-cache-retirement.json`.

- Initial read-only Opus mechanism review of candidate `f03c9b324` returned
  **NOT SATISFIED**, with two valid blockers: the package compiler retention path
  lacked its host triple, and E2E worker arguments bypassed the cgroup clamp.
  Review evidence is preserved outside Git at
  `/workspace/validation-work/evidence/candidates/f03c9b324a9296709120f6503574b6a9cb408adb/opus-m1-m2.md`.
  The owned run was cancelled (exit 130); its incomplete source preparation and
  raw journal remain failures, not acceptance evidence.
- One correction batch now derives retained compiler paths from both real
  producers, checks actual host-qualified retention with a real temporary lease,
  clamps every E2E worker argument (including forwarded requests), and records
  effective workers. It also preserves inherited durations, keeps assertions'
  existing per-command deadlines, classifies observed OOM/ENOSPC/cancellation,
  moves metadata-structural cold compilation into named preparation, and keeps
  functional/performance outcomes independent after a functional failure.
- The measured stopped-run cgroup had roughly 8.99 GB charged memory, of which
  only 36.8 MB was anonymous and 6.28 GB was inactive file cache. Treating all
  charged cache as unavailable caused a false admission failure. The correction
  uses unused capacity plus clean inactive file cache, excluding dirty/writeback
  and unevictable bytes and retaining the same capacity and reserve. This is an
  availability estimate within the existing limit, not extra memory or a lower
  required workload.
- Explicit graph ownership continuity is now available for this session's own
  interrupted run. All consumers start unpassed and must run again; failed
  assertion results and partial performance captures are never reused.
  Recovery of the next run uses the retained owner
  `c151f65d-5f60-4b70-9916-ba481a7d1992`, after verifying matching worktree,
  device/inode/UID and a free exclusive lease. Unknown owners remain ineligible.
- Follow-up M2 work still includes complete correctness-checkpoint consumption
  and stronger descendant adoption/reaping: cancellation killed the owned nested
  builds but this environment's PID 1 retained dead orphan Cargo/rustc entries.
  No live detached build was observed, and those dead entries hold no live cwd
  or graph lease. Do not claim the full reaping/recovery requirement is delivered.
- Corrected candidate validation and the remediation review remain pending at
  this record. No implementation PR is merged or phase gate waived.

- Owned branch: `codex/validation-contracts-evidence-20261003`; draft
  [PR #4280](https://github.com/sifr-lang/sifr/pull/4280). M1 and this first M2
  scheduling item are implemented, not accepted or merged.
- Integrated the existing cloud foundation at `f88973102` into this branch.
  Its source PR #4259 and SQL qualification retain their pending disposition.
- Added prospective cgroup-v2 admission, separate cold preparation deadlines,
  isolated graph leases, exact retained compiler copies, supported Cargo cleanup,
  net recovery observations and immutable source/runtime-bound journals.
  The live merge assertion selection is unchanged. Step checkpoint consumption
  remains disabled until its dependency closure and accounting are proven.
- Candidate `f3c4d417e` passed 25 runner self-test groups, strict contracts,
  profile checks, the file-size guardrail and whole-candidate whitespace checks.
  Its actual cloud run exposed a compiler build inside generated-input
  acquisition. The owned run was cancelled (exit 130) before assertion execution;
  the journal records `cancelled`, not a pass. The producer preparation is now
  moved after sysroot graph retirement, with a regression checking that early
  acquisition contains only locked fetch commands.
- Raw first-run evidence:
  `/workspace/validation-work/evidence/m2-cloud-execution-f3c4d417e.log` and
  `/workspace/sifr/target/verification/execution-journals/83d7de3a-5c2d-4634-9b66-fff55ce3ee43/`.
  Preserve both when creating the corrected candidate; the run does not qualify
  correctness or performance. Full compiler execution and Opus review remain
  required. M3–M6 remain unfinished.

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

## M2 continuation — measured cache preparation and independent performance

The user's instruction to continue supersedes treating unknown shared-cache
ownership as a prerequisite for progress. The shared debug cache remains intact;
normal Cargo preparation validates and consumes its existing outputs.

At observed commit `973c821c774fc1cec4d8a33d9dc74141fe87d489`, the Rust workspace,
locks, toolchain, configuration, stdlib and sysroot inputs matched the earlier
`d5cedf1` preparation. A bounded native `package_build.py --metadata-only`
observation completed both original `cargo test --no-run` configurations in
68.16 seconds with 17,113,088 bytes net additional allocation. The observation
admitted 2 GiB growth, 8 GiB disk reserve, 6 GiB resident memory and 2 GiB memory
reserve, and watched a stricter disk floor during execution. It executed zero
test assertions, deleted no cache, and supplies preparation cost evidence only.
The external raw log is
`/workspace/validation-work/evidence/m2-warm-metadata-observation-973c821c7.log`.
This measurement does not qualify a new compiler candidate or prove every
remaining preparation fits. Prospective cache-aware admission remains necessary.

The next bounded implementation moves physical performance admission and the
measurement suites after selected correctness guardrails, areas and toolchain
checks for ordinary profiles. Host-independent frontend, LSP and policy suites
execute as correctness work. Missing performance admission blocks only
measurement and the final gate, with a separate performance outcome. Blocking
step timing verdicts persist until the final gate instead of suppressing later
correctness assertions. Cloud's existing independent qualification route remains.
No suites or cases are dropped, and required qualification remains blocking.

The first targeted regression run caught an incorrect test expectation that
all guardrails execute in manifest order; the established inventory guards run
first. The corrected check verifies the complete multiset, including duplicates,
while asserting exact area and toolchain selection. The failed run is preserved
in `m2-independent-performance-targeted.log`; the corrected run is separately
recorded in `m2-independent-performance-targeted-fixed.log` outside Git.
Full implementation gates, delivery, and M3–M6 remain open.

Opus rejected candidate `400b65edef4c3f4550be27338da898cbb6dc159c` because its
whole-area partition still suppressed the frontend/LSP correctness suites when
admission failed and misclassified their failures. The rejected review remains
outside Git at `sifr-claude.ehp26g/response.md`. The remediation partitions suites,
keeps correctness failures functional, removes reference admission from policy
and frontend-only area invocations, and reconciles both passing partitions into
the full canonical area result. Missing measurement cannot publish a complete
passing area result. The existing create-pr area timing allowance includes both
partition durations. The full original selection and numeric budgets remain.

## M2 bounded cache-admission candidate

PR #4286's corrected routing candidate is
`e64f28fc7c71edccfd6111cb35ce54da3a68075b`; Opus remediation review is SATISFIED
with no blockers. Its 46 targeted regressions and all 29 final-candidate
foundation groups passed. Review and raw evidence are outside Git under
`/workspace/validation-work/evidence/candidates/e64f28fc7c71edccfd6111cb35ce54da3a68075b/`.
The combined performance-area timing is cumulative across both partitions; the
correctness-part timing is also displayed separately and is not an additional
elapsed allocation. Running correctness before reference admission deliberately
permits correctness preparation even when that host cannot qualify measurements.

The next bounded resource change registers a 2 GiB metadata preparation attempt
when native Cargo outputs and fingerprints are present. This is an allocation
hint, not an assertion or compilation reuse receipt. Cargo still validates and
executes both original preparation configurations. A stale hint can fail the
bounded attempt; it cannot qualify omitted work or trigger an unchanged automatic
retry. Unknown cache presence retains the original cold metadata estimate.

Remaining preparation now admits each original command sequentially instead of
requiring its summed cold allocation at entry. Prospective per-command estimates
are 2 GiB with a native cache hint and 6 GiB otherwise, retaining the existing
8 GiB disk and 2 GiB memory reserves and worker bounds. Coordination admits only
its own bookkeeping, never a whole build. Completed preparation still does not
qualify any later runtime assertion. No selected preparation or assertion is
removed and no shared cache is cleaned.

The monitored commands receive an absolute filesystem floor that limits their
net growth and keeps the original reserve plus 1 GiB stopping headroom. The
owned execution loop checks that floor before spawn and while draining output;
exhaustion is a failed infrastructure outcome with owned-tree teardown, not a
pass. This is polling containment with explicit headroom, not a filesystem quota
or a guarantee against arbitrary concurrent writes. Nested stricter floors are
preserved, and the caller's environment is restored afterward.

Focused regressions cover stale/symlink/unknown cache hints, original command
execution, per-command admission, worker clamping, environment restoration,
invalid budgets, rejection before spawn and a real growth-failure injection with
a detached termination-resistant descendant. These tests establish the mechanism;
the next native cache-only observation and complete constrained workload remain
separate acceptance obligations. M1/M2 implementation gates and M3–M6 stay open.

At candidate `5681d9ad0d527dcc5791bfa0134e9f9a5f444f39`, a fresh native
metadata-only observation using the new scheduler and monitored 2 GiB allowance
completed in 69,255 ms. Its journal is
`target/verification/execution-journals/8cb9025f-1e3e-49a8-a01d-aae8feb049e5`
in the main worktree; raw log is outside Git at
`/workspace/validation-work/evidence/m2-bounded-metadata-5681d9ad0.log`.
Both original no-run configurations executed, no cache was deleted and no runtime
assertions or whole-candidate qualification are claimed.

Opus rejected that candidate because a nested pre-admission or post-execution
input failure could become a bare command failure and be misclassified as an
assertion by the coordination step. The rejected review is preserved outside Git
at `sifr-claude.NXGiZj/response.md`. The remediation propagates the original
exception from the entire admitted step, preserving classification through both
journal entries. Regressions inject admission refusal before command execution
and source drift afterward, verifying the outer entries remain infrastructure
failures. Unknown failure details fail conservatively as unavailable.

## M4 prospective level registration — work in progress

M2 cache-admission candidate `ed3f0b800ed9a4dbc3c516d104f55db47b779c4d`
passed 60 targeted tests and 30 foundation groups; Opus remediation review is
SATISFIED with no blockers. Its review is outside Git at
`/workspace/validation-work/evidence/candidates/ed3f0b800ed9a4dbc3c516d104f55db47b779c4d/opus-cache-admission.md`.
M1/M2 implementation gates and full constrained acceptance remain open.

The next prospective level contract preserves the existing five-case smoke and
ten-case representative selections. Smoke runs its original one warmup/one
observation workload checks without a physical reference and cannot qualify
numeric regression or p95 budgets. Representative keeps named-reference admission
and limits its threshold claims to the selected cases. Native `full` now selects
all 65 manifest cases instead of accidentally reusing the ten-case representative
list, and its budget checker requires complete inventory. Eligible native metric
checks do not replace shared-cloud paired qualification.

`performance_levels.json` binds the benchmark-manifest hash, selections, sampling,
reference requirements and permitted claims. Canonical partition results retain
and independently reconcile that policy. Level checks reject omissions, duplicates,
manifest drift and false smoke qualification. The shared-cloud v2 protocol remains
unchanged at all 65 cases and 5,120 fixed pairs, with independent receipt acceptance;
no cheaper numeric qualification protocol or smaller paired count is introduced.

Policy-only trend checks validate the existing policy/baseline structure without
claiming its freshness. Ordinary trend/reference qualification remains strict and
rejects stale evidence; a regression proves both behaviors. Stored timestamps,
measurements, expiry windows and numeric thresholds are unchanged. Smoke evidence
is explicitly rejected by numeric budget qualification.

Focused level/routing/shared-cloud tests pass. The broader benchmark self-test
failed in the pre-existing resistant-process-group cleanup: the old group-only
adapter left a zombie and refused another sample. Its raw log is preserved at
`/workspace/validation-work/evidence/m4-performance-levels-benchmark-selftest.log`.
`benchmark_process.py` is unchanged from the parent, so this is recorded as the
M2/M4 owned-process dependency, not a passing validation result or a level-selection
regression. A bounded production benchmark-custody repair must precede acceptance;
no unchanged retry, case removal or weakened cleanup assertion is authorized.
Generated-program metrics, paired qualification, M3 and M5/M6 remain pending.

## M2/M4 benchmark process custody — work in progress

Level-registration candidate `b0bab262557878052e2a8319af7185f043e18dc9`
is draft PR #4288. Its scoped Opus review is SATISFIED with no blockers;
the review is outside Git at
`/workspace/validation-work/evidence/sifr-claude.c6ingI/response.md`.
Acceptance gates and remaining milestone obligations remain open.

The production benchmark adapter now uses the canonical owned-process executor.
On Linux its dedicated subreaper reaps detached and termination-resistant
children before another sample can start. Missing cleanup confirmation,
cancellation, infrastructure failure and truncated capture reject the sample.
The prospective capture bound is 16 MiB per output stream. Normal native exit
codes and UTF-8 output are preserved; deadlines still have no qualified metrics.
An early-exiting command retains its actual exit code after descendant cleanup.
The wall timer includes supervisor overhead without subtraction; matched tooling
on both endpoints is required before any new paired qualification.

Shared-cloud tooling identity now includes the exact canonical executor,
supervisor, disk monitor and package initializer bytes. Endpoints missing these
files or using different bytes fail admission. Counts, budgets and AB/BA schedules
are unchanged; no historical receipt is requalified under the new identity.

The repaired broad benchmark self-test passed real cooperative, resistant,
detached, leader-exit and closed-pipe trees with PID disappearance verified from
the following sample, plus fail-closed capture/custody controls and SIGKILL status.
All 92 performance-area tests and all 30 runner foundation groups passed.
A receipt regression also rejects changed or absent supervisor bytes. Raw logs
remain outside Git under `/workspace/validation-work/evidence/m2-benchmark-custody-*`.
The initial repair test's old leader-exit timeout expectation failed and is
preserved; the corrected assertion requires actual normal exit plus cleanup.
This is not compiler throughput, whole-gate or constrained acceptance evidence.

## M3 conservative change selection — work in progress

Benchmark-custody candidate `be6b4cdc1ced25b3ee0b56776f841001f2a93cb1`
is draft PR #4289. Scoped Opus review is SATISFIED without blockers at
`/workspace/validation-work/evidence/sifr-claude.IoH9RB/response.md`.
The create-PR gate is running against its exact clean full checkout; its log is
`/workspace/validation-work/evidence/create-pr-be6b4cdc1.log`. A 9 GiB monitored
filesystem floor preserves the original reserve plus stopping headroom. No shared
cache is removed and incomplete preparation is not counted as assertions.

The next bounded selector registers `sifr_verify changes plan|run --base SHA
--head SHA`. It derives reasons and complete canonical jobs from actual exact
commit diffs. Documentation-only and empty valid diffs retain every create-PR
obligation; shared, area, unknown, malformed, unavailable and undecodable inputs
broaden to the complete merge profile. Rename detection is disabled so both the
deleted and added paths are classified. This initial conservative policy removes
no cases and makes no unmeasured speed claim. Area refinements wait for measured
preparation/assertion costs and explicit dependency mapping.

Plans retain the complete canonical profile plan and reasons for all selected
area, guardrail and toolchain jobs, and state that execution has not occurred.
Execution requires the exact named clean committed checkout; a dirty candidate
cannot acquire passing evidence under an earlier SHA. The existing profile runner
remains the execution authority. Missing diff information broadens selection;
it never becomes a documentation-only skip. Trusted CI aggregation, main reuse,
scheduled hardening and protected enforcement remain separate unfinished work.

## M3 trusted aggregate and scheduled hardening — work in progress

Conservative selector candidate `1d1e79489a43437f8a3ff5b5253a3c41950012b8`
is draft PR #4290. Scoped Opus review is SATISFIED without blockers at
`/workspace/validation-work/evidence/sifr-claude.wjrztC/response.md`.
Its three focused regressions pass. An earlier 31-group foundation run passed;
the later short-deadline failure remains recorded. An isolated diagnostic of that
unchanged control passed with outcomes 0/exit followed by 124/safety_deadline.
No failed foundation outcome is converted to a passing full run.

The next CI candidate adds actual queue-candidate checkouts and commit-bound
profile selection. Main-push and manual validation select one merge profile;
scheduled validation selects nightly. Automatic pushes no longer duplicate
create-PR, merge and conditional release qualification. Nightly schedules all five
bounded instrumented fuzz targets on the event's exact commit and retains
receipts, working corpora and minimized findings even after failure. The existing
fuzz subprocess adapter still needs canonical owned-process custody before full
constrained acceptance; scheduling is not evidence that campaigns have executed.

`validation-required` is published by a separate default-branch `workflow_run`
workflow. Candidate validation has read-only permissions and no persisted checkout
credentials. The publisher executes trusted
source and reads candidate Git objects as data. It independently derives the
required profile, fetches all jobs from the current run attempt, and checks
mandatory successes, completion freshness, repository/workflow identity and
actual candidate binding. Missing, skipped, cancelled, failed, stale, duplicated,
partial and untrusted results cannot yield success.

A selection-job immutable artifact records the actual checkout SHA and producer
run/attempt. The publisher verifies GitHub's artifact digest, exact bounded ZIP
inventory, run provenance and candidate identity. It rejects changed PR heads or
bases and regenerated merge candidates that differ from executed evidence.
Candidate workflow bytes must equal the trusted default-branch definition;
workflow changes therefore need a deliberate reviewed trust bootstrap before
protected enforcement. No candidate checkout or executable artifact runs in the
privileged publisher. Missing API/publication evidence leaves the required check
absent and cannot pass.

Ten focused policy/publication/artifact controls pass, as do workflow regressions,
46 uv pin controls, uv invariants and size/whitespace checks. Branch rules are not
yet changed: trusted definitions must first be delivered and observed on main.
Main-push evidence reuse, measured finer selection, real scheduled campaigns,
platform qualification and external enforcement remain open. No CI run or full
acceptance is claimed by these local controls.

The exact benchmark-custody candidate's create-PR gate stopped during preparation
at its declared filesystem floor after 894,904 ms. Its failed raw log and report
are retained at `/workspace/validation-work/evidence/create-pr-be6b4cdc1.log` and
`/workspace/sifr/target/validation_lane_reports/create-pr.latest.json`.
No qualification assertions ran and no cache was removed. This is failed gate
evidence, not a passing preparation receipt. Remaining implementation continues;
no unchanged gate retry or reduced workload is authorized by this result.


The aggregate's first scoped review rejected candidate
`97164d12460d5dbe6548010f552f555bb652b42b`: PR checks must qualify the current
PR head while binding the executed synthetic merge candidate, and a same-named
GitHub Actions job could impersonate an aggregate from the Actions app. The
rejected review is preserved at
`/workspace/validation-work/evidence/sifr-claude.jdlXds/response.md`.

Remediation posts the PR context on the current head and retains the verified
merge candidate in its external identity and summary; merge-group/main checks
remain on their actual candidate. Twelve focused controls pass, including a real
publisher-path head/merge distinction and rejection of the Actions integration.
The publisher's Actions token is now read-only. A separately installed GitHub App
with Checks/write is mandatory for publication, minted through the pinned official
App-token action and limited to this repository. The returned check must identify
the declared separate App. Missing credentials cannot produce a protected pass.
Before enforcement, configure `VALIDATION_CHECK_APP_ID`, the trusted publisher's
`VALIDATION_CHECK_APP_PRIVATE_KEY`, and pin that exact App integration ID in the
required-check rule. No App, secret or rule is provisioned or claimed by this
implementation; credentials are never copied into repository evidence.

Failed-jobs-only reruns do not qualify: all mandatory jobs and the candidate
artifact must come from one complete current attempt. Fork runs lacking one
unambiguous PR snapshot fail closed and need a separately implemented trusted
admission route before their qualification can be claimed. Anonymous commit
fetch is sufficient for this public repository; private-repository support is
not claimed. Workflow trust bootstrap and separate App provisioning remain
explicit prerequisites for protected enforcement.


## M3 sustained fuzz custody — work in progress

The instrumented fuzz adapter now invokes the canonical bounded process owner for
preflight, builds, campaigns, minimization and replays. It retains actual native
exit status and counters from bounded output, reaps detached resistant children
before returning, and records infrastructure/cancellation/capture failures
separately from compiler findings. Incomplete capture cannot qualify guided
executions, a stable finding or a project-tree export. The prospective capture
bound is 16 MiB per stream; the retained human-readable tail remains 8 KiB.

Eighteen focused controls passed, including a real detached resistant timeout and
PID absence in a following command, missing custody and capture/cancellation
rejection, preserved counters outside the bounded tail, existing minimized-seed
replays and all current frontend/diagnostic/project target contracts. This is
adapter coverage, not execution of five real instrumented campaigns. The budget,
engine version, corpus inventory, seed preservation and two-replay obligations
remain unchanged. Raw evidence is outside Git at
`/workspace/validation-work/evidence/m3-fuzz-custody-targeted.log`.

Trusted aggregate remediation `6387097081ea1b03b3c1c2c243462562b4b488c3`
passed scoped Opus review with no blockers; review is outside Git at
`/workspace/validation-work/evidence/sifr-claude.OE7Ot6/response.md`.
All 31 foundation groups passed for the sustained-fuzz adapter with Cargo idle;
its earlier concurrent short-deadline failure remains historical failed evidence.
Separate App credential custody and main reuse are the next bounded M3 item.

## M5 actual published predecessor acquisition — work in progress

Sustained-fuzz custody candidate `3b3e2cb54b9ca42688654bc5b051da5d94237bd9`
is draft PR #4292; its scoped Opus review is SATISFIED with no blockers at
`/workspace/validation-work/evidence/sifr-claude.DGIVbw/response.md`.
Protected App-environment custody candidate
`4fd667541296fc0afbcc8518b715e6c47a6f8a32` is draft PR #4293 and passed
scoped review at `sifr-claude.wxZnp5/response.md`. Environment creation with the
current token returned HTTP 403; the user was asked to configure the environment
and separate App privately in GitHub. No credential or enforcement deployment is
claimed, and implementation of independent remaining milestones continues.

The published-predecessor registry records all four real beta.16 native archives
from GitHub release 368902044, published 2026-08-11, publication source commit
`11581e0630407d397079c032d0cd30fb87f4795e`. Fetching verifies the registered
public URL, exact byte count and SHA-256; decoded allocation is measured from the
verified tar inventory, with block/directory allowance and retained reserve.
Escaping entries fail before extraction, and extraction checks the reserve at
entry boundaries. Preparation records zero runtime assertions and preserves
failed/incomplete attempts; candidate versions must follow this real predecessor.
This registry does not claim first-release inapplicability or silently select a
same-source synthetic predecessor.

The actual Linux x86_64 archive (73,787,525 bytes, SHA-256
`8d796c321cc2b5a898c5e751c6769154b068060a69ae3aa302051e4fa9bb5448`)
was downloaded, independently hashed and decoded to 20,386 files totaling
414,570,775 bytes under
`/workspace/validation-work/published/beta16-linux-predecessor/`.
Its manifest binds beta.16, the native target and the same published source commit.
The installed binary reports beta.16 and hashes to
`d4d781ddaead3aa71139dec1a32c3692773ea2787f0c3f60d402fffb36c3bad1`.
The archive and per-file custody receipt remain outside Git. Later stricter
admission changes are separate from that historical acquisition observation.

The old published CLI lacks `--print compiler-identity` and `doctor
--verify-integrity`; those failed probes remain in
`/workspace/validation-work/evidence/m5-published-predecessor-native-identity.log`.
Published predecessor qualification must use its declared older supported CLI
and independently verify archive/package hashes; it cannot fabricate a newer
embedded identity or relabel these failures. Five focused controls pass for real
version ordering, complete native registry, exact transfer/hash bounds, safe
extraction, reserve refusal and preserved failed preparation receipts. Actual
upgrade/reinstall/rollback, candidate qualification, all required native platforms
and M6 acceptance remain unfinished.

## M4 compiled-program observations — work in progress

The registered `generated-program-observations-v1` protocol separates compilation
from native process startup/throughput, actual CPU time, peak RSS and executable
size. Its two correctness oracles have fixed smoke/representative/full counts
(1/10/25 measured processes, with 0/2/2 warmups). Empirical nearest-rank p95 is
computed over process observations; every observation has one inner timing
sample. GNU Time's 0.01-second wall resolution and launch overhead are explicit.
Unavailable allocation instrumentation remains unavailable, never zero. This
observational protocol makes no numeric regression qualification claim and cannot
replace the required independent 65-case/5,120-pair shared-cloud v2 protocol.

Preparation requires the clean committed compiler receipt, actual Cargo JSON
application release profile, actual rustc CPU-target arguments, independent native
executable hashes, resolved runtime-library hashes and retained compilation logs.
Capture uses the canonical process supervisor, rejects output/status/artifact
changes, preserves failures, and verifies completion freshness and recomputed
raw-file counters, process counts and summaries. Canonical performance rules now
execute its contract controls. These controls compile a clearly test-only C
fixture, not Sifr: no native Sifr preparation or qualification is inferred.

Targeted controls passed 7 tests; the complete performance area passed 100 tests.
Profile validation and the 900-line source guard passed (4,395 files). The first
control attempt failed because GNU Time was absent; it remains preserved. A
session-local GNU Time 1.10 was then extracted from Debian's public package
`time_1.10-0.1_amd64.deb`, SHA-256
`4b789fd1edea74d9d95dab1c7eff3f062a844f9b541e0a776f2d93a19e7295ab`.
Logs are outside Git under `/workspace/validation-work/evidence/` with prefix
`m4-generated-program-`. Native observations, reliable allocation instrumentation,
independent comparisons, broad acceptance and final phase closure remain pending.

The published-predecessor acquisition candidate `e1724df488d239abbf1d53488e1d68f21d0a6d4c`
received a SATISFIED read-only Opus review at
`/workspace/validation-work/evidence/sifr-claude.RzPTmU/response.md`. This scoped
review covers acquisition and controls; actual upgrade and release-platform
qualification remain unfinished.

The first compiled-program observation review at `dca288a19f6bb0b8e9ffd699a98bab62386ff8a8`
was NOT SATISFIED: its independent checker did not re-derive application release
profiles and CPU flags from retained command events. The review remains at
`/workspace/validation-work/evidence/sifr-claude.XlccNX/response.md`.
The correction verifies complete, case-bound byte ranges in both JSONL event
files, their SHA-256 hashes, the actual Cargo executable, its release profile and
an exact, nonconflicting rustc `target-cpu=generic` argument. Each program uses a
separate private application cache so later preparation cannot overwrite the
first program's actual Cargo executable before independent checking. Negative
controls include rehashed non-release events, a rehashed wrong CPU target,
missing profile/command data and changed raw logs. Captures now retain an initial
incomplete receipt and leave the successfully checked completion bytes unchanged.
GNU Time below 1.8 is rejected because its older peak-RSS scaling is unreliable.
The first corrected full-area run failed because the timer identity test used
an unparseable test-only version label. Its failed v2 log remains preserved.
The control now uses an explicit supported version; the corrected v3 full-area
run passed 101 tests. Earlier runner foundation coverage passed all 31 groups. Broad candidate gates and native Sifr
qualification remain pending.

## M4 rescoped actual application-artifact identity repair

This bounded repair addresses the stopped observation item's real-producer
mismatch. Selection follows actual Cargo bin target/executable identity and the
generated manifest, and requires exactly one matching rustc bin invocation.
Ambiguous or unrelated-crate evidence cannot substitute. The controls now run a
real Cargo/Rust binary with a hash-suffixed target, rather than inventing that
event shape. A real Sifr startup producer control also compiled and ran `42`:
`/workspace/validation-work/evidence/native-program-dc371fc6-c0b3-4964-84dd-7e3a76f67193/`.
It records the actual opt-level 3/overflow-checking Cargo artifact,
`sifr_output_c1bab39743a391ac`, generic rustc CPU arguments and delivered hash.
This is explicitly a prototype producer control against compiler source
`dca288a19f6bb0b8e9ffd699a98bab62386ff8a8`, not final candidate qualification.
An earlier control failed because `--offline` was applied inside the repository
package without its package lock; that failure is preserved separately.

Program preparation copies registered source bytes into an owned standalone
location, avoiding accidental inheritance of the repository's Sifr package.
Cargo remains explicitly offline. Contributor compilers use their receipted
source sysroot; installed product compilers retain their packaged authority.
The separately registered, explicitly selected constrained observation allocation
uses one GiB of prospective growth after the actual startup graph measured 123
MiB, a two-GiB reserve and one-GiB monitored stopping headroom. The standard
allocation and mandatory compiler shared-cloud qualification obligations remain
intact. Process observations also bind the actual stable cgroup/CPU/memory limits.

The full performance controls passed 104 tests, including the three real-Cargo
identity/ambiguity/codegen-alias controls. Profiles and source size passed (4,397
files). Native preparation/capture at the corrected clean candidate, required
allocation instrumentation, broad gates and numerical compiler qualification are
still required before M4/phase acceptance.

A separate compact source-compiler observation passed a cold locked/offline build
in 586.90 seconds with 1,915,352,054 graph file bytes. It used explicit
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`, two Cargo workers and its own
leased graph. Actual Cargo retained opt-level 1, assertions and overflow checks;
no functional assertion or performance comparison is inferred. The shared
25-GiB debug cache was preserved. Its resource/raw Cargo record is outside Git at
`/workspace/validation-work/evidence/compact-438a5635-1c78-46ed-b771-c826413d339f/`.

## M2 explicit compact source-profile scheduling

The actual default create-PR attempt at `be6b4cdc1ced25b3ee0b56776f841001f2a93cb1`
failed its declared disk floor during generated compiler preparation, before any
qualifying area assertions. The historical log remains
`/workspace/validation-work/evidence/create-pr-be6b4cdc1.log`.
The next bounded repair introduces an explicitly selected compact policy for
Linux source profiles, based on the retained debug=0/incremental=0 cold compiler
observation above. It preserves every canonical selection and default policy;
prospective allocations retain 2 GiB disk reserve and at least 1 GiB monitored
stopping headroom. It rejects conflicting compiler settings and records the
configuration in execution identity. Sysroot graphs receive session UUID paths,
retire only with their matching leases, and their required runtime consumers run
before remaining preparation. Only selected metadata configurations prepare.
Generated compiler preparation now matches assertion-time offline configuration.
The policy changes resource scheduling, with no build/assertion reuse claim and
no waiver of numeric regression acceptance.

Five controls passed for selection equality, conflicting settings, actual shared
producer/consumer paths, private-graph retirement with shared/other-session cache
preservation, exact selected metadata preparation and failure blocking.
An initial foundation attempt failed because the newly created sparse worktree
had no `target` directory; that failure remains recorded. The directory was
created before retrying. The corrected foundation run passed all 32 groups; profiles and the source
size guard passed. Broad native gates and this repair's scoped review are
pending. The user authorized continuing through successive bounded items.

The repaired native program candidate `b5b5e382d8427e4ebb90693aa9a1b198c105d103`
received SATISFIED Opus review in
`/workspace/validation-work/evidence/sifr-claude.gmFV1n/response.md`.
Clean native preparation, full 54-process capture and independent checking passed:
`/workspace/validation-work/evidence/generated-programs-b5b5e382d/prepared.json`,
`/workspace/validation-work/evidence/generated-program-full-b5b5e382d/receipt.json`;
logs `m4-native-preparation-b5b5e382d.log`, `m4-native-full-capture-b5b5e382d.log`
and `m4-native-full-check-b5b5e382d.log`. These descriptive observations include
startup, throughput, CPU, RSS and binary size. Allocation metrics remain unavailable;
independent compiler comparison, broad gates and M4/phase acceptance remain pending.

## M3 trusted publication integration guard repair

The actual compact create-PR gate at `83e25b1cdf243a3251d6db2e7c993c9b0cb0e8e8`
stopped before preparation: the submodule ownership policy had not classified
the new trusted publisher checkout. Its failed observation and native logs remain
at `/workspace/validation-work/evidence/create-pr-compact-83e25b1cd/`.
The bounded integration repair classifies both the publisher's exact workflow-SHA
checkout and the candidate selector's exact merge/source-SHA checkout by full
workflow/job/condition/input identity. Wrong workflow, job, condition, ref, path
and repository controls remain rejected. Neither job compiles submodule source.
It also integrates the independently reviewed main-only protected publication
environment implementation at `4fd667541296fc0afbcc8518b715e6c47a6f8a32`
(Opus SATISFIED, `sifr-claude.wxZnp5/response.md`). Its credentials remain an
external dependency: environment administration returned HTTP 403, and no App
key or branch-rule enforcement was configured by this session.

Submodule policy/self-tests and local-first policy/self-tests passed. An initial
publisher unit-test invocation used system Python outside the pinned verification
environment and failed importing `sifr_verify`; the preserved v2 invocation uses
locked uv. Publisher tests and environment controls passed there. Native gates,
this repair's scoped review and final required-check enforcement remain pending.

## M2 compact policy review correction

The initial compact scheduling review at `83e25b1cd` is NOT SATISFIED:
`/workspace/validation-work/evidence/sifr-claude.Ig6auj/response.md`.
It identified an in-scope policy-selection defect: inherited internal scheduling
variables could lower the default reserve without the explicit compact flag.
The one-batch correction rejects a policy variable without the explicit option,
rejects conflicting policy values with it, and rejects any inherited graph
session so the owned scheduler assigns the UUID. Negative controls cover absent
options, empty/unknown/default/compact values and inherited sessions.
Foundation tests construct independent mock profiles, so their subprocess isolates
these two outer scheduler variables and restores them on both success and failure;
this does not modify the outer native profile's environment. Six compact controls
passed, and all 32 foundation groups passed with both internal variables inherited
by the foundation command. Logs `compact-controls-remediation-final.log` and
`compact-foundation-remediation.log` remain outside Git. Remediation review and
native final-candidate gates remain pending; the ongoing native attempt on the
prior integration candidate is frozen and recorded separately.

## M4 historical second review and rescope

The second compiled-program review at `63ef5ebb51649217aacf6635510a7c559443f2bf`
is NOT SATISFIED, preserved outside Git at
`/workspace/validation-work/evidence/sifr-claude.1s5cW6/response.md`.
It found a new mechanism defect: preparation/checking use a literal application
name even though Sifr generates a hash-suffixed Cargo binary name. The passing
controls used hand-written events and did not cover that real producer shape.
Per the phase-closure-loop skill, this observation item stops unaccepted and is
rescoped. Its source/reviews and failed evidence remain intact. A separate repair
must derive the delivered application's actual Cargo executable/target and
matching rustc bin invocation, reject ambiguity/substitution, and validate the
path against a real Sifr native build before any preparation/capture acceptance.
No observation PR may merge or satisfy M4 until that repair and native evidence
are delivered. Work on published predecessor transitions continues independently.

## M5 published flat installation and explicit migration primitive

The registered beta.16 installer is the actual published asset 510759033,
19,617 bytes, SHA-256
`b28fb92b0344fe3938b797d41416f707f3de63856021c0fa1f70ef467feb8f9a`.
The bounded native rehearsal executed those unchanged bytes through an allowlisted
local transport serving only the independently verified published archive. Install
and forced reinstall both passed, all 20,386 managed payload files matched the
archive after each, and the actual predecessor passed its supported version,
sysroot JSON and doctor JSON assertions. Its receipt and native commands remain
outside Git at `/workspace/validation-work/published/beta16-native-install-rehearsal/`.
The producer refuses foreign hosts, altered inputs, extra/missing/changed payloads,
links and incomplete native processes, preserving failed receipts and raw output.
Its claim is only published predecessor installation/reinstallation and payload
identity; it cannot qualify candidate upgrade or modern CLI identity protocols.

The candidate installer continues to reject mutable layouts by default. The
one-time migration primitive requires `SIFR_MIGRATE_LEGACY=1`, the standard
root/bin layout, complete regular payloads, and the matching schema-v2 receipt,
compiler version and sysroot version/target. This is an explicit owner declaration:
the owner must first stop every old compiler/LSP. An installer lock alone cannot
make a mutable old compiler root safe. The old payload and exact receipt remain
retained under a legacy generation. Rollback is armed before the first rename,
discovers entries by actual presence, and restores the flat payload and receipt
on partial migration, interruption, or failure after publishing the new receipt.
Downgrade to an old flat installer requires a separate empty root; overriding the
immutable installation with those old bytes is not supported or claimed.

Synthetic migration controls passed default refusal, successful migration and
old-payload retention, missing-member refusal, wrong receipt target/root, nested
link refusal, a receipt-publication failure and interruption after a rename.
Existing immutable install/version controls also passed. All nine published
predecessor/acquisition/installation controls are enrolled in the distribution
case inventory and passed. Initial prototype/control failures remain retained,
including a minimal unit fixture missing required managed members; that fixture
was completed before the passing run. Logs `m5-migration-controls-final.log`,
`m5-published-contract-controls-final.log`, `m5-existing-install-version-controls.log`
and `m5-published-native-install-rehearsal.log` remain outside Git.

Candidate upgrade against this actual predecessor, representative persisted
state, candidate reinstall/rollback, all four native platforms, compatibility
qualification, artifact promotion custody and broad gates remain required.
This primitive does not complete M5, publish a release or satisfy final acceptance.

The first flat-installation/migration review at `0dc8026ce` is NOT SATISFIED:
`/workspace/validation-work/evidence/sifr-claude.AmrVmu/response.md`.
It found that a POSIX signal handler invoking cleanup without exiting could
restore the payload and then continue migration with rollback disabled. The
initial interruption control's nonzero injected command had masked that defect.
The one-batch correction uses EXIT cleanup plus explicit HUP/INT/TERM exits.
Signal controls now inject TERM while their commands return zero, requiring the
installer itself to exit 143. They cover interruption after a legacy rename,
after stage copying and after selector publication, with exact old flat payload
and receipt restoration. Matching directory/file types and rejection of FIFOs or
other special files strengthen complete-payload preflight. Temporary control roots
are canonicalized for hosts where TMPDIR has symlink ancestors.

The corrected migration controls, existing immutable install/version controls and
all nine published-contract controls passed. Logs are
`m5-migration-remediation-final.log`, `m5-immutable-version-remediation.log` and
`m5-published-contract-remediation.log` outside Git. Remediation review and the
larger native transition/platform/broad-gate obligations remain pending.

## M2 compact cache-presence forecast refinement

The native gate at `5203f15a083ecc3d07e170e73098b359bf5ea864` passed source
preparation in 1,197.74s, retained the 107,012,216-byte compiler and verified its
compressed/decoded hashes, then safely retired its UUID-owned graph. It recovered
1,723,006,976 net filesystem bytes. The gate stopped at metadata admission before
executing that command: required 5,368,709,120 bytes versus 5,254,221,824 available.
Its failed gate observation/logs remain outside Git under
`create-pr-compact-5203f15a0/`; no runtime suite qualification is inferred.

An independently bounded diagnostic ran the exact metadata-structural preparation
against this session's existing source compiler cache, with debug=0/incremental=0,
two workers, offline Cargo and the three-GiB monitored floor. It passed in 457.74s,
adding 869,552,128 net filesystem bytes; zero runtime assertions were executed.
The command/limits/raw logs are retained under `metadata-structural-5203f15a0/`.

The next bounded refinement uses partial compiled-library presence only for the
explicit compact policy's existing one-GiB cached growth estimate. Every selected
Cargo package needs its own regular rlib and fingerprint for grouped test commands.
The exact generated-graph preparation wrapper can use the compiler-presence hint;
unknown interpreter/module/options/revision shapes cannot. Cargo always executes
and validates all native fingerprints, and changed/incorrect hints can only fail
at the disk floor, never pass an unexecuted assertion. Default policy hints and
allocations are unchanged. Unknown compact remaining commands use a prospective
two-GiB growth attempt with the same reserve/headroom; this is no capacity promise.
Six compact controls and all 32 foundation groups passed, including new partial
library/group/wrapper/symlink controls. Source whitespace keeps foundation below
900 lines. Native final-candidate gates and scoped refinement review are pending.

Compact remediation `22f73c131` received SATISFIED Opus review at
`/workspace/validation-work/evidence/sifr-claude.PMiCAm/response.md`.
Publisher integration `5203f15a0` received SATISFIED review at
`/workspace/validation-work/evidence/sifr-claude.sCMDiE/response.md`.
Published migration remediation `977fde39f` received SATISFIED review at
`/workspace/validation-work/evidence/sifr-claude.NcstYN/response.md`.
These bounded approvals do not qualify the failed broad gates, pending allocation
observations, independent compiler comparison, actual upgrade/platform coverage,
main evidence reuse, administrative enforcement or final phase acceptance.
## M4 separate generated-program allocation observations

The bounded allocation protocol builds a separately identified instrumented
release/generic artifact from the actual prepared generated Rust. System allocator
counters record successful allocation/reallocation request counts and requested
bytes plus deallocation calls during generated main. Reporting overhead, libc,
loader and post-main cleanup are excluded. These single-threaded workload counts
cannot be transferred to the uninstrumented timed artifact or numeric regression
qualification. Instrumentation and prospective policy bytes are registered.

Preparation now hashes generated Rust and rejects source substitution. Collection
retains original/transformed Rust, manifest/lock/config, actual Cargo/rustc events,
artifact/dependency hashes, bounded process logs, independent output oracles and
raw counters. An independent check runs before immutable success publication;
failed attempts retain state without a success receipt. Real Rust allocator
controls cover allocation, zeroed allocation, reallocation, deallocation and
reporting failures. A real Cargo collector control explicitly stubs only its CPP
identity boundary; it does not qualify the Sifr frontend. All 108 performance controls, profile contracts, file-size
guardrails and diff checks passed; the retained control log is
`/workspace/validation-work/evidence/allocation-performance-controls-final.log`.
Actual fresh Sifr preparation/collection, scoped review and broad gates remain
pending, as do independent paired compiler performance and final acceptance.


The first allocation review at `905f808084219a7187fa91e7e9727fa40a561c19`
is NOT SATISFIED, retained at
`/workspace/validation-work/evidence/sifr-claude.3fXfa5/response.md`.
Fresh actual Sifr preparation and allocation collection completed at that candidate,
but review found missing generated package inputs: the collector omitted build.rs
and .sifr-cargo-resolution, losing the original loader link argument. Those
observations remain historical evidence and do not qualify the corrected recipe.
The one-batch correction preserves every regular package input except the rewritten
main and Cargo outputs. Collection and checking compare the actual original and
instrumented Cargo profiles and rustc arguments, normalizing only Cargo application
hash fields and relocated output/dependency paths. Loader link flags remain exact.
The real Cargo control now supplies a loader build script and resolution marker;
a missing loader script must fail collection without publishing a success receipt.
Corrected native recollection and remediation review remain pending.
## M2 memory-backed validation storage

The host exposes an 8.8-GiB tmpfs at /tmp in addition to the workspace filesystem.
The bounded scheduler correction resolves the actual Linux mount for its worktree
and charges prospective build/retained-copy growth on tmpfs or ramfs against the
same cgroup memory budget as process RSS and other temporary storage. Disk reserves
remain unchanged. Controls cover nested disk mounts, escaped mount names, missing
mount identity and a RAM-backed stage that disk capacity admits but shared memory
rejects. Eighteen resource controls passed; foundation, native gates and scoped
review remain pending. This enables using owned temporary capacity without
cleaning shared targets or treating advertised RAM storage as extra memory.


## M4 independent reference measurement runtime

The independent merged compiler reference remains
`5fbeee50c2f70abd47e5bfcb98d1f1cd0b978042`. Inspection before endpoint preparation exposed
an admission mismatch in the tooling protocol: paired captures require identical
reviewed process-custody bytes, but the historical-reference validator allowed only
the performance directory and rejected those same runner runtime files. The bounded
correction declares the exact four measurement-only runner files already hashed by
paired capture. Both endpoint tooling identity and historical-reference admissible
diff use that same list. Compiler, lock, toolchain, dependency gitlinks, Cargo setup
and general profile runner changes remain forbidden. Controls and scoped review
are pending. No reference build, pair capture or performance acceptance is inferred.


The first reference-runtime control run failed only the actual Cargo allocation
control's disk admission: required 3,355,443,200 bytes versus 3,314,692,096
available. Its control hard-coded the workspace parent for temporary outputs,
ignoring the available TMPDIR storage. The bounded fixture correction uses the
canonical configured temporary directory; its collector/admission requirements
are unchanged. Failed control output remains at
`/workspace/validation-work/evidence/reference-runtime-performance-controls.log`.


## M4 independent reference input-closure follow-up

Shared measurement runtime `1d3696c0c03755544954ac8365c5e20baef4cc8c` received
SATISFIED scoped Opus review at `sifr-claude.ZojzFY/response.md`; 109 performance
controls passed at `reference-runtime-performance-controls-v2.log`. Review raised
a separate historical-reference blind spot: Git rename detection could hide a
compiler deletion when its replacement path lies inside allowed tooling.
The bounded follow-up disables rename folding and requires a committed clean
reference worktree. A real Git control moves unchanged compiler bytes into the
allowed tooling directory and requires rejection, then restores the compiler,
accepts an exact measurement-only overlay and rejects uncommitted tooling bytes.
Targeted controls, review and actual reference preparation remain pending.

### M2 structural-only assertion allocation — 2026-10-04

The frozen native create-pr compact run at `15f3944dba9cce1702622684d455abc8144cd369` completed source and metadata preparations, then failed memory admission before any area assertions: 11,811,160,064 bytes required versus 11,520,053,248 available. Its failed evidence remains in `/workspace/validation-work/evidence/create-pr-tmpfs-15f3944db`. This was a shared-memory allocation failure, not a disk blocker.

A separate exact-head structural diagnostic passed both selected Rust tests and the metadata-doctor sequence in 28.72 seconds with 247,361,536 bytes maximum process RSS and zero swaps. Raw timing, suite results, and doctor evidence are retained in `/workspace/validation-work/evidence/metadata-structural-15f3944db-v3`. Earlier diagnostic environment-mismatch failures remain failed. The successful attempt used the original preparation's exact PATH and profile settings; receipt identities were not weakened or rewritten.

This bounded item adds a prospective structural-only allocation: two GiB resident peak, 512 MiB additional tmpfs, 256 MiB filesystem growth, and unchanged disk/memory reserves and monitored headroom. It applies only to the exact `metadata-structural` selection in compact execution. Broader, mixed, and unknown selections retain the generic allocation; source preparation remains mandatory because the doctor consumes its actual compiler. No assertions, fixture selections, deadlines, or freshness checks are removed. Targeted controls cover selection preservation, conservative fallback, and the observed capacity boundary. Full candidate create-pr and final merge qualification remain required.
## M5 single-target native installer fixture

The bounded generator option `--qualification-target` produces an explicitly
labelled single-target installer from one independently verified native archive.
It rejects unknown targets and unlisted installation hosts. Default generation
still requires all four supported target archives; it cannot infer absent native
qualification or fabricate foreign platform packages. This fixture is for actual
published-predecessor transition rehearsal before all-platform qualification.
The new control verifies the default missing-archive refusal, unknown-target
refusal, matching-target native installation and rejection on another target.
Its first run expected a different error phrase and failed; the observed
`unsupported target` refusal is preserved in `m5-native-installer-fixture-controls.log`.
The corrected control passed at `m5-native-installer-fixture-controls-v2.log`.
Existing installer/migration controls, scoped review, actual native candidate
upgrade/persisted-state/reinstall/rollback and platform acceptance remain pending.

### M5 actual native candidate preparation — 2026-10-04

This bounded item prepares one actual native optimized candidate with the canonical release build/package script and the accepted single-target qualification installer generator. It binds the source commit, locked dependency input, actual selected Cargo/rustc/Python bytes, raw native Cargo executable/profile record, packaged compiler, archive/checksum, generated installer, commands and completion. It admits the owned graph before compilation, retains raw failures, independently checks a pending record before exclusive receipt publication, and retires only its successful Linux graph after preserving the compiler. Dedicated Darwin hosts use measured VM capacity; unknown storage/hosts fail closed and their graphs remain retained. No release or version is published.

Five custody controls reject substituted commands, optimization, compiler manifest/source/target, tool identity, locked input, raw output, archive checksum, installer, incomplete processes, unpublished records and failed producer state. These use explicitly synthetic unit bytes and make no native execution claim. An artifact-area case runs those controls with the pinned interpreter. Existing published-installation controls still pass. Actual optimized native preparation, published-predecessor migration, representative persisted user state, reinstall/rollback, and all four native targets remain pending; this preparer is not qualification evidence and always reports zero runtime assertions.

The initial native preparer review found an unexecutable enrolled shell case, a noncanonical compiler CPU flag, and GNU-only Darwin storage inspection. One remediation batch makes the case executable and verifies it through the actual distribution runner, removes the extra CPU flag, and uses BSD `df` plus `diskutil` plist device/filesystem/bus evidence. Darwin requires an explicit dedicated-host operator declaration; RAM disks and unknown storage buses are rejected. Six candidate controls and three capacity controls now cover the corrected recipe, preserved build failures without success publication, failed completed-state rejection, actual cgroup delegation, BSD device queries, and shared/unclassified/RAM-backed Darwin refusal. These are unit observations, not foreign-platform execution.

### M2 preparation coordinator disk-floor correction — 2026-10-04

The native compact create-pr run at `81bc5ca1ecb07e2e24d08dac78db37093b496e3e` passed source/compiler preparation, metadata preparation and the complete structural suite. It then failed generated Cargo preparation with 5,726,146,560 filesystem bytes still free. Its child had a prospectively admitted two-GiB allocation, but inherited the enclosing coordinator's 5,726,732,288-byte floor, derived from the coordinator's 256-MiB allowance. The failed raw run remains `/workspace/validation-work/evidence/create-pr-tmpfs-81bc5ca1e`; it is not a full profile pass.

The coordinator now performs capacity admission without imposing a cumulative growth floor over its independently admitted and monitored children. Every child retains its original allocation, reserve, stopping headroom and any stricter caller floor. Nested controls prove that a child can grow beyond the coordinator estimate within its own admitted allowance, while a stricter caller floor still rejects the attempt and is restored afterward. Seven compact and nine cache/admission controls passed. This does not change compilation, case selection, assertion reuse or source preparation identities.

The session retired an obsolete successful owned compiler graph after exact retention/restoration of its compiler path, reclaiming 2,590,064,640 bytes before the preserved raw compiler restoration. Failed cleanup-cwd evidence was retained separately. Two clean obsolete owned review worktrees were retired after retaining their ignored target data; their committed branches and remote refs remain available. The independent historical compiler baseline is now preparing into a separate leased disk graph, with all reserves unchanged, rather than adding that graph to the shared RAM filesystem. Its earlier unadmitted and launcher failures remain failed.

### Native execution and completed baseline — 2026-10-04

The compact create-pr invocation at 81bc5ca1ecb07e2e24d08dac78db37093b496e3e
failed after source preparation and metadata structural assertions passed.
It ran for 1417 seconds; the structural area completed in 32.468 seconds with
actual Rust tests and doctor recovery checks. Remaining preparation failed when
the parent's cumulative coordinator floor overrode the child's admitted floor.
Raw output, journal and failure observation remain under
/workspace/validation-work/evidence/create-pr-tmpfs-81bc5ca1e. The separate
structural diagnostic passed with a 247361536-byte maximum RSS. These results
qualify neither the complete create-pr profile nor a later candidate.

PR 4310's accepted coordinator correction removes only the outer cumulative
monitor; child admission, reserves, monitoring and stricter caller floors remain
mandatory. Its first foundation run failed a pre-existing half-second process
control under a concurrent compiler build. The serialized foundation rerun now
passes, retained at
/workspace/validation-work/evidence/coordination-floor-foundation-serialized.log.
The original failure remains recorded. Review is SATISFIED at
/workspace/validation-work/evidence/sifr-claude.8OhqbB/response.md for
54e4211bdd351006f3825f3b0e746b84a6f7b584. No broad corrected profile has passed.

The independent baseline's compiler and frontend helper are prepared at
1e81537db6046dbf7381f193152bcf4e7c89ec02, preserving the historical compiler
5fbeee50c2f70abd47e5bfcb98d1f1cd0b978042 with only reviewed measurement overlays.
The successful continuation observation is independent-baseline-1e81537db-v3;
its previous disk-floor failure remains failed. After verifying both output
binaries, the completed owned preparation graph was retired under its original
lease, retaining both binaries and raw cleanup evidence. This recovered
2242191360 bytes. Endpoint copies and their receipt identities are unchanged.
No paired compiler performance invocation has been captured.
### M5 actual published transition qualification — 2026-10-04

This bounded item adds a native rehearsal from the registered, hash-verified
Beta 16 archive and installer to the exact optimized candidate bundle. It creates
representative user-owned source, project configuration and persisted state,
executes the program before and after migration and reinstall, checks package
integrity, preserves the published payload, and injects failures after the
transaction switch to verify restoration of the published flat installation and
of the exact current candidate generation. It does not claim a supported version
downgrade or publish a release. Eight required runtime cases and sixteen exact
commands must complete; an independent checker validates raw output, oracles,
coverage, immutable inputs, selected generations and completed producer state
before a success receipt can be published.

Five synthetic evidence controls and four existing published-install controls
pass. The executable distribution-area control also passed through its runner;
these controls do not qualify a native installation. The explicit manual CI path
calls a read-only reusable workflow for all four native host targets, builds the
actual committed optimized candidate, downloads the registered published bytes,
runs and independently checks the transition, and preserves exact bundles and
raw success/failure evidence. Workflow regressions reject missing targets,
substituted source, bypassed qualification and write permissions. Toolchain,
profile, size and diff checks pass. Scoped review, actual native execution,
all-platform results, required local gates and release promotion remain pending.

The independent historical compiler endpoint at measurement overlay
1e81537db6046dbf7381f193152bcf4e7c89ec02 is now prepared, including its actual
frontend helper. The successful continuation is retained in
/workspace/validation-work/evidence/independent-baseline-1e81537db-v3/observation.json.
The earlier disk-floor failure remains failed. Its completed owned graph was
retired only after verifying both protected endpoint binaries, with raw cleanup
and retained copies under independent-baseline-1e81537db-v2/retirement-after-completed-v3.
No compiler performance comparison has yet been captured or accepted.

The initial published-transition review is NOT SATISFIED at 9346d06ae:
/workspace/validation-work/evidence/sifr-claude.l0CH8k/response.md.
The reusable workflow used the unavailable runner context in job-level env. One
remediation batch declares host_kind on the two Darwin matrix rows and uses the
allowed matrix context. Regression controls reject a missing Darwin declaration
and the original forbidden context. Actual native/platform qualification and
broad gates remain pending; the first candidate is not approved.

### M5 native qualification version authority — 2026-10-04

Actual optimized preparation at ee9426f0f exposed that crates/sifr/Cargo.toml
contains the source-package placeholder 0.0.0. The preparer had incorrectly used
that value as the proposed upgrade version. Published-predecessor policy correctly
refuses it because it predates Beta 16; no upgrade is qualified. The original
attempt and any resulting preparation-only bytes remain preserved outside Git.

This bounded correction reads the canonical package rehearsal's literal
RELEASE_VERSION, currently 0.1.0-beta.1300, without executing source expressions.
It verifies a declared prerelease newer than the registered predecessor before
building, embeds that exact declared version through the canonical release
builder, and records qualification-only version role, separate source package
version and the authority-file hash. The independent checker validates all three.
This is a prospective qualification fixture, never a claim that Beta 1300 was
published or that source Cargo version is a released product version.

Seven native candidate controls now include the real 0.0.0 source placeholder,
separate canonical version selection, authority drift and rejection of placeholder,
old predecessor or computed/unknown declarations. Three capacity and five
transition controls plus workflow/diff checks pass. Actual corrected preparation,
transition qualification, scoped review, native matrix and broad gates remain open.
### M3 conservative main correctness-job reuse — 2026-10-04

This bounded item suppresses six duplicated main-push correctness jobs only when
an independently verified, complete and fresh pre-merge merge-profile producer
executed the exact resulting commit. It covers smoke fuzz/property, SQL WASI
build and four compiler-component native targets. The merge profile, guardrails
and performance remain fresh. This does not claim whole-profile deduplication or
cross-commit equivalence; those remain open when their dependency closure cannot
be proved. Unknown, inaccessible, stale, partial or recursive producers execute
all jobs fresh.

The producer stores its decision in a digest-verified current-attempt artifact.
The trusted publisher reads that artifact, then independently recomputes source
workflow, exact executed candidate artifact, merged PR/merge-group provenance,
complete job inventory, run/attempt identities and 24-hour completion. Skipped
current jobs count only through that verified source producer; a skipped merge
profile can never qualify. The selector has read-only Actions access and no
check/publication credentials. Scheduled hardening and release boundaries are
unchanged.

Five synthetic provenance tests cover thirteen invalid source mutations, merged
PR/full-profile requirements, current skipped-job accounting and nine artifact
substitutions/source-drift controls. Seven publisher controls and three existing
aggregate controls pass. The skipped-job control exposed a missing-timestamp
attribute error; unknown timestamps now reject explicitly. Workflow, toolchain,
profile, size4419 and diff checks pass. These are controls, not an actual hosted
reuse observation. Scoped review, required local gates, actual trusted publication
and whole-profile main-push deduplication remain pending.

The initial main-reuse review is NOT SATISFIED at d29f983e5:
/workspace/validation-work/evidence/sifr-claude.yr3r4H/response.md. Skipping a matrix
at job level can suppress expansion, leaving four required target jobs absent.
The remediation keeps all four matrix rows, runs a current reuse marker, and
skips heavy steps only when reuse is selected. Two nonmatrix jobs remain skipped.
The aggregate now independently requires each platform's actual mandatory step
outcomes for fresh/source evidence, or a positive marker plus verified original
producer evidence for reused component jobs. A selector cannot make skipped
component assertions appear fresh merely by leaving the matrix job successful.
Controls use this actual expanded-job shape and reject a skipped source assertion.
All affected controls pass; actual hosted reuse and acceptance gates remain open.

### M5 canonical metadata container version repair — 2026-10-04

The actual optimized compiler at ee9426f0f compiled successfully in 12m06s.
The subsequent canonical package preparation failed because its distribution
validator and synthetic packaging fixture expected metadata format 4, while the
actual canonical compiler emits format 5. The full failed native preparation is
retained under /workspace/validation-work/evidence/native-candidate-ee9426f0f;
its partial package and generated metadata are preserved. This is a packaging
failure, not a disk or compiler-build failure and not native qualification.

This bounded repair accepts only canonical format 5, preserving the existing
identity, digest, directory and size bounds. A control compares the distribution
version with the compiler's declared authority; old/future format controls
recompute their digests and still reject. The synthetic fixture emits current
format 5 explicitly and remains excluded from production Cargo packaging.
Seven metadata controls pass. The corrected validator also accepted the actual
6,554,972-byte metadata container produced by the real optimized compiler;
its diagnostic binds actual compiler, metadata and validator hashes at
/workspace/validation-work/evidence/native-metadata-format5-diagnostic.json.
That diagnostic does not qualify the failed 0.0.0 package or a published upgrade.
Scoped review, canonical corrected native preparation, transition and full
native/local acceptance remain pending.

The initial format repair review is NOT SATISFIED at b4afce888:
/workspace/validation-work/evidence/sifr-claude.u6P6ZT/response.md. A second
synthetic producer in governance/qualification_fixture_support.py still emitted
format 4. The remediation uses the validator's canonical CONTAINER_VERSION in
that fixture. All four target fixtures now validate format 5 and the seven
metadata controls pass. The affected full stable-prepare self-test reaches an
existing incomplete archive fixture and fails missing
crates/sifr_sql_runtime/Cargo.toml. The same failure is reproduced with the
unchanged format-4 validator and governance fixture at base 1edcfa46d; both raw
logs are preserved (metadata-format-governance-diagnostic.log and
metadata-format-governance-baseline.log). It is not a format regression and not
a passing governance gate. The separate distribution qualification fixture
inventory follow-up must align with the current required SQL runtime payloads;
no SQL compiler/runtime implementation or externally owned PR is changed here.

### M2/M5 explicit owned native preparation continuation — 2026-10-04

The failed native packaging attempt leaves a successfully compiled optimized
compiler in its private graph. This bounded continuation requires an explicit
operator-selected original preparation, its private UID/device/inode/UUID lease,
recorded failed state, exact locked input and tool/target identity, ancestor source
and original producer bytes, complete original raw commands, and a unique actual
optimized Cargo compiler record. Linked, active, unowned or unknown graphs reject.
It never reuses an assertion or success receipt: actual Cargo, packaging,
metadata production and installer generation all execute for the current clean
source and corrected qualification version.

Before cache mutation it retains the original compiler in a bounded, byte-verified
compressed file and fsyncs the containing directory. The new attempt uses a
separate output/receipt and binds the original state, events, graph ownership and
retained bytes; its independent checker revalidates custody and the live compiler
against the exact packaged compiler. The original failure state is unchanged.
The continued graph is retained even after success because its original failed
consumer remains recorded; no failed graph retirement is authorized. Known warm
continuation admits one GiB of additional growth; memory peak, temporary growth,
reserves and monitoring floor remain unchanged. Cold preparation retains its
original allocation and retirement behavior.

Five synthetic ownership/retention controls, seven candidate, three capacity and
five transition controls pass. The executable artifact case passes through the
actual distribution runner (native-continuation-case.json). Read-only inspection
and lease acquisition of the actual original graph also passed and changed no
compiler bytes or failure state; native-continuation-owned-cache-inspection.json
records that zero-assertion observation. Initial synthetic busy-lock expectation
used the wrong exception type and failed; the corrected control accepts the
framework's actual BlockingIOError. Actual corrected native continuation,
qualification, scoped review and required local/native gates remain pending.
### M5 governance qualification fixture inventory — 2026-10-04

The existing stable-prepare self-test's synthetic archive omitted the two SQL
runtime Cargo manifests already required by verify_release_archive. This bounded
fixture-only repair adds those manifests to the explicitly synthetic target
payload. It changes no SQL compiler/runtime, public support claim or native
qualification assertion. Digests and manifests continue to derive from actual
fixture contents; archive requirements remain unchanged.

All eight existing stable publication prepare self-tests now pass, including
normal/incident preparation, activation recovery, input-drift refusal, summary,
CLI producer and safe artifact extraction. Raw output is retained in
/workspace/validation-work/evidence/qualification-fixture-sql-inventory.log.
Source size and diff checks pass. The earlier missing-file failures remain
failed; this is synthetic governance coverage, not real package/platform proof.
Scoped review and required local gates remain pending.
### M2 bounded native transition allocation — 2026-10-04

The fixed eight-case published transition executes one Cargo worker and one small
persisted math program. Its prospective combined process estimate is now three
GiB rather than the generic six-GiB compilation estimate. The actual broader
source preparation observed a 1.8-GiB maximum child RSS and structural doctor
execution 247 MiB; those are context for the estimate, not a measurement of this
transition or simultaneous RSS. Actual transition memory remains to be measured.
Disk growth (three GiB), temporary growth (512 MiB), both two-GiB reserves, runtime
commands, assertions, fixed user program and every monitoring floor are unchanged.
RAM-backed storage still charges its full additional growth to shared memory.

Six transition controls pass. The added admission control admits the fixed RAM
rehearsal at 8.5 GiB total shared requirement and rejects eight GiB available,
while preserving the five-GiB disk requirement. Its initial negative test tried
to mutate the immutable Resources value and failed; the corrected test constructs
a separate value. This prospective stage policy does not qualify a native run or
change broad/native-compiler preparation allocation. Scoped review, actual
transition, empirical peak and required local/native gates remain pending.

## M3 complete exact-commit main profile reuse — 2026-10-04

The earlier six-job reuse implementation still reran the merge profile on main.
This bounded extension includes `local-first-merge` in the independently verified
pre-merge producer inventory. The source must have executed the actual profile
step successfully, completed the full mandatory job inventory within 24 hours,
and validated the identical commit under the protected workflow bytes. A prior
create-PR profile, missing/skipped profile step, changed candidate, stale run,
recursive push producer, or substituted decision never authorizes reuse.

The current matrix still expands and records its explicit reuse marker. Every
heavy profile step runs when provenance is unavailable or invalid. The trusted
publisher rechecks the source independently before accepting the current marker;
selector output alone cannot qualify a skipped runner. This consumes a complete
prior merge outcome, including its performance gate, rather than combining
partial captures or deriving a new performance observation from an old sample.
Cross-commit equivalence remains conservative: an unknown equivalence executes
fresh validation.

Five provenance suites, three aggregate suites, seven trusted publication suites,
the workflow regression guard, and the source-size guard pass. Full acceptance
gates, scoped Opus review, actual hosted reuse, and protected main delivery remain
open; this record is implementation progress, not phase completion.

## Native execution observations — 2026-10-04

At frozen candidate `2a33012413016e22d8477d555a08eddf4819d35b`, the actual optimized
native package and installer preparation passed. The qualification-only version
is `0.1.0-beta.1300`; the source package placeholder remains `0.0.0`. The verified
archive SHA-256 is `4509783f3b4ef21da61260c7f9d201a9e37c1cea250c2cee83abb744895ee17a`.
Raw commands, the actual Cargo optimized artifact, custody, and the independent
prepared receipt are outside Git in `native-candidate-2a3301241`. Preparation
assertions remain zero. The original failed preparation and graph are retained.

The first actual published transition stopped at an uncached predecessor Rust
dependency. Thirty-one distinct actual published bridge-probe manifests then
fetched their locked dependencies without changing their locks. A new invocation
passed the published program, post-switch migration failure rollback, the restored
published program, upgrade, new version, and installed integrity. It stopped on
the expected one-retained-predecessor inventory: rollback left an additional
legacy directory. Both failed invocations remain failed with raw process output;
remaining persisted-state/reinstall/rollback assertions are not claimed.

Four native targets were dispatched on the frozen candidate in GitHub Actions
run `37178223306`; the observed jobs are queued. This is dispatch evidence only.
Completed review PRs #4315–#4317 were merged into the integration branch, not
protected main. Their clean owned checkouts were retired after checking their
PR head identities and ancestry; source commits and review evidence are retained.
## M5 actual transition prerequisite and retained inventory repair — 2026-10-04

The frozen native rehearsal exposed two concrete qualification omissions. The
published compiler needs dependency cache preparation before its offline program
runs. Its successful post-switch rollback intentionally retains a receipt-only
legacy directory, so a later successful migration has one intact predecessor
plus that residue, rather than one total legacy directory.

The bounded repair prepares a dependent public-stdlib Cargo project from the
actual published manifests and seeded lock, fetches online, then checks locked
and offline. The published package inputs stay byte-identical. Preparation has
zero runtime assertions, bounded process output/deadline, separate cache-disk
admission and the unchanged two-GiB reserves. Its exact generated manifest,
prepared lock, commands and raw output are independently checked against the
retained published input bytes. Public stdlib features select runtime leaves;
compiler-private runtime structural support is absent from the published package
and is not introduced into the user's program. Native CI preserves this nested
preparation evidence on success or failure.

Migration qualification now requires exactly the known receipt-only rollback
residue and one independently verified intact predecessor. It rejects extra
legacy directories, unexpected residue bytes, symlinks, receipt substitution or
any predecessor payload drift. The actual installer is unchanged; rollback,
upgrade, user-program, reinstall and integrity commands remain mandatory.

Fifteen focused controls pass (six runtime ledger, five predecessor inventory,
four dependency-evidence suites). Actual dependency preparation and its checker
also passed against the unchanged published Beta 16 installation; this is a
preparation observation with zero runtime assertions. The two earlier native
transition failures remain failed. Scoped review and the corrected full native
rehearsal remain pending.

To recover shared memory, the closed test installation payloads from both failed
rehearsals and the failed packaging staging copy were archived losslessly outside
Git. Every regular file hash/size/mode, directory and symlink target was compared
against the decoded archive before removing its staging copy. Failure states,
raw commands, user projects, Cargo graphs and original published assets stay in
place. Custody manifests under `/tmp/sifr-closed-native-payloads-20261004` record
exact restoration paths. No historical failed result was relabelled as passing.

## M5 native uv platform checksum correction — 2026-10-04

Actual hosted jobs in run `37178223306` failed before native preparation because
the reusable four-platform workflow supplied the Linux x64 uv checksum to both
Darwin architectures and Linux ARM. Their GitHub check annotations preserve the
checksum mismatch and skipped native commands; none is a qualification pass.
The invariant guard separately rejected the unqualified matrix runner expression.

This bounded repair records each platform's uv 0.12.10 archive digest from the
upstream GitHub release asset registry and selects it through a finite explicit
include matrix. The toolchain guard accepts only that literal runner/checksum
binding, keeps the canonical version-file authority, and rejects unknown runners,
wrong hashes, arbitrary checksum expressions, empty/duplicate/malformed rows and
implicit matrix axes. `macos-15-intel` is explicitly mapped to Darwin x64. Existing
Linux x64 workflow digests and every native runtime assertion remain unchanged.

The uv guard self-test passes 54 controls, including the new finite-matrix
negative controls. Scoped review and an actual corrected hosted rerun remain
pending. This is a prerequisite repair, not native qualification or final closure.

## M5 persisted math fixture correction — 2026-10-04

The corrected frozen `615fb6bd8cf7ac86ef0b85e5c4f6e09b547c0293` native preparation
passed. The next actual transition passed published installation/program,
post-switch migration rollback, restored published program, upgrade, candidate
version and package integrity. Every persisted user file retained its original
hash. The candidate program printed `Ok(9)`, however, rather than `9`: the fixture
used `int(float)`, whose current exact-integer conversion returns a result. The
math `sqrt` function itself still declares and returns a float. The invocation
remains failed, with its raw output and unchanged user hashes retained.

Use the shared integer-returning `sifr.math.isqrt` operation instead. An owned
standalone project initialized by the actual Beta 16 binary ran `isqrt(81)` with
both the actual published compiler and the prepared candidate. Both native
processes completed offline with stdout exactly `9`; commands and raw output are
at `/tmp/sifr-shared-persisted-math-20261004`. This is direct fixture evidence,
not a complete upgrade qualification. No compiler implementation, public API,
expected output, runtime command, case count or rollback requirement changes.
The existing fifteen transition controls still pass. Scoped review and a fresh
complete transition remain pending.

The corrected hosted macOS ARM job now passed Rust/uv/Python setup and reached
native preparation. It stopped at the Darwin storage-authority guard. The raw
job log and downloaded failed state from run `37180131233` are retained outside
Git. This is a new observed provider classification failure, not an assumed host
capacity or native execution pass; collect the actual disk authority next.

## M5 hosted native capacity diagnostics — 2026-10-04

The observed macOS ARM failure did not preserve the underlying disk authority
inputs. Collect bounded raw `df`, `diskutil info` and APFS inventory, VM page
counts and CPU/memory totals before native preparation, and preserve those files
in the existing always-uploaded evidence. Each command records its exit cause,
truncation and output digests. These observations grant no resource admission
and do not change the storage guard or any runtime requirement. Allow direct
dispatch of the existing four-native-target workflow so this rehearsal can run
without also dispatching unrelated parent jobs. Scoped review and actual hosted
observations remain pending.

## M5 actual Linux native qualification and device probe repair — 2026-10-04

The exact `f02e5b3b80ca9f44014b0cd2b2a9dd395da24c41` Linux bundle preparation
and fresh published transition passed. The independent receipt checker confirmed
all eight runtime cases, sixteen commands, unchanged persisted user files, native
reinstall integrity and both transaction rollback paths. Wall time was 4:41.64,
maximum child RSS 550328 KiB, with no swaps. Raw commands, receipt and independent
check remain outside Git. Closed installed payloads and generated native cache
were archived losslessly; only the successfully qualified intermediate Cargo
cache was cleaned under its exclusive matching owner lease. Failed graphs and
all historical failed states remain unchanged. Restoration custody records
preserve exact historical live paths. This is Linux evidence on that frozen
candidate, not four-platform or protected-main delivery.

Scoped fixture review was satisfied in #4321; diagnostics review was satisfied
in #4322. Both are merged only into the integration branch. Direct four-target
run `37181469650` reached x64 Darwin admission, which observed 9533071360 bytes
available against the unchanged 9663676416-byte requirement. Its diagnostic
`diskutil info` invocation rejected the directory path. Resolve the actual
`/dev/` device from the preceding successful bounded `df` result before querying
it. Missing or incomplete `df` output leaves an explicit unavailable-device
observation. No capacity guard, reserve, runtime requirement or numerical budget
changes. Other native hosts and final acceptance remain pending.

## M5 cold native source dependency preparation — 2026-10-04

A fresh x64 Darwin job on `7eb6d331ce2c2871ff332a6b3887fd80c093e3f3`
passed storage and memory admission, then the actual offline package command
failed because its fresh Cargo cache lacked `sha2`. Preserve that failed state
and raw command; successful admission is not a compiler or native pass.

Add explicit locked online source dependency acquisition followed by a locked
offline availability check before the existing offline native compilation. The
preparation binds source and producer commits, canonical tool bytes, source
manifests/lock/config/toolchain, exact commands, bounded raw output and completion.
It has zero runtime assertions and independently checks its completed receipt.
Its Cargo-cache filesystem is admitted separately for one GiB prospective growth,
512 MiB resident peak and the existing two-GiB reserves and three-GiB floor.
Changed inputs, process failures, truncation or incomplete/resealed receipts
remain nonqualifying. CI preserves preparation failures in the existing upload.
The offline build and all eight native qualification cases stay unchanged.
Five focused control suites pass; scoped review and actual cold hosted execution
remain pending.

The locked source preparation and its independent checker passed locally against
the unchanged full source at `7eb6d331ce2c2871ff332a6b3887fd80c093e3f3`, using
the separately committed `faa7620f25454d02cc7cec0ce10d0ca9beea8fba` producer.
This existing warm cache observation has zero assertions and is not a cold
hosted build. Initial scoped review was satisfied. Enroll the five control suites
in the existing executable native-preparation contract; this changes no helper
mechanism or native qualification assertion. Enrollment review and cold hosted
execution remain pending.
