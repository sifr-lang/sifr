# Ad Hoc Phase: Complete Local Correctness Gate Within Ten Minutes

status: in progress; integrated acceptance pending
registered: 2026-10-19
phase_id: LOCAL-GATE
current_stage: V4e selected-category diagnostic recorded; V4l context-coherence repair registered; V1 feasibility and integrated acceptance remain open
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

The user also authorized continuing this phase to completion in the same task.
This supersedes the new-session/stop-after-one-item defaults; close each bounded
item before starting the next.

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

Current evidence: Bounded repairs through V0.20 are accepted below. Final
integrated completeness remains open.

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

Current evidence: V1b1–V1b4 ownership separation and native/installed/SQL qualification
are accepted. The same-source six-worker SQL protocol passes in 788.350s, still
above the budget. V1c measured and rejected concurrent build overlap; complete
build-path feasibility remains open. Earlier profile and sanitizer observations are retained.
V1e's later matched baseline takes 793.732s excluding external probe preparation.
Its Cranelift opt0 candidate takes 734.057s but makes the complete PostgreSQL
component runtime probe 8.6 times slower; that override is rejected.

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

Current evidence: V2a metadata validation facts and V2b bounded compiled-component
ownership and V2c's durable-record validation facts are accepted below. Further
whole-suite attribution and integrated acceptance remain open.

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

Current evidence: V3b/V3c, retained Rust-interop fixtures (V3e/V3h) and bounded
native bundle processing (V3f/V3j) are accepted below. Compiled-Python lifecycle,
complete command/version reuse proof and integrated acceptance remain open.

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

Required proof: Run complete 44-command matrices, every currently selected native role
(the historical inventory counted 87; the V4e refresh discovers 88), and all 29 compiled
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
| V0 executable coverage | In progress | Bounded repairs accepted below, most recently V0.20 / #4389; final completeness pending |
| V1 mandatory build path | In progress | V1a experiment / #4383 recorded; V1b1 / #4384 through V1b4 / #4392 accepted; V1c scheduling and full feasibility pending |
| V2 repeated metadata/component work | In progress | V2a / #4386, V2b / #4394 and V2c / #4397 accepted; integrated acceptance pending |
| V3 compatible build reuse | In progress | V3b/V3c/V3e/V3f/V3h/V3j merged; compiled Python, command/version proof and integration open |
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


### V0.19 acceptance — lint verification authority and failure propagation

PR [#4388](https://github.com/sifr-lang/sifr/pull/4388) accepts implementation
`bb9de83ca671254090ae721c20c11be9b65ed33a`, based on V0.18 merge
`a75fdfc7840da8284198825b4638ba4b76684764` (#4387). Only the two scoped Python
verification adapters change. The diagnostic-class guard now binds to the actual
code-action owner and rejects missing owners/gates and missing wire payloads;
its complete analysis/LSP prefix prohibition is preserved. The suppression
adapter exits nonzero for every expected/observed status mismatch and reports
both values, closing the reproduced expected-one/observed-zero false success.
Real lint assertions remain unchanged.

All **18 developer-tooling static variants pass** on that exact candidate;
the sum of their individual timed categories is **54.132s**. This is a prepared,
local diagnostic run, not the complete gate. Completion-quality is **3.688s**
with retained preparation versus V0.18's **635.736s** preparation run. That
contrast confirms preparation dominance; it is not a performance gain caused
by this policy repair. Suppression rules and full self-tests pass in **6.684s**
and **4.689s**; the new exit-status and real negative-control PASS markers are
verified. Controls cover matching zero/nonzero expectations, unexpected success,
unexpected failure, wrong nonzero status and signal termination. Diagnostic-class
controls cover the valid/restored layout, prefix seeds in both roots, misplaced
former gate, missing current owner, missing gate, each missing wire snippet and
missing conversion owner.

The existing qualified policy-versus-hard-diagnostic code-action regression
passes: exactly **1 passed / 0 failed / 70 filtered**, **9.190s** including
preparation (6.18s test runtime), using the retained non-incremental analysis
configuration. Profile, file-size, whitespace, local-link and only-two-changed-
paths checks pass in **3.645s**. Rust/compiler inputs and profile selection are
unchanged. Python is pinned to 3.14.7 and the observed interpreter optimization
level is zero. Full final gates and V5 are not claimed.

Independent review is **SATISFIED**, no blockers, retained externally at
`local-gate-implementation-20261020/reviews/bb9de83ca671254090ae721c20c11be9b65ed33a/review.md`.
Other evidence: `v019-static-validation.json`, `v019-input-proof.json`, per-check
`v019-*` receipts/logs and the preserved pre-repair `v019-exit-probe.json`.
Subsequent repository changes update this phase record only.

Review follow-up adjudication: the formatter adapter has the same expected-
nonzero false-success defect; V0.20 owns it. The suggested frontend-cache
adapter is **not** affected: it accepts only zero and raises a child status only
when nonzero. `v020-before-probe.json` confirms formatter expected-one/actual-zero
exits zero, frontend-cache actual-zero returns normally, and actual-two exits
two. Preserve the raw review; do not change the safe adapter. Assertions under
an intentionally optimized Python invocation remain a separate verification
execution-policy suggestion for V4; canonical validation here used optimization
level zero.

### V0.20 bounded repair — formatter verification failure propagation

Apply the demonstrated expected-exit repair to
`verification/areas/developer_tooling/check_formatter_rules.py`. Every mismatch
must fail, including unexpected success in formatting-drift and invalid-syntax
negative checks. Retain command output and report the expected/observed status.
Enroll the same matching/mismatching status controls in its canonical self-test;
preserve the actual formatting, string-content, idempotence and invalid-syntax
assertions. Do not change compiler behavior or build selection in this repair.

Audit maintained first-party Python expected-exit adapters for this exact failure
family and preserve the source/condition findings externally. Do not rewrite
correct zero-only status propagation or unrelated test adapters. The frontend-
cache counterexample above is explicitly out of the repair scope.

Named acceptance: the complete six-variant developer-tooling formatter suite,
new exit-status controls, real positive and negative formatter checks, affected
compatibility scan and self-test, profiles/file-size/whitespace checks and scoped
independent review. Require final assertion markers as well as exit status.
Reuse retained compiler targets and keep the 30-minute category cap. Return to
V1 dependency separation afterward; full final gates remain required.


### V0.20 acceptance — formatter verification failure propagation

PR [#4389](https://github.com/sifr-lang/sifr/pull/4389) accepts implementation
`d39f00e0c6dd147883b96fbde48259b05e1003a1`, based on V0.19 merge
`ac4d64d1145e2cfd7fd9e335e0c4c972a0cc6ebe`. Only the formatter adapter changes.
Every expected/observed child-status mismatch now exits one and reports both
statuses, preserving child output. Its canonical self-test adds six controlled
matching/mismatching pairs, including unexpected success, wrong nonzero status
and signal termination. Existing real formatting, string-content, idempotence
and invalid-syntax assertions remain.

All six formatter variants and both affected compatibility variants pass on
that exact candidate. Formatter rules take **146.912s**, including preparation;
the complete formatter self-test takes **4.689s**. The other four formatter
variants take **0.685s, 0.685s, 4.687s and 0.683s**, respectively. Compatibility
checks take **19.180s** and **0.637s**. Final real-check and new-control PASS
markers are verified. Profile/file-size/whitespace/link/input checks pass in
**6.184s**. No source changed during validation, no category hit the cap, and no
worker remained. These are per-category diagnostics, not full-gate elapsed.

The source-bound failure-family audit parses all **546** tracked first-party
Python files under scripts and verification, with no parse skips, recording
per-file hashes and expected-status comparisons (including compound conditions)
and direct child-status exits. Remaining direct propagation is guarded by
nonzero status; other expected-status adapters record explicit failures. This is
a scoped pattern audit, not a proof about every possible Python bug or embedded
source string. The original formatter false success and the correctly behaving
frontend-cache counterexample remain in `v020-before-probe.json`; the latter
adapter is unchanged. Audit evidence is `v020-exit-family-audit.json`.

Independent review is **SATISFIED**, no blockers, stored externally at
`local-gate-implementation-20261020/reviews/d39f00e0c6dd147883b96fbde48259b05e1003a1/review.md`.
The reviewer inspected source and relied on the recorded execution evidence;
it did not independently rerun remote tests. Additional suggestions—verification
assertions under optimized Python and adding output sentinels to status
controls—are follow-up hardening, not omitted acceptance criteria. Canonical
validation used optimization level zero. Evidence includes
`v020-formatter-validation.json`, `v020-input-proof.json` and per-category logs.
Only phase records follow the reviewed implementation. Final gates and V5 remain
open.

### V1b2 bounded implementation — separate raw metadata ownership

Move the raw schema/container/store from `sifr_sysroot` into an explicit lower
`sifr_metadata` crate. Keep lightweight sysroot resolution/digests independent of
that crate. The observed metadata-store-only edit previously rebuilt stdlib
manifest, imports, lowering, frontend and codegen through the sysroot edge;
retained driver preparation took **355.911s**. Remove that edge without a sysroot
re-export or compatibility dependency. This prerequisite does not itself deliver
the mandatory semantic/native compiler separation or prove a gate runtime.

Move existing metadata source unchanged, retaining internal declaration order,
format version 5, field order, diagnostics, lazy/full semantics, resource limits
and all tests. Bind direct services/driver consumers and compiled identities to
the new owner. Move only its necessary dependencies; retain sysroot dependencies
used by its remaining layout/integrity/digest responsibilities. Enroll the new
owner in workspace and verification inventories, all applicable profile package
lists, configuration checks, GCQ input ownership and first-party graph policies.
Add dependency-direction regressions to reject raw metadata edges/re-exports in
sysroot and frontend-only consumers. Update genuine current schema-generator and
format-admission paths, method and filesystem ledger paths without changing
classifications, and the architecture owner description. Preserve all external
lockfile versions/checksums; explicitly reconcile approved verification graph
hashes only for intentional first-party graph changes.

Named acceptance:

- All 28 metadata and 27 remaining sysroot tests; compiler-services tests; all
  metadata producer/reader tests including the ignored complete 732-fixture
  producer/reader corpus. Prove moved source byte equivalence and format/schema
  parity, including installed/source admission contracts.
- All 732 positive, 569 negative and six runtime-failure E2E inputs, retaining
  canonical selection and outcomes. Compare complete metadata corpus results
  with the accepted portable baseline; only the already proven unordered
  `stdlib_modules` report array may be normalized in external comparison.
- Actual resolved Cargo graph and compiled-input identity proof, with a
  controlled metadata-only mutation/restoration and observed build freshness:
  lightweight sysroot/manifest/lowering/codegen must not depend on the raw
  metadata owner; genuine services/driver consumers must invalidate. Record
  mutation and restored candidate identities separately; no timing subtraction.
- Affected dependency/inventory/profile/configuration/GCQ/direction, method
  dispatch, filesystem and distribution checks with their controls; formatting,
  file-size, whitespace and changed-link checks; exact-candidate independent
  review. Retain 17 configurations and existing test selection, adding new owner
  membership where needed. No blanket profile or incremental-mode change.

The legacy metadata inventory CLI still points at historical documentation;
that pre-existing historical-site check is not in the canonical runner. Keep
historical evidence unchanged. Validate its live schema/field check at the new
owner, record the historical CLI issue for V4 tooling ownership, and do not
revive obsolete site requirements as part of this extraction.

Before rebuilding, verify resource headroom. Archive only inactive,
session-owned obsolete artifacts with checksum/lease/consumer checks and
restoration receipts; preserve current compatible build targets and failure
records. Keep the 30-minute category cap. V1 semantic/native separation, full
SQL clean/reused/independent-clean qualification, final integrated gates and V5
remain mandatory after this bounded prerequisite.


#### V1b2 discovered legacy field-check failure

The first extraction candidate `0959472a31342b2aba4dc0ff0dd23bfba1a27f2b`
passes 14 of 15 focused policy/verification categories; the legacy payload-field
check fails. A source archive of the exact pre-extraction base
`c569420aea1779486d381defdc198ea096c5e2b2` reproduces that failure before any
relocation: `ExternalDefs field inventory changed`. The baseline probe had
expected to reach the stale site list, but execution stops at the earlier field
comparison; retain that diagnostic and its original failed expectation.

`ExternalDefs` contains two process-local capabilities, `provider` and
`typed_method_processor`, both optional `Arc<dyn ...>` authorities. All other
field names match `SemanticExports`; the old checker only classifies `provider`
as host-only. This is checker drift, not a field dropped by the extraction.
The historical inventory still exists at its original documentation path; it was
not moved into a history directory. V4 owns correcting the live field
classification and its regression controls, separately from preserving the
historical site catalogue. Do not mark this failed check as passing or silently
remove its final reconciliation obligation.

V1b2 retains its stronger relocation-specific proofs: every raw metadata source
and fixture is byte-identical, format and payload field order are unchanged,
producer/reader and full-corpus parity must execute, and installed/source
admission must pass. Preserve the legacy failure and its unchanged-input hashes
alongside these results under the approved intermediate policy. The new crate's
initial compile also exposed an omitted `ring` declaration; add the same existing
pin to the new owner and reconcile its audited direct declaration. Keep that
failed 8.782-second preparation attempt; no test executed successfully in it.


#### V1b2 policy ownership preservation

The final owner audit found one path-scoped rule that also needs the new owner:
the no-compatibility check's flat-install-layout prohibition currently covers
CLI and sysroot sources. Retain its coverage of the moved metadata source by
adding the new crate prefix, with a negative control for every covered owner.
This is part of V1b2's ownership/inventory preservation; it does not add a fallback
or change compiler behavior. Validate the real guard and its canonical self-test.
The compiler/runtime candidate is frozen in its separate validation checkout;
prove unchanged compiler and runtime inputs before reusing its evidence for
this Python-policy-only follow-up.


#### V1b2 discovered producer fingerprint omission

The complete native corpus passes all 732 positive, 569 negative and six
runtime-failure inputs. The new GCQ same-mtime ownership control then fails:
the producer fingerprint enumerates each registered crate's manifest and source
but omits its `build.rs`, including the new metadata owner's identity emitter.
Retain this failed execution. Correct the registered producer-root inventory to
include existing build scripts, and prove equal-mtime content edits and restoration
for both moved owners' build scripts as well as their manifests/source. Preserve
all three real Clippy-isolation controls. This is the V1b2 producer-identity
acceptance boundary, not a cache bypass. V3 still owns complete producer/input
inventory and materialization reuse across all toolchain/environment contexts.
The Rust candidate and completed native evidence remain frozen; execute changed
GCQ controls on the corrected candidate and prove unaffected inputs before reuse.


### V1b2 closure — raw metadata ownership / #4390

Implementation candidate `20d783933d8000e63e0263008833cd4027ce04bf` passed
its named acceptance and independent review. The 35 moved source/fixture files
remain byte-identical; format 5, fields, validation semantics and external
package versions/checksums are unchanged. Lightweight sysroot/manifest/imports,
lowering, frontend and codegen do not depend on the raw metadata owner.
Services/driver use it directly and include its compiled input tokens.
All 17 merge configurations remain selected; profile membership is additive.

Runtime observations belong to `4b9f39d19a391305255a706e9423026bb145432b`.
The final candidate differs only in phase records, the compatibility policy and
GCQ fingerprint/controls. `v1b2-final-input-equivalence.json` proves unchanged
Rust/compiler/runtime, generator, profile and distribution/sysroot inputs.
Compatibility controls ran on `87e8f7c3f8e2c8dfbd1fed95d319e00933219414`;
GCQ controls and the final size guard ran on the final implementation candidate.
Do not relabel these observations as execution on another SHA.

| Named observation | Result | Wall seconds |
| --- | --- | ---: |
| Raw metadata / remaining sysroot | 28 / 27 tests pass | 58.759 / 9.224 |
| Services / producer / reader | 26 / 15 / 9 tests pass | 101.454 / 133.780 / 28.698 |
| Full source/provider emission corpus | All 732 fixtures pass | 106.243 |
| Initial driver preparation | Workspace-change preparation passes | 687.599 |
| Metadata-only edit / restore | Correct consumers rebuild; lower compiler owners remain fresh | 18.717 / 12.264 |
| E2E executable preparation | Pass | 696.199 |
| Positive / negative / runtime-failure E2E | 732 / 569 / 6 inputs pass | 271.396 / 7.690 / 62.841 |
| Dependency audit / sysroot CLI | 15 / 3 tests pass | 12.704 / 6.195 |
| Distribution full / qualification | 72 / 1 variants pass | 855.229 / 28.703 |
| Real installed/source admission | Exact metadata identities/diagnostics and rejection/recovery pass | 230.813 |
| Corrected GCQ ownership controls | All four tests pass, including three Clippy-isolation controls | 1.140 |

These are distinct, sometimes overlapping workloads, not an additive full-gate
runtime or a matched speedup ratio. The controlled source edit changes the real
metadata compiled tokens; restoring its bytes restores both tokens exactly.
The initial preparation results above still exceed ten minutes. V1's remaining
build work and V5 acceptance cannot be inferred from the fast metadata edit.

All 1,464 emitted source/Rust files and other corpus row fields/source maps match
the accepted baseline. Only the previously declared unordered `stdlib_modules`
HashSet report array is compared as a set, with duplicate rejection; 191 rows
reorder. The initial cross-candidate portable-hash equality check failed because
that digest includes workspace/lock-bound stdlib inputs and record anchors.
Preserve the failed comparison. An isolated diagnostic using the real retained
producer under the exact baseline compatibility envelope reproduces all
6,563,453 physical metadata bytes and the baseline portable digest. It publishes
nothing; actual candidate corpus, installed admission and native tests keep their
real changed identities. `v1b2-identity-aligned-projection-proof.json` and
`v1b2-qualified-corpus-comparison.json` bind these distinct proofs.

The installed check packages the exact native dev-profile binary built by the
successful self-update distribution case. It validates complete structural data,
source/installed metadata and diagnostics parity, incompatible/missing/corrupt
metadata rejection, restoration and lazy checking without navigation source.
It is not production release qualification or the complete installed-public-emit
corpus; those retain their owning final contracts. The original omitted-ring
compile, legacy field checker failure, initial digest-comparison failure and
GCQ build-script omission are preserved. The latter omission is repaired by
including existing build scripts for registered producer crates.

`v1b2-final-acceptance.json` binds 41 successful named categories to raw
specifications, receipts, output and resource hashes. The collected external
bundle is `v1b2-evidence.tar.gz`, SHA-256 `a8152ef9dbfeba39f101afa85546f792ea1c12856b042615e4a7f647cb3b25ef`.
Independent review is **SATISFIED**, with no blocking findings, stored outside
the reviewed tree at `reviews/20d783933d8000e63e0263008833cd4027ce04bf/review.md`.
The reviewer checked the full diff and all acceptance-row evidence hashes without
repeating broad validation. This closure changes records only.

The planning-only regression-control entry now points to `sifr_metadata`; its
28-test filter and unchanged source anchors are verified from executed evidence.
Follow-ups remain explicit: V1 owns permanent transitive graph guards alongside
the next ownership boundary; V3 owns complete GCQ producer inputs (including the
pre-existing omissions of sysroot, compiler services and identity); V4 owns the
legacy live field checker, historical/current documentation clarity and report
ordering. Historical metadata consumer documents stay unchanged. Final gates,
semantic/native separation, V2–V5 and measured sub-600-second acceptance remain
open.

### V1b3 bounded implementation — canonical checked bootstrap and native observer

Separate the existing stdlib bootstrap's checked-source loop from native
materialization while preserving its full-native public result and format-5
output. One canonical loop owns ordered parsing/lowering, canonicalization,
export/re-export accumulation, checked HIR and transitive import facts. Native
materialization owns the syntax session, Rust-source identities, backend
features, emission support and private interop. Do not add an alternate parser
or lowerer, a fake complete native result with empty defaults, or a semantic-only
product before explicit artifact roles and admission exist.

Preserve event and error order: retain private declarations and empty private
HIR/interop; emit a nonempty module before installing its own exports/signature
support; later modules see prior successfully installed state. Keep canonical
source order, existing diagnostics and source locations. Split helpers by
responsibility below 900 lines. Do not change optimization, incremental modes,
external tool/dependency versions, profiles or test selection in this item.

Named validation: focused real-source observer ordering, failure and empty-private
controls; all existing driver stdlib assertions (56 before additions); services,
producer and reader tests; the complete 732 source/provider emission corpus and
732 positive, 569 negative and six runtime-failure inputs. Compare exact emitted
outputs and portable projection against V1b2 under appropriate equal-input
conditions; preserve unexpected differences for diagnosis. Retain source/installed
admission, affected inventory/schema/direction/method/filesystem contracts and
classifications, formatting, file-size and links. Review the exact candidate.

This item prepares the mandatory semantic/native boundary; it does not claim a
Cargo dependency reduction or gate timing. Explicit role/versioned metadata,
construction/consumer ownership and the complete three-SQL-tool clean/reused/
independent-clean qualification remain required before accepting V1.


V1b3 implementation started from V1b2 merge `9c4ef618228944aa821c69382979124df136d453`.
The first focused compile (`v1b3-initial-observer-controls`, 4.687s) caught a
new test indexing `ModuleMap` rather than using its lookup API; no tests ran.
The corrected four observer controls pass (`v1b3-observer-controls`, 46.288s
including preparation). Both observations remain preserved. The final candidate
also uses a borrowed dynamic observer to avoid instantiating the large checked
loop per observer, and adds a real native-path error/empty-private control before
full named validation. No external dependency, profile or test selection changes.

The full split-brain guard reports three pre-existing codegen test references.
An exact baseline archive reproduces identical failures (`v1b3-split-brain-baseline-proof.json`):
the test-ownership walk follows `mod` edges but not `include!` descendants of
`method_authority_tests`. V4 owns this guard reachability repair and negative
controls; do not exempt production includes or count this guard as passing.
The existing bootstrap parser/lowerer site controls pass. The complete coverage
area, dependency direction, method/filesystem ledgers, format and size checks pass.


### V1b3 closure — checked bootstrap and native observer / #4391

Exact implementation candidate `55853501c71eae78a50741adfcd8f24935676dc4`
passes all 22 named acceptance categories and independent review. The canonical
checked loop owns source-order parse/lower, canonical HIR, exports/re-exports and
transitive imports. A borrowed native observer retains private interop before
empty-private handling and renders before current-module export installation.
The public result remains complete native data; format 5, dependency versions,
profiles and selected tests remain unchanged. No semantic-only artifact is admitted.

| Named observation | Executed result | Wall seconds |
| --- | --- | ---: |
| Compiler services | All 31 tests, including five new boundary controls | 77.298 |
| Driver preparation | Lower owners fresh; services/driver rebuild | 73.819 |
| Driver stdlib | All 56 assertions pass | 50.774 |
| Metadata producer / reader | 15 / 9 tests pass | 129.815 / 27.759 |
| Explicit complete emission corpus | All 732 fixtures pass | 107.796 |
| E2E preparation | Pass, including runner overhead | 90.099 |
| Positive / negative / runtime-failure E2E | 732 / 569 / 6 inputs pass | 123.869 / 7.748 / 11.251 |
| Sysroot CLI | All three tests pass | 5.749 |
| Distribution full / qualification | 72 variants / nine internal qualification controls pass | 247.530 / 28.757 |
| Real installed/source admission | Exact metadata/diagnostics, integrity and rejection/recovery pass | 226.353 |

All observations above run on the exact implementation candidate. The ignored
producer child is exercised by its parent process protocols; the complete
normally ignored corpus is explicitly executed. Positive E2E reports 732 outcomes,
732 passes, zero failures and zero original batch failures. Compiled native
artifacts may be reused; assertions execute. These overlapping workloads and
retained-target observations are not additive full-gate timing or a matched
speedup estimate. The earlier approximately 82-second E2E number was Cargo's
build duration; the complete receipt is 90.099 seconds.

Against the accepted V1b2 runtime candidate, all 1,464 source/Rust files, source
maps and remaining row fields match exactly. The portable metadata digest also
matches exactly: `df5aaca26bc0424e901cd1457419191d3ed1e0345da0b1980cca8c0f070c9a6b`.
Only the already declared unordered `stdlib_modules` report array is compared
as a set, rejecting duplicates. The physical metadata/compiler envelope changes
as expected. No controlled-envelope rewrite is needed for this item.

The real installed check uses the versioned dev-profile binary built by the
successful self-update case and canonical packaging. It verifies complete
structural metadata, exact source/installed identity and valid/invalid diagnostic
parity, installed integrity, incompatible/missing/corrupt metadata rejection,
restoration and lazy checking without navigation source. It is not production
release qualification or the full installed-public-emit corpus; final contracts
retain those obligations.

The final method ledger (6,400 sites), filesystem ledger (3,316 sites), coverage
area, dependency direction, canonical bootstrap-site controls, formatting and
file-size checks pass. Preserve the initial test compile failure and the full
split-brain guard failure. An exact baseline archive reproduces the three
codegen test-include findings; V4 owns `include!` test-reachability controls and
repair. The full split-brain guard is not counted as passing. The first external
final-guard script had a quoting NameError before any checks; its separate
corrected script ran the final checks successfully.

`v1b3-final-acceptance.json` binds 22 categories to specifications, receipts,
outputs and resource hashes. `v1b3-structure-proof.json` records unchanged
schemas/manifests/profiles and actual lower-owner Cargo freshness. The complete
corpus, current metadata artifact, installed observations and failed evidence
are in `v1b3-evidence.tar.gz`, SHA-256
`817167adb48719b0ed050ab060775b74c68940a2aba4ac56dd1839a418cf18e7`.
All 1,673 manifest file hashes were verified after collection. Independent
review is **SATISFIED**, with no blockers, at
`reviews/55853501c71eae78a50741adfcd8f24935676dc4/review.md`, outside the reviewed
tree. It confirms ordering, lifetimes, API and policy preservation and accepts
the recorded named evidence without repeating broad validation. This closure
changes records only.

Follow-ups: V1b4 owns the invariant that the native observer's live dependency
map matches the checked map installed at completion, plus the permanent
transitive dependency protection identified in V1b2. V4 retains the baseline
field-checker and test-include guard repairs. A read-only ELF inspection found
1,205,774,477 bytes of debug sections in the 1,312,681,704-byte ordinary dev CLI;
V4 may measure limited-debug development/test preparation with unchanged
assertions and diagnostic/backtrace requirements. This does **not** explain the
SQL bottleneck: historical source and actual normal/opt1 SQL trial settings
both explicitly used `CARGO_PROFILE_DEV_DEBUG=0`. Do not rerun a redundant SQL
debug-disable experiment or infer timing savings from binary sizes.

### V1b4 bounded implementation — semantic consumption and native production

Deliver the actual dependency cut needed by the three linked SQL tools. Keep
canonical frontend checking for migrations and the real component host for SQL
profiles; remove driver/native-emitter ownership from their normal resolved
Cargo closure. Preserve common checked HIR, semantic/nominal authority, source
ordering and diagnostics rather than adding a SQL-specific parser or lowerer.
Move native production/interop/preview behavior to its honest native owner and
share only the lower representations and operations actually needed by both.

First resolve the construction and admission contract from real callers. A
complete typed in-memory checked-stdlib product and an explicitly role-versioned
persisted artifact are distinct designs; compare their preparation/lifetime needs
before selecting one. Never pass an empty/default native product as semantic
success. A new persisted role requires explicit schema/version, keys, descriptor
and consumer admission, corruption/incompatibility controls and native rejection.
A typed in-memory product does not waive selected-sysroot/identity correctness,
installed rejection behavior, complete native production or source/installed
parity. Record the chosen contract before changing its implementation; do not
mark the remaining role/admission requirements satisfied merely by moving code.

Keep external dependency/tool versions and assertions. Preserve the native
compiler's complete metadata, Rust output, interop and installed-package behavior.
Register any necessary schema/profile changes explicitly. Add permanent resolved
transitive graph controls and enforce the checked/native dependency-map invariant.
Qualify the actual three-tool clean-A, unchanged-A and independent-clean-B protocol
with all seven mutation controls, within-profile executable byte equality, real
migration/provider/editor/host-tool behavior, missing compiler/product/version
and incompatible metadata failures, and the affected complete native/installed
corpora. Include preparation and cleanup. The existing SQL trace contains a
167.05-second codegen unit (normal clean A), but overlapping unit durations are
not additive savings predictions. V1 feasibility and final phase gates remain
open until measured acceptance succeeds.

#### V1b4 selected construction/admission contract

Select a complete typed in-memory `CheckedStdlib` for source semantic consumers;
retain format-5 complete native artifacts for native production and installed
consumers. Fresh-process diagnostic N/S/S/N preparation was 0.672875 / 0.207799 /
0.211957 / 0.656064 seconds, with source capture separately 0.0116–0.0318 seconds.
All four checked projections exactly matched complete native serialization
(portable digest `df5aaca26bc0424e901cd1457419191d3ed1e0345da0b1980cca8c0f070c9a6b`).
This is construction evidence, not a SQL qualification or final gate speed claim.
Evidence: external `v1b4-checked-fresh-probe-proof.json`; retain the earlier
single-process probe and the initial missing-include diagnostic failure.

A lower `sifr_semantic_services` crate owns canonical checked bootstrap, shared
metadata consumption/nominal projection, captured source identities, and real
SQL frontend/profile services. Native services consume these owners and retain
Rust generation, native metadata production/qualification, interop and previews.
Source semantic contexts capture selected sysroot inputs and retain the complete
checked product for their immutable lifetime; no empty native product, persistent
semantic cache or native capability is implied. Installed contexts use the same
strict descriptor, target, compiler and content admission as native consumers.
Explicit metadata overrides are validated against captured source compatibility
without fallback. Failures are not memoized; independent contexts do not inherit
another identity's owner. Native format 5, descriptor schema 1 and serialization
remain unchanged. Share the canonical single-file frontend pipeline with native
callers; preserve ordering, query finalization and diagnostics. Native dependency
maps must match the checked owner, with a programmer-invariant assertion.

Register the new first-party crate in workspace, identity and applicable profile
inventories, retaining all existing selections and external dependency pins.
Intentional root/fuzz lock changes may add only the first-party ownership graph;
update approved lock identities with proof of unchanged external versions and
checksums. Permanent resolved-graph controls must reject native dependencies in
SQL production closure and raw metadata below frontend/lowering/codegen owners.


#### V1b4 development checkpoint — qualification still open

Initial implementation `7ace84ac6` compiled the semantic, native-service and
native-driver test binaries in 228.559 seconds. Thirteen semantic-service tests,
22 native-service tests, 12 metadata-reader tests, 15 producer tests, 56 stdlib
tests and the explicit 1,307-fixture semantic/native frontend comparison passed.
The comparison covers all 732 positive, 569 negative and six runtime-failure
source fixtures; successful frontend results preserve generated Rust and SQL
migration projections, and failures preserve rendered diagnostics. Real native
execution and the previous captured native corpus remain separate requirements.
The earlier `bac8d713b` preparation failed on a missing moved filesystem import
before assertions; that receipt is retained. Runtime checks overlapped cache
relocation and are correctness observations, not timing acceptance.

`v1b4-test-inventory-proof.json` confirms all 31 original services tests remain
selected across the two owners, with four additional semantic-context tests.
Every pre-existing field in all five profile files is unchanged; four profiles
add the semantic crate's unit group. `v1b4-lock-graph-proof.json` confirms exact
external package records (including versions/checksums/dependencies) remain
unchanged in root and fuzz locks: 858 and 234 external records respectively.
Only first-party graph entries and the approved fuzz-lock digest changed.
Resolved and conservative manifest graph controls pass, including transitive
aliases, edge kinds and target predicates. Newly moved filesystem effects retain
their reviewed classifications; new test effects are explicitly classified.

A full incremental-cache copy was stopped after 771.473 seconds because it
caused substantial filesystem contention; it did not replace the original
cache. Capacity recovery instead verifies older individual entries before
relocation, preserving logical paths and current entries on their original
volume. Retain both operation receipts and per-file hash evidence. Final source,
installed/native corpus, linked SQL clean-A/reused-A/clean-B qualification,
independent review and final phase gates are not yet accepted.


V1b4 pre-final cross-check found two in-scope inventory omissions before starting
final runtime qualification: the new crate was absent from the maintained Cargo
manifest/owner audit, and four SQL source guards still selected the old physical
service file. Preserve the four failed guard receipts and the missing-inventory
proof. Reconcile the one new manifest, its existing-version tempfile declaration
and two num-bigint lock-owner edges without altering registry releases/checksums.
Follow both existing native and driver delegation links to the semantic owner;
mutation controls must reject removal of either link and the original mechanisms.
The same inventory sweep registers the semantic library and build script in the
coverage matrix with the existing first-party compiler/merge classification.


V1b4 qualification found that the semantic-services build script used the
feature-gated identity build API without requesting `sifr_identity/build` on
its own build dependency. Existing dependency unification made the observed
builds pass; declare the used feature explicitly instead of relying on that
incidental activation. Preserve all `160e24c` qualification evidence with its
observed identity. Recheck the resolved features and affected candidate artifacts
before final review; previous receipts are not silently relabeled.

### V1b4 closure — semantic consumption/native production / #4392

Exact implementation candidate `27859a22fcc61bc959a3ee1243fa330d7adb2513`,
based on `8509b21ac40302cf39b536850331b556ca1b17db`, passes all 56 named
applicable validation categories and independent review. All three linked SQL
tools retain canonical frontend and component-host execution while excluding
`driver`, native compiler services, codegen and native stdlib-artifact ownership
from their actual production dependency closure. The semantic owner retains
complete checked source data and shares strict installed metadata consumption;
native consumers retain full production, interop and runtime behavior. No new
persisted role, fallback, schema change or external dependency upgrade is included.

| Named observation | Executed result | Wall seconds |
| --- | --- | ---: |
| Semantic / native services | 13 / 22 tests pass; all 31 original service tests retained | 28.328 / 59.230 |
| Driver stdlib / metadata producer / reader | 56 / 15 / 12 tests pass | 48.213 / 125.254 / 30.198 |
| Complete semantic/native frontend comparison | All 1,307 positive/negative/runtime-failure source inputs | 81.730 |
| Complete source/metadata emission comparison | All 732 positive fixtures | 105.746 |
| E2E preparation / positive / negative / runtime-failure | Preparation plus 732 / 569 / 6 outcomes pass | 51.728 / 155.771 / 7.686 / 11.194 |
| Distribution full / internal qualification | All 72 variants / nine controls pass | 227.359 / 28.698 |
| SQL behavior, editor and host tools | All 21 categories; 438 ordinary tests pass | 890.102 total |
| Three-worker linked SQL protocol | Clean A, unchanged A, independent clean B; three equal executable hash sets and seven mutation controls | 1,045.844 |
| Real source/installed admission | Version-matched package, identity/diagnostic parity, integrity and recovery | 258.846 |
| Version-matched reference corpus | All 732 fixtures; initial test preparation 17m03s, assertion body 108.83s | 1,137.507 |
| Complete installed qualification | 732 public emissions/maps, four native cases, relocation, editor generations, rejection/recovery | 901.000 |

These are named category observations with their preparation included, not an
additive full-gate duration or matched before/after speedup. Optional ignored
live-database tests are not passes. The SQL protocol still exceeds ten minutes;
V1 feasibility and complete integrated acceptance remain open.

All 1,464 source/Rust files, source maps and remaining corpus fields match the
accepted V1b3 output. Only the already declared duplicate-free unordered module
set compares as a set. The real compiler/metadata identities change. A separately
recorded identity-aligned diagnostic proves complete physical metadata equality
under controlled baseline compatibility; it is never published as actual product
identity. The actual corrected candidate is used throughout final qualification.
Its explicit identity build-feature declaration leaves all 490 observed Cargo
package/target/feature/profile artifact entries unchanged from the preceding
implementation. The unchanged SQL stage reports every unit fresh, and both
independent clean stages report every unit non-fresh. All owned protocol targets
are removed.

Preserve all earlier failures and their observed candidate identities: the moved
import/inventory/guard omissions, unsuccessful capacity-copy operation, and the
external reference setup that paired a 0.0.0-linked test compiler with beta sources.
That reference correctly failed version admission before any fixture executed.
The replacement uses the canonical matching version/source contract and a retained
version-owned target. No version check was weakened. The split-brain guard still
reports its three proven pre-existing test-include findings and is **not a pass**;
V4 owns its repair along with the previously recorded field-checker work.

`v1b4c-final-acceptance.json` binds all 56 passing categories to exact-candidate
specifications, receipts, outputs and resource observations. The complete portable
packet `v1b4c-evidence.tar.gz` has SHA-256
`4dddd43299c656b263b3eb67baf4cdb969d365ffca0ffce55277c3a27ff6187f`.
All 5,342 manifest file hashes and sizes were verified locally before review.
Independent review is **SATISFIED**, with no blockers, outside the reviewed tree
at `reviews/27859a22fcc61bc959a3ee1243fa330d7adb2513/response.md` (SHA-256
`0f63b17c5614db7e637b662a6f5b1faccb20eba55fe3914fb34a219991578312`).
It independently recomputes all 280 per-category evidence hashes and confirms
ownership, admission, publication/lease ordering, tests and graph separation.
This closure changes phase records only.

Follow-ups for the remaining phase: V4 must register recurring execution of the
new 1,307-input semantic/native comparison, make the graph self-test survive
Python optimization, and retain the native-map invariant in release builds rather
than relying only on `debug_assert_eq!`. Preserve the established editor host
lifecycle when V2 adds bounded compiled-component reuse; analysis already retains
its Engine across configured reloads. V3/V4 retain version/profile invalidation,
large debug-artifact preparation/copy costs and complete fixture ownership.
Installed command attribution is 492.545s for public emits, 105.609s for native
cases and 108.075s for other recorded commands; the remaining 194.771s includes
copies, hashing, editor operations, setup/reporting and cleanup and is not a pure
I/O estimate. Full final create-PR/merge correctness gates and V5 remain mandatory.

#### Additional same-source worker-allocation diagnostic

After named qualification, a six-worker trial on the same frozen implementation
passes the complete canonical SQL protocol in **788.350s**, versus **1,045.844s**
with three workers. Clean A / unchanged A / clean B take 385.659 / 8.108 / 392.886s.
All 490 actual package/target/feature/profile entries match the earlier protocol;
each clean stage has zero fresh entries and the unchanged stage has 490. All
three executable hashes match within the trial, all seven controls pass, and
all owned targets are removed. Two-second process-tree RSS samples reach about
4.07 GiB; shared pages may be counted twice and sampling is not an exact peak.
The declared allocation is eight GiB plus the unchanged two-GiB memory reserve,
six Cargo workers and two reserved CPU slots. No other build overlaps the trial.

The observed reduction is 257.493s (24.6%) in one sequential same-source comparison,
not a statistical result or a complete-gate saving. Independent protocol path maps
change embedded product identities, so cross-protocol binary equality is not
claimed. Both complete measurements include preparation and cleanup. Six workers
still do not meet 600 seconds. This is diagnostic evidence, not an adopted resource
policy. The 36-file supplemental packet `v1b4j6-evidence.tar.gz`, SHA-256
`0374e74c9230d61c4661da222373acdd9c25bba0cdbd9972ec31ec2154f6f4be`,
was independently hash/size verified after collection. It supplements the reviewed
implementation without changing code or replacing the earlier protocol evidence.

### V1c bounded investigation — overlap independent SQL builds

The six-worker sequential experiment still exceeds 600 seconds. Investigate
whether the two independent clean builds can run concurrently without losing
reproducibility coverage. Independent output trees are a required property;
the existing temporal serialization is not itself a comparison assertion. Do
not assume this makes the complete gate fit: measure the whole protocol first.

Start with an external diagnostic using the existing canonical host selection,
locked/offline identities, path maps, `build_native`, artifact selectors and all
seven rejection controls. Keep empty distinct A/B targets, the unchanged A
rebuild, three linked executable comparisons, actual clean/reused Cargo artifact
freshness and cleanup. Inspect build-script output ownership and confirm no B
output/input points into A. Hold source/configuration fixed; record actual start
and completion order. Shared registry source and operating-system caches remain
as in the sequential protocol, never a substitute for independent build output.

Admit a total of six Cargo jobs, three per clean build, with two CPU slots
reserved, an eight-GiB memory allocation plus the existing two-GiB reserve, and
12 GiB of disk allocation plus the existing 20-GiB reserve. Record sampled
aggregate residency and actual capacity; reject insufficient admission rather
than lowering reserves. No other build may overlap this diagnostic. Retain the
30-minute cap and all failed/incomplete receipts. Compare complete elapsed time
with the same-source sequential observations; a single trial is diagnostic,
not V5 acceptance or an additive savings estimate.

Production adoption is conditional on the measured result and explicit process
ownership. Use bounded shared supervision for the two build commands; preserve
signal/deadline propagation, bounded output, child/descendant cleanup, failure
reporting and removal of owned temporary outputs only after all consumers stop.
Test sibling failure, cancellation, deadline, malformed/missing outputs and
insufficient capacity. Do not call the main-thread-only `process_execution.execute`
from threads or introduce unowned workers. The source change must remain a small
explicit independent-build schedule, not a general daemon or distributed runner.
Keep the historical sequential observations as the diagnostic reference; do not
add an automatic fallback after a concurrent failure.

Named acceptance includes focused orchestration and process-owner mutation tests,
all seven original SQL controls, the actual complete linked native protocol and
resolved unit/feature/profile parity, source ownership and cleanup proofs, and
independent exact-candidate review. Compiler/runtime implementations are unchanged
by this scheduling item; retain their already observed evidence without relabeling
it as execution on a new candidate. Complete final gates and V5 remain mandatory.

### V1c outcome — concurrent clean builds not adopted

The diagnostic on `f2392670efd4840ce5294c81a03a56e4549e6fc4` preserves all
490 actual Cargo unit signatures, clean/reused freshness, all three linked
executable comparisons, seven rejection controls and owned-target cleanup.
The two independent builds overlap for 766.919s. Clean A / unchanged A / clean B
execute in 769.631 / 6.941 / 767.498s; complete protocol time is **780.353s**.
Functional qualification passes; the 600-second timing objective fails.
Compared with 788.350s for sequential builds at the same total six-worker budget,
this one observation is too small a gain to establish a benefit. Do not adopt
production concurrent scheduling or add process-supervision complexity for it.

The host exposes eight logical CPUs on four physical cores with SMT2. This
explains why logical-worker counts must not be treated as independent physical
capacity; it does not establish the exact cause of the observed timing. Sampled
aggregate process RSS reaches about 4.88 GiB (shared pages may be double counted),
and sampled available memory remains above 6.18 GiB. Sampling started after early
preparation and is not an exact peak. Source and reserves remain intact; 986
noncomment dependency files and artifact/build-output records show no cross-target
build dependency. No implementation source changed.

The 38-file packet `v1c-evidence.tar.gz` has SHA-256
`9ce22fe69bd212d6a6bb7f333c87204082e8eb2a4c51ea8c17af4391b7467d11`;
all manifest hashes and sizes were checked after collection. It retains complete
commands, receipts, ordering, resource samples and ownership evidence. This
record-only outcome needs documentation checks, not another implementation review.
V1 feasibility and complete final gates remain open.

### V2b bounded implementation — compilation-scoped component reuse

Prioritize eliminating repeated compilation before changing Wasmtime optimization
or thread policy. Keep the current Engine configuration, dependency graph and
fresh guest execution semantics. Add an explicit cloneable compilation owner
containing one Engine and a bounded in-memory compiled-component LRU. Key entries
by the artifact hash only after verifying actual bytes and all request/registration
contracts. Bind reuse to the exact owner/Engine policy, rejecting incompatible
stack policy. Bound retained entry count and mapped compiled-image bytes; document
that active executions and compilation working memory are not cache residency.
Do not serialize native code, add a global owner, cache failures or reuse Stores.
Release the owner lock before guest instantiation/execution.

Every execution retains a fresh Store, instance, fuel and resource limits. Response
cache semantics remain separate: compiled reuse alone never reports a response
cache hit or skips request execution. Publication follows successful compilation
and forbidden-import validation. Revalidate hash, protocol and registration on
hits. A changed valid artifact compiles independently; altered bytes with an old
hash reject even when path/mtime is unchanged.

Share the owner through prepared SQL profiles and their clones, including schema
preparation, typed methods and application queries. The editor must retain its
existing configured-reload host lifetime, release on disable, and publish nothing
on failed initialization. Empty profiles must not initialize an Engine. Verify
actual caller wiring, not only an isolated per-host cache. Preserve existing
compiler diagnostics, provider responses and emitted/runtime assertions.

Named checks: host tests with compile counts, hit/miss/eviction/image bounds,
incompatible policy, invalid hash/import/protocol, fuel/memory/output limits and
fresh-state controls; semantic-services profile/typed-method tests; complete SQL
providers including all six PostgreSQL artifacts; analysis/editor and LSP tests;
SQL frontend/driver/real CLI and affected source/dependency/coverage contracts.
Use the retained target with explicit preparation and a 30-minute category cap.
Compare exact responses in controls and retain existing fixture assertions. Record
complete category timing against prior observations without treating sums as a
full-gate result. Independent exact-candidate review precedes the intermediate
merge. Complete final create-PR/merge gates and V5 remain mandatory.


V2b qualification refinement, registered before generation: the canonical SQL
component producer inventory includes the host crate's source, even for changes
excluded by the guest target. Preserve that current inventory and regenerate all
ten SQL artifacts and producer receipts with the canonical pinned producers and
the retained owned guest target. Only each producer's declared output paths may
change during its run. Do not patch source fingerprints to bless prior binaries.
Compare all regenerated guest hashes with the prior artifacts, then execute the
complete provider and real SQL application assertions against the resulting
candidate. The independent words component has no dependency on the host crate;
verify its existing provenance rather than rebuilding unrelated inputs. Keep all
source-guard/provenance failures and the invalid larger-stack test setup as failed
evidence. Precise target-specific producer inventories remain V3 work, not an
exception introduced here. Extend the real application regression with configured
reload, changed schema, same-path/same-mtime corruption, recovery and disable/enable
checks; a configured reload passes its resident owner into profile preparation.

The complete producer-input comparison also finds three already stale inputs on
the V2b base: root Cargo.toml, root Cargo.lock and sifr_sql_contract/src/lib.rs.
They belong to earlier work in this same phase. Five additional host/test source
inputs change in V2b. All three SQL families have these eight differences; the
independent words producer has none. Preserve this attribution in
`v2b-provenance-drift-attribution.json`; do not attribute all producer drift to the
new cache or imply the earlier focused SQL checks included this qualification.
Canonical regeneration reconciles the current full phase-owned source state.


### V2b acceptance — bounded compiled-component ownership

Implementation candidate `dd65d042ae3414f0120b61cbbe297240df7b36a4`, based on
`fb76f78fbe87c3d197f279363b9f80b4ecda31d0`, is accepted through
[PR #4394](https://github.com/sifr-lang/sifr/pull/4394). Only phase records follow
that reviewed implementation. The compilation owner retains at most eight entries
and 128 MiB of mapped compiled images. These bounds exclude transient compilation
and active execution references. Engine configuration statements are unchanged.
Authenticated bytes, registration/protocol/request validation and fresh
Store/instance/fuel/memory execution remain authoritative; compiled reuse never
reports a response-cache hit. Prepared profiles, CLI query processors and editor
reloads share the owner. Requirement-only profiles retain their owner even when
there is no query host; empty profiles remain lazy and disabling releases ownership.

All **32 named categories pass on the exact candidate**, including **473 Rust test
cases** across their selected targets, the five coverage-area cases and the
component qualification's 39 mutation controls. Filtered driver/package checks
retain their declared scope; zero-test doc targets are not counted as executed
Rust cases. All 11 real application tests, 28 component-host tests, 14 semantic
service tests, all six PostgreSQL artifacts, 158 frontend unit tests, 71 analysis
unit tests and 114 LSP unit tests pass. Original component-host and application
test source is preserved with insertions only. Every receipt has no time/resource
stop, leftover process group or tracked-source mutation. The 30-minute cap,
20 GiB disk reserve and 2 GiB memory reserve remain enforced. Native categories
use two Cargo workers, two Rayon threads and one Rust test thread.

The following are individual category elapsed times including preparation. Their
**1,735.810-second sum is affected-item validation, not an integrated gate runtime**.

| Final-candidate category | Seconds | Result |
|---|---:|---|
| `component-qualification` | 1.742 | Pass |
| `guard-filesystem` | 16.248 | Pass |
| `method-policy` | 33.257 | Pass |
| `method-policy-selftest` | 1.240 | Pass |
| `sql-component-host` | 5.243 | Pass |
| `sifr_semantic_services` | 17.313 | Pass |
| `sql-native-application` | 479.715 | Pass |
| `sql-sifr_sql_postgresql-integration` | 81.800 | Pass |
| `format` | 8.245 | Pass |
| `resolved-direction` | 6.744 | Pass |
| `coverage-matrix` | 54.269 | Pass |
| `sql-sifr_sql_tool` | 37.299 | Pass |
| `sql-sifr_sql_sqlite_tools` | 38.276 | Pass |
| `sql-sifr_sql_mysql_tools` | 73.493 | Pass |
| `sql-sifr_sql_postgresql_tools` | 112.952 | Pass |
| `sql-sifr_frontend-integration` | 38.783 | Pass |
| `sql-sifr_sql_mysql-integration` | 25.846 | Pass |
| `sql-sifr_sql_sqlite-integration` | 14.393 | Pass |
| `sql-driver-sql` | 74.397 | Pass |
| `sql-pinned-python-preparation` | 0.750 | Pass |
| `sql-sifr_frontend-lib` | 45.336 | Pass |
| `sql-sifr_analysis-lib` | 75.304 | Pass |
| `sql-sifr_lsp-lib` | 252.474 | Pass |
| `sql-host-tool-package` | 77.291 | Pass |
| `sql-host-tool-cli` | 151.229 | Pass |
| `sql-guard-schema_profiles` | 0.740 | Pass |
| `sql-guard-query_substrate` | 0.742 | Pass |
| `sql-guard-schema_polymorphism` | 0.740 | Pass |
| `sql-guard-incremental_editor` | 0.738 | Pass |
| `sql-guard-host_tools` | 0.738 | Pass |
| `guard-file-size` | 7.738 | Pass |
| `filesystem-selftest` | 0.737 | Pass |

Real application execution is 400.66 seconds within its 479.715-second category;
Cargo reports 1m11s preparation. One-second stdout sampling attributes approximately
223 seconds to CLI/editor provider diagnostics, 74 seconds to valid-template
provider checks, 34 seconds to native execution and 25 seconds to the new
reload/schema/corruption/recovery control. These are approximate observations,
not instrumentation or a repeated performance qualification. The earlier
513d902ef candidate's application run passed in 609.877 seconds (406.83-second
body); the changed preparation cost must not be attributed to cache speedup.

The six-artifact PostgreSQL test body takes 38.16 seconds versus the earlier
189.57-second observation, approximately 80% lower in this comparison. Existing
provider assertions and exact-response/fresh-state controls remain. This is not
an additive full-gate saving or statistical guarantee. Preparation remains
material: PostgreSQL tooling reports 1m42s of preparation in 112.952 seconds;
the final host-tool CLI category reports 2m20s in 151.229 seconds.

Canonical regeneration passes for MySQL (38.506s), SQLite (62.668s) and PostgreSQL
(297.284s), with all ten current artifacts and all four producer provenance
records validated. The three inherited and five current changed producer inputs
remain separately attributed. Recursive section comparison locates guest byte
differences in nested module name custom sections; execution against the exact
regenerated bytes is still required and passed. No product dependency, tool
version or Wasmtime Engine setting changed. Historical invalid-stack fixture,
source-guard, provenance and inventory failures, plus intentionally incomplete
intermediate batches, remain preserved as failures/incomplete evidence.

Evidence packet: `v2b-evidence.tar.gz`, SHA-256
`a3888418af22212e9c67aac1f02b0e2a9e761d047b6d6dfaf8592b0a16ed97d7`.
The packet's 320 manifest entries were independently hash-verified after transfer;
`v2b-final2-summary.json` binds commands, results, preparation and test summaries
to the candidate. Exact-candidate independent review is **SATISFIED**, no blockers:
`reviews/dd65d042ae3414f0120b61cbbe297240df7b36a4/response.md`, SHA-256
`73c2cdec5755d4d982e1c5fc943d8acc5c17f036b86f3cd5795aeb728fc69094`.
The review's pending-validation note is resolved by the complete final2 batch;
no code changed after review.

Follow-ups remain explicitly owned: V4 guard/lifecycle coverage adds an ordering
mutation for hash-before-lookup and an actual requirement-only editor reload
regression. The clone/policy test and source inspection do not substitute for
that latter caller regression. Per-hash compilation slots are only a future
suggestion if parallel profile preparation is introduced; no additional cache
coordination is justified now. V3 retains target-specific producer inventories.
V2 durable history, V3/V4 preparation and build reuse, complete final
create-PR/merge correctness gates and V5 remain open.

### V4b bounded investigation — CLI preparation ownership in LSP parity

Bring this attribution work forward before another implementation. The current
LSP unit helper `cli_sysroot_status` invokes `cargo run --locked -q -p sifr`
internally. The subsequent host-tool CLI category rebuilt compiler crates despite
unchanged source. This is evidence of hidden preparation, but the exact reason for
the return rebuild must be established before claiming a cache defect or savings.

Record a retained A/B/A experiment: unchanged CLI test preparation, the exact
existing LSP sysroot parity test with nested Cargo fingerprint logging, then the
same CLI preparation again. Use owned retained targets, exact candidate/binary
identities, selected tool versions, capacity preflight and the 30-minute cap.
Capture actual dirty reasons, resolved feature/build-unit differences, binary
identities, compilation and execution times. Do not run two builds on the same
target concurrently or use a source edit to manufacture invalidation.

If the ownership transition is confirmed, prefer placing the real CLI/LSP parity
assertions in the CLI-owned integration boundary, which already receives Cargo's
exact built binary. Preserve every existing equality assertion and add transport
coverage without exposing production internals solely for tests. A moved test
must have an explicit old/new coverage mapping and remain selected in canonical
profiles. Do not add ambient-binary discovery, hidden Cargo preparation, stale
artifact fallback or a new service. Register the concrete implementation and
named checks after attribution, before editing source. This investigation is not
a waiver of full final gates or a sub-ten-minute result.


V4b first attribution result (implementation unchanged): retained CLI preparation
reports 458/458 fresh artifacts before and after executing the existing LSP
parity test directly. The three categories pass in 6.202s, 5.695s and 5.685s.
The compiler and LSP test binary hashes/inodes remain unchanged. This rejects
attributing the earlier 2m20s CLI preparation to the nested command alone; it
does not measure the outer LSP test preparation or intervening package-test step.
Before implementation, extend the control to include exact LSP library
preparation, the same parity execution, CLI preparation, the original package
host-tool category, then CLI preparation again. Capture artifact freshness and
actual fingerprint reasons at each transition. Do not relocate a test or claim
rebuild savings without evidence supporting the relevant preparation boundary.


### V4b attribution closure — no repeated CLI rebuild reproduced

Recorded through [PR #4395](https://github.com/sifr-lang/sifr/pull/4395).

No implementation, test selection or coverage change is made. On unchanged
implementation candidate `dd65d042ae3414f0120b61cbbe297240df7b36a4`, all nine
bounded diagnostic categories pass. The merged phase plan differs from that
candidate only in Markdown; source and retained targets remain fixed.

| Controlled category | Seconds | Cargo artifact result |
|---|---:|---|
| CLI test preparation A | 6.202 | 458/458 fresh |
| Existing LSP parity test, direct execution | 5.695 | Real CLI command and original assertions pass |
| CLI test preparation A again | 5.685 | 458/458 fresh |
| Outer LSP library preparation | 24.702 | 449/450 fresh; one LSP test binary rebuilt |
| Parity test after outer preparation | 4.683 | Real CLI command and original assertions pass |
| CLI preparation after LSP preparation/execution | 4.684 | 458/458 fresh |
| Original package host-tool category | 2.685 | 293/293 fresh; seven selected tests pass |
| CLI preparation after package transition | 4.682 | 458/458 fresh |
| Settled LSP preparation | 3.182 | 450/450 fresh |

Cargo attributes the one LSP test rebuild to `FsStatusOutdated(StaleDependency)`:
its build-script dependency has a newer timestamp. The rebuilt LSP test binary
is byte-identical. Across all nine artifact snapshots, the compiler SHA-256 is
`f222589d5f61f6b08cea7e3119470c34847bb8f1acdcebc8a2cf16693ef14dce`
and LSP test SHA-256 is
`d715cd96e7da4f54db534f77bb8119b5661a5833cb67d2eca1a46f78a9145f90`.
Snapshot/hash collection is recorded outside category timings; these are attribution
experiments, not full-gate measurements. Every category retains the cap/reserves,
cleanup and source-change checks; no category stopped or left a process group.

The controlled evidence rejects blaming the earlier 140-second CLI preparation
on repeated invalidation from these unchanged LSP/package transitions. That older
run has no captured Cargo dirty reasons, so its precise cause remains unproven.
Do not infer that all caches are correct in every edit scenario. The proposed
parity-test relocation is not pursued on this evidence. The meaningful next
question is compilation cost on cache misses and after real input edits.

Evidence packet `v4b-evidence.tar.gz`, SHA-256
`d43564868df61e3a52ffb3f753e1f64ae86c3d8525841e046d11dacf1eb048c4`,
contains 72 hash-verified manifest entries, including full fingerprint logs,
artifact freshness, snapshots, commands and unchanged-source receipts. This
record-only closure needs documentation checks, not another external code review
or a repeated full gate. Final integrated gates and V5 remain open.

### V1d bounded investigation — build-only Rust optimization

Refresh cold SQL critical-path evidence on the current implementation before
adopting a profile change. The prior six-worker independent-clean timing at
`27859a22fcc61bc959a3ee1243fa330d7adb2513` includes Cranelift codegen (197.41s),
lowering (168.35s), frontend (107.77s), metadata (79.83s), PostgreSQL C preparation
(57.16s) and SQL-tool identity build scripts (about 20–23s each). These durations
overlap and are not additive. The canonical native qualification already sets
`CARGO_PROFILE_DEV_DEBUG=0`; resolved `cc` already enables `parallel`. Neither
repeating those settings nor extrapolating older timings is an optimization.
The captured effective profiles identify build-only `sifr_identity` and `sha2`
as optimization level zero, while target/runtime versions retain levels one/two.

First measure the existing build-helper workload and a standard Cargo build-only
optimization override. Keep normal target/runtime optimization, Engine policy,
feature selection, dependency versions, fixture assertions and qualification
scope unchanged. Verify actual emitted Cargo unit profiles rather than assuming
an environment/configuration override took effect. Do not infer end-to-end gains
from a helper micro-measurement.

For any promising candidate, compare the complete canonical clean-A, unchanged-A,
independent-clean-B SQL protocol on the same source and declared six-worker
allocation. Preserve linked binary identity/equality within each protocol, all
seven rejection controls, preparation and cleanup, and zero-fresh/complete-fresh
expectations for clean/reused builds. Use owned targets, capacity preflight,
20 GiB disk and 2 GiB memory reserves, and a 30-minute category cap. Record changed
host-build profiles separately from unchanged native target profiles and compare
resolved features/units. A build-only override must not silently alter C compiler
optimization or normal runtime code. Retain failed/no-benefit experiments.

No production profile change is registered yet. If the diagnostic is beneficial,
register its exact manifest/configuration scope, input/provenance implications
and focused acceptance checks before implementation. Blanket opt0 remains
unaccepted. Compiler-edit costs, full current-host correctness coverage and the
complete sub-600-second V5 matrix remain mandatory.


### V1d diagnostic outcome — helper improvement is not a cold-gate improvement

Record PR: [#4396](https://github.com/sifr-lang/sifr/pull/4396).

On unchanged implementation `dd65d042ae3414f0120b61cbbe297240df7b36a4`, with the
prospective plan merged as `3a662f50326c928002160d1ea48db2631ae3fa44`, an external
build-dependency probe runs the actual `emit_local_token` and
`emit_product_identity` functions against the SQL tool's complete source inventory.
Its resolved external dependencies are a verified subset of the root lockfile.
Three repeated workloads have medians 12.394s at build-only opt0, 1.911s at
build-only opt1, and 2.859s with only `sifr_identity` and SHA-256 0.11.0 overridden
to opt1. All nine emitted identity outputs have the same SHA-256,
`d2c3893493d87b2be7695789eeeb60239a2669cd90cae8f0ca289ca799ae7084`.
These helper timings are not whole-build or whole-gate savings.

The complete canonical SQL protocol then runs sequentially for baseline and the
narrow two-library override, with the same frozen source, six Cargo workers,
resource allocation/reserves, debug-off policy and runtime optimization levels.
The actual override is version-qualified for SHA-256 0.11.0; version 0.10.9 is
unchanged. This is one matched diagnostic observation, not statistical acceptance.

| Stage | Baseline seconds | Selective helper override seconds |
|---|---:|---:|
| Clean A | 386.896 | 384.806 |
| Unchanged A | 7.021 | 6.888 |
| Independent clean B | 391.757 | 385.444 |
| Canonical protocol, including preparation and cleanup | 786.744 | 778.281 |
| Complete diagnostic category, including admission/observation overhead | 787.352 | 779.872 |

Both protocols pass all seven rejection controls, all three linked executable
comparisons, zero-fresh clean builds, complete-fresh unchanged builds and owned
target cleanup. All 490 resolved package/target/feature units match. Exactly two
host build-dependency units change opt0 to opt1; all 269 native target units and
all other emitted profile fields remain unchanged. No category reaches its cap,
violates resource reserves, changes source or leaves a process group behind.
Independent protocol path maps differ, so cross-protocol executable equality is
not claimed; equality holds across all three stages within each protocol.

In clean A, the three SQL-tool identity scripts fall from 18.38–19.59s to
4.02–4.61s. Their work overlaps other compilation. The complete diagnostic gain
is only 7.480s (0.95%), and both observations remain above 600 seconds. Do not
adopt this profile change as a demonstrated cold-gate optimization. V4 may revisit
it for a measured real edit cycle; no production manifest/profile, test selection
or component provenance changes are made here.

The trace also records PostgreSQL's build-script compile as unblocked by its
serde_json build dependency at 23.49s but starting at 293.33s, followed by its C
build at 307.54–363.84s. Treat this as queueing evidence, not proof that moving
source files or deleting a dependency will recover that interval. Cranelift,
lowering and frontend machine-code generation remain substantial overlapping
work. V1 feasibility and all final gates remain open.

The packet `v1d-diagnostic-evidence.tar.gz` has SHA-256
`99a02ba492a6bdd4231fa8b5f55224f5188e161788b8f8b463755770c01066ab`;
all 1,646 manifest hashes and sizes were independently verified after collection.
It contains commands, actual profiles, Cargo timings/build-script output, complete
protocol receipts, probe outputs and the source-frozen observation scripts.
This is a record-only closure and needs documentation checks only.

### V2c bounded investigation — durable-history repeated validation

Measure the current exact `history_over_4096_keeps_publishing_and_recent_aba`
test, retaining all 4,097 publications, periodic pruning, retention assertions,
recent ABA restoration and old-record eviction. Include Cargo preparation and
cleanup; preserve the 30-minute category cap, source freeze and existing disk/
memory reserves. Capture the existing history report when the test completes;
missing/capped execution is not a pass. Inspect the actual read/hash/decode and
filesystem work before choosing an implementation. Other history controls remain
separate named categories if the long test reaches the cap.

The source currently authenticates and deserializes every retained record during
generation admission, repeats admission before and after publication, and reads
records again for consumers. Its stored record digest JSON-serializes an already
serialized byte vector. A candidate to measure is a bounded Store-owned cache of
validated immutable record facts: read and authenticate current bytes on every
admission, bind reuse to both the observed byte digest and expected stored record
identity, and reuse only successfully validated schema/context/diagnostic facts.
A cache hit must never replace current file ownership, path, size, generation or
retention checks. Keep the existing disk format, stored digest definition, fsync,
publication and lease/GC semantics unless later evidence justifies and explicitly
registers a broader change. No timestamp-only trust, global cache or cached error.

Before implementation, register the concrete cache representation, residency
bounds and exact affected controls. Required acceptance includes all project-cache
and history tests, same-path/same-mtime corruption, mismatched record identity,
invalid schema/context/diagnostics, cache misses/hits/eviction and release, active
readers, concurrent publication, interrupted/crashed writers, pruning and storage
failure. Preserve every original assertion and all 4,097 publications. Add focused
controls for any new reuse path and compare measured complete baseline/candidate
work. Formatting, affected policy inventories, file-size checks and independent
exact-candidate review remain required. Full phase gates and V5 remain open.


### V2c measured baseline and concrete implementation contract

The exact 4,097-publication test passes on `dd65d042` in 449.354s overall;
Cargo reports 49.75s preparation and libtest reports 394.73s execution. The remaining
wrapper/Cargo overhead is included in the overall measurement. All original history,
pruning, ABA and eviction assertions execute. Retained records reach and remain at
128; final retained payload size is 216,320 bytes. Four observed lookup checkpoints
are 0.723ms, 31.394ms, 61.130ms and 23.435ms; these are individual observations.
Cargo records stale dependency timestamps during preparation; this is not proof of
repeated invalidation or a broken cache. The immediately repeated artifact preparation
is fresh and takes 4.183s. The current binary contains 37 project-cache tests across
five modules; retain that complete named inventory in candidate validation.

Read-only observation shows substantial logical reads and relatively little physical
reading. A partial 379-sample worker trace includes 157 samples waiting in
`jbd2_log_wait_commit` and 217 runnable samples; these proportions are not exact full-run
CPU/I/O accounting. Preserve the durability cost in all candidate measurements.

An authenticated 1,690-byte record is copied read-only into an external probe.
Across alternating probe rounds, reserving its validated length changes eight actual
read calls to two per read and reduces approximately 7.4us to 4.5us. The candidate
probe additionally pays for another metadata query. It returns identical bytes;
this small primitive improvement is not a large whole-test saving. A second probe
links the actual fresh frontend/identity libraries: median per 10,000 operations is
0.592289s for the existing stored digest, 0.112661s for decode/schema/context/diagnostic
validation, 0.706739s for their combined admission, and 0.093293s for a raw byte digest.
The raw digest alone is not a measured cache-hit implementation or whole-gate result.

Implement one bounded record-admission improvement in driver project storage:

- Allocate bounded read buffers from the already checked file length, while retaining
  the limit-plus-one read and post-read growth rejection. Reuse the single metadata
  observation for file type and length; keep ownership/path checks before opening.
- Add a Store-owned, generation-shared successful-fact LRU with at most 128 entries.
  Each entry contains only the 64-character stored identity and 64-character observed
  byte digest; do not retain decoded records or payload buffers. Bind the owner to one
  immutable semantic context and release it with the last Store/Generation owner.
- On every admission, read current bytes with existing file checks and hash them with
  domain separation. Reuse a fact only when both byte digest and expected stored
  identity match. On a miss, run the original stored-digest, decode, schema, context
  and diagnostics checks before insertion. Never retain a failed validation.
- Consumers still deserialize current authenticated bytes. Preserve original mismatch
  messages where the generation and consumer paths distinguish them. Keep every
  generation, size, record count, aggregate-history and lease check outside reuse.

Keep the exact stored digest and disk schema, hardlinks/copy behavior, all fsyncs,
pointer/hint publication, namespace/generation leases, pruning and cancellation.
No crate/dependency/profile change or new persistent artifact is in scope.

Named acceptance: all 37 original project-cache tests (the long history case has
its own 30-minute category); added hit/miss/eviction/zero-capacity/lifetime controls;
validly hashed invalid schema/context/diagnostic/JSON rejection; mismatched expected
identity; changed same-length bytes with restored mtime; permissions and symlink
substitution after a warm hit; bounded-read empty/limit/oversize/growth/error controls.
Run affected frontend persistence tests, formatting/file-size checks, method and
filesystem inventory contracts, and scoped independent exact-candidate review.
Compare complete history baseline/candidate times and observed assertion IDs without
claiming additive savings. Full phase gates and all V5 scenarios remain mandatory.


### V2c acceptance — bounded durable-record validation facts

PR [#4397](https://github.com/sifr-lang/sifr/pull/4397) merged as
`c9ebfb725ca1218c3aeaf73065353bf683f6b6e9`; final implementation candidate
`b58f45f6ea0ff04234081b4ea57c4fea98c8077d`. Every admission rereads the owned
current file and authenticates its raw bytes before reusing a successful fact
bound to the expected stored identity and immutable context. The LRU retains at
most 128 identity pairs, with no decoded payloads or failed validations. Bounded
reads reserve the checked file length and retain growth/error rejection.

All 37 original project-cache tests and eight new controls pass. The 44 shorter
cases take 25.710s overall / 22.69s execution. The full 4,097-publication history
case takes 196.401s overall / 193.58s execution, versus baseline 449.354s overall /
394.73s execution. This single before/after observation shows about 51% lower test
execution time. Baseline preparation rebuilt stale dependencies and included
lightweight process sampling; do not attribute the outer-time difference solely
to this implementation or claim a repeated full-gate result. All non-timing
fields at four history checkpoints match, including retention and byte counts.

All nine frontend persistence tests pass (2.194s overall / 0.11s execution).
Filesystem inventory passes on the final candidate (15.696s). Method policy,
filesystem/method self-tests, formatting and file-size checks passed on identical
Rust inputs before inventory-only commits. The canonical inventory generator's
unrelated manual-classification resets were rejected; outside-item row content
and ordering are unchanged. The final inventory adds 20 and removes three sites
owned by the two affected Rust files. Source comparison proves all five original
test files and the exact write_new, publish, point and prune_store bodies remain
unchanged. Durability, disk schema/digest, leases, hardlinks and pruning remain.

Evidence packet `v2c-evidence.tar.gz`, SHA-256
`16e67c100f4740ea6a0ef9e47a4c44c6caf5501997ce303d30bbd351c74e9570`, has 137
independently hash-verified entries. `v2c-acceptance-summary.json` reconciles every
original/new test against final enclosing all-pass counts, retaining nested child
output. Exact-candidate independent review is **SATISFIED**, no blockers:
`reviews/b58f45f6ea0ff04234081b4ea57c4fea98c8077d/response.md`, SHA-256
`12f1c49a4898bca240704762face7c1bf52f520cac6809d9c860d4f6f004d2af`.
The first review request failed authentication; its incomplete evidence is retained
and is not a pass. Initial missing-constructor-field and inventory failures remain
preserved. GitHub full-scope create-PR job 119152636146 failed admission because
SIFR_PERFORMANCE_REFERENCE was unset; its downstream tests did not run. CI policy
migration remains a later-stage owner, not evidence of local correctness.

Review suggestions: measure owner-mutex contention before changing miss locking;
retain awareness that aggregate-size rejection now precedes digest rejection on
multiply-invalid data; consumers still decode current authenticated bytes and a
miss performs full validation first. No speculative concurrency change is needed.
V1 build feasibility, V3/V4, complete final correctness gates and V5 remain open.

### V3a bounded investigation — generated materialization input authority

Before expanding shared build reuse, resolve the incomplete GCQ materialization
identity. Its hand-maintained producer list omits IR/type-system and newly split
compiler owners, filters source extensions, and caches a generated directory when
Cargo.toml merely exists. Inspect actual selected-compiler preparation, source /
project imports, external component/configuration inputs, output completion and
consumer lifecycle before choosing a replacement. Adding two crate names alone
cannot establish a complete input boundary.

Investigate whether canonical materialization on every invocation, followed by
content-preserving reconciliation of generated inputs into a leased stable owner,
can remove this duplicate semantic cache while retaining compatible Cargo
artifacts. This is a hypothesis, not an approved implementation or measured gain.
Compare actual representative simple, multi-file and runtime-bearing entries,
including compiler preparation, repeated unchanged materialization, source edits,
output bytes and mtimes. Inspect the existing materializer before duplicating its
identity or publication machinery. No source/profile/coverage changes during the
diagnostic. Retain the 30-minute category cap, owned paths, resource reserves,
failed evidence and clean source identity.

Register the concrete implementation and acceptance controls after these results.
Any subsequent implementation must preserve full assertions, input/permission /
symlink checks, failure recovery, active-owner and child leases, bounded retention,
relevant environment/toolchain/trust distinctions, and exact current outputs.
Full final correctness gates and all V5 scenarios remain mandatory. V4's previously
recorded hash-ordering mutation and real requirement-only editor reload regression
also remain owned follow-ups, not implied completed coverage.


### V3a findings and V3b implementation boundary — fresh GCQ inputs

The V3a probes on `2425ba2455cf69f0e64c6e7a4a13c622d8915f4c` reproduce two
correctness defects: changing copied compiler-owner inputs or an imported source
(with its mtime restored) leaves the GCQ entry key unchanged; a matching directory
containing only Cargo.toml bypasses compiler preparation and materialization.
The real CLI confirms that the imported-source edit changes generated Rust.
Correct canonical CLI materialization takes 3.184s / 3.684s / 7.192s on the second
simple / project / runtime request, and rewrites every output mtime despite
identical bytes. Process traces observe eight / twenty formatter processes in the
simple / runtime cases. Descendant process lifetimes overlap and are not additive.

A corrected external shared-CompilerContext probe uses canonical workspace
selection and the actual CLI product identity. All output bytes match the CLI for
all three sampled entries; its second materializations take 0.946s / 1.383s /
4.610s, with 1.122s one-time metadata preparation. These are diagnostic samples,
not complete-corpus or gate qualification. The earlier probe chose the wrong
workspace mode and is retained as invalid equivalence evidence. Likewise, the
initial CLI probe's unsupported offline invocation is retained as a probe failure.

V3b removes the duplicate persistent semantic cache from GCQ. Every materialization
request must call the canonically selected compiler using current inputs, into a
new private directory inside the caller's run root. Remove the incomplete producer
fingerprint rather than extending its list. Existing shared Cargo dependency
artifacts remain under their current owner. Existing run diagnostics and failed
outputs remain retained; this step creates no reusable cross-run output store and
adds no pruning of other owners. Private output names isolate consumers, including
concurrent requests. Preparation still covers every positive corpus entry and all
required companions; all profile selections and assertions remain unchanged.

This correctness prerequisite may increase intermediate runtime. It does not
claim a speedup or ten-minute acceptance. A subsequent registered item must address
canonical batch preparation, duplicate formatting/publication, dependency-target
identity and bounded persistent-owner retention using measured complete inventories.
Do not restore an approximate Python input fingerprint to recover speed.

Named V3b acceptance: focused controls for repeated fresh materialization, changed
same-mtime imported/compiler inputs (including moved stdlib owners and non-listed
extensions), incomplete old output, compiler admission failure, failed partial
output followed by successful retry, concurrent consumer isolation, and unchanged
Cargo dependency-target routing. Reconcile unchanged entry/profile inventories.
Run existing GCQ source-quality/selection tests, generated Cargo preparation policy
checks, file-size guardrails, and real canonical CLI materialization for all 93
positive entries in bounded categories, preserving every failure. Compare generated
bytes for representative entries and exercise same-mtime provider edits. Require
independent exact-candidate review. Full final phase gates and V5 remain open.


### V3b acceptance — fresh, private GCQ materialization

PR [#4398](https://github.com/sifr-lang/sifr/pull/4398) merged as
`367e379f1adfeffa158bda7968f5797fc679f5a4`; final implementation candidate
`07165950b3333ff7808413de1705aa96f4a0c470`. The incomplete producer fingerprint
and manifest-only shortcut are removed. Every materialization request invokes the
canonically selected compiler into a private directory. A fixed `entry-` prefix
keeps entry labels out of directory selection, including relative traversal,
absolute paths and oversized labels. Existing caller cleanup/failure retention,
compiler selection and Cargo dependency-target routing remain unchanged.

All 14 GCQ self-tests and 45 preparation-policy tests pass. Existing Clippy tests
are unchanged; nine materialization controls include the original moved-stdlib
owner inputs, omitted compiler owners/extensions, same-mtime changes, incomplete
output, compiler admission failure, failed partial output/retry, concurrent
consumers and path-label isolation. The controls reject the previous implementation.
An intermediate candidate's label escape was reproduced in owned temporary space
and corrected before review. Final inventory (20.696s), file-size checks (5.641s),
all seven real smoke gates (93.734s) and focused Clippy checks (16.689s) pass.

All 93 positive entries materialize through the real compiler. Category durations:
concurrency 73.229s; demos 245.332s; E2E representatives 445.939s; multi-module
projects 35.704s; stdlib flows 120.754s. Their sequential aggregate is 920.959s.
These runs use a new GCQ target context, which also affects native probe identity;
this is functional qualification and context-preparation cost, not steady-state
performance acceptance. All 93 portable graphs fetch locked without manifest/lock
drift (36.202s). Comparison with the earlier candidate covers every one of 452
output files: 270 are byte-identical; 182 Cargo manifest/lock files differ only in
the expected exact Git revision. Four real representative cases each pass three
fresh requests and an intentionally damaged previous-output control. A same-mtime
provider edit changes the new generated provider while preserving the earlier
consumer. Those real controls take 151.269s. The inventories remain 93 positive
entries, five negative seeds and 264 companion sources; companion execution is
not implied by this inventory proof. All other 27 production functions and the
complete area runner AST are unchanged.

Evidence `v3b-final-evidence.tar.gz`, SHA-256
`e68ac7561568ee4ba693c9f7291cc97c0605760fef1d05ce0bcf2e33eac5ad2e`, has 1,460
independently hash-verified files. Final acceptance is recorded in
`v3b-final-acceptance-summary.json`. Exact-candidate independent review is
**SATISFIED**, no blockers: `reviews/07165950b3333ff7808413de1705aa96f4a0c470/response.md`,
SHA-256 `7cee3b618fc7b7ae4818e7ced4a629c8ba31be2f1a0deab1faa69abd03068a73`.
Earlier failures remain preserved: incomplete review-checkout corpus; an external
capacity-inventory mistake before tests started; V3a's wrong-mode and offline
command probes; and the pre-fix label escape. None is relabeled as a passing run.

GitHub job 119176414793 fails full-scope admission without
SIFR_PERFORMANCE_REFERENCE. Job 119176415265 reports a missing first E2E run
signature after its unchanged direct Cargo entrypoint; the underlying temporary
run log was not uploaded, so its underlying cause is unknown. Preserve both logs;
CI/runner diagnostics and final integrated qualification own these failures. No
CI or full-phase pass is claimed. The scoped local checks and review meet the
approved intermediate policy.

Review follow-ups: repeated preparation creates new small source directories
under the revision-owned preparation root; bounded retention needs an explicit
consumer/lease lifetime before pruning. Compiler-versus-generated-Cargo target
routing remains a separate defect. V1 build feasibility, remaining V3/V4 work,
complete final correctness gates and V5 remain open.

### V3c bounded investigation — preserve the compiler's Cargo context

The generic GCQ `run_command` currently overwrites CARGO_TARGET_DIR for every
command when SIFR_GCQ_SHARED_ROOT is set, including Sifr compiler and emit calls.
`NativeToolchain` explicitly binds CARGO_TARGET_DIR into its environment identity;
that identity also participates in native-family and normal Cargo seed identities.
The profile's revision-owned GCQ root therefore changes the compiler's native probe
context even when its actual compiler, source inputs and caller target are unchanged.
This is a code-verified ownership problem; its isolated runtime benefit is unmeasured.

First compare the unchanged wrapper with a diagnostic adapter that explicitly
preserves the outer target for compiler calls. Use the canonical selected compiler,
frozen sources, the same flags/working directory and private materialization in both
cases. Exercise simple, multi-module, runtime and networking cases under different
fresh GCQ roots, including repeated requests. Record actual target context, selected
compiler identity, native-family observations, output bytes and category timing.
Distinguish new-context preparation from compatible reuse; do not present either as
a complete gate or an additive saving. Retain capacity/allocation/reserve checks,
owned targets, failure evidence and the 30-minute category cap.

If validated, register the concrete implementation and acceptance before editing:
make generic process execution inherit the caller environment, and pass the existing
generated Cargo target explicitly at the generated-Cargo call sites. Audit corpus,
native, rustfmt, demos and Clippy helpers rather than guessing ownership from an
executable's filename. Preserve every argv/flag/wrapper and all actual assertions.
Do not remove CARGO_TARGET_DIR from NativeToolchain identity, change portable source
qualification, add an output-success cache, or broaden generated Cargo target sharing
without a separate lifetime/lease contract. Preparation-source retention remains an
owned follow-up. Complete final phase gates and V5 remain mandatory.


### V3c implementation contract — explicit process context ownership

The first three frozen-candidate comparisons completed: current wrapper/new GCQ
root 179.788s, preserved compiler context/initial root 174.284s, preserved context/
second new GCQ root 101.743s. Materialization alone was 169.215s, 166.023s and
93.487s respectively. All 24 outputs match byte for byte under the same selected
compiler; the third run creates no new native family. At registration, the reverse-order current
wrapper control was still running against its unchanged checkout; its completed
result is recorded below. This demonstrates
avoidable repeated context setup, not a full-gate timing forecast. Evidence belongs
to `v3c-context-probe`; preserve all four categories before final acceptance.

Implement only explicit process-context ownership: generic `run_command` inherits
the caller environment unless given an explicit target. Corpus Cargo check,
Cargo fmt and native Cargo build select `cargo_target_for_run` themselves. Existing
Clippy calls already select that target. With a profile-owned shared root, generated
Cargo retains that exact target; standalone gates without it use their run-owned
target instead of the ambient compiler target. Compiler build/emit, freshness and
control scripts, rustfmt and produced executables inherit the caller environment.
Do not infer ownership from executable names or change compiler cache identities,
Cargo flags, wrapper selection, fixtures, assertions, profile membership or leases.

Acceptance: new behavioral controls must reject the old implicit routing, preserve
all caller variables (including unset target), isolate explicit child target changes,
and retain failure/truncation handling. Exercise corpus/fmt/native ownership with
and without shared roots, including native execution after Cargo returns its artifact.
Retain the previous materialization and Clippy controls. Run GCQ self-tests,
generated preparation-policy tests, file-size guardrails and selection/inventory
contracts. Materialize every one of the 93 positive entries through the final
compiler, compare every output to the qualified predecessor with only the expected
literal revision difference allowed in Cargo manifests/locks, and fetch all graphs
locked. Run all seven smoke gates plus focused Clippy. Exercise the complete demo
freshness protocol with an explicitly bounded two-worker allocation and preserve
its exact outcome; unrelated baseline drift retains its owner. Record all diagnostics,
capacity, input identities and the 30-minute per-category cap. Require exact-candidate
independent review; final integrated gates and the V5 timing matrix remain open.


### V3c acceptance — explicit compiler and generated-Cargo contexts

PR [#4399](https://github.com/sifr-lang/sifr/pull/4399) merged as
`c874d7bfd1cb3786954939f06afd99d2dec6ec30`; reviewed and validated candidate
`aad7aaadf13277b5df3a0c3f208ad7cf89b7689d`. The merged tree matches that candidate.
Generic GCQ execution now inherits the caller environment, including compiler
build/emit, helper scripts and produced executables. Corpus check, Cargo fmt and
native Cargo build explicitly select their generated target; Clippy already did.
With a shared root, the target remains exactly shared-root/cargo-target. Without
one, standalone generated Cargo uses its run-owned target. Compiler cache identity,
argv/flags/wrappers, fixtures, assertions, selection and leases remain unchanged.

All 18 GCQ self-tests pass (1.145s), including four new controls with subcases for
present/absent caller targets/shared roots, complete environment preservation,
child-only overrides, failures/truncation, and corpus/fmt/native target ownership.
The previous implementation fails eight of those subcases with zero harness errors.
All 45 preparation-policy tests pass (3.140s), as do file-size checks (7.140s),
structural/selection contracts (0.645s) and complete-checkout inventory (20.687s).
All previous test classes are unchanged. Removing only the target-routing additions
leaves the production assertion/argv AST unchanged; all six generated Cargo calls
have explicit targets. Inventories remain 93 positive, five negative and 264 companion
sources, with no profile or compiler/native-family implementation changes.

All 93 positive entries materialize through the final canonical compiler:
concurrency 62.234s, demos 186.303s, E2E representatives 440.441s, multi-module
36.707s, stdlib flows 119.258s; sequential aggregate **844.943s**. All 452 output
files match the qualified predecessor: 270 exactly, 182 Cargo manifest/lock files
only after substituting the expected exact revision. All 93 graphs fetch locked
without manifest/lock drift (36.711s). Real private-output, damaged-old-output and
same-mtime provider controls pass (149.774s). All seven smoke gates pass (95.236s),
as does focused Clippy with its controls (16.696s). Complete freshness passes for
all 264 companion sources (351.889s), using the actual freshness/debt protocol with
an external two-worker allocation override. Production worker policy is unchanged.
No resource cap, orphan cleanup or tracked source mutation occurred in these runs.

The completed frozen-candidate context comparison has four categories:
current wrapper/new root 179.788s; preserved context/initial root 174.284s; preserved
context/another new GCQ root 101.743s; current wrapper/reverse-order new root
180.791s. All 32 materializations match byte for byte with one selected compiler
identity. A/B/D create a new native family; C creates none. Networking first
requests take approximately 95/94/34/100s, while repeated requests take about 34s.
This isolates avoidable repeated context setup. It is not a complete-gate forecast;
the remaining 845s materialization cost alone still exceeds the phase objective.
Do not add these savings to overlapping earlier measurements.

Evidence `v3c-final-evidence.tar.gz`, SHA-256
`cf42ecb067d777c279db72dead8b0fe8cb761e3e3ff0d46e26acb2b5d726a3b5`, contains 866
independently verified manifest files. `v3c-final-acceptance-summary.json` binds the
final receipts and output comparisons to the candidate. Independent review is
**SATISFIED**, no blockers, in
`reviews/aad7aaadf13277b5df3a0c3f208ad7cf89b7689d/response.md`, SHA-256
`56bf5e1c115acd596789a8232aba56c952a365679d8f9c5d6d8919acff05342f`.
The reviewer confirms target ownership between preparation and execution and all
six Cargo call sites. The formerly pending reverse-order observation is now closed.
Future gate additions retain responsibility for explicit target routing and controls.

CI is not claimed green: job 119193367056 stops at missing
SIFR_PERFORMANCE_REFERENCE; job 119193367364 reports a missing first E2E signature
through the unchanged direct Cargo entrypoint. Its underlying temporary run log is
not published, so the underlying test failure remains unknown. Both logs are retained;
CI/runner diagnostics and final integrated qualification retain ownership. Initial
log retrieval failures are recorded separately and are not validation results.
V1 feasibility, native/Python fixture reuse, preparation-source retention, typed
within-invocation artifact ownership, global budgets, complete final gates and V5
remain open. No full-phase or sub-ten-minute acceptance is claimed.

### V4c bounded investigation — repeated compiler setup across the complete corpus

The canonical 93-entry materialization still costs 844.943s and complete companion
freshness 351.889s with the V3c ownership fix. Earlier V3a experiments matched CLI
bytes for only three entries when using the correct workspace mode and a shared
CompilerContext; the wrong-mode predecessor is invalid evidence. Do not generalize
that small prototype or adopt a batch interface from those measurements alone.

First audit the current CLI build/materialize and emit paths against the public
CompilerContext APIs. Identify preparation and per-entry validation explicitly:
workspace selection, application profile, compiler/product identity, metadata,
Rust probes, diagnostics, private outputs and failure behavior. A byte match cannot
prove that a required validation step ran; retain an assertion/path inventory too.

Keep the complete qualified V3c checkout/candidate frozen for diagnostic comparison.
Capture its canonical Cargo artifact messages and selected compiler/product identity;
bind any external Rust prototype to those exact libraries and tool versions. Run
both sides with identical working directory, PATH, target, flags, wrapper and resource
allocation. Compare fresh-context and shared-context API execution without adding
an output-success cache. Every entry must still pass through canonical compilation.
Cover all 93 positive entries and all 264 companions in capped categories, retaining
complete output/diagnostic differences. Exercise failing input, same-mtime imported
edits, failed-then-successful requests, repeated entry order and private consumer
isolation where the proposed lifetime is relevant. Preserve distinct intentional
determinism repetitions; do not merge semantically different modes or input owners.

Measure preparation, per-entry work, actual process/probe behavior and memory. Use
the established allocation/reserve checks and 30-minute category cap. Preserve
failed/incomplete evidence, including a prototype that omits canonical behavior.
Register a concrete production boundary and acceptance contract only after the
complete comparison shows a useful, correct simplification. No production batching,
new daemon, profile change or assertion deduplication is authorized by this diagnostic
entry alone. The user's full phase scope remains active, including final integrated
correctness gates and the complete V5 scenario matrix.

### V4c closure — complete corpus and compiler setup measurements

Diagnostic candidate `aad7aaadf13277b5df3a0c3f208ad7cf89b7689d` remained frozen;
registration `ff6e20279e6221e846e985252d67d4bf9bcafc03` changes documentation only.
The external probes link the exact libraries selected by canonical locked Cargo
preparation, use the qualified compiler/product identity, and preserve CLI package
admission, workspace/profile selection, source providers and operation-specific
Rust checks. All 714 requests completed: 93 materializations and 264 emissions in
each of fresh-context and shared-context modes. All generated bytes match the
qualified CLI corpus. All diagnostics match between the API modes; each of the
three nonempty companion warnings additionally matches an actual CLI invocation.

| Complete diagnostic category | Fresh context per request | Shared context |
| --- | ---: | ---: |
| Materialize all 93 positive entries | 592.028s | 588.028s |
| Emit all 264 companions | 365.885s | 346.372s |

Both modes above run in one process. A fresh context is **not** a fresh process:
the existing metadata producer already retains successful immutable validation
facts per process. Context sharing alone saves less than 1% of materialization and
about 5.3% of emission in this single ordered comparison. This does not justify
adopting shared-context lifetime complexity. No production batching, daemon,
context sharing, output-success cache, profile or assertion selection was changed.

The 18-request control sequence also completes under fresh contexts (59.717s),
shared context (57.215s) and CLI (96.239s). It covers repeated and reordered input,
same-size/same-mtime imported edits, two expected invalid inputs and recovery,
unrelated input, warnings and damaged old consumers. Every output and diagnostic
matches; altered input changes the result, recovery restores it, and new outputs
remain independent of damaged old outputs. Actual invocation capture confirms 41
Rust probes for networking materialization and zero for its emit operation in both
API modes. CLI networking controls were traced, so their time is instrumented and
is not an uninstrumented speed comparison.

Three independent metadata processes measure first complete preparation at
1.099/1.090/1.091s, subsequent preparation at 0.030–0.031s, and a fresh context's
provider selection/open after preparation at 0.064–0.068s. Consumer load statistics
alone omit the first complete preparation. Earlier V3a prototypes also omitted CLI
package admission; their byte comparisons stand but they cannot establish an
end-to-end speedup or attribute all startup cost to metadata. Canonical admission
alone measures approximately 0.69–0.72s in five observations. A source audit confirms
that each new request still captures inputs and checks current metadata bytes;
shared-context generation freshness must not be assumed equivalent for future work.

Process traces independently corroborate the call inventory. Simple materialization
executes one Cargo metadata operation (0.724s) and eight rustfmt children (0.360s).
Networking materialization executes 41 Cargo checks (13.355s aggregate interval
union), one metadata operation (0.697s) and 26 rustfmt children (1.351s), within an
instrumented 52.903s invocation. Networking emission executes no Cargo checks and
16 rustfmt children (0.843s), within 4.892s. Descendants are grouped under their
direct compiler child; intervals are not double-counted as parent-plus-child time.
These measurements do not explain the whole remaining parent-process cost and are
not full-gate estimates. Materialization GNU maximum RSS is 448660KiB fresh and
457432KiB shared; these are per-process maxima, not simultaneous process-tree peaks.
No resource cap, orphan cleanup or tracked source mutation occurred.

Two diagnostic harness failures remain preserved: an initial helper compilation
omitted `?` when comparing modification times, so no tests executed; the repaired
helper compiled successfully. A completed control validation then expected lowercase
`warning` instead of the actual serialized `Warning`. The saved unchanged results
were revalidated with the corrected assertion; the 54 control requests were not
rerun. Neither failure is presented as a compiler regression or silently discarded.

Evidence `v4c-diagnostic-evidence.tar.gz`, SHA-256
`c901eb79325c023ffee136bd03c0071225dbb03c7c866d615ec6b9efc553cd4d`, contains 1963
independently verified files, including sources, exact library identities, commands,
outputs, negative results, traces and receipts. The prior CLI 844.943s materialization
and 351.889s freshness observations have different process/cache/parallel execution
envelopes and are not matched speedup baselines. All 93 entries are diagnostic
qualification coverage; the current merge GCQ profile selects 12 representatives
(two native), while freshness covers all 264 companions. Do not treat 93-entry
qualification time as the merge profile's materialization time.

V4c investigation is complete. Repeated process preparation and Rust-probe work
remain candidates for a separately registered simplification, with canonical
admission and every required check retained. V1 feasibility, native/Python fixture
lifetime, V4 integration, complete final gates and V5 remain open.

### V4d contract — conservative test include reachability

Repair the phase-owned split-brain guard's three reproduced false positives before
collecting the next integrated failure inventory. The current source walk follows
`mod` declarations but misses literal `include!` descendants of test-only modules.
Teach that same ownership graph to follow source-relative literal Rust includes.
Exempt an included file only when every known incoming edge is test-gated or owned
by an already proven test-only file. Keep production and mixed-owner includes
checked. Do not exempt files by name or directory, expand arbitrary macros, add
parser/lowerer allowlist entries, or change the production compiler.

Acceptance: preserve all existing alias/bootstrap/parser controls; add real temporary
file controls for nested includes, inline and cross-file test ownership, mixed
production/test owners, duplicate references, comments/string decoys, unsupported
include expressions, unreferenced files and cycles. Unknown include expressions must
not create a test-only edge; preserve conservative production checking. The old
implementation must fail the positive include regression controls while negative
controls continue to detect production calls. Run the complete repository guard,
its self-tests, affected manifest/inventory contracts and file-size checks with the
pinned Python. Require exact-candidate independent review. This removes a known
false blocker; it does not claim a runtime improvement or waive final phase gates.

### V4d implementation and focused validation

Literal includes now participate in the existing test-ownership graph, using their
source file's directory. Normal/raw literals and macro delimiters are recognized;
comments, strings and unsupported expressions do not establish ownership. Direct
`cfg(test)`, inline test modules and cross-file test owners are recognized. A file
with a production owner, an unresolved ownership cycle or no known owner remains
checked. No compiler code, approved parser site or profile membership changes.

The repository guard passes (21.656s); all existing and added self-tests pass
(0.643s). An independent baseline/candidate comparison (36.157s) reproduces exactly
the three previously recorded `method_authority_tests` findings on the baseline and
zero on the candidate across the same Rust source inventory. The baseline fails the
new positive include regression while passing all new negative controls and every
existing self-test. An AST comparison proves all prior self-test assertions and
unaffected guard functions unchanged. Manifest registration still runs the guard
and its self-tests once each. File-size checks pass (2.655s).

The first new self-test failed because its synthetic production module used the
wrong Rust module-relative path; the corrected fixture uses an explicit path. That
failed execution (0.649s) and source remain preserved. A shell quoting failure while
preparing that correction did not mutate sources or execute tests. No false failure
is reclassified as a passing test. Exact-candidate review and merge remain pending
at this record. Complete final phase gates and V5 are still required.

### V4d closure — literal test include ownership / #4401

PR [#4401](https://github.com/sifr-lang/sifr/pull/4401) merged as
`98d53e68fd` after focused validation and independent **SATISFIED** review, with no
blockers, on candidate `fc9f8e90d47d1910ed7ac2d3da80b198198fb9aa` against
`0904d38ef15ec750d313338f23cf38c8d7e26804`. Evidence `v4d-evidence.tar.gz`, SHA-256
`b521203cbeccd8170d6fc86baac28e8a942123df13f81e390d39fb74b53577c0`, contains 34
independently verified files. Raw review lives outside Git at
`reviews/fc9f8e90d47d1910ed7ac2d3da80b198198fb9aa/response.md`, SHA-256
`25c3ba66f1e47ff858fd21fc416293a31c75f34ab741b372083e0a56c6f3fff2`.
The reviewer independently identifies nine newly classified test-only files, with
no current production owner and exactly the three expected findings removed.

Follow-ups remain separate: source ownership should eventually seed Cargo target
roots as production owners even if they also have gated incoming edges; the old
`#[path] mod` route already had that limitation, and no current source is affected.
Compound cfg expressions and inner cfg attributes remain conservatively checked.
No full-phase timing or final acceptance is claimed.

### V4e bounded investigation — refreshed integrated failure inventory

Run the actual current correctness-only profile with `--no-fail-fast`, after a
fresh resource/ownership/tool/plan preflight. The purpose is to discover independent
failures across the integrated changes before choosing the next repair; this is
not the final acceptance candidate. Preserve canonical selection, prerequisites,
commands and assertions. Use the existing owned compiler target and bounded worker
allocation. Do not count a missing, failed, capped or prerequisite-blocked category
as passing, and do not replace a failed assertion with a successful filter.

Apply the existing 30-minute cap to the diagnostic invocation and retain canonical
step timings/output and process cleanup. If the aggregate cap stops the profile,
record which categories ran and which remain unexecuted; subsequent diagnostics
may execute remaining canonical categories separately under their own caps. Do not
silently continue consumers whose actual preparation failed. Preserve the exact
candidate, resource allocation, identities and observed test counts. Assign each
failure to its existing phase owner or a newly bounded repair before modifying it.
Complete final create-PR/merge gates and V5 remain mandatory.


### V4e initial diagnostic result — incomplete, not acceptance

The first integrated diagnostic stopped after 759.556s at the existing disk reserve;
crate-test preparation was still compiling the distinct optimized/debug test graph.
The five initial source guards passed, but main assertion categories did not run.
Generated preparation also failed because this session had not published the exact
candidate revision before its first Cargo fetch. That setup failure is retained;
it is not a compiler defect. No blocked or unexecuted category counts as passing.
Process cleanup completed without a tracked source change. Twenty-four obsolete,
inactive, session-owned executables were copied and hash-verified before removing
the original copies, recovering 14.614 GiB; their recoverable bytes and manifests
remain outside Git. The interrupted ten-crate configuration is being resumed on
the unchanged candidate with explicit Cargo failure collection and a four-job
diagnostic allocation, not as final profile timing evidence.

Clarification to V4c: merge GCQ assertion selection is twelve representatives, but
current online generated preparation materializes and fetches all 93 entries.
Consequently the 93-entry cost remains part of the actual profile prelude. No
preparation selection is waived or narrowed by the earlier diagnostic wording.

### V4f contract — collect independent crate failures

Repair `--no-fail-fast` inside the canonical crate-test configuration loop. Currently
the first failed configuration aborts the loop even in explicit collection mode;
ordinary grouped Cargo commands also omit Cargo's failure-collection flag. Preserve
all selected packages, targets, features, filters, order, environments and assertions.
In collection mode, continue after ordinary command/assertion failures, print each
configuration's timing/status, then fail the aggregate using the first ordinary
failure. Default fail-fast behavior remains unchanged. Propagate cancellation,
negative signal status and non-exit termination causes immediately, even after an
earlier ordinary failure. Apply the same terminal-stop rule to the existing native
per-case scheduler so the outer loop cannot conceal a safety stop.

Acceptance: controls for default stopping, ordinary multi-configuration collection,
Cargo-versus-harness flag placement, preserved selection/environment/native dispatch,
nonzero aggregate result, and terminal failure precedence at both loop levels.
The old implementation must fail new collection/terminal regression controls. Run
affected configuration-plan, native scheduling, preparation, execution-scope and
profile contracts, compare emitted canonical plans, and check file-size guardrails.
Require exact-candidate independent review. This improves diagnostic completeness;
it does not bypass failed preparation, reduce coverage or claim a gate speedup.
Final integrated create-PR/merge gates and V5 remain required.


### V4f implementation and focused evidence

Explicit collection now passes Cargo's `--no-fail-fast` before harness arguments
and executes all independent crate configurations, retaining the first ordinary
failure as the aggregate result. Negative signal returns and non-exit termination
causes immediately propagate from both configuration and native-case loops, even
after earlier ordinary failures. Default stopping, native inventory/count checks,
all command selections and environments remain intact.

Eight new regression tests pass (3.376s). The unchanged implementation fails six
of these tests and errors in one (1.648s); that negative result is retained. The
first edit helper had a Python quoting error before any source mutation, so a
subsequent attempted focused run still tested the baseline and failed (0.657s).
The corrected helper and successful run remain separately identified.

Related tests execute 39 cases (2.725s): 38 pass and the existing frozen-scope
assertion fails because it expects 17 configurations while the plan contains 18.
An independent unchanged-baseline run reproduces that exact mismatch. This belongs
to V4 scope-contract reconciliation, not this scheduler repair; preserve and fix it
separately after tracing the added selection, without reducing test membership.
Configuration/native/collection, preparation and qualification policy checks pass;
both emitted correctness plans and all eight profile/mode compiler plans compare
identically against the unchanged baseline (15.266s total). Canonical `profiles
check` passes (2.190s), as do file-size guardrails (8.173s). No full integrated pass
or ten-minute timing acceptance is claimed. Exact-candidate review is next.


### V4f closure — complete crate failure collection / #4402

PR [#4402](https://github.com/sifr-lang/sifr/pull/4402) merged as `1ef6aa18ea`
after independent **SATISFIED** review with no blockers on candidate
`cc63437e74e2e093fe64c5b83bbf80b50c4ce07d`, base `98d53e68fd5e192eccfae3530335c8a4ff6c83a0`.
The 43-file evidence archive `v4f-evidence.tar.gz` was independently verified,
SHA-256 `8a57f3b6033d1a6b1fd2ce8564ca9a410a8ada6f3e49a21e4b01da70d4677dab`.
Raw review remains outside Git at `reviews/cc63437e74e2e093fe64c5b83bbf80b50c4ce07d/response.md`,
SHA-256 `b2ca60ddceeff11858a9440b45dd62741d6cc73131c60fa744941beb2b5c03fa`.
Reviewer suggestions: use the process layer's literal `safety_deadline` cause in
future test maintenance (the current rule correctly rejects every non-exit cause),
and document that terminal failure takes aggregate precedence while prior ordinary
failures remain in individual logs/status lines. The latter is intentional here.
The frozen-scope count mismatch remains failed and is assigned to V4g below.

### V4g contract — preserve frozen coverage across new crate enrollment

The frozen-scope self-test compares only the number of Cargo configurations against
17. PR #4392 deliberately enrolled the separate semantic-services owner, producing
18 configurations. PRs #4384 and #4390 additionally enrolled stdlib-artifact and
metadata inside the existing additive compiler group. Preserve those enrollments
and the original frozen evidence; neither reduce coverage nor rewrite history.

Replace the stale crate-count assertion with a comparison against every frozen
configuration's actual IDs, execution/preparation commands and isolation class.
Project only newly enrolled package pairs out of an expanded additive group when
comparing its old required membership; retain every original flag, filter and
relative order. Additional isolated configurations are allowed. Missing, duplicate,
regrouped or weakened original selections must fail. Do not change production
profile selection, runner execution or the remaining scope assertions.

Acceptance: unchanged current selection passes; baseline count check reproduces
its known failure. Negative controls must reject loss/reordering/duplication of
original ownership, altered features or native filters, changed preparation, and
broken isolation, even when the number of configurations stays the same. Retain
all original frozen configuration bytes and compare emitted plans before/after.
Run execution-scope, configuration-plan/native/collection, preparation and profile
contracts and file-size checks; require independent exact-candidate review. Final
phase gates and V5 remain open.


### V4g implementation and validation

The frozen-scope check now compares every original configuration's IDs, command
flags/filter order, preparation and isolation. Only plain new `test -p <package>`
entries in an additive group are projected away for the historical comparison;
new isolated groups remain selected. The original frozen evidence is byte-identical.
The checker uses explicit exceptions and remains effective under Python `-O`.

All 54 related execution-scope/configuration/native/collection/preparation tests
pass (2.945s), including seven new positive/negative coverage controls. All 16
scope and coverage tests also pass with optimized Python (1.667s). Canonical scope,
configuration, preparation and qualification policies and the independent emitted
plan comparison pass (2.886s); both correctness plans and eight compiler profile/mode
plans are unchanged. The unchanged baseline still fails its old `18 != 17` count
assertion. Profile validation (0.641s) and file-size guardrails (6.650s) pass.
The initial scope-only run passed before replacing Python assertions with explicit
checks (2.202s); final evidence above covers the hardened implementation. Review
and merge remain next; no complete gate or ten-minute acceptance is claimed.


Before review, the same coverage comparison was strengthened to retain original
configuration order as well as ownership/order within each configuration. The
negative control swaps two original configurations and must fail. All 54 related
checks pass again (2.399s), all 16 optimized-Python scope checks pass (0.638s), and
file-size checks pass (5.660s). This affects only the coverage self-check; emitted
selection and frozen evidence remain unchanged. The earlier candidate and its
archive remain diagnostic history, not review of this final candidate.


### V4g closure — frozen coverage preservation / #4403

PR [#4403](https://github.com/sifr-lang/sifr/pull/4403) merged as `5f62d3fd26`
after independent **SATISFIED** review with no blockers on candidate
`95f2499e73e2f1397ef9f6f41eaa9cc946ba2020`, base `1ef6aa18eada6c80636021c03cad5482c28b1b95`.
The 56-file archive `v4g-final-evidence.tar.gz` was independently verified,
SHA-256 `5c55139bc6aa5d0de9940a76bee0368ba8822e143a8b7e291cbb1803e270250d`.
Raw review is outside Git at `reviews/95f2499e73e2f1397ef9f6f41eaa9cc946ba2020/response.md`,
SHA-256 `ffa673d314b0f0c225e5139a2b82ac7631d232c751a3d15cd7fe6d23f7a85073`.
Follow-ups: if future grouping keys permit additional integration-only groups,
explicitly qualify their isolation; the current planner cannot produce that case.
Future additions to a filtered integration group currently fail closed and need an
explicit contract extension. The mutation tests use the full merge profile while
the scope entry uses correctness selection; an existing equality test preserves
that equivalence. None reduces present coverage or blocks this bounded repair.

### V1e bounded investigation — JIT compiler build/runtime tradeoff

The refreshed SQL build timings place roughly 161 seconds of LLVM work in the
Cranelift codegen unit, overlapping other units. Prior blanket optimization-zero
measurements hurt metadata validation, and the build-only helper experiment did
not materially reduce the whole protocol. Neither establishes the effect of this
specific runtime dependency. Test one diagnostic override: `cranelift-codegen` at
opt0, preserving every other resolved profile field, package, feature and target.
Keep all first-party compiler/metadata code at its existing optimization levels.
This is a measured feasibility experiment, not approval of a production override.

Compare complete canonical clean-A, unchanged-A and independent-clean-B SQL
qualification for a fresh baseline and candidate on identical source, with the
same six-worker allocation, locked/offline resolution, path maps and debug/LTO
settings. Preserve all seven artifact rejection controls, exact three linked
executable selection/equality within each protocol, zero-fresh clean builds,
complete-fresh reuse and owned-target cleanup. Observe actual Cargo profiles;
reject accidental additional optimization/feature changes. Keep the existing
30-minute cap per diagnostic category, resource preflight and reserves.

Measure the runtime cost as well: compile the existing, unchanged PostgreSQL
component integration-test source and support against the exact native libraries
from each clean SQL graph, with identical probe compiler flags. Preserve every
assertion and every supported checked-in PostgreSQL component. Run repeated fresh
processes, including its valid queries, syntax/schema diagnostics, source spans,
sequence and view checks. Record probe preparation separately from canonical
build stages and total diagnostic wall time. It may not change Cargo target
inputs or freshness. Preserve source/guest/library hashes, flags, logs, outcomes
and any runtime regression. Do not infer total-gate savings by subtracting one
unit's duration or accept faster compilation at an unqualified runtime cost.

If promising, register exact production changes and full semantic/runtime/profile
identity acceptance before implementing them. This diagnostic does not replace
normal compiler, E2E, native/Python or final phase gates; V1 feasibility and the
sub-600-second V5 matrix remain mandatory.


### V4e diagnostic continuation — all create-PR crate configurations complete

On unchanged diagnostic candidate `bd154f70ba8a3f813463bb6bb37bade0ec6c2613`, all
thirteen canonical create-PR smoke crate configurations subsequently completed
successfully, each with its own 30-minute cap. No completed category changed source,
left descendants or hit a resource/cancellation limit. These were separate runs
with retained preparation; their sum is not one integrated profile measurement.
The ten-crate continuation used four build jobs; the others used two. All used
one libtest thread and retained the canonical package/feature/filter selections.

| Configuration | Outer seconds, including its remaining preparation |
| --- | ---: |
| Compiler component | 5.686 |
| SQL contract | 4.688 |
| MySQL | 12.193 |
| SQLite | 6.688 |
| PostgreSQL | 42.706 |
| Ten-crate diagnostics/frontend/LSP/package group | 342.061 |
| Lowering | 212.808 |
| Stdlib default | 0.682 |
| Stdlib feature API | 3.686 |
| Runtime default | 8.687 |
| Runtime HTTP | 18.195 |
| CLI binary default selection | 155.964 |
| Driver library default selection | 818.916 |
| **Sum of these separate completed diagnostic categories** | **1632.958** |

The ten-crate group's test preparation reports 93 seconds; its 114 LSP unit tests
report 213.18 seconds. Driver preparation accounts for roughly 54 seconds and its
harness reports **752 passed, 82 ignored, 0 failed**, with **764.33 seconds** in
execution. Those 82 ignored native cases belong to the separate full-profile
native selection and have not been executed by this default driver invocation.
The retained-history-over-4096 stress assertion passes. This complete driver
observation, even without preparation, is above the full-gate target and requires
further attribution; successful focused history timings are not the whole suite.

Before the final six configurations, the allocation preflight paused at 27.57 GiB
free. Four additional obsolete, inactive, session-owned service/analysis executables
were copied and independently hash-verified before removing original copies,
recovering 4.016 GiB and restoring 31.586 GiB free. All archived bytes remain
recoverable outside Git. The original not-started capacity record is preserved,
followed by the later successful receipts; no failed attempt was reclassified.

The 115-file archive `v4e-crate-evidence.tar.gz` was independently verified,
SHA-256 `25492b224646619cb80abfcb55f0d6b146f519c9ba8be1a34d60e6b42943281c`.
It retains the earlier incomplete integrated attempt, source/command/resource
bindings, category logs, timings and recovery proofs. Archived executable payloads
remain separately retained. Generated preparation's earlier unpublished-revision
failure still requires a successful retry; merge-only crate configurations,
ignored native cases, area/toolchain verifications and final gates remain open.
Stable libtest rejects both JSON output and report-time flags; the read-only
capability checks are retained separately. Do not enable unstable toolchain
workarounds or infer exact case timings from unlabelled nested output.

The follow-up exact-binary `--ignored --list` discovery finds **6 CLI and 82 driver
cases, 88 total**. The original 87-role count is historical evidence, not a ceiling
on current execution. Both binaries are the ones used by the completed V4e default
crate runs, and the native runner already enumerates its selection dynamically.
`v4e-current-native-discovery.json` retains their paths, file identities, complete
names, commands and successful discovery outputs. These ignored assertions still
require execution; listing them is not coverage acceptance.

### V1e outcome — reject the JIT optimization override

Both complete canonical SQL build protocols pass on frozen diagnostic source
`bd154f70ba8a3f813463bb6bb37bade0ec6c2613`, with the source-equivalence proof to the
prospective phase record retained. Each protocol preserves all seven rejection
controls, zero-fresh clean builds, all-fresh unchanged builds, equal bytes for all
three linked tools across its three builds, and target cleanup. All 490 resolved
units have identical package/target/features. The sole effective profile change
is native `cranelift-codegen 0.135.1` opt1 to opt0; debug assertions, overflow
checks and all other emitted profile fields remain unchanged.

| Observation | Baseline | Cranelift opt0 |
| --- | ---: | ---: |
| Clean A | 388.125s | 359.546s |
| Unchanged A | 6.535s | 6.598s |
| Independent clean B | 398.059s | 366.900s |
| Protocol including cleanup, excluding external probe preparation | 793.732s | 734.057s |
| Unchanged PostgreSQL component assertions, fresh process | 38.314 / 38.399 / 38.478s | 330.599s, one complete run |

The completed candidate runtime executes both original tests, all supported
PostgreSQL components and their query/schema/diagnostic/span/sequence/view
assertions. It passes, but takes **8.6 times the baseline median**. Under the user's
pragmatic-testing authority, reject the profile and stop its second repetition
rather than spending another eleven minutes confirming an already unacceptable
tradeoff. The third repetition was not started. The original candidate wrapper
therefore remains **interrupted**, return `-15`, with the explicit owned-process
stop record, partial output, no remaining descendants and no source changes.
Repeated candidate runtime qualification is incomplete; this is a rejection
decision, never an accepted performance result.

The original baseline wrapper also remains failed: its external probe initially
omitted the host dependency search directory. The independent build protocol
still completed successfully. Retaining 805 completed clean-B artifacts through
read-only hard links allowed the probe to be repaired without rebuilding. Every
retained file has a recorded hash; the repaired probe uses identical test sources
and compiler flags, passes in three fresh processes, and removes all extra
retention links. Baseline clean-B includes that retention observation overhead;
one build pair does not establish a precise statistical speedup.

Cranelift's clean-A compile unit falls from 202.43s to 46.72s, while the complete
clean build saves only 28.58s. Lowering still takes 158.83s in the candidate, with
146.63s in codegen, and the remaining PostgreSQL build/link chain finishes later.
These overlapping unit durations are not additive savings.

The independently verified 109-file archive `v1e-evidence.tar.gz` has SHA-256
`0fcb81e687679b0f4d20ad6ec1fdae81c3e46f5b26c35c5913e8010cf5e77a3a`.
It preserves the original failure, recovery, completed and interrupted runtime
runs, build timings, exact inventories, six guest hashes, profile comparison and
cleanup evidence. Diagnostic executables remain separately retained by hash.
No production profile is changed. V1 feasibility, all remaining phase work and
complete final gates/V5 remain open.

### V4h bounded investigation — complete driver attribution and scheduling

V4e's 764.33s driver body used an explicitly serial diagnostic environment.
Ordinary non-ignored profile selections do not currently set libtest concurrency,
so this observation is not an inherent floor for the existing gate. Establish the
individual costs and a bounded parallel comparison before reworking storage or
changing the runner's allocation policy.

Use the unchanged retained driver configuration and its complete discovered
834-case inventory: 752 active assertions and 82 ignored native cases, whose
separate execution remains required. Keep source, target, Cargo selection,
features, nested Cargo/Rayon allocation and E2E settings identical. Compare one
libtest thread with two, running only one suite at a time and retaining the
existing 30-minute cap, 20 GiB disk reserve and 2 GiB memory reserve. Include actual
Cargo admission/preparation and cleanup; do not infer a whole-gate speedup from
these separate observations.

For serial attribution, observe only the main harness thread's stdout writes
using the available stable tooling; preserve the unchanged test executable and
all assertions. Qualify the diagnostic parser first with real delayed tests,
ignored cases and child/thread output that deliberately imitates harness results.
Reject missing/truncated/duplicate inventory or a missing summary. Compare the
actual Cargo-runner discovery with the exact executable's inventory before
starting the long run. Attribute serial write-boundary latency only, including
dispatch/tracing overhead. Parallel completion output does not establish each
test's start time and must not be reported as individual parallel durations.

Keep raw traces, outcomes, binary/source identities, resource receipts and every
failure. Do not use unstable Rust test flags, replace the canonical assertion
inventory with parsed child text, or adopt a production concurrency change from
an unqualified diagnostic. This item is measurement only. Complete native
selection, other suites, final gates and V5 remain mandatory.

### V1e/V4e record closure — #4404

Documentation-only PR [#4404](https://github.com/sifr-lang/sifr/pull/4404) merged
candidate `b40a0bfca16dccb31f6cef5946ae241ffb2bc8c9` as
`0969826f528a11be99da393cce56d24eb3c29a58`. It records the rejected JIT profile,
complete crate diagnostics, current native inventory, V4g closure and prospective
V4h investigation. Documentation checks and independently verified evidence
support the record; it changes no implementation and claims no final gate pass.

### V4h outcome — serial attribution and a failed parallel comparison

The unchanged driver executable
`9d67373b896871be0b024670ba83dca5bf23a296f0c2189bf9a8518c851a6f10`
executes all 752 active tests successfully with one libtest thread; 82 ignored
cases remain separately required. The harness takes 803.540s; Cargo execution
including tracing takes 805.946s, and the outer diagnostic including discovery
takes 814.757s. Serial write-boundary attribution accounts for 803.475s:

| Family | Active cases | Observed serial seconds |
| --- | ---: | ---: |
| Project cache | 45 | 247.650 |
| General driver tests | 224 | 174.834 |
| Build | 363 | 153.948 |
| Metadata producer | 15 | 120.949 |
| Test runner | 6 | 39.593 |
| Standard library | 56 | 39.338 |
| Metadata reader | 12 | 25.776 |
| Process execution | 5 | 1.370 |
| Remaining four families | 26 | 0.017 |

The 4,097-publication history case takes 223.700s. This preserves its complete
publication, retention, ABA and durability assertions. Partial process I/O
observations do not identify a dominant function or prove that fsync dominates.
The separate kernel wait samples were not bounded to that case; their explicit
qualification excludes history-specific attribution.

The same executable, inventory and nested worker environment with two libtest
threads finishes **failed** in 300.610s: 751 pass, one fails, 82 are ignored.
`history_over_4096_keeps_publishing_and_recent_aba` receives `WouldBlock` from
publication at `history_tests.rs:23`. The outer diagnostic takes 306.896s.
This failure-shortened result is not a speedup or concurrency acceptance.
Both wrappers preserve source identity and leave no descendants. Parallel
completion timestamps are not individual case durations.

A separate pinned-Rust control reproduces a relevant lock-lifetime mechanism:
an unrelated child paused before exec retains a parent's open file description
despite close-on-exec. Closing the parent's locked file leaves a new contender
blocked until the child executes; explicitly unlocking before close lets the
contender acquire immediately. This establishes the mechanism, not the identity
of a particular child in the original uninstrumented failure. V4i owns the
deterministic production regression and repair.

The independently verified 85-file archive `v4h-evidence.tar.gz`, SHA-256
`3fc83033e1c9cf3b0e83788663d3a9feb9db579f2667553cf0b5ec8165e278e0`,
preserves both complete inventories, raw traces, timing/resource receipts,
failed and repaired parser controls, partial-observation qualifications and
the Rust lock control. No production scheduling change is accepted.

### V4i bounded repair — process-local project lease lifetime

Scope: give project-cache writer, stage, generation-reader and namespace/GC
locks an explicit RAII owner that unlocks on its final parent-side release.
Retain close-on-exec, nonblocking admission, stable lock inodes and the existing
writer-before-generation ordering. Share the namespace owner through `Arc`;
do not create independently unlocking owners by cloning its file descriptor.
Keep native child-owned leases and their intentional inheritance unchanged.
Do not add retry sleeps, serialize the entire test suite, remove fsync, reduce
the 4,097 publications, or change the stored format and validation authority.

Register this contract before implementation. First add a deterministic control
using the production process-local guard and a real child paused before exec,
including exclusive/shared ownership, final shared-owner release, live-owner
exclusion and independent-reader exclusion. A control with close-only release
must demonstrate the original failure. Then run the complete project-cache
tests, native child-lease controls and the complete default driver suite with
two threads under the existing 30-minute cap and capacity reserves. Preserve
all failures. The full driver pass is correctness evidence, not a whole-gate
speed claim or a new production scheduling policy. Run formatting, size and
affected ownership guardrails, then exact-candidate independent review.
Final native selection, integrated gates and V5 remain open.

### V4i closure — process-local lease release / #4405

PR [#4405](https://github.com/sifr-lang/sifr/pull/4405) merged candidate
`95984515b9ac0afa323b434377a2ac5f1b097af1` as
`d872d2329f5fb970a54ed563da3d7908b3bf7d64`. Project publication, staging,
generation readers and housekeeping now release their process-local lock
explicitly when the final owner drops. Namespace readers share that owner through
`Arc`; intentional native child leases retain their existing ownership.

All three new controls pass with the production guard. A synchronized real
fork/pre-exec control exercises exclusive and shared locks, shared-owner lifetime
and live exclusion. Replacing only explicit unlock with close-only release makes
that regression fail with `WouldBlock`; independent readers and failed admission
have separate passing controls.

The complete two-thread default driver selection passes: **755 active tests,
zero failures, 82 ignored, zero filtered**, preserving every prior case and adding
the three regressions. This includes all 48 project-cache tests, all three native
child-lease controls and the complete 4,097-publication history. The harness takes
347.420s, Cargo execution with tracing 349.927s, and the complete diagnostic
367.910s including retained preparation/discovery/cleanup. Formatting, file-size,
source dependency ownership and driver maintainability checks also pass.

These are correctness diagnostics, not V5 timing acceptance: preparation reused
the retained target after an earlier 100.761s intentionally interrupted build,
and lightweight guardrails overlapped the beginning of the run. The interrupted
attempt detected an obsolete import; its stop record and partial evidence remain.
No successful speedup is inferred from V4h's failed parallel run, and no production
libtest allocation changes here.

All 87 files in `v4i-evidence.tar.gz` are independently verified; archive SHA-256:
`fc74058524f2c535da45fdc2702978449600a6c8bf26a903585f9f6b975694b4`.
The exact-candidate independent review is **SATISFIED**, with no blockers; raw
review SHA-256:
`0178d97bd4f72215333993cc51c544dd9a8c2fe78e1e10946a37c1b8f92b3f1c`.

V4 follow-up owner: the editor reader in
`crates/sifr_compiler_services/src/editor/saved_checks.rs` still takes namespace
and generation locks as raw files. An unrelated fork can similarly delay their
release and subsequent exclusive pruning. That separately owned reader was not
changed in V4i; reconcile its shared lease lifetime explicitly rather than
assuming the driver repair covers it. The guard's documented close-on-drop
behavior after an unlock error remains unchanged. Final native selection,
complete phase gates and V5 remain open.

### V1f bounded investigation — lowering optimization tradeoff

The mandatory SQL clean/unchanged/independent-clean protocol still exceeds the
complete gate budget by itself. V1e showed that making the JIT compiler cheap to
build caused an unacceptable runtime regression. Keep that rejected option
closed. The remaining concrete candidate is the native `sifr_lowering` unit:
V1e's clean-A observation spends 170.120s there, including 158.360s of codegen
in the baseline. Its opt1 candidate in V1a did not solve the whole build path.

Measure one narrower diagnostic override:
`profile.dev.package.sifr_lowering.opt-level=0`. Cargo's actual artifact metadata
confirms that this package override changes both the native library and its host
build script from opt2 to opt0; record both units. Keep the JIT/compiler-component
dependencies, type-system optimization, debug assertions, overflow checks and
every other unit/profile field unchanged. Do not edit production profiles.
Use the unchanged canonical three-build SQL protocol, all seven rejection
controls, exact resolved units and features, and within-profile equality of all
three linked tools. Run one protocol at a time with six Cargo workers, the same
CPU/memory/disk reserves and the existing 30-minute category cap.

Compile runtime probes from those exact native artifacts before their canonical
target cleanup: the unchanged PostgreSQL component tests and unchanged
`sifr_sql_tool/tests/migration_source.rs` tests, including real canonical source
checking and full source-stdlib preparation. Preserve all source fixtures and
assertions; compile probes with identical flags in baseline and candidate.
Measure preparation separately, then execute in fresh processes. Complete at
least one runtime comparison promptly; reject and stop further repetitions if
it already demonstrates an unacceptable tradeoff. Preserve interruptions as
incomplete. If viable, complete three runtime observations and compare exact
outcomes; these limited probes still cannot authorize a production profile.

A matched baseline is needed for the newly added source-checking probe, rather
than extrapolating from V1e's JIT-only runtime result. Compare complete elapsed
protocol time, not additive overlapping unit durations. Verify compiler/SQL/
fixture/toolchain equivalence between the frozen source and this prospective
record. Any source, inventory or harness error invalidates its affected result.
Adoption would require a new implementation contract and broader semantic/native/
installed qualification; full phase timing and final gates remain mandatory.

### V1f outcome — lowering alone does not solve the build budget

Both complete diagnostic protocols pass on frozen implementation
`95984515b9ac0afa323b434377a2ac5f1b097af1`; the prospective phase records differ
only in this Markdown file. No production profile or source was changed.

| Observation | Normal development profile | Lowering opt0 diagnostic |
| --- | ---: | ---: |
| Clean A | 387.104s | 364.125s |
| Unchanged A | 6.589s | 6.563s |
| Independent clean B | 394.767s | 363.463s |
| Complete build protocol, excluding external probe preparation | 789.540s | 735.175s |
| External probe preparation | 3.230s | 3.385s |
| Source migration probe, three-run median / worst | 0.557s / 0.591s | 1.194s / 1.215s |
| PostgreSQL component probe, three-run median / worst | 38.377s / 38.450s | 38.439s / 38.631s |
| Whole diagnostic, including preparation and six probe processes | 910.919s | 858.405s |

All 490 package/target/feature entries match in every stage. Each clean stage
reports zero fresh entries; unchanged A reports all 490 fresh. Only the native
lowering library and its host build script change optimization level, from two
to zero. Other profile fields, runtime environment, fixture hashes and normalized
test outcomes match. All seven rejection controls pass; the three linked tool
hashes match across each protocol's builds; all owned targets are removed.
The four original tests execute in each of three fresh-process repetitions per
profile. The source probe performs canonical source-stdlib checking but uses its
existing test dialect; it is not complete SQL/native/installed qualification.

The single matched build observation saves 54.365s (6.9%), while source checking
is 2.142 times slower and component runtime is effectively unchanged. Do not adopt
the lowering-only override: it still misses the complete gate budget by itself.
This is not a statistical estimate or a claim that the entire gate becomes faster.

Cargo's clean-A trace illustrates the overlap: lowering falls from 170.280s to
25.030s, but frontend remains 84.650s and Cranelift remains 201.020s. These unit
durations cannot be summed as savings. The PostgreSQL build-script compile unit
is reported unblocked at 23.360s but starts at 282.130s in the candidate. Thus its
late C compilation is not, by itself, proof of a component-host dependency block;
resource scheduling contributes. No scheduler change or crate split is qualified
by this observation.

`v1f-comparison.json` binds the actual inventories, profile differences, source
hashes, tool identities and complete runtime outcomes. The portable packet
`v1f-evidence.tar.gz` has SHA-256
`574972f663143aa53c7cbcd6d3feb5a6817c11d875a64a4a36341bb8305898ca`;
all 126 manifest files were independently hash/size verified. Both wrappers
report no reserve stop, remaining descendants or tracked source mutation.
All final phase gates and V5 remain open.

### V1g bounded investigation — combined semantic compilation costs

V1f rules out treating one crate's codegen reduction as the full build saving.
Test one coherent extension on the same frozen source: opt0 for
`sifr_lowering`, `sifr_frontend`, `sifr_semantic_services` and
`sifr_sql_contract`, leaving the global development profile and all other
package overrides unchanged. In particular retain the optimized type system,
Cranelift and hashing dependencies. Do not reopen V1e's rejected Cranelift opt0
or change guest JIT policy. This remains an external diagnostic, not a production
profile edit.

Use the same canonical clean A / unchanged A / independent clean B protocol,
seven rejection controls, six Cargo workers, two Rayon workers during builds,
memory/disk reserves, 30-minute category cap, linked-tool equality and cleanup.
Compare all actual units and features. The expected profile differences are the
four native libraries plus lowering's host build script; the other three build
scripts already use opt0. Reject any other unit/profile drift.

Reuse V1f's recorded baseline only after verifying exact source/toolchain,
protocol, probe, guest, environment and allocation identities; do not rerun an
unchanged baseline merely to produce another record. Compile and run the same
unchanged migration-source and six-major PostgreSQL probes from the candidate's
actual clean-A libraries. Preserve three fresh-process observations and exact
outcomes, or explicitly stop and classify incomplete if a first complete
comparison demonstrates an unacceptable tradeoff. Include preparation in the
whole diagnostic and report its separate build-protocol subtotal.

The purpose is to find whether these overlapping compiler units collectively
constrain feasibility, not to approve a speculative profile. If this candidate
still cannot fit the build budget or its runtime penalty defeats the saving,
record that result and stop this profile experiment. Adoption requires a separate
implementation contract and complete affected compiler, semantic/native, SQL,
source/installed and final phase qualification. No cached test success or omitted
reproducibility build may satisfy the goal.

### V1g outcome — combined profile still exceeds build feasibility

The V4i closure, V1f result and prospective V1g contract were merged in
[#4406](https://github.com/sifr-lang/sifr/pull/4406), candidate
`84d6a36852dda8ef00d0a07dbd4d68e7726d32ee`, merge
`2b7d752fb8e5f1cb48d26a6aebf078adcdc2402b`. The frozen implementation remains
`95984515b9ac0afa323b434377a2ac5f1b097af1`; differences are phase Markdown only.

The combined diagnostic passes the complete SQL build protocol in **675.300s**
excluding 3.341s of external probe preparation. Clean A / unchanged A / clean B
take **328.654 / 6.684 / 338.956s**. Compared with V1f's unchanged 789.540s
baseline, the observed reduction is 114.240s (14.5%), still above the complete
gate's 600-second budget before other required tests.

The baseline was reused under the prospective input-bound diagnostic contract,
not executed again: all 126 archived evidence files, actual source/toolchain,
protocol/probe/guest hashes, runtime environment and allocation match. All 490
resolved package/target/feature entries match in all three stages. The only
profile differences are the four declared native libraries and lowering's host
build script. All debug assertions, overflow checks and other profile fields
remain unchanged. Both clean stages have zero fresh entries; unchanged A has all
490 fresh. Seven rejection controls and three linked-tool byte comparisons pass,
and all owned build targets are removed.

All four original runtime tests pass in each of three fresh-process repetitions,
with matching normalized outcomes. Source checking has median/worst
**1.189 / 1.206s**, versus **0.557 / 0.591s** normally: 2.133 times the baseline
median. PostgreSQL component median/worst is **37.914 / 38.244s**, versus
**38.377 / 38.450s** normally. Do not claim a component-runtime speedup from these
three observations. The whole candidate diagnostic, including preparation,
cleanup and all six runtime processes, takes **797.319s**.

The first attempt failed preflight in 0.685s because the diagnostic compared
raw `rustc -vV` output with the canonical record's trimmed output. The complete
version, compiler commit and LLVM version matched; only a trailing newline
differed. Preserve that failed receipt and original helper. The corrected helper
normalizes that newline and starts a separate recorded attempt. Both wrappers
report no resource-reserve stop, remaining descendants or tracked mutation.

`v1g-comparison.json` reconciles all inventories, profile differences and outcomes.
`v1g-evidence.tar.gz`, SHA-256
`de746b956cf80e4c721666eb6ea3ff7af4e1ce1341eb4b137c6a8052a19fcf08`,
contains 72 independently hash/size-verified files, including the immutable V1f
baseline archive. No production profile is changed. Stop this profile experiment;
it does not establish build feasibility or qualify broader compiler behavior.
V1, the remaining execution/fixture work, complete final gates and V5 stay open.

### V4j bounded repair — shared editor/project process-local leases

Address the explicit V4i review follow-up in
`sifr_compiler_services::editor::saved_checks`: namespace and generation readers
still hold raw `File` locks, so an unrelated child paused between fork and exec
can retain their locks after the Rust reader is dropped. This is the same
demonstrated process-local ownership gap, not permission to change intentionally
inherited native-child leases or increase test concurrency.

Move the proven `ProcessLease` implementation to the existing shared
`sifr_cache_storage` owner. Document unique file ownership, sharing through
`Arc<ProcessLease>` when needed, and the prohibition on using it for child-owned
locks. Driver storage/housekeeping and editor saved checks must consume this one
implementation. Preserve nonblocking admission, stable inodes, complete record
validation, durability, GC selection and all native-child lease behavior.
Both consumers already depend on the shared storage crate; no dependency pins or
profile membership need to change.

Keep the existing driver lease regressions and their exact test IDs through a
test-only harness importing the shared production type. Add an editor regression
using a real complete saved generation and stable namespace/generation lock
files. Pause a real child before exec with a bounded readiness handshake; drop
the actual editor reader and require both exclusive locks to be available before
the child continues. The original raw-file implementation must fail this control.
The test must always release/reap its owned child, including on assertion failure.
Also exercise failed owner/generation admission and retain existing malformed
saved-check behavior. Do not substitute sleeps for readiness.

Validate the failing original editor path, the corrected editor/service and
cache-storage suites, the shared guard's close-only negative mutant, all driver
project-cache and intentional native-child controls. Use the complete default
driver suite if it is more efficient than repeated preparation of those families.
Run formatting, size and affected source/ownership guards, and confirm unchanged
profile selections. Update the architecture's project-storage ownership record.
Exact-candidate independent review remains required before this intermediate
implementation merge; complete final phase gates and V5 are not waived.


### V4j closure — shared process-local lease owner / #4408

PR [#4408](https://github.com/sifr-lang/sifr/pull/4408) merged as
`dda9fb2ba53330e59ee22f971daa5e794229f8e1` after **SATISFIED** independent review
of candidate `67da62e36af1aa874de2bae079ec8d6ac15e0d63`, base
`6c1c599f2ef77ca872bd2f4b591d712efbe27b0f`, with no blockers. One shared
`ProcessLease` now protects both driver project storage and editor saved-check
readers. Intentionally inherited native-child ownership is unchanged.

The original private-fixture editor reader failed the real paused-child control
with namespace `WouldBlock`; the fixed editor controls and all 24 compiler-service
tests passed. Shared guard controls passed, while the close-only negative mutant
failed as required. The full default driver suite passed 755 tests with 82
explicitly ignored, preserving its exact 837-test inventory and all 48
project-cache and intentional native-child controls. Preparation was 81.294s,
execution 349.342s, complete wrapper 434.952s. Formatting, file size, source
ownership, driver maintainability and unchanged emitted plans passed. This is
intermediate correctness evidence, not full-gate or V5 timing acceptance.

Cache-storage validation remains **failed**, not waived or reclassified: both
complete runs had nine passes and one failure in the unchanged
`idle_and_live_lease_waits_remain_bounded` control. One isolated single-test pass
did not establish a fix. V4k below owns this explicit follow-up. The initial
permissive editor fixture and missing test-harness `io` import attempts also
remain preserved as failed diagnostics, separate from the original production
regression. CI run `39634057520`, job `119339672832`, failed before correctness
setup because the full-profile performance reference was unset; no CI-green
claim or workflow repair is included. Cross-platform fixture birth-time support
remains outside this current-host speed target.

Manifest-verified 158-file archive `v4j-evidence.tar.gz`, SHA-256
`62b23732e92cc5db5c8e9235e57dcec20d9e3b5ae52e83f6c0027297e38b86fb`.
Raw review is outside Git at
`reviews/67da62e36af1aa874de2bae079ec8d6ac15e0d63/request.2OcUOk/response.md`,
SHA-256 `61506c855185f54a7be3e8d9fcbc1898768bc8dda1d89872324d28918abc60ea`.
The review correctly reported the heartbeat cause as uninstrumented at its input
snapshot; the later V4k syscall observation below supplies new evidence without
rewriting that review. Final integrated gates and the whole phase remain open.

### V4k bounded repair — deterministic lease deadlines and real renewal evidence

Own the unchanged storage-control failure discovered during V4j. Two complete
unmodified suites failed `idle_and_live_lease_waits_remain_bounded`; the individual
test passed once. A subsequent syscall-observed complete suite also failed. The
worker's three `fdatasync` calls took 30.454 / 30.598 / 31.133ms, in addition to
its configured 10ms interval. Writes were 41.666 and 41.323ms apart, exceeding
the test's 40ms idle allowance. The waiter returned before the next renewal,
while the owner remained locked. This demonstrates a test assumption failure;
it does not justify weakening production idle/hard deadlines or durability.
Original observation archive `v4k-heartbeat-observation.tar.gz`, SHA-256
`45f98ebb5f59e623a60f63543dd6eeb0e9296055f64ef9f28ef00c4ce754d5a2`.

Separate the two responsibilities currently coupled by tiny wall-clock budgets.
Extract only the private elapsed-deadline state/calculation used by the real
waiter, accepting observed monotonic time and file modification identity. Test
that actual production calculation with deterministic timestamps: idle boundary,
renewal resets only idle, hard deadline remains absolute despite renewal,
hard-before-idle precedence, unchanged/unavailable modification observations,
and bounded poll delays. Keep the original test ID, and retain real held-lock
coverage: timeout/cancellation must preserve the stable inode and exclusion;
a waiter must acquire after owner release. Add separate real `LeaseActivity`
evidence for multiple completed owner-note renewals and worker shutdown. Observe
actual progress with a generous failure bound; do not assume a disk flush or
thread scheduling latency below 40ms. Do not replace the renewal worker with a
mock in the I/O test.

Preserve exported APIs, default 30s idle / 41-minute hard / 5s renewal policy,
nonblocking admission, cancellation ordering, lock inode ownership, fsync,
process-local and intentionally inherited child lease behavior. No lock breaking,
timeout extension, ignored test, fallback or retry-until-green behavior. This
item is an independent verification reliability repair, not a V4j regression or
an asserted whole-gate speedup.

Named acceptance: focused deterministic deadline controls; full cache-storage
suite on both owned temporary storage locations; disabled-renewal and disabled
idle-reset negative controls; shared guard and real editor regressions; affected
driver live-holder, wedged-owner and metadata wait controls; formatting, size,
affected source ownership and unchanged emitted correctness selections.
Use the approved intermediate review policy with exact candidate/evidence.
Final integrated gates and all V5 timing scenarios remain required.


### V4k closure — deterministic deadline and real renewal controls / #4410

PR [#4410](https://github.com/sifr-lang/sifr/pull/4410) merged as
`7a4fd1dce3ce2556126d7d22689ce095673df776` after **SATISFIED** review with no
blockers on candidate `1ba41b0238cd7a1161053a6a38bd0fe58b015106`, base
`9020ec5f6aee1f4ae444e5219fc6fbbf695f1606`. The private deadline calculation is
now tested with controlled timestamps; production still observes real monotonic
time and modification identities. Public APIs, production deadline/renewal
constants, worker implementation, durability and child ownership are unchanged.

Both complete storage runs passed all 14 tests, retaining the ten original IDs
and adding three deadline controls and one real-renewal control. Wrapper times
were 2.187s and 0.688s on the two owned temporary-storage locations. The actual
production deadline code passes while its no-idle-reset mutant fails; the actual
renewal test passes while its disabled-worker mutant fails at the 10s failure
bound. All 24 compiler-service tests passed (50.217s including preparation).
All five driver live/wedged/retention controls, three process-local lease controls
and three intentional child-lease controls passed (29.721s combined including
preparation). Formatting, size, source ownership and unchanged emitted plans
passed. Some light checks overlapped; these are correctness diagnostics, not
whole-gate timing evidence. Original failures remain failed in their archives.

Manifest-verified 136-file archive `v4k-evidence.tar.gz`, SHA-256
`a038bc0eff884eb8501c595b3d8f1c136869e8a3326ceb195c2b3ed8021baa10`.
Raw review outside Git:
`reviews/1ba41b0238cd7a1161053a6a38bd0fe58b015106/request.fsdpu9/response.md`,
SHA-256 `dbb512dc38a7ffac7e0e0b6b3b1ef14784da2ce366571a343de1bc02e0c26de5`.
Nonblocking suggestions: explicit saturating-duration spelling and a direct
error-message projection assertion; the real driver live-holder integration
remains required in V5. No full-gate, CI-green or sub-ten-minute result is claimed.

### V3d bounded investigation — reusable native fixture lifecycle

Measure the two actual callers of `ReusableScenario`: explicit advanced-data
native preparation and its runtime roundtrip assertion. The helper currently
deletes and recopies the fixture on acquisition and deletes it again on drop;
its callers then rewrite the runtime dependency and evidence entrypoint. A stable
path alone therefore does not prove reusable Cargo inputs. This evidence applies
to those two roles, not all 88 current native roles or the 29 Python examples.

Run each exact selected role in a fresh process against the published frozen
candidate, with retained owned targets, explicit native worker/resource budgets
and a separate 30-minute diagnostic cap. Preserve stdout/stderr, exits and exact
selection. Observe actual nested Cargo commands and fingerprint dirty/fresh
reasons, distinguishing diagnostic instrumentation from acceptance timing. Record
preparation, assertions and any failures independently. Check current fixture,
overlay, lock, toolchain and Cargo-context identities. Do not clean shared targets
or assume an unchanged-path cache hit. A failed or capped preparation does not
turn its dependent assertion into a pass; run independently executable roles
when safe, and preserve the preparation failure.

Use the result to register the implementation boundary before editing production.
The intended direction is final-content reconciliation under a stable owned
lease: preserve unchanged bytes/mtimes, restore mutations, remove stale entries,
reject symlink substitution and bound retained inactive inputs. Compose the final
manifest/entrypoint overlay before reconciliation, avoiding restore-then-rewrite
invalidation. Existing fixture locks broadly inherit; replacement must bind the
lease only to actual metadata, probe, build and runtime children and retain those
consumers after parent death. A process-local unlock-on-drop owner is not suitable.
Trace all consumers through canonical build planning before selecting an API;
current public package-build calls take no input lease. No global descriptor
inheritance, missing identity input, copied successful assertion or unbounded
retention may be introduced. Broader native/Python/command-matrix and V5 coverage
remains required after the bounded item, and complete final gates remain open.


### V3d diagnostic closure — cold compilation and avoidable fixture churn

Frozen candidate `183e5b603ab985d6b6c4b67a7e44d20d0fe70b60` ran both exact
advanced-data roles separately with two native workers and retained owned targets.
Preparation reached the 30-minute cap (1801.459509s outer, terminated, incomplete).
The session guard verified all owned descendant groups stopped. Its subsequent
runtime role passed the complete original roundtrip/output/stderr assertions:
111.272127s outer, 3.093267s inventory/preparation, 104.358159s execution including
Cargo/tracing, 101.81s libtest body. The preparation failure remains a failure;
this is a fresh-process runtime observation using its retained build artifacts.

All 13 fixture files (163640 bytes) were byte-identical across acquisitions, but
all 13 mtimes changed. Cargo explicitly marked the Arrow/tensor bridge source
files stale, then their dependent probes and final binary dirty. The traced
runtime role executed 24 rustc processes: 21 check units (including bridge
libraries and dependent probes), two release bridge libraries and one final
binary. The initial preparation observed 927 rustc invocations across distinct
probe/final target families, mostly cold dependency work. These are invocations,
not unique crates; exec-to-parent-SIGCHLD durations are diagnostic observations,
not exact compiler phase timings and cannot be added as elapsed wall time.

The frozen raw traces, fixture snapshots, commands, resource records, capped and
passing receipts, descendant cleanup records, helpers and reconciled summaries
are preserved in `v3d-evidence.tar.gz` (12656 manifest-listed files), SHA-256
`5c8b96dc0d53add2474f30284abdcfc4b05b41e85e2c5d42c32cc81ae0fb7a7c`.
No V5 timing or cold-build feasibility claim follows from these instrumented
observations. The fixture fix cannot eliminate the demonstrated initial native
compilation. Mutable native target partitioning by complete lock authority is a
separate hypothesis requiring its own contract and negative controls.

### V3e bounded implementation — one retained advanced-data input owner

Preserve the current advanced-data fixture path and both role/test identities.
Replace the generic reusable-fixture allocator with one explicitly named logical
advanced-data owner per compiler checkout. This bounds production retention to
one tree per owner; there is no revision-based allocation or general eviction
registry. Cap the freshly composed final input inventory at 64 MiB and 4096 files,
reject over-budget input before mutation, and retain permanent lock inodes.
Tests of the helper use isolated temporary storage rather than adding retained
production fixture identities. Existing unknown owners remain protected.

Freshly read the canonical fixture and compose the rebased runtime manifest and
evidence entrypoint before reconciliation. Preserve bytes/mtimes of unchanged
files, restore altered/missing inputs, remove stale files/directories, handle
file/directory replacement, and reject source/destination symlinks and nonregular
inputs. Reuse the existing private file writer. Retain the owned tree on drop;
no marker or previous success authorizes skipping fresh source validation.

Use the existing explicit native-child lease primitives. Carry input ownership
through Cargo metadata, canonical package planning before any probe, generated
build/capture and runtime command construction. Unrelated exec children must not
inherit the fixture lease; intended children must keep it after parent death.
Do not use process-local unlock-on-drop ownership for these child-held leases.
Public callers without fixture ownership retain their existing behavior and all
resolution/trust/profile identities remain unchanged. Split cached entrypoint
responsibilities if needed to preserve the source-size guardrail.

Named acceptance: unchanged-byte/mtime and final-overlay tests, mutation/missing/
stale/file-directory restoration, source/destination alias rejection, size/count
bounds, concurrent owner exclusion, and actual intended-child versus unrelated-
child lifetime controls. Retain the original fixture regression identity. Run
both advanced-data native roles on the candidate (separate 30-minute caps) and
inspect Cargo freshness on the retained-input follow-up while preserving every
runtime assertion. Run the affected native child/resolution identity controls,
ordinary driver suite, format/file-size/source-ownership checks, profile checks
and unchanged emitted-plan comparison. Use exact-candidate scoped independent
review before merging. Broader native/Python/matrix work, full final correctness
gates and every V5 scenario remain required; no sub-ten-minute claim is made.


### V3e closure — retained final inputs and explicit child ownership

Merged [#4413](https://github.com/sifr-lang/sifr/pull/4413), reviewed candidate
`29417909829d1b81cc65ae36a655346229b95ed5`, merge
`53d0a8ead1dc8395867b2761eee66f09ce1918e9`. One fixed advanced-data fixture owner
now reads canonical files and composes both overlays before reconciling its
retained tree. It preserves unchanged mtimes, restores current content, removes
stale paths/type conflicts, rejects aliases and keeps permanent lock inodes.
Its lease reaches metadata, package planning before probes, native build/capture
and runtime commands. Unix parent descriptors remain close-on-exec; intended
children explicitly retain them. Existing Windows behavior is unchanged.

The implementation retains at most one tree per checkout with 64 MiB/4096 final
files; canonical traversal additionally counts directories toward a stricter
4096-entry limit. Private controls use temporary storage. No revision allocation,
previous assertion result, resolution identity omission or eviction service was
introduced. Cached entrypoint orchestration moved into its own module to remain
under the source-size guardrail.

Validation on the reviewed candidate:

| Check | Result | Observed seconds |
| --- | --- | ---: |
| Focused fixture ownership/reconciliation | 5 passed; original regression ID preserved | 72.747412 including compilation; 1.13 test body |
| Advanced-data native preparation | Exact role passed | 118.865451 outer |
| Advanced-data runtime roundtrip | Exact original status/stdout/stderr assertions passed | 74.252214 outer |
| Ordinary driver suite | 759 passed, 82 ignored; all 837 prior IDs retained and 4 added | 353.886846 outer; 342.823507 execution; 2.662712 preparation |
| Formatting | Passed | 12.392669 |
| File-size guardrail | Passed | 9.883333 |
| Source ownership | Passed | 9.431571 |
| Profiles check | Passed | 1.430621 |
| Plans/configuration comparison | Both correctness plans and all 8 configurations unchanged | 1.187438 |

The ordinary suite includes all 48 project-cache tests plus native child-lease
and Cargo-resolution identity controls. Light guards overlapped the focused
compilation, so their times are correctness observations, not additive gate
performance. The initial extracted-module/visibility compilation failure was
repaired and preserved separately. Final reviewed checks passed without source
mutation; native and ordinary diagnostic sessions had no surviving descendants.

The follow-up native role still invokes Cargo 22 times but executes **zero**
rustc processes, compared with 24 before. All 13 final fixture files are identical
to baseline; bytes, mtimes and inodes survive both candidate roles and remain
unchanged after runtime completion and lease release. Actual descriptor evidence
shows the Cargo/rustc children retaining the fixture lock while parent handles
remain close-on-exec. The killed-parent control independently proves intended
child retention and unrelated-exec exclusion.

The 111.272127s to 74.252214s runtime comparison is one instrumented observation
with different preparation/capture initial states, not paired V5 qualification.
It proves avoided fixture invalidation, not a cold-build speedup or a complete
sub-ten-minute gate. The earlier capped cold preparation stays incomplete.

Evidence: `v3e-evidence.tar.gz`, all 1388 manifest-listed files verified, SHA-256
`125d58e66838024d4ebb4973be923a5ed8bf6bbf2c3f188b8636fc57427d2353`.
Exact-candidate independent review: **SATISFIED**, no blockers; response SHA-256
`925a2beeda96c732b9ffe9bb8d0ca0d1f61db44612453a7497e28a65ad4dac69`.
The fixture owner retains nonblocking follow-ups to align entry/file count
wording, document that `target` directories are skipped at every depth and empty
canonical directories are not retained, and keep/remove the redundant private
validation guard deliberately. These do not affect the current 13-file fixture.

CI's [create-PR job](https://github.com/sifr-lang/sifr/actions/runs/39646337313/job/119382785062)
again stopped before correctness setup because `SIFR_PERFORMANCE_REFERENCE` is
unset. This existing qualification configuration failure is preserved separately;
no CI-green or final-gate pass is claimed. Broader native/Python/command-matrix
work and every final correctness/V5 scenario remain open.

### V3f bounded item — native bundle inventory and content processing

The successful V3e runtime executes an 894150648-byte native executable despite
requiring no Rust compilation. Current `NativeSnapshot::inspect` hashes the
initial bundle, and every call site immediately calls `with_runtime`, which
hashes the complete inventory again. Finalized output verification separately
compares current bytes. Source inspection proves the duplicate pass; its share
of runtime is not yet measured. Do not conflate this with fixture invalidation,
change application features/debug information, or weaken content validation.

First measure the current inspection, runtime-inventory and verification stages
using the actual selected compiler/dependency configuration and retained bundle,
with explicit ownership, resource preflight, a 30-minute category cap and fresh
processes. Preserve any abbreviated or failed evidence. An isolated diagnostic
may compile verbatim current source fragments against the actual resolved
identity library; record source/library hashes, Cargo artifact selection and all
commands, and distinguish it from complete gate timing. Compare an owned
prototype that gathers the complete inventory before hashing once. Preserve
exact field ordering, domain, final digest and existing failure outcomes; do not
use a metadata-only shortcut, memoized digest or cached successful assertion.

Adopt the single-final-inventory approach only after equality/negative controls
and a measured benefit. Keep every caller under its existing native-family and
artifact leases, keep fresh finalized-content verification and runtime capture,
and preserve same-name library conflict rejection and existing sidecar checks.
Cover missing/changed payloads, captured corruption, runtime-library conflicts
and destination/order binding. Update all actual native application/test runner
callers together; no fallback or alternate pipeline is authorized.

Named candidate checks: affected native reuse and new bundle controls, the
original advanced-data preparation/runtime roles (with retained-input freshness
verified), ordinary driver suite and its preserved inventory, formatting,
file-size/source-ownership guards, unchanged profile/configuration selection and
independent exact-candidate review. Use repeated alternating diagnostic cases
where needed to separate the hash benefit from page-cache/preparation state.
No full-gate claim follows from a microbenchmark; complete final gates, cold
invalidation feasibility, broader fixture work and V5 remain mandatory.


### V3f closure — one complete native bundle identity / #4415

[PR #4415](https://github.com/sifr-lang/sifr/pull/4415) merged the reviewed candidate
`9e76e34b5efd66b3566901ae42fe14271a05fd9d` as
`14df6a918a7cbf8ab245437c241d1495f6d03144`. Every application/test-runner
caller now gathers the complete ordered executable/sidecar/runtime inventory
before hashing once. The identity domain, ordered fields, final digest, leases,
fresh content verification/capture and rejection controls are preserved.

Before implementation, six alternating fresh processes compared verbatim baseline
source with the complete-inventory prototype using the actual resolved opt-level-1
identity library and retained 894150648-byte native executable. Median construction
was **9.306315s → 4.659510s**; all six final identities were identical. Median
fresh verification was **0.975585s → 0.977047s**. The initial colder baseline
verification remains recorded. All 14 paired equivalence/rejection controls pass.
This is a stage diagnostic, not a complete gate measurement.

| Candidate check | Result | Observed seconds |
| --- | --- | ---: |
| New native bundle controls | 6 passed | 83.757664 including driver compilation |
| Advanced-data preparation | Exact original role passed; 22 Cargo, 0 rustc | 64.745797 outer |
| Advanced-data runtime | Exact original assertions passed; 22 Cargo, 0 rustc | 49.717955 outer |
| Complete ordinary driver | 765 passed, 82 ignored; 841 prior IDs retained plus 6 | 350.387076 outer; 2.313073 preparation; 340.991334 execution |
| Format / size / source ownership | Passed | 8.729483 / 7.221962 / 6.727934 |
| Profiles / plan comparison | Passed; both correctness plans and all 8 configurations unchanged | 0.723825 / 1.229412 |

The driver suite includes actual native-reuse/dylib, child-lease, Cargo-resolution
and all 48 project-cache controls. The 82 ignored roles are not counted as
ordinary passes. All 13 retained native fixture files preserve bytes, mtimes and
inodes. No validation source mutation or surviving diagnostic descendants was
observed. Light guards overlapped compilation; their timings are not additive.
The 49.72s runtime versus V3e's 74.25s is a single instrumented observation with
different cache/capture state, not a paired whole-workload claim.

One diagnostic mistake is retained: the initial native preparation used direct
Python rather than the preceding pinned `uv` launcher. Its changed PATH produced
a different native-toolchain family and unnecessary compilation. It was cancelled
at **139.283143s**, all descendants stopped, and repeated with the matching
launcher. This initial attempt remains **incomplete**, not a pass and not evidence
of a product regression. The original V3d cold preparation cap also remains open.

Evidence: `v3f-evidence.tar.gz`, all 4617 manifest-listed files verified, SHA-256
`9048d6aa37a1de4270c30765fecf06fcfe5615d190fb1d745fb1ada65667886f`.
Independent exact-candidate review: **SATISFIED**, no blockers; response SHA-256
`212f0ce0cfaeed319b87b84d96c16306036a1f3c551f7aeeb3cd4b3b4ea19823`.
The native-storage owner retains separate follow-ups for clearer sidecar-variable
naming and unnecessary intermediate publication. Foreign-platform golden-digest
qualification remains outside this current-host phase. No content memoization
or reduced freshness assertion was introduced.

[CI create-PR](https://github.com/sifr-lang/sifr/actions/runs/39651139339/job/119399451696)
stopped before correctness setup because `SIFR_PERFORMANCE_REFERENCE` is unset.
This existing qualification-configuration failure retains its separate owner;
there is no CI-green or complete-gate claim. Complete native/Python/matrix coverage,
cold build feasibility, integrated resource scheduling and V5 remain open.

### V3g bounded investigation — complete native role execution and attribution

V3e/V3f qualified the advanced-data preparation/runtime pair and preserved all
ordinary driver assertions. They do not establish execution or timing of the
complete selected native inventory. Before another native optimization, refresh
that complete inventory and execute every selected role from the canonical merge
configuration, retaining preparation and each role's result separately. V4e
observed six CLI and 82 driver roles; reconcile exact names, not just counts.

Use the pinned verification launcher and existing owned compiler/native targets.
Prepare the selected Python environment, exact CLI/driver test configurations and
required metadata/native prerequisites. Record preparation once as preparation;
do not hide it in bootstrap or count it as all downstream assertions passing.
Keep source/argument/output/diagnostic semantics and each isolated test process.
Record observed candidate, configuration, selected binary identity and exact argv.

Each preparation category and native role has its own 30-minute diagnostic cap.
Collect independent exit failures and continue to the next role after proving all
owned descendants have stopped. A deadline is incomplete evidence, never a pass.
Stop further launches for an unresolved resource reserve, surviving child or
unexpected source change. Preserve failures and partial logs. Check effective
CPU/memory/free disk and target size before heavy preparation; keep the existing
20-GiB disk and 2-GiB memory reserves and bounded nested worker allocation.

Report every role, its assertion status, outer elapsed time and any incomplete
preparation. Retain compiler/Cargo invocation evidence and actual dirty reasons
for slow or surprising cases, distinguishing trace overhead and concurrent
process intervals from wall time. Do not infer an optimization benefit from
counts alone. Check that unchanged fixture inputs retain bytes and metadata.
The known alternate-launcher cancellation is not a product-cache regression.

This item authorizes diagnostics and evidence records only. Register an explicit
bounded repair after identifying a root cause; do not silently change native
selection, result reuse, Cargo authority, trust, features, durability or public
diagnostics. The remaining Python examples, command matrices, cold SQL
feasibility, integrated execution and V5 acceptance remain separately required.

### V3g closure — complete native inventory and repeated cold fixture authority

The observed candidate is `e22445c7bf4b887c2d767cc8fb7422510c182fce`.
The canonical merge inventory exactly matches V4e: six CLI and 82 driver native
roles. All **88 passed**, including complete 732-positive-fixture emission and
1,307-fixture semantic/native frontend comparisons. No role failed, timed out or
was skipped. The protocol child helper remains explicitly identified; role counts
are not counts of independent semantic assertions. Required preparation passed,
and every owned process session ended empty with no tracked-source change.

| Diagnostic category | Roles | Sum of outer role times |
| --- | ---: | ---: |
| CLI generated builds | 6 | 106.498s |
| Package/workspace metadata builds | 7 | 51.115s |
| Process protocol helper | 1 | 3.838s |
| Complete emission/semantic corpora | 2 | 193.133s |
| Python interoperability | 9 | 389.322s |
| Rust interoperability | 43 | 3,359.339s |
| Package namespaces/specialization | 2 | 26.195s |
| Workspace/project compilation | 18 | 161.876s |

Role time totals **4,291.316s**. Separate preparation is **215.032s**; between-role
orchestration is **17.687s**. The sequential campaign took **4,524.035s**. These are
instrumented diagnostic observations, not a complete gate runtime or V5 result.
Per-category caps stayed at 30 minutes. The largest role was resource alias
rejection at **396.494s**, followed by resource lifecycle at **379.985s**. Read-only
analysis and one disk inventory overlapped parts of the campaign, so these are
not idle-host comparative benchmarks. From role 030, smaller native graphs used an
8-GiB prospective data working allocation; advanced-data retained 20 GiB. The 20-GiB
disk and 2-GiB memory reserves and actual resource monitors were unchanged; both
helper versions and the allocation decision are retained.

The traces contain 745 nested Cargo and 6,425 rustc process observations, with no
unparsed exec events or missing rustc completion signals. The separate C-tool
attribution retains missing parent-completion events; it does not claim complete
C-tool duration coverage. Fifty-three nonzero
compiler exits occur in passing rejection roles. The original summary incorrectly
classified these as absent completions; preserved `trace-summary-v2.json` files
correct that analysis using the same raw signals. This never changed a test
result. Cargo separately reported 109.980s of fresh outer test-graph preparation
across the 88 isolated invocations; that time is already included in role totals.

The HTTP loopback role takes 258.919s. Its observed Cargo intervals cover 247.799s,
while the generated runtime command takes 0.181s. The same TLS build-script basename
appears in two separate Cargo families: 89.427s for the probe and 108.291s for final
build. These are nested/overlapping process intervals, not additive removable
savings, and matching basenames alone do not prove complete build equivalence.

Repeating that exact role with the same selected test binary passes in 257.336s.
An fd2-write observer captures 310 missing Cargo fingerprints and two generated
probe-root file freshness reasons, with no unparsed fingerprint messages. The
repeat selects new probe/final mutable families. Independent reproductions of
`IdentityEncoder`, lockfile digests and probe-family policy exactly match both
observed probe-family hashes. The canonical lockfile bytes, native toolchain,
owner and trust/environment policy are unchanged; changing the temporary
fixture lockfile path accounts for the changed probe family. This proves the
fixture-lifetime cause without removing authority fields or bypassing Cargo.
The two probe-root freshness reasons remain legitimate work; not every compiler
invocation is labeled waste.

Conversely, the retained advanced-data fixture's 13 files (163,640 bytes) preserve
all captured bytes, timestamps, inode identities and modes relative to V3f after
preparation and both selected roles. Its selected preparation/runtime roles pass
in 56.307s/35.864s with zero rustc invocations. Different initial states prevent
using those observations as a paired V3f-versus-V3g speedup claim.

Frozen evidence: `v3g-evidence.tar.gz`, **126,195** files,
SHA-256 `d02d0e2c2c061d87f0628f719bbf04826a45ce516c7f4dd5843f410eb5c5ce47`. It includes every per-role command/result/resource/trace,
process guard, inventory, helper version, source identity proof, corrected summary,
repeat diagnostic, manifest and sortable `v3g-report.html`. Local extraction is
`local-gate-implementation-20261020/v3g-reviewed-diagnostic` and is verified against
the per-file manifest. This is record-only diagnosis; no implementation review is
claimed or required. The phase remains open.

Next: retain final Rust-interop fixture inputs under explicit ownership. Separate
follow-ups retain the observed probe/final target split, full Python import-probe
partitioning and repeated outer Cargo preparation. No profile switch or shared
trust-context shortcut is authorized by these observations. Remaining Python
examples, command matrices, mandatory cold SQL feasibility, final integrated
gates and all V5 scenarios remain required.

Record-only closure and next contract: [#4417](https://github.com/sifr-lang/sifr/pull/4417).

### V3h bounded implementation — retained Rust-interop fixture inputs

Generalize the existing `ReusableScenario` owner across the native Rust-interop
scenario tests. Preserve each scenario's current assertions and intentional
pristine-to-mutated transitions. Canonical fixture bytes and final static
manifest/source/bridge overlays are prepared before a single reconciliation;
unchanged retained files keep their timestamps. Ownership identity includes the
checkout, fixture, scenario and named test variant, not a fresh PID/time suffix.
Keep the accepted advanced-data identity unchanged.

Route every use of the retained inputs through that explicit owner: metadata,
canonical package checking, native builds, direct Cargo controls and generated
runtime commands. Add a private ownership parameter to the existing canonical
check path if needed; retain its public default and behavior. Do not introduce a
second checker, implicit path-based ownership inference, globally inheritable
descriptors, or a compatibility fallback. Audit all direct command and mutation
helpers, including native build-script and SQLx mutation controls.

Fresh independent native build-script targets A/B and the armed-sentinel control
remain fresh. No assertion pass is cached. Keep complete lock, toolchain, source,
Python, trust, profile and finalized-artifact identities. Do not change generated
application profiles, Cargo family partition policy, dependency versions, fixture
selection, emitted diagnostics, runtime outputs or expected failures in this item.

Acceptance: preserve the exact selected native names and every scenario branch;
run all affected Rust-interop roles and their prerequisites under the existing
30-minute per-category cap. Demonstrate repeated unchanged runs of the expensive
HTTP, resource, backend and advanced-data cases with actual Cargo freshness
reasons, file bytes/metadata and compiler/process evidence; report cold retained
preparation separately, and include it in later integrated acceptance. Validate
source/manifest/lock/bridge/trust mutations, variant separation, stale-file and
type reconciliation, alias/size rejection and explicit intended-child ownership
across parent death. Preserve unrelated-child noninheritance. Run focused driver
regressions, formatting, size, ownership and applicable coverage self-tests, then
one exact-candidate independent review. Full final phase gates and V5 remain
required; no ten-minute result follows merely from this repair.

### V3h closure — retained native fixtures and private child outputs

Merged [#4418](https://github.com/sifr-lang/sifr/pull/4418), candidate
`f9f033797d6c79d1b0207b2ce1e81459c933bbef`, merge
`a889d7df46f595e84984a2167dd8c7dba53f9348`. Final static overlays now precede a
single owned fixture reconciliation. Metadata, checking, native compilation,
direct Cargo controls and runtime commands retain explicit intended-child leases.
Fresh A/B and armed-sentinel targets and all assertions remain independent.

The first candidate's complete repeat exposed three real permission regressions:
SQLx recreated query metadata and native/proc-macro sentinel outputs inherited
mode 0664. Four exact offending files were renamed into a private evidence
quarantine under exclusive fixture leases; their bytes, modes and identities were
preserved. The repair uses the existing private writer for SQLx recreation and
sets umask 0077 only in the owned Cargo child after fork. It leaves the parent mask
and strict fixture validation unchanged. A subprocess control starts from parent
mask 0002, performs two fresh Cargo builds, verifies private recreated files and
successful reacquisition, and verifies that the parent mask remains unchanged.

| Final-candidate validation | Observed result |
| --- | --- |
| Fixture / child-lease controls | 8 / 3 pass; guarded 97.249s including rebuild |
| Static, ownership and matrix checks | 6 pass; 246 matrix self-tests; guarded 27.735s |
| Repaired native scenarios | All three pass twice; four recreated files mode 0600 |
| Complete Rust-interop category | Same 43 roles pass; 824.152s |
| Additional freshness observations | 6 pass; 118.546s; no missing fingerprints or dirty reasons |
| Ordinary driver regressions | 768 pass, 82 explicitly ignored native roles; 850 total |
| Ordinary driver wall time | 355.091s execution, 2.368s inventory; guarded 363.396s |

The 43-role baseline was 3,359.339s; the retained repeat is 824.152s, a measured
2,535.188s (75.47%) reduction. Both have 481 Cargo invocations; rustc invocations
fall from 5,653 to 379. All 53 expected nonzero compiler outcomes match per role.
There are no unparsed exec events or missing rustc completions. Every captured
fixture mutation is explained by an intentional control. Snapshot roots are those
exposed by explicit Cargo metadata manifest paths; direct-Cargo-only sentinels are
covered separately by actual controls, repeats and explicit permission inspection.
The six fd2-only observations establish Cargo freshness, not compiler counts.

These are instrumented diagnostic observations, not idle-host statistical results
or a complete gate. Retained preparation from the prior candidate is declared;
every final-candidate assertion ran again. Initial retained preparation and failed
repeats remain separately recorded. The earlier 42-versus-43 orchestration error
was reconciled by executing the omitted adjacent role. A stale local diagnostic
helper caused a separate 51.728s cancelled repeat; its source, raw logs and process
cleanup receipt remain preserved. Neither failed nor cancelled run counts as a
pass. All completed diagnostic process sessions ended empty and tracked source
stayed frozen. Parallel driver completion timestamps are not individual durations.

Frozen evidence: `v3h-fixed-evidence.tar.gz`, 150,945 files, 29,816,214 bytes,
SHA-256 `e4a0f4c59559c245bac5babdfc0e8bce23e2982b9d7003e8ec3a669cfaf59789`.
The local `local-gate-implementation-20261020/v3h-fixed-reviewed-diagnostic`
extraction was checked against every manifest hash and size; carried baseline
summaries also match the prior frozen V3g manifest. Independent exact-candidate
review is **SATISFIED**, no blockers; response SHA-256
`e00f65911aa911af48ae0e13a7d32f4d5c90c0b56846cf9354ff38551a61af9d`, stored
outside the reviewed tree under `reviews/f9f033797d6c79d1b0207b2ce1e81459c933bbef`.
Test-maintenance follow-ups retain consistent safety comments and using the private
writer if future mutations recreate files rather than modifying existing ones.

[CI qualification](https://github.com/sifr-lang/sifr/actions/runs/39679071645/job/119496483938)
stops before correctness setup because `SIFR_PERFORMANCE_REFERENCE` is unset.
[Determinism](https://github.com/sifr-lang/sifr/actions/runs/39679071645/job/119496483862)
reports a missing signature without published inner diagnosis; its root cause is
not established. Prior Windows qualification failed in unchanged libpg_query/18
MSVC C sources requiring unavailable Unix headers. These retain their CI and
cross-platform SQL owners. No CI-green, full-gate or V5 claim is made.

### V3i bounded investigation — streaming native bundle bytes

The remaining native bundle path reads entire files into memory for identity and
holds two full file buffers for each content comparison. An actual advanced-data
executable is 894,150,648 bytes; comparing two copies requires about 1.79 GB of
payload allocation. Observed advanced-role peak RSS is about 1.85 million KiB.
Additional filesystem reads vary across retained runs even with zero rustc work.
The allocation is proven by code; neither its end-to-end cost nor streaming gains
are established. Large debug sections remain part of the required artifact bytes.

First run an isolated, owned prototype against the actual retained bundle, using
fresh byte reads and alternating baseline/proposed observations. Preserve exact
IdentityEncoder field framing, order and digest; executable, sidecar and runtime
library inventory; basename-conflict rejection; source/destination error handling;
leases; capture semantics; and every fresh content verification. Bound working
memory without memoizing content, trusting size/mtime as equality, removing debug
information, changing profiles or dropping assertions. Hashing a declared stream
length must fail on truncation, extra bytes or read errors without leaving a
partially updated encoder. Equality must handle partial reads and interruptions.

Use isolated owned copies and existing leases when reading mutable source paths.
Keep source code unchanged during measurement. Record elapsed, CPU, filesystem
I/O, peak RSS, input identity, allocation/reserves and all failures. No OS cache
flushing or cold-build timing claim. Cover empty/boundary/large streams, equal and
unequal content, unchanged-size/mtime corruption, sidecars/runtime basenames,
read failures and identity equivalence. Apply the 30-minute diagnostic cap and
existing disk/memory reserves. Stop if the safe working allocation cannot fit.

This contract authorizes prototype diagnostics only. Adopt production changes
only after evidence supports a separately recorded bounded implementation contract
with affected tests and exact-candidate review. Full native coverage, remaining
Python examples and command matrices, cold SQL feasibility, integrated resource
scheduling, final create-PR/merge gates and all V5 scenarios remain required.

### V3i outcome — bounded bytes prototype qualifies for implementation

The isolated prototype uses frozen candidate
`f9f033797d6c79d1b0207b2ce1e81459c933bbef` and the current Cargo-selected opt-1
identity/SHA libraries. It runs against independently owned copies of the actual
894,150,648-byte advanced-data bundle. Original family/artifact leases cover the
copy; all timed captures target only owned copies. Both copies have SHA-256
`4ac1d7351faafff9bd30dddcd1d1d8bab02cbfddc7a7e16f8468b97752184a76` before and
after the experiment. Preparation passes in 3.187s; the guarded experiment takes
62.230s, including 22.705s of copying/hashing. Both process sessions end empty.

Six fresh processes alternate baseline/prototype/prototype/baseline/baseline/
prototype. All identities and required payloads match. Median observations:

| Actual bundle stage | Whole-file baseline | Bounded prototype |
| --- | ---: | ---: |
| Inventory plus identity | 4.642s | 4.220s |
| Fresh verification | 0.953s | 0.183s |
| Unchanged capture comparison | 0.948s | 0.184s |
| Complete measured process | 6.547s | 4.598s |
| Peak RSS | 1,748,848 KiB | 2,600 KiB |

Thirty-nine stream controls pass: empty/boundary/multibuffer payloads, independent
field encoding, arbitrary short reads, interruptions, unequal lengths/content,
truncation, extra bytes, injected read errors including after mismatches, and no
partial encoder update after failure. Fourteen existing bundle cases also pass,
plus explicit unchanged-size/mtime corruption and repair. Current-source and
captured corruption, sidecar order/aliases, runtime basename conflicts, missing
sources and destination binding remain covered.

Copying and preflight hashes intentionally warm the page cache. This is neither
cold-I/O evidence nor a complete gate speedup. The implementation still reads all
current bytes, including after detecting a mismatch so that later I/O failures
remain visible. Source read errors always propagate; destination read errors can
trigger capture under the existing capture policy, but verification rejects them.
There is no source/profile/dependency/selection change in this investigation.

Frozen prototype evidence: `v3i-evidence.tar.gz`, 81 files, SHA-256
`8a48bfb86ca34e0a10b4f99b0b6c56f03592e67b3c146eacc420ac1b7ec3a432`.
All extracted manifest hashes and sizes match. The two large owned copies remain
remotely with exact hashes recorded; all source/control/raw measurement evidence
is archived in `local-gate-implementation-20261020/v3i-reviewed-prototype`.

### V3j bounded implementation — streaming identity and native comparisons

Implement the qualified bounded streaming operations in the existing authorities.
Add a transactional reader field to `sifr_identity::IdentityEncoder`: retain the
exact name/length/payload framing and commit only after exactly the declared bytes
and EOF are read. Retry interruptions, accept partial reads, reject short/extra
payloads and propagate errors. Keep the existing byte-slice API and identity
schema unchanged. File identity uses the length from the same open file handle;
no metadata or digest memoization.

Move native fresh byte comparisons into a focused internal helper with an
explicit destination-error policy. Bound working memory, drain the streams so
late errors are visible, and preserve strict source errors. Use it for basename
conflict checking, verification and capture. Preserve required inventory/order,
sidecar/debug payloads, copy behavior, family/publication/artifact leases and
current diagnostics. No dependency, profile, trust, durability, selection or
result-reuse changes. Keep generated and public behavior unchanged.

Acceptance: run focused identity stream controls and native snapshot/comparison
controls, including unchanged-size/mtime corruption and missing/unreadable inputs;
all ordinary driver regressions; canonical retained advanced preparation/runtime
and representative HTTP/resource/backend native roles with exact selected binary
identity. Compare the large-bundle diagnostic against the production candidate,
recording CPU, elapsed, I/O and peak RSS, and retain every required byte check.
Use existing owned targets, safe resource reserves and per-category 30-minute cap;
record preparation separately. Run formatting, size and source-ownership checks
and one exact-candidate independent review. Final integrated create-PR/merge gates,
complete current-host selection, cold feasibility and all V5 scenarios stay open.


### V3j closure — bounded native byte processing / #4421

Merged [#4421](https://github.com/sifr-lang/sifr/pull/4421), candidate
`325f011c1b71c785c693d4d9cf69211ce40487d0`, merge
`8e8d49895aa6362e4d594e0096eaa069d01f8684`. The existing identity authority now
supports transactional, exactly length-delimited reader fields. Native basename,
verification and capture comparisons use bounded buffers, read current bytes and
retain their explicit destination-error policies. All source errors remain fatal.
Inventory, aliases, sidecars, diagnostics, copy behavior and leases are unchanged.
No identity schema, profile, dependency, selection or assertion-result cache changed.

Four identity and ten native byte/snapshot controls pass. Guarded focused validation
is 148.029s, including a 138s driver rebuild. Whitespace, formatting, size and
source ownership pass. The complete ordinary driver passes **772 tests**, with
82 explicitly ignored native roles and 854 total names, exactly the prior inventory
plus four new regressions. Execution is 371.070s, inventory 2.618s and guarded time
380.400s; the prior guarded observation was 363.396s. No ordinary-suite speedup is
claimed. All 48 project-cache cases and affected native/cache/lease checks execute.

The canonical 82-role native inventory and Python/metadata preparation pass.
Seven selected advanced preparation/runtime, backend SQLx, HTTP runtime/rejection
and resource alias/lifecycle roles all pass with the same selected binary as the
ordinary suite. Their execution sums are 202.727s before and 193.667s after, single
instrumented observations. Cargo counts match per role (86 total); both sides have
zero rustc invocations, no unparsed exec events and no missing compiler completions.
Advanced preparation maximum reported RSS falls from 1,846,076 KiB to 262,416 KiB.
Every owned diagnostic process session ends empty and source remains frozen.

The production bundle experiment preserves the exact legacy identity and all
payloads on the same independently owned 894,150,648-byte copies. All bundle and
unchanged-size/mtime corruption controls pass. Six balanced production observations
have these medians:

| Actual bundle stage | Whole-file baseline | Bounded production |
| --- | ---: | ---: |
| Inventory plus identity | 10.685s | 6.359s |
| Fresh verification | 12.855s | 14.479s |
| Unchanged capture comparison | 13.006s | 15.160s |
| Complete measured process | 36.609s | 35.964s |
| Maximum reported RSS | 1,748,696 KiB | 2,644 KiB |

Unlike V3i's low-I/O prototype observations, **every production run reports
8,731,960 filesystem input blocks**. Preliminary hashes did not guarantee retained
pages. Historical raw descriptions that call these warm-file measurements are
explicitly corrected in `v3j-io-observations.json`; neither individual stage
slowdowns nor the differing regimes are hidden. The source of the cache/I/O
variation is not established. Memory bounds are proven and measured; uniform
elapsed-time improvement is not. Cold and full-gate qualification retain this cost.

A separate balanced 64-KiB/1-MiB/4-MiB buffer probe passes all variant bundle controls
and identity comparisons but returns to low-I/O conditions. Larger buffers make
comparison stages slower there, and unchanged-construction variation obscures
small total differences. It does not establish their storage-heavy behavior, so
no larger-buffer change is adopted. No OS cache flushing or profile change was used.

Frozen evidence: `v3j-evidence.tar.gz`, 3,647 files, 4,216,991 bytes, SHA-256
`04ead928d8f428753ab13ca13429100343a27c5a421b914acb2e9f085831f6e0`.
Every extracted manifest hash/size and 21 carried V3h baseline files match the
previous frozen extraction. Evidence is retained under
`local-gate-implementation-20261020/v3j-reviewed-diagnostic`.
Independent exact-candidate review: **SATISFIED**, no blockers; response SHA-256
`e4d9bd256d83f2d86259d1240169262d86a1869401fb6291f032076310627403`, outside the
reviewed tree under `reviews/325f011c1b71c785c693d4d9cf69211ce40487d0`.
The identity/native-storage owners retain optional helper deduplication and
smaller-thread-stack follow-ups; current buffers are 64 KiB for identity and
128 KiB per comparison. The full-gate owner retains the observed I/O regimes.
These suggestions do not introduce new acceptance requirements or justify a
public utility API solely to remove a small local helper duplication.

V3h's record/next contract merged through
[#4419](https://github.com/sifr-lang/sifr/pull/4419); V3i's result/V3j contract
through [#4420](https://github.com/sifr-lang/sifr/pull/4420).
Complete current-host gates, cold feasibility and every V5 scenario remain open.
No CI-green, full-gate timing or phase-completion claim follows from this item.

### V3k bounded investigation — complete compiled-Python lifecycle and execution

Refresh the canonical compiled-Python inventory before another lifecycle repair.
The historical inventory has 29 examples across nine suites; reconcile the current
manifest, declaration/fixture inventories and actual configured cases by name.
Explain additions/removals rather than silently selecting a historical subset.
Distinguish compiled examples from static/self-tests, live external integration
and the nine separately measured driver-native Python roles. Preserve the approved
current-host scope; this item does not launch release or external-live contracts.

Prepare the exact selected CLI, pinned Python environment and required metadata
once through the canonical context/ownership helpers, retaining actual binary,
interpreter, configuration and candidate identities. Run every current compiled
example and its declared assertions, diagnostics, exit and output markers. Record
preparation, each example/suite and cleanup separately. Use the production
launcher/runner when available; any isolated diagnostic invocation must preserve
its exact arguments and environment and must not be labeled a canonical gate pass.
If a suite stops before later examples, retain that failure and execute independently
runnable remaining examples under their own recorded boundaries.

Each category/example has the existing 30-minute diagnostic cap; preserve any
stricter canonical timeout as a separate outcome. Continue independent cases after
proving owned descendants stopped; stop launches for surviving children, source
drift or insufficient safe reserve. Inspect current CPU/memory/disk/target capacity
and declare realistic per-category work allocation plus the existing 20-GiB disk
and 2-GiB memory reserves. Retain prepared targets and failure evidence; no shared
cleanup. Do not invent results for an unavailable environment or external service.

For costly or surprising repeated preparation, repeat a representative unchanged
example with the same selected products and capture actual nested Cargo context,
compiler invocations, dirty reasons and fixture bytes/metadata. Keep observations
separate when instrumentation differs. Trace Python probe/venv identity without
collapsing it to the base interpreter or dropping ABI/trust/environment inputs.
Identify the responsible owner and register a bounded repair before changing
fixtures, reuse policy or compiler behavior. No test-result caching, dependency
pin changes, assertion omissions or global resource-policy change is authorized
by this diagnostic item.

Report complete selected names, statuses, observed timings and incomplete work.
The remaining complete command matrices, PostgreSQL default/explicit 18 and
13→18→13 reuse proof, V1 cold feasibility, V4 integration and V5 stay required;
this diagnostic does not close V3 or the phase.

### V3k closure — complete compiled-Python lifecycle diagnosis

Frozen source candidate `84d5c4ed3769bacec7b98e6060a37af9fbcba11b` selects
**29 compiled examples in nine suites**, within a merge Python area containing
30 suites. Every compiled example, source-policy check, required output marker,
certification and immutable certification recheck passed. This is a complete
standalone compiled-example diagnosis, not a full Python-area or profile pass.
The separately owned driver-native Python roles are not double-counted here.

| Compiled suite | Examples | Preparation | Execution |
| --- | ---: | ---: | ---: |
| Arrow | 1 | 134.763s | 79.239s |
| Async context | 1 | 67.223s | 35.707s |
| Async declaration | 1 | 54.716s | 22.700s |
| Buffer | 5 | 309.863s | 58.723s |
| Callbacks | 7 | 316.366s | 82.739s |
| Dataframes | 3 | 190.292s | 43.216s |
| DLPack | 1 | 99.746s | 46.711s |
| Libraries | 8 | 467.450s | 107.255s |
| ML | 2 | 135.766s | 42.716s |
| **Total** | **29** | **1,776.184s** | **519.007s** |

Shared bootstrap is 263.897s and required metadata preparation 9.694s. The sum of
these guarded categories is **2,568.782s (42m49s)**. Bootstrap includes the actual
candidate CLI rebuild; its preparation/hash portion is 260.624s. Python environment
preparation itself is 0.080s. These are instrumented observations on retained
owned targets, not quiet-host timing acceptance or a clean-build benchmark.
Each category preserves the 30-minute outer cap and the runner's stricter
600-second command timeout. No testcase reached either timeout.

An original buffer-preparation attempt stopped at preflight before any case ran:
free space was below the declared working allocation plus reserve. That failed
attempt remains failed. Lossless compression of 24 previously archived obsolete
executables reclaimed 12,881,174,528 bytes; verified archival of one obsolete CLI
test executable reclaimed 1,315,295,232 allocated bytes on the build volume.
All original executable bytes remain recoverable, with hashes, modes and restore
instructions. Current products, dependency/incremental targets, leases and failure
reports were retained. Capacity recovery is administrative work, separately
reported rather than hidden inside or counted as a passing test. The retry has a
new label. Every completed diagnostic session ends empty and source stays clean.

Both preparation and execution invoke Cargo 168 times. Preparation records 2,896
rustc invocations, including target queries, across separate native families.
Execution records 58: one generated application and its build script per example.
The build and run applications have distinct generated identities; their dependency
families are reused. This does not support describing the entire cache as broken.
All compiler completions were observed, with no nonzero compiler statuses or
unparsed exec events. Equivalent normalized dependency argv across families is
only a diagnostic lead, not proof that environment or trust identities can merge.

Execution contains 37 fresh Python environment probes totaling 132.454s of observed
process intervals, 25 signature-inspection processes totaling 11.000s and 16
individual distribution-version queries totaling 1.454s. The 29 actual example
program processes total 21.332s, including startup/imports. Parent intervals
include children and must not be added together as independent wall time.
Preparation likewise has 37 environment probes totaling 137.434s. Lock checks
are below one second in each aggregate; lock validation is not the demonstrated
bottleneck. The runner recreates fixture packages, repeats certifications and
compiles an exported preparation product which execution does not consume.
Removing preparation would still leave the first required native compilation;
its complete 29m36s is not a defensible saving estimate.

An isolated replay of the exact Arrow environment probe takes 3.611s. Under
cProfile it takes 7.806s and produces the identical complete JSON payload.
`packages_distributions()` is called seven times, accounting for 6.601s of
profiled cumulative time and repeatedly walking distribution file metadata.
Profiling overhead prevents treating that cumulative time as an unprofiled saving.
The current code scans after each import, including overlapping declared/native
roots. A naive first-scan cache would change observation semantics when imports
alter Python paths or metadata, so it is not accepted as the repair.

The unchanged Arrow preparation replay passes in 75.727s; a distinct runtime
replay using the profile's binary/probe-cache selections passes in 86.234s.
These replays additionally trace writes and retain fixture bytes/metadata and
Cargo fingerprint output. Preparation invokes Cargo 19 times and rustc once;
Cargo explicitly reports a newer build-script dependency for the generated
application. Runtime invokes Cargo 19 times with zero rustc invocations. Fixture
recreation preserves the eight common input payloads but changes metadata and
removes the generated lock/certification records before regenerating them. The
full cause of the build-script timestamp change remains with the native lifecycle
owner. These replays do not reproduce full-profile scheduling and must not be
used as an uninstrumented A/B speedup claim. Exact replay compiler counts,
input changes, raw diagnostics and limitations are retained in `v3k-findings.json`.

Frozen evidence: `v3k-evidence.tar.gz`, 64,332 files, 8,554,101 bytes, SHA-256
`021c58abf246f3237d8344500e83a8086e0cd75e5f9ded29cf7f2f7f6603a968`.
The extracted manifest is checked against every retained file under
`local-gate-implementation-20261020/v3k-reviewed-diagnostic`. Recovery payloads
remain separately retained with included hashes and restore records.

The full profile, cold feasibility, remaining command matrices and PostgreSQL
qualification remain open. There is no sub-ten-minute or phase-completion claim.

### V3l bounded repair — one fresh Python metadata observation per probe

Qualify and implement a coherent per-invocation environment observation in the
existing `sifr_package` Python probe authority. Preserve selected interpreter and
venv identity, declared/native root order and multiplicity, actual imports,
origin/error records, distribution names/versions, ABI/platform/library fields,
trust inputs and project/lock digest checks. Do not resolve a venv interpreter to
its base interpreter. Do not cache across processes or skip any required import,
certification, assertion or lock check.

The proposed observation contract is explicit: perform declared and native import
attempts in their existing order, then discover distribution ownership once from
the resulting environment and attach fresh metadata to each successful import.
Capture environment paths consistently with that final observation. Preserve
per-root import failures and fail closed for unavailable distribution metadata;
never convert an observation error into an empty successful record. Each command
still makes a fresh observation. This intentionally replaces interleaved partial
metadata snapshots with one post-import observation; do not claim adversarial
import-time mutations have byte-identical old behavior. Define and test their
expected final-state behavior before implementation is accepted.

First qualify an external prototype against all distinct recorded example probe
requests and the selected pinned interpreter. Compare complete ordinary-environment
payloads, including all names, versions, origins, errors, paths and digests. Add
isolated controls for duplicate/overlapping roots, missing/failing imports,
namespace/distribution ownership, changed paths or metadata during import,
missing/failing metadata, and a second invocation after an environment change.
Retain failure attribution and ordering; do not swallow interrupts as successful
observations. If those controls cannot support a clear safe contract, record the
rejected approach rather than adding heuristic invalidation or compatibility paths.

For a qualified implementation, run focused probe/package tests and affected
Python environment/certification contracts, followed by all 29 compiled example
assertions with the actual selected candidate. Measure paired unchanged probe
observations without profiling, separately recording compiler/environment setup
and unchanged native preparation. Compare full emitted payloads in ordinary cases
and explicit expected final-state results in mutation controls. Retain every
failed/incomplete observation and the existing resource reserves/time caps.
Run formatting, file-size/source-ownership checks and exact-candidate independent
review. Full integrated create-PR/merge gates and V5 stay required at phase end.

Other measured costs stay assigned to the gate/native-build owners: duplicated
example lifecycle work, separate dependency families, repeated compiler selection,
and full-profile scheduling. This item does not remove coverage, merge native
trust contexts, alter dependency pins/profiles, or claim it alone meets 600 seconds.


### V3l closure — fresh post-import Python metadata / #4424

PR [#4424](https://github.com/sifr-lang/sifr/pull/4424) merged as
`0cc269031b8d68ca63caf4bd4526d0c7d5e02c02`. The independently reviewed candidate is
`958be37c1281f40ef3fe92ccb332986edebb4b5a`, against base
`39390166a60946cdac66aa82059db92123d706cd`.

The probe preserves declared/native import order and multiplicity, then discovers
distribution ownership once from the post-import environment. Every invocation
still imports and observes freshly. Ownership failures fail otherwise successful
roots while retaining actual import failures; version lookup errors remain
root-specific. Interpreter/venv, ABI/platform/library, project/lock digests and
native trust checks remain intact. There is no cross-process memoization or
removed certification/assertion. Import-time mutations intentionally follow the
registered final-state contract rather than interleaved old metadata snapshots.

All 36 focused qualification categories passed on the frozen candidate. The
package suite passes 170 tests with zero failures/ignores. All 29 compiled examples
in nine suites pass preparation, execution, source-policy checks, output markers,
certification and immutable rechecks. All nine driver-native Python roles pass;
the complete native inventory remains the same 82 roles. Affected Python self-test,
environment, readonly and binding contracts pass, as do formatting, file-size and
source-ownership checks. Ten new observation controls exercise the authoritative
embedded script, including errors, interrupts, overlapping roots, mutation during
successful/failed imports, real namespace ownership and a new process after
metadata/lock changes. Both ordinary and optimized-Python control runs pass.
The preliminary static run used system Python 3.14.4; authoritative qualification
repeats the checks with the exact pinned Python 3.14.7.

| Focused observation | Guarded elapsed |
| --- | ---: |
| Compiled-example preparation, all nine suites | 387.427s |
| Compiled-example execution, all nine suites | 310.346s |
| Nine driver-native Python roles | 304.361s |
| All 36 focused categories, including setup and other checks | 1,388.125s |

These are isolated category sums on retained targets, not an integrated invocation
or final timing acceptance. V3k was traced while V3l is untraced; their suite totals
are not a controlled before/after speedup. The separate untraced ABBA experiment
replays all 29 distinct requests, 116 complete observations, preserving every
ordinary-environment JSON payload. The sum of per-case medians is 50.205s baseline
versus 23.358s candidate (approximately 53.5% lower). That supports the probe-only
benefit, not a predicted full-gate saving. All completed sessions end empty and
tracked source remains unchanged.

Frozen evidence `v3l-evidence.tar.gz`: 928 files, 551,398 bytes, SHA-256
`bdffef85f23f5fa1d4e18aff4721d0c0e35821975419c38309fc4599142f4432`.
All 927 manifest payload files were independently hash/size verified locally under
`local-gate-implementation-20261020/v3l-reviewed-diagnostic`.
Independent review is **SATISFIED**, with no blockers; external response
`reviews/958be37c1281f40ef3fe92ccb332986edebb4b5a/request.ZMuNau/response.md`, SHA-256
`49a14f317b864e937ca5f4752ef903186b08d7b5f798a8888f4ff0f849ecde7b`.

Follow-ups remain separate with the Python verification owner: improve control
failure messages/style, and compare selected-venv identity against the parent's
expected venv rather than a redundant same-process prefix assertion. The full
29-request equality matrix supplies the actual selected-venv identity evidence.
The unused `do_import=False` branch is pre-existing. CI create-PR again fails before
setup because `SIFR_PERFORMANCE_REFERENCE` is absent; preserve that existing
infrastructure failure, not a CI-green claim. Full gates, cold feasibility, remaining
command/PostgreSQL matrices and V5 remain open.


### V4e resumed diagnostic — integrated failure discovery before further optimization

The user directed on 2026-10-22 that, after V3l closes, the next milestone is a
complete integrated correctness result, not another isolated optimization. Resume
V4e's existing bounded diagnostic contract on the combined published candidate.
Inspect the emitted `merge --correctness-only` plan and compare selection with the
canonical manifests. Verify exact tools, prepared targets/interpreters, CPU/memory,
working allocation, disk reserves and source ownership before launching. Verify
that the exact revision used by generated portable Cargo graphs is published and
fetchable; retain the old unpublished-revision and disk-reserve failures separately.

Run `scripts/run_all_tests.sh --profile merge --correctness-only --no-fail-fast`
with the established diagnostic safety bounds, recording ordinary elapsed time
without strace or profiling. Continue independent failures, but never execute a
consumer whose prerequisite failed. Preserve required checks, variants, assertions,
clean/invalidation work and every failed/incomplete receipt. Do not disguise cold
preparation as bootstrap. If the aggregate diagnostic cap stops the invocation,
list every unexecuted category and explicitly schedule remaining independent
canonical categories under their own 30-minute caps with their real prerequisites.
A cap, blocked prerequisite or missing run is not a pass.

Reconcile all selected coverage and report executed coverage, remaining failures,
total observed invocation time (separately from follow-up category sums), and the
three largest measured costs. Use those results to select the largest root cause.
Batch related repairs under one bounded acceptance contract and exact-candidate
independent review. Keep focused intermediate validation and input-bound evidence
reuse; broaden testing when integration changes justify it. Reserve repeated final
timing qualification for a credible integrated candidate. Final create-PR/merge
correctness gates and every V5 scenario remain mandatory. This sequencing change
neither weakens acceptance nor establishes that the full gate can meet 600 seconds.


### V4e resumed diagnostic outcome — selected categories reconciled

The frozen candidate is `f2f841679a4f61b8c4252791d30fd2cfbc44ecb7`.
The diagnostic inventory is complete at the selected-category level: all 105
selected area suites have reports, all 18 Cargo configurations pass, and all 732
E2E fixtures pass. This is **not** a complete passing integrated invocation or V5
acceptance. Known failures and blocked nested controls remain below.

The original integrated attempt reached its 30-minute diagnostic cap during
preparation (1,802.337s). Independent follow-ups continued the canonical selection
under their own caps. Recorded follow-up invocations sum to 26,506.035s; initial
follow-up guards/prerequisites add 497.941s. These are diagnostic effort totals,
including failed/capped attempts, repetitions and resource refusals. They are not
a steady-state gate duration. The separately executed architecture tail adds
127.755s; archive administration and targeted failure replay are separate.

External cleanup removed build artifacts during a connectivity interruption. The
user confirmed cleanup was finished and authorized rebuilding. Matching source
remained unchanged. Old preparation readiness was invalidated; the continuation
rebuilt required artifacts. Those post-cleanup costs are cold diagnostics, not
normal iteration performance. No obsolete compressed recovery archive was deleted
by this work.

| Completed category | Observed duration and coverage |
| --- | --- |
| E2E | 553.995s guarded; 732 source/runtime cases, 186 native groups pass. No native cache manifest entries at start; not a warm-hit timing. |
| Cargo configurations | 2,710.960s summed execution; 245 target summaries, 5,557 passing test executions, no failures. Repeated feature/role configurations are included. 94 ignored and 74,283 filtered observations are not passes; separate selected configurations execute many filtered/ignored roles. |
| 19-package additive compiler configuration | 1,287.980s preparation; 236.351s execution; 2,781 tests pass. Its codegen target runs 1,780 tests in 8.25s; LSP takes 162.96s, including nested Cargo work. |
| Cache/compiler/semantic services | Preparation 6.190s / 157.798s / 216.811s; execution 1.681s / 24.694s / 4.689s; 14 / 24 / 14 tests pass. |
| SQL | All 66 selected cases recorded: 62 pass, four provenance failures. Case-body sum 2,736.160s; 30 preparation commands total 1,201.303s. Nested bodies must not be added to their invocation totals. |
| Python | All 30 suites pass, summed invocation time 1,866.200s; all 29 examples prepare in 1,530.072s. Combination passes in 1.229s, certifying seven capabilities, nine compiled evidence entries and three resource-zero evidence entries. Compiler/authoring preparation is additional. |
| Sysroot release | All four selected suites pass: installed smoke 138.759s, boundary equivalence 239.821s, metadata structural 95.297s, metadata corpus 884.180s. Source preparation 1,245.962s; package preparation 1,098.832s. Separate diagnostic suite processes repeat packaging that the combined adapter shares. |
| Installed metadata details | 732 public emission commands take 471.614s; four native programs 89.264s; nine other recorded commands 33.084s. Live generation, relocation and rejection/recovery controls pass. These are nested subprocess times; copying, LSP controls, source corpus, packaging and orchestration also belong to the enclosing suite. |
| Guards / architecture policy | 15 selected guards accounted for, 13 pass and two fail. All 54 architecture-policy tests accounted for: 53 pass, one unsafe-inventory failure; the 21 tests after its original fail-fast stop were explicitly executed. |

Among complete individual assertion invocations, the SQL clean A / unchanged A /
independent clean B protocol takes 1,416.898s, generated driver roles 1,311.998s,
and generated-code-quality 1,035.881s. The clean SQL protocol already disables
debug information; repeating a SQL debug-disable experiment is not justified.
No category sum or failed-path duration establishes full-gate feasibility.

All manifest case identities reconcile. Two sysroot report IDs use suite names
where the manifest uses explicit aliases; the ledger preserves both identities
and records the mapping. Two diagnostic-helper mistakes are retained as failures:
V7 used a report path outside the repository, and V8 initially combined Python
reports in profile order rather than canonical manifest order. All Python suites
were rerun with valid paths; canonical-order combination then passed. Production
assertions and the production combination contract were not changed.

Open owners remain: method-dispatch/unsafe/filesystem/transfer inventory
reconciliation; diagnostics hygiene incorrectly matching a deliberate Rust error
fixture; generated-program ASan's 180-second timeout; package-area file size and
stale digest-type guard; imported-call generated Rust ABI failure; and stale SQL
component provenance after V3j. The architecture failure is the same missing
unsafe inventory, not a separate implementation defect. Both SQL mutation
entrypoints reject their stale baseline before executing controls: those controls
remain **blocked**, not measured passes. Rebuild actual components before their
qualification; never relabel stale artifacts. The declared missing future
HTTP/text-i18n fixture remains a skip under its capability owners.

Confirmed avoidable work:

- Sysroot source preparation uses a private Cargo target despite matching all
  534 completed library/build-script/binary fingerprint JSON inputs in the main
  compiler target. Final binaries differ by 1,344 bytes; fingerprint equality
  does not claim binary equality. Both independent source and installed behavior
  still require qualification after sharing the canonical preparation authority.
- Python preparation and execution differ only in PATH among native toolchain
  identity inputs: execution adds another verification-venv activation. A
  reconstructed preparation identity matches the real header; adding that entry
  predicts the observed runtime header exactly. The real runtime process confirms
  the PATH, and a representative prepared/runtime main source and lockfile match.
  Generated binary names differ. No fix or speedup has been measured yet.
- The prior attribution that package recreation deleted prepared native output
  was too strong: that output is a sibling of the recreated package, and survives.
  Fix the proven launcher mismatch before redesigning fixture lifetime.
- Versioned corpus preparation changes `SIFR_RELEASE_VERSION` on the ordinary
  target. Structural qualification rebuilds compiler crates in 47.56s, then corpus
  qualification rebuilds them in 45.92s. Preserve version-specific behavior;
  naive target sharing across incompatible identities is not a solution.
- Three codegen library fingerprints have the same direct features/profile but
  different transitive fingerprints. They are not interchangeable cache entries.
- Package hashing starts separate hashing/formatting processes for roughly
  23,000 files. A streaming equivalent is a candidate, not a measured saving.

Main development compiler preparation and packaged release preparation consume
4,583.740 CPU seconds combined, before the broader checks. The extra private
source build consumes another 2,453.480 CPU seconds. Even idealized use of eight
logical CPUs needs 572.968s for the first two preparation invocations alone at
that observed CPU work. This is a lower bound, not a parallel-runtime prediction.
Cold/invalidation acceptance needs less compilation work, not scheduling alone.
GNU time's reported swap field does not establish zero Linux swap I/O; this
diagnostic did not collect per-invocation VM-counter deltas. Maximum reported RSS
is a single-process maximum, not simultaneous process-tree memory.

Evidence remains outside the Git tree under
`local-gate-implementation-20261020`. The sortable
`integrated-diagnostic-report.html` includes every selected suite, all recorded
invocations, SQL/Python details, 245 Rust target summaries and 745 installed
qualification commands. Frozen diagnostic archive:
`v4e-resumed-complete-diagnostic-evidence.tar.gz`, 2,135 files, 15,499,878 bytes,
SHA-256 `b701b0e7b9108e867cd0d28961f33c9d04fe7263634aa6c48004faac063a4be4`.
All payload hashes/sizes were independently verified locally. Detailed Python
archive SHA-256 `7bcd9c315caa5b507c7013b02eeee66232918cf40b3b9581789b5e6ed638aab7`
and sysroot archive SHA-256
`db10e09d424539650f76eb2140f0594c4d148d665e633583831645d2e1fcb1f5`
are bound by included manifests and separately verified. Completed owned sessions
are empty; the candidate remains clean. V4e's diagnostic inventory is recorded;
correctness repairs, V1 feasibility, remaining V3 matrices and V5 remain open.

### V4l bounded repair — matching preparation and execution contexts

Register this item before implementation. The root contract is that a prepared
development compiler or Python native graph is consumed under the same intended
build context. The diagnostic proves two runner-created mismatches: a duplicate
source-compiler target and redundant verification-environment activation.

Use the already selected verification interpreter for verification-owned child
processes. Keep locked activation of the separately owned Python-interop project.
Do not normalize arbitrary PATH entries, discard environment inputs, weaken
native identities, reuse test pass results, or change selected assertions.

Use the canonical Cargo-selected development compiler authority for sysroot
source qualification, respecting the caller's target and flags. Cargo must still
establish freshness and select the actual executable; an existing pathname is
insufficient. Preserve locked/offline behavior, inherited deadlines, the existing
900-second source-consumer limit, failure propagation and child cleanup. The
versioned packaged release compiler remains a distinct required producer.

Named acceptance:

- Focused launcher/segmentation and Cargo/sysroot preparation self-tests, including
  exact inherited environment preservation, selected interpreter identity,
  canonical suite order, failed/missing preparation, executable selection and
  deadline behavior. Preserve existing failure/zero/missing-evidence controls.
- Compare emitted create-PR and merge correctness plans and run profiles check;
  selected suites, crate configurations and exclusions must remain unchanged.
- Exercise real preparation-to-consumer identity: matching Python native headers
  and Cargo freshness, and the canonical source compiler in the retained target.
  Preserve distinct flags/targets and reject a mismatched compiler override.
- Run all 30 Python suites and complete compiled certification, all four selected
  sysroot suites, and representative ordinary area launches (rust_interop and
  regression). Record preparation, assertions and ordinary elapsed time separately;
  do not infer a full-gate saving from avoided cold work.
- Run file-size and diff checks. Obtain a SATISFIED independent review for the
  exact implementation candidate under the approved intermediate policy.

This item does not solve release-version invalidation, compile-profile costs,
package-hash process overhead or the recorded correctness failures. Those remain
bounded follow-up owners. Final create-PR/merge correctness gates and every V5
scenario remain required; no sub-600-second result is claimed by this repair.


### V4l closure — preparation/execution context coherence / #4427

Implementation candidate `982e9119d0e45882aafba0e5c1d99415aca11c09` is qualified
against base `c1c2c33d3098fed4f7b37244303b80fae78c8c38`. PR #4427 merged as `dcc3d033861269212ce9ebbeb315ce35dce81343`.
Verification-owned children now use the selected interpreter directly; the
separately owned Python project still uses locked activation. Sysroot source
qualification uses the canonical Cargo-selected compiler in the caller target,
with offline/locked preparation, inherited deadlines and fail-closed errors.
Structural-only selection now enrolls source preparation as well.

The 19 focused controls pass (3.652s), full runner foundation checks pass
(93.221s), and plan/profile/lock/size/diff checks pass (5.224s). Emitted correctness
plans remain exactly equal: 71 create-PR and 105 merge suites. The original
foundation invocation omitted the required `--` before `self-test` and did not
execute tests; retain that failed invocation beside the corrected passing run.
The validated working diff matches the committed candidate; Git's default index
abbreviation differs between checkouts, so the binding uses explicit nine-digit
abbreviations when reproducing its SHA-256.

Real affected qualification passes all 41 suites / 53 top-level variants, with
nested assertion evidence preserved:

| Category | Ordinary observed duration / coverage |
| --- | --- |
| Python | 572.544s; all 30 suites; complete certification: 7 capabilities, 9 compiled entries, 3 resource-zero entries. All 29 examples prepare in 295.356s. |
| Sysroot | 1,354.583s combined; all four suites. Boundary 312.928s; installed smoke 23.462s; complete metadata corpus 770.922s; structural 242.848s. All 732 installed public emissions and native/integrity/recovery controls pass. |
| Rust interop | 32.202s; all five suites, 13 variants. |
| Regression | 38.199s; both suites, six variants. |
| Other preparation | Workspace 2.185s; compiler identity 93.298s; canonical sysroot source 6.687s; Python environment 1.185s; authoring 74.244s; sysroot package 53.719s; maintained demos 47.710s. |

Independent live-process observations prove preparation and Python execution have
identical native inputs. Reconstructing both tool identities agrees with the
actual runtime Cargo family's header. No environment key or PATH entry is
normalized away. An initial unfiltered observation sampled an unrelated Sifr
command; the role-filtered observation and actual Cargo family corroboration are
the accepted evidence, with the investigation limitation explicitly recorded.
All 534 fingerprint JSON files in the old private source target preserve bytes
and mtimes. Thirteen actual Cargo controls pass (2.682s), covering edits, flags,
explicit host/target, lock drift, mismatched/matching overrides, invalid source
despite an existing binary, and recovery. Every owned process session ends empty;
source remains clean at the candidate.

These are retained diagnostic observations, not a controlled cold A/B speedup or
a full-gate duration. Sharing source preparation does not remove version-context
rebuilds; the combined corpus/structural sequence still rebuilds compiler crates.
Python preparation plus execution alone exceeds the whole-gate budget. V1
feasibility, other correctness repairs, final gates and every V5 scenario remain
open; do not mark the phase complete.

Evidence: `v4l-evidence.tar.gz`, 252 files, 2,774,057 bytes, SHA-256
`f05a073cc4ce608b55f59bec312a43620171711d94329d48f411c157b1326a0c`, outside
the Git tree in `local-gate-implementation-20261020`. Archive and every payload
hash/size were independently verified locally. Exact-candidate review:
`reviews/982e9119d0e45882aafba0e5c1d99415aca11c09/review.md`, SATISFIED
with no blockers, SHA-256
`731066266c82d05a880d59d6d6d034aeddc5c4aac0d10b80ef13df0fc1e29d84`. Review evidence remains outside the commit it approves.

CI is not claimed green: full create-PR admission rejects missing
`SIFR_PERFORMANCE_REFERENCE` before setup, as previously recorded. Deterministic
report CI reports a missing signature; its job log does not establish the
underlying cause. Preserve both with their existing CI owners. Intermediate
acceptance follows the approved phase policy, not a waiver of final qualification.

Non-blocking review follow-ups belong to the common compiler resolver: render
its failure message with a real newline; consider separating deadline data from
memoization identity only if that remains safe; and convert non-object Cargo
JSON into an explicit preparation failure. None changes this item's passing
contract or permits weaker freshness. They are recorded, not silently included
in the reviewed implementation.

### V4m bounded correctness repair — project slice-call ABI closure

Register this item before implementation under LOCAL-GATE's integrated
correctness dependency work, with emitted-Rust/codegen as technical owner. The
completed diagnostic reproduces OSS-CURATED-0002: source checking succeeds but
generated Rust passes an owned vector to an imported slice parameter. The fixture
and its checksum must remain unchanged. This is a compiler failure, not a timing
win or a fixture problem.

Source history places the slice cleanup in d16595d17b (2026-09-22), with later
project borrow work in e20efbe981 and LOCAL-GATE's eec85331b. This is not attributed
to V4l. Current source suggests slice signatures can change during per-file
canonicalization after project-wide call plans were collected. Prove the stage
boundary with a failing focused regression and an owned fresh-cache reproduction
before implementing the correction; source inspection alone is not final proof.

Make final caller/callee conventions coherent across the complete generated
project. Preserve lexical/module/owner identity, import aliases and reexports,
macro-contained calls, ownership-sensitive vector behavior and native error
reporting. Do not change a source fixture, insert untyped blanket borrows, suppress
a compiler error or add a fallback emission path. Keep per-file cleanup from
silently changing an exported ABI after project closure. Prefer one canonical
project transformation and explicit invariants over duplicate repair passes.

Named acceptance: focused before/after stage-boundary regressions, direct,
qualified and aliased/reexported calls, same-named functions in different modules,
macro calls, transitive chains, input-order independence/idempotence, and owned
uses that must not become slices. Run affected canonicalizer/project codegen
unit tests, native regressions that execute the original assertions, the full
codegen library suite, the complete ecosystem selection, and affected project
and interop regressions. Resolve named filters to actual tests and reject zero
execution. Run the full positive E2E inventory because this changes shared
code generation. Run Rust formatting, file-size and applicable codegen policy
checks; retain pre-existing guard failures separately and reject new drift.
Obtain exact-candidate independent review under the approved intermediate policy.

Retain preparation and each failed attempt, with the 30-minute diagnostic cap
and stricter inner limits. Final full correctness and V5 timing acceptance remain
open. This bounded repair restores valuable coverage; it is not itself a claim
that the full gate meets ten minutes.


### V4m closure — project vector calling conventions / #4429

Implementation candidate `ac7c15c39de2ce825af9fd4098f9896330ffa15a` is qualified
against base `454302290fb3176b73702cb1e34cb91c7c20ea65`. PR #4429 merged as
`663f15fd914f881ba9c29eeef63ce8c8269336ff`.

Per-file project cleanup now preserves vector signatures, including borrowed
vectors, function values and private functions consumed by child modules.
Finalization no longer changes signatures without their callers. The original
project borrow authority remains unchanged. The rejected late global repair was
removed after independent review exposed lexical identity and consumer gaps.

The untyped borrowed-vector literal rewrite was removed. Safe allocation
elimination now requires an explicit receiving shared-slice type, an unshadowed
builtin vec macro and list form. Numeric byte constructors and typed byte-method
boundaries emit arrays directly. Repeated forms, unresolved vector consumers and
non-literal expressions retain their semantics. All 264 demo companions were
regenerated and compared; only six ABI-related files changed, 11 insertions and
11 deletions. An earlier refresh that introduced unrelated byte/decimal vector
allocations was rejected and preserved before correcting those typed emitters.
No source fixture, dependency lock, lint debt, selection or assertion was weakened.

The original fresh-cache OSS-CURATED-0002 reproduction and native stage-boundary
probes establish the baseline E0308. The unchanged original fixture passes on the
final candidate. Eighteen native regressions compile and execute original,
canonicalized and finalized Rust, with idempotence and coverage of aliases,
reexports, distinct owners, shadowing, macros, owned/mutating uses, function values,
private parent/child consumers and 24-function chains in both module orders.
Typed literal controls cover evaluation order, non-Copy elements, runtime
repetition, shadowed macros and unresolved receivers. One driver regression
exercises the real formatting/finalization/project-const pipeline.

All 13 named qualification categories passed on the exact candidate:

| Category | Guarded seconds | Coverage |
| --- | ---: | --- |
| package (focused) | 15.832 | 1,782 unit tests and 18 native integration tests; zero ignored/filtered failures |
| driver (focused) | 260.065 | One actual driver pipeline regression |
| workspace (prep) | 2.907 | Workspace preparation |
| compiler_identity (prep) | 20.335 | Canonical compiler preparation |
| OSS-CURATED-0002 (oss-replay) | 27.368 | Original fresh-cache fixture and runtime assertions |
| ecosystem_compatibility (area) | 68.864 | One suite, 20 variants |
| full (e2e) | 154.438 | All 732 unique positive fixtures, 186 native groups |
| project_workspace (area) | 603.872 | Four suites, 17 variants |
| maintained_demos (prep) | 8.538 | Required maintained-demo preparation |
| rust_interop (area) | 31.406 | Five suites, 13 variants |
| regression (area) | 25.268 | Two suites, six variants |
| generated (prep) | 699.683 | All 93 positive generated entries |
| generated_code_quality (area) | 908.547 | One selected suite, all ten modes |

The sequential guarded sum is 2,827.121s (47.1 minutes). This is bounded diagnostic
qualification, not a full-profile run or V5 performance acceptance. E2E body time
was 129.532s; all original source/runtime assertions executed. Generated quality
passed inventory, corpus, native, panic scan, intrinsic panic lint, rustfmt,
strict Clippy, determinism, demos and freshness. Demo validation uses Cargo check;
native execution is reported separately. Freshness checks the maintained emitted
companions. The independent reconciler verifies exact nonzero suite/variant and
fixture counts, all 186 generated manifest/lockfile hashes, all 98 declared corpus
source inputs, candidate identity, no tracked source changes and empty owned
process sessions. No failed, capped, missing or skipped category was counted pass.

Scoped policy comparisons preserve exactly 20 identical baseline method-guard
failures and 218 prior filesystem findings. Four native-regression output
operations were inventoried; edited method sites/relationships were source-audited
without changing classifications or owners. Rust formatting, 900-line file-size
and diff checks pass. The full codegen rebuild earlier took 1,029.402s and compiler
preparation 293.939s; those recovery/diagnostic costs are retained, not presented as
unchanged-run timings. The driver qualification needed a distinct dependency
configuration; artifact existence alone is not Cargo freshness.

Exact-candidate independent source review is SATISFIED with no blocking findings:
`reviews/ac7c15c39de2ce825af9fd4098f9896330ffa15a/review.md`, SHA-256
`2592981f884597a1a9455044b0a886fcffedc1469e65877e1c096296e2140d4f`.
Review overlapped the last two test categories with their pending status explicit.
Its merge prerequisites were then satisfied on the unchanged candidate; the
reviewer explicitly required no recursive source review after that completion.
Review evidence remains outside the approved Git tree.

Final evidence: `v4m-final-qualified-evidence.tar.gz`, 171 payloads,
22,451,603 payload bytes, 5,094,158 archive bytes, SHA-256
`9f08f40af858a17b88451f3135138d5391b376b15d07ba90337890caa240aace`.
Archive and every payload hash/size were independently verified locally. Retained
R7 failed broad qualification is separate, SHA-256
`61f3edfd1022154a45a22398302a151ee88e92019900ce83d1b343ad27c8c605`.
R8–R10 repair evidence has 149 payloads, SHA-256
`afdaaa9584738072645ccb0f8ded0416b2bd45ed76f09f4164cec302049e4d81`.
Original failures, interrupted builds, helper/test-setup failures and rejected
refreshes remain failures; successful follow-ups do not relabel them.

CI is not claimed green. Exact-head logs show full-profile admission requiring
unset SIFR_PERFORMANCE_REFERENCE and a determinism job reporting a missing
signature without exposing the underlying failure. These match previously
recorded symptoms, not proof of identical causes. Keep their CI owners; the
approved intermediate policy does not waive final phase qualification.

Separate canonicalizer follow-ups: existing direct-name slice cleanup retains
lexical-shadow/repeated-form risks and method/trait/function-value consumer gaps
in single-file mode. The scalar-shadow authority needs analysis of textually
inherited macros across child inline modules. The diff narrows the old untyped
rewrite risk; typed-callback controls do not qualify all legacy cleanup paths.
No broad proof of compiler soundness is claimed. Other integrated failure owners,
V1 feasibility, final create-PR/merge correctness and all V5 scenarios remain open.


### V4n bounded repair — reconcile source-bound guard inventories

Owner: LOCAL-GATE V4 integrated guardrail reconciliation. Registered after V4m
closure before implementation. Completed broad discovery identifies the same root
cause across four registries: previously delivered lease, native snapshot and
retained fixture changes were not fully reconciled with their source-bound
ownership records. This item repairs those records after auditing the actual
current operation; it does not disable a guard or treat a failing check as passing.

Scope: method dispatch sites and relationships (20 preserved findings), unsafe
ABI external segment (16 missing operations/allowances), direct filesystem effects
(218 preserved unclassified/stale rows, including multiplicities), and the
TypeScript/Go transfer direct-read inventory (three sysroot helper operations).
Reproduce the current exact delta first. These counts identify the known baseline;
verify current source rather than assuming the counts cannot change.

Audit each added/changed operation against its source owner and contract. Preserve
semantic/build/tooling/output boundaries and source-bound fingerprints. Remove
stale records only when the old operation really moved or disappeared. For unsafe
entries, bind thread, lifetime, aliasing and ownership obligations to the actual
operation and its surrounding controls; guard admission alone is not proof of
safety. Inspect failed-readiness cleanup for owned child groups before documenting
blanket lifecycle guarantees. A discovered behavioral defect needs its own bounded
repair with native regression; do not conceal it in an inventory refresh.

Named acceptance: each affected guard and its complete self-tests, all architecture
policy tests without fail-fast hiding the tail, schema/coverage inventory and
file-size/diff checks. Compare the complete before/after site sets, classifications,
source partitions and exact operation bindings. No new guard bypass, waiver,
classification enum or broad unsafe allowance. Preserve previous failed receipts.
Rust sources and test assertions should remain unchanged for the inventory repair;
if source changes prove necessary, run their affected regression contract and
repeat exact-candidate source review. Obtain independent exact-candidate review
under the approved intermediate policy. Other diagnostics/package/ASan/SQL owners,
full phase correctness gates and V5 timing acceptance remain open.
