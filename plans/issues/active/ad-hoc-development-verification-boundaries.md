# Ad Hoc Development and Verification Boundary Redesign

- Phase ID: `DEV-VERIFY`
- Status: in progress; D1 and D2 delivered; D3 baseline recovery, D4 durable history under qualification
- Recorded: 2026-10-28
- Related execution contract: [LOCAL-GATE](ad-hoc-local-correctness-gate-under-ten-minutes.md)

## Planning record

The recommendation below is preserved verbatim from the user-provided handoff.
Its historical observations and local evidence links retain their original context.
The eight-minute ordinary development gate is a proposed acceptance target, not a
measured result. Existing qualification requirements remain authoritative until
an explicit policy change is accepted, as described in section 11.

## Recommendation (verbatim)

**My final recommendation is to redesign the development and verification boundaries around the guarantees Sifr must provide.** That includes substantial changes where necessary: product identity, test organization, artifact ownership, build dependencies, storage operations, and verification orchestration.

The objective is a fast, dependable environment that stays understandable as the project grows. Eight minutes is the proposed acceptance target for the ordinary development gate; it remains to be demonstrated.

The following is the consolidated handoff for the next agent.

**1. Objective, scope, and operating principles**

Deliver a development environment where:

- Ordinary supported changes receive comprehensive local correctness feedback within **eight minutes, including preparation**, leaving margin below ten minutes.
- Every required correctness guarantee has an explicit owner and executable evidence.
- Test-only changes avoid rebuilding unrelated production artifacts.
- Compatible immutable products are reused; mutable test state remains isolated.
- Cache invalidation, concurrency, interruption, and failure recovery are reliable.
- Developers and agents do not routinely repair caches, clean disks, or manually reconstruct validation environments.
- The architecture remains simple enough to understand, review, and maintain.

Cross-platform qualification and benchmarks remain separate from this local correctness objective. The original 12–13-hour cloud performance protocol is a different workload and must not be used as this gate’s baseline.

Large changes are acceptable when they remove a root cause. They must earn adoption through correctness evidence and complete development-cycle measurements. Available agent capacity is a reason to investigate and validate thoroughly; it does not make unnecessary infrastructure desirable.

**2. Evidence and its limits**

The investigation combines three sources:

| Source | What it establishes | Important limitation |
|---|---|---|
| Historical Sifr measurements | Repeated compilation and preparation have produced substantial avoidable costs; several repairs delivered large improvements | Different candidates and retained build states prevent combining all savings |
| uv and Ruff source inspection | Practical patterns for test boundaries, prepared executables, isolation, profiles, and scheduling | Their workloads and timings are not equivalent to Sifr’s |
| CodeScene analysis | Complexity, historical coupling, declining health, and prioritization problems | It does not measure build time, prove coverage, or identify runtime bottlenecks |

The latest **saved diagnostic inspected** took **86m21s**, at candidate `88124daf`. It was unsuccessful and incomplete: all 732 native E2E fixtures passed, but 13 Python suites were unexecuted. Preparation, crate tests, and SQL verification accounted for approximately **76% of recorded CPU time**. This is evidence of where to investigate, not today’s passing baseline. [Diagnostic receipt](/Users/yaseralnajjar/work/sifr/continuation-evidence/local-gate-implementation-20261020/v1cw-received-result.json)

The current local source inspected during this handoff was `f2f841679a4f61b8c4252791d30fd2cfbc44ecb7`. It still contains broad product-input hashing and release application profiles in test compiler contexts.

The uv checkout inspected is `d19f1cd49`; Ruff is `b54922fd73`. These are observations about those checkouts, not claims about current upstream releases.

**3. What CodeScene contributed—and what remains unavailable**

The audit covered **54 report views, all 45 crate summaries**, additional folder breakdowns, all 85 declining entries, all 70 predicted-risk entries, all 36 improvements, and the three rising hotspots. Both available native PDF reports were downloaded.

It was not an individual source review of all 3,681 analyzed files, all 103 friction files, or all 30 recommended refactoring targets. Architecture mapping was unconfigured; runner X-ray had not been run; several coverage and organizational reports were restricted. Those gaps must remain explicit.

The important conclusions are:

| Finding | Engineering implication |
|---|---|
| Bundled SQLite contains 364,595 lines at health 1.14, approximately 30% of the health map | Separate bundled code from first-party prioritization |
| Sifr runtime `src` scores 8.78, while the combined runtime folder scores 1.78 | The aggregate does not justify rewriting the runtime |
| `profile_runner.py` declined 7.16 → 5.90; `run` has cyclomatic complexity 41 | Simplify execution-policy responsibilities and make failure transitions explicit |
| Collection-method emission and frontend graph/cache queries are rising hotspots | Inspect their responsibilities and change patterns before further expansion |
| Codegen, lowering, frontend, and driver have substantial historical coupling | Compare that coupling with actual dependency and invalidation graphs before changing crate boundaries |
| Generated demo outputs appear in risk rankings | Review generators and generated-output contracts separately |
| Ticket extraction recognizes `SHA-256` and `UTF-8`; author aliases may be duplicated | Correct mappings before drawing delivery or knowledge conclusions |

The CodeScene revision is **`bee445d6`**, which predates subsequent work. Historical findings must be checked against the implementation candidate.

Useful follow-up analysis should be targeted: examine the relevant refactoring/friction entries and function history when working on a specific boundary. Configuring architectural components would improve future analysis. Reading every historical commit or maximizing a global health score is not a prerequisite for fixing demonstrated build waste.

[Full CodeScene audit and report index](/Users/yaseralnajjar/Downloads/codescene-sifr-8159151-review/report.html) · [Structured measurements](/Users/yaseralnajjar/Downloads/codescene-sifr-8159151-review/measurements.json)

**4. Establish one authoritative inventory of guarantees and inputs**

Before changing test mechanics, reconcile the current phase, emitted gate plan, actual test discovery, and historical evidence.

For each obligation, record:

- The behavior or failure mode it proves.
- Its owning test and execution boundary.
- Required compiler configuration, dependencies, and prepared products.
- Mutable resources that require isolation.
- Whether it executes on every ordinary gate or qualifies particular inputs.
- Expected outcomes and how missing execution is detected.

Distinguish language coverage from repeated setup. Multiple tests may legitimately cover the same feature through different boundaries. Remove repetition only after identifying what each assertion proves.

Use one authoritative inventory with derived reports. Avoid introducing several manually synchronized manifests.

**Acceptance:** every existing obligation is mapped; new selections cannot silently drop ignored tests, doctests, negative cases, integration variants, or prerequisite-dependent consumers.

**5. Repair production and verification identity boundaries first**

Current identity construction traverses whole package directories and broad `crates`/`third_party` trees. Test content can therefore affect production identities even when production compilation excludes it. This remains confirmed in [identity construction](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_identity/src/build.rs).

The remedy is an explicit distinction between:

- Inputs that determine a production artifact.
- Inputs that determine a test executable or fixture.
- Inputs that determine whether previous verification evidence remains applicable.

Do not solve this with a blanket filename rule that ignores anything named “test.” Embedded fixtures, generated files, build scripts, manifests, feature selections, toolchains, and external inputs belong to the product that actually consumes them.

Move suitable public-behavior tests into cohesive integration targets. Keep tests of private implementation invariants with their owning libraries. Avoid creating hundreds of independently linked targets.

**Acceptance:** demonstrate an edit matrix covering assertion changes, fixture changes, implementation changes, input addition/deletion, features, toolchains, and generated dependencies. Verify both necessary invalidation and legitimate reuse. A stable digest alone is insufficient; observe actual rebuilding and consumer behavior.

**6. Complete explicit preparation and artifact ownership**

Adopt the uv lesson of selecting an executable once and running isolated tests against it.

Preparation should provide an explicit description of:

- Executable and artifact paths.
- Relevant input and configuration identities.
- Successfully completed prerequisites.
- Lifetime and ownership requirements.

Execution consumes these prepared products. It creates fresh mutable projects, environments, runtime state, and processes where the contract requires them.

Build commands remain appropriate when the test is proving build behavior. Otherwise, tests should not rediscover the compiler, select a different profile, or rebuild equivalent dependencies incidentally.

Repair publication and lifetime boundaries so consumers cannot observe partially written products or lose active artifacts to cleanup. Preserve distinct products for genuinely different feature graphs, sanitizers, interpreters, platforms, and independent-clean-build obligations.

**Acceptance:** concurrent consumers, interrupted preparation, producer failure, stale inputs, relocation, corruption, and cleanup all behave correctly. Measure actual compiler invocations and preparation work, not just cache-hit messages.

**7. Reorganize tests around the boundary each guarantee requires**

Ruff demonstrates that substantial semantic coverage can exercise real library APIs, while separate integration tests exercise product behavior.

| Guarantee | Required boundary |
|---|---|
| Parsing, typing, diagnostics | Real frontend APIs |
| Generated Rust and source maps | Real emission APIs |
| CLI arguments, streams, exit status | Prepared CLI subprocess |
| Native semantics | Compile, link, and execute |
| Python/SQL interoperability | Actual applicable integration |
| Installation and relocation | Actual installed package |
| Durability and crash recovery | Actual storage and process behavior |

Audit expensive groups assertion by assertion. For example, a declaration-only SQL check should not initialize an execution engine unless that is part of its contract.

Retain the full native corpus. Do not replace native semantics with snapshots or durable storage with an in-memory substitute.

**Acceptance:** preserve all mapped guarantees and relevant failure detection while reducing complete category CPU and elapsed time. Similar test counts alone do not prove equivalent coverage.

**8. Treat generated programs and native integrations as a compilation workload**

Separate measurement of:

1. Compiler/frontend execution.
2. Shared native dependency compilation.
3. Fixture-specific compilation.
4. Linking.
5. Program execution.

Eliminate remaining fragmentation across Python, SQL, generated-driver, installed-product, and other suites using the existing authoritative preparation mechanism.

Share compatible immutable dependencies while preserving source, environment, and process isolation. Preserve interpreter/ABI and build-configuration distinctions. Tests that verify environment creation or isolation still need independent environments.

Batch only where semantics remain intact. Process exits, globals, initialization, imports, and isolation can make combining fixtures into one executable invalid.

Test contexts currently select release application builds. Investigate which obligations require that configuration. Any change must account for optimization-sensitive behavior and build-versus-execution tradeoffs; it is not automatically safe to switch everything to debug.

**Acceptance:** changed fixtures compile appropriately, all runtime assertions execute, incompatible configurations remain separate, and complete category measurements improve.

**9. Fix expensive dependency and storage designs at their source**

Crate boundaries should reduce dependencies that consumers do not need. Neither Ruff’s architecture nor CodeScene implies that every large crate should be split.

For each proposed boundary change, inspect:

- Resolved dependencies and feature graphs.
- Build-script inputs.
- Rebuild propagation after representative edits.
- Linking and code-generation costs.
- Public responsibilities and ownership.

The key question is whether a consumer can stop compiling or linking a substantial dependency without duplicating authority elsewhere.

For durable history/publication, investigate whether small updates repeatedly copy, hash, or synchronize large generations. An incremental representation with immutable records and small atomic publication metadata is a candidate—not a predetermined implementation.

Preserve visibility, crash consistency, durability on successful return, reader ownership, history retention, and safe reclamation. First try to make all 4,097 real publications affordable. Moving the production-scale workload to a separate stress check would be a policy change requiring explicit acceptance.

**Acceptance:** meaningful improvements in complete edit cycles and production operation costs, with failure-injection and recovery tests appropriate to the new design.

**10. Simplify verification orchestration instead of adding special cases**

CodeScene and source inspection support separating these responsibilities:

- Selection and prerequisite planning.
- Preparation.
- Resource allocation and execution.
- Outcome recording.
- Coverage and timing assessment.

Keep the implementation proportionate: a small explicit execution plan and bounded runner may be sufficient. Do not introduce a general workflow platform unless the problem actually requires one.

Make outcome semantics explicit: passed, failed, blocked, cancelled, unexecuted, and legitimately reused evidence are different states. Continue independent work after failures; block dependent consumers correctly. Functional failure and timing-budget failure must remain distinguishable.

Evaluate nextest for ordinary Rust tests after boundaries are clear. Compare identical selections, including process-helper behavior, ignored-test enrollment, and separate doctest obligations. Retain non-Rust verification orchestration.

**Acceptance:** focused tests cover failure transitions and resource conflicts; integrated reports reconcile all selected obligations. The runner becomes easier to understand without weakening its behavior.

**11. Make qualification policy explicit**

Two goals must not be conflated:

- **Ordinary development feedback**, using valid retained products and comprehensive behavioral checks.
- **Expanded qualification**, including applicable independent clean builds, toolchain changes, reproducibility, and release-specific checks.

Input-bound evidence reuse can avoid repeating unchanged qualification. It requires complete recorded inputs and a supported environment contract. Unknown relevance means rerun.

This preserves evidence about unchanged inputs but does not freshly exercise every environmental condition on every invocation. That tradeoff must be accepted explicitly.

Until the canonical phase policy changes, the existing acceptance requirements remain authoritative. An eight-minute ordinary gate must not be reported as satisfying a broader gate it did not execute.

**12. Measure the complete experience and qualify the result**

The proposed budget is:

| Category | Engineering budget |
|---|---:|
| Preparation after a supported ordinary edit | 90s |
| Semantic and ordinary unit tests | 60s |
| Complete native corpus | 90s |
| Python, SQL, and other integrations | 120s |
| Ordinary installed-product checks | 60s |
| Policy, inventory, and runner checks | 30s |
| Reporting and cleanup | 30s |
| **Total** | **480s** |

These are design budgets, not forecasts or permission to skip checks.

Measure at least:

- Unchanged retained invocation.
- Test-assertion edit.
- Fixture edit.
- Representative frontend/codegen edit.
- Runtime edit.
- Relevant packaging or SQL build-input edit.
- Dependency/toolchain change and empty-cache behavior, reported separately where the accepted contract requires it.

Record candidate identity, machine resources, storage state, profiles, preparation, elapsed time, CPU, memory, I/O, build invocations, and coverage. Explain overlapping execution rather than summing overlapping durations.

Use repeated ordinary runs for timing qualification and targeted instrumentation for diagnosis. Include all required preparation in the timer. Averages must not hide frequent budget overruns.

Sufficient local CPU, memory, and fast storage are legitimate parts of the solution. Select hardware from measured workload and critical-path evidence. More cores cannot fix serial work or unsafe ownership.

**13. Execution sequence and completion criteria**

The implementation agent should:

1. Reconcile current repository state, active phase, inventories, and outstanding failures.
2. Establish a complete diagnostic baseline; account for every unexecuted category.
3. Repair production/test identity boundaries and explicit product consumption.
4. Re-measure, then address the largest remaining category through dependency, test-boundary, or algorithmic changes.
5. Simplify orchestration alongside the changes that clarify its responsibilities.
6. Evaluate scheduling and remaining profile choices.
7. Complete independent review and the applicable final correctness, invalidation, coverage, and timing qualification.
8. Update development instructions so the supported workflow is obvious and reproducible.

Use focused checks for intermediate changes under the existing approved policy. Run broader discovery when shared changes justify it. Preserve owned artifacts and evidence; avoid concurrent mutation of validation inputs.

The work is complete when the accepted gate repeatedly meets its target, all required obligations are accounted for, invalidation and concurrency are correct, and routine use needs no manual repair. A collection of individually faster tests is not sufficient.

**Supporting handoff material**

- [Previous conversation, including uv/Ruff analysis](/Users/yaseralnajjar/.codex/attachments/7f788114-0603-4ebe-8953-d336c0830932/Pasted%20text.txt)
- [CodeScene audit](/Users/yaseralnajjar/Downloads/codescene-sifr-8159151-review/report.html)
- [Code-health PDF](/Users/yaseralnajjar/Downloads/codescene-sifr-8159151-review/codescene-sifr-8159151-code-health.pdf)
- [Portfolio PDF](/Users/yaseralnajjar/Downloads/codescene-sifr-8159151-review/codescene-sifr-8159151-portfolio.pdf)
- [Active local-gate phase](/Users/yaseralnajjar/work/sifr/codebase/plans/issues/active/ad-hoc-local-correctness-gate-under-ten-minutes.md)
- [Verification contracts](/Users/yaseralnajjar/work/sifr/codebase/verification/README.md)

**This is the complete recommended direction: repair the boundaries that create unnecessary work, preserve the guarantees, simplify ownership, and prove the result across real development scenarios.** The remaining uncertainty concerns the final measured runtime and which architectural changes earn their place—not whether the objective warrants substantial work.

## Execution record — 2026-10-28

The user authorized implementation through completion and coherent batches before
expensive final validation. Intermediate candidates retain affected tests and
independent review; complete final correctness, coverage and timing qualification
remain required. This sequencing does not accept the proposed ordinary/expanded
qualification split in section 11 or waive LOCAL-GATE invalidation scenarios.

### D1 — dependency contract ownership (in progress)

Move the 17 repository dependency integration targets, support code and evidence
from the production manifest crate to `verification/tools/dependency_contracts`.
Retain all assertions, private production unit tests, profile enrollment, and
shared additive compiler configuration. Update policy ownership inventories.
Do not add path exclusions to production identity hashing.

Acceptance: discover and execute every relocated test; compare emitted profiles;
show that an audit assertion edit rebuilds its test without recompiling production
libraries or CLI, while a production input edit still invalidates its consumers.
Measure complete affected preparation plus execution and CPU, with a target of
at least 50% lower test-edit cycle cost. Cold reconstruction after confirmed
cleanup is reported separately and is not an ordinary-cycle speedup.

The retained diagnostic remains failed/incomplete. Its 125-file evidence archive
was recovered and SHA-256 verified; recovery does not change the test outcomes.
A separate eight-command Cargo publication probe confirmed that split-filesystem
outputs cause a fresh Cargo invocation to copy the executable again. A consistent
filesystem layout retained hardlinks without rewriting fresh executable bytes.
The cleaned compiler target will therefore be rebuilt as a whole target layout.


D1 focused evidence (candidate still under qualification): all 17 relocated targets
plus the production manifest unit tests passed, 108 tests total with zero ignored
or failed tests; the retained-target invocation took 4.378 seconds. Cold initial
compilation took 40.58 seconds but that first invocation failed on a stale owner
record. A second discovery run exposed the separate manifest-count record; both
records were corrected without removing assertions. The failed attempts remain
preserved. All eight delivery profile/mode combinations preserve prior commands
and configuration counts. The new suite joins the existing additive group.
Profile validation and 12 crate execution checks passed with pinned tools.
Rebuild controls, complete affected policy checks and independent review are
pending; these focused results are not a passing full gate or a measured speedup.


### D1 closure — 2026-10-28

- Merged [PR #4513](https://github.com/sifr-lang/sifr/pull/4513).
- Implementation candidate: 6418846ce142a7392c4f9ef45e2cf8a43eb26954.
- Merge: 6916f48778442aa5825cfd1389465ea832fee453.
- Matched audit-edit cycle, including CLI preparation and all 108 dependency and
  manifest tests: **16.435s / 40.125 CPU seconds → 4.857s / 4.449 CPU seconds**,
  70.45% lower elapsed and 88.91% lower CPU. The same 108 test names executed.
- Before: 13 production libraries and the CLI rebuilt. After: the audit target
  rebuilt, production libraries and CLI stayed fresh, and CLI bytes were identical.
  A production input edit still rebuilt consumers. The original candidate was
  restored and its 108 tests passed again; no source drift or surviving workers.
- All 56 affected architecture-policy tests and 12 crate execution checks passed.
  All eight profile/mode comparisons preserve previous commands and configuration
  counts. Formatting, file-size, filesystem effects and method policy passed.
- Independent review: **SATISFIED**, no blocking findings, for the implementation
  candidate. Evidence is retained outside Git in
  dev-verify-d1-review-8s4tsc8k/response.md.
- Verified evidence archive: dev-verify-d1-evidence.tar.gz,
  SHA-256 dc5ad118579be44f31f6e51ad50186c514799cc345457f3e3e08a604ee1d1957.
  The earlier inventory failures and diagnostic-helper failure remain preserved.
- Source-policy discovery now includes workspace verification tools. The three
  existing corpus-oracle literal sites were intentionally added to its inventory.
- One-time base/candidate setup differs because workspace manifests and lock
  ownership changed; it is excluded from the edit-cycle comparison. Cold CLI
  reconstruction after cleanup took 823.58s and is not ordinary timing evidence.
- Full-profile CI stopped before preparation because the performance-reference
  setting was absent. Owner: performance reference admission / CI configuration.
  It is outside this local correctness item; no full-profile CI pass is claimed.

Follow-up coverage in the phase edit matrix must include shared test-support edits
and included verification JSON changes, as well as the already required fixture,
addition/deletion, feature, toolchain and generated-input controls. The focused
D1 gain does not establish a full-gate runtime.

### D2 — preparation and execution boundaries (delivered)

The next integrated chunk makes prepared compiler ownership explicit and repairs
segmented Python failure discovery. Consumers must use the admitted product,
retain required independent runtime/process state, and reject stale, corrupt,
interrupted or mismatched preparation. Build-behavior tests retain real builds.

An ordinary Python suite failure under requested failure collection must preserve
its result and allow independent suites to execute. Cancellation, capacity
failures, abnormal termination, truncation and invalid execution evidence stop
collection. Failed or missing results never enter passing certification. All
selected suites must have an observable terminal or unexecuted outcome.

Validate these boundaries with focused positive/negative process tests, then
execute all Python suites and the paired generated-code contract with retained
targets. Measure actual producer invocations and full preparation/execution cost.
Do not waive final phase gates, qualification or the ordinary-gate target.

D2 merged in [PR #4514](https://github.com/sifr-lang/sifr/pull/4514), merge
748d312418f2ac07cd0b645c220200b664fc5e8a. Reviewed implementation candidate:
98dbc3490b91c976cea602d5cc9455e8963c04e6; base:
6916f48778442aa5825cfd1389465ea832fee453.

- Each profile publishes an immutable compiler copy, with a content-bound
  admission manifest and active owner/consumer leases. Each new invocation still
  enters Cargo. Explicit source-build contracts always enter Cargo.
- Ordinary Python failures continue only under requested collection. Abnormal
  execution, truncation and invalid evidence stop collection. Failed or missing
  results cannot certify success; every selected suite records an outcome.
- All 70 pinned affected checks, the complete runner foundation (79.245s), profile
  checks, file-size guard and documentation checks passed on the candidate.
- Complete selected merge pair: all 30 Python suites passed, all seven required
  compiled capabilities passed with nine evidence records, and all ten selected
  generated-code variants passed including strict fresh demo companions.
- Pair elapsed: 961.757s including 149.135s preparation. Python 809.539s and generated
  quality 403.789s overlap. No cap, source drift or surviving process. This is a
  scoped result, not full-profile or final timing qualification.
- The first Python runtime suite included a Cargo-reported 3m45s build of the
  shared test binary; its five assertions took 0.25s. The following suites reused
  it, with 15 and 21 assertions taking 0.51s and 0.47s. Do not interpret this cold
  build as runtime assertion cost or a retained ordinary-run baseline.
- Cold reconstruction before this run took 1030.716s: generated preparation 326.758s
  and authoring preparation 688.701s. Retained candidate preparation took 144.373s
  and 1.515s respectively. This cache-state comparison is not a D2 code speedup.
- The first publication prototype rehashed the entire compiler in every child,
  regressing four-consumer time 2.880s→7.284s. It was rejected. The final producer
  hashes during copying and binds the small manifest into the inherited token;
  consumers check published file identity and ownership. One actual Cargo build
  supplied four sequential plus four concurrent consumers with zero Cargo calls
  in those consumers. Consumer resolution plus identity execution was about 4ms;
  publication was 1.986s. No general percentage speedup is inferred from that
  small control because its first baseline build was slower than later builds.
- Independent review: **SATISFIED**, no blockers, for the exact candidate.
  Evidence: dev-verify-d2-review-s8x2lfi3/response.md.
- Verified 111-payload archive: dev-verify-d2-qualified.tar.gz, SHA-256
  74138b33dd22c9e1cfbbb19dd6185c12eb469e5abaccbca020fc9ffb96216d8c.
  Rejected initial experiment and controls remain separately preserved in
  dev-verify-d2-initial-evidence.tar.gz, SHA-256
  f4a97099894cea16b9213699575ba3b477d7449f9ecf14c41061f8dab28ff663.

Review follow-ups and ownership:

- Prepared development consumers receive their explicit source sysroot because
  a target directory may resolve outside the source tree. This is the prepared
  product's declared source context. Installed/discovery qualification continues
  to clear that setting. Preserve those dedicated discovery checks in the final
  matrix; broad incidental execution with the explicit context is not discovery
  evidence.
- Publication detects changes during copying, but exclusive ownership must also
  hold between Cargo completion and the start of copying. The README now names
  this boundary. Do not claim protection against another process replacing the
  Cargo product in that policy-prohibited interval.
- Later runner simplification should surface retained failed suite-part details
  in the reconciled report, beyond the current complete status journal.
- Full-profile CI still stops before preparation at missing
  SIFR_PERFORMANCE_REFERENCE. Owner: performance-reference/CI configuration.
  No CI or full-profile pass is claimed.

### D3 — ordinary Python verification costs (investigation)

Python was the longer branch of the complete selected pair. First establish its
retained ordinary cost now that the previously hidden native runtime build has
completed. Reconcile all 30 suites and the existing compiled capability matrix;
account for preparation and overlap with generated quality. Do not optimize from
cold-build time disguised as assertion time.

Then instrument a representative dominant operation and use the code and actual
child-process timings to select one coherent root cause. Inspect repeated
compiler/Python preparation, subprocess and frontend/probe work, native dependency
reuse, and the boundary needed by each assertion. Preserve isolated mutable
projects, environments and processes wherever the guarantee requires them.
Do not replace execution with static checks or silently change required feature
configurations.

Use the phase's shared 120s integration budget as a design constraint, not a
forecast. Compare the complete affected ordinary cycle before and after any
candidate; retain it only for a substantial demonstrated improvement with the
same obligations. Stop polishing subgroups already good enough and reassess the
critical path. Generated preparation is separately over its 90s category budget;
reducing a shorter overlapping branch alone cannot prove a faster pair.

Intermediate work retains focused regressions and independent review. Final
phase coverage, invalidation and repeated timing qualification remain required.

#### D3 retained baseline and prioritization

At the unchanged D2 candidate98dbc3490b91c976cea602d5cc9455e8963c04e6,
the complete selected pair passes in497.492s including preparation. Python
executes all30 suites in357.748s; generated quality executes all10 variants in
314.351s, overlapping Python. All7 compiled capabilities and their9 evidence
records reconcile. Generated preparation131.989s remains the dominant part of
136.467s declared preparation. This is a retained-state diagnostic, not a new
implementation speedup or complete480s gate acceptance.

The74-payload archive dev-verify-d3-retained-qualified.tar.gz is independently
SHA-verified locally:1e99aac2e3e150943d9a8252c5da2e54e65d72149a85abea448658444e01a983.
The native Python test graph now executes in0.572/0.873/0.865s across the three
runtime suites. It is already good enough for ordinary retained execution.

Targeted traces preserve actual assertions and are separate from ordinary timing.
Binding authoring runs in34.405s; CLI subprocess work dominates. Public Python
metadata ownership discovery scans roughly20,000 installed files per call in
this environment. Native example traces show about0.14s retained Cargo work,
about0.6s stdlib loading, and1.4–1.6s generation for several cases. A formatter
observer over all8 library examples records136 calls but34 distinct input hashes,
with5.821s inside formatter calls including observer overhead. Do not turn these
partial intervals into projected whole-gate savings. The first library trace
executes its examples but fails its external report-path presentation; preserve
that failed diagnostic. The corrected report-path observer passes. A separate
native process trace selects a different invocation environment and rebuilds a
native family; it is not retained timing evidence.

Do not expand small formatter or metadata optimizations before current macro
prioritization. The full emitted merge correctness-only plan is now undergoing
an ordinary, untraced integrated discovery run, with failure continuation,
1750s per top-level category and retained targets. Preparation reconstruction
remains charged. If a cap stops the runner, independently schedule the remaining
categories with their actual prerequisites and retain the original incomplete run.

### D4 — durable history publication representation (prospective)

The accepted phase explicitly permits redesigning storage while retaining all
4097 real durable publications. Reuse V1be's attribution as historical evidence,
then verify the current full history and project-cache costs. Source review
confirms generation directories still link every retained record and perform
separate durable writes for record, manifest, generation publication, cache
pointer and optional workspace hint. A hardlink-only change is insufficient.

Implement one immutable bundle per publication: a bounded manifest and the new
record in one file, with explicit references to immutable payload owners for
older retained records. Publish the complete bundle durably before atomically
updating the authoritative cache pointer. A workspace hint is advisory: publish
it atomically, but it cannot add a durability prerequisite beyond the already
durable cache commit. Do not pack/copy all prior payloads on each publication,
introduce a database dependency, weaken fsync on the authoritative commit, or
reuse correctness results across changed inputs.

Preserve the128-record/64MiB retained-history bound,16MiB record bound, ordered
ABA reuse, old-input eviction, current-byte hashing and semantic validation,
read-only/missing-workspace discovery, cancellation, disk-full recovery, stable
namespace/generation leases and active-reader survival. Cleanup must retain
payload owners referenced by every live generation, acquire exclusive ownership
before deletion, and preserve unknown/unsafe state. Use a distinct versioned
storage identity; do not add an old-format compatibility path or delete old
namespaces as migration. All current assertions remain; representation-specific
path/link checks must prove equivalent immutable sharing and reclamation.

Before adoption require: at least50% lower ordinary complete4097-publication
runtime, bounded near-limit write amplification, all affected cache/history/
corruption/lease/crash/recovery tests, explicit publication-boundary failure
controls, no leaked process ownership, independent exact-candidate review, and a
measured complete ordinary-driver contribution. Preserve rejected experiments.
An improvement in this item does not close the480s goal; final phase gates and
all existing coverage/invalidation/timing qualifications remain required.


#### D4 implementation checkpoint (unqualified)

The format now lives in `sifr_compiler_services::project_history`, shared by
writer and editor reader. This removes their duplicate manifest definitions and
prevents a writer-only schema change from silently disabling editor reuse.
Direct immutable payload owners are retained by the current history and every
live predecessor. New controls exercise near-limit payloads, bounded write
amplification, malformed envelopes, inherited-byte corruption, and legacy
namespace preservation. Existing current-byte and same-mtime controls mutate
the record inside its envelope, preserving the original guarantee.

A local diagnostic compiled the shared implementation and executed all 53
project-cache tests: 52 passed, including the full 4,097-publication test; one
unchanged lease test failed when executing `/bin/true`, absent on this host.
The failure occurred after its lease-release assertions. Preserve this result;
it is not a complete affected-test pass and remote lease qualification remains
required. The initial default temporary path traversed a symlink rejected by
cache security; a private canonical scratch path corrected the test setup.
No cache security check was relaxed. Local diagnostic elapsed time is not a
matched performance comparison with historical remote measurements.

The full baseline discovery at the unchanged D2 candidate stopped cleanly at
its preparation safety deadline: 1,775.205s overall, 1,750.317s preparation.
Workspace, fixtures, Python environment and generated preparation passed;
crate-test preparation was capped after 1,614.636s. Remaining preparation and
all main execution categories were unexecuted. Canonical preparation recovery
is running one node at a time with real prerequisites and separate 1,750s caps.
The original incomplete report is retained. No current full-gate runtime or
speedup is established by this recovery.


The separate editor-reader diagnostic executes three tests: inherited-payload
reuse/corruption and rejected-reader lease release pass; the unchanged
fork-before-exec control fails at the same absent `/bin/true` host utility after
its lock assertions. This pre-existing host-path issue belongs to the portable
lease test harness and is not repaired in D4. Source dependency-direction,
filesystem-effect inventory/self-tests, formatting, and file-size checks pass.
A first dependency-direction attempt lacked non-host registry metadata; fetching
the locked metadata repaired setup without changing the lockfile.


The consumer audit also includes cache CLI ownership inspection and both DX.13
and DXF CLI acceptance scripts. Disabled-reuse checks now hash actual project
bundles (with a nonempty assertion), rather than vacuously finding no old
`manifest.json` files. CLI pruning first proves referenced payload owners are
protected, then removes both discovery pointers and performs another real
frontend publication to make abandoned histories eligible. It preserves dry-run,
actual reclamation, idempotence, post-prune checking and orphan-namespace checks;
no cache metadata is fabricated. Count-bound eviction remains covered by the
existing complete history and live-reader tests.


D4 resource-bound correction: the first GC prototype held one descriptor for
all inactive generation candidates. Under a 64-descriptor process limit the real
reader/eviction regression failed to reclaim the evicted generation (3.642s),
confirming the defect. Replace that design with one short context-level reader
admission lease: new readers acquire it shared until their generation lease is
held; GC holds it exclusively through marking and reclamation. Existing readers
continue using their own leases. GC probes/rechecks entries one at a time. A
checked-in subprocess control reruns the complete existing eviction/active-reader
assertions at the low descriptor limit. Driver and editor both test admission.
The prior low-descriptor 4,097-publication experiment was cancelled after 63.433s
because its assertions did not require reclamation; its incomplete result is
retained and is not a pass. Normal full-history coverage remains required.

The local CLI diagnostic also exposed pre-existing stale DXF expectations for
the cache inspection root and owner inventory. The affected contract now verifies
the actual cache root, the complete owner namespace set, protected legacy owner,
v3 pruning owner, and matching text/JSON descriptions. This is part of qualifying
the changed cache CLI boundary, not a removal of empty-cache assertions.


The repaired admission design passes all five focused bundle controls locally
in 7.94s, including the exact low-descriptor eviction regression that failed
against the first prototype. Retained preparation plus execution is 30.171s;
19.62s is recompilation. This is correctness evidence, not the matched history
speedup required for adoption. Full affected Linux validation and ordinary
before/after timing remain pending.


At candidate `79feeea8ea4e61fa178d236bd2707b3ee1e23a87`, the real local cache
CLI contract passes all 36 isolated calls. The full local project-cache run
executes 55 tests: 53 pass, including all 4,097 publications; the known absent
`/bin/true` harness failure remains, and the empty-store contract detects that
admission authority was created during `Store::open`. Correct this by establishing
the permanent admission inode only under the first writer lease. Empty-store
opening and pruning remain read-only with respect to writer authority; incomplete
writer setup grants no reclamation authority. The original pressure/count test
passes after the correction (0.11s execution, 14.156s with recompilation).
The editor suite executes four controls: three pass, including admission and
inherited payloads; the same host-utility failure remains. None of these local
partial results substitutes for complete Linux qualification.

#### D4 measured rejection and prospective bounded byte proof

Candidate `c200fb10e39baa436da9fb78d2176feaf9d15a11` is draft PR #4515,
not adopted. On the same private SSD scratch and unchanged allocation, the
complete ordinary 4,097-publication test passes at 173.822s before and 114.317s
after: 34.23% lower, below the prospective 50% threshold. User CPU changes from
46.09s to47.70s while system CPU falls43.24s to22.30s. The unchanged ordinary
driver baseline passes794 cases with82 ignored native/corpus obligations in
705.444s. The candidate's subsequent ordinary run was cancelled at288.349s
because the implementation failed its adoption criterion; editor/CLI stages were
not executed. No partial result counts as complete qualification.

The first new baseline was cancelled at152.877s after detecting rotational
scratch, repeating a placement rejected in V0.6a1. Its evidence is preserved.
The accepted comparison above explicitly uses SSD scratch without changing any
fsync operation. The complete13-node preparation recovery passes independently;
98 payloads are locally SHA-verified in dev-verify-d3-preparation-recovery.tar.gz
(6ce38d5f5bb99a90e2507246b69ea71560135d61df6307bd0a7ea0d7e66a5312).
Its5287.327s sum includes separately repeated prerequisites after cleanup; it is
not ordinary latency, a test-execution pass, or a contiguous full gate.

A targeted129-publication control passes under CPU sampling. The sample places
36 of69 user-CPU samples in SHA-256 compression. This is an attribution lead,
not a precise whole-test percentage. The outer diagnostic wrapper fails because
its same-user session guard sees the privileged observer; preserve that failure
and do not weaken the generic guard or count it as qualified execution.

Before further implementation, extend D4's proof representation while preserving
its acceptance threshold and semantic guarantees. Retain at most2MiB of already
validated record bytes per existing Store-owned validation cache, in addition to
the existing count bound and digest facts. Every lookup still reads current owned
file bytes. Exact equality with the retained validated bytes proves the prior
stored-identity/context validation still applies; unretained or changed bytes
use the existing digest/semantic validation path. This replaces hashing on every
small-record cache hit with a stronger exact-byte comparison, not metadata-only
freshness or reuse of a cached compiler result. Oversized records retain the
existing bounded digest facts. Do not change record hashes, serialization, public
results, observations, filesystem durability, or the history-test workload.

Add focused controls for the byte budget, byte-proof eviction with retained
digest facts, oversized/zero-budget records, changed bytes and incorrect stored
identities. Retain every existing same-mtime, corruption, context, recency and
owner-lifetime control. Repeat complete matched history timing and complete
ordinary-driver/editor/CLI qualification on the revised candidate before review.
The original prototype remains a rejected performance experiment.

#### D4 opened-payload consolidation (under qualification)

Candidate `138a714aa403075951e780e5ecc747550adfe91f` passes the unchanged
4,097-publication test in 105.862s (39.10% below baseline), so adoption remains
rejected. User CPU falls to 31.67s; system CPU is 28.47s. All nine local validator
controls pass. The dependent ordinary-driver/editor/CLI sequence did not start.
A corrected independently bounded observer passes its session guard: the real
129-publication control passes, with 323 user samples and no lost samples;
SHA-256 accounts for 19.20%. A separate syscall diagnostic passes and records
79,306 metadata queries and 20,042 opens. Instrumented durations are not timing
qualification. These observations justify consolidating repeated payload reads.

The next candidate checks the actual opened Unix file's ownership, permissions
and regular-file type, avoiding the redundant pathname metadata check before
opening it. The parent directory remains checked on every read; final symlinks
are forbidden. Nonblocking open permits rejecting FIFOs without hanging. Windows
retains its existing ACL/reparse checks. The history reader reads the bounded
header and expected payload together, then verifies the header before returning
bytes. No durability operation, retained-history bound, record hash, publication
count, lease or corruption assertion is removed. Three direct opened-object
controls and all six bundle controls pass locally. Repeat the unchanged complete
history comparison before accepting this candidate; the 50% threshold remains.
