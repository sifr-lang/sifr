# Ad Hoc Development and Verification Boundary Redesign

- Phase ID: `DEV-VERIFY`
- Status: in progress; intermediate deliveries through D22; final phase qualification open
- Latest delivery: D22 frontend assertion ownership; D23 integrated cost discovery in progress
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

### D1 — dependency contract ownership (delivered)

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

### D4 — durable history publication representation

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

#### D4 single immutable commit authority (prospective)

Candidate `ec70ed8f93683711086e5590ac60aeeda2f04d64` passes in101.492s
(41.61% below the173.822s baseline), still below adoption. User/system CPU are
31.01s/25.15s. Do not repeat it unchanged. The syscall control establishes four
fsyncs per publication: durable bundle and directory, then durable pointer and
directory. The next design removes the separate mutable cache pointer, rather
than relaxing either committed data durability or fresh-byte validation.

Use a writer-serialized monotonic sequence plus manifest digest as the committed
bundle name. The shared driver/editor reader selects the highest committed name;
its sequence, digest, context and complete current payloads must validate. A
corrupt highest commit is a miss, never implicit reuse of a lower generation.
Staging/prepared names are outside the committed index. Flush the complete staged
bundle, validate it, then atomically rename and synchronize the commit directory.
That rename is the visibility point; successful directory synchronization is the
durability boundary. Keep the workspace hint advisory. Old committed bundles
remain independently discoverable after an interrupted publication. No database,
mutable append log, metadata freshness shortcut, or compatibility reader is added.

Preserve the existing process-death controls at precommit preparation boundaries,
and add death after durable commit, sequence/digest mismatch, overflow, ambiguous
head and malformed-name controls. GC must retain an invalid highest commit and
all ambiguous references, preserve live readers, and never let pruning roll back
the valid head. Preserve every existing 4,097 publication, current-byte, bounds,
read-only, reader-admission, low-descriptor and CLI assertion, adapting only
representation-specific operations. Enumerating commit names adds work
proportional to unpruned directory entries; measure this explicitly with4,097
unpruned names and bound memory to one selected head. Do not claim constant-time
head discovery. The existing >=50% complete-history adoption criterion, full
ordinary-driver contribution, affected qualification and independent review remain.

The local integrated cache sweep executes61 tests:58 pass, two inject publication
failure into the now-obsolete pointer parent, and the known missing `/bin/true`
process helper remains unavailable. Move both permission failures to the actual
`generations` publication directory; their original diagnostic/status assertions
then pass. The three commit-index controls pass, including4,097 unpruned names.
The first new commit controls cover successful durable publication before hint
creation and rejection/preservation of a corrupt or sequence-mismatched head.
The latest broad local diagnostic is not a full pass.

The new protocol also makes its Unix initialization dependencies explicit:
synchronize newly created directory entries, the immutable namespace owner, and
the stable namespace/context/generation lease directory entries before a commit
can be returned. Empty ownership inodes must survive restart alongside the
payload. Do not remove those barriers to win the timing comparison. Precommit
process-death hooks now name stage validation and commit readiness; they still
assert that interrupted private work is undiscoverable and safely reclaimable.
The removed mutable-pointer failure injection is replaced with corruption of the
actual committed head in the real CLI recovery contract.


#### D4 closure — 2026-10-28

Delivered in [PR #4515](https://github.com/sifr-lang/sifr/pull/4515).
Exact qualified/reviewed implementation:
`7233ef0e3232e0dc4cfe6b33d57f2d76ffb179c8`.
Merge: `4c7e34ae785c074cdfb07dccd58a55bc097df350`.
The prospective contracts and rejected experiments above remain historical;
the final implementation uses the later single immutable commit authority.

| Complete retained-target workload | Before | Final candidate | Result |
| --- | ---: | ---: | --- |
| All 4,097 real durable publications | 173.822s | 52.731s | Pass; 69.66% lower |
| Ordinary driver suite | 705.444s | 540.029s | Pass; 23.45% lower |

The history test also executes inside the ordinary suite; these times must not
be added. Final candidate test preparation took12.209s separately, including
recompilation, and is not a claimed speedup over unchanged baseline preparation.
The earlier implementation-only candidate's529.518s ordinary result is preserved;
the final candidate's540.029s result governs closure.

All805 ordinary driver tests pass, versus794 before. Compiled inventories prove
zero removed IDs and11 added regressions. The82 ignored entries are unchanged;
they include separately enrolled native assertions and subprocess helpers run by
parent tests. This ordinary-suite summary does not claim82 direct executions or
complete gate coverage. All7 editor tests,18 cache-storage tests,3 commit-index
tests,36 real cache CLI calls and31 independent-process DX13 CLI invocations pass.
All command/session guards report clean source and no remaining owned processes.

The initial CLI/cache/index qualification failed under ambient umask0002 because
new test fixtures inherited group-write permission. The production ownership
checks correctly rejected them. The final repair creates private fixture roots
and payloads and a private CLI workspace, preserving strict rejection and every
assertion. The initial failures remain failed evidence. Final pinned formatting,
file-size (4,561 files), dependency direction, method authority (6,458 sites/235
nodes), filesystem effects (3,734 sites plus self-tests), and diff checks pass.
All139 changed-document local links outside code examples resolve.

Independent read-only Claude Opus review: **SATISFIED; no blocking findings**.
It inspected source, raw qualification, durability, reader/GC ownership, corruption,
byte-proof invalidation, bounds and preserved assertions. Evidence remains outside
the reviewed Git tree, keyed to the exact implementation candidate:

- `dev-verify-d4-final-evidence.tar.gz`, SHA-256
  `b98ea325ad9ed7a135fc1e2c22765931f0ffc7671bbb087f9fd7f8ab0ffd204e`;
  archive hash and all398 payload hashes independently verified after transfer.
- `dev-verify-d4-final-summary.json` records exact binaries, commands and results.
- `dev-verify-d4-review-bwk05s9m/response.md` records the completed review.
- The earlier173-payload diagnostic archive remains immutable and retains all
  rejected, failed and cancelled experiments.

Review follow-ups, outside D4 acceptance: evaluate now-unread advisory workspace
hint writes; explicit diagnosis/recovery for externally fabricated ambiguous
sequence heads; parent read-permission requirements for durable Unix directory
creation; permanent generation-lock growth and pre-existing orphan `lock_tree`
descriptor retention. Preserve lease authority and durability in any later repair.

CI is not claimed passing. The create-PR job stops before setup because
`SIFR_PERFORMANCE_REFERENCE` is absent. The deterministic-report job reports a
missing first-run signature, also observed at baseline98dbc (job122841251014) and
candidate7233ef (job123010834726). Its child log is suppressed and not uploaded;
the underlying CI cause remains unclassified, and the same symptom alone does
not prove an identical cause. Owner: local-first-validation/E2E determinism
wrapper. No CI policy or workflow was changed to merge this locally qualified
intermediate item under the approved phase policy.

#### D3 integrated discovery continuation (prospective)

Next run the complete canonical `merge --correctness-only --no-fail-fast` sweep
on retained targets, using the same owned SSD scratch location as D4. A fresh
emitted plan selects732 E2E fixtures,41 crate-suite entries and20 verification
areas; it excludes only the representative benchmark capture. Include all
preparation in the sweep's elapsed time. Keep the1,750-second diagnostic safety
cap per category and reserve checks; preserve every failed/incomplete outcome.
Schedule independent categories blocked by a cap or failed prerequisite explicitly.
Use the resulting complete cost/failure picture to choose the next macro repair.
Do not treat separate preparation recovery or D4 acceptance as a full-gate pass.
Complete phase correctness, coverage, invalidation and repeated timing qualification
remain required; the480-second complete ordinary gate is still unproven.

The first integrated continuation at25a9d8361 stopped after22.694s before
preparation because DX.5 consumer documentation retained two stale line anchors
in `project_cache/mod.rs`. Review of the generated diff confirms only line
numbers changed; symbols, operations, classifications and payload fields are
unchanged. Refresh those two documentation rows and run the inventory checker
and its13 self-tests before retrying. The failed sweep, lane report and all
blocked categories remain preserved; no main test execution is claimed.

#### D3 integrated inventory reconciliation (delivered)

The second sweep remains frozen at `7285e1fb34cef95eba226799fc175e71fc0b5078`.
Its all-category failure continuation exposed integration omissions from D1/D4:
the coverage matrix still classifies the 17 relocated dependency targets under
their former production crate; four new audit identifiers violate the existing
naming policy; and two FIFO-test unsafe sites lack local inventory records.
The last omission also stops the architecture-policy self-tests after 35 tests.
These are actual failed checks, not timing failures or cancelled execution.

Prepare one bounded reconciliation while the unchanged remote sweep continues:
move those exact 17 classification records to the verification package, rename
only the rejected local audit identifiers, and author the FIFO allowance/call
records with source-bound thread, lifetime, alias and ownership obligations.
Preserve every assertion, target classification and profile assignment. Do not
change the scanner, weaken the naming rule, or grant a broad unsafe allowance.
The two FIFO records belong to the existing cache-owned external partition.

Before merging this follow-up, run the current strict coverage readiness check,
its negative self-tests, profile-assignment matrix, taxonomy and its self-tests,
whole-tree unsafe/ABI checker and complete architecture-policy self-tests without
fail-fast, and all relocated dependency/production manifest tests. Require pinned
tools, formatting, file-size, diff/local-link checks and independent exact-candidate
review. The previous failing sweep remains preserved. Reconcile every unexecuted
test after diagnostic limits; no earlier failure becomes a pass by changing records.
This repair makes no performance claim and does not close the full phase gate.

The first focused qualification at78d11 passes strict coverage, profile assignment,
the whole-tree taxonomy and unsafe/ABI checks, and all56 architecture-policy
self-tests. The separately invoked taxonomy unit suite exposes a stale helper
name left by a85b56bb2; update that call to the existing renamed helper, preserving
all positive and negative assertions. Dependency audits also require the pinned
HTTP/application demo submodules, absent in the local checkout. Initialize those
declared inputs before repeating qualification. Preserve the failed144.280s run;
it is not a passing candidate. This remains a bounded verification reconciliation.

The unchanged integrated sweep separately found SQL component provenance drift:
PostgreSQL, MySQL and SQLite differ only in rootCargo.toml andCargo.lock after D1;
words-component provenance is unchanged. These files also feed embedded component
identities, so updating producer receipts alone would be invalid. Owner: this
phase's production/verification identity boundary. Keep real artifact rebuilding
and necessary invalidation until that boundary is repaired and qualified. This
is separate from the inventory candidate; the complete phase gate stays open.

#### Inventory reconciliation closure — 2026-10-28

Merged [PR #4516](https://github.com/sifr-lang/sifr/pull/4516) with normal merge,
without an administrative override. Exact qualified/reviewed candidate:
`371e10b7a40a852ea676ae7420c0f88ffd31ac5f`; merge:
`e03da58ceef199196cc5380849a08d3939416a5f`.

Pinned focused qualification passed in137.169s: all108 Rust dependency/manifest
assertions,10 taxonomy unit tests,56 architecture-policy tests without fail-fast,
strict coverage readiness and negative controls, profile assignment, the complete
410-site unsafe/ABI inventory, formatting, file-size and diff checks. The metadata
consumer inventory and its13 tests also pass. An exact record comparison confirms
all17 moved target records retain order/classification/profile and every other
package record is unchanged. The audit rename affects three identifiers and one
message; it changes no assertion. All107 local documentation destinations resolve;
the final record addition introduced no new links.

Independent read-only Claude Opus review: **SATISFIED; no blocking findings**.
Evidence: `dev-verify-inventory-review-2pt06der/response.md`, outside the reviewed
tree. The archive `dev-verify-inventory-evidence.tar.gz` has SHA-256
`9b5f67944f4899f198338820426942cce8d94f1051b8221a50867f6b94eaf7d7`;
all82 archived payload hashes were verified. The initial144.280s failed run is
preserved. CI was still running at merge; this record does not claim CI success.

The complete remote diagnostic remains frozen at7285 while these separately
qualified repairs are delivered. Its SQL category finished in956.726s, including
the passing511.420s clean/reused/independent-clean linked build protocol. Component
provenance failures remain open. Finish the diagnostic, account for every missing
execution, and use controlled same-binary/context probes to choose the next macro
improvement. Full phase correctness, coverage, invalidation and repeated480-second
qualification remain outstanding.

### D5 — isolated ordinary driver tests and bounded concurrency (investigation)

The frozen integrated diagnostic executes ordinary driver tests with
`RUST_TEST_THREADS=1`. Its independent metadata producer/recovery assertions
perform real production repeatedly, so serial execution is a material cost.
This observation is about the recorded diagnostic configuration, not proof that
every normal developer invocation is serial. The local 30-test stateless stdlib
codegen diagnostic took10.430s; optimizing that smaller group is deprioritized.

Before comparing complete ordinary driver execution at one versus four workers,
remove the one process-wide cwd mutation in the workspace relative-path test.
Its private mutex does not protect unrelated tests. Run the exact existing
relative-path assertion in a child with an explicit cwd; require a successful,
nonempty libtest result and verify the parent's cwd remains unchanged. Keep the
test's discovery identity and all assertions. No global compiler/context cache
or source/input identity relaxation is part of this item.

Then measure the same candidate's complete ordinary driver selection with
retained products. Bound nested Cargo and Rayon work along with outer libtest
workers, retain reserve monitoring and the diagnostic cap, and record actual
elapsed time, CPU, peak process-tree memory, all passed/failed/ignored identities,
and preparation separately. Inspect failure modes rather than retrying unchanged
failures. Adopt a default scheduling change only after all assertions pass and
complete-category elapsed time improves by at least15% without unsafe resource
oversubscription. Compare emitted gate selections and affected runner self-tests
if policy changes; otherwise retain this as a measured experiment. Independent
exact-candidate review and focused workspace/driver regressions remain required.

The remote integrated source stays frozen until its evidence is archived. This
local preparation does not alter that run. Full correctness, coverage, invalidation
and480-second phase qualification remain outstanding; SQL's511.420s independent
clean build protocol is still a separate critical-path problem.

The local focused workspace check passes all three selected discovery tests with
four libtest threads, including the subprocess relative-path assertion. Cargo
preparation took16.73s; total guarded invocation19.492s. Formatting, file-size
guardrails (4,561 files) and diff checks pass. No concurrency policy is adopted
yet; whole-category comparison is still required.

#### Integrated discovery and D5 diagnostic checkpoint

The frozen7285 full sweep ended after5253.364s (87m33s), failed/incomplete.
All17 preparation nodes passed. Ordinary driver execution passed805 tests with
82 ignored in566.133s; CLI/lowering passed in609.452s. The combined crate category
hit its1750-second diagnostic cap during generated-driver execution, so its
remaining cases are incomplete and the final native corpus was blocked. Neither
is an assertion-failure result. Recover the complete generated-driver category
and native corpus separately with their own unchanged caps before source sync.

The complete original archive has6663 payloads and SHA-256
`acf463f3e86d7187e848c425f2116b972cf1ffeca605ef6522264c42a68a96d1`.
Its creation completed and the execution session was empty with unchanged
tracked sources. Connectivity failed during transfer; the partial local archive
is explicitly labeled incomplete and has not passed payload verification.

D5 candidate `a360b871fed11857732dcf583819c47bcf608ad8` completed a local
four-worker diagnostic in230.43s:797 passed,7 failed,82 ignored. Nested Cargo
and Rayon each had one worker. Peak sampled process-tree RSS was2.068GiB;
no reserve violation or remaining child was observed. This is not passing
qualification or a paired performance comparison. Four failures are existing
`/bin/true` assumptions in native capture/process-lease tests; one is the native
snapshot socket fixture exceeding the platform path limit under the diagnostic
TMPDIR. Owners: those existing test fixtures; no portability repair is included
in D5. The other two failures are missing local preparation: the attached-API
fixture's locked `autocfg1.5.1` package and the pinned Python environment.
Owner: diagnostic environment preparation. Preserve all failures; do not silently
skip them or substitute this run for the prepared-environment qualification.

Next restore connectivity, verify the full archive locally, finish the controlled
same-binary CLI probe and capped missing-category recovery at7285, then synchronize
D5 and run its same-candidate serial/concurrent comparison. Scheduling policy,
SQL provenance, final gates and480-second timing acceptance remain open.


#### Recovery resumed and D5 scheduling prototype

Connectivity is restored. The original discovery archive is now fully downloaded:
all6,663 payload hashes pass local verification. The incomplete transfer remains
separately preserved; the completed diagnostic remains failed/incomplete.

The first recovery helper failed before tests because its external script lacked
an import path. The next attempt was cancelled after424.416s with a clean owned
session: directly invoking the pinned Python interpreter bypassed the canonical
`uv run --project verification --locked` entry point. A controlled environment
comparison found a different PATH, which participates in native toolchain identity;
the helper selected a new native dependency target. Preserve this diagnostic
setup failure, its environment comparison and partial build evidence. Restart
through the canonical entry point without weakening cache identity. This is not
evidence that the normal gate loses its retained artifacts.

The unqualified D5 prototype bounds only the existing `sifr_driver_lib`
configuration. It samples current admission capacity, shares slots between
libtest workers and nested Cargo/Rayon work, preserves smaller explicit requests,
and stops the owned command if its memory reserve is lost. Other crate selections,
preparation allocations and generated-test scheduling are unchanged. Default
adoption remains conditional on the passing same-candidate comparison above.

Local allocation/selection controls pass23 tests. The first combined local
control attempt failed four process-cancellation checks because the OS rejected
process-group signals; three are existing progress checks and one is the new
reserve-monitor integration control. Preserve that failure and qualify these
checks in the prepared execution environment. No passing cancellation claim is
made. Formatting, file-size guardrails (4,563 files) and diff checks pass.

#### Missing-category recovery and D5 complete comparison

The frozen7285 recovery is complete. Generated-driver execution passed all81
selected ignored tests in690.963s, with53.799s separate advanced preparation
(747.674s guarded total). Native E2E passed all732 fixtures across186 native groups
in574.829s (575.591s guarded):41.108s frontend,0.046s planning,519.492s native
builds and12.205s execution. It had0/186 native cache hits and includes artifact
reconstruction; this is coverage recovery, not warm timing acceptance. Previously
interrupted trust-revocation and trusted-build-script cases now pass. The original
integrated run remains failed/incomplete; separate recovery cannot relabel it.

All94 recovery archive payloads were verified locally. Archive SHA-256:
`d6e3b15703a2b6643f9397fe44e6d09040cb244fafe836e597453f2734ee14fe`.
The source/installed-CLI diagnostic also passes18 invocations with exact matching
outputs. Its small per-invocation savings are not a whole-gate forecast.

D5 runtime qualification observed the exact candidate
`f008ad7d3ddac04e3bc6d4b94c54e33be2f3d605` with unchanged tracked sources,
canonical pinned uv entry, retained targets and four admitted resource slots.
Both complete ordinary-driver runs passed805 tests,0 failed,82 ignored,0 filtered.
Those82 ignored entries remain separately owned; D5 changes no test selection.

| Measurement | Serial diagnostic allocation | Bounded parallel allocation |
|---|---:|---:|
| libtest / nested Cargo / Rayon workers | 1 / 4 / 4 | 4 / 1 / 1 |
| Complete category elapsed | 498.296s | 161.236s |
| Guarded invocation elapsed | 499.318s | 161.838s |
| Inclusive category CPU | 447.392s | 515.356s |
| Peak sampled session RSS | 1,486,778,368 bytes | 1,883,725,824 bytes |
| Minimum available memory | 9,690,161,152 bytes | 9,182,781,440 bytes |

Elapsed time improved67.64%, exceeding the prospective15% criterion. CPU grew
15.19%; this is a bounded concurrency improvement, not elimination of equivalent
CPU work. Sampling sums session RSS and may double-count shared pages. Both
sessions ended empty with no reserve violation. Separate canonical preparation
took38.599s, including three passing workspace discovery assertions. Forty
allocation, selection, build-budget, progress/cancellation and native-allocation
self-tests pass, including actual owned-process teardown. Pinned profile checks,
formatting and file-size checks also pass. This qualifies the category relative
to the recorded explicit serial configuration; the old unset libtest default
could already parallelize, so this is not a measured gain for every prior default.

The whole-tree filesystem-effect guard found one missing ownership record for
the subprocess test's new canonicalize call. The correction classifies that exact
call as tooling-input; all3,735 sites then pass. Coverage readiness, profile
assignment and taxonomy checks pass. Preserve the initial failed guard and the
earlier local cancellation failures rather than replacing their evidence.

The final inventory-and-prose update uses the prospective input-binding protocol
recorded before updating the measured candidate. Runtime evidence remains labeled
f008ad7d. Reuse requires unchanged tracked/submodule inputs except the exact new
inventory row and this phase prose, identical emitted plan and discovered tests,
and a byte-identical driver executable after canonical Cargo preparation. Rerun
the affected ownership guard on the final candidate. A failed binding requires
affected runtime qualification again. The measured executable SHA-256 is
`033c223b9f6ab8cb1d60a0d6e6553dc21704ce2838307a7c15f1e39963761b96`.
Independent review of the final candidate remains required before adopting D5.

Full phase correctness, coverage, invalidation and repeated480-second timing
qualification remain open. The SQL511.420s independent-clean protocol and native
build allocation remain macro priorities; do not sum gains across different
candidates or call these separate category runs an integrated gate pass.

#### D5 closure

Merged [PR #4517](https://github.com/sifr-lang/sifr/pull/4517). Qualified and reviewed
final candidate `3a9fd8388c622b22ce35fb3047c21b55efa6066a`; normal merge
`7ae4903dce3531c547c5dbb7bb0e847c6692b04a`. Final input binding passed after canonical
Cargo preparation, which remained fresh. All3,735 filesystem ownership sites pass.
The first binding helper failed before tests because its external import path
omitted the repository; the corrected invocation passes and both are retained.

Independent read-only Claude Opus review: **SATISFIED, no blocking findings**,
`dev-verify-d5-review-76_kg5dr/response.md`. Evidence archive
`dev-verify-d5-qualified-evidence.tar.gz` has SHA-256
`c0a28fc9503db26c27f4bd6092c57a92c4ac21c338869ffff5bc282992c442c7`;
all52 payloads verified after transfer. All12 local documentation destinations
resolve. CI is not claimed passing. Follow-ups: ad hoc execution without prior
preparation compiles the driver with the nested job allocation; document that
preparation assumption if exposing such an entry point. The child test's libtest
summary check fails closed if Rust changes its human-readable output format.

### D6 — explicit production inputs and private test ownership (prospective)

The current local-token producer hashes the complete package, and the executable
aggregate hashes every file under crates. Consequently private assertion edits
can change production identities and invalidate downstream products even when
Rust excludes those assertions from production compilation. D1 removed one
verification-only crate from that namespace; it did not repair this general
ownership boundary.

Establish positive production input roots for first-party packages using their
manifests as the ownership authority. Include manifests, build scripts, generated
source inputs and declared embedded assets. Keep the existing conservative
toolchain/configuration/dependency identities. Do not derive identity from a
previous build's dep-info, a Git-only file list, or a blanket test-name exclusion.
Missing or invalid ownership must fail closed. Source archives must work.

Move private test modules outside production roots while preserving Rust module
privacy, discovery IDs, assertions, fixtures and snapshot order. Begin with the
driver and identity ownership boundary, then expand only through verified module
ownership. Keep distinct build configurations and required native semantics.
Do not introduce many integration crates or replace native execution with mocks.
Reconcile the affected policy inventories without changing classifications.

Acceptance requires actual Cargo builds and consumer execution, not just stable
digests: assertion/shared-test-support and fixture edits rebuild the correct test
products without changing production identity; production implementation/assets,
input addition/deletion, features and toolchains still invalidate the required
products. Compare discovered test identities, all affected focused controls and
complete driver coverage. Check observed rustc/build-script input paths against
the declared roots as an independent omission check. Retain cold preparation
separately from representative edit-cycle timing. Require a material complete
cycle improvement before adoption, plus exact-candidate independent review.

This item does not yet remove root manifest/lock invalidation or qualify the
SQL provenance repair. Do not update existing component producer receipts without
rebuilding their artifacts. Full phase gates, complete coverage/invalidation and
repeated480-second qualification remain open.

#### D6 implementation checkpoint

Baseline at3a9fd838: canonical production CLI build plus the same focused parser
test took1.315s unchanged,16.476s after an assertion-only edit,13.548s after restoring
it. The production binary hash changed for the assertion edit and returned exactly
on restoration. Separate initial preparation took22.126s. These are narrow edit
cycles, not full gate timings; raw results remain `dev-verify-d6-baseline-*`.

The prototype declares positive package-relative production roots in first-party
Cargo manifests, plus explicit workspace-relative roots for parser/vendor inputs
and generated/embedded assets. Both local and aggregate identities use this
authority; build-script extra-input lists move into the same manifest ownership.
The aggregate enumerates explicit production workspace members instead of watching
the entire crates directory. Global Cargo/lock/toolchain inputs remain conservative.
Identity framing advances to v2; no old-identity fallback is introduced.

All56 files in the driver's central private test module tree are relocated
byte-for-byte to tests/unit at the same depth. Identity tests are likewise outside
src. Existing module names/privacy remain intact. Other crates' inline/private
test content remains conservatively owned until separately migrated; this is not
a claim that every assertion in the workspace is already separated.

Independent include inspection found standard-library sources embedded by
sifr_stdlib_manifest outside its package directory. Their workspace root is now
explicitly declared. Declared embedded Markdown must be hashed like any other
asset; the prior extension exclusion is removed. Source-archive Cargo controls
prove unchanged/private-test freshness, feature separation, asset addition/removal
and external embedded-asset invalidation with actual consumer execution. Nine
identity controls are being qualified; complete-driver comparison and independent
review remain pending. The metadata consumer audit now includes private tests/unit
sources and has a regression preventing their silent disappearance after relocation.

#### D6 namespace correction and interrupted preparation — 2026-10-29

The initial candidate1f6af6c6 is not qualified. Its first preparation was interrupted
by an execution-host reboot after the production build, while driver tests were
still compiling. On return the checkout was clean and retained targets/caches were
present. Preserve the incomplete logs; no terminal passing receipt exists. This
also separates the earlier baseline from the new boot for timing interpretation.

The resumed attempt was deliberately cancelled after116.027s, with an empty owned
session, after code inspection found a namespace conflict: application discovery
already owns package.metadata.sifr and rejects unknown keys. Compiler build-input
ownership now uses the separate package.metadata.sifr-build namespace. A focused
package regression proves compiler metadata does not claim application discovery,
can coexist with valid application metadata, and cannot weaken rejection of invalid
application fields. Its source lives outside production roots under tests/unit.
Nine identity controls pass again after the namespace correction.

Before adoption, run the revised candidate's actual edit cycle and full affected
coverage. In addition to the historical baseline, use a same-boot causal control:
temporarily declare the private unit directory as a production input, reproduce
the assertion-triggered production rebuild, then restore exact manifest/test bytes
and the original production binary. This isolates the ownership effect without
attributing host-reboot effects to the implementation. It is a counterfactual
ownership variant, not a claim to rerun the historical implementation. The complete
observed-input audit and independent review remain required.

#### D6 retained edit-cycle and coverage checkpoint

Runtime candidate `f6c7255f97963af0444fb544807b94fd7daea686` passed the same-boot
ownership intervention. With private tests separated, the assertion edit plus
production build and focused test took5.065s: no production compilation, unchanged
production binary, and an actual driver test rebuild. Temporarily including
`tests/unit` in the driver's declared production inputs reproduced the unwanted
rebuild and took22.676s for the same edit cycle (77.66% lower when separated).
The intervention is a controlled variant of this candidate, not the historical
implementation. Exact original manifest/assertion bytes and production binary were
restored. Initial candidate preparation357.983s remains separate from these timings.

The independent observed-input audit covers3,347 source-input entries and451
absolute build-script watches for the actual native CLI configuration, with zero
undeclared inputs. It does not claim coverage of unbuilt feature/platform graphs.
All31 prior build-script extra-input sets are retained; all56 relocated driver test
files remain byte-identical to the base. Driver discovery matches the D5 inventory
exactly. Complete ordinary driver execution passes805 tests,0 failures,82 ignored,
in215.700s category time (217.043s outer guard). The ignored preparation helper
passes separately in68.510s; the81 generated assertion cases remain under execution.
These results do not constitute the final phase gate or a warm-category speedup.

Source-archive controls additionally execute real consumers across the pinned
Rust1.98.1 and the installed1.99.0-nightly toolchain: source identity remains stable,
configured identity changes, and restoration recovers the original identities.
An actual production-code edit and a build-script-generated Rust dependency both
invalidate and execute their changed behavior. The first auxiliary fixture omitted
optional watched roots and failed a freshness assertion; the corrected fixture's
attempt with1.98.0 was correctly rejected by the minimum Rust version. Both failed
attempts remain preserved beside the successful controls. These isolated experiments
do not change the project's pinned qualification toolchain.

No compiler, test, fixture or build inputs change after this runtime candidate.
The final correction retains `package.metadata.sifr` for the application package
example, distinct from compiler build ownership `package.metadata.sifr-build`.
A final policy check exposed two stale method-inventory fingerprints after the
namespace correction. Only those fingerprints are refreshed: the reviewed lexical
operations, classifications, owners and obligations remain identical. The method
inventory check now passes6,458 sites and235 reviewed nodes; the original failure
is preserved. Final format, file size, unsafe, filesystem and metadata checks also
pass (4,567 files,410 unsafe sites and3,742 filesystem sites respectively).

Bind final review to the final commit and retain the actual runtime candidate.
Before reusing runtime evidence, compare all tracked/submodule inputs, allowing
only this prose, the architecture prose and the two reviewed inventory fingerprint
fields; require byte-identical driver executable and unchanged discovery after
canonical Cargo preparation at the final commit. Complete generated execution, shared-helper/fixture edit controls,
independent review and final phase qualification remain pending.


#### D6 closure — 2026-10-29

Merged [PR #4518](https://github.com/sifr-lang/sifr/pull/4518), final candidate
`d389315721d73d723d4d00989f7ff35d65246084`, merge
`4e90fb72c2cf5b312944a0c296514d5dee3fb6cb`. Independent read-only Claude review:
**SATISFIED, no blocking findings**, `dev-verify-d6-review-e_28luh7/response.md`.
The complete driver coverage and ownership checks are qualified; the full phase
remains open.

All81 generated assertion cases pass in439.735s outer-guard time, with their
preparation helper passing separately. Shared-helper and embedded-fixture edits
rebuild and execute the affected test while preserving production bytes, in14.685s
and5.788s respectively; originals are restored and re-executed. Final binding
passes after canonical Cargo preparation: all tracked/submodule inputs are unchanged
except the documented prose and two inventory fingerprints; the driver binary is
byte-identical (`4c289ad236e862e4c5686a146f66a53cdd290d2a4eb3608e5cb5672c37357a15`),
as are discovery and emitted correctness plan. Runtime remains labeledf6c7255f;
no false final-SHA execution claim is made. The final method inventory passes.

`dev-verify-d6-qualified-evidence.tar.gz` has SHA-256
`ad258e0b9c7e587e3b45a7212b954c5f62c7e1250800d0d477fb6fc591138b48`;
all129 payloads verified after transfer. Earlier failed, cancelled and interrupted
attempts remain included. Supplemental local source-archive controls and the final
review are retained alongside it. CI is not claimed passing.

Review follow-ups: make the external dep-info auditor recursively consume package
ownership, and consider guarding unregistered build-script reads. Manual source
inspection found no current omissions. The unconsumed tracked
`crates/sifr/tmp_hashlib_api_subset.txt` remains with its original owner. Other
private tests under src, root-manifest/lock invalidation, SQL component provenance,
other configurations and full phase qualification remain open.

### D7 — native corpus execution ownership and scheduling (prospective)

Establish the current complete732-fixture native-corpus cost before adopting more
architecture. Its latest recovery574.829s included0/186 cache hits and an actual
native dependency rebuild, so it is not a representative retained baseline. The
current harness serializes native groups and invokes Cargo separately for each,
with one Cargo job despite a four-slot verification allocation.

First prepare the exact current E2E test configuration and account separately for
any reconstruction. Use the retained owned native cache. Measure normal complete
runs, every fixture and native assertion intact, with the existing one-Cargo-job
setting and then the admitted Cargo budget if the baseline exceeds the90s native
corpus design budget. Keep all other inputs/profiles identical, report cache hits,
frontend/native/runtime costs and resource reserves, and do not call reconstruction
an optimization baseline. If the current retained category already meets its
budget, record that and move to the next larger cost; no change is required merely
to consume the investigation budget.

If repeated per-group process/root-unit scheduling remains material, investigate
one Cargo invocation for units proven to have the same native dependency and feature
configuration, retaining distinct executable targets and each fixture's process.
Preserve typed dependency-plan authority; do not unify unrelated feature sets or
replace assertions. Shared mutable targets retain a single build owner. Source
failure isolation, original batch failures, locked/offline resolution, stale/corrupt
artifacts, interruption and safe lifetime/cleanup remain required. Do not introduce
a second dependency resolver or treat cached output existence as fresh validation.

Any implementation requires complete-category improvement, affected harness and
allocation/failure regressions, unchanged selected coverage and independent review.
The existing focused intermediate policy applies. This item does not alter the
mandatory SQL independent-clean-build obligation or close final480-second acceptance.


#### D7 retained measurement and prioritization — 2026-10-29

At unchanged candidate `d8956dad0f792de7c656d37d95bd1faa6017f16e`, exact
preparation passed in16.540s. Complete732-fixture prime execution passed in143.185s;
it refreshed resolution receipts despite186/186 native groups being fresh. This
is not the matched steady-state baseline. Subsequent complete runs passed in
102.308s with four inner Cargo jobs and104.766s with the default one job. Both
retained186/186 fresh groups and observed zero rebuilt compiler units. The default
run spent44.349s in frontend compilation,44.681s in native build validation and
11.753s in runtime. These are normal telemetry, not instrumented timings.

The concurrency difference is small and unreplicated; adopt no concurrency change.
The90s category budget remains unmet. A read-only census finds71 distinct normalized
manifest-and-lock configurations among186 groups (the largest contains58 groups).
This establishes a possible consolidation opportunity, not equivalence of typed
plans or permission to merge feature configurations. Cohort ownership would also
need to preserve resolution, artifact retirement and failure-isolation contracts.
Defer that implementation behind larger measured costs; retain D7 as open timing
debt rather than reporting its budget passed. All commands and failure evidence
remain in the D7 evidence records. Two census-helper failures affected postprocessing
only and are preserved; the corrected census completed successfully.

### D8 — generated-test preparation and execution ownership (prospective)

The last complete generated-driver category passed81 assertion cases in439.735s,
with one preparation helper executed separately. Its isolated tests still invoke
Cargo once per assertion. The81 observed command durations sum to876.385s across
two workers; this sum is not category elapsed time. The longest cases include
independent native-build evidence106.901s and backend integration92.204s, but this
candidate changed production identities, so retained repetition is required before
attributing those costs to steady-state work.

First run the complete unchanged generated selection with fresh execution of its
preparation helper and preserve every assertion. Then compare a bounded diagnostic
that admits the exact Cargo-selected test executable once and launches the same
isolated cases directly. Cargo must remain the authority for executable selection,
working directory and launch environment; do not reconstruct them from guessed
paths or silently change configured runners. Preparation, publication and execution
costs must all remain counted. The diagnostic may use frozen owned artifacts to
measure feasibility; production adoption additionally requires explicit immutable
artifact ownership, bounded lifetime and failure/corruption/interruption checks.

Preserve current native allocation, process isolation, exact inventory and execution
counts, helper ownership, delegated-corpus policy and failure continuation. Builds
performed inside tests remain real, including both independent native evidence
builds. No profile or guarantee is removed. Reject a new execution mechanism if its
complete-category benefit does not justify the added responsibility. Use the normal
focused intermediate checks and independent review before adoption; the full phase
correctness and timing requirements remain open.


D8 tooling decision: evaluate nextest's existing Cargo preparation, binary metadata,
process isolation and outcome reporting before implementing a custom launcher, as
already called for by section10. The diagnostic pins the official0.9.150 Linux
asset (SHA-256`b0959bc64232974224cf9fb22c915845398c0e8668fa7831eb5b43fb1eef781f`)
in session-owned tools; it does not install globally or change project tool policy.
Keep two workers and the same inner build/Rayon allocation for the first comparison.
Reconcile exact ignored-test inventory, helper exclusion, all81 terminal outcomes
and no retries. Include preparation/admission time. Existing ordinary/doctest and
non-Rust boundaries remain unchanged in this diagnostic. A drafted custom-launch
probe remains unexecuted; it is not validation evidence or adopted infrastructure.


D7 evidence is archived as `dev-verify-d7-evidence.tar.gz`, SHA-256
`26ca400ac97b7c56f0335129a8b0814939dbad2a3b1f537ed8a78e2c9d480cb0`;
all41 payloads verified after transfer. No native concurrency change was adopted.

D8 retained baseline passes81 assertions with the preparation helper actually
executed:437.442s outer guard,388.500s first-to-last assertion span,776.000s summed
(overlapping) assertion commands. Peak sampled aggregate RSS3,869,106,176bytes;
minimum available9,935,069,184bytes; no reserve stop, leftover session or source
change. The independent native-build case remains105.383s. Advanced-data runtime
is67.663s despite preparation; this remains a separate freshness/ownership lead,
not an established diagnosis. Backend integration is29.968s, so its earlier92.204s
must not be used as the present steady-state cost.

Nextest discovery passes in2.725s, selects exactly the same81 assertions and excludes
only the declared preparation helper. Its actual driver executable has the same
SHA-256 as the Cargo baseline and D6 qualification. The complete unchanged two-worker
comparison is running. No new production dependency or runner implementation has
been adopted, and no full-phase timing claim follows from discovery.


D8 diagnostic correction: the first direct-binary nextest run was deliberately
cancelled after288.551s and its owned session emptied. It reconstructed native
families, so it is not a matched retained timing or a complete pass. A1.200s
Cargo launch capture confirms the difference: normal Cargo exports CARGO_HOME,
RUSTUP_HOME, RUSTUP_TOOLCHAIN and CARGO; direct nextest invocation inherited them
absent. CARGO_HOME presence is an intentional native identity input; do not weaken
that identity to conceal the experimental setup error. A CLI-only Cargo alias
control was rejected before tests (0.693s), and that evidence is retained too.

The pinned tool is now linked only into the owned verification virtual environment,
which is already on the recorded PATH. Use normal `cargo nextest` dispatch without
changing PATH, global installation, compiler flags or native identity rules. The
symlink target/hash is recorded for safe ownership and later retirement. Repeat
exact discovery and input comparison before interpreting the corrected complete
run. Custom launch capture code remains diagnostic-only; no custom runner is adopted.


Corrected D8 nextest execution passes all81 assertions in351.701s including
preparation; nextest admission/execution334.146s, summed case durations664.870s.
All81 successful JUnit outputs retain the libtest attestation of exactly one
executed assertion;806 other entries are explicitly skipped by the selection.
The actual test process native environment, including loader path, matches the
Cargo baseline. Peak sampled aggregate RSS3,951,939,584bytes, minimum available
10,144,133,120bytes; the owned session is empty and tracked source unchanged.
This is not yet an attributable85.741s runner gain: preparation and some case
costs also declined. A second complete original-runner execution is running
on the retained state before deciding adoption.


#### D8 diagnostic conclusion — 2026-10-29

The second original-runner execution passes all81 assertions in374.635s including
preparation, with348.223s assertion span and695.042s summed commands. Compared
with the corrected nextest run, the observed total difference is22.934s (6.12%),
with14.077s difference between assertion span and nextest admission/execution.
These differently bounded execution measurements are not a precise causal estimate.
The first Cargo baseline's82 commands (including preparation) sum to821.763s
against779.520s of reported libtest execution, approximately42.243s combined
launcher/orchestration overhead. Overlapping durations are never added to wall time.
Defer nextest adoption: this improvement is too small and variable to prioritize
over the measured native and independent-build costs. No runner, allocation,
selection, tool pin or production source change is adopted.

A targeted advanced-data process trace passes in30.148s including instrumentation.
It observes the final generated root rustc invocation at17.012s (including its
6.076s linker child); actual runtime is0.013s. This is diagnostic timing, not
acceptance. Temporary test-only reporting then observes9.900s libtest execution
(after44.95s test-binary preparation): fixture0.112s, Cargo entrypoint0.495s,
compiler build9.279s. All seven generated project files retain identical bytes
and mtimes. The reported native stage6.397s includes publication and must not be
called pure Cargo compilation time. The compiler report records a finalized cache
hit. An interleaving control also passes (15.969s combined, instrumented), without
reproducing the earlier expensive rebuild. The source of that rebuild remains
unresolved; do not claim global cache failure or implement a speculative repair.
The diagnostic-only source was restored exactly, the owned nextest symlink removed,
and failure/incomplete/trace evidence retained. The changed test executable requires
normal Cargo readmission before future acceptance.

### D9 — independent SQL build cost (prospective)

The previous complete native SQL clean/reused/independent-clean protocol passed
in511.420s and alone exceeded the proposed full ordinary-gate target. Measure
its actual current compilation graph before changing dependencies or policy.
Retain all three linked MySQL/PostgreSQL/SQLite executables, locked/offline Cargo,
the exact existing dev profile and constant two-root path remaps, reused-A byte
comparison, removal of A before independent clean B, and exact executable-byte
comparison. Cross-target checks remain outside this local investigation.

Use Cargo's unit timings and separately recorded clean-A/reused-A/clean-B commands,
CPU/resources and selected artifact inventories to identify the critical dependency
chain. Preserve diagnostics and failed artifacts, check capacity first, and keep
the1,750s diagnostic cap. Do not treat a cold build as host-performance evidence
or adopt input-bound qualification reuse without the policy decision required by
section11. The next implementation must remove unnecessary production/build
dependencies or other measured compilation work while preserving the actual
qualification guarantee; first inspect the existing feature boundaries so already
delivered isolation is not proposed again. Full phase480s acceptance stays open.

D8 evidence archive: `dev-verify-d8-evidence.tar.gz`, SHA-256
`7609504413f23e7b132b09f67e215215a016e7f415b1b691eab12fdd9b51d6e2`; all121 payloads independently verified after transfer.


#### D9 measured input-aggregation repair (prospective)

Clean SQL A passes in257.513s (339 artifacts, none fresh). Cargo reports391
units; longest overlapping unit durations include AWS-LC native generation80.210s,
LALRPOP55.610s, PostgreSQL native parser41.310s, and two syn builds35.480/35.040s.
These unit durations are not summed wall time. Native SQL providers are genuinely
used by catalog and migration operations; do not remove them on an unused-dependency
assumption. Independent B remains running.

A read-only reproduction of the current product input traversal identifies174,350
file visits over18,758 distinct logical paths,1,610,816,360 repeated bytes versus
336,312,284 unique bytes. The inherited Ruff root is traversed15 times. The small
SQL-tool build scripts include full product identity and each takes approximately
12–13s in clean A. The census is a structural explanation, not a measured speedup.

Consolidate declared roots into one deterministic input inventory per identity
calculation. Preserve all current logical input paths and explicit ownership
validation, optional-root behavior, source archives, symlink observations and
Cargo add/remove/change tracking. A broad root must not swallow an explicitly
declared descendant beneath an otherwise excluded target/.git/__pycache__ path.
Do not cache correctness results or file bytes across invocations. Bump identity
framing for the changed deterministic representation; use no compatibility fallback.

Before adoption, prove inventory equivalence against the previous implementation
on the current full source tree, shared/overlapping declaration and exclusion
controls, real Cargo invalidation and private-test isolation, and complete SQL
qualification. Measure the actual identity producer before/after with the same
source tree/environment and retain preparation separately. Then qualify affected
compiler/host-tool consumer pairing and obtain independent review under the
approved intermediate policy. The full phase gates and480s acceptance remain open.


#### D9 input aggregation qualification — 2026-10-29

Runtime candidate `cea605c1a875746a198dbfc50a311f7a04f29844` implements the
per-calculation root union. No persistent cache, excluded production input,
profile change, or compatibility path is introduced. Both local-package and
product identity framing advance explicitly to v3. This invalidates the old
representation; unchanged identities across that migration are not expected.

All 11 focused identity tests pass (10.390s command including preparation, 8.28s
libtest), including real Cargo source-archive add/remove/rename/symlink/configuration
invalidation, shared ownership, exclusion boundaries, and private-test isolation.
The first attempt failed to compile because an assertion message referenced a
shadowed inventory variable; the corrected attempt and original failure are kept.

Three alternating old/new actual producer runs on the same source tree give
median 9.655s before and 1.833s after, 81.02% lower. The old producer visits 174,350
logical file entries; the new producer visits 18,758, exactly the same unique
inventory with each file read once. Complete input paths/content hashes and both
producer binaries are retained. Preparation and inventory verification are outside
those per-producer timers; the complete diagnostic command is 42.332s. This is a
local producer comparison, not full-gate acceptance.

Actual generated Rust consumer controls pass unchanged reuse, declared producer
asset edits, and restoration. An independent source-archive control passes with
Rust 1.98.1 and installed 1.99.0-nightly: source identity remains toolchain-independent,
configured identity changes, repeated builds are fresh, and an actual production
code edit rebuilds and changes the executed result. The initial 1.98.0 attempt
correctly failed the declared minimum Rust version; preserve that diagnostic.

The complete SQL native protocol passes, retaining all 339 selected artifacts,
all three linked executables, constant two-root remaps and the original dev
profile. Clean A 255.488s, reused A 1.083s (339/339 fresh), independent clean B 262.834s;
A was removed before B. All three executable hashes match across all three builds.
Outer elapsed 520.937s versus baseline 528.929s. This small observed difference is
not repeated host-performance qualification. Peak sampled aggregate RSS 1.68GB,
minimum available 9.75GB; no reserve stop, surviving process or source mutation.
The baseline's B run overlapped a read-only input census; retain that limitation.
The large producer improvement does not solve independent clean-build latency.

Seven real compiler/native-tool sandbox tests pass in 52.59s, including direct and
confined MySQL/PostgreSQL/SQLite schema execution, admission/lock drift, missing
grants and incorrect compiler selection. Native tool preparation reports 1m24s;
test preparation 51.16s; complete guarded command 189.594s. These preparation costs
are not steady-state runtime assertions. All 10 protocol controls also pass (six compiler-session and four native-controller
cases): identity mismatch, worker death, timeout, diagnostics, shutdown and lazy
selection. Complete guarded command 130.775s includes 2m04s compilation; execution
remains separate in the logs. Independent review is recorded below.

Method-authority and filesystem-effect guards initially identified the extracted
exclusion predicate, changed lexical contexts and two new fixture reads. Explicit
ownership records were updated without changing either guard. Corrected guards,
unsafe-ABI and DX metadata inventories pass; formatting/file-size checks pass.
These policy-record changes follow the runtime candidate and leave its Rust,
manifest, lock, toolchain and consumer inputs unchanged. The reviewed final
candidate must retain that binding. Full phase gates, outstanding SQL component
provenance repair and repeated 480s qualification remain open.

D9 evidence archive: `dev-verify-d9-evidence.tar.gz`, SHA-256
`09bae68ef6b1df5d4e16bfe3429661631373f9414eaf2d3be7dbdd496d61e3b4`;
all 200 payloads verified after transfer. It includes failed attempts, source/input
bindings, actual producer executables, complete SQL timings/artifact selection,
consumer controls and policy results. Build targets remain retained separately.


D9 delivered through [PR #4519](https://github.com/sifr-lang/sifr/pull/4519),
reviewed candidate `30b988a26a876dccd7365dd36111973ba0687b52`, merged as
`1988fe8f6ce21cb4de809dfa3ee7dcf28dfda9a7`. Claude Opus verdict SATISFIED,
no blocking findings. Review and candidate binding are retained outside the tree
in `dev-verify-d9-review-s7v8xmey/response.md` and `binding.json` alongside the
local evidence archive. Follow-up suggestion: explicitly assert emitted watch
lines for excluded-name descendants in a future identity-test item. Logical
symlink aliases intentionally remain distinct inputs; no deduplication by inode
or canonical path is proposed. Record-only spacing/status corrections do not
change the reviewed implementation.

CI `local-first-create-pr` job 123196711202 failed before setup because the full
profile had no `SIFR_PERFORMANCE_REFERENCE`. The saved log identifies performance
reference admission as the owner; all downstream steps were blocked. That is not
a correctness execution or passing full-profile result. The intermediate local
policy and independent review cover this bounded merge; final phase gates stay open.

### D10 — MySQL statement parser build boundary (prospective)

D9's clean SQL timings still contain approximately 57s compiling LALRPOP before
the provider can compile. Inspecting the 145-line provider grammar shows that it
frames semicolon-separated statements, admits nine starting keywords, and retains
the remaining tokens. Detailed SQL syntax belongs to the existing handwritten
lowering code. Establish that boundary precisely before adopting a generated-file
pipeline or replacing the generated framing parser.

First compare a small explicit statement framer against the existing generated
parser in owned diagnostics. Preserve token values, statement spans, empty and
trailing-separator behavior, invalid starting tokens, lexer error precedence,
SQL modes, diagnostics and actual downstream AST/semantic behavior. Exercise all
token classes and state transitions, bounded exhaustive and deterministic property
cases, existing provider/compiler/property tests and retained corpus cases. Keep
the current generator as the independent comparison authority during investigation;
do not add a production fallback or two runtime parser authorities.

Adopt the smaller production implementation only if this evidence demonstrates
behavioral equivalence and the dependency graph actually removes the general
parser generator from normal native and component builds. Otherwise use a separately
owned pinned generator with byte-for-byte freshness verification; do not add an
unchecked generated artifact or conceal generated code from policy ownership.
Retain every existing SQL test and supported series. Keep generated/provider
provenance honest: rebuild affected component artifacts and qualify their current
inputs; never relabel old receipts. Dependency/lock changes must be intentional,
and architecture/verification ownership records must describe the final design.

Acceptance includes focused negative/positive regression tests, relevant policy
and SQL area contracts, native compiler/tool integration, actual component behavior,
and the full independent clean/reused/clean SQL build protocol. Compare the complete
cost, not only generator compilation. Keep the existing preparation accounting,
capacity reserves and 30-minute diagnostic cap. Use one independent review for the
bounded implementation. No ordinary/expanded qualification split or full-phase
480-second acceptance is implied by this item.


D10 diagnostic at `466bb00c6172f0ef9b7dae858c443cd2aad60f9f`: the explicit
framer matches the retained original generated parser on all 378,505 streams of
72 token classes through length three, 87,381 representative state sequences
through length eight (including lexer errors), 20,000 seeded longer cases and
28 actual lexer/SQL-mode cases. It compares complete results, spans, tokens and
error payloads. The successful guarded command takes 2.786s including compilation.
Two rejected prototypes are preserved: alphabetical expected-token ordering was
wrong; a state-dependent correction was also wrong. The complete comparison proves
that expected tokens retain grammar declaration order in every tested state.
No production change was made during those diagnostics.

The implementation now removes the general generator from the production and
qualification dependency graphs, keeps the former grammar as historical evidence,
and leaves detailed SQL lowering as the same authority. Before adoption the actual
production function must pass the same independent comparison, all relevant tests
and component/native qualification. Component receipts have not yet been rebuilt;
they must not be treated as current on this work-in-progress candidate.


#### D10 actual producer and consumer qualification — 2026-10-29

Source candidate `4be4dc11bac7f523f1adf4e848e683456e9f570c` removes the generator
and 11 now-unreachable lock packages without changing any retained package version.
The actual production function passes the original generated parser comparison:
378,505 token streams, 87,381 representative state sequences, 20,000 deterministic
longer streams and 28 actual lexer/SQL-mode cases. Complete tokens, spans and error
payloads match; guarded elapsed is 3.238s including diagnostic compilation. The
production function SHA-256 is
`839dd02d7aaae07035629882ce6b5f602e2b9b5807251983dadc541ed1689ce0`.
Both rejected ordering prototypes remain preserved.

All 17 initial MySQL unit/compiler/property tests pass in 46.956s including
compilation. Eight dependency, SQL ownership and provenance-input commands pass
in 8.355s. Formatting, file-size guardrails and method-authority, filesystem-effect,
unsafe-ABI and DX metadata policies pass without weakening or changing inventories.
Those initial component smoke assertions used the previous checked-in artifact;
the rebuilt-component checks below supersede that limitation.

All affected SQL components were actually regenerated with the canonical producers:
MySQL (all three series) 114.387s, PostgreSQL (all six majors) 244.446s, SQLite
33.305s. Shared virtualizer preparation after prior target cleanup took 93.751s
and is reported separately. PostgreSQL's first attempt failed before compilation
in 2.332s because the retained SDK extraction lacked its binaries. The preflight
had checked directory/archive presence, not extracted content; it did not establish
SDK readiness. A fresh owned extraction from the SHA-256-authenticated official
archive passed complete file/link validation in 15.966s; the successful retry uses
that extraction. Preserve the incomplete installation and failed attempt. This is
preparation recovery, not a product change or steady-state performance measurement.

The words fixture's current source and producer provenance still validates, so no
unaffected artifact was rebuilt. All four manifests validate after production.
All 13 changed SQL outputs were hash-verified during transfer, committed in
`b28227e0cfdf4de9739bf5c7e8949e27ea4bacbb`, and synchronized byte-for-byte before
consumer validation. Producers ran serially with two Cargo jobs, 20GiB disk and
2GiB memory reserves; source inputs and other families remained stable. There was
no timeout, reserve violation or surviving owned process in the successful runs.

The final component consumer check passes in 121.753s including compilation:
17 MySQL framing/compiler/property tests, two PostgreSQL tests that execute all six
component variants, and 14 SQLite compiler tests. Every selected test ran; none
was ignored or filtered out. All eight component/provider contract commands pass,
including 63 component qualification mutation controls. Per-command elapsed is
17.902s MySQL, 58.255s PostgreSQL and 38.172s SQLite; reported test execution is
0.11s, 1.32s and 0.06s respectively. These are distinct measurements, not added
savings. Native-tool integration and the complete clean-build protocol remain
pending at this checkpoint; neither full-phase acceptance nor an eight-minute
runtime is established.


Native compiler/tool integration also passes on `b28227e0`: all seven named
compiler-sandbox tests execute in 52.40s, with native-tool preparation 42.71s and
test preparation 45.29s. Complete guarded elapsed is 142.040s; the separate 22
non-sandbox tests in that target were intentionally outside this focused command,
not claimed as executed. The final phase retains their coverage obligation.
No source mutation, reserve stop or surviving process occurred. The complete
clean/reused/independent-clean native SQL protocol is now running on the same
candidate with the baseline's four-job allocation and remapping contract.


The complete native SQL protocol passes on `b28227e0`: clean A 232.027s,
unchanged reuse 1.557s (298/298 artifacts fresh), independent clean B 236.454s.
All three linked executable hashes match across all builds; A is removed before B.
Total guarded elapsed is 472.055s, versus D9's 520.937s: 48.882s / 9.38% lower
in this diagnostic comparison. Artifact emission decreases from 339 to 298 with
the removed generator dependency graph. These counts do not represent dropped
tests or native tools. The same three tool binaries, dev profile, two-root remaps,
locked/offline dependencies and four-job allocation remain in the protocol.
Peak sampled aggregate RSS is 1.407GB, minimum available memory 9.977GB; no reserve
stop, source mutation or surviving owned process. Targets and raw Cargo timing
reports are retained. A single cold-build protocol comparison is not repeated
host-performance qualification, and its saving must not be added to other timings.

The simpler provider implementation, equivalent framing behavior and actual full
protocol reduction support this bounded adoption. The remaining 7m52s SQL protocol
alone nearly consumes the proposed eight-minute gate budget. Full phase acceptance
remains open. After independent review and merge, run a fresh integrated correctness
sweep with failure continuation and per-category caps to select the next substantial
repair from current costs; do not extrapolate a complete gate from these focused
results or silently move required qualification out of the ordinary gate.


D10 evidence archive: `dev-verify-d10-evidence.tar.gz`, SHA-256
`09e8c4764e4789a1d194998f8cec1f8ef65d1d7b3edacd2212fea93c46e3df21`;
all 248 payloads verified after transfer. This includes failed prototypes, the
failed SDK attempt and authenticated recovery, actual component outputs, input
bindings, consumer results, full SQL timings, policy checks and diagnostic helpers.
Build targets and the extracted SDK remain retained separately. The evidence binds
the implementation at `b28227e0`; subsequent phase-record-only updates do not alter
those implementation or validation inputs. Independent review is the remaining
bounded-item adoption prerequisite.


Initial independent review confirms the actual production equivalence harness,
component hashes/provenance, dependency removal and current qualification evidence.
It reports NOT SATISFIED for three stale current-design descriptions: the MySQL
architecture paragraph, the private-tool list and the capability-matrix grammar
behavior entry. All three now describe the provider-owned parser boundary; the
historical grammar and producer records remain unchanged. The affected SQL
contracts and all 13 contract mutation controls pass. All four component manifests
still validate, confirming that no producer input or output changed. This metadata
fixture correction requires a scoped remediation review; no compiler rebuild or
broad test repetition is warranted. Initial review evidence remains outside the
Git tree in `dev-verify-d10-review-6fo5osl_/response.md`.


#### D10 closure — 2026-10-29

Delivered through [PR #4520](https://github.com/sifr-lang/sifr/pull/4520), final
candidate `6b104374eb6084c1014a354bf6e348dc43386019`, merged as
`8054fd42c48c499399d2579bcfcdd827ae981c49`. The initial review confirms the
implementation and evidence but remains NOT SATISFIED for its three stale-record
findings. Scoped remediation review is SATISFIED with no remaining blockers,
retained outside the tree in `dev-verify-d10-remediation-tk218uwn/response.md` and
`binding.json`. It confirms that the final candidate differs from the tested
runtime only in the declared documentation/verification metadata records.

Nonblocking SQL metadata follow-ups: remove the formatting space in
`check_mysql.py`, choose a consistent `reconciled_at` representation in the Rust
registry record, distinguish current/history entries in the MySQL grammar evidence
row, and remove the pre-existing duplicate provider key in MySQL qualification.
These are recorded suggestions, not new phase blockers or silently changed inputs.
The original full create-pr CI job 123210774820 stopped before setup because
`SIFR_PERFORMANCE_REFERENCE` was absent; its log is retained as
`dev-verify-d10-ci-create-pr.log`. No full-profile or CI-green claim follows from
this bounded merge. Full phase gates and repeated 480s acceptance remain open.

### D11 — current integrated correctness discovery (prospective)

After D5/D6/D9/D10 and the D7/D8 diagnostic decisions, run one fresh complete
`merge --correctness-only --no-fail-fast` selection with canonical preparation,
retained owned targets, the same four-slot allocation and 30-minute diagnostic cap
per category. Inspect the emitted plan and capacity before launching. Preserve
all original failures and prior evidence. Capture source/command/environment
identity and preparation separately; account for every failed, blocked, cancelled
or unexecuted category. Schedule remaining independent categories explicitly if
an aggregate cap prevents their execution. Do not mark partial execution as a pass.

Use ordinary execution for elapsed time; only apply targeted instrumentation if
the completed sweep identifies an unresolved large cost. Keep the source candidate
frozen during execution. This checkpoint is discovery, not a repeated timing
qualification or permission to exclude expensive guarantees. Select the next
bounded repair from the current largest measured cost and its real assertion
boundary. Prefer removing repeated compilation/preparation or an unnecessarily
expensive test boundary over replacing the runner for a marginal gain.


D11's first complete selection at `567d6b8597005d47ccad0a5c0da555bce8a9f592`
stops at admission in 24.701s outer / 23.380s facade elapsed. The dependency-direction
guard still permits only SHA-2 for `sifr_identity`, although D6 introduced optional
TOML parsing behind its explicit build feature. The other five early guards pass.
The runner correctly blocks all preparation and consumers for invalid inventory;
no broad test timing or coverage was measured. The complete incomplete-run archive
is retained as `dev-verify-integrated-d10-discovery-evidence.tar.gz`, SHA-256
`585b16b4890642aad9e61c22ccfc3170ba041350f219b3375ef68d5efec991c1`.

A separate smallest affected check reveals the related default-feature omission:
`cargo test --locked --offline -p sifr_identity --lib --no-run` fails in 1.200s
because cfg(test) includes the build helpers but no TOML dev dependency exists.
The earlier explicit-feature checks did not cover this configuration. Preserve
that diagnostic instead of treating the successful build-feature run as sufficient.

The bounded admission repair will retain optional, explicitly selected production
build helpers, add the same workspace TOML dependency for unit tests, and make the
policy recognize that precise boundary. Negative controls must reject unconditional,
default/transitively default, renamed and target-specific production activation.
Check the actual default and build-feature unit-test configurations and resolved
normal dependency graph; keep all existing assertions. Do not add TOML to ordinary
identity consumers or bypass the admission guard. Refresh affected producer receipts
through actual production, then resume full discovery on a frozen candidate. The
previous failed selection and unexecuted categories remain explicit.


Admission repair source `f623d46a1f79a2d6d4853e48b807f06cc1106e3e` passes the
actual dependency-direction guard/self-tests, all 11 identity tests under default
features (10.110s command / 8.36s execution), the same 11 under explicit build
features (8.594s / 8.28s), and all 15 dependency-audit tests (2.173s command).
Cargo's resolved normal dependency graph confirms that default identity contains
SHA-2 and excludes TOML; the explicitly selected build graph includes TOML.
Complete guarded check elapsed is 29.206s with no source mutation or reserve stop.
Formatting and file-size checks pass. Target-specific policy mutations are further
isolated from optionality/alias errors, so each ownership condition is exercised.

The only changed SQL producer input is `crates/sifr_identity/Cargo.toml`; all
three SQL receipts therefore require actual production refresh. The words fixture
input map is unchanged and does not require rebuilding. The admission archive's
19 payloads were verified after transfer. No registry version or lockfile changes
are required by the inherited test dependency.


Actual component production refresh succeeds: MySQL 24.973s, PostgreSQL 228.957s
for all six majors, SQLite 19.848s. All ten regenerated component binaries are
byte-for-byte identical to the prior qualified artifacts; only the three source-bound
receipts change. All four manifests validate, and the words fixture remains untouched.
The refreshed outputs are transferred with SHA-256 verification before restarting
integration. No old receipt is relabeled without executing its canonical producer.

#### D11 admission closure and preparation recovery — 2026-10-29

[PR #4521](https://github.com/sifr-lang/sifr/pull/4521) merged exact candidate
`b74b293944cd7dd0a60cda0d63830f258b839b5b` as
`b34d432f11fa7e10cb26e20ff9297ad6f7bdcfbd` after independent `SATISFIED` review.
The focused archive `dev-verify-d11-repair-evidence.tar.gz` has 102 verified
payloads and SHA-256
`982322ce1acf3c33512b380dda7858eebd704a983ab0fe6c77f3e333f43e9b5d`.
The full-profile CI job stopped before preparation because the performance
reference was absent; this is not CI-green or final phase acceptance.

The subsequent r2 discovery exhausted the aggregate preparation cap: 1787.561s
outer elapsed, 1750.263s preparation, exit 124. Sanitizer preparation received
194.940s of the remaining budget. All downstream consumers were blocked, not
failed assertions. Connectivity recovery confirmed an empty owned process session
and unchanged source. All 209 archive payloads verify in
`dev-verify-integrated-d11-discovery-r2-evidence.tar.gz`, SHA-256
`27a3c9074213afd9f45f79727c028c9a62f6fb7e204a3adae05f854b3c42392f`.

Each unfinished preparation node then ran independently under a 1750s cap:

| Preparation node | Guarded elapsed | Outcome |
|---|---:|---|
| Runtime sanitizers | 225.505s | Pass |
| Fuzz/property | 34.473s | Pass |
| SQL | 733.479s | Pass |
| Maintained demos | 17.231s | Pass |
| Compiler identity | 4.773s | Pass |

Every session ended empty. These are preparation results, not consumer assertion
counts. The recovery archive includes the initial helper import failure as well
as the corrected execution; all 75 payloads verify in
`dev-verify-d11-preparation-recovery-evidence.tar.gz`, SHA-256
`6e6e5d1ce9b54a1d1db3b520ee930094e50cfa6caf994ce31569da164554155f`.

#### D11 integration reconciliation — in progress

The complete r3 discovery retains candidate `b74b29394`, the inspected selection,
ordinary execution, failure continuation, and category caps. Preparation passes
in 212.003s with retained products. This comparison with recovery is a build-state
difference, not an implementation speedup. The remote execution inputs remain
frozen while an independent local checkout prepares confirmed repairs.

Three integration failures have been identified:

1. Coverage readiness still classifies the MySQL `custom-build:build-script-build`
   target removed by D10. Remove only this nonexistent target; retain the library,
   compiler tests, property tests, and live differential classification. Readiness
   and all 58 negative controls (including 31 Python-delivery mutations) pass locally.
2. Sixteen generated diagnostic documents retain driver fixture paths from before
   D6. Run the canonical `gen-error-docs` producer. Every generated change is a
   representative-fixture relocation from `src/tests` to `tests/unit`; diagnostic
   text, IDs, and examples are unchanged. The canonical docs-sync and link checks
   pass. The other 185 diagnostic variants passed in the frozen discovery.
3. The external diagnostic launcher selected an older Node runtime instead of the
   already provisioned, exact editor-owned Node/npm pair. This is a diagnostic
   environment defect. A separate read-only `node_toolchain_contract` check passes
   in 0.366s with Node 26.8.2 and npm 12.0.2 at the same source candidate. The other
   70 distribution variants passed. Preserve the original failed area and require
   exact Node/npm preflight for subsequent launches; do not change repository pins.

Batch the two source reconciliations with focused checks and independent review
under the approved intermediate policy. This does not waive the remainder of the
current discovery, any blocked category recovery, final complete gates, coverage
and invalidation qualification, or repeated 480-second acceptance. Archive the
frozen run before replacing any reports or transferring the repaired source.

#### D11 reconciliation closure — 2026-10-29

[PR #4522](https://github.com/sifr-lang/sifr/pull/4522) merged candidate
`b5a7fd153fe3f38eace384ed4272c800c8a9aa0c` as
`f5fc0fa5ceca2b2a80159d113088042f16605a0b`, with independent `SATISFIED` review.
Readiness, 58 negative controls, canonical diagnostic generation/synchronization,
links, and file-size checks pass. Its archive has 20 verified payloads, SHA-256
`831e3f82587dcf1087ab0133ecbf1146219e34b6e719eba7a262201bd518a80f`.
The full-profile CI job again stopped before setup for the absent performance
reference. The frozen remote discovery still uses the original b74 candidate.

The external discovery launcher explicitly sets `RUST_TEST_THREADS=1`; the D5
allocator correctly honors that smaller request. These serial diagnostic timings
are not the default driver allocation's latency. Subsequent ordinary comparisons
must record effective outer and nested workers, and use the exact provisioned
Node/npm pair. Do not multiply four test workers by four nested Cargo workers.

### D12 — eliminate repeated policy scanner work (prospective)

The r3 method and unsafe main scans take 34.321s and 59.198s; developer-tooling
architecture controls additionally take 89.368s. Local profiling identifies two
avoidable operations: each `Source` tokenizes identical Rust text twice, and each
contract pair normalizes all four fields even after an earlier field differs.
An in-memory local prototype reduces the two main scans from 50.832s to 28.002s.
That single local observation is not remote or full-gate timing acceptance, and
its cold-child tests copied the unchanged original sources.

Tokenize each immutable source once, deriving comments and code tokens from the
same ordered token list. Normalize contract fields lazily and stop comparison at
the first unequal field. Preserve exact pair-specific binding removal, all four
obligations, nontransitive connected-component grouping, discovered sites, policy
records, ordering, and diagnostics. Do not cache correctness across invocations,
weaken negative controls, or replace lexical authority with text matching.

Acceptance: the actual edited sources must pass all architecture-policy controls,
including cold subprocess copies, each-field distinction and existing nontransitive
grouping controls. Compare complete main-check outputs with the unchanged source,
then measure the affected commands on the retained remote environment after D11
is archived. Adoption requires lower complete affected-check cost with unchanged
decisions. Use focused checks and independent review under the approved intermediate
policy; complete phase gates and repeated 480-second qualification remain open.

The SQL duplicate-execution inventory and bounded CLI concurrency experiment remain
separate measured leads. This local repair does not mutate the running remote inputs.

The actual on-tree implementation passes 57 architecture-policy tests in 23.906s,
including copied-script cold subprocess checks and the new four-field distinction
control. Both complete inventory checks pass unchanged counts: 6,458 method sites /
235 reviewed nodes and 410 unsafe sites / three complete source partitions. Local
main-check elapsed is 13.226s + 14.661s = 27.887s. File-size and diff checks pass.
These results use Python 3.14.7; remote comparison and independent review remain
pending. No policy inventory or compiler/runtime source changed.

#### D11 completed diagnostic and D12 closure — 2026-10-29

D11 executes all selected categories: canonical wall 5006.040s, facade 5019.521s,
guard 5022.365s. Thirty-five steps pass; coverage readiness, diagnostic docs-sync,
and the Node environment contract fail as previously recorded. No category is
blocked or unexecuted. The source is unchanged and the owned session ends empty.
The original run stays failed despite the subsequent focused repairs.

All 19 crate configurations pass in 1735.989s, including 805 ordinary driver tests
(508.368s),80 generated-driver assertions (420.286s), and six generated CLI
assertions (47.442s). Of the driver's82 ordinarily ignored cases,80 have generated
execution, one is explicit preparation, and the corpus assertion is owned by the
selected installed metadata corpus. All 732 native fixtures pass in 136.833s;
frontend compilation44.357s, native-build stage79.011s, runtime11.908s. All 186
native groups reuse fresh artifacts but still perform Cargo validation. The
external one-thread diagnostic override remains a material timing caveat.
All 6868 archive payloads verify in `dev-verify-integrated-d11-discovery-r3-evidence.tar.gz`,
SHA-256 `ef27d985bbd8daa30a398d35680b49402c53489d642c5109be081b621d76e881`.

[PR #4523](https://github.com/sifr-lang/sifr/pull/4523) merges reviewed candidate
`1c407e4cd0ffaa04d689b0cdc0919b185470b358` as
`6bf629893c3cf050044ff76379e4616338244c8c`. Independent review is `SATISFIED`,
without blocking findings. Remote main scans pass in 25.108s and 26.975s, versus
34.321s and 59.198s in D11: 93.519s→52.083s. Including both self-test entry points
and the architecture suite, affected elapsed is 183.755s→95.773s (one ordinary
same-host comparison, not repeated phase qualification). All 57 tests pass remotely,
including cold copies; main outputs match local outputs byte-for-byte. The
103.717s guarded run includes file-size/diff checks and ends empty without drift.
The baseline suite has 56 tests; the candidate adds one four-field regression.

Final evidence archive `dev-verify-d12-evidence.tar.gz` has 23 verified payloads,
including the separately verified 28-file remote packet; SHA-256
`b6f40204ad5434dbd6da05d912bf223cee7f63a2ee89875fde48eac02c384a0a`.
Review records remain outside Git, keyed by candidate. Follow-up suggestions
(unused record parameter and additional binding-label variants) are optional
policy-maintenance work. Full-profile CI again stops before preparation for the
missing performance reference; final phase gates and 480-second acceptance stay open.

### D13 — SQL editor execution boundary (prospective)

The current SQL incremental-editor case runs all tests and doctests in
`sifr_frontend`, `sifr_analysis`, and `sifr_lsp`, taking 116.518s. The ordinary
crate phase then executes those same suites. All four delivery profiles already
enroll the three complete, unfiltered crates in both smoke and full modes.
The SQL area inherits the same execution environment, with only its category
deadline differing; observed executable paths match. The expensive language-server
run includes unrelated workspace and protocol tests (104.90s in the SQL case).

Narrow this area case to library tests for SQL, embedded analysis and template
documents. Current discovery selects 12 frontend, 11 analysis and two LSP tests.
Keep every complete crate command, integration target, doctest, feature graph,
and existing ignored-test owner in the ordinary profile selection. The SQL query,
migration and schema integration cases keep their existing area enrollment.
Standalone incremental-editor verification becomes an explicit focused check;
complete development validation still owns the full crates.

Require three nonempty, successful harness outcomes for the focused case; zero
selection, ignored-only execution or missing crate output cannot pass. Validate
profile ownership before execution: selecting this area requires each original
complete crate in the profile's actual crate-test mode. Negative controls must
reject removed, filtered, disabled or wrong-mode owners. Keep one shared command
definition for preparation and execution, and preserve all profile selections.

Acceptance: compare actual selected test identities with current discovery;
execute all 25 focused assertions with the existing pinned environment and verify
unchanged complete crate selections in every delivery profile/mode. Run SQL
incremental-editor qualification/mutation checks, runner ownership/preparation
controls, profile checks and file-size checks. Measure the complete affected case
and reject a negligible gain. Independent review and final phase gates remain.
Do not introduce cached-success reuse or a general delegation framework.

Prior profile experiments remain authoritative: blanket opt0 caused 6.697x slower
metadata validation, and selective semantic opt0 caused 2.133x slower source checks.
They are not repeated as a shortcut. CLI allocation, Python fixture identities,
native artifact validation and the independent SQL build cost remain separate leads.

#### D13 closure — 2026-10-29

[PR #4524](https://github.com/sifr-lang/sifr/pull/4524) merged candidate
`f90486060e7a3a2f1d1e865de9fc27f9e60897a0` as
`147910bae7f9a96bd6dc3b542e9714682c0a1e6e`. Independent review is **SATISFIED**,
with no blockers. Actual Cargo discovery and execution agree on all 25 selected
assertions: 12 frontend, 11 analysis and two LSP tests. The focused Cargo case
passes in 2.913s versus the prior broad command's 116.518s; the entire focused
area, including its qualification contract, passes in 3.069s. This is one ordinary
comparison with retained products and an explicit single test thread.

All 17 focused execution/ownership controls and all 18 preparation controls pass
remotely. SQL qualification and its four metadata/six source mutations pass.
All six supported full/correctness-scoped emitted plans remain equivalent; all
four delivery profiles retain complete unfiltered crates in both modes. No Rust,
feature graph, complete crate command, integration target or doctest changed.
The guarded diagnostic ends empty after 13.390s, without drift or reserve failure.

The initial broader local preparation attempt has two platform-related failures:
canonical `/private/var` versus `/var` temporary-path comparison and process-group
SIGKILL permission. These belong to preparation/process controls and stay recorded;
the remote suite passes without changing their implementation. The first external
plan-comparison helper also incorrectly requested unsupported correctness scope for
nightly/release; its corrected helper checks only supported combinations. Neither
failed attempt is relabelled as passing.

Archive `dev-verify-d13-evidence.tar.gz` verifies all 84 payloads; SHA-256
`e8852bf92f241f6a4c6fb369caf063c2b04a5e0c6dafc42494737fe7881961bd`.
Review: `dev-verify-d13-review-wnz6yufy/response.md`, outside Git and bound to the
candidate. Optional follow-ups are streaming focused stdout through cancellation
and detecting newly added editor tests whose names miss the focused filters;
complete crate execution already retains those tests. Full-profile CI again stops
before preparation for the missing performance reference. Final phase acceptance
is still open. This subgroup is now small enough; move to larger costs.

### D14 — bounded CLI integration execution (prospective)

D11 spends 555.103s in the complete CLI/lowering configuration, with the external
diagnostic explicitly forcing one libtest thread. This is not default-policy
latency: the ordinary CLI configuration currently inherits libtest's worker count,
while nested Cargo/Rayon already have their own budgets. D5 bounds only ordinary
driver tests. CLI cwd-changing tests already execute in isolated child processes;
standalone native publication deliberately retains its mutex and artifact lease.

First measure the unchanged complete CLI/lowering selection with three admitted
outer workers and one nested Cargo/Rayon worker, using the existing allocator in
an external diagnostic. Keep every assertion, ignored/generated owner and corpus
exclusion unchanged. If this passes, compare a same-candidate one-worker run with
three nested workers, retaining inputs, environment and artifact identities. Check
all target outcomes and exact ordinary test identities, not only Cargo's exit code.
Each diagnostic retains the 1750-second cap and capacity/process-cleanup controls.

Only adopt bounded CLI allocation after the complete comparison passes and shows a
material elapsed benefit versus serial diagnostics (at least 15%). Do not claim a
speedup over the formerly unset libtest default from that comparison. Any production
change must use one shared allocation policy, preserve smaller explicit user limits,
reject explicit overcommit, and retain cancellation/reserve behavior. Keep native
specialized execution separate. Use focused allocation/selection controls and
independent review; full phase gates and repeated 480-second acceptance remain open.
A faster CLI category alone does not satisfy the 120-second integration design budget.

The initial external probe stopped before preparation because it assumed four slots
were always admissible. Available memory is close to the four-slot boundary; that
is a diagnostic precondition error, not a test failure. Record it and compare fixed
three-slot allocations (3/1/1 and 1/3/3) below the existing profile ceiling, retaining
the reserve and refusing execution if three slots are unavailable. Do not change
production resource policy to force admission.

The first admitted CLI diagnostic fails because the external helper omitted the
canonical explicit source sysroot and revision-specific portable dependency fetch.
Its 130.382s failed execution is not a speedup. Preserve all failed targets and
prepare the canonical workspace, fixture, Python and generated dependency nodes,
then publish the compiler through its owned session. Prove both the portable
relocation and source-sysroot CLI cases before another complete comparison.

The shared allocator prototype also bounds `SIFR_E2E_CARGO_BUILD_JOBS`: two ordinary
CLI assertions invoke standalone native builds through an explicit harness job
setting, which otherwise could exceed the outer-times-inner budget despite bounded
`CARGO_BUILD_JOBS`. Keep their publication mutex/lease and all assertions unchanged.
The full corpus keeps its separate specialized allocation.

A follow-up development-interface gap is now concrete: invoking a crate category
outside the profile does not automatically acquire its canonical sysroot and
portable-revision prerequisites. This caused the failed external diagnostic. A
future focused-category entry point should select the same authoritative commands
and prerequisite graph, without reconstructing that setup in ad hoc helpers. Keep
that interface work separate from the current allocation candidate.

Canonical prerequisite preparation passes in 358.469s, including both focused
setup proofs. Its 93 generated materializations and locked fetches account for
348.086s. A subsequent direct-Python measurement is cancelled after 183.031s when
launcher inspection confirms it bypasses the canonical `uv run` entry. Native
identity includes PATH, so it is not accepted as ordinary timing evidence. The
owned session is empty after cancellation; no serial comparison ran. The corrected
external helper asserts the verification virtual environment and leading PATH
entry before launching any tests. Preserve the cancelled evidence and require the
canonical one-layer uv entry for both paired runs.

The first canonical three-worker pass is rebuilding retained native dependency
families (observed optimized `encoding_rs` and debug dependencies), while an
unrelated-Cargo-manifest test deliberately owns a private cold cache. Treat the
first canonical pass as functional priming, not warm timing acceptance. Preserve
its results, run the serial baseline, and require a retained three-worker repeat
for the comparison. Intentional per-test cold-cache work remains included in both.

### D14 qualification — bounded CLI allocation

The canonical frozen `f90486060e7a3a2f1d1e865de9fc27f9e60897a0` comparison passes:
531.559s serial versus 439.194s with three outer workers and one nested worker,
17.376% less elapsed time. Both execute exactly 1,454 passing test identities,
11 ignored identities and identical per-target counts/binaries/commands. Both
sessions end empty without cap or source mutation. The first 425.766s three-worker
run is preserved as dependency priming, not the accepted comparison.

SQL applications improve 203.30s to 109.85s; CLI binary tests regress 58.39s to 88.70s
and runtime observability 17.82s to 28.77s in this allocation comparison. Inclusive
CPU 702.30s to 709.35s is effectively unchanged: this is scheduling improvement,
not removed compilation. The full 120s integration design budget remains unmet.
Do not claim improvement over the former unset libtest default from serial data.

Actual implementation candidate `40006daf8b3bdf1a749eda4e72a85c70f7a6f3c1` passes
all 24 focused controls (including real cancellation), complete runner foundation
in 76.120s, profile/workflow contracts, source-size and diff checks. Six emitted
plans match the base. Native Rust, manifests, lockfiles and profile inputs are
unchanged from the timing candidate; diagnostic allocation used the pre-existing
allocator at explicit widths. Candidate controls verify the shared allocator's
new dispatch, selection preservation and standalone native override bounds.
The 84.250s candidate validation guard ends empty and unchanged.

The 11 ignored entries remain explicit. One pre-existing lowering deque inference
assertion lacks an established exact execution owner; owner is lowering/stdlib
compatibility. It is unexecuted, not a pass, and remains a final coverage follow-up.
Other ignored/generated/helper contracts and the separately owned native corpus
are unchanged. Failed/cancelled setup attempts remain preserved.

CI full-profile create-pr job 123440352375 stops before validation setup because
`SIFR_PERFORMANCE_REFERENCE` is missing. This is the existing configuration issue;
there is no CI-green claim. Full phase gates, complete coverage/invalidation and
repeated 480-second acceptance remain open.

D14 is delivered in [PR #4525](https://github.com/sifr-lang/sifr/pull/4525),
merge `3bf9891065f1fbff2ccb57242d64f3017b49be32`. Exact-candidate independent
Claude review is SATISFIED with no blockers, retained in
`dev-verify-d14-review-u4xpjoua/response.md`. The 133-payload evidence archive
`dev-verify-d14-evidence.tar.gz` is SHA-256 verified:
`2f98aaa9d2eb7f3331da46cf7e92e2756665cbd4dbccb2b51be0ae2293353773`.
No smoke-mode timing claim is made. Clarify the full configuration's shared
lowering allocation in the next documentation update; no new allocator exception
is needed. The local macOS cancellation failure is retained; the actual remote
control passes.


D14 evidence correction: after merge, inspection found that the retained three-worker
run used the old launcher, changing PATH and TMPDIR relative to serial. The 17.376%
comparison is withdrawn; it is not matched performance acceptance. Functional
results,24candidate controls and source review remain passing. Merge4525 exists,
but D14 performance qualification is reopened. Preserve its original archive and
review as historical evidence of the mistaken comparison. The corrected run must
assert the full recorded execution environment matches serial except the declared
worker counts, and assert identical TMPDIR before launching. Do not begin a D15
implementation until this qualification is reconciled.

### D14 corrected qualification — matched environment

The corrected r5 run passes in 422.091s versus the 531.559s serial baseline:
20.594% lower elapsed time. Before execution it asserts equal recorded environment
values except the declared worker counts, plus identical temporary-storage parent.
The comparison independently verifies those equalities, exact 1,454 passing and 11
ignored identities, commands, binaries and per-target counts. Outer guard 425.933s;
no cap, source change or leftover processes. Preparation 2.529s versus 2.549s.
The original17.376% comparison remains invalid; the corrected evidence does not
rewrite that history or justify the premature merge decision.

Corrected SQL application cost 203.30s to 108.20s; CLI binary 58.39s to 89.23s;
build-output 76.12s to 69.10s; CLI integration 88.31s to 63.84s. Inclusive CPU 702.30s
to 708.97s still shows a scheduling improvement, not removal of compilation.
The same frozen native candidate and implementation binding remain explicit.
No implementation changed after the SATISFIED source review; the 24 actual-candidate
controls and full foundation remain applicable. This record-only correction does
not claim a new source review, smoke timing or final phase acceptance.

The corrected 11-payload archive `dev-verify-d14-r5-evidence.tar.gz` is verified:
`324118827761970a145994327b5e7b2d3f6b1eb184d7791aff2f09ffd671fc59`.
Keep it with the original archive, serial baseline and withdrawn comparison.

### D15 investigation — repeated preparation and native ownership

No D15 implementation is selected yet. First measure the complete canonical
93-project generated preparation with retained products. The earlier 348.086s
observation used a different launcher: 310.489s was materialization and 37.005s
recorded Cargo build/fetch commands. It is not a current matched warm baseline.
Use the unchanged canonical environment and all original public materializations,
locked fetches, source-sysroot and portable-relocation proofs. Compare the actual
preparation to its 90s engineering budget before selecting work.

A separate source-backed lead concerns native storage ownership. The lock-mode
CLI fixture creates a temporary working directory; `owner_scope()` selects that
non-Git directory, producing a new owner on every invocation. Four lock policies
then partition it again. Retained native targets contain 24 byte-identical library
fingerprint JSON files out of 25 across eight inspected targets. This confirms
artifact duplication, but the inspected invocations include differing toolchain
environments and do not establish a matched saving.

Do not remove owner or lock-mode keys blindly: those families also contain mutable
generated project roots, lock state and cleanup ownership. Map tests that require
working-directory semantics versus tests that can use explicit project paths.
If storage architecture changes are justified, preserve independently owned
mutable roots, request resolution/trust, exact Cargo flags, lease/capture behavior,
source replacement, changed-input invalidation and cleanup protection. Verify
actual reused units and fresh runtime assertions, not fingerprint equality alone.
Prioritize the largest current cost; do not turn this lead into an implementation
before the baseline demonstrates value. All final phase obligations remain open.


### D15 measured decision — reject finite materialization batching

The isolated prototype `368ac774c1c2eaba6b910d388ac3d4e004086721` retains
all 93 public scalar materializations, locked fetches and exact output-tree parity.
The matched comparison takes 128.864s scalar versus 125.426s batched for
materialization: 2.668% lower elapsed time. This does not justify another protocol;
do not merge the prototype or change production preparation to use it. Total
materialization plus fetch is 165.330s versus 149.436s, but the scalar side includes
first-revision fetch work and is not a matched recurring-cost saving. Both guards
finish with empty owned sessions. The complete experiment archive is
`dev-verify-d15-batch-experiment.tar.gz`, SHA-256
`324409f1c7ac7629d10949ee298989a612d0368b25aede036b1f7a9305d17561`.
Failed precondition/helper attempts remain preserved, not reclassified as passes.

The canonical retained full prerequisite baseline is 168.280s including both
prerequisite proofs; materialization is 129.499s. Its preceding 341.389s pass is
priming. The verified baseline archive is `dev-verify-d15-baseline-evidence.tar.gz`,
SHA-256 `ce335fd51a1b3c0609244b3d17999e9eabe98d3246b511e3f7938017e8a046ce`.

Diagnostic-only candidate `904f52db43240cdeac9699d513f8f5f1ef5851b7` executes all
93 scalar materializations in 130.360s. Summed overlapping request stage durations
include generated-project formatting 79.589s, Rust interop metadata/probes 68.754s,
and standard-library loading 65.132s. Native Cargo children account for only
15.487s within probe qualification. There are 1,224 rustfmt calls. These are nested
and overlapping attribution measurements, not additive wall time or acceptance.
Further representative instrumentation at `999693dc3b1ecfcc9c960d214b40277957187552`
locates substantial project canonicalization and repeated file formatting costs.
Do not merge either diagnostic candidate. The 769-payload verified archive is
`dev-verify-d15-stage-evidence.tar.gz`, SHA-256
`ab3e1dd70f051be0bc959cc43d2f2741d25c712becca7942dafec97205837fa7`.

### D15 normalization experiment — resolve support demand before normalization (rejected)

Representative attribution narrows the formatting cost to repeated semantic syntax
cleanup. Text/i18n runs 11 syntax cleanup passes, totaling 0.844s within 1.777s
project canonicalization; refreshing support imports invokes another full source
canonicalization. Native file-runtime similarly spends 1.103s in nine syntax
passes within 2.093s project canonicalization. These instrumented samples are
causal diagnostics, not corpus-wide savings. Repeated rustfmt inputs alone save
only 0.42–0.47s per sampled project and do not justify a replay cache.

Resolve compiler-owned support imports after project field/name preparation,
then prune dead owners before expensive semantic normalization. Keep the existing
final support-import/canonicalization fixed point: later transformations can still
remove references. This changes ordering, not coverage, formatter authority or
success caching. Do not adopt unless the generated source contract remains valid
and the complete preparation measurement demonstrates material benefit.

Require full codegen unit regressions and affected driver project/materialization
checks, including transitive support demand, fixed-point stability, cross-file
references, same-named fields, namespaces and bridges. Compare all 93 generated
trees to preserved scalar outputs; separate manifest revision-only differences
from generated source. Preserve locked fetches, portable relocation and
source-sysroot proofs. Measure uninstrumented complete canonical preparation after
priming; 90s remains the preparation design budget. Exact-candidate independent
review is required before intermediate merge. Full phase coverage, invalidation,
final gates and repeated 480s qualification remain open. No ordinary/expanded gate
split or coverage reduction is authorized here.


D15 ordering candidate `50c541c9552a9ebb17b436aee8e006700d093a01` passes all
1,786 codegen unit tests, with no ignored tests, in 14.51s after 274s of test
preparation. The subsequent production CLI build takes 174.708s. These are observed
build-command costs, not isolated per-crate compile times or a matched edit-cycle
saving. Formatting/size checks and four representative materializations pass;
the complete focused guard takes 516.500s and exits empty without source mutation.

Complete canonical preparation passes all 93 projects, locked fetches, portable
relocation and source-sysroot proofs in a 200.803s guard. Every generated tree is
byte-identical to the preserved scalar baseline after replacing only the exact
Sifr revision in Cargo.toml/Cargo.lock. Materialization alone takes 132.114s,
versus the prior retained 129.499s baseline. There is no demonstrated improvement.
The overall first-candidate preparation includes revision-specific work and must
not be treated as a matched recurring regression. Representative timings likewise
show no convincing saving. Reject the implementation; do not spend another broad
timing/review cycle or merge the ordering change. No production code from D15 is
adopted. The first helper attempt failed on a renamed allocation-module import
before preparation; preserve it as a helper failure, not a test result.

The 696-payload rejected-candidate archive is SHA-256 verified:
`dev-verify-d15-normalization-rejected.tar.gz`,
`08fd19aaac0e21896cf6ef125b6c6d6e24c20ff3f78ffc6e1e0a85b2ec5084f6`.
The additional 195-payload pass/formatter diagnostic archive is
`dev-verify-d15-normalization-diagnostic.tar.gz`, SHA-256
`4d1ee19e14271f4ecbbb79dd621ae306bd30853ca17418ba266f703746c3b6c8`.
Retain rejected branches and evidence; neither rejection weakens any final gate.

### D16 investigation — compiler test and production compilation boundaries

Shift to preparation after an ordinary compiler edit, whose design budget remains
90s. The D15 candidate spent 274s preparing codegen tests and 174.708s subsequently
building the CLI; the 1,786 tests themselves took 14.51s. Distinguish compiler
configuration priming from recurring rebuilds before assigning those costs to a
specific crate or claiming a saving.

Inspect actual Cargo configurations, dependency features, build-script ownership
and rebuild propagation. The codegen manifest still declares the entire src tree
as production input while private test modules remain inside that tree. Verify
whether assertion-only edits propagate into production identities and builds;
D6's driver/identity qualification must not be generalized to unmeasured crates.
Map private-invariant tests versus public emission contracts before choosing a
boundary. Move tests or extract a coherent compiler responsibility only when a
consumer can stop compiling/linking substantial unrelated code. Preserve private
authority and avoid test-only public APIs or hundreds of separately linked targets.

Qualify actual unchanged, assertion, fixture and implementation edit cycles with
retained targets, full discovered-test reconciliation, appropriate regressions and
positive invalidation controls. Do not change optimization profiles, skip native
semantics, weaken identities, or silently split the acceptance gate. Measure the
complete affected development cycle and require a material improvement before
adoption, plus independent exact-candidate review. Continue the approved focused
intermediate policy; complete final phase gates and 480s qualification remain open.


### D16 implementation candidate — codegen test ownership and public contracts

A controlled assertion-message edit at `fa02de4d84b914ea8e478f016cf36ae4e7db9bee`
passes all 1,785 tests but costs 144.777s for the unit-test command (14.78s actual
assertions) and 20.054s for the subsequent production CLI build. Cargo reports
one rebuilt codegen test unit and six rebuilt production units. The CLI hash
changes despite no production logic edit. The complete guard is 167.790s. The
preceding 388.475s baseline includes configuration/revision priming and is not the
recurring comparison. Verified evidence: `dev-verify-d16-before-evidence.tar.gz`,
SHA-256 `e796cdb77efa120e12c71898090cd722c5c0f87a99ed47c3368b8098e4f15441`.

Move explicitly test-owned sources outside `src`, retaining logical private module
paths through cfg(test) declarations. Actual production dep-info independently
confirms exclusion of the moved test files. Extract inline test modules without
changing their assertions. Move public emission contracts into one integration
target consuming the existing public library; retain private invariants as unit
tests. Shared source-to-emission helpers have one test-owned definition. No new
product API, optimization-profile change, result replay, or coverage exemption.

Before adoption require exact discovered-test reconciliation, all codegen tests
(including the existing native ABI target), positive production invalidation,
unchanged/assertion/fixture/edit controls, production artifact reuse, relevant
identity and coverage-policy checks, and independent review. Reconcile source
evidence paths where relocation affects executable contracts. Measure retained
complete affected cycles after configuration preparation; 90s/480s budgets and
all final phase acceptance obligations remain open. This candidate is not yet
qualified or merged.

#### D16 focused qualification — public edits improved, implementation rebuilds remain open

Candidate source `9a7959ccdc3ed89f45b20a3edb32fe4e76c0c909`, with documentation
candidate `58fedb49d9ebf9f8a40e176652f6feec115bc59f`, preserves the exact union
of all 1,785 original test discovery identities: 1,488 in the unit target and 297
in one public emission-contract target, without duplicates or ignored tests.
All pass, together with the existing 18 native ABI tests. Both canonical
create-PR and merge memberships still select the complete package. No test-only
public compiler API, optimization-profile change, or result replay is added.

Initial candidate qualification takes 562.072s, including 389.887s for the complete
codegen test command and 163.136s for the subsequent CLI build. Configuration and
revision preparation are included; this is not a recurring edit-cycle saving.
Follow-up literal-preservation/policy corrections pass in a 195.842s guard.
All 98 changed production owners retain identical Rust tokens after removing only
comments and explicitly cfg(test)-owned modules/declarations. All 71 extracted
inline modules preserve their fixture and snapshot literals, except four relative
include paths corrected to the same source files. Actual production dep-info
excludes the relocated test sources. Test-only hooks retained inside production
files remain conservative production identity inputs; do not generalize the
measured test-module boundary to those hooks.

Retained controls use the same canonical launcher, three compiler slots, one
libtest thread, and separate 1,750s diagnostic deadlines. Each row executes all
1,785 original tests plus 18 native ABI tests, then builds the production CLI.
Rows represent alternative edit scenarios, not additive parts of one gate.

| Controlled edit | Codegen test command | CLI build | Complete guard |
|---|---:|---:|---:|
| Unchanged | 20.523s | 0.570s | 25.205s |
| Public assertion message | 23.070s | 0.563s | 27.209s |
| Embedded Sifr fixture comment | 21.645s | 0.613s | 26.203s |
| Assertion remaining in the unit target | 145.374s | 0.582s | 149.777s |
| Shared test-support assertion message | 40.878s | 0.614s | 45.215s |
| Positive production-input comment | 37.897s | 18.381s | 60.227s |
| Actual canonicalizer implementation edit | 305.355s | 143.894s | 453.473s |

The same public assertion edit previously took 167.790s. The earlier control
selected `--lib`; the candidate also executes the 18 native ABI tests. This is a
substantial observed codegen test-edit improvement, not matched full-gate timing.
The public assertion and fixture controls rebuild only the integration executable;
the private-unit control rebuilds only its unit executable; shared support rebuilds
both. Every test-only control keeps all 456 CLI build units fresh and the CLI
byte-identical. Positive production controls correctly rebuild six CLI units and
change its hash. A comment tests invalidation, not implementation compilation cost:
the separate implementation control replaces the canonicalizer's Option-to-Result
expression with an equivalent explicit match and costs 453.473s. No diagnostic
implementation change is adopted.

Controls are a recorded cumulative edit sequence, retaining prior controlled
inputs unchanged while measuring each next edit. The first controller stops
before shared-support mutation because its diagnostic matcher finds two strings;
it restores all sources. Its four completed guards remain passing evidence,
while the incomplete controller remains failed. The corrected continuation
resumes the exact last passing input state and executes only the remaining cases.
All source changes are finally restored and the production CLI is rebuilt to its
original SHA-256 `cd2e1db67f66131f1408871b8c2c4e2eebe7b1db3399e403e824dedcbea13756`.
The first type check's private-access errors remain preserved; their three mixed
modules stay unit tests rather than exposing private emitter APIs.

The final canonical policy guard passes in 230.822s, including 137.872s restoring
the production CLI. Formatting, file-size limits, method policy (6,458 sites and
235 unchanged node meanings), 18 method-policy self-tests, unsafe ABI policy
(410 sites/three partitions), filesystem effects (3,744 sites), Python runner
self-tests, emitted-Rust inventory, coverage matrix and profile checks pass.
The constituent reference scan includes relocated test callers; its regression
rejects removed or changed callers. All method classifications and owners are
preserved. A separate unprepared local Python self-test was cancelled when it
started compiler preparation; it is not counted as a pass. Prepared canonical
validation above supplies the completed evidence.

The 272-payload complete archive is SHA-256 verified:
`dev-verify-d16-qualification.tar.gz`,
`92ef330c99bd6192391db06e45b6099f1b52eb4a08d411713dd4de8f7954f2e4`.
PR [#4528](https://github.com/sifr-lang/sifr/pull/4528) merged as
`a7cae3e12add975767e3dea1e9f0c3aa6a191b35`. Independent Claude Opus review of
`e919b8f14e4f43a64138b1ce9488c82d88cad475` returned SATISFIED with no blockers.
The final qualification-record/README edits are documentation only; reviewed and
validated implementation inputs match. Review response SHA-256:
`7554e589d8f5fdc1b54efbfe02438e9b83dde00e2d6edf5a4794cf10a69dd30a`.
The unused shared helper and existing CLI lint warnings are deferred follow-ups,
not reasons to repeat completed qualification.

The 149.777s unit-target assertion and 453.473s implementation cycle remain above
the preparation design budget. The next macro item must address substantial
compiler recompilation and its test/CLI configuration boundaries; inspect the
measured units and coherent ownership before choosing crate extraction. Do not
keep optimizing the already short public-contract cycle. The 90s preparation
and 480s complete gate targets, complete phase gates, invalidation qualification,
and any unresolved coverage obligations remain open.

### D17 investigation — generated Rust compilation ownership

The D16 implementation edit rebuilds a codegen unit-test executable and two normal
codegen library configurations. The normal libraries use identical codegen features
and optimization settings but differ in nine direct dependency fingerprints.
This is an isolated codegen-test/CLI sequence: the canonical gate already groups
additive compiler packages. Do not claim the complete gate lacks that grouping,
or align unrelated feature configurations merely to force cache reuse.

The generated Rust normalizer contains 107 source files and 26,951 lines. Its
syntax transformations are coupled to adjacent generated-support filtering and
visibility rules, plus compiler-owned nominal naming and builtin-error catalogs.
Investigate a single generated-Rust responsibility with a narrow production API;
keep language identities in their existing authority and pass resolved names or
explicit catalogs across the boundary. Do not copy catalogs, introduce test-only
public APIs, or drag HIR/type-system dependencies into a purported syntax-only
owner. Existing production-workspace-inputs package ownership recursively respects
the dependency's production-inputs, so relocated tests must remain excluded.

A candidate must retain optimization semantics, every existing assertion and
native ABI check, complete coverage selection, source-bound identity, private
invariant tests, and byte-equivalent generated outputs. Reconcile test discovery
across owners. Compare complete retained implementation and test-edit cycles with
D16, not initial configuration preparation. Require a material measured saving
before adoption; use the 90s preparation budget to decide whether more work is
needed, and stop optimizing categories already within budget. Run applicable
policy/identity regressions and independent exact-candidate review. Full phase
coverage, ordinary invalidation scenarios and the 480s gate remain open.


#### D17 implementation — generated Rust syntax owner

The candidate extracts normalization, generated-support filtering, visibility and
macro-syntax invariants into `sifr_generated_rust`. Its production dependencies are
only Rust syntax libraries. Language nominal identities and builtin-error catalogs
remain in codegen/type-system ownership and are passed explicitly. The codegen
facade retains its public normalization contract. Both crates retain opt-level 2;
this is an ownership change, not a weaker compilation profile.

Private invariants move with their implementation; frontend-to-native integration
contracts remain in codegen. The expected original discovery union is unchanged:
1,220 codegen unit tests, 268 generated-Rust unit tests, and 297 public emission
contracts, plus the existing 18 native ABI checks. No test-only public fields or
syntax APIs were introduced. All 31 split support-filter test functions retain
identical assertion tokens and literals; four language-identity tests remain in
codegen and 27 syntax tests move. A task-local integration helper independently
checks the emitted declaration instead of accessing the syntax owner's private
parser fields.

All four delivery profiles retain their previous selections and enroll the new
crate alongside codegen. The additive compiler preparation/execution group,
coverage inventory, dependency-direction checks, semantic/native dependency graph,
method-authority scanner and generated unsafe-ABI scanner include the new owner.
Production input ownership recurses into the new package's declared `src` inputs;
its test-only files remain outside production identity.

#### D17 qualification — retained edit cycles and unchanged generated output

Rust implementation candidate `ca428c254534d1928f234224602a1bd681227bac`
passes the complete affected test selection, typechecking and production CLI build
in 260.846s. That candidate-preparation run is not a recurring speedup claim.
Every subsequent control executes all 1,785 original test identities and 18 native
ABI tests, with no ignored tests and an empty owned process session afterward.

| Retained scenario | D16 complete cycle | D17 complete cycle | D17 tests / CLI commands |
|---|---:|---:|---:|
| Unchanged | 25.205s | 25.196s | 20.605s / 0.568s |
| Private assertion edit | 149.777s | 29.208s | 24.393s / 0.659s |
| Normalization implementation edit | 453.473s | 226.822s | 207.261s / 15.603s |
| Emitter implementation edit | Not measured | 119.768s | 89.377s / 26.371s |

These are separate scenarios, not additive gate components. The private assertion
cycle improves 80.5%; normalization implementation improves 50.0%. The emitter
control has no matched before measurement. Cargo reports 186s test preparation for
the normalization edit and 69s for the emitter edit, before their respective CLI
builds. The 90s preparation budget remains unmet; the already short private-test
category needs no further optimization. Actual Cargo artifact evidence confirms
that the extracted production library is shared between tests and the CLI rather
than duplicated for their different codegen configurations.

All diagnostic source edits are restored. The private-test control retains the
original CLI bytes; both implementation controls rebuild and change the CLI.
Restoration reproduces its original SHA-256
`7cce985d797fc42d51681d26fa490b17a413ea2a330ac2a70494d19ec77c557d`.
Source comparison records 133 relocated files with identical tokens and ten
explicit boundary edits. All 31 split support-filter test functions retain their
tokens and literals. Six added test-helper method-policy sites remove none of the
previous 6,458; all 235 authority-node meanings remain.

The full policy sweep passes 14 checks, including formatting, size, method authority
and its 19 self-tests, dependency boundaries and negative controls, 14 execution
configuration tests, 11 build-identity tests, filesystem policy, emitted inventory,
coverage readiness and profile checks. Its unsafe inventory check correctly fails
on 16 code-proof paths for two moved tests. Data-only correction
`7dad324982ad3764708a1c39e699b697060716e5` fixes those paths without changing proof
text, fingerprints or Rust implementation. The original failed run remains failed;
the corrected inventory is checked separately. The six emitted plans (four delivery
profiles and create-PR/merge correctness scopes) preserve all previous selections;
only the new mandatory crate membership is added.

All 93 selected generated projects materialize successfully in 179.183s and compare
byte-for-byte against retained baseline `368ac774c1c2eaba6b910d388ac3d4e004086721`,
allowing only exact candidate revision substitution in Cargo.toml/Cargo.lock.
The first locked fetch fails because the candidate revision is not yet published.
After publishing the branch, recovery reuses the recorded trees and unchanged CLI;
it does not repeat materialization or reinterpret the failed fetch as a pass.
Recovery passes all 93 locked fetches in a 43.214s guard. The corrected unsafe
inventory passes remotely in a 28.209s guard, retaining all 410 sites and three
partitions. A structural comparison proves that the only difference from the Rust
qualified candidate is those 16 proof paths; the CLI still has the qualified hash.
All validation sessions end empty, with no reserve cancellation or unexpected
source mutation.

The complete 814-payload archive, including failed attempts and the 93 generated
trees, is verified locally: `dev-verify-d17-qualification.tar.gz`, SHA-256
`43508c09fbd70ee44d38295f9227d217fa8b46cad44d41b952431f9029df5de6`.
The remaining qualification-record update is documentation only; source and policy
inputs match their recorded candidates. Independent exact-candidate review remains
required before merge. Complete phase coverage, ordinary invalidation qualification
and the 480s gate remain open.


#### D17 independent review correction — preserve the emitted-source guard

Initial review of `a4d53785b9f4ae304ef78a8071751abf76928963` returns NOT SATISFIED:
the intrinsic-panic/source-emission guard still walks only codegen, dropping 16
matching production lines after relocation. No current forbidden emission was
found, but future detection coverage was lost. Candidate
`28828536aa46aedb1aafa87b9d6552e72b1d5763` restores the same scan and test-file
exclusions across both production owners. Two new regression tests reject the old
single-owner scanner and pass after repair. The canonical intrinsic-panic-lint
case, all 18 source-quality helper tests, coverage matrix and size guard pass in a
7.190s guarded session. Both codegen surface reproduction commands now select both
packages; their actual profile enrollment was already complete.

The 23-payload remediation archive is `dev-verify-d17-review-remediation.tar.gz`,
SHA-256 `f04c210beb87d9b25a32ef10cac4f5041952fef55fb46321a2f0b6e603c2aa29`.
No Rust/compiler/Cargo input changes in the remediation; retain the completed
1,803-test, edit-cycle, generated-byte, identity and unrelated policy evidence.
The source guard and its new tests are independently reviewed again at the final
candidate. The first review remains rejected evidence, not a pass.

Follow-up assigned to the DX metadata-inventory owner: the already stale
`internal_docs/compiler_dx_metadata_consumers.md` / `check_dx_metadata_inventory.py`
need reconciliation, including the new syntax owner and its three moved
project-support-pruning consumer sites. This predates D17 and is not selected by
its affected gate; do not silently repair that separate inventory here. The
reviewer's task-local parser suggestion needs no change: malformed syntax remains
covered by the relocated private invariant tests. Final phase timing and complete
coverage qualification remain open.


D17 merges in [PR #4529](https://github.com/sifr-lang/sifr/pull/4529) as
`caae77bc604bf90c576f34cfc315ba7824de7740`. Independent re-review of final candidate
`0bf712d31d0df4c0a2a2baed763932f44fadf0f8` returns SATISFIED with no blockers;
response SHA-256 `bf9caf429228785121dacb95ddaaaa79a0cf21458d58eebb8f52cd06fb675575`.
Both qualification archives and every payload hash are verified locally. This
closure record requires documentation checks only, not repeated implementation
review or compiler qualification.

### D18 — integrated discovery under normal allocation (prospective)

Refresh the complete `merge --correctness-only --no-fail-fast` selection after
D12–D17. The older D11 diagnostic explicitly forced `RUST_TEST_THREADS=1`, so its
83-minute aggregate cannot stand for current normal development latency. Remove
that diagnostic override for this run; let the existing canonical allocator choose
bounded ordinary test workers and nested build work. Keep the canonical single uv
entry, exact Node/npm and compiler tooling, owned retained targets, environment
identity, resource reserves and complete selection. No production scheduling or
profile change is proposed by this diagnostic.

Inspect the emitted plan and capacity before launch. Use the existing 1,750s
per-category diagnostic cap, failure continuation and dependency blocking. If the
aggregate preparation deadline leaves independent nodes unexecuted, recover those
nodes individually with the same cap and account for every consumer explicitly.
Do not repeat a failed preparation unchanged or classify missing execution as a
pass. Preserve the complete previous reports before any subsequent invocation.

Record preparation separately from execution, source/configuration identities,
actual outer and inner concurrency, coverage, cache state and the three largest
complete-category costs. Initial reconstruction after a new configuration is not
a warm optimization baseline. Use the current category budgets (90s preparation,
60s semantic/unit, 90s native corpus, 120s integrations, 60s installed product,
30s policy and 30s reporting) to pick the next macro repair. Keep already adequate
categories out of the optimization loop. This checkpoint is discovery; it does not
waive final gates, ordinary invalidation scenarios, independent clean-build or
sanitizer obligations, repeated timing qualification, or the complete 480s target.


#### D18 admission reconciliation — omitted consumer locations after D16/D17

The first normal-allocation discovery at
`07c113c25b1eb98f256e034aadcda0441d7da04a` stops at admission in 25.697s outer /
22.254s facade elapsed. Five early guards pass; DX metadata inventory fails.
Preparation and every consumer are blocked, not passed. The complete 17-payload
failure archive is `dev-verify-d18-admission-failure.tar.gz`, SHA-256
`1ed2c6d0f285287f38d42cf6665c4882a485c2c6855b7fe0d5a6e6316214bd1f`.
Preflight verifies all exact tool versions, eight effective CPUs, approximately
10.2GiB available memory and more than the required disk/memory reserves. Diagnostic
thread/job overrides are removed; the existing allocator selects the work budget.

Read-only historical reconstruction resolves the earlier D17 ownership uncertainty:
the inventory matches all 1,154 rows before D16 (`fa02de4d84`) but no longer matches
after D16 (`a7cae3e12a`), when discovery falls to 1,118 rows. The 36-row reduction
is a net count, not a complete enumeration of omitted symbols. This is this phase's
test-boundary integration omission, so reconcile it as a bounded admission repair;
do not delegate it indefinitely as an unrelated external failure.

The scanner must include each maintained owner's entire `tests` tree, covering
both private units and public contracts exactly once, and include the extracted
syntax owner. Preserve production/test classification and every payload-field/live
authority check. A new regression covers production, private-unit and public-contract
consumers in both codegen and generated-Rust owners without mocking the owner list.

The refreshed inventory contains 1,171 rows. Comparing symbol/operation/classification
multisets accounts for all three changed old keys: the already test-only
`stdlib_error` helper now has its correct test classification; the two canonical
stdlib-name operations delegate enumeration to `nominal_replacements`, with explicit
codegen and syntax-owner read boundaries. Fourteen newly discovered external-test
consumers are added, all classified as test setup/parity. Remaining changes are
relocated source anchors or unchanged classifications. No payload field or runtime
implementation changes. Validate inventory and its full self-tests, source size,
unchanged emitted selection, and independent review, then resume complete discovery
with all failed/blocked evidence preserved.


Admission candidate `211f508d19729d29ce85d624614936a6e13c0bb4` passes the actual
inventory, all 15 self-tests, file-size guard and profile validation in a 9.692s
session. Its emitted merge-correctness plan is exactly identical to the failed
sweep's selection. The new regression rejects the former discovery boundaries.
A one-to-one relocation reconciliation maps all 1,154 original rows to distinct
current rows, including split test helpers; the three classification changes above
are the only operation/classification changes. No Rust or Cargo inputs change.

Both the failed-admission archive and the 31-payload focused qualification archive
are SHA-256 verified locally. Focused archive:
`dev-verify-d18-admission-qualified.tar.gz`,
`99ad073aa62ae1140f82aeddc8b17548f9e374143010911a84e627d451f986d3`.
The final qualification-record edit changes only this phase document. Independent
review of the admission repair remains required before merging and resuming the
complete normal-allocation sweep. No main-test execution or full-gate timing is
claimed from admission-only checks.


D18 admission repair merges in [PR #4530](https://github.com/sifr-lang/sifr/pull/4530)
as `ac8b720c1d8b65519ef7e052f097bf1e8c4fe3ba`. Independent review of final candidate
`53f2706f7d4685c0efc0abafac384f0d474a81da` returns SATISFIED without blockers;
response SHA-256 `91d7dcb139874a52b8f169422dc3bf62daea3070e1617c33f63cffd3bd5ec9e0`.
The reviewer independently reconciles every old consumer symbol and runs all 15
inventory self-tests. Existing path-name classification and regex-operation
heuristics remain deferred metadata-inventory follow-ups, with no current missing
consumer demonstrated. This closure is a record-only update.

The complete normal-allocation discovery resumes at the published merge revision,
with unchanged compiler inputs and the same retained targets. All six admission
guards pass. The owned remote source remains frozen throughout the diagnostic;
its category results and final coverage status will be recorded after completion.
The retained failed-admission run remains failed and does not become a full-gate
pass through this repair.


D18 discovery r2 ends at the aggregate preparation deadline: 1,750.273s preparation,
1,794.393s outer elapsed, exit 124, no reserve stop, tracked mutation or surviving
owned process. Six admission steps pass, preparation fails by safety deadline,
and all remaining 31 selected steps are blocked. Main tests did not execute.
The 210-payload archive `dev-verify-d18-discovery-r2-guard-evidence.tar.gz` is verified
locally, SHA-256 `a50ee352c0b4092a401b4b6e5ee0ddc5cdf5b995136cfb859d63cabba19104ef`.

Completed preparation: workspace 0.686s, fixtures 0.072s, Python environment 0.088s,
generated projects 146.347s. Crate preparation is incomplete at 1,603.067s;
its shared compiler configuration completes in 204.885s, compiler services in
103.547s and driver library in 73.466s. The advanced-data native helper is capped
at 989.303s after the earlier commands consumed part of the aggregate allowance.
These nested child durations must not be added to the preparation total.

Read-only observations capture a native Cargo dependency check and a newly created
native family. Comparing the selected process environment with D8 finds PATH is
the only changed recorded environment field after tool restoration; native identity
explicitly includes PATH. This is initial configuration reconstruction, not a
retained-category regression measurement. Do not weaken native identity or reuse
old correctness evidence to conceal this difference. A scheduling lead is also
recorded: one sequential preparation helper receives the two-worker/two-job native
execution allocation despite the profile's four slots. Its possible benefit is
unmeasured and must not be assumed to explain the full delay.

Recover the failed crate-preparation node and each blocked independent node with
its actual canonical prerequisites and a separate 1,750s deadline, then resume the
full diagnostic selection. Retain the original incomplete run. The source remains
frozen at the published D18 merge; this record-only work does not alter it.


A subsequent read-only CLI identity probe confirms that changing only PATH changes
native identity and restores the historical `b233a905` identity with the old PATH.
However, its reconstructed current context produces `1998381f`, not the observed
fixture's `c421f889`: the exact active-context reproduction assertion fails. Preserve
that partial diagnostic and do not attribute the entire fixture identity difference
to PATH alone. This probe neither builds nor changes cached products.


D18 independent preparation recovery completes all 13 previously failed/blocked
nodes at the unchanged merge revision. Every guard ends with an empty owned
session, no reserve cancellation and unchanged source. Outer guarded durations
include prerequisite checks and must not be added to a normal gate total:

| Preparation recovery | Outer seconds |
|---|---:|
| Crate tests | 801.168 |
| Release compiler | 203.836 |
| Authoring | 30.205 |
| Tooling | 19.195 |
| Frontend syntax guardrails (`performance` preparation node; no benchmarks) | 14.192 |
| Oracle | 1.689 |
| Sysroot source | 2.181 |
| Sysroot package | 22.698 |
| Runtime/platform area graph | 231.940 |
| Fuzz/property area graph | 33.205 |
| SQL/platform area graph | 413.598 |
| Maintained demos | 9.189 |
| Compiler identity | 12.192 |

SQL preparation itself takes 403.680s; crate preparation itself takes 797.543s.
These successful setup recoveries do not establish that consumer assertions pass.
The complete 113-payload archive is verified locally:
`dev-verify-d18-preparation-recovery-evidence.tar.gz`, SHA-256
`ab23091e8afd4ccfb54227ae6fe724d9ba496f2e16c79ee087119a4d7d2b151e`.

With the failed discovery and all recoveries preserved, full discovery r3 starts
at the same frozen `ac8b720c1d8b65519ef7e052f097bf1e8c4fe3ba`, using retained
products and normal allocation. Its purpose is to execute the previously blocked
consumer categories and establish current costs. It is not final timing acceptance.


D18 r3 passes all 17 preparation nodes in 218.453s. Major retained setup costs
are generated projects 139.200s, crate tests 33.397s, sysroot package 16.137s and
SQL area graph 15.262s. Release compiler preparation takes 1.171s. These are
current retained observations, not an ordinary-edit or complete-gate qualification.

The continuing sweep exposes another integration omission in
`check_stdlib_native_intrinsic_allowlist.py`: `error_refs.rs` was counted through
an inline test's `CompilerIntrinsicId` reference before D16; after test relocation
it is stale as a production intrinsic owner. Seven runtime roots also disappear
from the scanner. Several remain genuinely emitted as structured `RustExpr::Path`
segments (including byte count, encoding and checked string padding); deleting
their allowlist records would conceal production behavior. D17's syntax owner
contains additional textual runtime references that the old single-root scan
omits. The repair must distinguish actual production references from test strings,
cover both applicable owners, and retain missing/stale/duplicate rejection with
negative controls. Do not broaden scans to tests merely to make old counts match.
Read-only triage is retained as `dev-verify-d18-intrinsic-discovery-analysis.json`.
Source remains frozen for the ongoing full sweep; no scanner repair is applied yet.


Further r3 discovery passes the complete generated-quality category (332.918s),
Python interoperability (582.296s), diagnostics (145.838s), runtime/platform with
sanitizer smoke (121.366s), algorithmic compatibility (7.420s), distribution
publication (139.312s) and installed/sysroot checks (398.855s). Python and generated
quality overlap under the existing 1+3-slot allocation; do not sum their durations
as elapsed time. The run is still incomplete at this checkpoint.

Generated-quality attribution: freshness 127.762s, corpus 61.836s, determinism
55.212s, demos 39.234s, inventory 21.498s, clippy 12.250s and native 11.539s.
These are the category's recorded variants, not extra cost outside its total.
Installed/sysroot attribution: metadata corpus 225.174s, boundary equivalence
125.382s, installed smoke 27.517s and structural metadata 20.596s. All four pass.
Python's largest suites are callbacks 158.505s and binding authoring 104.082s.
Within callbacks, the first CFFI case takes 82.678s and asyncio 38.910s; later
individual cases range from 5.194s to 12.481s. Do not call the whole Python category
fully warm: first-use native families were observed after environment restoration.

Two additional guards fail on the same test-relocation boundary: the pre-v1
regression guard references the former source-tree async-runtime test path, while
`check_split_brain_guardrail.py` treats relocated contract/support tests as
production parser/lowering consumers. The latter must recognize actual Cargo test
entrypoints and their owned module/include closure while still rejecting files
also reached by production. A blanket `/tests/` exemption is not an acceptable
repair. Batch these with the intrinsic inventory reconciliation after current
failure discovery. Keep production-source protection, historical regression
assertions, and executable negative controls. These failures do not justify
stopping independent categories or calling the full gate passed.


### D19 — restore guard ownership after compiler/test extraction (prospective)

Repair the three D18 integration failures as one bounded source-ownership item.
Keep D18's remote discovery frozen at `ac8b720c`; prepare this candidate in the
separate owned checkout. No in-progress diagnostic report is overwritten.

Recognize Cargo test targets and their module/include closure in the frontend
ownership guard, retaining production ownership whenever a file is also consumed
by a production target. Honor disabled automatic tests and explicit target paths;
prove that production includes, test-looking filenames and shared files do not
escape. Rebind the pre-v1 return-capture proof to its relocated test without
changing the required assertion. Reconcile the intrinsic inventory against both
production owners and structured runtime paths, retaining all real runtime roots
and missing/stale/duplicate rejection. Cover the extracted syntax owner in the
structured-Rust source guard too. Do not restore tests as production evidence.

Validate the affected guards and mutation/self-tests, unchanged profile selection,
file-size and formatting policies; qualify any production emitter edit against
all affected codegen/syntax tests and exact generated-output parity. Require one
independent exact-candidate review before the intermediate merge. All D18 failures
remain historical failures; final integrated coverage, ordinary invalidation and
90s/480s timing acceptance remain open.

D19 local policy qualification passes all 13 focused commands: each repaired guard
and its self-tests, method-authority inventory and mutations, file-size policy,
pinned Rust formatting, and diff checks. The exact emitted merge-correctness plan
matches D18. Cargo entrypoint tests include conventional and explicit targets,
custom crate-root sibling modules, disabled discovery, production overlap and
orphan-source negative controls. The padding emitter now supplies its complete
structured runtime path explicitly; it retains all four helpers and the existing
error conversion. Compiler tests and generated-output parity remain pending.
The method inventory update changes only affected source/context fingerprints and
bindings; site, node and relationship memberships and classifications are unchanged.
Local evidence is `dev-verify-d19-local-focused/result.json`, bound to file hashes.
The first expected method-inventory failure is not counted as a pass.

D18 r3 SQL discovery completes 72 variants: 68 pass and four provenance checks
fail. The independent clean-build protocol passes in 474.854s; the category totals
669.925s. PostgreSQL parser matrix takes 94.668s and native command tests 44.532s.
Read-only reconciliation shows all three maintained SQL component manifests differ
from current source inputs only in root `Cargo.toml` and `Cargo.lock`; component
source inputs are otherwise equal. The PostgreSQL baseline/submodule check passes
on the diagnostic checkout. These artifact-provenance failures belong to SQL
component qualification and need a separate bounded repair/requalification after
D19; do not rewrite hashes to bless stale products. The continuing main-test sweep
and final integrated acceptance remain incomplete at this checkpoint.

An independent D19 control using pinned Cargo metadata catches the Rust 2015
explicit-target auto-discovery rule before review. The guard now resolves inherited
editions and follows per-target-kind legacy defaults while honoring explicit
`auto*` overrides. Eight real temporary-package configurations exactly match
Cargo's test and production target paths without compiling. The affected guard,
self-tests, file-size and diff checks pass after this correction; prior unrelated
checks remain applicable because their implementation inputs are unchanged.
Control evidence: `dev-verify-d19-cargo-role-control.json`. Its first harness attempt
stopped on macOS temporary-path canonicalization before producing a verdict; that
incomplete attempt is retained and does not count as a pass.


D18 r3 completes in 4,690.690s (78m10.690s): 32 selected steps pass, five fail and
one is blocked. The crate category reaches its aggregate 1,750.546s diagnostic
cap; the native corpus is blocked by that safety stop. This remains a failed,
incomplete integrated run. Generated CLI passes 6/6; generated driver executes
68 passing assertions, one capped assertion and 11 unexecuted assertions. The
capped command exits for the category deadline, not an assertion failure.

A separate recovery at the same frozen revision executes all 12 remaining driver
assertions and all 732 native corpus fixtures successfully. Recovery takes
223.323s outer; the corpus command takes 187.594s, including 44.788s source
compilation. It does not turn the original run into a contiguous gate pass. All
owned sessions end empty, without reserve stops or source changes. Both archives
are verified locally, including every payload's size and SHA-256:

- Discovery: 6,854 payloads; `dev-verify-d18-discovery-r3-guard-evidence.tar.gz`,
  `0f8358bdf50ba1993fc601e68d96e314ce36e3c65f59fa1477cc02f18887e23e`.
- Recovery: 38 payloads; `dev-verify-d18-remaining-r1-guard-evidence.tar.gz`,
  `1f47a753fbd80d15e06a4ff7cfa2fc0b804dc73e6729eafaca22d2cae34a8015`.

One additional lowering assertion, `test_bare_deque_call_resolves_without_import`,
is still ignored because it needs the driver-loaded compatibility registry. Its
execution owner must be resolved before final complete-coverage acceptance.
This historical gap is resolved by D22 below; the D18 execution remains incomplete.

D19 merges in [PR #4531](https://github.com/sifr-lang/sifr/pull/4531) as
`a8fb4c04aa566294be0bbfd358a50fe6f341479d`. Exact candidate
`e9b10dde21e769a88b7945fc895ecbe7117a7117` passes all 1,803 affected Rust tests
(1,220 + 297 + 18 + 268), none ignored, and the padding runtime assertions. Padding
emitted bytes are identical to baseline. All 93 generated projects retain identical
file inventories and bytes, allowing only the expected candidate revision change
in Cargo manifests and lockfiles. Candidate qualification takes 231.347s outer;
materialization parity takes 113.754s. These include rebuilds and are not gate
performance acceptance. The 51-payload qualification archive is verified locally:
`dev-verify-d19-qualified.tar.gz`, SHA-256
`c08905d9e422eedf544f4d1759cd22d017b3b157ac10b0a5ad0940d95fea0021`.

Independent exact-candidate review returns SATISFIED, no blockers; response
SHA-256 `f1362d86fa3a1d86e0f00cc8fd66a72358a6e9e43374afb08c4f00d3eb590d38`.
Retained follow-ups: pre-existing module resolution for unannotated children of
`#[path]` modules; missing regression-proof files producing a raw I/O exception;
lexical rather than evaluated runtime-root discovery; diagnostics for invalid
inherited editions and unsupported unstable Cargo build forms. No current missing
production consumer was demonstrated. These do not change the final gate scope.

### D20 — CLI fixture ownership and native dependency reuse (prospective)

D18's lowering/CLI command passes in 441.843s. Its Cargo-ordered target summaries
attribute 111.47s to SQL native applications, 70.18s to build-output behavior,
39.88s to legal method names and 30.66s to runtime observability. These four targets
contain 48 passing assertions; 252.19s is their body-time sum, not a matched
optimization baseline. Source inspection shows fresh external fixture CWDs enter
`owner_scope()` and therefore select distinct native artifact families. Legal-name
fixtures additionally select fresh explicit cache roots. Repeated dependency
compilation was observed in separate legal-name fixture targets.

Keep unique per-case source projects, databases, generated roots and runtime
processes. Investigate placing behavior-only fixture directories under one owned
checkout root, using the existing retained native cache. Preserve production owner
isolation and every assertion. Do not share storage in tests whose contracts
require a fresh cache, permission boundary or lease. Preserve unrelated invocation
CWD checks and application/workspace discovery behavior. Do not globally redirect
temporary paths or introduce a compiler ownership override.

Before adoption compare the same complete four-target selection, including its
preparation, under matched pinned tools, retained targets, environment and the
canonical four-slot libtest allocation. Measure baseline, first candidate use and
retained candidate use; report reconstruction separately. Require all 48 assertions,
unchanged public profile selection, formatting/size checks, independent review and
no isolation failures. Investigate native family identities to corroborate the
mechanism. Adoption requires at least 25% lower retained complete affected cycle;
a 120s execution budget for this group is a provisional stopping point for this
item, not permission to exceed the overall 120s integration or 480s gate budgets.
If it does not earn a meaningful reduction, retain the evidence and reject it.

SQL component provenance requalification remains a separate owned repair. The
475s independent SQL clean-build obligation and final integrated coverage,
invalidation and timing qualification remain mandatory; no gate split is approved.


The first D20 candidate is rejected by its own unchanged assertions: placing
fixtures below the product checkout changes ancestor manifest discovery for
standalone cases. Noncanonical `..` components also correctly trigger cache-path
admission and SQL identity comparisons. Preserve that failed 130.264s run; it is
not a speedup. The next candidate creates one real, retained Git fixture repository
outside the product source tree, below the session's temporary root, and creates
unique canonical fixture directories within it. Git owns the shared native scope;
no fabricated marker, production owner override, manifest-discovery exception or
weakened assertion is introduced. Existing explicitly private cache controls remain
private. The same complete 48-assertion selection and adoption criteria apply.

The valid baseline executes all 48 assertions in 233.101s, 251.341s outer including
preparation. SQL companion executables needed 12.330s requalification after the
prior production compiler change; report this separately rather than inflate a
retained speedup. Initial diagnostic harness attempts missed the canonical
prepared-compiler/sysroot session and SQL companion preparation; preserve all
failures. After the existing SQL preparation script refreshes its companions, all
48 pass without remote source changes. Follow-up: make focused CLI companion
prerequisites explicit in the canonical preparation contract instead of relying
on a previously produced executable merely existing.


#### D20 closure — 2026-10-29

[PR #4532](https://github.com/sifr-lang/sifr/pull/4532) merges as
`9f721f1e7d2fe9f898d476e494fb18bd639fa23c`; exact reviewed candidate
`d2df51d91132646e032143088766ec1aca074617`. All 48 tests pass, with identical
test names, environment and actual four-worker allocation. Complete affected cycle:
251.341s baseline, 232.827s first use, 150.271s retained candidate (40.21% lower).
Comparing the complete candidate against baseline execution alone (233.101s)
still yields 35.53% lower time, exceeding the 25% adoption criterion. Execution
is 145.445s, above the provisional 120s group budget. A separate 165.781s result
used three workers and is functional evidence only. Family filesystem observations
corroborate reuse but provide a lower bound, not a complete execution census.

All 132 payloads in `dev-verify-d20-qualified-v2.tar.gz` are verified; archive
SHA-256 `f1f9f8dd4e09eac05a9a1b0caa6759b4a4372a0dfdaaff87f53a3faa01dec03e`.
Independent exact-candidate review is SATISFIED with no blockers, response SHA-256
`110c0fe9bfec6a3e29baf2a629b562980a7bb63bc837a6c8ff13387ad40b93c1`.
Formatting, source ownership, size policy and unchanged selection pass. The merged
Git tree equals the reviewed tree. Failed preparations and rejected candidates
remain preserved. Full coverage, invalidation and 90s/480s qualification remain open.

Follow-ups owned by fixture/preparation infrastructure: serialize concurrent
initialization of the same retained Git fixture owner; declare Git >=2.28 as a
prerequisite; declare SQL companion preparation for focused CLI tests; remove stale
inherited allocation metadata (both matched runs actually used four workers).
Do not weaken owner isolation or private-cache controls to address these items.

### D21 — SQL component artifact requalification (prospective)

D18 identified four failing qualification variants. Current source reconciliation
must confirm which component inputs changed. Rebuild affected MySQL, PostgreSQL
and SQLite artifacts with their canonical pinned producers; do not update hashes
without executing production. Preserve prior artifacts and actual producer outputs,
validate authenticated SDK and parser inputs, then transfer each output with a
SHA-256 check. Record byte equivalence or differences explicitly. The words fixture
is rebuilt only if its own declared inputs changed.

Run all affected component/provider contracts and mutation controls and the native
component consumers, with bounded resource guards and retained targets. Keep
source inputs frozen during production; allow only declared family outputs.
Require exact-candidate independent review before intermediate merge. This repairs
artifact correctness and makes no gate speedup claim. The independent SQL clean
build protocol, ignored deque assertion and final integrated qualification remain
separate obligations; no ordinary/expanded gate split is approved.


#### D21 closure — 2026-10-29

[PR #4533](https://github.com/sifr-lang/sifr/pull/4533) merges as
`992d16aa33ff7a4b56549be863f6a5a31a5ee4bb`; reviewed candidate
`61577a7894ddb69e4d8a9fa8c72df63fdc98986f`. Canonical producers run on frozen
`cf04295a853ee71946cb0bc952510ed6973f9185`: MySQL 28.311s, PostgreSQL all six
majors 233.431s, SQLite 19.294s. All ten binaries are byte-identical to the prior
artifacts. Each receipt changes only root Cargo manifest/lock hashes and aggregate
source digest. All 13 transferred outputs are SHA-256 verified. Words remains
unchanged and validates. No code, tool identity, profile or selection changes.

All eight component/provider contract and mutation commands pass. Specifically,
D18's four failures are `sql-component-qualification`,
`sql-component-qualification-mutations`, `sql-postgresql-compiler-qualification`
and `sql-postgresql-compiler-qualification-mutations`: the two shared component
variants check multiple families, explaining four repaired variants from three
receipt changes. Native consumers pass all 33 tests (17 MySQL, two PostgreSQL
executing six majors, 14 SQLite), zero failed, ignored or filtered. Guarded
qualification takes 80.233s including compilation. All producer and successful
consumer sessions end empty, without drift or reserve stops.

All 108 payloads in `dev-verify-d21-qualified.tar.gz` are hash/size verified;
archive SHA-256 `c6d66f18bbd3e877b6450ad336d7560c85622b5f8d58a2215af53e5ee39d456e`.
Review SATISFIED, no blockers; response SHA-256
`bc53621dd71bf46c0088441b5fb32c24c441be073ebfd0c05f9b562a89d04a91`.
The initial abbreviated-SHA consumer launch stops before tests in 0.687s and is
preserved as failure; diagnostic launchers should reject abbreviated input earlier.
Coarse root Cargo input invalidation remains a known producer-contract follow-up.
No gate speedup is claimed. Historical integrated failures remain failures; final
correctness, coverage, invalidation and timing qualification remain open.

### D22 — give the ignored deque assertion its frontend owner (prospective)

The lowering test `test_bare_deque_call_resolves_without_import` is ignored because
its source imports the driver-loaded standard-library compatibility surface. Move
its exact unannotated `deque([1])`, append and popleft source/assertion to the driver
frontend test owner, which supplies that registry through the canonical public
lowering API. Use a name that accurately describes imported constructor inference.
Do not add a registry substitute to the standalone lowering layer or alter product
behavior merely to satisfy the old placement. Remove the old ignored copy only
when the new ordinary owner executes successfully.

Require the focused assertion, its complete frontend group, the remaining lowering
ownership/async group, unchanged profile selection, formatting/size/source ownership
checks, and independent exact-candidate review. Verify the exact source string and
ordinary discovery; required execution must not be substituted with a skip. If a
real semantic failure remains in the correct layer, preserve it and diagnose its
owner before claiming coverage. This is a coverage repair, not a gate speedup.


#### D22 closure — 2026-10-29

[PR #4534](https://github.com/sifr-lang/sifr/pull/4534) merges as
`25da923a51657ec7cdeec06c1b2f4285bcb601ca`; exact reviewed candidate
`d60c0cb8e377fa1859e2dd325da23b05c453817a`. The previously ignored lowering
`test_bare_deque_call_resolves_without_import` is now ordinary driver test
`tests::single_file_frontend::test_imported_deque_constructor_infers_element_type`.
Its source, success assertion and message are identical; only the canonical
frontend owner, explicit test compiler context and accurate name change.

The focused assertion passes, then the full frontend group passes 33 tests and
remaining lowering ownership/async group passes 52. These are 85 distinct tests;
the focused repeat is not an extra unique test. None in these selected groups is
failed or ignored; other groups are explicitly filtered. Ordinary discovery includes
the new test, ignored-only discovery excludes it and lowering discovery excludes
the old copy. Focused command 56.551s includes rebuilding after D21 input changes;
assertion execution is 5.41s. Frontend group is 6.078s command / 5.53s execution;
lowering group 9.136s command / 0.02s execution. Total guarded 74.245s. No timing
improvement or full-gate acceptance is claimed.

Source parity, unchanged emitted plan, formatting, size and frontend ownership
checks pass. All 24 payloads in `dev-verify-d22-qualified.tar.gz` are verified,
SHA-256 `e1774f8400b8d846929898ec9b75e074fc1a0cb309c684aa5345201553db2657`.
Independent review is SATISFIED with no blockers; response SHA-256
`0492f4f6d1621a2d1382215e0008fa9f5655c7e262874d8ac16017262ef984e7`.
Future additions to the 803-line frontend test file should split by responsibility
before reaching the existing 900-line limit. This closes the named coverage gap,
not the final integrated coverage or 480s qualification obligations.

### D23 — integrated correctness and current cost discovery (prospective)

After D19/D21 repair the known integrated assertion failures, D20 removes repeated
CLI dependency builds and D22 restores the ignored assertion, run one complete
current merge correctness selection on a frozen candidate. Use canonical allocation,
retained targets, pinned tools and failure continuation. Record resource capacity,
target size, planned allocation and 20GiB disk / 2GiB memory reserves before work.
Preserve the 1,750s diagnostic category cap and independent category continuation.
If an aggregate crate deadline leaves later groups or the native corpus unexecuted,
freeze the original result, explicitly recover only those missing selections under
the same candidate with their own caps, and do not relabel the original as passing.

Report every terminal status, preparation separately, complete elapsed, overlapping
area groups correctly, and largest current subcategory costs. Retain all raw failure
and incomplete evidence before any later run overwrites canonical report paths.
Use ordinary telemetry, not tracing, for these elapsed measurements. First-use
reconstruction after changed compiler inputs is not a steady-state performance
baseline. Compare only input-compatible retained follow-ups when selecting an
optimization. Budgets remain 90s preparation, 60s semantic/unit, 90s native corpus,
120s integrations, 60s installed product and 60s policy/metadata, with the complete
ordinary gate at 480s. These are stopping/acceptance targets, not measured results.

The SQL independent clean-build and sanitizer obligations remain required. A
read-only lead finds both syn 2.0.117 and 3.0.5 compiled under a global syntax-library
optimization override, approximately 35s each in D10 clean SQL. Investigate actual
roles and complete-cycle impact before changing it; no profile or optimization
change is authorized by this observation alone. Prefer the largest demonstrated
repeated work after this integrated diagnostic. Full phase correctness, invalidation,
repeated timing and final merge qualification remain open.


D23 first discovery on `7b19e10c90cbd5d699c6912ba7f21872a6ba8668` stops at
metadata-inventory admission before preparation or main tests. Guarded elapsed
50.736s; five guard steps pass, one fails and all dependent work remains blocked.
The 20-payload failure archive is verified: `dev-verify-d23-discovery-guard-evidence.tar.gz`,
SHA-256 `c4bfbbb9ed79a1a246ff3bf1c2e9a592b45a328271adf7ed9f0cdea469f495ae`.
The optional recursive native-cache size census was separately stopped after more
than 80s; this is neither test execution nor a test timeout. Capacity admission
retains the measured 168GiB compiler target, approximately 33GiB free build storage,
216GiB free native storage and 10GiB available memory.

The admission mismatch is exactly nine Markdown line-anchor updates from D20/D22.
An ordered audit proves all 1,171 consumer identities, operations, classifications
and matching-anchor multiplicities unchanged. No new consumer was introduced;
the initial progress hypothesis is corrected by this audit. Refresh only those
anchors through the canonical generator, then run the actual guard and its 15
mutation/self-tests. These are record-only changes, not a policy relaxation or a
reason to rerun already-qualified compiler tests. Before retrying integration,
check the related source inventories too. Keep this first failed run immutable.


The related-inventory audit additionally finds D20 omissions before another sweep:
eight existing SQL fixture literal-comparison fingerprints changed with their
enclosing fixture context; three filesystem effects moved into the new CLI owner
helper; four manual create/remove calls disappeared when TempDir took ownership.
Retain all eight test-assertion/test-owner classifications and refresh only their
fingerprints. Classify the helper's two canonicalizations as tooling-input and its
create_dir_all as output-effect; remove only the four nonexistent old calls.
All other policy records remain byte-equivalent. The unsafe ABI inventory already
passes all 410 sites. Preserve these failed inventory checks. This reconciliation
changes policy records, so it requires affected guard/self-tests and independent
review, unlike the Markdown anchor-only refresh. Runtime code and test sources
remain the already-qualified D20/D22 inputs; no duplicate runtime qualification is
needed if an exact changed-path audit proves that binding on the final candidate.
For subsequent source-fixture edits, run these dependent inventories with focused
validation before review; do not rely only on frontend ownership classification.


#### D23 inventory reconciliation delivery — 2026-10-29

[PR #4535](https://github.com/sifr-lang/sifr/pull/4535) merges as
`2459704782a776d6e1302d830622c1f0cfe08c3b`; reviewed candidate
`0a507c18592e2952800c75221c4294ec1b606e60`. The merged tree equals the reviewed
candidate. All six affected guard/self-test commands pass, and independent review
is SATISFIED without blockers. Response SHA-256:
`b66192067f85a647b0b7cce2eb5cbbe511497a6fe9d4ff823bbda512f69f0cc6`.
The reviewer independently reruns all three actual inventory checks. Runtime,
compiler, tests, runner, profiles and locks remain unchanged from D22 qualification.

Follow-up owned by verification infrastructure: automate dependent inventory checks
for fixture-source changes so location/fingerprint reconciliation is not missed by
focused validation. The failed first discovery remains failed. Next run is
`dev-verify-d23-discovery-r2`, the full canonical correctness selection with failure
continuation and the same category/resource limits. This delivery closes only the
inventory repair; integrated discovery and final phase qualification remain open.

#### D23 dependency-inventory reconciliation (prospective)

The retry remains frozen remotely at `0e36952fb786d3b357002616842d90750f85a96d`.
All 36 completed top-level categories pass before crate execution. SQL completes
all 72 variants in 669.408s, including the mandatory clean/reused/independent-clean
protocol at 475.600s. These observations do not establish full-gate acceptance.

The additive Rust configuration subsequently fails 13 dependency-contract tests
at the same inventory admission: `crates/sifr_generated_rust/Cargo.toml` is absent
from the maintained Cargo inventory. This is an omitted dependent record from
D17's crate extraction, owned by this phase. Independent reconstruction from the
Git index and every maintained TOML manifest finds exactly one additional
manifest, four additional direct declarations, and no removed declarations.
The required totals are 161 manifests and 217 direct declarations. The new
prettyplease, proc-macro2, quote and tempfile owners use already-audited versions;
no dependency version, checksum, lockfile, pinned graph or audit policy changes.

Prepare the two JSON record corrections locally while the remote candidate and
its inputs stay unchanged. Continue independent test groups, preserve the failed
sweep, and archive its evidence before changing the remote source or overwriting
reports. Then run the entire `sifr_dependency_contracts` crate, including its
existing missing/extra/duplicate/input/checksum mutation controls, plus the
affected policy checks and file-size guard. Require exact-candidate independent
review under the approved intermediate policy. Reuse unrelated runtime evidence
only after proving that the final changed paths contain these two records and
phase prose exclusively; retain the original observed candidate identity.

The reconciliation does not eliminate any measured runtime cost or close the
480s goal. Do not rerun the full sweep solely for these record updates. Finish
coverage reconciliation, then choose the next macro improvement from the current
preparation and execution measurements. Complete final phase gates, invalidation
and repeated timing qualification remain required.
