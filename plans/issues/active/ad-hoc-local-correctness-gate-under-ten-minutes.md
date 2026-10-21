# Ad Hoc Phase: Complete Local Correctness Gate Within Ten Minutes

status: in progress; integrated acceptance pending
registered: 2026-10-19
phase_id: LOCAL-GATE
current_stage: V0.19 lint verification authority; V1 semantic/native separation remains open
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
edit/invalidation and coverage matrix. V4a delivers the explicit correctness-only
selection boundary described below; benchmark/reference or foreign execution is
not silently removed by this intermediate policy. An unavailable mandatory final prerequisite
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
bounded and sequential. V0.1/V0.4 merged in PR #4357 at `ae729a7c7aa4b4e0d38aa4a6c4aebf2651b50d9c`.

| Item | Bounded ownership / prerequisite | Status |
| --- | --- | --- |
| V0.1 | Distribution metadata format admission and its producer/fixture parity controls | Merged #4357; named checks and independent review pass |
| V0.2 | SQL provenance and moved-source qualification guards | Merged #4362; 43 native tests and 33 checks pass; review SATISFIED |
| V0.3 | Canonical launch context and Python runtime registry/fixture admission | Launcher merged in V0.3a; Python registries/fixtures qualified and reviewed in V0.3b |
| V0.3a | Rustup-selected native Cargo/rustc launch context | Merged #4360; full affected preparation and independent review pass |
| V0.3b | Python runtime registries and LSP selected-input admission | Merged #4363; 114 LSP tests and four focused area variants pass; review SATISFIED |
| V0.4 | Distribution archive baseline fixtures and downstream governance assertions | Merged #4357; named checks and independent review pass |
| V0.5 | Maintained Cargo manifest/lock inventory admission | Merged #4361; 108/108 tests pass; independent review satisfied |
| V0.6 | Driver interop test inputs, output/key contracts and ownership/pruning | Complete; all four repairs merged |
| V0.7 | Nominal collision dispatch (coordinate emitted-Rust owner) | Merged #4368; native and three E2E cases pass |
| V0.8 | Generated method-slot argument conventions (codegen owner) | Merged #4370; 1,766 codegen tests, both native checks and review pass |
| V0.9 | Ancestry constructors and consuming-upcast visibility (codegen owner) | Merged #4371; 1,770 codegen and 19 native tests pass; review SATISFIED |
| V0.10 | CLI eager/deferred schema parity | Merged #4372; all 12 named CLI tests and guards pass; review SATISFIED |
| V0.11 | Nineteen positive E2E failures (coordinate #4344) | V0.11a merged #4373; V0.11b merged #4374: all 732 positive and 569 negative fixtures pass; independent review SATISFIED |
| V0.12 | Demo async-generator/TryExcept emission and generated freshness | Merged #4375; complete named acceptance and scoped review pass |
| V0.13 | Sanitizer fixture/target propagation with instrumentation retained | Merged #4376; focused acceptance and scoped review pass |
| V0.14 | Process lease inheritance and complete preparation dependency ordering | Split into V0.14a preparation prerequisites and V0.14b process lease inheritance |
| V0.14a | Explicit pinned Python environment prerequisite for all selected consumers | Merged #4378; declared prerequisites, 114 LSP and two native bridge tests, runner controls and review pass |
| V0.14b | Process lease inheritance limited to the intended consuming children | E2E #4379 and production #4381 accepted; reusable source fixture ownership remains V3 |
| V0.14b1 | Explicit E2E suite child lease ownership | Merged #4379; focused checks, all 732 positives and scoped review pass |
| V0.14b2 | Explicit production child lease ownership | Merged #4381; named ownership/native/CLI checks and scoped review pass |
| V0.15 | Test-root include source-direction guard | Merged #4359; named checks and review pass |
| V0.16 | Method-policy inventory reconciliation | Merged #4369; full method guard passes |
| V0.16b | Application-profile fixture inventory reconciliation | Merged #4377; focused acceptance and scoped review pass |
| V0.16c | Deterministic canonical lowering-error negative control | Merged #4380; driver boundary, all 64 ordinary E2E tests and scoped review pass |
| V0.16d | SQL guest safety proof references after diagnostic projection | Merged #4382; complete unsafe audit and independent review pass |

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

Current bounded item **V4a** is brought forward to support the user's approved broad
failure-discovery approach: derive an explicit correctness-only selection from
canonical delivery profiles, preserving all correctness suites, crate configurations,
guardrails and E2E semantics. Separate its evidence identity from full profiles;
keep benchmark admission mandatory whenever benchmarks remain selected. Do not
change default full-profile behavior. Required checks: runner/profile self-tests,
profile schema validation, emitted-plan comparisons with the frozen scope,
facade argument forwarding, and affected workflow contracts. Then use broad
no-fail-fast diagnostics to collect independent failures under the existing
per-subcategory 30-minute/resource caps. Complete final acceptance remains open.

V4a implementation evidence: ten focused scope/admission tests pass (1.202s
outer command); all 26 runner foundation groups pass (57.729s). Profile schemas
pass (0.746s), local-first workflow admission and event/profile contracts pass
(0.758s), and file-size guardrails pass (5.737s). Real facade-emitted plans retain
all nonselection fields and remove exactly one benchmark suite per local profile:
create-pr 72 → 71 area suites; merge 106 → 105. The frozen correctness scope,
17 Cargo configurations, 15 guardrails and complete merge E2E corpus are retained.
Reports and area results use a distinct correctness identity; full-profile
reference admission remains tested. Independent review SATISFIED, no blockers, on candidate
`429882fe25f0afd2f723be9b680808d26452e8d0`, base
`ae729a7c7aa4b4e0d38aa4a6c4aebf2651b50d9c`; merged in
[#4358](https://github.com/sifr-lang/sifr/pull/4358) at
`1e27d55d660f90222d6ee785db546d2d9ab30b80`. Raw review is retained
externally under `reviews/429882fe25f0afd2f723be9b680808d26452e8d0/review.md`.
No whole-gate
pass or timing claim follows from these runner checks. Receipts: `v4a-*` in the
implementation evidence bundle.

The first correctness-only merge diagnostic (`v4a-merge-discovery`) reached all
five early guards and stopped after 16.190s: four passed, source dependency
direction failed on three codegen test includes and two guided replay provider
constructions. All dependent steps remain blocked; no compiler build began.
The runner retained the failing status and separate correctness evidence. The
unchanged failure is not rerun as acceptance. V4a review suggestions about adjusted
performance-area budgets and shared identical E2E/native cache semantics are owned
by V5's timing/reuse matrix, not unresolved V4a correctness blockers.

**V0.15 — source-direction admission:** classify the split test sources from their
actual test-only include ownership; preserve production reference rejection and
negative controls, including mixed test/production use. Verify and register actual
provider composition roots for the guided replay boundary and moved SQL editor
boundary without permitting provider construction in lower services generally.
Run guard positive/negative self-tests, real repository guard, file-size and
relevant workflow/foundation checks. This bounded prerequisite repair precedes the
next broad diagnostic; unrelated compiler changes remain separate V0 items.

V0.15 implementation: extend existing test-source ownership through literal
include chains, with production include/module reachability taking precedence.
Keep unknown include expressions scanned. Register the actual guided-replay
composition boundary and moved SQL editor root; begin scanning compiler_services
for illegal provider construction too. No Rust behavior, imports, assertions or
crate dependency rules change. The guard passes on repository sources; negative
controls retain rejection for shared production fragments (include, default module
and path-attribute module) and lower-service provider construction. An initial
classifier attempt still rejected the three test includes because it followed the
test-root declaration from lib.rs; its failed receipt is retained, and the corrected
classifier respects the guard's existing test-root classification. Final validation:
repository guard passes (4.192s); all guard self-tests pass (0.740s), including
mixed production ownership and both lower-service construction controls; file-size
guardrails pass (5.722s), workflow contracts pass (0.740s), and whitespace/local
links pass. No Rust source changes require a compiler rebuild. Initial independent review on `fce96a6c233af77b90b0f90a768ec8b9e23fa4d4`
reported SATISFIED, but its note about same-line and inline-module shared
production use identifies an in-scope classifier gap introduced here. A new
regression demonstrates the false exclusion; the remediation conservatively
retains every matching production module filename, same-line literal includes,
and all candidate fragments when a production include expression is unresolved.
These new controls must pass before a fresh scoped review; the initial verdict
alone does not approve the remediation. Five fragments are newly classified:
four codegen method-authority includes plus sysroot metadata test_dispatch.rs,
all reached exclusively from existing test roots in this repository.
Remediation validated in an owned isolated checkout while the broad diagnostic
kept its original candidate unchanged: real guard passes (8.522s), full guard
self-tests including new controls pass (1.710s), and file-size passes (5.320s).
These are diagnostic wall times under concurrent host activity, not performance
qualification. Workflow evidence is reused because its inputs are unchanged;
record-only local links and whitespace pass. The remediation review of
`aa5cec1ee996e2425323683e7286c4b226e7847b` was NOT SATISFIED: conditional
path attributes and inline-relative paths could still hide shared production
fragments. The correction matches path assignments including cfg_attr and raw
strings, conservatively retains all matching basenames, and recognizes raw module
identifiers. New negative controls cover each form. Final real guard passes
(5.061s), all guard regressions pass (1.095s), and file-size passes (3.063s).
Unresolved production include expressions deliberately retain all fragments for
scanning, so future unrecognized syntax fails closed. Final independent review SATISFIED, no blockers, on
`0fe59a9320967e958c6b4edabd328cf32de509c4`; merged in
[#4359](https://github.com/sifr-lang/sifr/pull/4359) at
`a36a01f926c3d59f147e138b982f691d3c50233c`. Final raw review is
external at `reviews/0fe59a9320967e958c6b4edabd328cf32de509c4/review.md`.
Evidence: `v015-path-*`, earlier `v015-final-*` and unchanged `v015-workflow`.
The existing file-wide composition-root exemption is retained; tightening it to
functions is separate future guard design work, not required for this repair.

**Next bounded item V0.3a — selected native toolchain launch context.**
The current broad diagnostic confirms generated-code preparation fails when nested
Cargo build scripts launch rustc without its selected toolchain loader environment.
A minimal independent probe reproduces this without the compiler: both version
queries pass; direct Cargo build fails with missing libLLVM; the same selected
Cargo launched through `rustup run` passes and its binary prints the expected
output. Separate build targets prevent cache reuse hiding the failure. Evidence:
`native-launch-probe-confirmed/results.json`; the first probe's incorrect expectation
that a direct version query should also fail is retained as a failed diagnostic.

Preserve the canonical rustup-selected execution context for version discovery and
Cargo children, keep exact selected Cargo/rustc paths and captured caller settings,
and bind the launcher into native build identity. Keep explicit paired executables
and non-rustup installations direct; no fallback, global environment mutation or
component removal. Required controls: subprocess-isolated launcher/selection tests,
missing-tool and explicit-pair behavior, configuration/identity regression tests,
a real nested build-script integration, focused sysroot tests, fmt/size checks,
and rerun the affected generated-code preparation. Python fixture/registry and
permission repairs remain separate V0.3/V0.6 work. Final complete gates stay open.

### V0.3a implementation and focused evidence

Rustup-resolved toolchains now retain the selected launcher for version discovery
and Cargo execution. Cargo/rustc paths remain exact; explicit paired tools and
non-rustup installations remain direct. The native identity domain is version 2
and includes launcher mode, path, content digest and selection. Existing native
artifacts therefore require requalification under the new identity; this is not
claimed as a cache speedup. Captured caller environment and Cargo configuration
validation remain unchanged.

Subprocess-isolated regression controls cover the nested rustc launch, launcher
byte invalidation, missing selection rejection, explicit pairs and direct tools.
The nested-launch regression fails before the repair (`v03a-nested-regression-before`,
9.150s, one selected test). All 50 sysroot tests pass afterward
(`v03a-sysroot-final`, 14.157s including incremental preparation; 1.35s test runtime).
The real `NativeToolchain` API also builds a fresh tiny Cargo project whose build
script invokes rustc with an initially narrow loader path; the generated program
prints `native-api-ok` (`v03a-api-integration`, 2.632s overall). This uses the
actual modified library, not a separately implemented launcher. Formatting passes
(`v03a-fmt`, 17.195s) and size guardrails pass (`v03a-file-size`, 4.174s).

Preserved diagnostics: the first 310.246s build selected zero tests because its
exact filter lacked the module prefix, so it is preparation only. The corrected
initial launcher regression fails in 2.113s. The first complete post-repair run
passes 49/50 in 202.796s including compilation; its new identity test incorrectly
expected libtest output to start at column zero. Correcting that output parser
produces the full 50/50 result above without further production changes. The
queued API probe correctly did not execute while that suite was failing.

V0.3a full affected preparation passes at the reviewed implementation candidate
`6cf4b5c178912cb0a4184a9469ba620ad49e8ea6`: **93/93** positive generated-code entries,
1,087.996s overall, including 95.896s compiler preparation. Reconciliation verifies
exact selected IDs, uniqueness, revision and all 186 manifest/lock hashes against
the final report. Evidence: `v03a-generated-preparation/{receipt.json,generated-report.json,reconciliation.json}`.
This is a cold native/materialization preparation result, not a full gate or a
warm speedup; it remains above the ten-minute objective by itself. It does not
qualify generated Rust execution, which retains its separate assertions.

Independent review is SATISFIED with no blockers on that same implementation
candidate. Raw review: `reviews/6cf4b5c178912cb0a4184a9469ba620ad49e8ea6/review.md`,
external to the reviewed tree. Merged in
[#4360](https://github.com/sifr-lang/sifr/pull/4360) as
`8858bde63f80b3b40860da3b1b7a48b31a91caaa`. The 50 libtest rows include
49 assertion-bearing tests and one subprocess-dispatch helper that returns early
in the parent run; its real child assertions are exercised by the parent tests.
Review suggestions retained for later identity/timing work: measure launcher-file
hashing cost; consider additional isolated selection/path identity controls.
Redundant consistent RUSTUP_TOOLCHAIN propagation needs no change.

The original broad merge-correctness diagnostic against frozen
`fce96a6c233af77b90b0f90a768ec8b9e23fa4d4` stopped at **1,586.715s** on the disk
reserve, before the 30-minute cap or main test suites. Its generated preparation
failed in 34.950s at the nested toolchain boundary. Its advanced native preparation
was still incomplete; it is not a measured greater-than-30-minute category or a
full-gate result. Cleanup signalled remaining process-group members; a subsequent
process inspection found no compiler/test children. Source identity stayed fixed.
Evidence: `v015-merge-discovery`.

Capacity recovery preserved a checksum-verified copy of this session's inactive
native Cargo target on the larger volume, under its original family lease, before
releasing the old storage. Lease inode, owner metadata and archive receipt remain
preserved (`native-target-archive.json`); approximately 4.1 GiB was recovered.
Subsequent native work uses an explicit private cache on that volume. The new cache
state is recorded, and no warm-comparison claim is made across this relocation.
The compiler dependency target remains retained.

PR #4360's remote create-PR workflow failed before compiler tests because the
benchmark-inclusive default lacked SIFR_PERFORMANCE_REFERENCE. This known
benchmark admission prerequisite is outside this local-correctness item; raw job
log is retained as `ci-4360-create-pr.log`. No CI pass is claimed. The approved
focused intermediate policy governs this merge; complete integrated correctness
gates and V5 still remain mandatory.

### Next bounded item V0.5 — maintained Cargo inventory admission

Reconcile tracked first-party manifest/lock inventory and exact audited dependency
owner declarations with current maintained inputs. Preserve exclusions for vendor,
gitlinks and third-party repositories, exact declaration identity, checksum
validation and negative mutation controls. Existing registry release evidence is
historical authority, not permission to upgrade dependencies or claim a fresh
latest-version audit. Inspect new declarations and lock identities before updating
the inventory; genuinely new package/release evidence must be independently
verified or assigned to its owner. No dependency-version or Cargo.lock changes
are authorized by this bounded item.

Start with the failing inventory/direct-declaration tests, then run all affected
stdlib-manifest dependency contract tests together to expose related drift in one
pass. Required acceptance: exact tracked inventory and declaration comparisons,
all affected dependency/version/lock contracts and their negative controls, full
`sifr_stdlib_manifest` tests, fmt/file-size checks and scoped independent review.
The full phase gates remain deferred under the approved policy.

### V0.5 diagnostic and ownership boundary — policy approved

Baseline `8858bde63f80b3b40860da3b1b7a48b31a91caaa`: the direct dependency target
runs nine tests, six pass and three fail at tracked inventory admission (6.578s).
Independent Git-index/TOML inspection confirms **153 manifests / 35 locks** versus
133 / 26 recorded, with no removals, and 211 direct declarations versus 201 recorded.
Two additional registry package names, libfuzzer-sys and windows-sys, need audited
source evidence. New lock identities also require official checksum verification;
no new release metadata has been blessed.

A diagnostic update of the exact tracked inventory alone lets the full affected
crate progress. Its first build failed because the isolated checkout had not yet
materialized two required demo gitlinks, not because their source files were
missing from the pinned repositories. Independent owned checkouts of the exact
tracked HTTP (`76fea17316d37aa536f7bd618660613cbef793d2`) and application
(`af14ce5adbcc1cc6d775ba8b1800c50aeb0e9afa`) revisions repair that preparation.
The complete `cargo test --locked --offline -p sifr_stdlib_manifest --no-fail-fast`
then executes **102 tests: 97 pass, five fail**, in 17.212s including preparation.
No tests are filtered/ignored. The compile-failed attempt remains preserved.

The five failures expose:
- stale exact num-bigint and rusqlite owner/lock edge inventories;
- missing audited checksum rows for newly admitted lock identities;
- stale manifest/declaration counts and registry owner rows;
- a real version-policy mismatch: the fuzz graph resolves syn 3.0.6 while the
  uniform first-party edge contract requires 3.0.5.

Further declaration inspection also exposes exact older anyhow/serde_json pins in
`verification/tools/maintainability_builtin_input`. Its
[architecture owner](ad-hoc-architecture-correctness-current-main.md) explicitly
requires retaining the locked dependency graph and forbids dependency/API upgrades.
The user approved **“Audit pinned verification graphs (Recommended)”** on
2026-10-20, before implementation. Retain product-version checks; admit only
explicitly named verification graphs against approved manifest/lock identities
and independently verified official registry checksums. Reject drift. This changes
the uniform-release policy only for those pinned graphs; it does not authorize
any dependency upgrade, lockfile change, blanket verification-directory exclusion,
or weakening of the product contracts. Preserve the separately owned tooling
pins. Add mutation controls for graph admission, input identity, declarations,
source/checksum identity and product-policy boundaries. The full affected crate
and independent review remain required before this bounded item merges.

Evidence: `v05-inventory-before`, `v05-inventory-inspection.json`,
`v05-all-contracts-inventory-only` (preparation failure),
`v05-submodule-preparation.json`, `v05-all-contracts-prepared`,
`v05-reconciliation.json`, and `v05-inventory-only.patch`. The inventory-only
experiment was restored from its preserved original bytes after all tests ended;
its exact patch remains externally reproducible against the recorded base.
V0.5 is not complete, and the phase has no final-gate or timing acceptance.


### V0.5 implementation and validation — reviewed

The approved audit covers **153 manifests, 35 locks, 211 direct declarations and
44 exact maintained owner/lock dependency edges**. The two named verification
graphs authenticate all **24 manifest/lock inputs** by SHA-256 before receiving
an exception to uniform product release versions. Their **371 registry release
identities** are checked against independently retrieved official checksums;
the full lock identity also retains their exact Git revisions. Every registry
declaration still participates in the exact path/table/alias/package/requirement
inventory. Product declaration and resolved-edge version checks remain unchanged.
No Cargo.toml, Cargo.lock, dependency version or verification tool pin changed.

Fresh official-index retrieval verified all **471 required versions across 379
packages** with zero missing versions or checksum mismatches. Existing September
release observations retain their original timestamps; additional locked release
rows carry the new response digest and observation timestamp. This is checksum
reconciliation, not a claim that all historic release pins remain today's latest.
Raw official responses and retrieval receipts are preserved in
`v05-registry-evidence/` outside the reviewed tree.

Focused audit: **15/15 tests**, 6.619s including preparation. The first full batch
exposed one remaining stale count (seven rusqlite owners, including the CLI,
versus six recorded); its failed evidence is retained. Final full crate batch:
**108/108 tests**, no failures/ignored/filtered tests, **11.128s** including
preparation (`v05-approved-all-contracts-final`). This retains all 102 original
tests and adds six pinned-graph mutation/coverage controls. Format passed (19.186s),
file-size guard passed (7.671s), and source dependency-direction guard passed
(5.096s). Concurrent guard timings are diagnostic durations, not isolated speedup
measurements. The initial Python guards used the system 3.14.4 interpreter;
a final toolchain audit caught that mismatch. Both guards were rerun successfully
through the locked verification environment with asserted **Python 3.14.7**, in
6.591s combined (`v05-pinned-python-guards`). The pinned rerun is the acceptance
evidence; earlier runs remain diagnostic evidence. Six added controls reject missing/changed inputs, scope expansion,
unapproved registry rows/checksums/sources, and product-version relaxation.
Independent review: **SATISFIED**, no blockers, for implementation
`f4981e547d33514744567ea31f80bcc5846e5df2` against
`8858bde63f80b3b40860da3b1b7a48b31a91caaa`. Raw review is preserved outside Git
at `reviews/f4981e547d33514744567ea31f80bcc5846e5df2/review.md`.
[PR #4361](https://github.com/sifr-lang/sifr/pull/4361) merged as
`ad1e26aac38787b669414e8572dcf9146433ced6`; final phase
gates/timing remain open. CI's benchmark-inclusive create-PR job stops before tests
because `SIFR_PERFORMANCE_REFERENCE` is unset (`ci-4361-create-pr.log`); this is
not a CI pass. The approved intermediate local policy remains applicable.

Review follow-ups: the dependency-release owner should separately refresh historical
September release pins; this item only reconciles official checksum evidence.
Cross-platform qualification should consider raw-byte pinned-input hashes under
Windows line-ending conversion. Direct exemption-routing and Git-revision-specific
mutations are optional additional controls; current exact product-path exclusion,
product helper controls and every-input mutation tests cover the required policy.


### V0.2 SQL qualification — validated and reviewed

Base: `ad1e26aac38787b669414e8572dcf9146433ced6`. Exact pinned PostgreSQL parser
repositories and WASI-Virt were materialized in the owned checkout. Initial
lightweight batch ran **33 commands: 23 passed, 10 failed**, in 8.094s. Four
commands stopped on SQL component source provenance; six inspected the former
driver implementation instead of the current compiler-services owner.

Three source qualification guards now follow the canonical SQL service and also
require the driver's public delegation. Existing 25 payload mutation cases are
retained, with **15 additional source mutation controls** covering the delegation,
provider resolution, query registry and schema requirement authority. After this
repair the batch runs **29/33 successfully** (14.168s); the same four producer
provenance failures remain until artifact regeneration. These failed attempts
remain external evidence, not passing qualification.

The official WASI SDK 34 archive digest, size and every extracted file/link were
verified (26.091s including download/extraction). WASI-Virt's first offline build
stopped on a missing cached locked dependency (0.640s); locked dependency fetch
completed in 3.141s and the actual offline producer build completed in 69.680s.
No dependency manifest or lock was changed.

Canonical artifact producers, using Rust 1.98.1 and Python 3.14.7, regenerated all
**ten SQL component artifacts** and their actual producer receipts:

| Producer | Supported variants | Elapsed including producer checks |
|---|---|---:|
| MySQL | 8.4, 9.7, 26.7 | 52.177s |
| SQLite | 3.53.4 | 51.580s |
| PostgreSQL | 13 through 18 | 309.356s |

These are artifact-generation/preparation measurements, not complete gate timings.
The source was frozen during each producer run; only that producer's declared
artifact/receipt outputs could change. The source hashes were not patched to
bless existing binaries. At this producer checkpoint, runtime assertions, final lightweight qualification,
guards, independent review and merge remained required; completed results follow below.
Evidence: `v02-lightweight-baseline[-cases]`, `v02-source-repairs-batch`, `v02-source-repairs-cases`,
`v02-submodule-preparation.json`, `v02-sdk-prepare`, `v02-sdk-preparation.json`,
`v02-virtualizer-build`, `v02-virtualizer-fetch`, `v02-virtualizer-build-prepared`,
and `v02-{mysql,sqlite,postgresql}-producer`.


V0.2 final validation on implementation
`3d01c1638ed672ef56b38c8405457d37a6ba1dd1`:

| Check | Executed coverage | Elapsed including preparation |
|---|---|---:|
| Lightweight SQL qualification / mutations | 33/33 commands pass | 9.647s |
| Component host and sandbox | 18/18 tests pass | 4.382s |
| MySQL compiler and component execution | 9/9 tests, all three component variants | 19.383s |
| SQLite compiler and component execution | 14/14 tests, the supported component | 9.376s |
| PostgreSQL component execution and registrations | 2/2 tests, all six majors | 190.983s |
| File-size, source direction and area contracts | All pass, Python 3.14.7 | 8.157s |

All **43 native tests** executed with no ignored/filtered tests. Cargo checked the
candidate at the retained native build location; the suites read and execute the
new artifact bytes there. All ten SQL variants passed actual host execution,
including the existing diagnostic/schema assertions. The native checks ran
sequentially with their own cap; lightweight/guard checks overlapped preparation,
so their durations must not be summed into a gate-wall-time claim. No full phase
gate or V5 timing scenario is qualified by this bounded validation.

Evidence: `v02-lightweight-final[-cases]`, `v02-component-host-tests`,
`v02-{mysql,sqlite,postgresql}-runtime-tests`, `v02-runtime-batch.json`, and
`v02-policy-guards`. [PR #4362](https://github.com/sifr-lang/sifr/pull/4362)
received **SATISFIED**, no blockers, on exact candidate
`e87c97ddf56a733c0d80b574d96213c733282030` against
`ad1e26aac38787b669414e8572dcf9146433ced6`. Raw review:
`reviews/e87c97ddf56a733c0d80b574d96213c733282030/review.md`, outside Git.
Subsequent phase-record edits do not change the validated implementation inputs.

Producer/reproducibility follow-up (existing on base, separate from V0.2): committed
components contain registry-source and C-source build paths. The producer remaps
only its repository Rust paths, and validation checks digest shape for some tool
identities rather than proving a hermetic build. The producer owner should add
registry/C-source path remapping and explicit output-affecting input qualification,
then rebuild and compare bytes under independently located roots. Keep this as an
explicit V5/release reproducibility limitation until that protocol is executed;
V0.2 does not claim cross-machine byte identity. The optional stronger structural
driver-delegation guard is separate; the existing text guard's required scope is
preserved and mutation-tested.

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

Current evidence: Bounded repairs and their acceptance are recorded below. V0.18
filesystem ledger reconciliation and final integrated completeness remain open.

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

Current evidence: The SQL full-protocol profile trial and sanitizer retention tests
are recorded; V1b1 shared artifact data is accepted. Semantic/native separation
and complete build-path feasibility remain open.

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

Current evidence: V2a bounded metadata validation facts are accepted below.
Component Engine/component reuse and durable-history optimization remain open.

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
policy above now governs intermediate implementation candidates. On V4a merge,
the explicit final LOCAL-GATE commands become:

```sh
scripts/run_all_tests.sh --profile create-pr --correctness-only
scripts/run_all_tests.sh --profile merge --correctness-only
```

These execute the complete selected correctness scope. The ordinary full profiles
remain available and unchanged for other contracts; this phase does not require a
duplicate benchmark-containing gate to qualify its explicitly benchmark-excluded
goal. V5 still requires complete passing evidence across every listed scenario,
including preparation, timing and cleanup. CI mirroring and foreign-platform or
benchmark optimization remain later work.

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
| Phase registration | Merged in [#4356](https://github.com/sifr-lang/sifr/pull/4356) | Reviewed plan candidate `c7e80fb6cb310fb1d4ff64c2e0179a2d148fb467`; documentation checks pass; independent review SATISFIED; subsequent edits only clarify documentation and record this handoff |
| V0 executable coverage | In progress | Bounded repairs accepted below, most recently V0.17 / #4385; V0.18 and final completeness pending |
| V1 mandatory build path | In progress | V1a experiment / #4383 recorded; V1b1 / #4384 accepted; semantic/native split and feasibility pending |
| V2 repeated metadata/component work | In progress | V2a / #4386 accepted below; component/history work pending |
| V3 compatible build reuse | Pending | Diagnostic evidence only |
| V4 runner/compilation/policy integration | In progress | V4a correctness-only selection accepted; scheduler, budgeting and catalog work pending |
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
closure loop. At that planning checkpoint, implementation remained pending and no full-gate pass was claimed.

After all implementation items merge, close only when V5 passes. Record remaining
out-of-scope work separately; no unresolved in-scope coverage or timing scenario
may be described as completed. A missed target triggers further root-cause work,
not a scope reduction or a promised speedup based on overlapping timings.

V0.2 merged as #4362 at `260110d9bc66f0812738650d63a80aab55dd3d12`.

**V0.3b entry scope (now merged):** Python runtime
registry and LSP input-authority admission. Confirm current failures before edits.
Arrow and DLPack runners retain old exact test inventories; review each added or
renamed ownership assertion against its originating change before reconciling
names, preserving missing/extra/duplicate rejection. The LSP selected-metadata
fingerprint test also fails: inspect both its Cargo fixture validity and the
production ancestor-input scan. Preserve selected environment invalidation and
owning-root isolation; do not turn its invariance assertion into a pass by simply
loosening the expectation. Run the affected runtime suites with pinned Python,
LSP fingerprint/authoring and external-input generation controls, affected runner
self-tests and standard guards. Other V0 items and full phase acceptance remain
open; no final gate or V5 timing result is claimed.

### V0.3b Python registries and selected LSP inputs

Implementation `88faa732b` repairs two independent admission defects. The Arrow
runner now requires all 15 current tests, including the H02 ABI/capsule and stream
release controls. The DLPack runner requires all 21 current tests. Its renamed
stopped-runtime test deliberately checks structural cleanup exactly once, per
`e1bff98c99e1286dfe4276f25f4fed6e118066f4`; restoring the obsolete zero-release
expectation would contradict that contract. The two H02 DLPack ABI/ownership tests
are retained. Exact runtime admission still rejects missing/duplicate tests and
now explicitly exercises extra/substituted-test rejection too. The unchanged
five-test buffer registry exercises the shared runner.

The LSP fingerprint had hashed every ancestor pyproject/uv.lock even with explicit
manifest overrides. The original selected-metadata test also lacked a Cargo target.
A fixture-only control still failed its original invariance assertion. The repair
retains Cargo/config/toolchain ancestry and recomputes canonical Python selection
on every observation, hashing its selected metadata/environment inputs. It adds
cache-retention and project-discovery controls, including nearer project creation,
incomplete projects, deletion, fallback to an ancestor and reappearance. The
existing per-input environment invalidation fixture now explicitly selects its
metadata, so missing-to-present transitions remain required for selected files.
No assertion is relaxed to permit unselected-input cache eviction.

Evidence outside Git under `local-gate-implementation-20261020/`:

- `v03b-lsp-fingerprint-before`: original exact regression failed, 53.296s.
- `v03b-lsp-valid-fixture-before`: valid Cargo fixture alone still failed, 17.785s.
- `v03b-lsp-library`: all 114 library tests passed, no ignored/filtered tests,
  234.534s including 17.30s initial compilation and a nested CLI build. The libtest
  interval was 214.43s; this includes the CLI build and is not pure assertion time.
- `v03b-policy-guards`: file-size and all area manifests passed; this attempt
  failed because the external command misspelled the source-direction script.
  Failure evidence is preserved. `v03b-source-guard-and-format` used the correct
  source-direction script and passed it plus `cargo fmt --check`, 12.146s.

Validation used pinned Python 3.14.7 and Rust 1.98.1, retained owned Cargo targets,
two Cargo jobs and explicit disk/memory reserves. The focused area run overlaps
the end of the full LSP run; neither duration is isolated performance evidence.
These are bounded correctness/admission repairs, not proof of the final gate's
600-second objective. Full integrated gates and V5 remain open.

`v03b-python-focused-area` passed all four selected variants in 402.193s:
LSP declaration authoring 397.250s (all four canonical commands), buffer release
5/5 in 1.700s, Arrow 15/15 in 1.241s, and DLPack 21/21 in 0.865s. These are runner
case times including their preparation. The large authoring time includes nested
CLI/Cargo preparation, not only LSP assertions. The explicitly filtered run used
`--allow-partial-certification`; the captured certification status is
`not-selected`, and this run cannot satisfy final Python-area/phase qualification. Exact runtime inventories, real
PyArrow transfer, missing/duplicate/extra/substituted admission controls all ran.
The runtime Cargo integration target reports zero selected tests separately;
coverage credit is only the 41 exact named module tests, not those filtered targets.
Raw reports, receipts, stdout/stderr and resource evidence are retained locally
and remotely. PR [#4363](https://github.com/sifr-lang/sifr/pull/4363) is reviewed;
merge is pending.


V0.3b review on `a0bb8008c56df8313ec1a8d7b65d6e27f2ca83f4`:
**SATISFIED**, no blocking findings. Raw review is preserved outside Git as
`reviews/a0bb8008c56df8313ec1a8d7b65d6e27f2ca83f4/review.md` in the implementation
evidence root. Later edits only record review/status and correct the certification
label from “partial” to the actual report's `not-selected`; no implementation or
validation input changes. Documentation diff and local links checked.

Separate LSP freshness follow-up: ancestor events outside a package root are not
routed back to that descendant root by `watcher_events.rs`, and watcher-covered
documents skip polling. On-demand observation sees the drift, but event-driven
refresh can wait until a request. This also affects existing ancestor Cargo/config/
toolchain inputs; it is not introduced by V0.3b. Owner: LSP external-input watcher
routing, outside this bounded admission repair. A manifest-failure/repair selection
test is an optional review suggestion; root manifest changes are already hashed
and existing invalid-manifest repair controls pass.

**Next bounded item V0.6a — interop fixture authority.** First reproduce current
Rust interop contract failures with the retained driver test target. Repair the
owned fixture/input boundary for the recorded missing Cargo manifests and bridge
source directory. Preserve production rejection of absent real Cargo authority;
no fallback, skipped probes or missing-input success. Keep pure contract assertions
and real Cargo probe/runtime integration explicit and require both. Run the complete
affected contract/probe groups, missing-manifest/source negative controls and the
full driver library diagnostic batch to identify downstream failures without
repeated preparation. Keep ownership/pruning and emitted-output/key assertions in
separate V0.6 follow-ups; other V0 semantic defects retain their named owners.
Required guards: formatting, source direction, file size and any affected area
contracts. Apply the approved focused intermediate review policy, then continue
through the remaining phase items. Final integrated gates/V5 remain open.


V0.3b merged as #4363 at `817409bcea035fa8d5f50ecbde1a3d813c2012b8`.

V0.6a subdivision after source/history inspection:

- **V0.6a1, validated:** expose the existing contract-planning stage to its unit
  tests, separately from native qualification. Commit
  `1cc3aadc4e66786d45ef5635def04bf2b32bb9a3` correctly removed a normal-mode
  missing-manifest success fallback. Sixteen positive contract-only tests had
  relied on that fallback without ever running real probes. Factor the same
  ordered production planning code into a method; production must still run
  planning, native probes, and input/cache-authority checks before returning a
  generated project. A test-only planning adapter may return the probe plan,
  never a qualified generated project or cache identity. Preserve every contract
  assertion and all real probe tests. Add a control proving planning success does
  not satisfy absent-manifest qualification, and exercise the planning/full-probe
  distinction with existing real compatible/incompatible signature fixtures.
  Run all affected contract groups and real probe controls. Record unrelated
  fixture/cache/ownership failures from the full driver diagnostic batch.
- **V0.6a2:** repair real inputs for tests that claim qualified metadata, cache
  authority, bridge aliases or installed sysroot resolution, including the missing
  bridge directory, abort-profile panic test and both native-links tests. These must execute real probes with owned, valid fixtures;
  no synthetic-success path. Preserve relevant filesystem failure controls.
- **V0.6b/c:** ownership/pruning and emitted-output/cache-key assertions remain
  separately bounded after current failures are reproduced.

This is a test-layer distinction, not permission to skip production/native gates.
Planning cannot create runnable or cache-qualified success. Existing real probe
and native/runtime coverage remains required in the integrated gate. No removed
assertions, fallback compiler, dependency changes or timing claim are authorized
by the subdivision. Standard Rust/area guards and scoped independent review apply.


### V0.6a1 contract planning closure

Implementation/review candidate `5b80358887226b5c6f25036c17e2ed214b102cab`,
base `817409bcea035fa8d5f50ecbde1a3d813c2012b8`, PR
[#4364](https://github.com/sifr-lang/sifr/pull/4364). Named checks pass and scoped
independent review is SATISFIED, no blockers. The production planning and remaining
qualification bodies are byte-identical to the base; the test adapter returns
only a probe plan. Sixteen contract-only assertions retain their contents and
real compatible/incompatible native signature controls still execute qualification.

Evidence below lives outside Git under
`continuation-evidence/local-gate-implementation-20261020/`:

- `v06a1-planning-admission`: new missing-manifest rejection control passes,
  72.243 seconds including preparation.
- `v06a1-policy-guards`: formatting, file size, source direction and all area
  manifests pass, 21.152 seconds. `v06a1-fixture-matrix`: canonical Rust interop
  matrix and self-tests pass, 13.147 seconds; pinned verification Python 3.14.7.
- `v06a1-driver-library`: completed full diagnostic, **711 passed, 23 failed,
  81 ignored, zero filtered**, 829.49 libtest / 832.687 outer seconds. No timeout,
  leftover workers or unexpected tracked changes. The suite is FAILED, not a
  passing gate. `v06a1-final-inventory.json` reconciles every remaining failure
  with the historical inventory: all sixteen targeted failures now pass, the
  new admission control passes, and no new failure ID appears. The 81 ignored
  native/corpus cases remain explicit final-gate obligations.
- `v06a-driver-library-before` remains INCOMPLETE: cancelled at 865.458 seconds
  after discovering rotational temporary storage distorted durable-publication
  testing. Its completed affected prefix remains valid diagnostic evidence;
  it is neither a gate pass nor a matched performance baseline.
- Candidate scratch storage was explicitly moved to a session-owned SSD directory.
  `v06a1-history-observation.json` observed the actual 4,097-publication history
  test at 247.829 seconds with 0.25-second stdout polling. This is approximate
  external observation, not an isolated benchmark or claimed code speedup.
- `v06a1-production-order-audit.json` records unchanged body hashes. Raw review:
  `reviews/5b80358887226b5c6f25036c17e2ed214b102cab/review.md`.
  The final diagnostic receipt was reconciled after review, before merge.

Remaining failures: sixteen real-input admission cases belong to V0.6a2,
including the abort-profile panic test and both native-links tests. Three
workspace ownership plus two housekeeping failures belong to V0.6b. The
sysroot Cargo cache-key assertion and attached-API emitted-output assertion
belong to V0.6c. Review also notes that the test adapter assumes declarations
and a context; check that assumption if future tests reuse it. Final integrated
create-PR/merge gates and V5 remain open; this intermediate repair makes no
whole-gate runtime claim.

V0.6a1 merged as #4364 at `a425c6b99658925c32fa2869da882e1f9f687173`.
V0.6a2 now supplies owned real Cargo/bridge/sysroot inputs for qualification tests.
It retains same-fixture cache comparisons and explicit absent-manifest/source
rejection controls. The existing sysroot Cargo-key shape assertion remains a
separate V0.6c failure; no production fallback or qualification bypass is added.


### V0.6a2 real qualification fixture evidence

Implementation candidate `8aa639c8ec6fe16b76f4bd52bd75f88330c238e3`, base
`a425c6b99658925c32fa2869da882e1f9f687173`, PR
[#4365](https://github.com/sifr-lang/sifr/pull/4365). The change supplies owned
Cargo projects and actual declared bridge files for sixteen qualification tests,
shares the existing miniature sysroot fixture, and keeps trust/cache comparisons
on the same fixture paths. The abort-profile test now has its real bridge source.
The mixed user/sysroot fixture selects one runtime/vendor authority and copies
its real declared native path-patch input from the repository. It does not replace
that patch with a stub. Production qualification is unchanged.

These tests exercise an already-resolved sysroot context, constructed explicitly
as before. They are not installed-distribution authentication or SQL runtime
qualification. Those independent integrated gates remain mandatory and unchanged.

Evidence under `continuation-evidence/local-gate-implementation-20261020/`:

- `v06a2-qualified-interop`: complete affected Rust interop/sysroot groups on the
  candidate, **226 pass, one pre-existing failure, zero ignored, 589 filtered**;
  63.99 libtest / 67.219 outer seconds. This is a FAILED diagnostic, not a green
  suite. `v06a2-final-comparison.json` reconciles all sixteen targeted failures
  to passing rows, one added passing source-rejection control, no missing IDs and
  no status regressions. The remaining plaintext sysroot Cargo-key assertion
  belongs to V0.6c. Actual native signature and absent-manifest controls pass.
- `v06a2-mixed-sysroot`: exact mixed-context native qualification and new runtime
  selection assertion pass, 60.772 seconds including 57.32 seconds preparation.
- `v06a2-reviewed-guards`: exact candidate formatting, source direction, file
  size and all area-manifest checks pass, 15.331 seconds, pinned Python 3.14.7.
- `v06a2-fixture-matrix`: canonical matrix and self-tests pass, 7.676 seconds.
  Later changes only remove unused imports and complete the mixed fixture; matrix
  inputs and enrollment are unchanged. `v06a2-test-name-audit.json` records no
  removed tests and one added missing-source rejection test.
- `v06a2-interop-groups` and `v06a2-interop-final` remain failed intermediate
  attempts. They exposed the mixed fixture's inconsistent runtime/vendor inputs
  and absent native path patch; neither is relabeled as passing. The final
  candidate executes both user and private sysroot probes successfully.

The broad prior driver baseline remains `v06a1-driver-library`: 711 pass,
23 fail, 81 ignored. Do not infer a new whole-driver pass or timing from this
focused repair. Workspace ownership/pruning and emitted-output/key assertions
remain separately owned V0.6b/c work. Complete phase gates and V5 remain open.

Independent review on the implementation candidate is **SATISFIED**, no blockers;
raw evidence: `reviews/8aa639c8ec6fe16b76f4bd52bd75f88330c238e3/review.md`.
Review follow-ups are assigned to V3 fixture ownership/reuse: replace the existing
clock-based TempSysroot directory naming with exclusive directory allocation
before relying on parallel fixture execution; measure the real native patch copy
before optimizing it. Keep actual source authority and equivalent assertions.
Preparation is included in the recorded elapsed times and remains part of V5.

V0.6a2 merged as #4365 at `f8ae3ec9a0ba9cbf71eb8b0f73ae29a7142a091e`.

### V0.6b bounded scope: cache ownership fixture permissions

Five current failures are reproduced on unchanged merged source under two umasks:
`v06b-baseline-umask-002` has zero pass/five fail; `v06b-baseline-umask-022`
has five pass/zero fail. The permissive attempt also left a worker in its owned
process group; the diagnostic harness terminated it and preserved that fact.
Retained failed native roots are mode 0775. Production correctly rejects owned
cache records writable by others. Fixture `fs::write` creates 0664 payloads under
0002; the production semantic writer creates private files independently of umask.

Repair only the three N06 root/metadata fixtures and two semantic-housekeeping
fixture inputs. Use exclusive private roots and production private-file creation,
keep deliberate unsafe/symlink/foreign-owner cases, lock inode and live-descendant
assertions, and all real crash/recovery boundaries. Force permissive umask in
dedicated test subprocesses without changing the parallel parent harness. Run
both umask controls, complete workspace/housekeeping groups, formatting/source/
size/area guards and scoped review. Production ownership checks remain unchanged.
The two cache-key/emitted-output failures remain separately owned V0.6c work;
complete integrated gates and V5 remain mandatory.


### V0.6b ownership fixture validation

Implementation candidate `fac0dc32ad1bf0d6bf9361f42b019963970700b4`, base
`f8ae3ec9a0ba9cbf71eb8b0f73ae29a7142a091e`, PR
[#4366](https://github.com/sifr-lang/sifr/pull/4366). Production cache policy is
unchanged. Three native fixture roots now use exclusive allocation with explicit
0700 permissions; eligible fixture records use the production private-file
writer. The two semantic-housekeeping cases supply private stage/scratch files
and an explicitly owned workspace. Dedicated subprocesses enforce umask 0002
without changing the parent harness. Housekeeping subprocesses must both succeed
and report one test actually executed, so empty selection is rejected.

Evidence under `continuation-evidence/local-gate-implementation-20261020/`:

- `v06b-baseline-umask-002`: all five failures reproduced, 4.189 seconds including
  preparation and cleanup; leftover worker signalled. `v06b-baseline-umask-022`:
  same source and five tests pass, 3.186 seconds. These diagnose permissions;
  failed execution is not performance evidence for a passing suite.
- `v06b-repaired-umask-*` remain failed intermediate attempts: pinned tempfile
  3.27.0 follows ambient directory permissions unless explicitly configured, and
  workspace-hint reclamation also rejects a group-writable workspace. Both
  assumptions were corrected explicitly. `v06b-private-umask-*` then pass all
  five tests under both masks; the final candidate additionally checks actual
  subprocess test admission.
- `v06b-final-groups-umask-002` and `-022`: complete workspace and housekeeping
  groups on the exact candidate, **12 pass, zero fail/ignored, 804 filtered** in
  each; 16.688 seconds including 11.24 seconds preparation, then 6.687 seconds
  retained. Counts include the internal `storage_child` entry; its real cases
  are exercised by the parent tests. No timeouts, leftover groups or tracked
  changes. Real crash/recovery, descendant leases, lock-inode stability, foreign
  scope, unsafe 0666 files, symlinks and pruning-count assertions remain intact.
- `v06b-final-guards`: formatting, file-size, source-direction and all area
  manifests pass, 13.141 seconds, pinned verification Python 3.14.7.

These are correctness repairs, not a measured whole-gate speedup. The two
recorded V0.6c assertion failures and complete integrated gates/V5 remain open.

Independent review is **SATISFIED**, no blockers; raw review:
`reviews/fac0dc32ad1bf0d6bf9361f42b019963970700b4/review.md`.
Follow-ups are assigned to V0.14/V3 child lifecycle and admission: extend the
pre-existing workspace child helper's exit-only check to prove actual test
execution, and retain bounded subprocess cancellation/cleanup when composing
these tests into the final gate. The new housekeeping child already rejects zero
selection; it currently relies on the enclosing runner for a timeout. Neither
observation waives final coverage, failure cleanup or deadline requirements.


V0.6b merged as #4366 at `e69e64d6bafb3bbe59963355c54b0491cf370c74`.

### V0.6c bounded scope: semantic assertion authority

The two remaining recorded driver failures are stale representation assumptions.
The canonical Cargo fragment now encodes `RustInteropCargoInputs` with the v2
identity encoder; its package ID is no longer plaintext in the fragment. Assert
the qualified structured package ID and prove mutating only that ID changes the
encoded key. The attached API call is emitted as `describe:: < Model >`, while
its test expects `describe::< Model >`. Parse the real generated Rust and assert
the concrete owner and exact generic bound lists structurally; do not normalize
away required type/trait differences or weaken the semantic contracts.

Keep production unchanged. Run affected sysroot and attached-API groups, guards,
and a complete driver library diagnostic after these sequential driver repairs.
Preserve native/ignored-case enrollment for the final gate. Require independent
review and keep all later V0 owners and integrated phase gates/V5 open.


### V0.6c validation and complete driver reconciliation

Candidate `f9ecdb1f66f410ecdc49883011710321cdeeb760`, base
`e69e64d6bafb3bbe59963355c54b0491cf370c74`, PR
[#4367](https://github.com/sifr-lang/sifr/pull/4367). Production is unchanged.
The sysroot test reads the qualified package ID and proves its contribution to
encoded cache identity. Attached-API checks parse real generated Rust and retain
public visibility, ordered exact bounds and the concrete call owner.

Evidence under `continuation-evidence/local-gate-implementation-20261020/`:

- `v06c-two-regressions` preserves the initial compile failure from using a
  pre-Syn3 trait-bound API. The helper was corrected against pinned Syn 3.0.5;
  that failed attempt is not a runtime pass.
- `v06c-regressions-pinned-syn`: both exact regressions pass, 59.719 seconds
  including 54.63 seconds preparation. `v06c-affected-groups`: all 16 sysroot and
  attached-API tests pass, zero ignored, 12.691 seconds including preparation.
- `v06c-final-guards`: formatting, file-size, source direction and area contracts
  pass, 13.838 seconds, using pinned verification Python 3.14.7.
- `v06c-full-driver`: **735 passed, zero failed, 81 ignored, zero filtered**;
  793.35 seconds libtest and **796.154 seconds** including preparation/cleanup.
  No timeout, leftover process group or tracked source change. The 81 ignored
  integration/corpus cases remain mandatory in their explicit final selections.
- `v06c-final-inventory.json`: all 23 failures remaining after V0.6a1 now pass;
  no original driver ID is removed or newly ignored. One missing-bridge-source
  negative control was added by V0.6a2. Interleaved subprocess stdout is accounted
  for explicitly: five parent outcomes are inferred from the complete successful
  libtest result; the child-only `storage_case` is not counted as a driver test.
- `v06c-history-observation.json`: the unchanged 4,097-publication durability
  test took about 253.085 seconds by external stdout observation (0.25-second
  polling), versus the earlier 247.829 seconds on the same SSD scratch. This is
  diagnostic attribution, not an isolated benchmark or a whole-gate speedup.

Independent review is **SATISFIED**, no blockers, keyed to the exact candidate:
`reviews/f9ecdb1f66f410ecdc49883011710321cdeeb760/review.md`. Follow-ups assigned to
emitted-Rust attached-API test ownership: consider exact fixture package-ID
comparison, explicit empty `where`-clause checking, and generated-source context
in assertion failures. These suggestions do not weaken existing obligations.
Record-only updates do not change validated inputs. Full integrated gates,
native semantic repairs and V5 timing acceptance remain open.


V0.6c merged as #4367 at `4790068c0899880f314ab34a0e9f82221ee1ee1e`.

### V0.7 bounded scope: typed nominal accessors

Current-source inspection refines the original diagnosis: lowering deliberately
classifies newtype `value()` and enum `name()`/`value()` as builtin intrinsics.
That authority is correct, but the typed codegen registry has no corresponding
receiver arms. Reproduce the exact ignored native collision case before repair;
retain nominal identity and strict authority checks. Add only supported zero-arg
accessor lowering, using existing generated accessor methods and structured
receiver types. Split registry tests by responsibility if necessary to preserve
the 900-line source limit, retaining test IDs.

The collision fixture currently builds but never executes its binary. Execute
that binary and require success so its unchanged runtime assertions actually run.
Validation: focused typed accessor/codegen authority controls; existing lowering
method-authority tests; complete codegen unit tests; the exact native collision
case; existing positive `newtype_basic`, `enum_value_property`, `enum_valued` E2E
fixtures (compile and execute); formatting/file-size/source-direction/area
contracts; independent scoped review. Verify actual selection counts. Include
rejection controls for wrong receiver/arity, unclassified or forged authority,
and preserve user/protocol dispatch. No name-based fallback or fixture skip.

Emitted-Rust ownership was checked against issue #4344 (open, unassigned), which
lists the three type fixtures among 19 failures. This bounded repair owns only
the shared accessor mechanism under LOCAL-GATE V0.7; report observed overlap to
V0.11 without claiming the remaining async/network/process/callable-field cases
fixed. V0.8/V0.9 own separate method-slot and ancestry/upcast failures. Complete
phase gates and V5 remain mandatory. The user's sustained phase-execution request
continues the registered sequence while preserving one bounded item at a time.


V0.7 policy scope includes moving the eight existing registry-test node records
with their unchanged tests and refreshing only this item's changed source sites
and changed node/reference fingerprints. Admission logic and owner rules are
unchanged. `v07-method-policy-base` reconstructs exact base Rust inputs in memory
and proves **99 pre-existing method-policy inventory errors**. Assign that drift
to **V0.16, method-policy inventory reconciliation**, before final gates; do not
silently reclassify unrelated sites in V0.7. The affected-policy check must prove
no new errors relative to that baseline and run policy mutation self-tests.


### V0.7 validation and runtime closure

Candidate `e364e340978a1229211115416f1f0e0166b40ace`, base
`4790068c0899880f314ab34a0e9f82221ee1ee1e`, PR
[#4368](https://github.com/sifr-lang/sifr/pull/4368). The canonical registry now
handles typed zero-argument accessors; authority admission is unchanged. The
registry test extraction preserves all Rust tokens and qualified test IDs. The
native collision fixture now executes its unchanged Sifr assertions.

Evidence under `continuation-evidence/local-gate-implementation-20261020/`:

| Check | Actual result | Total seconds |
| --- | --- | ---: |
| `v07-current-native-baseline` | Original unsupported `value()` failure reproduced on exact merged base | 49.712 |
| `v07-codegen-accessor-controls-resumed` | 3 new controls pass; direct/alias receivers and rejection boundaries | 395.036 |
| `v07-codegen-all` | 1,759 pass, zero failed/ignored/filtered; 13.97s test execution | 16.752 |
| `v07-lowering-authority` | 5 pass, zero failed/ignored; 1,144 filtered | 10.243 |
| `v07-native-collision` | Exact previously ignored case passes, including binary execution; 16.76s test execution | 202.052 |
| `v07-e2e-accessors` | All three selected programs compile and execute; zero failed/blocked | 250.029 |
| `v07-final-policy-guards` | Format, size, source direction and area contracts pass | 16.678 |
| `v07-method-policy-delta` | Same 99 baseline errors; zero added/resolved diagnostics | 36.681 |
| `v07-method-policy-selftest` | Mutation self-tests pass | 1.135 |

`v07-final-validation.json` reconciles all successful checks against the exact
candidate, with no timeouts, leftover groups or source drift. The three existing
E2E programs are `newtype_basic`, `enum_value_property`, `enum_valued`; all original
assertions remain. This closes their shared accessor mechanism from #4344, not
the remaining historical async/network/process/callable-field failures in V0.11.

The initial `v07-codegen-accessor-controls` attempt stopped at the 20-GiB disk
reserve after 328.480 seconds before any tests ran. It remains incomplete. Six
inactive, owned prior-build incremental directories were copied into a recoverable
archive and verified against per-file SHA-256 hashes before their originals were
reclaimed. No Cargo/rustc process was active during archival; binaries, reports,
failed inputs and the active build configurations were preserved.
`v07-archive-receipt.json` records about 6.8 GB moved and restoration instructions.
The resumed preparation is not a warm performance comparison.

E2E telemetry attributes its 34.179-second suite to 3.118 seconds Sifr compilation,
29.722 seconds cache preparation/retirement, 0.643 seconds native build and
0.033 seconds runtime, plus discovery/setup/reporting. Cargo rebuilt one generated
unit and retained 29 dependency units. The enclosing 250.029-second command also
includes CLI/integration-test preparation. Assign the observed retirement cost
to V3; preserve failure markers, ownership and leases while optimizing it. These
bounded timings are not a complete gate result or a ten-minute qualification.

Independent review is **SATISFIED**, no blockers, with raw evidence at
`reviews/e364e340978a1229211115416f1f0e0166b40ace/review.md`. Runtime checks were
pending when review began and subsequently passed on the same frozen candidate.
The suggested V0.15/V0.16 status rows are now explicit. Another pre-existing
observation belongs to the V0.11 emitted-Rust owner: generated enum accessors are
private, so cross-module calls need investigation if affected coverage exposes
that boundary. V0.7 does not claim to qualify cross-module enum access.

The full method-policy guard is still failing for the 99 reproduced older
inventory diagnostics; the delta result is not a global pass. Bring V0.16 forward
next to reconcile those records with source and independent review before further
compiler repairs. `v016-inventory-analysis` is read-only preliminary analysis,
not an inventory repair or acceptance result. Final integrated gates and V5 remain
open.


V0.7 merged as #4368 at `1384e2a89262bf7cc67ad4d39364bcb8d2d8228e`.

### V0.16 bounded scope: reconcile reviewed method-policy records

The preliminary audit resolves 99 guard diagnostics to 49 missing records,
40 stale records and two obsolete keys. All additions and 38 updates describe
test assertions/harness operations. The other two updates retain the existing
compiler-policy role and identical `main` routing comparisons; their enclosing
`from_entrypoint_with_stages` signature now has the already-implemented
`prepared_native` argument. The obsolete keys describe the moved E2E pass runner
and the removed literal `/opt/sifr` fixture. Add the pass-runner record at its
current location; remove the obsolete fixture comparison without inventing a
replacement literal site.

Update only `verification/policy/method_dispatch_sites.json` and this phase
record. Preserve unchanged rows, owner/admission rules, canonical authorities,
node/reference relationships and all compiler/test source. Require exact
source/schema reconciliation, method-policy mutation self-tests, checks that the
change touches only the audited records, file-size and area contracts, local-link
checks and independent review. No compiler rebuild is justified for these
inventory/record-only inputs. Rust/verification inputs on the merged base are
byte-identical to the audited V0.7 candidate (Git diff over `crates`, `scripts`
and `verification` verified empty); retain the prior 99-diagnostic baseline.

This repairs evidence binding; it does not waive enforcement or prove new runtime
coverage. V0.8 is next after this bounded closure, and complete final gates/V5
remain mandatory.


### V0.16 validation and review

Candidate `2fbbf117205d98f3e613c20241b27b4d440d4cd9`, base
`1384e2a89262bf7cc67ad4d39364bcb8d2d8228e`, PR
[#4369](https://github.com/sifr-lang/sifr/pull/4369). Only the method-policy
inventory and phase record changed. All 234 constituent records, policy versions,
existing owners/classes and admission relationships are unchanged. Existing rows
retain their order. Six of the interop test updates also bind the actual current
operation text (`plan` probe access after the contract/qualification split),
while retaining the same test-assertion ownership.

Evidence under `continuation-evidence/local-gate-implementation-20261020/`:

- `v016-reconciliation.json`: 49 additions, 40 updates and two obsolete keys
  removed, with exact before/after inventory hashes and unchanged-row checks.
- `v016-method-policy`: complete source/schema/reference reconciliation passes,
  **6,393 sites and 234 reviewed nodes**, 35.156 seconds. This clears all 99
  previously recorded inventory diagnostics, rather than only passing a delta.
- `v016-method-policy-selftest`: mutation self-tests pass, 1.139 seconds.
- `v016-area-size-guards`: file-size and all area contracts pass, 3.641 seconds.
- Diff, local-link and existing owner/class checks pass. Successful receipts
  bind the exact candidate without timeout, leftover processes or source drift.

Independent review is **SATISFIED**, no blockers. Raw evidence:
`reviews/2fbbf117205d98f3e613c20241b27b4d440d4cd9/review.md`. Its wording suggestions
are reflected above. No runtime tests were replaced or marked passing by this
record reconciliation. The separately recorded V0.7 runtime validation remains
unchanged. Continue with V0.8; complete integrated gates and V5 remain open.


V0.16 merged as #4369 at `bbd4a94beab12b7ea4d7d4184d267c0c0a2c83bf`.

### V0.8 bounded scope: preserve generated method-slot call/signature agreement

`v08-current-native-baseline` reproduces the same four `String` versus `&str`
errors on the current compiler (47.729 seconds). The original Sifr slot methods
have owned string inputs; generated slot calls follow those declared conventions,
while their target signatures are subsequently borrowed. Investigate the exact
canonicalization boundary before changing slot argument emission. Use small
before/after programs against the public canonicalization API and actual rustc
compilation to discriminate incomplete call rewriting from emitter conventions.
The first simple `Self`/receiver/generic-tuple probes passed because their bodies
did not qualify for the string-borrow rewrite. With a qualifying `to_uppercase`
body, four reduced programs fail after canonicalization while their original
Rust compiles/runs. Three checked-in native regressions likewise compile/run
before and fail after the retained baseline canonicalizer (`v08-new-tests-against-base`). No blanket borrow, name fallback, global
optimization disablement or fixture weakening is authorized by this diagnosis.

Acceptance: a root-cause regression that fails before and compiles/runs after
canonicalization; relevant ownership/borrow identity controls and complete
codegen unit suite; the unchanged native method-slot runtime case (mutable,
shared and no-context variants) and lifetime/thread/shared-context rejection
case; affected policy/area/format/size contracts and scoped independent review.
Keep V0.9 ancestry/upcast, V0.11 unrelated E2E cases and final gates/V5 open.


### V0.8 implementation and remaining diagnostic boundary

The typed cleanup treated `Self` as a literal module/type owner and could not
resolve those inherent calls. Unit-struct receiver values also lacked a type
fact. Resolve `Self` through the actual enclosing implementation, and derive unit
constructor types only from known declarations, retaining lexical shadowing and
opaque-type boundaries. Slot decoding now emits the already-known HIR input type
on its local binding, including receiver/value tuples. This preserves semantic
information across the external runtime call without guessing that helper's
return type. The existing borrow-only string optimization remains enabled; slot
argument conventions and original runtime/negative fixtures are unchanged.

The three primary native regressions execute original and canonical Rust for
nested owner identity, unit receiver dispatch and typed opaque-decoder boundaries.
Additional native lexical controls check that local values, types, traits,
imports and modules do not acquire an outer declaration's identity; the successive
validation records below bind each candidate's exact test inventory. The method-policy update refreshes ten existing Rust-IR
fingerprints and adds the `Self` identity comparison, with existing ownership,
classification and all 234 constituent records unchanged.

Separate follow-up, owned by V0.11's emitted-Rust correctness investigation:
unannotated values returned by unknown external generic calls can still expose
string-signature rewrites whose callers lack sufficient type facts. The retained
`v08-syntax-probe/borrow-results.json` records those pre-existing cases. V0.8
repairs compiler-generated method-slot adapters by retaining their known input
contracts; it does not claim general transactional ABI optimization across opaque
foreign code. This concern must remain visible in the phase's final correctness
assessment rather than being treated as a passing probe.


V0.8 diagnostic refinement: the first nested-owner test also exposed a pre-existing
qualified cross-module trait-call liveness failure (E0576), preserved in
`v08-new-tests-against-base` and `v08-static-probe-direct-call`. V0.11 owns that
separate liveness defect. Keep the same two runtime assertions but invoke the
trait within its module in the bounded borrow regression. The resulting
`v08-static-probe-module-local-call` compiles before cleanup and fails specifically
with the two expected E0308 `Self::normalize` mismatches afterward. The first
focused candidate had four passing checks and this unrelated liveness failure
(459.045 seconds, 456 seconds preparation, 0.81 seconds tests); it is not a pass.
The first guard run passed format/area/method-policy/self-tests but failed the
900-line guard at 903. Move call resolution beside declared-method facts to keep
the main typed-rewriter source below its limit. Both attempts remain retained.


`v08-codegen-all` on `97f1374e6ba97ef9e0e322a47f4c90c264886bc4`
ran all 1,763 tests: 1,762 passed and one existing emitter text assertion failed
because it required an untyped tuple binding. All four new regressions passed.
Update that assertion to require the preserved `(Record, String)` type; retain
its receiver-call assertion. The failed run took 331.980 seconds including
316 seconds preparation and 13.81 seconds tests. The review started on that
candidate was stopped before mutation once this failure was known; its partial
output is not an accepted review. Revalidate and review the final candidate.


V0.8 candidate `d3af617f95ce829b1a6e071652feaf1d17ac8e85` passed all 1,763
codegen tests (47.749 seconds outer, 32.79 preparation, 13.79 test execution), all
required guards (48.182 seconds), and the two existing native method-slot tests
(89.47 seconds test execution after 337 seconds preparation). Its scoped review
was SATISFIED with no blockers, retained outside Git at
`reviews/d3af617f95ce829b1a6e071652feaf1d17ac8e85/review.md`.

Before merging, address the review's lexical-edge findings because they concern
the new identity/type inference: distinguish local value items from type items,
retain both namespaces across nested block inference, prevent block-local impl
owners from resolving to an outer same-named owner, and clear an enclosing impl's
`Self` while visiting a nested trait. Replace the invalid binding-shaped shadow
control with actual compiling/running local-const/static/type-namespace controls,
and add native local-impl/nested-trait controls. The typed cleanup alone is used
for these controls to isolate identity resolution from unrelated whole-pipeline
optimizations; the three primary native regressions still exercise the complete
canonicalizer. Revalidate and obtain a fresh scoped review after this refinement.


Candidate `a5e1d362b4fb69f7ac2a759a9ec015392be20c1d` passes all 1,765 codegen
tests (372.562 seconds outer, 356 preparation, 14.32 execution), required guards
(49.833 seconds), and both native slot tests (392.056 seconds outer, 303 preparation,
87.02 execution). Its renewed scoped review is SATISFIED with no blockers, raw
record `reviews/a5e1d362b4fb69f7ac2a759a9ec015392be20c1d/review.md`.

Close its remaining opaque-owner suggestion in the same bounded identity repair:
local imports make the owner opaque, and a local module can shadow the leading
segment of a qualified owner path. Apply that proof boundary to both new `Self`
lookup and unit-constructor inference. A seventh native control covers local
aliases, local-module `Self`, and a receiver constructed through that module.
Successful native unit checks now remove their owned scratch directories; panic
or interruption preserves failed/incomplete inputs and diagnostics. The helper
owns only its freshly allocated temporary directory.

The review's generic-parameter shadowing suggestion is assigned to V0.11's
emitted-Rust investigation alongside the existing opaque-call ABI/liveness
follow-ups. It is not claimed covered by this slot-adapter repair. The measured
codegen-test and driver preparation costs also remain inputs to V4's build-time
work; the diagnostic two-job allocation is not a ten-minute acceptance result.


`v08-opaque-owner-before` proves the new alias/module control fails on the
unrepaired resolver: original Rust compiles and executes, but typed cleanup adds
incorrect `.as_str()` calls and rustc rejects both `Self::normalize` calls with
E0308. The attempt took 330.985 seconds, including 328 seconds preparation and
0.49 seconds test execution; its failure inputs remain retained. The final
control separately exercises the local-module constructor path without the
opaque import from the first program masking its inference.


### V0.8 final validation and review

Implementation candidate `849bf963c4ca11523db33e6fb252b4fbf0802d5d`, base
`bbd4a94beab12b7ea4d7d4184d267c0c0a2c83bf`, PR
[#4370](https://github.com/sifr-lang/sifr/pull/4370). Final receipts and file hashes
are reconciled in `v08-final-validation.json` under
`continuation-evidence/local-gate-implementation-20261020/`:

- `v08-opaque-codegen-all`: **1,766 passed, zero failed/ignored/filtered**;
  356.967 seconds outer, 340 preparation, 15.21 test execution. The inventory
  comparison retains every one of the previous 1,759 test IDs and adds seven
  native regression/control tests. Expected-panic test labels are included in
  the reconciliation; no test IDs are lost to its parser.
- `v08-opaque-native-slots`: both unchanged native tests pass, including all five
  runtime outputs and exact lifetime/thread/shared-context rejection diagnostics;
  394.014 seconds outer, 303 preparation, 88.68 test execution. The 814 filtered
  driver tests are outside this named intermediate check, not implied passes.
- `v08-complete-guards`: formatting, size, all area contracts, method-policy
  reconciliation, mutation self-tests and diff check pass in 45.171 seconds.
  The final inventory has 6,394 sites and 234 nodes: twenty existing Rust-IR
  records refreshed and one Self identity comparison added; no existing owner,
  class or constituent record changed.
- `v08-native-scratch-cleanup.json`: no new native-input directories remain after
  the successful codegen run, while the earlier failure's original/rewritten
  sources and diagnostics are still retained.

All successful receipts bind the same candidate, with no timeout, reserve stop,
leftover process group or source drift. Independent review is **SATISFIED**, with
no blockers; raw evidence is
`reviews/849bf963c4ca11523db33e6fb252b4fbf0802d5d/review.md`. Its final pre-existing
receiver-method lookup follow-up (local imports or multi-segment local owners)
is assigned to V0.11 alongside the recorded generic/opaque-call/liveness items.
The cosmetic include-file spacing suggestion does not change validation.

This closes the bounded method-slot repair after merge under the approved
intermediate policy. It does not complete the phase or establish the ten-minute
acceptance result. V4 retains the measured production-library/libtest preparation
costs as work; V0.9 ancestry/upcast is next. Complete integrated gates and V5 remain
mandatory, and all failed/superseded attempts above remain preserved.


V0.8 merged as #4370 at `01ed090d1fdcad7677f2e8107bd323732bb4da8e`.

### V0.9 bounded scope: ancestry constructors and consuming upcast visibility

Reproduce the historical borrowed exact-integer constructor mismatch and private
intermediate ancestor escaping a public associated type against the current
compiler. Diagnose their producer/canonicalizer ownership before editing. Run
the existing project-generic native group together to expose related ancestry,
identity and structural conversion failures; preserve unrelated failures for
V0.11. Retain all existing native assertions. Add minimal root-cause controls
where existing tests do not cover the repaired boundary.

Acceptance: affected constructor/upcast unit regressions, complete codegen unit
suite, unchanged native ancestry/upcast scenarios (including imported aliases,
transitive/structural conversions, same-basename owners and non-main modules),
affected formatting/size/area/method-policy contracts and independent review.
Use retained compilation contexts and the 30-minute diagnostic cap; reserve
20 GiB disk and 2 GiB memory. Final integrated gates and V5 remain mandatory.


V0.9 current baseline `v09-current-native-group` binds candidate
`a3ff03d2d` and executes all nineteen project-generic tests with ignored native
cases included: **17 pass, 2 fail**, 35.766 seconds outer (2.10 preparation,
31.74 test execution). The failures are exactly the same-basename ancestor
constructor's `SifrInt`/`&SifrInt` disagreement and the structural consuming
upcast's private `Mid` in public `Deref::Target` (E0446). Other related tests now
pass and are not repaired speculatively. All generated failed inputs are retained.

The proposed repair closes scalar-borrow propagation across callers before
per-file canonicalization, and closes visibility demands along the emitter's
fully qualified standard `Deref` ancestry edges. It retains owned inputs when
consumed, resolves declared module aliases, normalizes restricted audiences to
absolute scopes, and leaves unrelated structs, generic type parameters,
external paths and private/custom trait contracts alone. New tests check
fixed-point stability, source ordering, transitive parents and those boundaries.


V0.9 intermediate validation preserves both new baseline failures in
`v09-new-regressions-before` (342.024 seconds, 338 preparation). The initial
candidate failed compilation on the pinned Syn 3 trait-implementation tuple
shape; `v09-codegen-all` retains that 12.748-second failure. The corrected
candidate `ba7ee037d04e81432469cd9805b460f271452d09` passes 1,769 of 1,770 codegen
tests in 362.931 seconds (346 preparation, 15.05 execution); the remaining
existing test requires `pub(super)` instead of its equivalent `pub(crate)` in
a top-level module. Preserve that original assertion and render immediate-parent
audiences as `pub(super)`. Both ancestry regressions and added boundary controls
already pass at this checkpoint. `v09-guards-repaired` passes all guards in
45.681 seconds. Its independent review is SATISFIED with no blockers, conditional
on final tests, so this is not accepted final evidence. Re-review the rendering
correction and rerun the complete codegen suite plus unchanged native group.


### V0.9 final validation and review

Implementation `226b8f3acb5c09d3198fc147e98566249dafb16c`, base
`01ed090d1fdcad7677f2e8107bd323732bb4da8e`, PR
[#4371](https://github.com/sifr-lang/sifr/pull/4371). Exact-candidate receipts and
hashes are reconciled in `v09-final-validation.json` under
`continuation-evidence/local-gate-implementation-20261020/`.

- `v09-codegen-final`: **1,770 passed**, zero failed/ignored/filtered;
  334.399 seconds outer, 318 preparation, 14.55 execution. The inventory retains
  all previous 1,766 IDs and adds four root-cause/boundary tests.
- `v09-native-final`: **19 passed**, zero failed/ignored, 797 filtered outside
  this named group; 374.444 seconds outer, 336 preparation, 36.14 execution.
  The exact nineteen baseline IDs are retained. Both historical failures now
  compile and execute; all imported/generic/union/ancestry runtime assertions
  remain unchanged. No full-driver pass is inferred from this focused group.
- `v09-guards-final`: all formatting, size, area contracts, method authority and
  mutation self-tests plus diff check pass in 48.176 seconds. The inventory has
  6,391 sites and the same 234 constituents: four obsolete local path comparisons
  disappear when resolution uses the existing shared helper, and one test
  comparison is added. Every other site/class/owner remains unchanged.

All final receipts bind the reviewed candidate with no timeout, reserve stop,
leftover process group or source drift. Independent review is **SATISFIED**, no
blockers: `reviews/226b8f3acb5c09d3198fc147e98566249dafb16c/review.md`. Its required
codegen/guard/native conditions are now satisfied. The earlier review and failed
attempts remain retained; the interrupted initial review was not accepted.

Follow-ups: V4 owns avoiding repeated whole-project token rendering during
fixed-point analysis and investigating optional convergence diagnostics without
changing correctness. Visibility uses normalized audiences; equivalent relative
spelling is preserved for immediate parents, without promising identical spelling
for every equivalent source declaration. Imported ancestry remains covered by the
unchanged native tests; the existing emitter publicizes non-main module types.
The native unit control uses the selected pinned toolchain in the package cwd.
No final full-gate or under-ten-minute result is claimed; V0.10 and V5 remain open.

Additional V4 source finding: `sifr_identity::build::emit_local_token` visits the
whole package, including test files, and emits source/configuration tokens.
Moving tests only to another directory in that same package cannot remove that
identity input. Evaluate cohesive public-behavior test ownership together with
production-input identity; do not omit genuine production dependencies or expose
private APIs just to move tests. The observed 318/336-second preparations are
separate library/test contexts, not an additive full-gate forecast.


V0.9 merged as #4371 at `eec85331bb2e5cb2f6c851ed2fe3db346ae65471`.

### V0.10 bounded scope: independent CLI schema parity

Reproduce the current eager/deferred schema mismatch, then align the eager test
reference with the production test command's locked/offline/frozen flags. Keep
independent schemas, exact match/error/help/order equality, all existing loop
cases and production normalization semantics. Add explicit coverage for all
three-flag combinations in both normal and update construction modes and
representative option orderings. No production command behavior changes.

Acceptance: the complete eager-contract and deferred-parser test groups, existing
build/test lock-mode parse/normalization controls, added flag combinations,
formatting/size/area and affected policy checks, plus independent review. Use
retained targets and the existing 30-minute/resource limits. Final integrated
create-PR/merge gates and V5 remain required.


### V0.10 final validation and review

Implementation `0144543fe075eb7065632b333851fa88d3387d3f`, base
`eec85331bb2e5cb2f6c851ed2fe3db346ae65471`, PR
[#4372](https://github.com/sifr-lang/sifr/pull/4372). Only the independent eager
test schema and added regression change; production parsing, normalization,
profiles, locks, test selection and existing assertions are unchanged.

`v010-cli-baseline` reproduces the exact two composite failures: 9 pass, 2 fail,
143 filtered; 397.551 seconds outer, 387 Cargo preparation and 0.21 execution.
The missing fields cause equality failure at `sifr test`, so later loop entries
are not claimed covered by this baseline. `v010-cli-final` passes **all 12 named
tests**, zero failed/ignored and the same 143 filtered, in 40.799 seconds outer
(37.39 preparation, 0.31 execution). It retains all eleven original IDs and adds
one regression with 32 parity cases, explicit success for sixteen update-mode
inputs and expected values for each normal-mode flag. Both formerly failing
composite tests now execute their complete loops. The retained dependency context
explains most of the shorter final preparation; this is not a measured compiler
optimization or full-gate timing result.

`v010-guards-final` passes formatting, size, all area contracts, unchanged method
policy and mutation controls, and diff check in 49.192 seconds. The source is
743 lines. Independent review is **SATISFIED**, no blockers, in
`reviews/0144543fe075eb7065632b333851fa88d3387d3f/review.md`; its required final
checks now pass. The earlier review at `c02622144` prompted the added update-mode
acceptance assertion before final validation. Receipts, file hashes and retained/
added/repaired test identities are in `v010-final-validation.json`, under
`continuation-evidence/local-gate-implementation-20261020/`. No final timeout,
reserve stop, leftover process group or source drift occurred.

V4 owns the review's separate suggestion to collect every independent CLI schema
mismatch before reporting a failed aggregate. The optional duplicate value checks
and additional release/lock combinations are not acceptance gaps; exact reference
equality and the existing normalization/release controls remain retained. The
CLI baseline prepared a distinct existing Cargo dependency context; captured
process arguments and eight retained codegen fingerprint records are in
`v010-cli-preparation-processes.json` and `v010-codegen-contexts.json`. Differences
in dependency fingerprints are evidence for investigation, not proof of cache
corruption or of an unnecessary feature variant. V0.11 current E2E failure
reconciliation is next; final complete gates and V5 remain mandatory.


V0.10 merged as #4372 at `2c6adad497949b4a7855c5e26f150e62609916b7`.

### V0.11 discovery boundary: current complete positive E2E inventory

Issue #4344 remains open and unassigned; its technical compiler/codegen owner and
nineteen historical cases are explicitly coordinated by this phase. Three nominal
accessor cases already passed unchanged in V0.7. Run the entire current positive
E2E inventory against the integrated current compiler before assuming the other
sixteen remain the only failures. Preserve every fixture and assertion, stage,
error and timing. Use the retained owned Cargo/native targets, two compiler jobs,
two frontend/run workers and one native group with two Cargo jobs, reserving
20 GiB disk and 2 GiB available memory with the 30-minute category cap.

This is discovery, not a new full-gate pass: negative/runtime-failure corpora and
all other final selected verifications remain required. Group reproduced failures
by actual compiler boundary, then register the next bounded V0.11 subitem and its
regressions/acceptance before editing compiler code. Do not silently absorb
separate V0.12 demo/async work or the recorded lexical identity follow-ups; reconcile
shared causes explicitly when evidence shows they coincide.


### V0.11 current complete inventory and V0.11a bounded scope

At `1ecff7f7b`, `v011-full-positive-resumed` completed all 729 positive
fixtures: 713 runtime passes, 15 source compilation failures and one native
compilation failure (`callable_struct_field`, E0599). Outer wall time was
128.786596 seconds; source compilation 53.230751, native build 55.511499,
individual failure isolation 4.603599, runtime 7.333808, reporting/retention
1.024194 seconds. This retained-target diagnostic is a failed positive suite,
not final complete-gate acceptance or a performance forecast. The earlier
`v011-full-positive-baseline` stopped at 198.311322 seconds on the 20 GiB disk
reserve; retain it as incomplete. Two obsolete prior-day owned codegen
incremental directories were archived and independently hash-verified before
removal; current prepared contexts, native cache and failed evidence remain.
Recovery inventory: `v011-inactive-incremental-archive/receipt.json`.

The remaining source failures are the two async-generator close fixtures, six
JoinSet fixtures, task-context propagation, two TaskGroup CPU fixtures,
TaskScope blocking spawn, two scoped-process fixtures and the network HTTP/TLS
loopback fixture. These reconcile with #4344 after V0.7's three nominal accessor
repairs. They remain separate from V0.11a and V0.12 until their actual dispatch
boundaries are established. Raw evidence and per-case/stage reconciliation are
`v011-full-positive-resumed/` and `v011-full-positive-resumed-summary.json`.

V0.11a owns callable-field invocation in recursive expression emission. The
validated nominal dispatch branch currently emits every admitted call as a Rust
method, bypassing the receiver's existing callable-field query; the statement
branch already emits a field invocation. Preserve frontend method authority,
resolved receiver types, argument-place and convention handling, and ordinary
method precedence. Do not add a method-name fallback or weaken admission.
Preserve the original failing fixture and all existing assertions. Add focused
codegen and native controls for nested/direct invocation, reusable borrowed
string arguments, and an ordinary method with the same spelling.

Named acceptance: all codegen library tests (including existing authority
rejection, structural/async callable and exact-integer controls); original
callable-field and disjoint sibling fixtures, structural records, callable
borrow conventions and new native context regression; the complete compile-fail
E2E corpus; formatting, file-size, area and method-authority checks/self-tests;
one scoped independent review. Intermediate checks follow the approved
phase-end full-gate policy. No frontend/ABI/profile/test-selection changes are
part of this item.


### V0.11a final validation and review

Implementation `1bb2e0080c91d1d85d96f20e0687affa74ea765e`, PR #4373,
base `2c6adad497949b4a7855c5e26f150e62609916b7`:

- `v011a-callable-codegen`: both new regressions pass; 343.536747 seconds outer,
  341 seconds preparation and 0.01 seconds execution. This first focused run
  used `21018c3bee8e3176e0df04d6436dad3e55147c7e`; the only subsequent change
  was the method-policy graph fingerprint record.
- `v011a-codegen-all`: **1,772 passed, zero failed/ignored/filtered** on the final
  candidate; all 1,770 prior tests retained plus the two new regressions.
  17.249008 seconds outer / 14.45 seconds execution with preparation retained.
  Reconciliation includes expected-panic tests; `v011a-codegen-inventory.json`.
- `v011a-native-callables`: all five named fixtures pass source compilation,
  native compilation and runtime assertions, including unchanged original
  `callable_struct_field`, the new `callable_field_call_contexts`, disjoint
  sibling invocation, structural records and callable borrow conventions.
  273.101182 seconds outer / 255 seconds Cargo preparation / 9.762989 seconds
  E2E suite. These are stages of one invocation, not an additional full gate.
- `v011a-negative-all`: **all 569 compile-fail fixtures pass**, 9.246338 seconds
  outer. Expected diagnostic assertions remain intact.
- `v011a-guards-final`: formatting, 900-line guard, all area manifests, method
  authority inventory and its 17 mutation/self-tests pass; 51.414921 seconds.
  The first `v011a-guards` failed on stale graph context fingerprints and is
  preserved. No policy classification, role, signature, relationship or reference
  identity changed: 6,391 sites and 234 graph nodes remain; hashes change only
  in the edited function or references from its context. Reconciliation:
  `v011a-policy-graph-reconciliation.json`.

Independent review: **SATISFIED**, no blocking findings; raw report at
`reviews/1bb2e0080c91d1d85d96f20e0687affa74ea765e/review.md`. Its pending-test
conditions are now fulfilled by the receipts above. The optional structural
recursive-call test suggestion is a separate coverage enhancement; existing
structural-record native and codegen assertions remain passing. Review is not
final phase acceptance. `v011a-final-validation.json` binds receipt/log hashes,
retained/added test identities, native fixture identities, negative count and
review hash. No required skips, resource stops, leaked workers or unexpected
tracked changes occurred in the final checks.

The original E0599 is repaired without changing its fixture or relaxing method
authority. Fifteen current source-emission failures remain for the next bounded
V0.11 item; other V0/V1–V5 work and full final gates remain open. This change
establishes correctness, not a compiler-preparation speed improvement.


V0.11a merged as #4373 at `d1c69e325457ce64594a61091ab2cb258a4c183e`.

### V0.11b bounded scope: admitted runtime method emission

Own the fifteen source-emission failures reproduced by the complete current
positive suite: async-generator close (2), JoinSet add/spawn/terminal scenarios
(6), TaskGroup CPU (2), TaskScope blocking (1), task context (1), scoped process
(2), and HTTP/TLS loopback (1). Start with a single aggregate driver/codegen
regression using the existing demanded-stdlib frontend/codegen helper and the
unchanged fixtures. Require zero lowering error counts and no generated
`compile_error!`; retain detailed per-fixture error evidence and collect all
failures before asserting. Do not infer successful codegen from the presence
of runtime support definitions alone. Include already passing task/async
controls. This diagnostic step does not change production code or public
compiler diagnostics.

Static inspection suggests two dispatch gaps requiring validation: runtime
builtin methods missing from registry handling, and task-owner shortcuts that
accept only a subset of validated argument expression shapes. The generated
runtime already implements the methods. Establish the concrete failing methods,
receiver identities, argument conventions and runtime signatures before repair.
Register the implementation boundary from that evidence. Retain canonical
frontend authority, typed owner identity, argument/place/ownership semantics,
method arity and strict decline; no arbitrary method fallback or compatibility
route. Coordinate with V0.12 only if a demonstrated common cause requires it.

Named acceptance for the eventual implementation: new aggregate codegen
regression, focused wrong-owner/arity/authority/collision and ownership controls,
all codegen library tests, unchanged fifteen native E2E fixtures plus related
passing controls, all positive E2E fixtures after integration (now 730 with the
V0.11a addition), complete compile-fail corpus, required fmt/file-size/area/method
policy checks and scoped independent review. Preserve intermediate failures;
full final create-PR/merge gates and V5 remain mandatory at phase end. Separate
recorded lexical/opaque ABI follow-ups remain outside this dispatch repair.


### V0.11b diagnostic result and implementation boundary

Test-only candidate `00d9570b56a92c9c5e9fe6ab236ff602a5552579`:
`v011b-runtime-codegen-baseline` completed in 290.906024 seconds and reproduced
all fifteen failures while the five passing controls remained clean. The twelve
explicit errors are typed builtin declines for AsyncGenerator `aclose`, JoinSet
add/CPU/blocking helpers, TaskGroup CPU/context and TaskScope blocking helpers.
Network and both process fixtures expose a missing TryExcept statement; the
TryExcept boolean entrypoint collapses nested `Err` into `false`, so their actual
nested cause is not yet established. Preserve raw HIR-bearing error lines and
`v011b-runtime-diagnostics.json`; do not claim three additional causes from a
masked diagnostic. Keep the same aggregate regression to test the hypothesis.

Implement explicit typed registry support for the missing runtime helper
contracts, with exact receiver identity and arity. Share canonical TaskScope/
TaskGroup type construction between lowering and codegen; retain the existing
frontend authority. Route task-owner calls through the normal contextual
receiver/argument machinery instead of the restricted leaf shortcut. Keep
already working task join/cancel/timeout and JoinSet terminal paths in scope
for regression checks, without a general Rust-method fallback. Preserve
move-only handles, real coroutine argument adaptation and reusable context
values. Correct JoinSet's rewritten `__sifr_add_task` and
`__sifr_add_blocking_task` classification to mutable receivers and their known
growth effect, matching the runtime's `&mut self` contract. Retain checked place
validation rather than bypassing it in the backend.

Add canonical-owner/type collision and arity controls, strict authority controls,
and focused mutation metadata tests. Run affected type-system/lowering checks
in addition to the original acceptance list. If the three TryExcept failures
survive this repair, record their detailed cause and explicitly reconcile their
V0.12 ownership before broadening implementation. No production diagnostic
interface change is authorized by this boundary.


V0.11b first implementation `1e1edce1cbaa193b513c0bc8e9b51ba1790e5dc0`:
canonical task-owner control passed (13.274567 seconds); full lowering/type-system
suites passed 1,148 and 154 tests respectively (113.836628 seconds outer), with
one pre-existing ignored lowering deque test requiring the driver-loaded stdlib
registry, not counted as passing. All 1,774 codegen tests passed in
`v011b-codegen-all` (392.030126 seconds outer); an unused import from the removed
shortcut is corrected next. Guards passed (45.194222 seconds, 6,394 sites and
235 nodes). Native and aggregate post-repair validation remain pending.

Add direct native coverage for moving a BlockingTask into JoinSet, alongside the
new context-reuse native fixture. This exercises the second rewritten add helper
and its mutable receiver/affine ownership contract, rather than relying on the
registry arity control alone. Both new fixtures join the aggregate regression
(22 fixtures) and the full positive corpus (732 fixtures). Existing assertions
are retained. Review and final phase gates remain open.


V0.11b integration regression and repair boundary: candidate
`1ce62b3f3beb56d1dbcb1311d42f66867314fd18` passes the corrected 22-fixture
aggregate (7.694606 seconds outer / 4.04 seconds execution), all 22 focused native
fixtures (390.262610 seconds outer / 22.018239 seconds suite), and all 569
compile-fail fixtures (9.189224 seconds outer). Full positive integration completes
732 outcomes in 157.810195 seconds outer: 731 pass, and the unchanged
`task_handle_collection_loop_consumed` fails native Rust compilation with E0277
because structured emission uses `handles.iter().cloned()` for non-Clone task
handles. Prior conditional review is not accepted. Preserve all failed evidence.

The old simple-loop path had a local task-list consuming override; the shared
iterator ownership planner still selected Preserve/Clone for the same named list.
Removing the restricted task-owner expression shortcut makes the surrounding
async scope use structured emission, exposing this disagreement. Move the existing
Task-list Consume/Move rule into the shared planner and remove the duplicate
simple-loop override, so both paths agree. Do not make task handles clonable or
change the original fixture. Add a planner regression and whole-source generated
Rust regression; include the original fixture in the aggregate and focused native
selection (now 23, with the full positive inventory unchanged at 732). Rerun all
codegen tests, aggregate, focused native, full positive/negative suites and guards;
request updated independent review. This is an in-scope regression correction.


### V0.11b validated closure: runtime dispatch and consuming task-list iteration

Implementation candidate `ffa7a4ebfd3c55b5b72fadc08bde9185540f5cee`, PR #4374,
base `d1c69e325457ce64594a61091ab2cb258a4c183e`. All named acceptance now passes.
The aggregate establishes that the twelve explicit runtime method declines and
three masked TryExcept failures share the repaired dispatch path. The integrated
native run also validates the consuming task-list regression repair. No original
native fixture or assertion was changed: all 730 fixture files at the base are
identical, with two new fixtures for context reuse and BlockingTask transfer.

| Check / evidence label | Result | Outer seconds | Execution / suite seconds |
| --- | --- | ---: | ---: |
| `v011b-task-loop-codegen` | Seven focused task-handle controls pass | 306.528268 | 0.04 |
| `v011b-loop-codegen-all` | 1,776 pass; all 1,772 previous IDs retained, four added | 18.308197 | See raw test receipt |
| `v011b-loop-aggregate` | All 23 original/new/control fixtures emit without lowering errors | 232.884251 | 9.11 |
| `v011b-loop-native` | All 23 native fixtures compile and execute | 299.740247 | 14.332693 |
| `v011b-loop-positive-all` | All 732 pass; zero source/native/runtime or original batch failures | 115.849885 | 109.275594 |
| `v011b-loop-negative-all` | All 569 compile-fail fixtures pass | 9.246120 | See raw test receipt |
| `v011b-loop-guards` | fmt, size, all area manifests, method policy and 17 mutation self-tests pass | 52.141451 | Not separately measured |

The full positive suite includes source compilation 52.195900 seconds, native
compilation 47.908274 seconds, and runtime assertions 9.007272 seconds. These are
observations with retained preparation, not the complete correctness gate or a
prediction for compiler edits. Preparation remains explicit in every outer time.
The earlier 1,148 lowering / 154 type-system passes are reused only after checking
that those crate sources and workspace Cargo/lock/toolchain/config inputs are
unchanged since `1e1edce1cbaa193b513c0bc8e9b51ba1790e5dc0`. The pre-existing
ignored bare-deque lowering test remains separately recorded, not a pass.

Raw review is outside Git at
`reviews/ffa7a4ebfd3c55b5b72fadc08bde9185540f5cee/review.md` in the owned evidence
bundle. Verdict SATISFIED, no blockers; all conditional test requirements are now
fulfilled. `v011b-final-validation.json` binds each command, candidate, receipt,
stdout/stderr/resource hash, fixture names, test-ID reconciliation and review hash.
The invalid new-fixture assertion and the later full-suite task-loop failure are
preserved; neither earlier conditional review is used as acceptance evidence.

Follow-ups remain explicit: frontend move tracking for task-list comprehensions
and non-name places must reject or represent consumption consistently; it
currently only marks named for-loop sources moved. This is a pre-existing
lowering/ownership gap outside the repaired named-loop contract. A future
compiler-generated temporary JoinSet using `__sifr_add_blocking_task` also needs
its internal mutation classification; source calls already have checked mutable
places and the native transfer fixture passes. Neither follow-up removes or
weakens required phase assertions.

V4 build-context evidence now identifies real feature differences between the
retained driver and CLI codegen graphs, including `ruff_text_size` serde and
proc-macro dependency features; see `v011b-build-context-differences.json` and its
source fingerprint snapshots. Do not omit identities to force reuse. Three
obsolete prior-day owned test executables were archived and hash-verified before
removal to restore disk reserve; libraries/incremental caches, current executables
and failure evidence remain. Restore instructions and hashes are retained in
`v011b-obsolete-test-binaries/receipt.json`.

This is intermediate acceptance under the approved phase policy. Full final
create-PR/merge correctness gates and V5 remain open. Next: V0.12 canonical demo
compilation and generated companion freshness, using this repaired compiler.


V0.11b merged as PR #4374 at `9025dad7470efeb3647cf0ebacaf948a02a8aca8`.
Final reviewed/tested implementation remains `ffa7a4ebfd3c55b5b72fadc08bde9185540f5cee`;
final head `f95d522608f81db1f6a69abfd70191b6f8e138ab` changed only this record.

### V0.12 bounded scope and acceptance

First rerun the complete canonical `generated_code_quality:demos` and
`generated_code_quality:freshness` contracts with the repaired compiler. Preserve
all per-demo failures and freshness comparisons; prior 43-stale evidence remains
historical, not a claim about this candidate. Diagnose before changing production
code, snapshots or quality debt. No subset or stale-list allowance substitutes
for the original contracts. Use the selected compiler and retained owned targets;
include preparation in outer timing and retain the 30-minute diagnostic cap.

When emitted companions differ, reconcile each change to current source/compiler
behavior and preserve the demo's source assertions. Regenerate only justified
current outputs, then require a second complete freshness check and deterministic
comparison. Reconcile exact quality inventories if generated safety findings
change; do not increase allowed debt merely to pass. Any remaining compiler
semantic failure must be reproduced and bounded before a repair; do not discard
independent demos after the first failure.

Named acceptance: all canonical demo entries, all authoritative companion
comparisons, affected generated-code inventory/safety/format/determinism contracts,
focused regression checks for any actual code change, required guards, and scoped
independent review. Existing phase-end full correctness gates and V5 remain
mandatory. Separate sanitizer/process/caching/compilation work stays in its
recorded item. No demo source renames or unrelated refactors are implied.

### V0.12 measured baseline and companion reconciliation

At `9198c640e48dc7cd7df6b660e312f6bc83b37d46`, all ten canonical demos pass
in 10.190s after their portable dependency graphs are prepared. The initial
14.694s attempt failed on an unprepared offline Git dependency, before Rust
validation. Full-profile preparation was intentionally stopped after 549.512s
and 46 of 93 graphs, including all ten demos: it is incomplete, not a profile
pass. V3/V4 retain the excessive preparation and revision-root lifecycle work.

The complete freshness baseline compared 264 companions in 271.472s: 41 stale,
zero orphaned. Staging those 41 emissions outside Git took 68.226s with four
workers. Every emission succeeded without a `compile_error!` marker. All 61
distinct replacement blocks were inspected: string construction/comparison and
character slicing, collection ownership/deque methods, borrowed-list copying,
integer operator rendering, optional-character comparison, and the existing
BigDecimal precision/half-even rounding policy. These are current compiler
outputs; no demo source or assertion is changed.

The proposed complete set of 264 companions has zero generated safety findings
under the existing exact-debt policy. No debt allowance is added. Evidence is
retained in `v012-companion-proposal/report.json`, its per-file diffs and
`change-cohorts.json`, and `v012-safety-proposal.json` outside the reviewed tree.
Complete freshness, deterministic comparisons, affected quality contracts,
guards and independent review remain pending on the implementation candidate.

V0.12 strict follow-up found three generated quality failures after all 41
projects were prepared: `nested_function_backtracking` and
`nested_function_composition` each have three `implicit_clone` diagnostics;
`nested_helpers` has four `arithmetic_side_effects` diagnostics. All generated
safety scans were empty. The other 38 changed companions passed strict Clippy.
The ten canonical demos, their safety/format/determinism checks and four native
controls passed. The extra 41-companion determinism probe did not execute: its
threaded diagnostic helper used a main-thread-only process supervisor. Preserve
that harness failure and rerun with a corrected helper; it is not evidence of
compiler nondeterminism.

The repair distinguishes borrowed vectors from slices, preserving the owned-copy
result across reference depth, and uses resolved runtime-integer/range facts for
borrowed arithmetic cleanup. Unknown, shadowed and primitive types retain their
operations. No source assertion, lint suppression or quality-debt allowance is
changed. These implementation changes require fresh affected tests, emissions
and scoped review; the initial snapshot-only review does not approve them.

V0.12 compiler repair validation at
`906b48b8cb3f9aece2827a476edacf9d3b555499`: all **1,780 codegen tests pass**
(345.619s outer; Cargo reports 5m29s preparation and 15.20s test execution).
The four added tests cover borrowed vector/slice copies, reference depth, exact
integer range items, and unknown/shadowed identities. The existing borrowed
parameter regression now checks ordered Add/Sub call arguments in the Rust AST,
retaining the borrowed signature and no-extra-reference assertions. Its earlier
syntax-only failure remains in `v012-codegen-repair`; no assertion was dropped.

Required guards pass in 48.186s. The exact dispatch inventory retains all existing
classifications and 235 reviewed nodes; four new sites describe Rust-IR inspection
and test assertions. It does not create a source-language dispatch authority.
An independent Rust execution control verifies that vector copies through multiple
reference layers remain owned and do not mutate their sources.

The selected repaired CLI staged all **264** companions successfully and repeated
all **41** originally changed companions byte-for-byte. Only the three diagnosed
outputs changed: vector copying returns to `clone()` for borrowed vectors, and
borrowed exact-integer operations use the established explicit operator calls.
The complete staged safety inventory remains empty. The staging operation took
660.265s including compiler preparation; it is not full-gate timing qualification.
Those three generated files are reconciled; complete freshness, affected strict
quality checks, native regressions, broader E2E coverage and new scoped review
remain required before this item can merge.

### V0.12 final intermediate acceptance

Implementation candidate `313d9d869a487bdeb0c0b68654fdaaf191209920`,
[PR #4375](https://github.com/sifr-lang/sifr/pull/4375), passes all named
intermediate acceptance checks. The final net change is 38 generated companions;
the three repaired outputs returned to their baseline bytes. No demo source or
assertion changed, and no quality debt or lint suppression was added.

| Check | Result | Observed outer time |
|---|---|---:|
| Complete generated inventory and freshness | 264 comparisons pass | 294.986s |
| Complete positive E2E corpus | 732 pass | 126.769s |
| Complete negative E2E corpus | 569 pass | 8.687s |
| Portable generated-graph preparation | 51 pass; manifest/lock hashes unchanged | 693.584s |
| Affected strict Clippy and generated safety | 41 pass; empty findings | 112.748s |
| Complete canonical demos | 10 pass | 30.700s |
| Canonical panic scan and negative controls | pass | 1.680s |
| Canonical formatting and negative control | pass | 1.686s |
| Canonical determinism and negative control | pass | 94.236s |
| Canonical Clippy and negative controls | pass | 10.687s |
| Native semantic controls | six original assertion-bearing demos pass | 122.817s |

The positive corpus executed every fixture with 186/186 native cache hits:
60.003s source compilation, 41.385s native checks/builds and 8.683s execution.
These are diagnostic timings, not the V5 acceptance matrix. Generated preparation
alone still exceeds ten minutes; V2/V3/V4 must eliminate the demonstrated repeated
work before the complete gate can meet its target.

All 1,780 codegen tests and required guards passed on the identical compiler/test
inputs at `906b48b8cb3f9aece2827a476edacf9d3b555499`; only three generated
companions and this phase record changed afterward. All 264 companion formatting
checks are covered, with the final three rechecked using the canonical Rust 2024
command. The selected CLI hash remains
`0956feb8404b0b126098784e4235de914626a7820e7f49d580fbfe23e30304ee`.
All 264 repaired emissions succeeded and all 41 affected repeat comparisons were
byte-identical. Input reconciliation is retained outside Git.

Scoped independent review is **SATISFIED**, with its pending-test conditions now
fulfilled. Raw review: `reviews/313d9d869a487bdeb0c0b68654fdaaf191209920/review.md`
in the external implementation evidence bundle. `v012-final-acceptance.json`
binds results and review to the implementation candidate. Historical stale,
strict-Clippy, test-rendering, inventory and helper failures remain preserved.
The record-only closure commit does not alter validated implementation inputs.

Separate follow-up owners: formatting/guard ownership should cover Rust files
included through `include!`; future codegen quality work owns conservative global
shadow detection. A same-shape positive range-shadow control is a nonblocking
codegen-test suggestion. These are not additional V0.12 merge requirements.
Full integrated create-PR/merge correctness gates and the V5 timing matrix remain
open. Next bounded implementation item: V0.13 sanitizer fixture and target
propagation, retaining compiler and generated-program instrumentation.

### V0.13 bounded scope and acceptance

V0.12 merged as `538577f1eb34416d2c018f561fb4690002476b42` through
[PR #4375](https://github.com/sifr-lang/sifr/pull/4375). V0.13 starts from that
candidate. The unchanged sanitizer manifest still references absent
`demos/hello.sifr`; the prior diagnostic used the existing assertion-bearing
`crates/sifr/tests/e2e/pass/break_continue.sifr` without qualifying that manifest.

Repair the declared generated fixture and validate its required input before
starting expensive builds. Propagate the explicit current-host target to nested
generated Cargo as well as the instrumented compiler build, retaining Cargo
artifact authority for executable paths. Preserve caller build storage and flags
while ensuring encoded Rust flags cannot silently disable declared sanitizer
instrumentation. Keep existing tools, hosts, timeouts, native assertions and
structured skip policy; a required local smoke skip is not an acceptance pass.

Named acceptance: affected sanitizer policy/self-tests with missing-input,
target/argument, flag-precedence and failure controls; required runner/area guards;
real runtime and generated-binary ASAN smoke execution with compiler and generated
instrumentation verified against actual artifacts; retained preparation and
representative compiler-edit timing recorded separately under the diagnostic cap;
and scoped independent review. Preserve historical failures. Full final phase
gates and V5 remain required. General process supervision and preparation-consumer
ordering remain owned by V0.14; do not broaden this item into that lifecycle work.

### V0.13 policy validation and review remediation

The first 16 sanitizer policy checks and required guards passed. A broader runner
self-test fails in the unchanged DX.10 application-profile authority: it declares
729 positive fixtures, while the repository has 732. The missing entries are
`callable_field_call_contexts.sifr`, `join_set_add_blocking_task.sifr`, and
`task_context_reuse_across_spawns.sifr`, introduced by V0.11. The authority is
byte-identical to this item's base. V0.16b owns that exact reconciliation before
final phase gates; `v013-runner-selftest` remains failed, not an implied pass.

Initial scoped review of `35e113a0e5380ebf0902df274d7626e4dd038c5b` is
**NOT SATISFIED**: using shell parsing for inherited plain Rust flags changes
Cargo semantics. The first correction split on literal spaces, kept quotes and apostrophes,
preserved embedded tabs/newlines, and emitted encoded flags with the declared
instrumentation. Eighteen focused controls passed on that candidate.
A direct Cargo comparison confirms all five exercised inputs keep their original
outcomes and caller arguments: three valid cases compile, and two malformed
embedded-whitespace cases remain rejected. The required ASAN flag reaches rustc.
These are correctness controls, not performance qualification. Raw initial review
and both probe results remain in the external implementation evidence bundle.
Fresh affected checks and scoped remediation review remain required, together
with real instrumented execution/artifact evidence and retained/edit timings.

The remediation review of `bfaa2aa06f209172a1aae5cf8a4cd8af7639521a` also
found an in-scope blocker: Cargo trims surrounding Rust whitespace from each
space-separated piece. The first parser filtered empty pieces without trimming
retained pieces. This review remains **NOT SATISFIED** in the evidence bundle.

The final approach preserves plain flags as plain text and encoded flags in their
original encoding, appending required instrumentation in that same format. Cargo
owns execution parsing. Conflict validation follows Cargo's literal-space split
and Rust whitespace trim, preserving control separators that Python would strip.
It no longer translates caller plain flags into encoded flags, so a literal unit
separator inside a valid quoted cfg value keeps its original argument boundary.
Twenty-two focused tests and fourteen direct Cargo comparisons pass: ten valid
inputs compile and four malformed inputs retain their original rejection status.
The comparisons record actual rustc arguments before and after policy application.
No declared instrumentation or caller argument is silently dropped.

Initial runtime ASAN preparation passed in 71.730s and compiler ASAN preparation
in 536.659s at `35e113a0e5380ebf0902df274d7626e4dd038c5b`; both actual
executables define the expected ASAN symbols. These are preparation results,
not smoke execution or V5 timing passes. Rust sources are unchanged by subsequent
policy corrections. Final retained preparation, real smoke/artifact execution,
representative edit timing, guards and fresh scoped review remain required.


### V0.13 acceptance and measured edit cycle

Implementation candidate `56ce1fc32bb96331b29057180c1591a89d2ae539` is accepted in
[PR #4376](https://github.com/sifr-lang/sifr/pull/4376) under the approved
focused intermediate policy. Independent remediation review is **SATISFIED**,
with no remaining scoped blocker; raw review is externally keyed by that exact
candidate. The earlier two failed reviews remain preserved. Twenty-two policy
controls pass, as do formatting, file-size, area declarations and method-authority
guards. Fourteen direct Cargo comparisons retain caller argument semantics.

The canonical sanitizer smoke executes **two variants, both pass; zero skips and
zero failures**, in 61.772s outer. Runtime and generated
source assertions remain unchanged. Additional direct execution proves the
runtime test actually runs (one passed), and selects the generated executable
from the compiler's build report. Both that executed output and the compiler
selected from Cargo artifact JSON define `__asan_init`, `__asan_report_load8`
and `__asan_report_store8`; hashes and command environments are recorded.

| Diagnostic category | Result | Outer elapsed |
| --- | --- | ---: |
| Retained runtime preparation | Pass | 4.694s |
| Compiler preparation after policy flag-channel correction | Pass; first-party rebuild observed | 345.098s |
| Unchanged compiler preparation baseline | Pass | 9.191s |
| Representative compiler edit preparation | Pass | 215.928s |
| Edited compiler/runtime/generated artifact assertions | Pass | 65.739s |
| Restored compiler preparation | Pass | 177.509s |
| Restored compiler/runtime/generated artifact assertions | Pass | 38.715s |

The controlled edit changes one range-recognition expression from `args.len()`
to equivalent `args.iter().count()`, then executes the unchanged nested-helper
demo assertions with compiler and generated-program ASAN. The parent orchestrator
checks the exact owned diff before restoring original bytes, outside the capped
child process group. Final clean candidate preparation and original smoke-fixture
artifact execution pass after restoration. This experiment makes no permanent
Rust or fixture change. Each category retains the 30-minute cap, storage reserve,
memory reserve and process cleanup receipts.

These diagnostic observations overlap separate lightweight checks and use retained
storage. They are not an idle-host A/B benchmark, full-gate sum or V5 qualification.
The first-party rebuild after changing flag representation has matching Cargo
artifact features/profiles/paths and build-script output facts; its precise dirty
trigger is not proven. V3/V4 own fingerprint tracing and compilation/reuse work.
Do not count that preparation as an unchanged warm build.

A continuation diagnostic ran all 26 canonical runner self-test groups:
**25 pass, one fails**, in 221.673s. The sole failure remains V0.16b's three
missing application-profile entries; no failure was converted to a pass.
Shared-fixture checks account for 194.373s. Source inspection shows repeated
area-adapter input inventories, and an observed open descriptor points into the
vendor input tree. V4 owns reducing repeated work without dropping input identity;
this observation alone does not attribute every second to hashing.

Evidence: external implementation bundle `v013-final-acceptance.json`,
`v013-compiler-edit-experiment/`, the named receipts/artifact proofs, and
`reviews/56ce1fc32bb96331b29057180c1591a89d2ae539/review.md`. The initial sweep-launch import failure,
initial policy test-construction failure, earlier blocked runner execution and
both rejected flag implementations are preserved. Review's later-phase condition
for V0.16b/full gates/V5 is tracked under the approved phase-end policy, not waived.
Next: reconcile V0.16b's exact three entries, then V0.14 and V1–V5. No complete
gate or sub-ten-minute claim is made.


### V0.16b bounded scope and acceptance

V0.13 merged as `1b326c724a46348369c717e8ba8e4529c7ae2cad` through
[PR #4376](https://github.com/sifr-lang/sifr/pull/4376). This inventory-only item
starts from that base. Reconcile the application-profile authority with exactly
`callable_field_call_contexts.sifr`, `join_set_add_blocking_task.sifr`, and
`task_context_reuse_across_spawns.sifr`, added by the earlier V0.11 repairs.
Retain lexical ordering, every existing fixture, full-corpus selection, native-run
assertions and both development/release application profiles. No source fixture,
selector, runtime assertion, executable command or profile membership changes.

Acceptance: preserve the failing baseline and prove the exact three-entry delta;
run DX.10/native selection controls and the full canonical runner self-test;
validate profiles and compare canonical emitted plans before/after; required
runner/area guards and a scoped independent review. Prior execution evidence for
all 732 positive fixtures remains unchanged-source evidence, not a new whole-gate
pass. Full final phase gates and V5 remain required. V0.14 process/preparation work
and V3/V4 input-inventory cost stay separate.


### V0.16b acceptance

Implementation `e113969e0fded7f7c8c8b414a71ee98a50e6021d`,
[PR #4377](https://github.com/sifr-lang/sifr/pull/4377), adds exactly three fixture
names and changes no other application-profile JSON value. The complete inventory
now matches all 732 positive fixtures, in lexical order. The baseline authority
is byte-identical to the failed V0.13 diagnostic; historical failures remain.

Eighteen DX.10/native selection controls pass (1.132s outer). Profile validation
passes (0.636s); six before/after emitted plans are identical: canonical create-pr,
merge, nightly and release, plus create-pr/merge correctness-only. The full canonical
runner self-test passes all 26 groups in 76.698s. Required formatting/file-size,
area declaration and method-authority guards pass in 45.660s. These are functional
validation timings, not matched before/after performance evidence. The first
self-test launch supplied the wrong CLI spelling and executed no tests; its exit2
receipt remains preserved separately from the corrected `--self-test` success.

Scoped independent review is **SATISFIED** on that implementation. Remaining review
conditions (canonical self-test and guards) have now passed on the same candidate.
Evidence lives in the external implementation bundle: `v016b-final-acceptance.json`,
`v016b-before-inventory.json`, `v016b-after-inventory.json`, named receipts, and
`reviews/e113969e0fded7f7c8c8b414a71ee98a50e6021d/review.md`.
V4 owns consolidating inventory maintenance so fixture additions cannot leave
parallel declarations stale; this repair retains the current exact-corpus guard.
Next bounded item is V0.14. Full final phase gates and V5 remain open.


### V0.14a bounded scope and acceptance

V0.16b merged as `5c40661094e51a83394703caad238167f52186e6` through
[PR #4377](https://github.com/sifr-lang/sifr/pull/4377). V0.14a starts from this
base and fixes only preparation dependency ownership. Existing LSP fixtures and
Python bridge archive tests consume the pinned Python area's `.venv`. The current
planner can leave SQL incremental-editor and full crate consumers eligible after
Python example preparation fails, because their environment prerequisite is only
an incidental effect of another area's preparation.

Add one explicit locked environment preparation node when a selected consumer
needs it. Bind the Python area, selected SQL LSP test graph and selected driver/LSP
crate graphs to their actual prerequisite. Keep environment preparation separate
from building/running compiled Python examples. Preserve exact tool/dependency
pins, fixture interpreter authority and caller Cargo storage/flags. Do not create
a fallback environment, weaken missing-environment failures, or remove Python
assertions. Keep independent preparation and execution eligible after a normal
failed prerequisite; safety/resource/cancellation failures retain current stopping
semantics. General process lease inheritance is separately owned by V0.14b.

Acceptance: reproduce the canonical graph's missing dependency; test environment
selection, exact commands/pins/storage, SQL-only and crate-only selection, no-demand
selection, prerequisite failure and independent continuation. Run the full runner
self-test and existing preparation/area setup controls, validate unchanged emitted
selection plans, and required guards. Execute actual locked environment preparation
with unchanged manifest/lock hashes, then the affected LSP and bridge archive test
consumers under their original assertions and the per-category diagnostic cap.
Scoped independent review remains required. Environment bootstrap and compiler
preparation are recorded separately; moving preparation earlier is not a speedup.
Full final phase gates and V5 remain open.


### V0.14a acceptance

Implementation `4ee052487a2fda39ddbd801127a3f51b0c4af4a0` is accepted in
[PR #4378](https://github.com/sifr-lang/sifr/pull/4378). The canonical graph now
has one explicit environment prerequisite for selected Python-area, SQL LSP,
LSP-crate and ignored driver native-archive consumers. Ordinary driver library
tests do not request that environment. A normal environment failure blocks its
dependent preparation and consumers, while independent work remains eligible in
no-fail-fast mode. A later compiled-example failure does not invalidate a
successfully prepared environment or block unrelated SQL/crate consumers.

The callback retains exact project/tool/interpreter pins and caller Cargo
storage/flags, removes the unrelated active environment and explicitly targets
the canonical area's `.venv`. The actual probe checks the existing CPython/GIL
contract and virtual environment prefix. No source fixture, Rust implementation,
runtime assertion, selected command or profile membership changed.

| Named validation category | Result | Outer elapsed |
| --- | --- | ---: |
| Affected preparation/area setup controls | 64 pass | 2.637s |
| Full canonical runner self-test | 27 groups pass | 113.945s |
| Emitted selection-plan comparison | Six plans identical | 0.649s |
| Profile declarations | Pass | 0.649s |
| Required formatting/file-size/area/method guards | Pass | 45.174s |
| Existing locked Python environment validation | Pass; manifest/lock unchanged | 0.690s |
| LSP compiler preparation | Pass | 45.279s |
| Original LSP assertions | 114 pass; zero failed/ignored/filtered | 207.196s |
| Native bridge compiler preparation | Pass | 323.046s |
| Original native archive/bridge assertions | Two pass; zero failed/ignored | 81.236s |

The inner `uv` command took 0.097795s; the outer preparation
receipt includes its harness/verification launcher. Both describe locked
re-validation of an existing environment, **not cold bootstrap**. Prior initial
environment bootstrap remains separately reported. Cargo preparation and test
execution have separate receipts; every category stays under its diagnostic cap.
These overlapping diagnostic observations are not a matched performance comparison
or an integrated full-gate sum. The source-level dependency fix establishes correct
ordering/failure accounting, not a claimed reduction of actual LSP/runtime work.

Independent review is **SATISFIED**, externally keyed by the exact implementation.
Its pending validation conditions have now passed on that candidate. Review's
timing note is clarified by retaining both inner and outer receipts; the raw
review is unchanged. The first implementation's two order-test failures remain
preserved: workspace/fixture fetch retains its original early position, followed
by the explicitly asserted environment step. The later narrowing adds a regression
control against unnecessary ordinary-driver environment preparation.

V3 follow-up, required before final qualification: compiled Python-example setup
and actual area execution still have pre-existing ambient
`UV_PROJECT_ENVIRONMENT` handling that can select a different environment. Align
their environment authority with the actual selected fixture interpreter, preserving
venv identity and pinned dependencies; do not normalize to the base interpreter.
The current environment-preparation protection does not imply that downstream
follow-up is already fixed. Conservative compile-node blocking remains visible;
finer independent preparation can be evaluated within V4 without dropping consumers.

Evidence: external `v014a-final-acceptance.json`, `v014a-baseline-graph.json`,
`v014a-final-graph.json`, `v014a-actual-environment/`, all named receipts, and
`reviews/4ee052487a2fda39ddbd801127a3f51b0c4af4a0/review.md` with timing/follow-up context.
V0.14b is next. Full final phase gates and V1–V5 remain open.


### V0.14b bounded sequence and V0.14b1 acceptance contract

V0.14a merged as `d594df65e44b958b1da195eb42d1f7f794c7707d`
([PR #4378](https://github.com/sifr-lang/sifr/pull/4378)).
V0.14b1 starts from that base and fixes E2E suite lease inheritance. The observed
parallel native-cache failure is caused by globally clearing CLOEXEC on the
suite descriptor: unrelated Cargo/fixture children inherit it and prolong ownership.
V0.14b2 owns the separate migration of production native family, publication,
resolution, artifact and semantic-storage consumers. V0.14b is incomplete until
both steps preserve their original ownership contracts.

V0.14b1 scope: add a Unix command-bound lease primitive that leaves parent
file descriptors close-on-exec, explicitly bind the E2E suite lease to its Cargo
build/retirement and fixture commands, and pass ownership through normal and
failure-isolation harness paths. Preserve Windows' existing behavior; foreign
execution remains outside this local goal. No test serialization, weakened
lease protection, implicit global registry, changed corpus, skipped assertions,
or cache-authorized Cargo bypass is permitted. The existing production entry
lock policy remains visible and owned by V0.14b2 during this bounded migration.

Acceptance: prove unrelated live children do not retain the suite lease while
intended live children do; prove protection after owner death, release after
child exit, independent simultaneous owners, command disposal/spawn failure,
and unchanged parent CLOEXEC state. Run the original native-cache tests with
parallel workers, affected harness/isolation/lifecycle controls, full positive
E2E assertions, formatting/file-size and affected method/area guards, then scoped
independent review. Keep preparation separate from execution and preserve every
failed/interrupted attempt under the existing 30-minute diagnostic cap. Full
final phase gates and V5 remain required.


### V0.14b1 broader diagnostic and V0.16c owner

The first E2E diagnostic candidate `7cf3bb0930c78bc1793e5d0071c7b901166c9790`
prepared its binary in 91.462s, passed ten native-cache controls with eight workers
in 7.691s, and passed all 732 original positive fixtures in 126.256s (suite 119.70s).
The other E2E batch took 250.901s: 51 passed, two failed, three explicit helper/session
qualification tests ignored, and eleven tests separately selected in the native-cache
and positive categories. These observations do not qualify the complete phase.

One failure is preparation: the offline diagnostic could not fetch its newly
published exact candidate revision for the relocated portable project. Repeat that
original assertion with network preparation enabled and unchanged locked resolution;
retain the offline failure. V3 owns explicit input-bound preparation for this graph.

**V0.16c (pending, next after V0.14b1):**
`test_structured_lowering_error_is_canonical_before_native_build` uses the positive
`enum_value_property.sifr` fixture as an expected lowering failure. The test body is
unchanged from the V0.14b1 base, and V0.7 already proved that fixture should compile
and run. This is a stale negative control, not a lease regression. Replace its
reliance on an unfixed language feature with a deterministic control of the actual
canonical lowering-error boundary, retaining proof that nonzero lowering errors
cannot become successful native preparation. Keep the positive fixture's successful
compilation/runtime assertions. Do not delete or skip the negative coverage. This
unrelated control remains failed in the broad diagnostic and required before final
gates; V0.14b1 acceptance must name its lease/fixture/isolation checks individually.

Initial V0.14b1 review was NOT SATISFIED: two E2E release assertions retained the
same concurrent pre-exec descriptor race already demonstrated in the primitive
tests. Remediation bounds only test-side release waits, retains structured
WouldBlock errors and immediate busy-owner rejection, protects test child cleanup,
and consolidates cached/standalone fixture command preparation. The real owner-death
and 30-second live-child negative controls remain. Fresh scoped review and affected
validation are required after that remediation. Earlier failures and the guard run
invalidated by an overlapping owned commit remain preserved; frozen guard evidence
supersedes that invalid attempt only, without reclassifying it as a pass.


### V0.14b1 focused acceptance

[PR #4379](https://github.com/sifr-lang/sifr/pull/4379) accepts implementation
`ed625d0816507a86274e2847572863905ca04e45` under the approved intermediate policy.
The Unix E2E suite descriptor stays close-on-exec in its owner. Each intended Cargo
build/retirement and fixture command explicitly receives an owned descriptor whose
flag changes only in that child. Failure-isolation paths carry the same ownership.
Unrelated concurrent child commands cannot keep the suite leased after exec. Windows'
existing handle/Job behavior is unchanged; no foreign execution is claimed.

The process controls retain actual killed-parent protection, independent concurrent
owners, command reuse, multiple leases, failed spawn/disposal and release on child
exit. A one-second test-only release wait accounts for transient pre-exec descriptor
copies; it cannot conceal the 30-second inherited sleeper. Production contention
still returns immediately. All non-contention errors retain their kind and fail
the test immediately; the foreign-owner case requires its specific diagnostic.
The guarded native children are killed and reaped even after assertion failure.

| Final-candidate category | Result | Outer elapsed |
| --- | --- | ---: |
| E2E compiler preparation | Pass | 47.733s |
| Cache-storage ownership controls | 10 pass; eight workers | 1.694s |
| Original/new native-cache controls | 10 pass; eight workers | 6.198s |
| Harness behavior controls | 19 pass | 5.189s |
| Affected isolation/worker controls | Three pass; unrelated V0.16c remains separately failed | 6.183s |
| Fixture isolation/failure/deadline controls | Four pass | 7.686s |
| Full original positive E2E corpus | 732 pass; zero failed | 122.756s |
| Original locked portable relocation assertion | Pass with network preparation | 21.699s |
| Formatting/file-size/area/method guards | Pass; 17 method guard mutation controls | 46.668s |

Every named final-candidate receipt reports no safety stop, leftover process group
or source mutation. The full positive corpus still executes all original assertions;
Cargo remains the freshness authority. Profile selections, generated source and
application profiles are unchanged. These diagnostic timings are not a matched
speedup measurement or a complete gate total; the repair removes incorrect lease
retention and makes concurrent controls reliable.

Independent remediation review is **SATISFIED**, externally keyed by the exact
candidate; its named execution conditions are now satisfied. The initial NOT
SATISFIED review, original failed storage attempts, permission failures, inventory
failure, invalidated overlapping-commit guard run, and broader two-failure E2E batch
remain preserved. The frozen successful guard runs do not relabel those attempts.

The broader batch's portable assertion was rerun successfully with its actual
published revision and locked resolution. The stale lowering negative control is
still a failure owned by V0.16c, not an accepted skip or a phase-level pass. V0.16c is
next; V0.14b2 and V1–V5 remain open. V3 also retains explicit preparation ownership
for the portable graph. No complete create-PR/merge gate or sub-ten-minute phase
qualification has been claimed.

Evidence: `v014b1-final-acceptance.json`, all named raw receipts and command specs,
`v014b1-fork-transition-proof.json`, `v014b1-guard-reconciliation.json`, both raw
reviews, and `reviews/ed625d0816507a86274e2847572863905ca04e45/review.md`.


### V0.16c bounded scope and acceptance

V0.14b1 merged as `e58a6355e4c44eff9dcb1550827a03d3de1570c9`
([PR #4379](https://github.com/sifr-lang/sifr/pull/4379)). V0.16c starts there.
Its root cause is a negative test depending on a temporary compiler deficiency:
V0.7 repaired the positive enum fixture, but the old harness test still demanded
its failure. The production lowering-error rejection is unchanged and remains
required. No compiler behavior, fixture expectation, supported feature or runtime
assertion may be weakened to make this test pass.

Replace that dependency with private driver tests of the actual
`resolve_single_file_metadata` boundary. Generate real valid codegen output, then
exercise statement, expression and item lowering errors separately and together,
checking canonical diagnostics and refusal to continue native preparation. Include
zero-error success and a warmed-resolution control so a successful prior resolution
cannot bypass error admission. Keep an E2E assertion that actual compiler diagnostics
propagate as fixture failure before native planning. The repaired enum stays in the
full positive corpus with all original assertions. No production test hooks or new
public APIs are needed.

Acceptance: focused driver boundary controls, affected E2E diagnostic/isolation
checks, formatting/file-size/area/method guards, then the complete ordinary E2E test
binary with its original corpus/runtime/harness assertions. Record explicit helper
and separately qualified session ignores as such; never call them executed unless
their parent control invoked them. Preserve the original stale failure and any new
failure, run each diagnostic category under the 30-minute cap, and retain compiler
preparation separately. Network preparation may fetch the published candidate's
locked portable source graph. Obtain scoped independent review. Full final phase
gates and V0.14b2/V1–V5 remain required.


### V0.16c focused acceptance

[PR #4380](https://github.com/sifr-lang/sifr/pull/4380) accepts implementation
`3aa646507003b44fdd58e5b238c97b744beedd44` under the approved intermediate policy.
The lowering-error control no longer depends on the repaired positive enum fixture.
Private driver tests exercise each real production-boundary counter independently
and together, before and after successful resolution, with exact diagnostic code,
severity, count and message. The E2E control preserves actual compiler diagnostics
as a fixture failure before native planning. Both strict no-Cargo assertions run
in isolated exact-test subprocesses; unrelated parallel tests cannot pollute their
process-wide trace. No production behavior, public API or positive fixture changed.

| Final-candidate category | Result | Outer elapsed |
| --- | --- | ---: |
| Driver test preparation | Pass | 63.745s |
| Driver boundary | Pass; all eight counter combinations and zero-error control | 6.189s |
| E2E preparation | Pass | 72.941s |
| Isolation controls | Four pass; eight workers | 3.689s |
| Formatting/file-size/area/method guards | Pass | 55.744s |
| Complete ordinary E2E through Cargo | 64 pass; zero failed; three explicit ignores | 223.311s |

The complete ordinary E2E invocation has no filter or skip. It executes all 732
positive, 569 negative and six runtime-failure fixtures, eight native parity cases,
structured-lowering ratios, emitted-code safety, portable relocation, cache,
isolation and harness assertions. The positive suite itself took 113.638s. Two
ignored subprocess helpers are invoked and asserted by their parent controls; the
separately qualified full-corpus session-equivalence oracle is not executed here.
All accepted receipts have no safety stop, leftover process group or source mutation.
These retained-build diagnostic times are not a matched speedup measurement or a
complete verification-gate total.

Independent remediation review is **SATISFIED**, keyed externally by the exact
implementation candidate. Its complete ordinary-E2E condition is now satisfied.
The initial review correctly rejected unisolated process-wide Cargo tracing; it
remains preserved. The capacity-blocked preflight is not execution. A subsequent
direct-binary diagnostic used the workspace working directory instead of Cargo's
crate directory: four fixture-dependent tests failed and some missing-directory
branches returned early. That attempt remains failed/incomplete; its nominal
passes are not corpus coverage. The corrected complete Cargo invocation above
supersedes it without changing its classification.

Capacity recovery archived only checksum-verified obsolete test executables with
no live consumers. Their raw receipts and bytes are retained. Evidence includes
`v016c-final-acceptance.json`, command specs and receipts, both reviews,
`v016c-full-e2e-launch-diagnosis.json`, and
`reviews/3aa646507003b44fdd58e5b238c97b744beedd44/review.md`.

Follow-ups: V4 owns fail-closed missing-corpus admission, bounded driver test-child
execution and measured compiler-identity/test compilation costs. V3 retains the
remaining reusable native fixture ownership migration. V0.14b2 production child
leases are next. V1–V5 and the complete final create-PR/merge gates remain open;
this acceptance does not establish the whole phase's sub-ten-minute objective.


### V0.14b2 bounded scope and acceptance

V0.16c merged as `e2efe1ef30b6b855f722f79175a36f2111e1aa2f`
([PR #4380](https://github.com/sifr-lang/sifr/pull/4380)). V0.14b2 starts there.
Production Unix family, publication, resolution, artifact and semantic metadata
leases still inherit into unrelated commands. Apply the already qualified
command-owned descriptor primitive to their intended native consumers; keep
semantic in-process ownership close-on-exec. Preserve the existing Windows
handle/Job policy and make no foreign execution claim.

Use a small explicit lease collection in scoped native Cargo policies. Carry
family/publication ownership through metadata preparation, native probes/builds
and portable resolution; retain the prepared resolution during its consumers.
Give cached executable artifacts a command constructor that retains the actual
entry, and migrate CLI and test callers. Bind generated-test execution to its
captured artifact. Dispose completed command/policy/resolution ownership before
the existing native-family release boundary. Do not put process-local descriptor
values into semantic/cache identity, omit identity inputs, serialize unrelated
commands, or add a global ownership registry.

Acceptance: focused storage/command ownership and original driver workspace/prune
controls, real killed-parent and unrelated-live-child cases, multiple owners,
spawn/disposal failure, retained family/publication/resolution/artifact ownership,
and release before independent finalized-artifact execution. Run original native
reuse, lock-mode, portable relocation, cached executable and generated-test
assertions. Use a broad ordinary driver batch where it diagnoses cross-test
interaction efficiently, keeping preparation separate and each diagnostic under
the 30-minute cap. Run the applicable guards and independent scoped review.
The final complete phase gates remain mandatory.

The test-only reusable native source fixture is a distinct input owner. Its
ambient inheritance is explicitly retained as required V3 fixture-lifecycle work,
not claimed repaired by production migration. Avoid introducing a public semantic
CompilerContext ownership API solely for that fixture; migrate its test-worker
ownership under V3 with the original native assertions. No source fixture, runtime
expectation, platform qualification or profile selection may be weakened here.


V0.14b2 audit clarification: the full unsafe inventory diagnostic found missing
records for the previously merged cache command-lease primitive and isolated
permissive-umask cache controls, as well as this item's new lifetime controls.
Reconcile those cache/driver contracts with their actual local source proofs as
part of the lease migration. Preserve unchanged owners and complete cross-record
comparison. This is safety admission, not merely copying new fingerprints.

The same audit found pre-existing stale code-proof references for the three SQL
guest module allowances. Assign their record-only reconciliation to **V0.16d**,
after V0.14b2 and before final phase gates. Do not claim a complete unsafe-guard
pass while those references remain invalid. B2 requires every cache/driver site
and all new obligations to validate; a full audit must expose only the separately
recorded SQL proof failures. The twelve unsafe-guard mutation controls remain
mandatory. The first full diagnostic took 185.383s including those controls;
V4 owns investigation of repeated-contract comparison cost without weakening its
cross-record rejection semantics.


V0.14b2 semantic fixture ownership correction: the complete services batch found
one pre-existing fixture failure under umask 0002. The override-retry test passed
its generic temporary parent as an already-created cache root, which the unchanged
private-cache check correctly rejected. Use an explicitly owned cache child, as
other provider fixtures do; retain every corrupt/stale override assertion and the
production permission check. This is the metadata ownership fixture exercised by
this item, not a relaxation of cache admission. Run the original regression under
both 0002 and 0077 and all semantic-services tests. Preserve the initial 25/26
receipt and label the restrictive-umask probe diagnostic-only. No runtime metadata
logic, cache identities, platform policy or test selection changes.


### V0.14b2 focused acceptance

[PR #4381](https://github.com/sifr-lang/sifr/pull/4381) accepts implementation
`a50013da171beb59ebe152e07ae6fed398934373` under the approved intermediate policy.
Unix production leases now follow their consuming Cargo/native commands explicitly.
Family, publication and prepared-resolution ownership covers metadata, probes,
builds and portable materialization. Finalized executables receive their artifact
lease through `CachedBinaryArtifact::command`. Generated tests release mutable
family ownership before executing protected immutable artifacts. Semantic metadata
keeps its owner in process. Parent descriptors stay close-on-exec; descriptor
values never enter cache identity. Windows policy is unchanged.

Real-Cargo, killed-owner, unrelated-live-child, multiple-owner, failed-spawn and
command-disposal controls pass. Original native reuse, cache, pruning, generated
test, CLI and portable assertions remain. The metadata fixture now creates its
own private cache child, preserving production permission checks. The obsolete
Unix `entry_lock` re-export is test-only; final preparation has no warnings/errors.

| Final implementation category | Result | Outer elapsed |
| --- | --- | ---: |
| driver preparation | Pass | 34.736s |
| cli preparation | Pass | 60.322s |
| e2e preparation | Pass | 17.377s |
| cli lock mode | 6 passed, zero failed/ignored | 76.786s |
| production controls | 3 passed, zero failed/ignored | 5.186s |
| workspace ownership | 10 passed, zero failed/ignored | 6.688s |
| native reuse | 7 passed, zero failed/ignored | 17.202s |
| generated runner | 4 passed, zero failed/ignored | 8.688s |
| bridge archives | 2 passed, zero failed/ignored | 79.229s |
| storage | 10 passed, zero failed/ignored | 1.184s |
| semantic 002 | 26 passed, zero failed/ignored | 43.713s |
| semantic 077 | 26 passed, zero failed/ignored | 42.207s |
| portable | 1 passed, zero failed/ignored | 21.198s |
| guards | Pass | 59.206s |

These categories overlap; their counts/times are not a complete gate total or a
matched speedup measurement. The storage count includes a helper whose child
assertions are exercised by parent controls. An earlier complete ordinary driver
run at `f9df2d0d762cfa2e5c9898cc900e2dbc84585222` passed 741 tests with 81 existing
integration/helper ignores in 790.704s. It remains that observed diagnostic,
not a final-candidate run. Every table receipt has no resource stop, leftover
process group or source mutation.

The complete unsafe audit at `971ee00e7a1717106cda84c5f0983258ded7ff00` remains
**failed** on exactly twelve pre-existing SQL guest proof references. It took
201.034s; all twelve mutation tests passed and complete cross-record comparison
reported no cache/driver/new-site error. Fresh final-candidate local validation
finds the same twelve errors. Safety sites, fingerprints, operations/evidence,
inventory contracts, auditor code and schemas are unchanged.
`v014b2-a500-unsafe-reconciliation.json` binds reuse of the pure cross-record checks
to identical inputs. V0.16d must repair the SQL proofs before final gates.

Independent review is **SATISFIED**; its named merge conditions are now met.
The initial code review and bounded remediation reviews remain externally keyed
by SHA. Final review: `reviews/a50013da171beb59ebe152e07ae6fed398934373/review.md`.
Acceptance, raw commands, receipts and output hashes are retained externally in
`v014b2-final-acceptance.json` and its named directories.

Preserved failures include the unsupported Cargo shell-alias control, incomplete
policy records, the metadata fixture's permissive-umask failure (25/26), missing
CLI sysroot selection, and portable relocation stopped at the disk reserve.
Two CLI failures cascaded from the first poisoned test lock before their native
assertions; the corrected final batch executes all six cases. The restrictive
umask probe remains diagnostic-only. Portable interruption was 19.220s at the
disk reserve, not a greater-than-30-minute classification.

CLI qualification explicitly selects the candidate source sysroot: these cases
change working directory and the owned Cargo target is outside source ancestry.
V3 owns that canonical launch input. Portable qualification retains its original
explicit fetch/locked-check contract; no additional offline guarantee is inferred.
Capacity recovery archived checksum-verified obsolete executables and older
incremental generations, retaining the newest generation and lock files in each
context. The reserve was not reduced; failed outputs were preserved.

Next: V0.16d SQL proof records, then V1–V5. V3 retains reusable source-fixture
child ownership and Python/launch preparation identities. V4 retains pure
safety-audit work, durable-history cost, compilation/capacity accounting and
fail-closed corpus admission. Complete final phase gates and measured
sub-ten-minute acceptance remain open.


### V0.16d bounded scope and acceptance

V0.14b2 merged as `87693e0377ae1d6bdcc3f64fde01370b08a35897`
([PR #4381](https://github.com/sifr-lang/sifr/pull/4381)). Its complete unsafe
inventory scan identified twelve invalid code proofs in three existing SQL guest
allowance records. V0.2 added provider diagnostic projection to each `analyze`
implementation; the inventory still quoted the previous function body.

Reconcile only the MySQL, PostgreSQL and SQLite guest allowance code references
with the current complete `analyze` bodies and their context fingerprints. Retain
SQL ownership, bounded-allowance rules, operation fingerprints and safety contracts.
The request remains owned while its clone enters analysis; projection borrows
owned request/response values and serialization returns an owned byte vector.
The generated WIT ABI boundary and allocation-transfer obligations are unchanged.
This is source-proof reconciliation, not permission to bless a weaker contract.
No Rust, test selection, dependencies, schema or runtime behavior changes.

Acceptance: the complete unsafe audit, including all cross-record comparisons,
must pass with zero omitted segments or exceptions. All twelve mutation controls,
file-size/whitespace checks and scoped independent review must pass. No compiler
build is needed for these record changes. Preserve the earlier twelve-error audit
as failed evidence; final integrated phase gates and V1–V5 remain open.


### V0.16d focused acceptance

[PR #4382](https://github.com/sifr-lang/sifr/pull/4382) accepts record-only candidate
`8a5491b84f01657ceff76392cfe114af4fd28670`. All twelve SQL guest code references now
quote the current complete `analyze` body with its context fingerprint. SQL ownership,
contracts, obligation rules and operation proofs are unchanged; no Rust changes.

The local proof precheck passes in 28.163s. Complete safety auditing of **382 sites
across all three partitions**, all **twelve mutation controls**, file-size checks
and whitespace checks pass in **191.231s**. The complete run retains every
cross-record comparison with no exceptions. These overlapping diagnostic checks
are not a whole-gate total. Receipts show no resource stop, leftover group or source
mutation. Independent review is **SATISFIED**, with no blockers; it inspected the
actual request/response ownership and diagnostic projection, not just hashes.

Raw review: `reviews/8a5491b84f01657ceff76392cfe114af4fd28670/review.md` in the external
implementation bundle. `v016d-final-acceptance.json` binds command specs, output
hashes and receipts. Earlier twelve-error audits remain failed historical evidence.
No compiler build was required. A non-blocking wording clarification about typed
request/response ownership remains with the SQL record owner. Next is V1's measured
mandatory build path; V1–V5 and complete final phase gates remain open.


### V1a bounded feasibility investigation

V0.16d merged as `fbb7015dd9c42025bc9279c0dd52e11b778e266d`
([PR #4382](https://github.com/sifr-lang/sifr/pull/4382)). Begin with a diagnostic,
not a production profile change: keep the normal development profile and compare
one candidate that sets `sifr_codegen`, `sifr_lowering` and `sifr_type_system` to
optimization level 1 instead of their current level 2. Other optimization levels,
features, packages and semantic/runtime checks remain unchanged. Use the same
explicit LTO-off and debug-off settings for both measured variants; these are
recorded experimental controls, not unqualified defaults.

Run a fresh current-source normal SQL build and candidate clean A, unchanged A
and independent clean B. Keep the canonical locked/offline package selection,
constant within-protocol path maps, exact linked executable selection and byte
comparisons. Record Cargo units, freshness, profiles, full command settings,
resources, preparation and cleanup boundaries. Retain artifacts needed for
matched runtime probes in owned external directories, labeling that retention;
this diagnostic is not a complete gate or final cleanup qualification.

Use isolated owned targets on the same filesystem with four Cargo jobs only when
effective CPU/memory admission permits the allocation and reserve. Each diagnostic
stage retains the 30-minute cap, memory/disk protection and source freeze. Do not
repeat a failed unchanged build. Compare metadata validation with the real linked
SQL graph in both variants, using identical supplied diagnostic metadata inputs,
and keep actual product/installed qualification separate and mandatory. No profile
is accepted from build time alone. If this simple candidate cannot fit the complete
budget without a runtime regression, proceed to the canonical semantic/native
ownership investigation rather than accumulating arbitrary package overrides.


### V1a diagnostic result — profile tuning does not meet the budget

Current-source candidate `5011e4ae8fdd792be2ad7c379f260f6d95ee0ccb` was measured
with three Cargo workers, the same filesystem, locked/offline resolution, explicit
LTO off and debug off. The four-worker attempt failed admission before compilation:
its effective memory did not cover the requested allocation plus the unchanged
2 GiB reserve. Both actual variants therefore used three workers.

| Diagnostic stage | Outer wall time | Cargo units / fresh units |
| --- | ---: | ---: |
| Normal profile, one independent clean control | 512.007s | 498 / 0 |
| Three first-party overrides at opt1, clean A | 505.989s | 498 / 0 |
| Same candidate, unchanged A | 8.189s | 498 / 498 |
| Same candidate, independent clean B | 514.568s | 498 / 0 |

All three exact linked SQL executable hashes agree within the candidate protocol.
All seven canonical artifact-selector mutation controls pass. Resolved package,
target-kind and feature inventories agree; only the three intended package
optimization settings differ. Candidate build-stage wall times sum to **1028.745s
(17m 09s)**. This sum excludes cleanup and is not a whole-gate time. The normal
control is a single clean build, not a complete baseline reproducibility protocol.
Sequential build order and a single control do not establish a small speedup: the
two candidate clean builds straddle the control. This profile is **not adopted**.

Matched runtime probes link the actual SQL build libraries, use the same supplied
diagnostic identity and the same **6,559,402-byte** metadata artifact, and run in
fresh-process ABBA order after separately recorded preparation. All metadata hashes
and recorded inventories/read counts agree. Six complete validations per variant
have medians **2.513624s normal / 2.524326s candidate** (ratio **1.0043**). These
probes do not qualify actual product/profile identity, installed artifacts or all
runtime semantics. No material runtime benefit is demonstrated.

Twelve diagnostic stages pass without resource stops, leftover process groups or
source changes. Preserve both earlier failures: four-worker admission and a probe
helper that incorrectly demanded globally unique library names across different
transitive package versions. The corrected helper selects only its three required
libraries by exact Cargo package identity; neither failure required repeating a
successful build. Raw commands, partial failures, Cargo JSON/timings, probe
preparation/runtime logs and receipts remain in the external implementation bundle:
`v1a-final-diagnostic.json` and `v1a-sql-profile-trial-j3/analysis.json`. Targets and
probe artifacts are retained for audit, explicitly outside final cleanup acceptance.

The unchanged-build cache works in this path. The measured cost is compiling the
full dependency graph twice, including the emitter. Continue V1's semantic/native
ownership work. V1–V5, the complete final create-PR/merge gates and sub-600s acceptance
remain open; no tests or valuable qualification obligations are removed.

### V1b1 bounded implementation — shared stdlib artifact data

Move the immutable `StdlibCode`, `StdlibEmissionCode`, `StdlibRustSource` and their
signature representation to a small lower artifact-data owner that depends on HIR
and types, not the emitter. Keep `StdlibEmissionView`, its syntax-session ownership
and all rendering algorithms in codegen. Update actual consumers and compiled input
identities. Preserve data layouts, declaration/order behavior, defaults, ownership,
serialization, emitted bytes and existing test assertions. Treat the new workspace
lock entry as an intentional first-party dependency change; do not alter registry
versions or separately pinned verification graphs.

This is one dependency-boundary prerequisite, with no claimed runtime gain. Native
capability selection, metadata role/schema changes, semantic bootstrap separation
and the SQL consumer switch are subsequent explicit V1 items. Do not introduce a
second frontend, compatibility fallback, cache-result substitution or test removal.

Named intermediate validation: full codegen library tests; compiler-services library
tests; affected driver metadata producer/reader and stdlib regressions; the complete
positive E2E inventory through its canonical Cargo entrypoint; formatting, file-size,
method/safety policy checks affected by the move, whitespace and changed-link checks.
Run focused checks first and diagnose preparation failures before repeating. Retain
build targets, inspect capacity before preparation, freeze the candidate for named
validation and independent review. The phase-end full gate policy remains unchanged.

### V1b1 shared stdlib artifact ownership — accepted implementation

Implementation candidate `cc52c0ed0cc9945fc151caa77b62a7fcadc199a1` in
[PR #4384](https://github.com/sifr-lang/sifr/pull/4384) moves the checked stdlib
records and signature aliases to `sifr_stdlib_artifact`. Its resolved dependency
closure contains IR and types and excludes codegen, compiler services and the
driver. Codegen retains the emission view, syntax-session lifetime and rendering.
All consumers and compiled identities use the new data owner. The moved record
definitions are unchanged apart from documentation, and `StdlibRustSource` is an
exact byte move. No runtime or full-gate improvement is claimed for this prerequisite.

The maintained first-party inventory and both affected lock graphs are coherent.
The root lock preserves all 858 existing external package records; the independent
fuzz lock preserves all 234. The latter also restores an already-declared codegen
`serde_json` edge and updates only the approved lock hash in the verification
policy. No external version, checksum or official-registry approval changes.
Actual fuzz `cargo metadata --locked --offline` resolution passes. This is a
first-party graph repair under the approved pinned-verification policy, not a
release-version alignment of the independently owned tools.

Named validation on that implementation candidate:

| Check | Observed coverage | Outer seconds |
|---|---|---:|
| Codegen preparation | Retained target | 2.188 |
| Codegen library | 1,780 passed | 17.189 |
| Compiler-services preparation | Retained target | 2.684 |
| Compiler-services library | 26 passed | 43.715 |
| Driver preparation | Retained target | 2.687 |
| Metadata producer | 15 passed; subprocess helper invoked by parent controls | 170.785 |
| Metadata reader | 9 passed; full corpus explicitly executed below | 40.721 |
| Stdlib regressions | 56 passed | 47.215 |
| E2E preparation | Retained target | 4.687 |
| Positive native E2E | 732 selected, 732 outcomes, 732 passed | 120.250 |
| Exact source/metadata Rust corpus | All 732; no selection, byte/source-map equality | 112.243 |
| Format/size/area/method/safety checks | Includes 17 method and 12 safety controls | 248.305 |
| Source dependency direction | Passed | 7.141 |
| Manifest and dependency contracts | 108 passed | 13.149 |
| Structural and resolved-graph checks | Passed; includes actual fuzz lock resolution | 1.642 |

These are command durations from concurrent correctness work, not a sum that can
be presented as one gate's wall time. Producer/reader helper and corpus exclusions
are reconciled explicitly, not counted as silent passes. The 732-case emitted
source corpus remains retained as a baseline for the subsequent semantic/native
separation.

Evidence is retained outside Git in `v1b1-final-acceptance.json`, its raw command
specifications/receipts/output, `v1b1-pinned-corpus-output/coverage.json`, and the
candidate-keyed independent review. Historical import compilation errors, stale
method fingerprints, manifest inventory/count failures, the initially incomplete
fuzz lock, resource-admission refusals and disk-reserve cancellation remain
preserved. The short exact selector that ran zero tests is explicitly rejected;
only the fully qualified entrypoint with all 732 outcomes is accepted. Earlier
candidate evidence is never relabelled as this candidate's execution.

Resource maintenance archived only obsolete session-owned outputs after checking
consumers and verifying copied bytes. Current compatible build artifacts and all
failure evidence remain retained. Both normal and CLI feature graphs were already
prepared. The final verification-only repair changed no Rust/Cargo/toolchain input;
its work allocation was 4 GiB, and the 20-GiB disk and 2-GiB memory
reserves remained enforced. An inherited numeric field in the preceding pinned
admission receipt disagreed with its 6-GiB allocation check; the explicit
clarification preserves the raw record and corrects that accounting.

Scoped remediation review is **SATISFIED**, with no blocking findings, at
`reviews/cc52c0ed0cc9945fc151caa77b62a7fcadc199a1/review.md` outside Git. The earlier review correctly blocked on
missing coverage/profile membership and the moved-owner GCQ producer fingerprint;
its raw report is retained under `reviews/26d61f8fdb716565ea3f1cf1b83325ea9ffa83a1/`.
Both defects are repaired. All four standard profiles enroll the owner, the
resolved compiler configuration count remains 17, and all seven supported emitted
plans preserve every existing selection. The complete runner self-test, 23
profile/allocation/scope controls, four generated-quality controls (including
same-mtime content mutation), and the permanent dependency-direction controls pass.
The additive artifact test harness builds and contains zero standalone tests; its
behavior is exercised by the existing codegen, metadata and full corpus checks.
This is not the rejected zero-fixture E2E invocation.

The broader coverage-area run is still **failed**, solely on five taxonomy
complaints in four baseline files. Membership, readiness, profile assignment and
negative-control cases passed. `v1b1-reviewed-contract-reconciliation.json` binds
the unchanged validator and complained-about files to base `9d85202e`. The full
failed area and its raw output remain retained; V0.17 below owns its repair before
final acceptance. No Rust implementation changed during the inventory/cache-key
remediation, but every named Rust/native check was nevertheless rerun on the final
implementation candidate.

Follow-ups stay in their named phase owners:

- V4: changing only verification-policy JSON changed the production manifest
  source token and rebuilt manifest/import/lowering/codegen units. Separate
  test-only identity inputs without weakening actual compiled-input coverage.
- V4: normal and CLI preparation resolve distinct transitive feature variants.
  Establish which variants are required before attempting to unify them.
- V3: GCQ producer fingerprinting already omitted IR and type-system inputs
  before this item. Repair complete producer keys in that owner with mutation
  controls; this item restores coverage of the newly moved owner only.
- V3: the preceding run retired 185 old group IDs and built 186 different IDs;
  there was no overlap, so it does not demonstrate that TTL removed reusable
  current outputs. The final run reused all 186 current native artifacts after
  Cargo validation and still executed every fixture assertion. This is a reuse
  observation across different compiler candidates, not an optimization trial.

V1's semantic/native boundary, V2–V5, the complete final create-PR/merge gates and
sub-600-second acceptance remain open.

### V0.17 bounded repair — coverage taxonomy precision and record ownership

The complete coverage-area sweep on V1b1 found five complaints in four unchanged
baseline files. Register this as a separate repair before the next compiler
architecture item. Preserve the failed broad-area evidence and the accepted
V1b1 implementation; do not rewrite historical statuses.

Investigate the exact matching expressions, then distinguish technical vocabulary
from delivery labels. The observed matches are `compiler capability evidence` in
the builtin-input schema, `contract_planning_tests` in two Rust module declarations,
`Item 5` in the SQL tooling reference heading, and the planning-record link in the
historical validation-entry inventory. The schema complaint is the technical
phrase, not its adapter identifier. Keep schema validation rules, independently
pinned tool graphs, Rust code and test selection unchanged. Retain narrowly scoped
technical vocabulary with negative controls proving an unrelated delivery label
on the same line is still rejected. Replace the numbered SQL heading with its
technical purpose. Put the historical execution-planning inventory under the
planning/review tree, preserving its content and correcting all changed links;
do not erase historical observations or change the other active plan's status.

Named validation: taxonomy self-tests and the complete taxonomy scan; all five
coverage-area cases, including its existing negative controls; profile validity,
file-size, whitespace and changed-link checks. Prove schema/manifest/lock and
compiler inputs remain unchanged. Use the approved focused-intermediate policy
and a scoped independent review; no compiler rebuild is required if this remains
verification policy and documentation work only. Complete final create-PR/merge
and V5 acceptance remain mandatory and open.


### V0.17 acceptance — precise taxonomy and preserved planning record

PR [#4385](https://github.com/sifr-lang/sifr/pull/4385) accepts implementation
`e85c572e802d682fc812018dc24821a74cb970a0`. The two exact technical tokens are
accepted with same-line negative controls; the numbered SQL heading is descriptive
and the historical inventory is preserved in the planning/review archive, with
both relative links corrected. Rust code, schema bytes, independently pinned
verification graphs, profiles and the other active plan's status are unchanged.

All five coverage-area cases, taxonomy self-tests, profile validity, file-size,
whitespace and changed-link checks passed. The complete bounded validation took
74.181 seconds, with no resource stop, leftover process group or source mutation.
The unchanged-input and content-preservation assertions passed. No compiler build
was needed for these policy/documentation-only changes.

Evidence: `v017-final-validation/{spec.json,receipt.json,stdout,stderr,resources}`,
`v017-validation-details.json` and `v017-coverage-area-result.json` in the external
LOCAL-GATE evidence root. Independent review is **SATISFIED**, with no blocking
findings, under `reviews/e85c572e802d682fc812018dc24821a74cb970a0/`. Suggestions
about still narrower vocabulary matching and spelling consistency are optional
follow-up policy work; they do not establish a regression in this candidate.
The original failed V1b1 coverage-area evidence remains failed and retained.

This closure changes only this phase record. V1 semantic/native separation,
V2–V5, complete integrated create-PR/merge gates and the ten-minute goal remain
open. No runtime gain is attributed to the taxonomy repair.


### V2a bounded implementation — invocation-local metadata validation facts

Sequence this V2 subitem before the remaining V1 separation work to reduce the
measured repeated validation cost of later producer and consumer checks. This
is a sequencing change within the approved whole-phase scope, not completion of
V1 or a replacement for its architecture/feasibility acceptance.

The existing implementation reads, hashes, canonically decodes and field-checks
shared records repeatedly while validating each complete-store graph root. Retain
only successful per-record reference/field-validation facts in one bounded,
invocation-local owner. Keep the immutable captured input as authority. Preserve
per-root traversal order, colors, cycle/depth/work checks, direct-reference kind
checks and first-error diagnostics. Do not retain graph verdicts or share facts
between stores or invocations. Leave lazy consumer decoding/retention accounting
separate. Eviction or an oversized fact must recompute, never skip validation.

Acceptance: measure repeated reads and complete validation on the exact retained
real metadata fixture before/after; compare its portable digest. Add hit/miss,
eviction and oversized-record controls; compare exact diagnostics for malformed
encoding/fields, absent/wrong-kind references, cycles, depth and work limits.
Prove a later stricter root cannot inherit an earlier root's success. Preserve
unchanged-path/mtime corruption rejection and existing producer cancellation
checkpoints; there is no cancellation callback inside the current store API.
Run the complete sysroot library tests, affected metadata producer/reader checks,
existing full emitted-source/source-map corpus parity, formatting, file-size and
affected verification guards. Required full final gates and V5 remain open.
Use a scoped independent review under the approved intermediate policy.

V1 semantic/native separation, component Engine/Store reuse, durable history,
fixture/build lifecycle work and the shared gate scheduler are outside this
bounded item and remain pending. Do not attribute additive whole-gate savings
from this local measurement.


### V2a acceptance — bounded complete-metadata validation facts

PR [#4386](https://github.com/sifr-lang/sifr/pull/4386) accepts implementation
`22ed35a76efbb03eeeafa389a03960475eaa7941`. Complete validation retains successful
canonical record/field facts only within that invocation, with FIFO eviction,
oversized-entry recomputation and an accounted budget capped at 32 MiB and half
the configured decoded-record limit. Immutable captured bytes remain authority.
Every root still performs its own traversal, cycle, depth, decode-work and kind
checks. Lazy consumers keep their original read/decode/allocation path and
separate retained-record accounting; no graph verdict or test result is cached.

The serial ABBA comparison used the same retained real metadata fixture, five
fresh stores per process and two processes per version. Median complete validation
fell from **2.286071 s to 0.998618 s (56.3%)**, with an identical portable payload
digest and approximately 10 MiB additional peak process RSS. Type reads fell
from 79,566 to 1,420 for 1,420 records; HIR expression reads fell from 108,211 to
13,534 for 13,534 records. These counters measure actual payload reads, not the
per-root validation work that remains enforced. This is a local diagnostic,
not a full-gate timing or product-identity qualification.

Named execution on `78d3af7209fc41cb67bc39f94ca11dfafaa6302f`:

| Category | Result | Wall time including command preparation |
| --- | --- | --- |
| Complete sysroot library | 55 passed, including all five new fact-reuse controls | 7.185 s |
| Sysroot clippy | No warnings in changed code; five pre-existing test warnings retained | 3.185 s |
| Driver preparation | Completed; retained outputs | 355.911 s |
| Complete metadata producer group | 15 passed; subprocess entry invoked by parent tests | 132.252 s |
| Complete metadata reader group | 9 passed; full corpus run separately | 27.696 s |
| Full source/metadata emission corpus | 732 fixtures, exact Rust and source-map parity | 105.739 s |
| Formatting, size, area, taxonomy and unchanged-input guards | Passed | 55.207 s |

The producer checks include real cancellation, concurrency, failure recovery and
publication. The corpus comparison also proves all 1,464 generated/fixture files
and the portable metadata digest match the retained V1b1 baseline. Its original
raw JSON equality check failed on 192 `stdlib_modules` arrays: the unchanged
producer serializes an unordered `HashSet<String>`. The corrected comparison
requires duplicate-free set equality only for that field and exact equality for
all other fields, maps, target/selection descriptors and files. Preserve that
initial failure; deterministic report ordering remains V4 work. The observed
V1b1 baseline `cc52c0ed0` and PR base `a85b56bb2` have identical compiler, fixture,
Cargo, toolchain/config, tool-pin and profile inputs despite squash-merge lineage.

An additional affected audit found six missing filesystem-effect rows and one
missing literal test-control row introduced by V2a. The final candidate adds
only those rows, preserving every existing row/classification. Filesystem
self-tests, the complete method-authority scan and its 17 self-tests, profile
validity, file-size and whitespace checks passed on the final candidate. The
full filesystem scan **still fails on exactly 344 base discrepancies**; V0.18
below owns them. Neither the initial 350-discrepancy scan nor the remaining
344-discrepancy scan is reclassified as passing.

Evidence reuse is input-bound: only the two inventory JSON files changed after
runtime validation. All 45 first-party Cargo artifacts were fresh, and driver
and sysroot test executable SHA-256 values were identical before/after. Freshness
checks took 2.758 s and 0.750 s; these are not new test executions. Original
runtime receipts retain `78d3af720`, and paired timing retains `fa20b3359` (the
later source difference was test-helper checked offsets/style only). Raw evidence:
`v2a-acceptance.json`, `v2a-artifact-reuse-proof.json`, `v2a-paired-probe/`,
`v2a-corpus-comparison.json`, `v2a-corpus-initial-comparison.json`,
`v2a-filesystem-audit-reconciliation.json`, `v2a-policy-validation.json` and the
named receipt directories in the external LOCAL-GATE evidence root.

Final independent review is **SATISFIED**, with no blocking findings, under
`reviews/22ed35a76efbb03eeeafa389a03960475eaa7941/`. The initial source review at
`78d3af720` is also retained, including the later audit qualification note.
Optional immutability-comment maintenance and existing sysroot test warnings
remain with their owners. V1, the remaining V2–V5 work, full final create-PR/merge
gates and the ten-minute target remain open. This record-only closure does not
change the qualified implementation inputs.

### V0.18 bounded repair — reconcile the filesystem ownership ledger

The exact V2a-base scan found 344 stale/unclassified filesystem-effect
relationships outside the V2a test file. Reconcile these against current source
before returning to the compiler dependency boundary work. Preserve the failed
runs and their source-bound baseline comparison. No scanner weakening, directory
exclusion, blanket permission or Rust behavior change is in scope.

Review each added/removed relationship. Preserve all still-live classifications;
carry reviewed ownership through demonstrably equivalent moved/renamed sites.
Classify actual new reads by semantic input, build identity or tooling input,
and actual mutations as output effects. Remove only relationships absent from
the canonical source discovery. Keep the inventory schema and discovery rules
unchanged. Record the proposed additions/removals and their ownership rationale
outside Git before applying the reconciled ledger.

Required checks: the complete filesystem scan and its negative controls; proof
that all live existing classifications remain unchanged; file-size, whitespace,
profile validity and changed-link checks. Run the developer-tooling static suite
to discover other cheap policy failures together; preserve and separately assign
any unrelated failure instead of hiding it in this repair. No compiler rebuild
is needed if only this ledger and phase records change. Use scoped independent
review under the approved intermediate policy. The full final gates remain
mandatory; V0.18 makes no runtime-speed claim.

The next compiler-boundary investigation should also account for the observed
355.911-second rebuild after a metadata-store edit: stdlib manifest depends on
sysroot layout/resolution, while the same sysroot crate owns metadata storage.
That edge propagates metadata changes into imports, lowering and codegen. Record
this as real dependency coupling, not a broken Cargo cache. The semantic/native
separation and the actual SQL clean/reused/independent-clean feasibility protocol
remain required; no gain from a future crate split is assumed here.


### V0.18 acceptance — current filesystem ownership ledger

PR [#4387](https://github.com/sifr-lang/sifr/pull/4387) accepts implementation
`4d7d485e9326abbf3dc42b34a96146e05f8cc47b`, based on V2a merge
`ca51b29c9fe57f9a80d8c54895be3103d328b7d8` (#4386). The unchanged source scanner
now accepts **3,316 sites**: retain all **2,999** live rows and their existing
classifications/order, add **317** current rows, and remove **49** absent rows.
The original 344 diagnostics count distinct site keys with multiplicities;
they are not a count of individual JSON rows. Repeated identical keys are
compared as multisets. No scanner, Rust source, dependency, schema or profile
selection changed.

New ownership is explicitly adjudicated: 204 mutations are output effects,
86 test/support/harness reads are tooling inputs, 16 native-cache reads are build
identity, and 11 LSP external snapshot reads are semantic inputs. The native-cache
reads qualify owner, artifact bytes/receipt, generated inputs or retirement
context even though they live in test support. Relocated effects retain their
old ownership. The proposal, source-derived additions/removals, rationale and
live-row/order preservation proof remain outside Git in
`local-gate-implementation-20261020` under `v018-*`.

The full filesystem audit passes in **16.146s**, its complete negative controls
pass in **0.639s**, and preservation/profile/file-size/whitespace/link checks
pass in **3.668s**. Independent review is **SATISFIED**, no blockers, at
`reviews/4d7d485e9326abbf3dc42b34a96146e05f8cc47b/review.md`. Review ran while broader discovery was pending;
all final discovery results below are recorded before merge. The only later
repository change is this phase record.

All **18** canonical developer-tooling static variants now have completed
execution: **17 pass**, and the existing linter-diagnostic-class guard fails
because it names the former implementation file. That failure belongs to V0.19;
it is not removed or reported as passing. Complete completion-quality execution
passes in **635.736s**, including its non-incremental compiler/test
preparation. Recovered suppression rules pass in **27.276s** and their
negative self-test in **4.692s**; both final PASS markers are explicitly
verified in addition to successful exits. Their current execution did not take
the false-success path identified below. Sum of final per-category diagnostic
times is **711.935s**, not whole-gate elapsed
or an isolated performance measurement. Raw original attempts are retained in
`v018-static-validation.json`; completed coverage reconciliation is
`v018-static-reconciled.json`.

Preserved failures/incomplete attempts: the first broad discovery was stopped at
70.478s when the nominal static suite launched Cargo in the review checkout.
Its runtime variants were moved to the retained compiler checkout. The first
external preservation diagnostic incorrectly used sets for duplicate site keys;
the corrected Counter-based proof passes without a repository change. Two
suppression attempts stopped on disk reserve (550.330s and 7.698s), not the
30-minute cap. An initial recovery admission refused insufficient allocation
headroom before launching tests. These remain interrupted/failed attempts.
Capacity recovery archived and checksum-verified inactive investigation scratch
and four superseded incremental generations; latest generations, leases,
current artifacts, reusable caches and failure evidence were retained. Archive
receipts include restoration instructions. No reserve was weakened.

Follow-ups: V3/V4 own the hidden Cargo preparation inside the static suite and
its different incremental configurations; V1 owns the codegen dependency still
pulled into editor completion checks. The 635.736s completion category is a real
preparation bottleneck, not a full-gate result. A reusable multiset preservation
check for future ledger updates is a tooling suggestion, not a prerequisite for
this bounded repair. Full final create-PR/merge gates and V5 remain open.

### V0.19 bounded repair — lint verification authority and failure propagation

Repair two source-confirmed verification defects together, without changing
lint/compiler behavior or reducing test selection:

- The diagnostic-class guard still requires the typed gate in
  `host/implementation.rs`, although `d7ecb71f1` moved action authority into
  `host/code_actions.rs`. Bind the guard to that owner and fail closed when the
  owner or required gate is missing. Retain the whole analysis/LSP prefix-gating
  prohibition and the diagnostic-class wire payload checks. Add controls for a
  valid current layout, missing/misplaced owner/gate, forbidden prefix gating,
  and missing conversion payload/owner.
- The suppression adapter exits with the unexpected child status. If it expected
  one but observed zero, `SystemExit(0)` falsely marks the incomplete verification
  successful. `v019-exit-probe.json` reproduces this through the real adapter
  function with a controlled child result at the V0.18 candidate. Make every
  expected/observed mismatch fail, retaining diagnostic output. Enroll controls
  for matching success and matching expected failure, unexpected success,
  unexpected failure, and a wrong nonzero failure code.

Named acceptance: real diagnostic-class guard and full self-tests; suppression
adapter mismatch regressions and real suppression positive/negative assertions;
the existing analysis policy-versus-hard-diagnostic code-action regression using
the retained completion-test configuration; affected profile/size/whitespace
checks and scoped independent review. Preserve V0.18 broad results with their
observed candidate identity; reuse only input-equivalent evidence. Update the
static-suite coverage reconciliation explicitly, without claiming a complete
phase gate. No compiler source change or clean build is required for this policy
repair. Then return to V1 dependency separation. Final phase gates remain required.
