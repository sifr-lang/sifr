# Ad Hoc Phase: Complete Local Correctness Gate Within Ten Minutes

status: in progress; integrated acceptance pending
registered: 2026-10-19
phase_id: LOCAL-GATE
current_stage: V0.3b merged; V0.6a1 contract planning boundary in progress
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
closure loop. At that planning checkpoint, implementation remained pending and no full-gate pass was claimed.

After all implementation items merge, close only when V5 passes. Record remaining
out-of-scope work separately; no unresolved in-scope coverage or timing scenario
may be described as completed. A missed target triggers further root-cause work,
not a scope reduction or a promised speedup based on overlapping timings.

V0.2 merged as #4362 at `260110d9bc66f0812738650d63a80aab55dd3d12`.

**Current bounded item:** V0.3b for Python runtime
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

- **V0.6a1, current:** expose the existing contract-planning stage to its unit
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
  bridge directory. These must execute real probes with owned, valid fixtures;
  no synthetic-success path. Preserve relevant filesystem failure controls.
- **V0.6b/c:** ownership/pruning and emitted-output/cache-key assertions remain
  separately bounded after current failures are reproduced.

This is a test-layer distinction, not permission to skip production/native gates.
Planning cannot create runnable or cache-qualified success. Existing real probe
and native/runtime coverage remains required in the integrated gate. No removed
assertions, fallback compiler, dependency changes or timing claim are authorized
by the subdivision. Standard Rust/area guards and scoped independent review apply.
