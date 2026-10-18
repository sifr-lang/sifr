# Ad Hoc Phase: E2E Iteration Throughput and Cache Correctness

status: planned; implementation not started
registered: 2026-10-18
current_stage: measurement complete; bounded implementation entry pending

## Objective and authority

Make development E2E feedback fast and trustworthy by eliminating repeated native
dependency compilation, unsafe binary reuse, avoidable preparation rebuilds, and
repeated compiler setup. Preserve the selected fixtures, assertions, diagnostic
contracts, emitted-code safety checks, and authoritative failure semantics.

The user requested this phase plan after detailed measurements. This record
authorizes no claim that implementation, review, or qualification has occurred.
The user specifically requires a practical route around slow/broken compiler
preparation: do not repeatedly launch a build that takes forever before making
progress on the harness. The bootstrap strategy below is part of the plan.

Use the [phase closure loop](../../../.agents/skills/phase-closure-loop/SKILL.md),
[agent instructions](../../../AGENTS.md), and
[verification contracts](../../../verification/README.md). Work on one bounded
item at a time. This plan is a concrete E2E implementation workstream related to
the [validation/resource execution plan](ad-hoc-validation-contracts-and-resource-aware-execution.md),
especially its M2 and M3; it does not replace its policy or enforcement work.
Reconcile ownership before implementation. Do not change roadmap sequencing or
claim the completed [DX phase](../archive/ad-hoc-compiler-dx-and-toolchain-reuse.md)
grants this phase a validation exception.

## Scope and non-goals

In scope:

- Native dependency reuse and source freshness in `test_e2e_pass`.
- Owned compiler/native build targets, safe artifact paths and retirement.
- Stable generated inputs, scratch-directory execution, bounded scheduling.
- Complete fixture accounting and explicit failed-batch diagnostic isolation.
- Invocation-scoped compiler/provider/native identity reuse after qualification.
- Concise agent iteration guidance, separate compiler/native build budgets,
  dependency-aware preparation failures and actionable stage reporting.
- Reproducible before/after measurements and required acceptance evidence.

Not in scope:

- Removing fixtures, weakening assertions, shortening intentional timeouts,
  changing optimization/assertion settings to obtain a faster number, or moving
  required merge coverage to an optional lane.
- Replacing the 12–13 hour cloud paired performance protocol. These measurements
  cover development E2E; cloud/release qualification retains its own authority.
- A general compiler rewrite, broad crate splitting, a new distributed cache,
  a custom scheduler framework, or alternate compiler semantic implementations.
- Silently absorbing the 19 observed compiler defects. They are separately owned
  correctness dependencies; classify them before requesting bounded repairs.
- Release publication, submodule updates, unrelated cleanup, or dependency changes.

## Measured baseline and provenance

Measurements were captured on 2026-10-14, not on a newly qualified current main.
Revalidate identities and recount the corpus at implementation entry.

| Input | Recorded identity |
|---|---|
| Primary repository | `30d2b39021174684ca9a2e695447d73ecbf25308` |
| Isolated measurement candidate | `36c87c5b2a0f178f4f590f4f975bb6197c6d43d3` |
| Measurement tracked-input digest | `faa457978e65377c996864c03417bddd40891e3bf5039b9417a9c7486d408451` |
| E2E executable SHA-256 | `1156a99bce6631bb9a5d6ac7b63ffb95dbcc4d96222dbacf8a35780f311f4113` |
| Tools | Rust 1.98.1; CPython 3.14.7; uv 0.12.10; verify repository pins at entry |
| Profiles | E2E/compiler artifact opt-level 1, debuginfo 2; generated native application opt-level 0, debuginfo 1, assertions and overflow checks enabled |
| Baseline allocation | 4 Sifr workers, 3 native workers, 1 Cargo job per native group, 3 execution workers, maximum 12 fixtures per group |

The measurement candidate has five existing distribution-prerequisite changes,
not compiler/E2E changes: `scripts/distribution/metadata_artifact.py`,
`scripts/distribution/metadata_artifact_checks.py`,
`verification/areas/distribution_release/cases/mock_metadata_binary.py`,
`verification/areas/distribution_release/governance/qualification_fixture.py`,
and `verification/areas/distribution_release/governance/qualification_fixture_support.py`.
Do not silently transfer these changes into implementation.

Evidence bundle layout (locate or restore it in the current environment; no
specific machine or absolute path is required):

```text
Campaign bundle:
iteration-measurement-20261010-yr3_l_4h/
Detailed evidence beneath the bundle:
e2e-deep-20261014/
Earlier inventory/provenance:
e2e-categories-20261013/
```

Start with `e2e-deep-20261014/report.html`, `analysis.json`, `validation.json`,
`closure.json`, and `code-evidence.json`. All 27 report checks passed; that
validates measurement accounting, not product correctness. Raw logs, generated
inputs and Cargo timelines remain linked from the HTML report. The earlier
`inventory.json` and `source-provenance.json` record fixture bytes and candidate
differences. If these locations are unavailable, recover evidence with its
identity or remeasure; this document alone is not executable qualification.

| Experiment | Before | After | Interpretation |
|---|---:|---:|---|
| 15-category first sweep | 33m33.45s | 8m29.07s | Separate targets versus one shared native target; same generated inputs/outcomes; about 75% reduction |
| 15-category unchanged reruns | 5m57.41s | 2m31.31s | About 58% reduction; category sums, not canonical full-gate wall times |
| Canonical, empty native target | 7m39.42s | 6m24.31s | Shared target in both; change 3 outer workers × 1 Cargo job to 1 × 3 |
| Canonical, dependencies warm, every batch Cargo-validated | — | 2m35.58s | Outer binary reuse disabled; all 729 selected; failed attempt |
| Canonical, dependencies warm, existing outer-cache reuse | — | 1m50.09s | Unsafe shortcut observed separately; not the proposed acceptance target |
| Preparation after source-directory fixture output versus scratch output | 25.60s | 4.16s | Identical fixture binary; identical final E2E binary; controlled invalidation test |
| Original compiler API, serial fresh versus shared context | 303.80s | 238.55s | 729/729 complete output identities match; not parallel gate wall time |

Do not add these savings together. Cold means empty native dependency targets,
with toolchain/vendor/OS caches retained; it is not a cold-host performance claim.
Observations are not statistical guarantees. Initial post-suite preparation also
took 84.87s, and a later rebuild took 49.98s; incremental state changes the cost.

Coverage and accounting:

- All 729 positive fixtures were attempted. Category grouping used 228 batches:
  645 passed, 19 directly failed native compilation, 65 were collateral-blocked.
- Canonical grouping used 185 batches: 631 passed, the same 19 directly failed,
  and 79 were collateral-blocked. Different grouping explains the counts.
- Independent runs recovered all 65 category-blocked healthy fixtures; the
  additional 14 canonical collateral fixtures had already passed in categories.
  All 710 runnable fixtures passed three native executions each (2,130 samples).
- Sum of the 710 per-fixture median runtimes: 7.96s. About 5s belongs to the
  intentional `async_with_secondary_cleanup_timeout` test; retain that check.
- Directly failing fixtures have no runtime measurement. Full-corpus timings
  above are failed attempts, not measurements of a fully passing gate.
- Baseline category sums: Sifr compilation/plans 138.796s; grouping 0.047s;
  native builds 1,871.489s; reached runtime 1.111s; residual 2.007s.
- Native builds dominate about 93% of category wall time. The baseline contained
  8,432 Cargo-managed Rust compilations: `syn` 215, `encoding_rs` 213,
  `num_bigint` 214, and `sifr_runtime` 213. Some variants legitimately differ.
- In overlapping native group-work, third-party Rust compilation accounts for
  about 72%, first-party runtime/stdlib about 5%, generated Rust about 2%, linking
  about 2%, build-script execution about 7%, and Cargo/probes/residual about 12%.
  Do not add overlapping work to wall time or double-count nested rustc probes.
- A separate serial staged probe measured native-toolchain resolution 138.47s,
  metadata admission 48.70s, parsing 0.072s, semantic/HIR work 7.65s, demanded
  stdlib materialization 33.91s, and Rust emission 35.38s. Its traversal omits
  some original API work; it is diagnostic attribution, not an exact partition
  of the original 303.80s or a removable-savings estimate.

## Root causes and code entry points

| Mechanism | Source to inspect | Consequence |
|---|---|---|
| Per-group target; inherited target removed; generated files always rewritten | [batch execution](../../../crates/sifr/tests/e2e_support/batch_execution.rs) | Dependencies repeatedly rebuilt; unchanged inputs can dirty Cargo |
| Binary-cache short circuit, two-hour expiry, group pruning | [harness model](../../../crates/sifr/tests/e2e_support/harness_model.rs), [fixture compilation](../../../crates/sifr/tests/e2e_support/fixture_compilation.rs) | Native source freshness can be bypassed; failed groups lose useful artifacts |
| One context per fixture; lowering statistics omitted by pass helper | [fixture compilation](../../../crates/sifr/tests/e2e_support/fixture_compilation.rs) | Repeated setup and late reporting of already observed errors |
| Per-compile native resolution | [entrypoint](../../../crates/sifr_driver/src/build/entrypoint.rs), [resolution](../../../crates/sifr_driver/src/build/entrypoint_resolution.rs), [native context](../../../crates/sifr_sysroot/src/native_context.rs) | Repeated executable selection, version subprocesses and identity hashing |
| Relative output inside watched package | [hashlib fixture](../../../crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr), [identity build tracking](../../../crates/sifr_identity/src/build.rs) | Ignored temporary output triggers compiler/E2E preparation rebuilds |
| One native binary per batch | [batch execution](../../../crates/sifr/tests/e2e_support/batch_execution.rs) | One compilation error prevents healthy siblings from running |

Confirmed cache negative control: cache a passing arithmetic fixture using a
disposable source sysroot; insert an unconditional `compile_error!` into that
copy's runtime; the outer-cache run still passed with a cache hit. Forced Cargo
validation failed correctly. Evidence: `invalidation/results.jsonl` and
`invalidation/method.json`. The original runtime was unchanged. This finding
concerns the tested direct E2E/development-source path, not every release cache.

Crate size is not the demonstrated primary cause. Existing compiler boundaries
already separate frontend, IR, lowering, type system, codegen and driver; runtime
and stdlib have optional capabilities. Splitting a crate while each batch still
rebuilds dependencies does not remove the repeated work. Revisit decomposition
only with edit-specific critical-path evidence after this phase's primary fixes.

### Additional instruction and runner findings

The 2026-10-18 source review identified these implementation items. Their runtime
benefits are unmeasured; do not add estimated savings to the E2E results above.

| Finding | Current authority | Planned item |
|---|---|---|
| Agent guidance already requires focused checks and docs-only validation, but the repeated-build diagnosis procedure is only in this phase | [AGENTS.md](../../../AGENTS.md), [verification guide](../../../verification/README.md) | R0: short iteration guidance linked to the canonical procedure |
| General `CARGO_BUILD_JOBS` is assigned from E2E per-group `cargo_build_jobs`, which is 1 in the reviewed merge profile | [profile runner](../../../verification/runner/sifr_verify/profile_runner.py), [merge profile](../../../verification/profiles/merge.json) | R1: separate compiler-preparation and native-batch budgets |
| A single preparation result blocks later checks even when some have no dependency on the failed preparation | [Cargo setup](../../../verification/runner/sifr_verify/cargo_setup.py), [profile runner](../../../verification/runner/sifr_verify/profile_runner.py) | R2: explicit preparation dependencies and independent failure handling |
| Developer-facing commands need a clear preparation/execution breakdown and actionable progress | [E2E entrypoint](../../../verification/runner/e2e/run_e2e_pass.sh), [step budgets](../../../verification/runner/sifr_verify/step_budgets.py) | R3: stage reporting and bounded diagnostics |

The [public facade](../../../scripts/run_all_tests.sh) is thin and defaults to
`merge`. Clarify explicit profile usage; do not silently change that default or
weaken its selection. Preparation with `cargo test --no-run` followed by execution
is not inherently duplicate compilation: matching inputs should reuse artifacts.
Keep this separation and verify reuse, including matching features, configuration
and build-script environment. The closure workflow already avoids redundant
review/gates for unchanged or record-only work; preserve those rules.

## Build-blocker strategy: progress without waiting on a full compiler rebuild

1. **Inventory before building.** Record base/candidate, dirty paths, submodules,
   tools, fixture inventory, available artifacts and ownership. Read Cargo traces
   to distinguish dependency download, real compilation, linking, lock waits,
   capacity pressure and false invalidation. Do not equate silence with a hang.
2. **Use existing evidence first.** Locate the retained executable through the
   campaign's artifact records; verify its hash above and compatibility with the
   current execution environment before using it. It can reproduce baseline
   scheduling, selection and cache behavior
   without rebuilding the compiler. Inspect `shared/run_v3.py`,
   `invalidation_run_v2.py`, `prep_loop.py`, `cold_canonical.py`, and
   `cold_balanced.py` in detailed evidence for successful disposable experiments.
   Do not rerun the entire historical campaign automatically.
3. **Separate two build domains.** Own a stable incremental compiler/test target
   for preparing the E2E executable, and a separate stable native dependency
   target for generated fixture programs. Neither may be another session's
   mutable target. Do not hardlink mutable artifacts or clean the shared host.
4. **Prove the mechanism cheaply.** Reproduce stale reuse with one small fixture
   and a copied sysroot. Test stable-input writes, Cargo-reported artifact paths,
   native target reuse, and scratch execution in disposable orchestration around
   the retained binary. Prior wrappers establish feasibility; they are not the
   production fix or validation of changed Rust code.
5. **Compile the smallest implementation candidate once.** Begin with the harness
   target and focused harness tests using the owned warm compiler target. Avoid
   touching compiler-wide identity machinery merely to avoid rebuilds. Retain
   incremental artifacts between attempts. Before any costly rebuild, check
   disk/RAM/CPU limits, existing targets, planned allocation and reserve.
6. **If preparation exceeds 30 minutes, cancel its owned process group, preserve
   evidence, and diagnose the specific stalled/expensive unit.** Do not repeat
   unchanged cold builds or run a broad gate to diagnose preparation. Inspect
   Cargo fingerprints/timelines and try one evidence-backed correction (for
   example, an owned warm target or removing test output from watched inputs).
   Read-only analysis and independent disposable experiments may continue.
7. **Keep the qualification boundary explicit.** A retained binary cannot validate
   new source. A wrapper cannot validate production Rust changes. If current
   compiler preparation remains blocked by an external defect or capacity,
   record its owner, exact command, failure/cap, and next action; stop dependent
   acceptance work. Never substitute old green evidence or reduced fixture
   selection. No milestone is accepted until its actual candidate is built and
   its required checks execute.

The 30-minute cap applies to each diagnostic/measurement attempt. Cancellation
means incomplete, never passed. Required gates retain their obligations; a gate
that cannot finish under the cap is an explicit blocker, not a waiver. Finish
independent measurement categories where possible, without repeatedly retrying
the same blocker. Determine a safe disk reserve from the current environment's
capacity, planned allocation and retained consumers; do not delete
historical logs, active leases, retained consumers or another owner's artifacts.

## Implementation milestones

All items are pending. Each item must record owned paths, baseline and candidate
SHAs, named checks, evidence locations, review, PR and unresolved dependencies.
Split a milestone into bounded implementation items when needed; do not implement
all milestones in one unreviewable change.

Default order: E0, R0, R1, E1, E2, R2, E3, E4, R3, E5. R1 is early because it
may unblock implementation preparation. Coordinate R1–R3 with the existing
validation/resource plan's M2/M3 owner and record one implementation authority;
do not create competing schedulers or duplicate policies. Adjust order only for
recorded dependencies, keeping one bounded item active. Existing reported stages
can support measurements before R3; a reporting redesign is not a prerequisite
for the primary cache fix.

### E0 — entry inventory and minimal reproductions

- Reconcile this baseline with current main and the validation plan's owner.
- Recount fixture selection and reconcile all 19 recorded failures below; do not
  assume historical failures remain current or that a changed count is a pass.
- Preserve the two minimal controls: stale runtime-source cache reuse and
  source-directory output causing an unnecessary preparation rebuild.
- Record available compiler/native targets and a safe capacity plan. If current
  preparation is blocked, use the build-blocker strategy rather than a cold gate.
- Acceptance: reproducible commands, current ownership, source/tool identity,
  selected inventory, blockers, and one concrete E1 implementation slice recorded.

### R0 — concise instructions for efficient iterations

- Add a short rule to `AGENTS.md`: start with the smallest affected check, retain
  owned build targets, and diagnose a slow/failed preparation attempt before
  repeating it unchanged. Link to the detailed verification procedure rather
  than copying this phase's measurements and operational instructions into the
  agent's default context.
- Document focused development, PR-candidate and merge-candidate commands in
  `verification/README.md`. Explain that the public facade defaults to `merge`
  and is not the normal command after each local edit. Keep defaults unchanged.
- Preserve full merge qualification, review requirements, exact-input evidence
  reuse, resource/ownership protection, and docs-only validation. Do not import
  this phase's 30-minute diagnostic cap as a universal gate-pass exemption.
- Acceptance: guidance remains concise; every change type has a clear next
  command; no existing required check is removed. Docs checks only for R0.

### R1 — separate compiler preparation from per-batch concurrency

- Replace the accidental coupling in `ProfileRunner.__init__`: the E2E Cargo
  jobs-per-group setting must not automatically determine the global compiler
  preparation budget. Define explicit preparation and native-build allocations
  using the existing profile/resource mechanisms, not a second scheduler.
- Respect effective CPU/memory limits, reserved capacity and the product of
  active outer workers and inner Cargo jobs. Increasing a number blindly is not
  the fix. Declare and report precedence between profile settings and permitted
  environment overrides; do not silently ignore either.
- Keep matching compiler preparation/execution configurations and feature sets;
  do not accidentally trigger a second compiler build through environment drift.
- Acceptance: runner self-tests demonstrate independent budgets, explicit override
  behavior and bounded total allocation. Measure representative preparation with
  the previous and new allocation under matched inputs, recording memory and
  elapsed time. A performance improvement is not assumed from source inspection.

### R2 — preparation failures block only their dependents

- Express the required preparation for selected checks using existing execution
  plans. Split the single aggregate preparation outcome into identifiable results
  and dependency edges. Deduplicate only genuinely matching preparation inputs.
- Under `--no-fail-fast`, continue independent selected checks when one preparation
  fails; mark its actual consumers blocked with a precise prerequisite reason.
  Preserve the documented fail-fast behavior when that option is absent.
- Shared invalid inventories, unsafe source identity, cancellation and exhausted
  host capacity can legitimately prevent further work. Do not execute consumers
  without valid prerequisites or bypass performance-reference admission merely
  to obtain more green rows. Separate functional dependencies from performance
  admission only where the existing guarantee contract permits it.
- Acceptance: deterministic self-tests cover two independent graphs, shared
  prerequisites, a failed preparation, timeout/cancellation, global unsafe inputs,
  default fail-fast and no-fail-fast execution. Missing, failed or blocked required
  results must leave the aggregate nonzero. Verify complete selection/outcome
  accounting and unchanged coverage in emitted plans.

### R3 — make preparation and execution costs visible

- Extend existing runner receipts and E2E stage output to distinguish compiler
  preparation, fixture compilation, generated native builds and execution. Include
  stage start/end, elapsed time, outcome, selected counts and effective budgets.
- Identify the active command and report Cargo lock waiting or the last observed
  compile/link unit when diagnostics support it. Label unknown residuals honestly;
  do not infer a hung process from missing output or double-count overlapping work.
- Use existing timeout/process-group handling. Ensure cancellation reaps owned
  descendants and preserves logs. Separate a correctness failure, infrastructure
  failure, timeout, blocked prerequisite and a performance-budget failure.
- Report both command-to-result time and individual stages. Moving costs into
  preparation must not create an apparent end-to-end speedup.
- Acceptance: parser/receipt/process tests cover normal completion, failure,
  timeout, cancellation and incomplete output. Reports reconcile totals and
  outcomes, and no wrapper suppresses failure or executes an unverified artifact.

### E1 — Cargo freshness with persistent native dependencies

- Remove the outer success-entry shortcut as authority for native freshness.
  Every generated project must receive Cargo validation before executing an
  artifact. Do not fix this by extending TTL or adding another incomplete key.
- Use an owned persistent native target across generated groups. Cargo remains
  responsible for dependency/toolchain/profile/feature compatibility; reuse does
  not mean collapsing distinct features or changing lockfiles.
- Consume the matching executable from Cargo's structured artifact output,
  checking the expected package/target and successful build status. The old
  wrapper's copy-to-expected-path behavior is measurement scaffolding only.
- Preserve the generated-source/dependency-plan authority and write files only
  when bytes change. Do not reuse a pre-error executable after a failed build.
- Retain native dependencies on failure. Separate transient group retirement
  from target retirement; protect leases/consumers, interrupted runs and ownership.
- Start with one native Cargo invocation at a time and an explicit Cargo job
  budget; measure before changing profile defaults. Concurrent invocations must
  not race on generated roots, manifests, artifact selection or cleanup.
- Define the existing `--no-cache`/environment control's exact post-change
  semantics in help and tests. It must never weaken freshness. A cold benchmark
  uses a new owned target rather than pretending disabled outer reuse is cold.
- Acceptance: real runtime-source edit invalidates a prior pass; unchanged native
  dependencies are not repeatedly compiled; fixture inputs/outcomes remain equal;
  missing/corrupt artifacts, toolchain/profile/features/configuration/source
  changes, failed/interrupted builds, and concurrency are covered. For any
  content-bound reuse claim, test content changes with preserved timestamps;
  do not imply Cargo alone proves a stronger contract than it implements.

### E2 — isolate runtime output from compiler inputs

- Execute each filesystem-writing fixture in a unique owned scratch directory;
  prefer a consistent per-fixture execution directory if resource-path contracts
  allow it. Pass required resources explicitly and preserve test semantics.
- Ensure timeouts/cancellation reap children and keep failure evidence. Clean
  only completed, owned scratch outputs under the retention contract.
- Keep identity checks; do not broadly ignore real compiler inputs to hide the
  false invalidation. Cover parallel execution and same-named temporary outputs.
- Acceptance: the hashlib fixture and filesystem/resource users pass; repeated
  execution leaves source inputs untouched; subsequent unchanged preparation is
  fresh, with the same executable identity and no unrelated compiler rebuild.

### E3 — explicit failures and complete independent feedback

- Surface nonzero structured lowering-error results through the canonical error
  contract before launching doomed native builds. Do not scrape text or mark
  unsupported generated code as success. Keep native compilation for errors only
  Rust detects; 18/19 historical direct failures had early counters, one did not.
- Keep normal batching on the passing path. Provide one bounded diagnostic
  isolation path for failed native batches: retry their members individually
  using shared dependencies, without recursive retry cascades. Preserve the
  original failed-batch result even if isolated members pass (possible interaction
  defect). This is explicit test failure isolation, not a compiler fallback.
- Classify infrastructure/timeout failures separately; do not fan out retries
  after unavailable tools, disk exhaustion or cancellation. Report unreached
  fixtures as blocked/incomplete and keep the aggregate nonzero.
- Preserve lexical discovery, expectation declaration order and deterministic
  reporting. Count unique fixtures separately from diagnostic attempts.
- Acceptance: one deliberately broken fixture cannot hide the outcome of a
  healthy sibling during diagnostic isolation; original failure remains visible;
  mixed failures, multiple failed groups and cap exhaustion never create a pass.

### E4 — reuse immutable compiler setup within one invocation

- Reuse provider/context/native toolchain resolution within a frozen invocation
  input snapshot. Prefer worker-local contexts where the API is not safely shared;
  do not introduce a global mutable cache or silently reuse state across runs.
- Separate reusable immutable inputs from mutable per-fixture analysis state.
  Preserve interop, rooted-entrypoint policy, source maps and diagnostic behavior.
  Keep `check` independent of native toolchain preparation.
- Acceptance: all selected fixture outputs match (generated Rust, source maps,
  required modules/features, interop identities and lowering statistics); negative
  diagnostics, fixture ordering, concurrent workers, changed sysroot/tool/config
  inputs and repeated compilation do not leak state. Reproduce a measured gain
  on the actual candidate; the historical serial probe alone is insufficient.

### E5 — final qualification and measured handoff

- Complete E1–E4 and R0–R3, required harness/area/profile checks on the final candidate, reconcile
  compiler-failure dependencies, and run the authoritative gate under current
  policy. No passing full-gate claim while required failures or skips remain.
- Publish raw commands, source/tool/profile identities, inventory, stage times,
  per-group compiler counts, outcomes, resource data and sortable category sums.
- Repeat comparable warm measurements at least three times; report all samples,
  median and range. Use matched reference/candidate conditions and preserve
  failed attempts. Keep the initial empty-target observation separate from warm
  performance qualification; do not claim a percentile from three samples.
- Target: fully passing warm `test_e2e_pass` under five minutes in the declared
  benchmark environment, with Cargo freshness, full selection and assertions intact. Measure outer
  executable preparation separately and report command-to-result time as well.
  This is an objective to verify, not an already achieved SLA. If missed, report
  the measured residual and revise the next bounded item; never cut coverage.
- Empty-target reference is 6m24s, not a promised sub-five-minute cold gate.
  Successful repairs may add native work previously skipped after errors.

## Compiler-failure dependency register

Initial technical owner: emitted-Rust/codegen workstream, coordinated through
[Emitted Rust Excellence](ad-hoc-emitted-rust-excellence.md). Named implementers
and issue links must be assigned at E0 after verifying the current reproduction.
Do not assume every fixture name represents a distinct root cause.

| Area | Recorded direct-failure fixtures |
|---|---|
| Async | `async_generator_aclose_before_start`, `async_generator_aclose_result`, `join_set_add_task_join_all`, `join_set_bound_terminal_await`, `join_set_cancel_all_evidence`, `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`, `join_set_spawn_cpu_join_all_ordered`, `task_context_propagation_basic`, `task_group_spawn_cpu`, `task_group_spawn_cpu_user_error`, `task_scope_spawn_blocking` |
| Networking | `network_http_tls_loopback_split` |
| Processes | `process_scoped_parent_cancel`, `process_scoped_spawn_handle` |
| Types | `enum_value_property`, `enum_valued`, `newtype_basic`, `callable_struct_field` |

The first 18 have structured lowering/emission errors; `callable_struct_field`
emits a field call as a nonexistent `Config.callback` method (Rust E0599).
Detailed compiler diagnostics are in `failure-attribution.json`. Record a minimal
reproducer, owning issue, affected checks and status for each root-cause cluster.
Harness work may progress independently while these remain explicit failures;
final green-gate qualification depends on resolution by the owning workstream.

## Quality contract and commands

Run commands from the owned repository root with pinned tools. Use the preparation
strategy and cap above for costly commands; do not begin with the whole table.
Existing focused commands to use according to the changed item:

```bash
cargo test --locked -p sifr --test e2e harness_behavior_tests
cargo test --locked -p sifr --test e2e dependency_plan_authority_tests
cargo test --locked -p sifr --test e2e test_cache_root_from_env_resolution
cargo fmt --check
python3 scripts/check_file_size_guardrails.py
uv run --project verification --locked python -m sifr_verify --self-test
```

Add focused regression tests for each implementation defect above. For E4, select
the affected driver/sysroot/compiler API tests and diagnostic/interop contracts
from the actual changed paths. Preserve and run affected E2E fail/runtime-fail,
emitted safety, structured-lowering and codegen-parity checks; the pass corpus
does not replace them. Record exact test names in the item's evidence ledger.

R1–R3 require affected profile/configuration-plan, Cargo preparation, step-budget,
receipt and process-control self-tests. Inspect the existing tests alongside
`cargo_crate_setup.py`, `cargo_setup.py`, `profile_runner.py`, and `step_budgets.py`
before adding new cases. Compare emitted selection and configuration plans before
and after changes; altered scheduling must not remove checks or their prerequisites.
Run relevant workflow contracts when the execution plan or reporting changes.
No compiler build is needed for pure mocked runner tests; these tests do not
substitute for a real candidate execution at acceptance.

Canonical pass entrypoint (full merge selection unless explicitly diagnosing a
named subset; subset runs never substitute for final full selection):

```bash
verification/runner/e2e/run_e2e_pass.sh --profile merge
```

If profiles, selection or runner policy change, run profile checks and compare
emitted plans; preserve the same selected guarantees and update affected runner/
workflow contracts together:

```bash
uv run --project verification --locked python -m sifr_verify profiles check
scripts/run_all_tests.sh --profile merge --emit-plan
```

Implementation PR/merge validation follows `AGENTS.md`: `create-pr` for the PR
candidate and `merge` for the final implementation merge candidate, plus named
affected checks. No new phase-end-only gate exception is granted here. Use the
closure skill's review/merge process when implementing; do not initiate external
review or a PR merely for this requested planning file. Actual release/live
integration retains its separate required contracts.

Planning-only validation: `git diff --check`, local-link verification, status and
scope checks. No compiler build, broad gate or performance rerun is needed to
register this document.

## Measurement matrix for implementation

Measure preparation separately from Sifr compilation, native dependency/root
compilation, linking, execution and reporting. Use existing timing facilities;
avoid a new profiling framework. Collect these conditions on exact candidates:

1. Empty owned native target with compiler/toolchain already prepared.
2. Unchanged warm run with every generated project Cargo-validated.
3. One fixture edit, including generated Rust changes; stable versus new group.
4. Runtime/stdlib source edits, including an intentional compile error in a
   disposable copy; old artifacts must not hide the error.
5. Representative lowering, codegen and shared-type/API edits: separate compiler
   preparation rebuilds from subsequent fixture work. This identifies whether a
   future crate split would reduce the critical path.
6. Changed toolchain/profile/features/configuration, missing artifact, interruption,
   concurrent attempts and failed-batch isolation.
7. Source-cwd versus scratch-cwd fixture execution followed by preparation checks.
8. Compiler-preparation job allocation independently varied from native-batch
   allocation, with CPU/memory limits recorded and no unrelated concurrent build.
9. An injected preparation failure with an independent selected check: compare
   default fail-fast and `--no-fail-fast`, including aggregate failure status and
   time to independent feedback. Preserve all blocked dependency records.

For each attempt retain command/environment inputs, selected fixture identities,
start/end, exit/cap cause, wall/CPU/peak memory, free disk/reserve, worker budgets,
fresh/rebuilt Cargo units, generated inputs and failure logs. Distinguish unique
compute from shared-target lock waits. Never divide batch dependency time evenly
among fixtures and present that as measured per-fixture compilation cost.

## Deferred crate decomposition decision

No broad split is approved by this plan. After E1/E2, use representative edit
timelines to identify expensive rebuild fan-out and serialized compilation.
Propose a small stable API boundary only when it removes measured downstream
work without changing semantics or increasing coupling. Compare warm edited
iterations and cold preparation; include linking and feature combinations.
If warranted, register a separate bounded item with before/after evidence.

## Execution ledger and next action

| Item | State | Candidate / PR | Validation / review | Next action |
|---|---|---|---|---|
| Planning | recorded and extended 2026-10-18 | none | documentation checks; local links and all 19 recorded failures reconciled | Implementation remains pending |
| E0 | pending | none | historical evidence retained | Reconcile source/artifacts/ownership and reproduce minimal controls |
| R0 | pending | none | none | Concise agent and verification iteration guidance |
| R1 | pending | none | none | Separate compiler-preparation and native-batch budgets |
| E1 | pending | none | none | Cargo authority and shared native dependencies |
| E2 | pending | none | none | Isolate fixture outputs |
| R2 | pending | none | none | Dependency-aware preparation and independent failure handling |
| E3 | pending | none | none | Early errors and bounded failed-batch isolation |
| E4 | pending | none | none | Invocation-scoped setup reuse |
| R3 | pending | none | none | Stage reporting, actionable progress and cancellation evidence |
| E5 | pending | none | none | Resolve qualification dependencies and measure final candidate |

First implementation session: read this file and linked authorities, inventory
the current owned checkout and available retained artifacts, then execute E0 only.
Record the exact R0/R1 and E1 slices and whether compiler preparation is available. Preserve
the pre-existing dirty `third_party/ruff` worktree; it is not owned by this plan.
No product implementation, compiler defect repair, gate waiver or performance
guarantee has been delivered by registering this phase.
