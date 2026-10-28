# Ad Hoc Development and Verification Boundary Redesign

- Phase ID: `DEV-VERIFY`
- Status: in progress; D1 verification test ownership
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
