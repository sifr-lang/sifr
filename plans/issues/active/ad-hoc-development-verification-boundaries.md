# Ad Hoc Development and Verification Boundary Redesign

- Phase ID: `DEV-VERIFY`
- Status: in progress; D1, D2, D4 and inventory reconciliation delivered; D3 integrated discovery
- Latest delivery: D5 bounded ordinary driver concurrency; D6 production-input ownership next
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
