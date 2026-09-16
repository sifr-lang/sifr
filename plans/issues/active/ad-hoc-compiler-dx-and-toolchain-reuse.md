# Phase DX: Compiler Developer Experience and Toolchain Reuse

status: planned  
design status: final  
phase-id: DX  
implementation baseline: `0f819c2f04bf5b2891074c55ba26369ddf4f13bd`  
architecture: [Compiler DX and Toolchain Reuse Architecture](../../../internal_docs/compiler_dx_architecture.md)  
execution model: sequential milestones; one bounded implementation item at a time

## Objective

Deliver the complete compiler-DX architecture: a versioned prebuilt stdlib with lazy semantic and emitted-code loading; reproducible, reusable native builds; persistent project checking; correct bounded editor sessions; and a local-first verification loop whose failures, cache behavior and performance are explicit.

The phase is complete when all required capabilities work through the ordinary installed CLI and existing LSP, preserve Sifr semantics and trust, pass the failure-injection matrix, and meet the declared supported-machine qualification. Project-level persistent compilation state is included in the required scope. Measurements establish implementation efficiency and remaining work against the completion criteria.

The phase identifier is `DX`. This file follows the repository's ad hoc phase location and milestone/definition-of-done structure. DX.1 registers the phase in both `plans/roadmap.md` and `plans/phases/index.md`, with links to this canonical phase.

## Source Of Truth

The architecture document owns behavioral contracts and the canonical acceptance matrix. This phase owns milestone scope, dependencies, status and completion evidence. A milestone cannot silently weaken the architecture; an architecture change cannot silently mark a milestone complete.

Existing subsystem documents remain authoritative for language/runtime semantics, Cargo/package trust, Python/SQL integration and release authorization. Milestone DX.1 explicitly reconciles the earlier Phase 35 process-local-only deferral and the old size-based cleanup policy. It does not rewrite historical evidence or create an alternative checker.

## Prerequisites

Before implementation, establish an owned worktree/branch/index and an exact base SHA. Read current `AGENTS.md`, the phase-closure skill, the architecture, and the code paths relevant to the selected milestone. Determine which previously identified defects are already fixed on the actual base; do not repeat merged work or treat a stale issue label as an active blocker.

Use the repository's selected Rust/Python/verification tool versions and required dependencies. Record the effective toolchain and cache/storage locations. The Linux i7-4720HQ / 12 GB reference remains a supported floor; a larger machine is not a prerequisite. A writable qualified local cache filesystem and enough owner-scoped storage for the selected validation are required. Network preparation, when necessary, stays explicit and separate from offline assertions.

No prerequisite requires a compiler daemon, another query engine, project-cache compatibility with older compiler builds, actual Marketplace publication, a pushed local candidate merely to validate local dependencies, or permission to mutate another session's artifacts.

## Depends On

- Existing Phase 35 frontend/query/cache ownership and Phase 36 analysis/LSP boundaries.
- Current checked-source stdlib, sysroot distribution, runtime and package/interop contracts.
- Current native generated-code and runtime-safety guarantees.
- Existing verification profiles, fixture inventory and phase-closure workflow.

These are integration dependencies, not a request to reimplement or requalify completed phases wholesale. Related known false-pass paths must be confirmed repaired or corrected before the new affected evidence is trusted. Externally owned unrelated failures remain with their owning issue.

## Feeds Into

All ordinary `check`, `build`, `run`, `test`, `emit`, formatter/lint and editor workflows; packaged toolchain install/update; future language/framework work that needs reliable feedback; and stable release qualification. This phase does not make new product-language or publication-governance decisions.

## Strict Non-Goals

- A replacement parser/type checker/semantic engine, public tsserver-like protocol, or mandatory compiler daemon.
- A general remote build/cache service, shared cross-user executable cache, or universal precompiled runtime binary matrix.
- New Sifr language semantics, new supported host/target platforms, new SQL/Python trust models, or weakened native/runtime assertions.
- Arbitrary expression-level persistent incrementality or broad data-structure rewrites without the module-level contracts and measurements.
- Recursive review/closure ceremonies or duplicated current-status/evidence ledgers.

## Architecture Ownership

Use the architecture's ownership table. In brief: source/syntax own text and ranges; lowering/type-system/IR own semantics; frontend owns dependencies and reusable result families; sysroot owns toolchain/metadata transport identity; driver owns producer/build/storage orchestration; codegen owns emitted payloads and transformations; analysis/LSP own existing editor layers; package owns dependency/trust/environment resolution; `sifr_verify` owns preparation/assertion selection and reporting.

Do not create reverse crate dependencies merely to share a persistence helper. Inject a store/provider through the lower-level owner contract. Rust compilation stays in Cargo, and the existing Python verification entrypoint remains the orchestrator.

## Global Execution Rules

Milestones execute in the order below. An implementation milestone may be split into bounded PR items, but each item retains the same milestone scope and completion criteria. Finish the selected item before starting another. All milestones are required for phase completion.

Run focused tests while developing and the appropriate existing create-PR/merge validation on the final candidate. Group related in-scope fixes before broad qualification. A failed prerequisite blocks dependents; independent diagnostic collection may continue only when safe. Do not repeatedly run an unchanged failed performance gate without new evidence or a controlled measurement reason.

Every milestone includes its own regression tests. Later consolidation/qualification milestones do not excuse shipping an earlier untested cache or identity boundary. Existing required assertions remain selected during migration; replacements must show equivalent coverage before old paths are removed.

Use the existing phase-closure skill: scoped review against exact base/candidate and acceptance criteria, final evidence outside the approved Git tree, and one compact record after merge. Do not rerun broad gates or external review for record-only changes. An unrelated base change does not invalidate unchanged relevant evidence; implementation/validation-input changes do.

## Milestone Sequencing

```text
DX.1  Baseline, policy and separate compiler measurement lanes
  -> DX.2  Entrypoint identities, test identities and toolchain context
  -> DX.3  Entry storage, process ownership and early DX fixes
  -> DX.4  Shared fixture, diagnostic and reporting integration
  -> DX.5  Stdlib indexed type/payload schema and bounded decoder
  -> DX.6  Canonical metadata producer and direct-test preparation
  -> DX.7  Layered externals and lazy CLI/LSP/codegen consumers
  -> DX.8  Full metadata and installed-toolchain qualification
  -> DX.9  Compatible native reuse and finalized artifacts
  -> DX.10 Application profiles and compiler feature groups
  -> DX.11 Editor snapshots, recovery and measured memory reduction
  -> DX.12 Semantic dependencies and reusable result families
  -> DX.13 Transactional cross-process project reuse
  -> DX.14 Proven interface propagation and editor persistence integration
  -> DX.15 End-to-end performance and distribution qualification
  -> DX.16 Closure and authoritative handoff
```

The early order removes observed operational costs before adding project persistence. Fixture/reporting integration precedes metadata and cache migrations. Project-generation inheritance/GC is introduced with its actual DX.13 consumer, not as a speculative DX.3 framework. Disk reuse follows measured metadata/native/editor improvements, but remains part of this phase's full required result.

### Release-eligible checkpoints within one required phase

After DX.8, the metadata improvement may be released when its exact installed artifacts pass the existing applicable package/publication qualification. After DX.11, native/profile/editor improvements may be released on the same basis. Neither checkpoint marks Phase DX complete, advertises unimplemented project persistence, changes release authorization, or requires additional duplicate reviews/gates for unchanged evidence. They are integration and release opportunities, not optional scope decisions.

Continue DX.12–DX.14 in sequence using the recorded checkpoint measurements as the comparison baseline. Measurements guide implementation and optimization of the required persistent capabilities. Final phase closure follows DX.15/DX.16; independently qualified user-visible improvements may ship at the checkpoints.

## Milestones

### milestone_dx_1: Baseline, Policy and Verification Contract

**Depends on:** prerequisites.  
**Primary owners:** verification, compiler/toolchain maintainers.  
**Architecture:** sections 1–3, 11–15.

**Scope**

- Register an ad hoc `DX` row in both `plans/roadmap.md` and `plans/phases/index.md`, and link the architecture. Record the actual implementation base, selected commands, owned paths and related already-fixed regressions.
- Explicitly adopt the new source-tree versus installed-metadata behavior, optional project-cache miss behavior, pressure-based cleanup direction and functional/performance/safety distinction. Update conflicting planning/rule documents in the same planning change rather than relying on agent interpretation.
- Add the `product-installed-optimized` and `contributor-dev` compiler measurement lanes. Select actual artifacts and record compiler build profile separately from generated-application profile and verification selection; remove hardcoded profile labels from reusable harness paths.
- Capture baseline workloads for fresh checking, unchanged/edited sessions, native first/no-op/rebuild, bare Cargo tests, LSP demanded loading/retention and contributor profiles. Product targets time the installed optimized executable, not `cargo run`. Add minimal permanent timing/cache-decision instrumentation without a second reporting framework.
- Classify legacy `budgets.json` entries by host/profile/workload. Archive superseded Mac/dev values as historical anchors for replacement lanes, retain valid active scoped guards and product limits, and record a prospective mapping. Do not blanket-disable existing budget enforcement.
- Freeze every current fixture's assertion depth and application profile before defaults can change. Freeze a paired demanded-stdlib steady/peak RSS workload and noise policy for the optimized installed LSP.
- Freeze stable benchmark/fixture IDs and the applicable host/resource/product contracts before optimizing. Record source, compiler, toolchain, profile and cache conditions.

**Definition of done**

- The current baseline and source-derived defects are separated from unmeasured target behavior; no improvement is claimed from old session numbers.
- Every existing selected assertion/configuration/profile is inventoried so later grouping and migration can prove coverage preservation. A dev-versus-product or Mac-versus-Linux comparison is rejected as incomparable, while a seeded valid same-lane regression still fails.
- Policy changes are prospective: old failures remain failures under their recorded policy, and no correctness/trust invariant is waived.
- The 12 GB workload and numeric product-target scope are explicit; no larger-machine prerequisite or opportunistic baseline relaxation is introduced.

**Validation and evidence**

Run relevant documentation/schema checks, baseline report validation and seeded performance-policy failures. Store baseline artifacts outside the reviewed source tree, referenced by digest and command. Complete Q07 and Q08 for lane/policy selection and establish fixed inputs for Q09. Required later cases include Q02, Q03 and Q06; this milestone establishes their fixed inputs rather than claiming the optimized outcomes already pass.

### milestone_dx_2: Entrypoint Compiler/Test Identities and Explicit Toolchain Context

**Depends on:** DX.1.  
**Primary owners:** compiler build, sysroot, driver/package.  
**Architecture:** sections 4 and 8.1.

**Scope**

- Embed the application-wide identity in `crates/sifr/build.rs` and inject it through immutable outer compiler contexts. Do not place a workspace-volatile identity in a common library or per-query global lookup.
- Implement canonical input tracking, including dirty content, inventory changes and submodule/generation inputs. Establish the dependency-scoped compiled-token composition for bare library tests described in architecture 4.2.1; no constant test identity or live-source-only surrogate is accepted.
- Inventory every application, direct library-test and integration-test context constructor. Keep test metadata identities separate from product identities, with complete configurations and explicit rejection of stale overrides.
- Define distinct compiler, semantic-target, stdlib-input, metadata, native and validation identities. Avoid recursive self-hashes and whole secret-bearing environment hashes.
- Establish the resolved native toolchain/context contract and thread it through existing Cargo/probe entrypoints without changing application profile defaults yet.
- Replace runtime dependence on ambient toolchain choice for internal temporary projects; retain intentional supported user/workspace configuration as recorded effective inputs.

**Definition of done**

- Two behaviorally different local compiler builds with unchanged package version/HEAD cannot share an incompatible compatibility ID.
- Source-archive builds and added/deleted compiler inputs participate in build tracking; installed checks do not hash the current executable or scan source trees.
- Toolchain/context selection is explicit and inspectable. Identity computed by one path is the same identity consumed by preparation, checking and artifact reports.
- Actual Cargo rebuild evidence shows an LSP-only edit does not recompile frontend/sysroot crates solely because an aggregate identity changed. Genuine dependency rebuilds remain expected.
- Binary package digest is separate from semantic/wire compatibility; presentation-only choices do not unnecessarily invalidate semantic work.

**Validation and evidence**

Complete I01, I02, I07 and I08, plus focused tests for explicit toolchain overrides, unavailable tools, deterministic identity encoding and secret redaction. Prove build-script identity regeneration behavior using actual rebuilds, not only unit tests on a hash helper. Record the effective invocation context and identity-generation overhead.

### milestone_dx_3: Entry Storage, Process Ownership and Early DX Fixes

**Depends on:** DX.2.  
**Primary owners:** driver storage/execution, verification, CLI diagnostics.  
**Architecture:** sections 7, 11 and 12.3.

**Scope**

- Move reusable generated storage to the persistent user/worktree cache root and move its staging beneath the destination filesystem.
- Harden current generated-artifact entry publication and concurrent-winner validation using per-entry/family OS locks, immutable final entries and conservative active-reader cleanup protection. Exercise real existing artifact consumers, not only empty scaffolding.
- Do not build project-record inheritance, hard-link promotion or project-generation GC here; DX.13 introduces them with persisted semantic consumers. Entry-level safety cannot be deferred because current native/test artifacts already use shared storage.
- Replace the 20 GiB automatic cleanup rule with owner-scoped pressure/obsolescence cleanup and dry-run reporting; protect active toolchains, candidate outputs and shared/native roots.
- Centralize subprocess outcomes, bounded stdout/stderr, safety deadlines, cancellation/descendant cleanup and runner-owned events. Add actionable existing sysroot/toolchain help and initial doctor/cache/timing surfaces.
- Separate generic correctness-step safety deadlines from named performance contracts under the DX.1 policy. Fix related harness false-pass/report contamination defects before using affected results.

**Definition of done**

- Cache migration works when TMPDIR and persistent storage are on different filesystems.
- Incomplete/concurrent artifacts are not trusted merely because a directory exists; entry pruning cannot remove actively used state. Project-generation inheritance/GC is explicitly not claimed implemented.
- A cancelled/timed-out command retains its original cause and partial streams and leaves no owned descendants mutating deleted/reused directories.
- Child text cannot set aggregate status; warm/cold outcomes and functional/performance results remain distinct.
- Large but useful targets are retained when space is sufficient. Resource failure reports identify the selected reserve/scope and next action.

**Validation and evidence**

Complete C01–C03, C05–C07, B10 and R01–R03 using real subprocess and filesystem failure injection. Include positive pressure cleanup and negative active-reader tests on existing artifact entries. These same primitives are extended and requalified with semantic generations in DX.13. Doctor's full metadata-generation coverage is completed in DX.8; this milestone must already improve current toolchain/setup errors. Capture before/after cold preparation and cleanup behavior without relabeling the old evidence.

### milestone_dx_4: Shared Fixture, Diagnostic and Reporting Integration

**Depends on:** DX.3.  
**Primary owners:** verification, diagnostics, codegen/IR, analysis.  
**Architecture:** sections 11–13.

**Scope**

- Consolidate fixture discovery, artifact preparation and case reporting through adapters over existing area manifests, preserving check, Rust-check, link, run, snapshot and quality assertions. Do not replace all suites with a new general test platform.
- Record application profiles and actual assertion selection in each native/runtime case. Preserve the baseline's release assertions before changing any CLI default.
- Add comparison/runner hooks for later source-versus-metadata and repeated-root tests. Exercise the existing paths and independent negative controls now; later payload/reader milestones must activate and qualify their own paths, not inherit an unearned pass from an empty hook.
- Make representative native/codegen coverage routine in create-PR and retain the appropriate full merge/release coverage. Preserve docs/demo companions that have an explicit user/audit purpose.
- Implement focused failure selection and no-fail-fast independent collection; move cheap inventory/contract checks ahead of expensive preparation where possible.
- Complete diagnostic source attribution, root-cause/error-recovery behavior, tested help/explain examples and apply-fix/recheck tests.
- Retire duplicate binary resolvers/report authorities after coverage parity is demonstrated.

**Definition of done**

- A negative fixture cannot pass because setup failed, and no speed flag downgrades a required link/runtime assertion.
- Cached binaries/metadata still feed actual selected assertions; original failures and case diagnostics remain available.
- Independent failures are collected and dependent cases explicitly blocked; no swallowed timeout or child-text-induced status exists.
- IR invariants and independent expected outcomes supplement source/cache equivalence.
- Record-only updates preserve valid unchanged-input evidence without a new broad gate or fabricated run.

**Validation and evidence**

Complete E07, R04, R05, R07, R08 and the existing-path Q01 negative controls, and rerun all relevant earlier harness seeds under the consolidated entrypoint. Diff the before/after assertion inventory. Require tested error-document examples, safe code-action results and per-case native/link/runtime stage outcomes. Reapply these checks when metadata and project persistence are connected. Streaming failures and no-fail-fast collection must already be usable by DX.5 onward.

### milestone_dx_5: Stdlib Indexed Type/Payload Schema and Bounded Decoder

**Depends on:** DX.4.  
**Primary owners:** sysroot, IR/type system, driver stdlib, codegen.  
**Architecture:** sections 4.3 and 5.

**Scope**

- Inventory every consumer of external stdlib definitions, HIR inventory, defaults, const/descriptors, generic/class templates, Rust support, source maps and interop summaries.
- Define the indexed private container and explicit serialized record families. Include per-module emitted Rust plus validation identity, not only semantic signatures.
- Implement indexed type/declaration/string tables: signatures refer to type IDs, nominal instances to declaration/type-argument records, and type parameters to binder-scoped identities. Retain shared decoded handles; do not serialize a full `Type::Class` in every signature.
- Define stable-ID encode/decode/re-intern rules, target/source identities, canonical ordering, reference checks and source-relative navigation data. Preserve permitted recursion and distinct nominal/partial-view identities without changing type semantics.
- Produce the exact read/mutate/clone/lifetime migration inventory for `ExternalDefs` and codegen consumers, including the driver project/export functions and frontend/lowering/analysis/LSP sites. DX.7 implements the layered view before enabling the metadata provider.
- Implement bounded header/index/record decoding and deterministic encoding independently of whole-live-object serialization.

**Definition of done**

- Every current payload consumer has an owned migration classification; no required semantic field is silently omitted.
- Artifact-local IDs preserve nominal/package identity independently of allocation order. Portable payloads do not embed producer paths or incidental map order.
- Unknown/incompatible/malformed inputs fail at the appropriate bounded boundary with actionable errors, not panics or excessive allocation.
- The format distinguishes semantic bodies, reusable Rust and application-dependent templates, and does not claim a fragment validation record proves final program validity.

**Validation and evidence**

Complete I05, I06, M06 and M16; add codec round trips for all record families, generated valid-record property tests and fuzz seeds for invalid lengths/offsets/IDs. Show deterministic payload hashes for repeated production input records, and type-table size/decoded-retention scaling for many signatures referring to one large class. Include same-name/different-package, binder identity, partial type views and valid recursion. These tests are prerequisites to exposing the decoder to normal commands.

### milestone_dx_6: Canonical Metadata Producer and Direct-Test Preparation

**Depends on:** DX.5.  
**Primary owners:** driver/CLI, verification preparation, distribution scripts.  
**Architecture:** section 6.

**Scope**

- Implement `sifr sysroot build-metadata` through the existing source frontend and codegen; it must operate without a preexisting metadata artifact.
- Implement one source-tree write-through ensure operation used by direct development launch, verification preparation, packaging and bare `cargo test -p sifr_driver`.
- Add explicit test-context setup using DX.2 test identities. On a miss, bare tests call the already linked canonical producer library function; no preexisting CLI binary, recursive Cargo subprocess or producer-in-build-script is allowed.
- Coordinate threads/processes by a process-local keyed success store and an OS per-key ensure lock with double-checked entry validation. Test identical-family waiters, distinct configurations, cancellation and producer death. Preserve failure causes without poisoning later attempts.
- Derive complete stdlib/input/target identities, emit semantic and codegen payloads, validate output, and publish under the durable ownership protocol.
- Wire packaging preparation to produce matching metadata before ordinary consumers switch in DX.7; preserve host/target execution distinctions and exact output identities.

**Definition of done**

- The producer never runs recursively from compiler `build.rs` and does not introduce fallback signature tables or a separate checker.
- Missing/stale development metadata is regenerated through the primary ensure path; matching metadata is reused without source bootstrap.
- Producer failure is preserved. Source mutation cannot publish an artifact falsely labeled for different live inputs.
- Normal installed mode has no source fallback. Development mode and artifact-output location are explicit and testable.

**Validation and evidence**

Complete M09, M12–M14 and R10; add end-to-end producer tests over the canonical inventory and a test that begins with no metadata file. Exercise read-only source with writable cache and missing/unwritable output diagnostics. Record producer time and output identities once per required configuration, including bare-test empty-cache/prepared-cache and N-process ensure measurements. Prove at most one successful production for a simultaneously requested identical key and complete independent assertion execution. Different compiler/test configurations must not be incorrectly forced onto one artifact.

### milestone_dx_7: Layered Externals and Lazy CLI, LSP and Codegen Consumers

**Depends on:** DX.6.  
**Primary owners:** driver stdlib, frontend/lowering, codegen, analysis.  
**Architecture:** sections 5.2–5.4 and 10.

**Scope**

- First refactor flat `ExternalDefs` into an immutable baseline view plus mutable project overlay at the existing semantic owner. Update `lower_module_with_externals_name_and_options`, sibling frontend entrypoints and all inventoried map readers/mutators; export collection writes only the overlay.
- Test lookup/visibility parity, failed/deleted module export invalidation and two-project isolation over the source-backed view before enabling metadata. Avoid adding provider I/O to lowering or sharing context-specific mutations.
- Then introduce metadata-backed demand-driven semantic/support providers and route all ordinary CLI, bare-Cargo-test, frontend/analysis/LSP and codegen consumers through them. Use shared indexed type records rather than eager full-class projections.
- Replace whole-stdlib cloning with shared immutable decoded values and per-compilation overlays. Use indexed dependency/interop summaries instead of all-body scans.
- Load selected semantic bodies for checking and selected Rust/templates for building; remove repeated whole-stdlib emission/syntax bootstrap in installed paths.
- Preserve validation for transformed/specialized/final assembled Rust. Keep the source-built reference provider for qualification, not as an installed fallback.

**Definition of done**

- Tiny installed checks and syntax-only operations do not reconstruct unrelated stdlib state.
- Generic/default/const/descriptor/interop-heavy programs retain their semantics and required codegen information.
- `check`, `emit`, `build`, `run`, test frontend paths and the existing LSP use the same metadata/semantic authority.
- No blanket validation bypass is introduced. Lexical joins, changed templates and application-dependent generation remain checked.
- No unqualified global stdlib result slot aliases different toolchain generations or permanently retains transient errors.

**Validation and evidence**

Complete M01, M02, M04 and M17; assert decoded-module/payload counters, not just elapsed time. Run source-versus-metadata diagnostic/codegen comparisons on representative fixtures and actual native tests. Capture fresh-check and paired optimized-installed LSP RSS/retained-allocation measurements under DX.1 inputs (Q09), separately from dev/test compiler measurements. Preserve emit preamble markers in representative M15 comparisons before the full corpus in DX.8. Full target/corpus qualification follows immediately in DX.8.

### milestone_dx_8: Full Metadata and Installed-Toolchain Qualification

**Depends on:** DX.7.  
**Primary owners:** verification, sysroot, distribution, source navigation.  
**Architecture:** sections 6.3, 11 and 13.2–13.3.

**Scope**

- Enumerate and structurally decode every canonical module/section and validate references/templates/emitted payloads for each required target configuration.
- Run the full maintained corpus through source-built and artifact-loaded providers, comparing exact emitted bytes under identical inputs and retaining native/behavioral oracles.
- Qualify packaging, relocation, pinned live generations, upgrade/rollback and two compatible stores in one process. Exercise cross-host portable payload reproducibility only under equivalent target/semantic inputs.
- Complete doctor metadata identity/integrity reporting and actionable installed/development mismatch diagnostics. Wire the full structural and selected behavioral tiers into the existing profiles.

**Definition of done**

- Qualification uses the exact metadata bytes shipped/consumed; no post-test regeneration silently replaces them.
- Ordinary E2E remains lazy while explicit metadata validation covers the entire inventory. API/behavioral coverage is separately enumerated and not inferred from decoding.
- Cross-target differences are not misclassified as nondeterminism; compatible cross-host portable payload comparisons are deterministic.
- Installation moves and live updates cannot mix old semantic records with new sources or codegen payloads.

**Validation and evidence**

Complete M03, M05, M07, M08, M10, M11, M15, R09, Q05 and the metadata-stage Q09 comparison. Include actual installed-layout commands, public `sifr emit` stdout and marker-dependent source maps rather than only internal strings/source-tree calls. Retain an artifact/source/coverage manifest and per-target qualification results; do not claim every possible generic instantiation is exhaustively covered.

**Checkpoint:** the exact metadata-enabled installed toolchain may be independently released after its applicable release qualification. Record only implemented claims and existing authorization; Phase DX and project persistence remain incomplete. This checkpoint is not another automatic whole-gate/review cycle over unchanged evidence.

### milestone_dx_9: Compatible Native Reuse and Finalized Artifacts

**Depends on:** DX.8.  
**Primary owners:** driver, package/native/Python integration, verification.  
**Architecture:** sections 8.1–8.3 and 8.6.

**Scope**

- Establish compatible Cargo storage families without mixing compiler-development targets with incompatible generated source/probe contexts.
- Use stable editable-project identities and input-bound immutable fixture/probe roots. Serialize source mutation and artifact capture; avoid root binary-output aliasing. Introduce the real native-family/read-execution ownership required for these consumers, reusing DX.3 primitives without implementing project-generation inheritance early.
- Let Cargo establish native freshness on ordinary requests; consume structured output paths and publish independent finalized Sifr artifact bundles.
- Preserve current trust, build-script/native/environment dependencies, local unpushed source validation, explicit portable-export freezing and actual Python loader selection.

**Definition of done**

- Unchanged generated inputs are not rewritten, and repeated prepared requests avoid unnecessary Rust compilation/linking.
- The returned runnable artifact is not a mutable shared Cargo output path. Native failure never implicitly runs an old executable.
- Shared dependency reuse does not hide a deliberately corrupted same-name root or lint case.
- Native artifacts invalidate when the actual toolchain/source/environment inputs change; package trust and runtime checks remain current.
- Portable exports have a qualified loader/dependency contract rather than undocumented developer-machine paths.

**Validation and evidence**

Complete I04, B01–B04 and B07–B09. Use the real two-root error/lint regression, concurrent mutation/output capture, native link failure and Python loader mismatch cases. Measure actual rebuilt units and cold/no-op/edited build durations before and after; invocation-count reduction alone is insufficient.

### milestone_dx_10: Application Profiles and Compiler Feature Groups

**Depends on:** DX.9.  
**Primary owners:** driver/runtime, verification, package.  
**Architecture:** sections 8.4–8.5 and 12.1.

**Scope**

- Implement explicit development/test/release generated-application profiles throughout commands, manifests, effective configuration, artifact identity and reports. Keep the distributed compiler itself optimized.
- Pin required overflow/debug/panic settings as specified by the architecture and validate effective overrides. Do not rely on debug assertions for supported-program semantics.
- Consolidate compatible compiler integration build/test selections, while retaining minimal/default/feature-isolation and native/host-target exception groups.
- Derive preparation and execution from the same configuration plan and concrete artifacts.
- Make the DX.4 fixture profile inventory enforceable: retain every existing release assertion, add development coverage and explicit default-command tests, and run the complete declared release `run-pass` selection with actual release binaries in release qualification.
- Measure dev-only, release-only and alternating dev/release storage, dependency builds, memory and cold/no-op/edit latency on the floor machine. Profile coexistence must not invoke size-triggered cleanup or evict the other active profile.

**Definition of done**

- Default run/build use the declared development application profile; release selection is explicit and test-harness differences are documented/tested.
- Supported numeric/error/ownership/resource behavior and required panic boundaries hold across the qualified profiles.
- Grouping neither adds an unintended all-features application nor removes a required feature-isolation/native assertion.
- Preparation and execution use the same binaries, features and environment; no hidden duplicate compiler build is introduced.

**Validation and evidence**

Complete B05, B06, B11, B12 and R06. Compare real application binaries in both profiles, include release execution outside the Rust test harness, and diff assertion/configuration inventories before/after grouping. Measure cold dependency and warm edit costs, actual duplicate/reused units and disk occupancy for both profiles under the shared-storage design. Neither unifying profiles nor testing only the new default can satisfy the retained release contract.

### milestone_dx_11: Editor Snapshots, Recovery and Memory Discipline

**Depends on:** DX.10.  
**Primary owners:** analysis, LSP, frontend/source, sysroot.  
**Architecture:** section 10.

**Scope**

- Ensure immutable request snapshots, pinned toolchain stores and authoritative overlay ownership across open/change/save/close/configuration events.
- Enforce generation checks for every stale-sensitive publication independently of cancellation timing.
- Establish the saved-source/overlay boundary and request-local identity required by the later persistence adapter. No project disk reuse is claimed here; DX.13/DX.14 connect it and re-run the overlay/stale-result matrix.
- Release old snapshots/closed projects and unreferenced decoded metadata; reduce demonstrated layout/clone/interner costs without arbitrary rustc-sized targets.
- Preserve syntax-only responsiveness and actionable recoverable setup diagnostics.

**Definition of done**

- Older work cannot overwrite newer diagnostics or editor state; every request follows the existing protocol terminal-response rules.
- CLI saved-source and editor overlay results never contaminate each other.
- Unicode/CRLF/path aliases and supported case rules preserve source identity and result ranges.
- Memory tests cover transient overlapping snapshots as well as steady-state retention and preserve scoped budgets.
- Pure formatting/syntax requests do not force unrelated metadata/native/environment work.

**Validation and evidence**

Complete E01–E06, plus metadata recovery/upgrade regression rechecks through the actual LSP transport. Complete Q09 with paired optimized-installed baseline/candidate open/edit/close/switch traces and heap/RSS measurements on the reference host. Report steady and peak deltas, not only pass/fail at 128 MiB; attribute any missing anticipated RSS decrease. Use layout assertions as regression guards only alongside total retention and latency evidence.

**Checkpoint:** the metadata/native/profile/editor capabilities may be qualified and released. Their measured user/contributor baseline is the comparison input for DX.12–DX.14. Reuse applicable candidate evidence and continue the remaining required milestones.

### milestone_dx_12: Semantic Dependencies and Reusable Result Families

**Depends on:** DX.11.  
**Primary owners:** frontend, lowering/IR, package, source.  
**Architecture:** sections 4, 9.1 and 9.3–9.4.

**Scope**

- Define persistence-facing module result families and completeness states without serializing the live workspace process.
- Capture the same source bytes used for analysis and record complete file-set, ordered resolution, absence, package/configuration and external semantic observations.
- Implement stable remapping for persisted types/declarations/source ranges and canonical diagnostic facts.
- Define the complete module-interface summary and conservative invalidation boundary before optimizing propagation in DX.14.

**Definition of done**

- Source hashes cannot describe a different read from the analyzed input.
- Missing/higher-priority modules, source-set/configuration changes and declared external inputs are represented as dependencies.
- Result completeness is separate from successful source compilation; checking success does not imply a ready HIR family, rich editor state or executable.
- Existing cycle policy and source/path semantics remain unchanged. Presentation-only changes do not force semantic recomputation.

**Validation and evidence**

Complete I03, P03, P04 and P07, plus serialization/remapping unit tests for every planned result family. Add explicit dependency-observation golden tests and conservative edit sequences. Do not enable cross-process semantic reuse until the complete input contract is covered.

### milestone_dx_13: Transactional Cross-Process Project Reuse

**Depends on:** DX.12.  
**Primary owners:** frontend/store adapter, driver storage, CLI.  
**Architecture:** sections 7 and 9.

**Scope**

- Implement `.sifrbuildinfo` generation hints and versioned module-result restoration through the existing frontend query API.
- Revalidate schema/compiler/context, source and dependency observations before reusing completed result families.
- Introduce the complete project-generation protocol with its real semantic payloads: private working generations, reader retention, inherited immutable records/hard-link-or-copy discipline, atomic final publication and project-generation GC. Extend DX.3 entry primitives; no finalized shared record is rewritten in place.
- Publish complete immutable generations, preserve still-valid unchanged records, and implement explicit cache-miss/read/write/cancellation behavior. Revalidate winners and manifest references; test process death and concurrent GC with actual restored module state.
- Add `--no-incremental` and read-only-workspace behavior without turning installed metadata or Cargo state into collateral cleanup targets.

**Definition of done**

- A new CLI process actually restores completed checking work and yields the same canonical diagnostics as fresh computation.
- Missing/corrupt/incompatible project state is safely recomputed; unwritable cache does not change a valid source result into a language error.
- Cancelled/crashed/transient computations cannot become globally successful records. Completed deterministic source errors obey their separate contract.
- Restoring a shallower result does not claim a missing deeper family exists; build/editor requests compute what is absent.
- A/B/A generation turnover preserves eligible results without weakening input validation.

**Validation and evidence**

Complete C04, C08, C09, P01, P02, P08–P10 and P12. Re-run relevant publication/reader/GC tests with actual semantic payloads, not only empty fixture records. Compare fresh and restored commands across process boundaries, including wall cost of validation and serialization. Record reuse counters and memory as well as timing.

### milestone_dx_14: Proven Interface Propagation and Editor Persistence Integration

**Depends on:** DX.13.  
**Primary owners:** frontend/lowering, package/component integration.  
**Architecture:** sections 9.3–9.4.

**Scope**

- Recompute changed modules and propagate invalidation through complete semantic interfaces/observations.
- Preserve importer checking when an ordinary implementation-body change demonstrably leaves every consumed semantic dependency unchanged.
- Include defaults, constants, generic/const bodies, inferred effects, ownership and relevant SQL/Python/component context in propagation.
- Keep unknown scopes conservative; do not add a replacement query engine or expression-level persistence.
- Connect eligible saved-source persistent results to the existing LSP session without writing unsaved overlays as CLI authority. Keep all DX.11 snapshot/retention properties and pin the same semantic/toolchain context in restored and live paths.

**Definition of done**

- Relevant body/default/constant/effect changes invalidate consumers even when superficial signatures match.
- Unrelated or interface-stable implementation changes retain eligible importer results while invalidating local/native work appropriately.
- In-memory and disk-restored propagation follow the same rules and remain correct after deletion, reconfiguration and error recovery.
- Tests distinguish reuse effectiveness from correctness: disabled caching cannot satisfy the reuse requirements.

**Validation and evidence**

Complete P05, P06 and P11, plus randomized bounded edit sequences compared against fresh independent runs. Keep independent expected semantic fixtures alongside equivalence. Record invalidated/reused module families with reasons, and prohibit unexplained expected-difference baselines. Re-run E01–E06 and Q09 with persistence enabled/disabled, including stale saved-source records, unsaved changes and old requests completing late.

### milestone_dx_15: End-to-End Performance and Distribution Qualification

**Depends on:** DX.14.  
**Primary owners:** performance, distribution, verification, all affected subsystem owners.  
**Architecture:** sections 6.3, 13 and 14.

**Scope**

- Run the fixed product/user/contributor workloads on the qualified 12 GB host and supported named reference configurations.
- Compare candidate and pinned comparable baseline, retain long-term anchors, and qualify actual percentile/resource claims with sufficient sampling.
- Verify all supported release targets and installer/update/relocation/rollback modes using exact packaged bytes and explicit native profiles/loader behavior.
- Complete the integrated timings/trace/doctor/cache surfaces and documentation for normal installed, source-tree, offline/prepared and error-recovery workflows.

**Definition of done**

- The fixed small-program fresh/unchanged CLI targets and applicable editor/memory contracts pass with demonstrated reuse mechanisms.
- Preparation, cache loading/writing and native work remain visible; noise/inconclusive measurements are not relabeled as passing or repeatedly sampled until green.
- The supported-machine workload works without a larger-machine prerequisite or weakening current correctness/trust/runtime coverage.
- Every required acceptance case has concrete evidence or is explicitly unresolved; unresolved required cases block phase completion.
- Packages contain the exact qualified artifact identities and do not require source-tree paths, ambient toolchains or undocumented native libraries.

**Validation and evidence**

Complete Q02–Q04 and Q06; execute the full relevant acceptance matrix under the final configurations. Preserve original cold and warm reports separately. Produce the final candidate's required gate and package qualification evidence once its implementation inputs are fixed. Actual external publication/account actions remain outside this phase.

### milestone_dx_16: Closure and Authoritative Handoff

**Depends on:** DX.15.  
**Primary owners:** phase owner and affected documentation owners.  
**Architecture:** section 15.

**Scope**

- Confirm all milestones, architecture invariants and acceptance cases are implemented and qualified; reconcile any renamed files/commands in both documents.
- Remove obsolete production source-bootstrap paths, unqualified global caches, competing identity/binary resolvers and superseded documentation after consumer migration is complete.
- Update architecture/index/roadmap summaries and the compact phase status with actual merged candidates, validation/review references and any genuinely separate follow-up work.
- Preserve original evidence and do not create a second monolithic execution diary.

**Definition of done**

- No required capability is deferred merely because of implementation time or complexity; no unproven performance claim is recorded as achieved.
- The final installed and development commands match the documented contracts, including project persistence, profiles and recovery.
- Relevant review and validation cover the same final implementation inputs. If closure changes implementation, validate the affected changes as a new bounded item.
- Documentation-only closure reuses qualified evidence and runs only relevant documentation checks; no recursive whole-phase external review or repeated broad gate is added.

**Validation and evidence**

Run link/schema/documentation checks and verify the complete evidence index. Reference the DX.15 final implementation qualification; do not rerun it for record-only changes. Mark the phase completed only after recording the real merged state.

## Definition Of Done: Whole Phase

The installed CLI and existing LSP use the same demand-loaded prebuilt stdlib with semantic records, emitted support and required templates. Source-tree production is explicit and non-circular. Toolchains and persisted records have complete identities; compatible native builds reuse Cargo artifacts without root/source/output corruption. Project checking restores real module work across processes, and edited invalidation is correct and effective. Editor results use the correct source snapshot and remain within their declared resource budgets.

The verification platform prepares each required configuration once, preserves distinct assertion depths, collects independent failures usefully and reports functional/performance/safety outcomes separately. Full metadata traversal is explicit, normal E2E remains lazy, and behavioral coverage is not inferred from structural decoding. Required package/native/Python trust and runtime boundaries remain unchanged.

All canonical acceptance cases in architecture section 16 are covered. The final target/corpus/resource evidence satisfies architecture section 14. No stale private-cache compatibility, hidden fallback checker, global unrestricted target sharing or evidence-rewriting shortcut remains.

## Execution Status

Implementation has not started. Update the table from actual merged work and evidence. The next action is to execute DX.1 against the actual current base.

| Milestone | State | Final candidate / merged PR | Validation evidence | Review evidence |
| --- | --- | --- | --- | --- |
| DX.1 | Not started | — | — | — |
| DX.2 | Not started | — | — | — |
| DX.3 | Not started | — | — | — |
| DX.4 | Not started | — | — | — |
| DX.5 | Not started | — | — | — |
| DX.6 | Not started | — | — | — |
| DX.7 | Not started | — | — | — |
| DX.8 | Not started | — | — | — |
| DX.9 | Not started | — | — | — |
| DX.10 | Not started | — | — | — |
| DX.11 | Not started | — | — | — |
| DX.12 | Not started | — | — | — |
| DX.13 | Not started | — | — | — |
| DX.14 | Not started | — | — | — |
| DX.15 | Not started | — | — | — |
| DX.16 | Not started | — | — | — |

## Handoff Format

After each bounded item, record only the current milestone/item, branch and candidate, PR state, relevant validation/review evidence, unresolved blocker if any, and exact next action. Keep historical logs and full reports in their evidence locations. An older issue, branch or review must explicitly identify its successor when superseded, so a new session cannot revive an obsolete requirement.

## Planning And Repository References

The architecture's source appendix contains the pinned Rust/TypeScript/Cargo mechanisms and current Sifr integration paths. Repository formatting and workflow references for this phase are [Phase 35](../../phases/35_performance_benchmarking_and_budgets.md), [Phase 36](../../phases/36_developer_tooling_and_ecosystem_hooks.md), [AGENTS.md](../../../AGENTS.md), and the [phase-closure skill](../../../.cursor/skills/phase-closure-loop/SKILL.md). These references preserve ownership and execution conventions; they do not authorize changing unrelated release or language requirements.
