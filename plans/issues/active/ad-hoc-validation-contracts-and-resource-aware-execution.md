# Ad Hoc Validation Contracts and Resource-Aware Execution

status: active
registered: 2026-10-03
current_stage: M0 delivered; M1–M5 implementation in progress

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
