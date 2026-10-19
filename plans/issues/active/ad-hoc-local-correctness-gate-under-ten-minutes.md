# Ad Hoc Phase: Complete Local Correctness Gate Within Ten Minutes

status: in progress; integrated acceptance pending
registered: 2026-10-19
phase_id: LOCAL-GATE
current_stage: V0.1/V0.4 reviewed; V4a correctness-only selection next
measured_baseline: `422c282720fd630aa938674f122d84aea7f1e26f`

## Objective and authority

Deliver one complete local correctness invocation in **less than 600 seconds**,
including candidate preparation, required clean qualification, test execution,
evidence reconciliation and child cleanup. Aim for 480 seconds of engineering
headroom; that is a target, not a measured forecast. Preserve every valuable
assertion, variant, negative control, isolation boundary and durability contract.
The complete integrated result is not yet demonstrated. A faster failed or
incomplete run cannot close this phase.

The user approved writing and merging this plan after the local E2E and full
verification investigation. This planning PR does not implement the changes,
change test selection or claim implementation acceptance. Work locally first;
CI rollout is a separate later stage. No device address or machine identity is
part of the design.

Use the [phase closure loop](../../../.agents/skills/phase-closure-loop/SKILL.md),
[agent instructions](../../../AGENTS.md) and
[verification contracts](../../../verification/README.md). Execute one bounded
item at a time. Subdivide large V0 repairs and V1 architecture work into explicit
owned items in this document before implementing them. Start the next item in a
new session. Preserve unrelated work and failed evidence.

This phase follows the
[E2E iteration phase](ad-hoc-e2e-iteration-throughput-and-cache-correctness.md),
whose implementation closed with known failures and qualification deferred. It
coordinates with the
[validation/resource execution plan](ad-hoc-validation-contracts-and-resource-aware-execution.md)
and [emitted Rust work](ad-hoc-emitted-rust-excellence.md). Reconcile ownership
before touching an externally owned failure. Explicitly assign in-scope repairs;
do not absorb unrelated issues or claim their failed paths as passing coverage.

The earlier E2E one-time merge exception and the archived DX policy do **not**
apply to this phase. This phase has its own user-approved prospective execution
policy below. Documentation-only planning/record updates require documentation
checks only. Actual releases retain release qualification.

## User-approved prospective execution policy — 2026-10-20

The user explicitly selected **“Focused intermediate checks; full final gates”**
in response to the proposal to use focused tests and independent review for
intermediate repairs and complete create-PR/merge gates on the final integrated
phase candidate. This supersedes the original planning-only requirement to run
full candidate gates for every intermediate implementation PR; it does not waive
any final coverage, failure semantics or timing acceptance.

Each intermediate item requires named affected tests, regression controls,
applicable area/workflow contracts, required formatting/file-size checks, and a
SATISFIED scoped independent review on its candidate. Only then may it merge.
Record failures owned by other phase items and preserve their evidence rather
than repeating the same broad failing gate unchanged. Register the next bounded
item and its checks before implementation.

For the V0.1/V0.4 distribution admission candidate, named validation is: enrolled
metadata producer tests (including producer/packager version parity and unsupported
format rejection), real retained-compiler metadata admission/rejection diagnostics,
all four synthetic governance archive admissions, the full distribution suite,
the qualification suite, file-size guardrails and documentation checks. No Rust
compiler source changes occur in this item. All original suite variant labels
must be preserved, with the new regression enrolled as additional coverage.

The final integrated implementation must pass the complete create-PR and merge
gates under the explicitly governed final profile policy, plus V5's full timing,
edit/invalidation and coverage matrix. V4 still owns the approved correctness-only
selection boundary; benchmark/reference or foreign execution is not silently
removed by this intermediate policy. An unavailable mandatory final prerequisite
leaves final acceptance open. No full-gate pass or performance-reference pass is
inferred from focused tests.

## Frozen scope and accounting

The baseline is the **complete current-host merge correctness selection**, not
every nightly, release or live-service configuration in the repository:

| Obligation | Baseline inventory |
| --- | ---: |
| Local area correctness suites | 105 |
| Cargo configurations, including native configurations | 17 |
| Guardrails | 15 |
| Positive E2E fixtures | 729 |
| Negative E2E inputs | 569 |
| Runtime-failure E2E inputs | 6 |
| Native case roles | 87 |
| Additional session/provider equivalence oracle | 1,298 inputs when affected |

These are overlapping coverage dimensions, not quantities to add together.
[The frozen scope](local-correctness-gate-evidence/scope.json) records selected
suite/case identities, Cargo commands, guardrails and source-input hashes.
It is a historical comparison artifact, **not a second executable selection
catalogue**. Generate actual selection from canonical verification manifests.
Reconcile new tests and changed contracts explicitly; these counts are not caps.

Exclude the representative benchmark, the cloud paired performance capture and
foreign-platform execution from this goal. Preserve local platform-policy,
packaging, installed-artifact, sanitizer, reproducibility and release-correctness
assertions already selected by the local gate. Explicit live-service and actual
release qualification remain separately required when applicable. The existing
merge profile includes a benchmark: do not call the unmodified profile the new
correctness-only entry point. V4 must define and validate that policy boundary
without silently moving correctness coverage to another lane.

The timer starts **before candidate preparation** and ends after final result,
evidence reconciliation and cleanup. Include compiler/test rebuilds, fixture
setup, installed artifacts, metadata, mandatory independent clean builds,
invalidation and restoration of ordinary development context. Report initial
toolchain/dependency bootstrap separately; never disguise a required per-run
clean build or an edit-triggered rebuild as bootstrap.

## Evidence and engineering decisions

All measurements below are scoped observations at the recorded baseline, not an
additive forecast. Different categories overlap, run with different cache states,
and sometimes fail before their downstream assertions. CPU/unit durations are
not wall-clock durations. The full green local gate remains unmeasured.

The original broad campaign retained 401 attempts: 295 successful processes,
102 failed and four disk interruptions. Its 53 categories and 2,578 nested timing
rows are diagnostics, not a single gate. The supplemental feasibility campaign
completed **21 timed experiments, all exit zero and below the 30-minute cap**,
without production source changes. The
[portable measurement manifest](local-correctness-gate-evidence/measurements.json)
retains individual attempt durations and raw-evidence hashes. Raw bundles remain
outside Git; hashes are provenance, not a substitute for execution evidence.

| Investigation | Observed result | Consequence |
| --- | --- | --- |
| Component/MySQL/PostgreSQL/SQLite suites | Serial mean 224.881s; bounded parallel compilation mean 73.452s. Two runs per mode, same 78 passing tests and one ignored live-MySQL test outside offline selection. | Promising latency improvement; retain semantic and resource controls. CPU rises from 222.395s to 259.720s; this is not eliminated work. |
| Component preparation | Candidate preparation 575.280s plus first execution 72.333s = 647.613s in a partially prepared target. | Warm execution alone cannot qualify the ten-minute objective. |
| SQL complete clean A / unchanged A / independent clean B under diagnostic opt0 | 407.926s; clean stages 194.896s and 199.896s; unchanged 6.829s. All three executable hash sets equal within profile; clean stages each emitted 498 non-fresh artifacts and unchanged emitted 498 fresh artifacts. | Retain independent clean qualification. This profile is a diagnostic, not an accepted default. |
| Matched SQL clean control with normal optimization | 424.812s for one clean build, same source/toolchain/storage/four jobs, debug off and LTO off. | Optimization is a major cost; this is one clean control, not a repeated full baseline protocol. Cached OS inputs and sequential order remain limitations. |
| Matched metadata validation profile comparison | 2.557530s normal versus 17.126814s opt0: 6.697 times slower. ABBA fresh-process order; six validations/profile; 498 package/target/kind/feature entries match. | Reject blanket opt0. Choose selective profiles using complete iteration cost and runtime correctness. Probe uses the same supplied metadata identity; it is not whole-product qualification. |
| Compiler and generated-program ASAN | Original unprepared compiler build 1,403.204s. Exact retained preparation 13.558s and 4.138s; existing-fixture diagnostic 24.359s with both instrumentations retained. | Preserve instrumentation. Original missing fixture is still broken; representative compiler-edit cost remains unmeasured. Symbols and generated fingerprint verified; short final native exec was not sampled. |
| SQL dependency audit | Replacing driver imports with compiler_services retains codegen, lowering and Wasmtime paths for all three tools. | A naming/import shuffle is insufficient; investigate real semantic/native ownership boundaries. |
| 44-command matrix | 875.204s → 379.913s with stable nested Cargo context → 153.225s using selected CLI directly, including 4.171s preparation. All assertions pass. | Correct nested invocation context and reuse selected products. Sequential cache order limits attribution. |
| Policy checks | 487.134s → 133.209s with pure scan memoization → 75.092s with three processes. Same 54 tests: 52 pass, two existing failures. | Reuse pure work; repair failures before claiming gate qualification. |
| Python examples | 29 cases / nine suites: 981.262s first successful state → 303.088s warm. | Existing reuse is already valuable; do not count it again as a new gain. Preserve certification commands. |
| GCQ preparation | 840.086s first root → 26.381s same root → 831.895s fresh root. | Separate revision-owned outputs from compatible dependency storage and fix incomplete identities. Ten following gates still fail; no green claim. |
| Advanced native tests | Preparation 1,152.3s versus runtime 45.895s; warm 71.422s still launches 24 rustc processes. | Investigate preparation/freshness, not shorter runtime assertions. |
| PostgreSQL native parser | Major switches trigger approximately 32–38s C rebuilds; explicit 18 → default 18 caused another approximately 33s rebuild. | Make compatible version identities explicit and allow required variants to coexist. |
| Durable history | 4,097 publications take 227.655s. | Attribute repeated hashing/decoding; preserve publications, fsync, ABA and retention tests. |
| Self-update target sharing | Naive shared target 136.517s versus private 10.882s. | Reject blind target sharing across incompatible version contexts. |
| Codegen unit tests | 1,756 assertions run in about 4.1s warm; libtest preparation has at least 851 sampled CPU-seconds and about 4.5 GiB process RSS. | Measure cohesive integration targets for public behavior; merely splitting source files cannot fix one large compilation unit. |

Timing labels matter: the SQL clean-control 424.812s is its inner Cargo stage;
the complete diagnostic attempt is 426.172s in the measurement manifest.
Protocol totals and test-command durations are labeled separately from component
stages. The manifest retains outer attempt times, including diagnostic overhead;
these must not be substituted for full-gate elapsed time.

The prior narrow E2E phase recorded a 98.585s median with 710/729 positive
fixtures passing. The later whole-target diagnostic took 193.54s, with a 94.280s
positive interval included inside it. These are different measurements and
neither demonstrates full green verification. Four of 87 native roles also fail.
The separate session-equivalence oracle took 224.78s and applies to affected
session/provider changes; it is not silently part of an unchanged merge total.

## Implementation work log — 2026-10-20

The user authorized continuing the phase sequentially on the remote execution
host until complete, using pragmatic diagnostics while preserving final correctness.
This supersedes the per-item new-session pause for this execution. The approved
prospective policy above governs candidate-gate timing; review, coverage and final
timing acceptance remain required. Work remains
bounded and sequential. V0.1/V0.4 acceptance becomes effective when PR #4357 merges.

| Item | Bounded ownership / prerequisite | Status |
| --- | --- | --- |
| V0.1 | Distribution metadata format admission and its producer/fixture parity controls | Validated and independently approved; effective on #4357 merge |
| V0.2 | SQL provenance and moved-source qualification guards | Pending |
| V0.3 | Canonical launch context and Python runtime registry/fixture admission | Pending |
| V0.4 | Distribution archive baseline fixtures and downstream governance assertions | Validated and independently approved; effective on #4357 merge |
| V0.5 | Maintained Cargo manifest/lock inventory admission | Pending |
| V0.6 | Driver interop test inputs, output/key contracts and ownership/pruning | Pending |
| V0.7 | Nominal collision dispatch (coordinate emitted-Rust owner) | Pending |
| V0.8 | Generated method-slot argument conventions (codegen owner) | Pending |
| V0.9 | Ancestry constructors and consuming-upcast visibility (codegen owner) | Pending |
| V0.10 | CLI eager/deferred schema parity | Pending |
| V0.11 | Nineteen positive E2E failures (coordinate #4344) | Pending |
| V0.12 | Demo async-generator/TryExcept emission and generated freshness | Pending |
| V0.13 | Sanitizer fixture/target propagation with instrumentation retained | Pending |
| V0.14 | Process lease inheritance and complete preparation dependency ordering | Pending |

V0.1 starts with packaging Python regressions before any compiler rebuild. Its
scope includes matching the canonical format, keeping unsupported-format rejection,
and making synthetic fixtures follow the real producer contract. Required evidence:
focused producer/binding/corruption tests, applicable distribution contracts, and
real retained compiler metadata admitted by the repaired packaging boundary.
A retained compiler can qualify unchanged compiler bytes only; it does not qualify
subsequent compiler edits. All final installed assertions and full gates remain open.

V0.1 evidence: the new producer-version regression failed before the repair
(format 4 versus producer format 5), then all six focused packaging checks passed
in 0.74s. A real retained compiler produced 51,025 records / 6,556,655 bytes;
packaging admitted format 5 and rejected rebound format 4/6 mutations (29.735s
including production). File-size guardrails passed. A complete distribution-full
run took 102.757s: 66/71 variants passed; five governance fixtures fail before
their intended assertions because required SQL runtime files are absent (V0.4).
The focused regression is now enrolled in the canonical distribution case scan.
Two earlier diagnostic attempts were explicitly stopped, preserved and not passed:
one launched a cold self-update target despite retained owned artifacts, and one
retry began before cross-filesystem target restoration completed. Retained build
selection and pinned Node were then restored without production scope changes.
Evidence bundle: `local-gate-implementation-20261020`, receipts `v01-*`.

V0.4 evidence: direct valid-fixture admission first failed on the missing SQL
runtime manifest (0.681s). Adding both required SQL runtime manifests made all
four synthetic target archives pass the unchanged production validator (0.689s).
The existing negative missing-SQL-runtime controls remain in the full suite.
All nine qualification groups passed (28.702s). Full distribution now passes
72/72 variants (119.253s), including the newly enrolled metadata producer check;
all original 71 variant labels are retained. These are local synthetic platform
contracts, not foreign-platform native execution or release publication.
No ten-minute whole-gate or complete phase acceptance is claimed.

The initial broad create-PR diagnostic stopped at performance-reference admission
in 0.677s before compilation; the reference remains owned by #4347. All downstream
steps are blocked, not passed. This failure is retained as
`v01-v04-create-pr`, and the approved intermediate policy avoids repeating it
unchanged while phase repairs proceed. No reference was fabricated or substituted.

V0.1/V0.4 closure: [#4357](https://github.com/sifr-lang/sifr/pull/4357),
reviewed candidate `027f6f752edcde539596386bc296576aadb1cf1c`, base
`59ff7f254aab2385e6196abec12265b1ced6ba26`. Independent review SATISFIED,
no blocking findings; raw review is outside Git in the implementation bundle at
`reviews/027f6f752edcde539596386bc296576aadb1cf1c/review.md`.
All named focused/area checks above passed; final file-size guardrail passed in
6.684s, changed local links and whitespace passed, and inventory comparison
confirms no original distribution variant removed. Final record-only changes do
not alter reviewed implementation inputs. No full gate or timing acceptance is
claimed. Nonblocking suggestions about compiler-reported format discovery and
standalone fixture constant sharing are deferred to future format-contract work;
the enrolled regression already fails closed on producer/fixture drift.

Next bounded item **V4a** is brought forward to support the user's approved broad
failure-discovery approach: derive an explicit correctness-only selection from
canonical delivery profiles, preserving all correctness suites, crate configurations,
guardrails and E2E semantics. Separate its evidence identity from full profiles;
keep benchmark admission mandatory whenever benchmarks remain selected. Do not
change default full-profile behavior. Required checks: runner/profile self-tests,
profile schema validation, emitted-plan comparisons with the frozen scope,
facade argument forwarding, and affected workflow contracts. Then use broad
no-fail-fast diagnostics to collect independent failures under the existing
per-subcategory 30-minute/resource caps. Complete final acceptance remains open.

## Execution stages

V0 establishes executable coverage and ownership first. V1 is the first hard
runtime feasibility decision. Work on independent measured optimizations only
with matched coverage/outcomes and explicit prerequisites; none can close the
phase while blocked assertions remain. V2–V4 deliver the resulting design and
V5 qualifies the integrated result. Prefer a small explicit dependency graph,
bounded in-process reuse and stable owned artifacts. No new daemon, distributed
runner, fallback compiler or elaborate scheduler is justified by current evidence.

### V0 — Restore executable coverage and freeze its inventory

- [ ] Implementation and acceptance complete.

Current evidence: Blocked assertions are identified; repairs not implemented

Decision: No final timing promise until the failed setup paths execute their downstream
assertions. Independent measured optimizations may proceed with matched outcomes; they
cannot close this item.

Work: Resolve the 13 recorded blocker families with their owners, as bounded repair items
with their own regression checks; do not bundle unrelated fixes into one opaque change.
Include metadata schema/installed packaging, native codegen failures, loader selection,
moved-source guards, Python registry drift, fixture omissions, SQL provenance, GCQ freshness
and process leases. Bind Python preparation to every actual SQL/LSP/crate consumer.
Preflight missing fixtures and supported tools cheaply. Preserve independent continuation
after a failed prerequisite while marking its consumers blocked.

Required proof: Regenerate the canonical selected plans; inventory observed test IDs,
configuration, variants, negative controls and required output markers. Require the old and
new coverage obligations to match, with explicit ownership for duplicates. Failed setup,
missing results, zero tests and required skips reject completeness. Run every previously
blocked assertion after its actual repair.

Reject: Do not change assertions to match a bug, count a filter string as executed coverage,
or count an unexecuted installed check as a fast pass.

### V1 — Prove the mandatory build path can fit the budget

- [ ] Implementation and acceptance complete.

Current evidence: SQL full-protocol profile trial and sanitizer retention tests recorded
above

Decision: Treat SQL as the first hard feasibility gate. Keep the measured whole critical
path, not individual unit-duration subtraction. The larger architecture work is conditional
on measured necessity.

Work: Keep clean A, unchanged A and independent clean B for all three linked SQL tools.
Measure optimization cost separately, then choose a coherent development profile only if
complete semantic/runtime checks justify it. Blanket opt0 is not an accepted default: its
faster build must be evaluated against the recorded metadata runtime regression. Prefer
measured selective changes that preserve runtime speed. If still too slow, split compiler
metadata production, frontend consumption and native emission by responsibility. SQL
migrations keep the canonical frontend; SQL profile execution keeps the real component host.
A protocol/host split can help other consumers but cannot remove real SQL execution. Inspect
the concrete coupling: CompilerContext prepares source metadata, semantic consumers share a
crate with Rust-support projection, and StdlibCode/InteropBuildPlan/PythonInteropPlan are
backend-owned shared data types. Move shared representations to their proper lower owner and
separate semantic consumption from Rust materialization only where behavior supports it;
moving imports alone leaves the graph unchanged. Keep an explicit producer/preparation
boundary, include its cost, and preserve identities, diagnostics, packaging and full product
reproducibility. The architectural candidate to investigate first is semantic
metadata/bootstrap separated from native Rust-support production, with shared
representations owned below the emitter. This could let SQL migration tooling keep canonical
semantic checking without embedding Rust emission. It is not implemented or
timing-qualified. Full compiler/native consumers must still produce and validate all
required Rust support; a semantic-only artifact must never satisfy a full installed/native
qualification. Any artifact-role or schema change needs an explicit contract, failure tests
and complete preparation accounting. Do not introduce a second parser/lowerer or an
unqualified external compiler shortcut.

Required proof: For an actual candidate, show resolved Cargo units before/after, linked
executable identities and within-profile byte equality, all seven build-qualification
mutation controls, SQL migration/provider/editor/host-tool tests, and source/installed error
parity. Include missing compiler/product, incompatible metadata and version rejection. For
sanitizer, test retained preparation and representative compiler edits with compiler plus
generated-program instrumentation before considering any scope change. Repair the missing
fixture explicitly and verify instrumentation of the actual executed outputs.

Reject: No warm cache substitute for an independent clean build. No prebuilt external
compiler hidden outside product qualification. No removal of compiler instrumentation merely
because the manifest labels the lane generated_binary. An opt0 diagnostic is not approval to
change the production profile.

### V2 — Remove repeated metadata and component work

- [ ] Implementation and acceptance complete.

Current evidence: Hotspots measured; component feature semantic trial above; caches not
implemented

Decision: Prefer bounded lifetime-local reuse. Start with Wasmtime parallel compilation only
if the full semantic checks pass and it fits the shared CPU budget; add compiled-component
reuse where repeated requests still dominate. Parallel compilation is a latency mechanism
and may increase CPU work; do not credit it as CPU-work elimination or let its Rayon pool
compete without accounting with other gate workers.

Work: Reuse validated immutable metadata payload/field facts within bounded validation;
retain per-root traversal/work/depth/cycle checks and distinct lazy-consumer accounting.
Share one compilation-scoped Engine/component owner across compatible requests. Revalidate
registration/artifact hash/protocol and create fresh Store/instance/fuel/memory state for
every execution. Keep request execution distinct from response-cache hits. For history,
attribute repeated decode/hash work before implementation; preserve all 4097 publications,
retention, corruption checks and fsync.

Required proof: Compare exact diagnostics, emitted bytes and runtime markers. Run malformed
metadata/reference/cycle/depth/work-limit/cancellation controls with cache hits, misses and
eviction. Corrupt bytes with unchanged path/mtime must reject. For components, execute all
providers and six PostgreSQL artifacts; test changed artifact, incompatible Engine policy,
hash/import/protocol errors, fuel/memory exhaustion, state isolation and bounded residency.
Run SQL CLI/editor ownership-lifetime tests; per-host caches are insufficient if callers
immediately discard the host. History tests retain real durability and concurrent
reader/ABA/crash/pruning behavior.

Reject: No global visited shortcut that changes graph limits, timestamp-only trust,
recursive mutex deadlocks, unbounded global caches, persistent native-code deserialization
or mocked durability. Compilation-only timings do not establish runtime semantics.

### V3 — Keep compatible builds reusable across real iterations

- [ ] Implementation and acceptance complete.

Current evidence: Input instability reproduced; lifecycle changes not implemented

Decision: Use stable leased ownership and complete identities, without caching a successful
test result in place of rerunning it.

Work: Centralize nested Cargo environment handling; launch the selected compiler directly.
Keep user flags/targets/features/wrappers. Separate revision-owned GCQ outputs from
compatible Cargo dependency storage and repair incomplete producer keys. Reconcile reusable
native/Python fixture trees by content under a lease. Preserve unchanged bytes/mtimes;
restore mutations, remove unexpected entries and reject symlink substitution. Keep distinct
trust/toolchain/version/instrumentation contexts. Make PostgreSQL major selection a
coexisting build identity. Isolate self-update versions unless both can coexist without
invalidation.

Required proof: Run complete 44-command matrices, all 87 native roles and all 29 compiled
Python examples. Compare exact argv semantics, stdout/stderr, exits, diagnostics and
markers. Exercise source/build-script/lock/compiler/feature/environment/trust changes,
default-explicit PostgreSQL18 and 13→18→13, same bytes rewritten, symlink swaps, killed
owners, child lease inheritance and two owners. Check actual Cargo dirty/fresh reasons and
rerun assertions on current inputs.

Reject: Do not omit identity inputs to manufacture cache hits. Do not resolve Python venv
identity to its base interpreter. Do not reuse one target blindly across incompatible
versions/flags. The measured naive shared self-update target was slower and is rejected.

### V4 — Reduce runner duplication and edit compilation costs

- [ ] Implementation and acceptance complete.

Current evidence: Policy/direct-CLI prototypes measured; final integration pending

Decision: A small explicit dependency graph and shared resource budget are sufficient. No
new daemon, distributed runner or elaborate scheduler is justified.

Work: Give identical within-run assertions one owner only after proving
source/configuration/environment/observed-selection equivalence. Retain distinct
feature/profile/mutation/reproducibility repetitions. Reuse pure scanner work and a bounded
pool. Compile each E2E positive once for safety/native execution. Disable only proven-empty
binary harnesses. Move public-behavior libtests into a few cohesive integration targets when
measured compiler cost justifies it; keep private invariants local. Audit redundant
default-feature identities and product-version invalidation. Define the local correctness
selection from one authoritative catalogue; do not fork a hand-maintained copy of the merge
suite list. Compare emitted plans so the only intended exclusion is the agreed
benchmark/foreign execution. Update verification docs and AGENTS.md command references for
the approved entry point; keep focused edit checks and full candidate acceptance. Final create-PR/merge requirements remain applicable under the governing policy;
the phase-specific prospective policy above covers intermediate items. The closed
DX phase exception does not carry into this work. Avoid adding an extra
duplicate complete gate when integrating the approved selection. CI changes remain a later
stage.

Required proof: Compare full assertion inventories and scanner findings, including existing
failures until repaired. Run policy negative controls and zero/missing/duplicate-evidence
rejection. Measure production-library, libtest and integration-target rebuild costs for a
real test edit, including added link time. Ensure CARGO_BIN_EXE and actual callers remain.
Use one CPU/memory budget for Cargo, libtest, Rayon, freshness workers and area parallelism.

Reject: Do not merely split source files within one compilation unit, make one binary per
assertion, expose private APIs solely for test movement, or claim separate overlapping
savings as additive.

### V5 — Accept only a complete, passing, repeatable sub-ten-minute gate

- [ ] Implementation and acceptance complete.

Current evidence: Not implemented or measured; acceptance remains open

Decision: Completion requires measured results, not confidence language or extrapolated
sums. Aim for 480 seconds so a result does not sit on the 600-second limit.

Work: Integrate the declared local correctness entry point and resource plan. Run all
selected assertions in one invocation with preparation and cleanup included. Fix failures;
rerun affected checks during work and the complete final gate on the final candidate.
Preserve every earlier failed/capped attempt.

Required proof: For each required scenario below, run at least three measured repetitions on
an otherwise idle declared resource envelope, report median and worst, and require every
qualifying run to pass in less than 600 seconds. Compare baseline/candidate with alternating
order where meaningful, equivalent reset state and separately logged setup. Record CPU work,
critical path, peak memory, artifact growth, blocking waits and cleanup. Three runs do not
justify a p95/p99 claim. An incomplete scenario remains open.

Reject: No moving mandatory clean builds outside the timer, silent test selection
reductions, failure-shortened wins, percentile claims from tiny samples, aggregate sums
mistaken for wall time, or treating a cancelled attempt as a pass.

## Correctness dependencies and repair ownership

These thirteen families were observed at the measured baseline. Their reason and
owner are starting points, not authorization to change expected results to match
bugs. Confirm current source before each repair, record the bounded child item
and follow the owning phase/issue. Preserve historical failed/incomplete receipts.
Existing E2E codegen work includes [#4344](https://github.com/sifr-lang/sifr/issues/4344).
The performance-reference dependency [#4347](https://github.com/sifr-lang/sifr/issues/4347)
remains separate; the correctness-only scope does not waive a current gate.

### Installed compiler and metadata qualification

The real producer emits metadata schema 5 while packaging validates schema 4. Package
creation fails, preventing installed equivalence, public emission, native execution,
relocation and corruption/recovery assertions. A failed 8.64-second warm package attempt does
not measure those assertions.

Repair the canonical producer/packager contract; then execute every installed assertion.
Independent diagnostic attempts preserve this blocker instead of modifying the artifact or
validator.

### SQL qualification mutation controls

Original qualification scripts fail their valid-baseline admission: component source
provenance has drifted and some textual ownership guards still inspect the former driver
location. Later mutation controls cannot be counted as executed.

Repair real source/artifact provenance and follow the canonical moved implementation;
preserve every mutation case and confirm that each still rejects its targeted defect.

### Original loader failures and remaining native compilation failures

The selected-toolchain loader context was diagnosed and the original failures retained.
All 29 declared compiled Python examples across 9 suites now execute and pass in the
diagnostic context. Native isolated follow-ups establish 83 passing cases and 4 generated-Rust
failures, detailed below. This closes the missing-execution measurement gap for the Python
examples but does not fix the canonical launch context, failed runtime registries or other
Python contracts.

Keep original failures visible. Diagnostic follow-up success is execution evidence, not a
repaired source launch path or a passing canonical gate.

### Distribution governance and publication-fixture assertions

Independent tests were attempted, but many stop while creating their valid baseline archive:
the fixture omits newly required SQL runtime crate manifests. Their intended downstream
mutations do not execute.

Repair the fixture against the real package contract and rerun each case. Preserve
required-file rejection; do not count the short failed setup as the complete test runtime.

### Maintained dependency version and checksum assertions

The recorded Cargo inventory omits 20 tracked manifests and 9 tracked lockfiles. Several
dependency tests fail admission before inspecting their intended resolved version/checksum
properties.

Review ownership of every new input, update the canonical inventory and audits, and rerun
the blocked assertions. Keep unknown/missing/duplicate-input rejection.

### Driver interop contracts blocked by incomplete fixture inputs

The complete driver library run reports 31 missing real Cargo manifests and one missing
bridge-source directory in tests that expected successful contract processing. These
failures occur before the intended interop assertions. Other failures concern
ownership/pruning and two output/key-shape assertions.

Repair the owned fixture/input boundary while preserving the production requirement for real
Cargo authority. Keep pure contract assertions distinct from real probe/runtime integration,
and rerun both; no missing-input fallback or skip.

### Native nominal-name collision semantics

The exact native case compiler_owned_nominal_paths_and_unions_build_beside_source_collisions
fails while compiling generated Rust: the user nominal __SifrIoFileHandle receives
builtin-method authority for value. Native runtime assertions do not execute. The original
failure took 28.787s.

Correct nominal identity/dispatch authority and rerun the unchanged collision and union
scenario plus affected builtin/runtime controls. This is a semantic defect, not a timeout or
cache-performance result.

### Generated method-slot argument conventions

The exact test_method_slot_runtime case fails in generated Rust before runtime: four calls
pass String where the generated target signature requires &str. Its 36.498s failed attempt
therefore does not measure the complete method-slot runtime assertions.

Owner: codegen method-slot call emission and argument conventions. Correct the generated
call/signature agreement and rerun the unchanged runtime case, including context variants.
Keep this semantic failure separate from performance and fixture-admission defects.

### Project ancestry and consuming-upcast native compilation

Two additional native cases fail before their runtime assertions: same-basename ancestry
emits a constructor call with owned SifrInt where &SifrInt is required (6.199s); structural
consuming upcasts expose private Mid through a public associated type (5.776s). Both are
actual generated-Rust failures.

Owner: codegen constructor argument conventions and generated visibility/upcast emission.
Repair these independently of timing changes, then rerun the unchanged native assertions and
related project/E2E controls. Their short failed timings are not complete runtime coverage.

### CLI eager/deferred schema parity after the test-command mismatch

Two composite parity tests stop at sifr test because the eager reference schema omits the
production locked/offline/frozen flags. Their later loop cases are not proven by the failed
aggregate.

Owner: CLI schema parity fixture. Align the reference with the intentional production
command contract, retain exact equality and all error/help/order checks, and rerun the
complete loops.

### Positive E2E fixtures blocked before native runtime

The unchanged-candidate positive inventory accounts for 729 fixtures: 710 pass, 18 fail during
source lowering, and callable_struct_field fails native compilation. Those 19 runtime
assertions did not execute.

Owner: affected lowering/codegen semantics. Preserve the individual errors, repair their
causes and rerun the original native assertions. Do not treat the failed 193.54s whole-target
attempt as a passing-gate prediction.

### Generated-code demo compilation failures

The canonical demos gate fails on unsupported async-generator aclose lowering. All later
independent demo entries were attempted; TCP echo/TLS loopback also fail on missing
structured TryExcept emission. These generated programs do not pass their selected Cargo
check contracts. Freshness independently completes its 264 comparisons and reports 43 stale
companions.

Compiler/codegen owner must repair supported async-generator and TryExcept emission, then
rerun the complete original demos gate. Review/reconcile stale companions under the intended
output contract; do not automatically bless them to obtain a pass.

### Generated-binary sanitizer fixture is absent

The original sanitizer manifest names an absent demo and remains unqualified. The initial
compiler-instrumented build took 23m23s. Subsequent exact retained preparation completed in
13.558s and 4.138s; a diagnostic existing fixture passed in 24.359s with compiler and
generated-program instrumentation retained. These successful diagnostics do not repair or
qualify the original manifest.

Repair the declared fixture and generated-target propagation while preserving compiler and
generated-program instrumentation. Check fixture existence before long builds. Verify the
actual executed outputs and measure representative compiler edits; retained preparation
alone does not establish edit-cycle cost. External-bundle references
`feasibility-sanitizer-proof.json` and `feasibility-scope-audit.json` are both
identified by hash in the committed measurement manifest.

## Focused validation and mutation controls

[Source-verified starting checks](local-correctness-gate-evidence/regression-controls.json)
list eight command groups and 21 named existing controls. They are entry points,
not the complete acceptance selection or evidence that those checks already ran
on an implementation candidate. Resolve filters to emitted test IDs, reject zero
tests and run every required ignored/conditional test explicitly. Refresh names
and paths when source changes.

Start with affected metadata, component, history, E2E cache or lease tests. Add
controls for each new mechanism: cache hit/miss/eviction, corrupt same-path bytes,
changed source/lock/toolchain/features/environment, kill/restart, competing owners,
symlink replacement, limits and failure recovery. Then run the full owning area
contract and applicable candidate gates. No existing assertion may be replaced
by a mock that bypasses its contract.

Use the exact pinned tool versions. Before long builds, record free disk, owned
target sizes, effective CPU/memory and allocation/reserve. Use one budget across
Cargo, libtest, Rayon and verification workers. Reclaim only inactive obsolete
session-owned artifacts. Diagnose slow preparation before repeating it unchanged.
For exploratory measurements cancel each subcategory at 30 minutes, terminate
its owned descendants, retain evidence, mark it incomplete and continue independent
categories. This cap never qualifies a failed or incomplete acceptance run.

Do not spend repeated compiler builds to prototype a pure runner change: use
runner self-tests, selection comparison and disposable fixtures first. Reuse an
existing instrumented/compiler product only when declared input identity proves
it still applies. For Rust changes, measure actual rebuilds; do not hide them in
manual preparation. Keep raw build messages and Cargo dirty/fresh explanations.

## Integrated acceptance scenarios

Except for the separately reported initial bootstrap, every required scenario
below must have at least three complete passing runs below 600 seconds to claim
this phase's full objective. Report median and worst observed duration; three
runs do not establish p95/p99. Restore equivalent pre-edit state before each edit
trial so repetitions measure rebuilds rather than an unchanged cache hit. Declare
exact patch, expected behavior, before/after identity and artifact reset boundary.
Use a declared otherwise-idle resource envelope; alternating baseline/candidate
order is required where meaningful. Include resource acquisition waits after the
invocation starts. Record the limits of the qualified envelope, not a universal
hardware-independent guarantee.

- **unchanged** — All artifacts retained; fresh test processes; full assertions execute.

- **test-only edit** — Real assertion/fixture change that demonstrably invalidates the relevant test or generated input; include rebuilt harnesses.

- **frontend edit** — Behavioral frontend/lowering change with matching expected results; rebuild affected compiler/test/native/ASAN paths and regenerate metadata where required.

- **codegen edit** — Behavioral emitted-code change; invalidate affected native outputs and preserve full E2E and installed checks.

- **runtime edit** — Change the actual Rust runtime dependency; rebuild affected generated programs and execute runtime/sanitizer checks.

- **session/provider edit** — Include the separate 1298-input session-equivalence oracle and its preparation, plus the full local gate.

- **new revision with identical relevant bytes** — Verify unrelated revision/output-root changes do not invalidate compatible dependencies or falsely bypass source checks.

- **bounded-retention cycle** — Repeat revisions and cache pressure until retention behavior is observable; no unbounded storage, active-owner eviction or stale pass.

- **toolchain/dependency/profile change** — Measure complete expanded invalidation after prerequisites are installed. Include every required compiler, artifact and test rebuild. This scenario must also pass below 600 seconds before claiming the goal covers these changes; do not silently classify rebuilds as bootstrap.

- **initial bootstrap** — Report downloads, environment creation and first compiler builds separately; never conflate this with per-run mandatory clean qualification.

For bounded-retention trials, measure each complete invocation through repeated
revision/pressure cycles and include required cleanup; do not average an oversized
run away. Toolchain/dependency/profile invalidation is an explicit acceptance
scenario after prerequisites are installed. If it cannot fit the budget, leave
that claim and the full phase acceptance open; do not silently rename rebuilds
as bootstrap or narrow the approved scope.

Record total elapsed time, non-overlapping process CPU work, critical path,
preparation/execution/cleanup stages, memory, disk growth, waits and executed
assertion inventory. CPU work divided by effective execution capacity is a lower
bound, not a runtime prediction. Parent and nested durations must not be added.
Final evidence must cover the exact integrated candidate and qualifying inputs.

Reject missing reports, zero tests, lost negative controls, wrong artifact roles,
stale identities, required skips, shortened failure paths and incomplete
consumers. Deduplication requires equality of actual input/configuration/selection
and one visible evidence owner. Distinct reproducibility builds and mutation
variants remain distinct obligations. Every original assertion must execute or
have an explicitly proved equivalent owner within the same complete invocation.

## Documentation, commands and policy delivery

V4 includes updates to AGENTS.md, verification/README.md, canonical profiles and
runner scripts only where needed to expose the approved local correctness entry
point and precise iteration guidance. Keep one authoritative catalogue; emit and
compare plans to prove benchmark/foreign execution are the only agreed exclusions.
Keep fast focused edit checks separate from final complete acceptance. Preserve
release/live requirements. Validate runner/profile self-tests and relevant area
contracts, file-size guardrails, and profile selection using:

```sh
uv run --project verification --locked python -m sifr_verify profiles check
```

The planning PR did not modify these commands or profiles. The approved prospective
policy above now governs intermediate implementation candidates. Until V4 explicitly
changes the governing contract, current full create-pr/merge requirements remain
applicable to the final integrated candidate. No extra duplicate full gate should survive the
approved integration merely because a new entry point was added. CI mirroring and
foreign-platform/benchmark optimization remain later work.

## Review, merge and completion record

Use one scoped independent review per bounded implementation item according to
the phase closure loop. Review exact base/candidate SHAs, paths, acceptance and
existing evidence. Only in-scope omissions/regressions block; record unrelated
follow-ups with owners. Never treat a timeout or missing review as approval.
Keep final candidate-keyed review evidence outside the reviewed Git tree.

For this planning PR: run `git diff --check`, verify changed local links, scope,
source-backed controls, evidence arithmetic and status. No compiler build or broad
gate is required. Merging this plan does not close V0–V5. Record the planning PR,
reviewed candidate, documentation validation and review evidence in the handoff.

| Item | Status | Merged PR / candidate / validation / review |
| --- | --- | --- |
| Phase registration | Recorded; effective when [#4356](https://github.com/sifr-lang/sifr/pull/4356) merges | Reviewed plan candidate `c7e80fb6cb310fb1d4ff64c2e0179a2d148fb467`; documentation checks pass; independent review SATISFIED; subsequent edits only clarify documentation and record this handoff |
| V0 executable coverage | Pending | None |
| V1 mandatory build path | Pending | Diagnostic evidence only |
| V2 repeated metadata/component work | Pending | Diagnostic evidence only |
| V3 compatible build reuse | Pending | Diagnostic evidence only |
| V4 runner/compilation/policy integration | Pending | Diagnostic evidence only |
| V5 integrated qualification | Pending | No complete green sub-ten-minute result |

Planning handoff: branch `codex/local-correctness-gate-phase`; review verdict
SATISFIED with no blockers. Candidate-keyed documentation validation and full
review are retained outside the reviewed Git tree under the investigation bundle's
`planning-pr-evidence/c7e80fb6cb310fb1d4ff64c2e0179a2d148fb467/` directory. The
linked PR records the final documentation head and merge identity, validation
summary and review. Reviewer suggestions about external references, timing labels
and formatting were incorporated as documentation-only corrections; no additional
implementation requirement or unresolved follow-up was introduced. No second
external review is required for these record/prose-only changes under the phase
closure loop. Implementation remains pending and there is no full-gate pass.

After all implementation items merge, close only when V5 passes. Record remaining
out-of-scope work separately; no unresolved in-scope coverage or timing scenario
may be described as completed. A missed target triggers further root-cause work,
not a scope reduction or a promised speedup based on overlapping timings.

**Next action:** finish the V0.1/V0.4 merge, implement V4a as registered above,
then collect broad independent failures and continue bounded V0 repairs. Preserve
this investigation's baseline and failures while collecting current evidence.
