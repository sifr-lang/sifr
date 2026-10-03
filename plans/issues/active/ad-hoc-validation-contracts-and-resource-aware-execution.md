# Ad Hoc Validation Contracts and Resource-Aware Execution

status: active
registered: 2026-10-03
current_stage: M0 delivered; M1 and M2 implementation in progress

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

Current state: M0 delivered through #4279; M1 and M2 implementation in progress;
M3–M6 remain pending.
The [entry inventory](../../../internal_docs/validation_execution_inventory_20261003.md)
records current selections, resources, verified protection, owners and gaps.
Next action: use the measured existing-cache preparation cost to refine prospective
admission, finish independent correctness/performance routing, and continue M3–M6.
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
performance area after selected correctness guardrails, areas and toolchain
checks for ordinary profiles. Missing performance admission still blocks the
performance area and final gate, with a separate performance outcome. Blocking
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
