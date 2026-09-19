# Phase DX: Compiler Developer Experience and Toolchain Reuse

status: in progress; DX.1–DX.14 complete; DX.15 qualification in progress
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

**Prospective validation policy (user-approved 2026-09-17):** each intermediate milestone requires its named acceptance tests, focused regressions for changed behavior, and scoped Opus review. Do not run a full create-PR or merge gate as an intermediate merge prerequisite. At phase end, run one full merge gate on the final implementation; correct failures and rerun affected checks as needed. Release-checkpoint qualification runs only when an actual release is requested. Record-only changes require relevant documentation checks, with no new external review or broad gate.

This policy supersedes earlier per-item gate requirements for Phase DX from this date forward. Historical failed or incomplete gates retain their original outcomes; deferral does not turn them into passes. Group related in-scope fixes before qualification. A failed named prerequisite blocks dependents; independent diagnostic collection may continue only when safe. Do not repeatedly run an unchanged failed performance check without new evidence or a controlled measurement reason.

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

Only if an actual release is requested after DX.8, the metadata improvement may be released when its exact installed artifacts pass the existing applicable package/publication qualification. After DX.11, native/profile/editor improvements may be released on the same basis. Neither checkpoint marks Phase DX complete, advertises unimplemented project persistence, changes release authorization, or requires additional duplicate reviews/gates for unchanged evidence. They are integration and release opportunities, not optional scope decisions.

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

**Checkpoint:** if an actual release is requested, the exact metadata-enabled installed toolchain may be independently released after its applicable release qualification. Record only implemented claims and existing authorization; Phase DX and project persistence remain incomplete. This checkpoint is not another automatic whole-gate/review cycle over unchanged evidence.

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
- Follow the user-directed host split of 2026-09-18: qualify bounded editor/Cargo resources and fixed product targets on the 12 GB Linux reference, and real desktop overlap separately on the named Mac with its actual 32 GiB hardware. Preserve the original combined 12 GB graphical-desktop scenario as not executed; neither infer a Linux pass from the Mac nor raise the Linux machine requirement.
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

## Acceptance-case sequencing

The architecture section 16 remains the canonical semantic case inventory. This plan owns the original first-milestone assignment; moving this scheduling column here does not change case IDs, requirements, ownership, or sequencing.

| Case ID | First milestone |
| --- | --- |
| I01 | DX.2 |
| I02 | DX.2 |
| I03 | DX.12 |
| I04 | DX.9 |
| I05 | DX.5 |
| I06 | DX.5 |
| I07 | DX.2 |
| I08 | DX.2 |
| M01 | DX.7 |
| M02 | DX.7 |
| M03 | DX.8 |
| M04 | DX.7 |
| M05 | DX.8 |
| M06 | DX.5 |
| M07 | DX.8 |
| M08 | DX.8 |
| M09 | DX.6 |
| M10 | DX.8 |
| M11 | DX.8 |
| M12 | DX.6 |
| M13 | DX.6 |
| M14 | DX.6 |
| M15 | DX.8 |
| M16 | DX.5 |
| M17 | DX.7 |
| C01 | DX.3 |
| C02 | DX.3 |
| C03 | DX.3 |
| C04 | DX.13 |
| C05 | DX.3 |
| C06 | DX.3 |
| C07 | DX.3 |
| C08 | DX.13 |
| C09 | DX.13 |
| B01 | DX.9 |
| B02 | DX.9 |
| B03 | DX.9 |
| B04 | DX.9 |
| B05 | DX.10 |
| B06 | DX.10 |
| B07 | DX.9 |
| B08 | DX.9 |
| B09 | DX.9 |
| B10 | DX.3 |
| B11 | DX.10 |
| B12 | DX.10 |
| P01 | DX.13 |
| P02 | DX.13 |
| P03 | DX.12 |
| P04 | DX.12 |
| P05 | DX.14 |
| P06 | DX.14 |
| P07 | DX.12 |
| P08 | DX.13 |
| P09 | DX.13 |
| P10 | DX.13 |
| P11 | DX.14 |
| P12 | DX.13 |
| E01 | DX.11 |
| E02 | DX.11 |
| E03 | DX.11 |
| E04 | DX.11 |
| E05 | DX.11 |
| E06 | DX.11 |
| E07 | DX.4 |
| R01 | DX.3 |
| R02 | DX.3 |
| R03 | DX.3 |
| R04 | DX.4 |
| R05 | DX.4 |
| R06 | DX.10 |
| R07 | DX.4 |
| R08 | DX.4 |
| R09 | DX.8 |
| R10 | DX.6 |
| Q01 | DX.4 |
| Q02 | DX.15 |
| Q03 | DX.15 |
| Q04 | DX.15 |
| Q05 | DX.8 |
| Q06 | DX.15 |
| Q07 | DX.1 |
| Q08 | DX.1 |
| Q09 | DX.7 |


## Execution Status

DX.1–DX.14 are complete and merged. DX.15 implementation and final qualification are in progress in the owned remote worktree; DX.16 remains not started. Intermediate validation follows the prospective policy above.

| Milestone | State | Final candidate / merged PR | Validation evidence | Review evidence |
| --- | --- | --- | --- | --- |
| DX.1 | Complete / merged | Candidate `5c7502f7aea3cd887a217c2fb3e289c7a1f2c653`; [PR #3840](https://github.com/sifr-lang/sifr/pull/3840); merge `de41ced4d65a3c511617219238f26052227fd6e8` | Product and contributor baselines validated; Q07/Q08, transport regressions, schema and guardrail pass; evidence below | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3840#issuecomment-5705061104) |
| DX.2 | Complete / merged | Candidate c6841ab44a6c0e738045586f32e36c5fa1155a26; [PR #3842](https://github.com/sifr-lang/sifr/pull/3842); merge 59be2f046f71e0e3ac626f51e8ffd2614573f8bd | I01/I02/I07/I08 actual rebuilds and focused regressions PASS; broad gate deferred by prospective policy | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3842#issuecomment-5709069845) |
| DX.3 | Complete / merged | Candidate `63640031f7b66e067b2cbb26849277faed725668`; [PR #3844](https://github.com/sifr-lang/sifr/pull/3844); merge `f7b733875ecd2a1bd86431daed4056249dad9212` | C01–C03, C05–C07, B10, R01–R03 and focused checks PASS; full gate deferred by prospective policy | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3844#issuecomment-5711325637) |
| DX.4 | Complete / merged | Candidate `fa8083ccfea3ec1b8d0fe8a5290c9a3d21efe834`; [PR #3846](https://github.com/sifr-lang/sifr/pull/3846); merge `41675c336e562ff43c4339630097a4533d321de7` | E07, R04, R05, R07, R08, existing-path Q01 and focused checks PASS; explicit unchanged-input reuse; full gate deferred | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3846#issuecomment-5712346406) |
| DX.5 | Complete / merged | Candidate `c7734025c242631a326901ce08c846ac42afd7ae`; [PR #3848](https://github.com/sifr-lang/sifr/pull/3848); merge `22a72c6c6c9f95242e3e60ba83e473f0edd60d11` | I05, I06, M06, M16: 14 focused tests PASS; schema/site, scoped clippy/fmt and guardrails PASS; full gate deferred | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3848#issuecomment-5712915895) |
| DX.6 | Complete / merged | Candidate `5581af0884edcb46d88419b9b7d66d1ce97531df`; [PR #3850](https://github.com/sifr-lang/sifr/pull/3850); merge `d53899694e8b65f47f7105fda34fec039459549a` | M09, M12–M14, R10 and focused producer/preparation/packaging checks PASS; two full bare driver runs 649 passed / 78 ignored each; full gate deferred | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3850#issuecomment-5717512054) |
| DX.7 | Complete / merged | Candidate `036e69592164b87d309f31ba43ee406a58ed79ca`; [PR #3852](https://github.com/sifr-lang/sifr/pull/3852); merge `fb6d432be20d945e0266030b076dbc76ce4c8a50` | M01, M02, M04, M17, representative M15/native parity and paired installed Q09 PASS; focused tests and guardrails PASS; full gate deferred | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3852#issuecomment-5719741485) |
| DX.8 | Complete / merged | Candidate `78f12c5cdd357f1601926f3fa78ec96f004108dd`; [PR #3854](https://github.com/sifr-lang/sifr/pull/3854); merge `1b55710f4a6e1d796be243c025b673f8958de72b` | M03, M05, M07, M08, M10, M11, M15, R09, Q05, Q09 PASS; 727-case corpus/native, four-target portable records, actual Linux/Mac qualification, installed generations and focused regressions; full gate deferred | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3854#issuecomment-5722224549) |
| DX.9 | Complete / merged | Candidate `2f1f3fb215d5263c0d1fb49d8c3346aafa299b33`; [PR #3856](https://github.com/sifr-lang/sifr/pull/3856); merge `455400351b3d989490191a09b3cb3d2c817404fa` | I04, B01–B04, B07–B09 and focused regressions PASS; actual native/Python exports, current Cargo freshness and paired rebuilt-unit/latency measurements; full gate deferred | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3856#issuecomment-5723374425) |
| DX.10 | Complete / merged | Candidate `d4b80812086538f6d26bf7830f30da54d52bffb0`; [PR #3858](https://github.com/sifr-lang/sifr/pull/3858); merge `65e870fced69b02c6833fa9f000d31d8c1bbefb8` | B05, B06, B11, B12, R06 PASS; 727 actual release fixtures, additional dev/default coverage, profile/configuration inventories and actual cold/warm/edit/alternating costs; final consumer regressions and explicit unchanged-input reuse; full gate deferred | [Opus: SATISFIED after adjudicated remediation](https://github.com/sifr-lang/sifr/pull/3858#issuecomment-5725527463) |
| DX.11 | Complete / merged | Candidate `c0d490d460ca4f8a286226da804b31744b5a0b3c`; [PR #3860](https://github.com/sifr-lang/sifr/pull/3860); merge `24eec746b3b72d1b3bdbbb073986092f629b1ab9` | E01–E06, nine Rust regressions, installed protocol/push-progress PASS; real upgrade/recovery and paired Q09 with scoped unchanged-input reuse; post-close ownership released, no active/peak RSS improvement claimed; full gate deferred | [Opus: SATISFIED after bounded remediation](https://github.com/sifr-lang/sifr/pull/3860#issuecomment-5726309175) |
| DX.12 | Complete / merged | Candidate `cc2874a319feef6f96f45954b91dc3bed79cc2e8`; [PR #3862](https://github.com/sifr-lang/sifr/pull/3862); merge `3959f0168f0630802548ad2b12d3623ab71f151c` | I03, P03, P04, P07 and all five result families: 14 tests PASS; ordered-observation golden, actual resolver/package edits, real writer and LSP encoding; guards PASS; full gate deferred | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3862#issuecomment-5726861173) |
| DX.13 | Complete / merged | Candidate `be67cc8dfa7d6db91f295557f08123d705cc79ff`; [PR #3864](https://github.com/sifr-lang/sifr/pull/3864); merge `9ee1d65db9a1f1851f46032f641d39dc27460bc0` | C04/C08/C09/P01/P02/P08–P10/P12; 12 DX.13 tests, focused regressions, 31 CLI processes and guards pass; evidence below | [Opus: SATISFIED](https://github.com/sifr-lang/sifr/pull/3864#issuecomment-5727613414) |
| DX.14 | Complete / merged | Candidate `2c3f23127966850e7ed961155bcee817514d56f1`; [PR #3866](https://github.com/sifr-lang/sifr/pull/3866); merge `10e917056bc26d7a7b21c1361cc9cecfb1c7afb0` | P05/P06/P11, 35 targeted Rust tests, 44 CLI invocations, native output refresh, E01–E06 enabled/disabled and 84 Q09 samples pass; guards pass; full gate deferred | [Opus: SATISFIED after bounded remediation](https://github.com/sifr-lang/sifr/pull/3866#issuecomment-5729004460) |
| DX.15 | In progress: fixed-candidate qualification and required gate repairs | — | — | — |
| DX.16 | Not started | — | — | — |

## Historical Handoff — DX.1 (2026-09-16)

Superseded for next-action purposes by the DX.2 handoff below; historical evidence is unchanged.

- **State:** complete and merged. Owned checkout:
  `yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/compiler-dx-orchestration`;
  branch `codex/compiler-dx-toolchain-reuse`. Implementation [PR #3840](https://github.com/sifr-lang/sifr/pull/3840),
  final candidate `5c7502f7aea3cd887a217c2fb3e289c7a1f2c653`, merge
  `de41ced4d65a3c511617219238f26052227fd6e8`; phase base
  `0f819c2f04bf5b2891074c55ba26369ddf4f13bd`.
- **Scope completed:** actual installed optimized/contributor artifact lanes;
  compiler/application/verification profile separation; prospective cleanup/cache
  policy; immutable legacy-budget mapping and Q07/Q08 seeds; frozen fixture
  assertion authorities and Q09 inputs; current baseline capture. The user
  explicitly included the required LSP transport correction in this batch.
  Canonical reads now handle coalesced/fragmented frames within one deadline,
  and cleanup retains the primary error. External report paths also work.
- **Validation:** seven independent transport regressions pass; benchmark runner
  self-test (Q07/Q08 and seeded numerical budget failures) passes on the final
  candidate. The 65-case manifest/schema and file-size guardrail evidence remain
  valid with unchanged inputs; final path-display changes preserve line counts.
  Final `git diff --check` passes. No compiler, lockfile, actual fixture or
  workflow files changed, so no broad gate was run under the user's batch rule.
- **Baselines:** product capture passes 21 checks, native first/no-op/edit, and
  21 demanded-stdlib LSP samples (one warmup); CV is 0.0059124324. Steady/peak/
  post-close RSS are recorded; unavailable allocation/decoder counters stay null
  with reasons. Contributor completion covers checks/native, unchanged/edited
  sessions and the two matching bare Cargo stdlib-cache tests. It explicitly
  consolidates provenance-bound successful original rows with the repaired
  prepared-session run; it is not a new timing claim for reused rows.
  No optimized performance target or improvement is claimed.
- **Preserved failures/preparation:** original ambient-toolchain and LSP failures
  remain failed. Contributor cold helper preparation hit its existing 180-second
  deadline; explicit helper preparation was then recorded, without changing
  deadlines. The prepared attempt exposed an outside-tree report-display error;
  that failed report is retained, and only the affected session selection was
  rerun after correction. Rust/Cargo 1.98.1 and two Cargo jobs were explicit;
  useful owned artifacts were reused. Bare Cargo's 661-second cold test build is
  retained in workflow cost, not hidden as free preparation.
- **Review:** Claude Opus 5 returned SATISFIED, no blocking findings, for the
  final candidate. [Published review](https://github.com/sifr-lang/sifr/pull/3840#issuecomment-5705061104).
  Review evidence lives outside the reviewed tree keyed by candidate SHA.
- **Follow-ups:** [separate review observations](ad-hoc-dx-baseline-review-followups.md)
  retain optional contract-driven capture and future product-policy migration,
  ambient-sysroot ownership, runner size, and direct-read diagnostic context.
  They are not DX.1 blockers or authorization for this session to implement them.
- **Blocker:** none.
- **Exact next action:** stop this session after this record update. DX.2 requires
  a new bounded session; no DX.2–DX.16 code was written. Local unrelated
  modifications remain untouched.

### Evidence retained outside the reviewed tree

Host/root: `yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx1-evidence/`.
Raw command receipts remain beside reports. The checked-in capture/preparation
commands are documented in `internal_docs/performance_budgets.md`; the exact
consolidation script is saved below, including artifact/lock/toolchain equality
assertions and per-row source paths, hashes and original row indices.

| Artifact | SHA-256 / result |
| --- | --- |
| `fixture-inventory.json` | 3215 frozen authorities/fixtures; `201476d3bf5d1abd3912967a06e156ab630203cb50cd3a0a9878ace55ff2197a` |
| `selftest-external-path.log` | PASS final candidate; `2d15e391d0fa19411c20b70eb34107c39698a8954b0814bd04b030f4b1e71056` |
| `transport-e1bbacdf4.log` | 7 PASS, unchanged transport; `3d8774e265a9d03b69052039dc0e913b74282c43ab8b5ef0d4bf33b07a837324` |
| `guardrail-e1bbacdf4.log` | PASS, final change preserves source line counts; `0371f7ff0407b48ec3c2b827e55899ed48ef9f856b1b3562c29558d86f65f945` |
| `product-5c7502f7a/receipt.json`, `dev-5c7502f7a/receipt.json` | Validated final-candidate artifact/profile/sysroot/toolchain receipts |
| `product-baseline-5c7502f7a/baseline.json` | PASS; `63630af41872a6592af2a242336b99b0228ad938a1adad76e389d730d03e7994` |
| `dev-completed-5c7502f7a/baseline.json` | Consolidated completion PASS; `13d1ae567054848ec6329dc24ad6b58daab7f0db80b068961919661858fb9393` |
| `consolidate-5c7502f7a.py` | Exact consolidation command; `73a8eaa7d97af1d2ee9d6fcdd7d9a7aa2a51492b936cce040381dca4fbf07d03` |
| `reviews/5c7502f7aea3cd887a217c2fb3e289c7a1f2c653/response.md` | SATISFIED; `07b875ef13e13e8f1964e960b179732a975541ccaaf8c5177d5344ded41eda35` |
| `product-baseline-28605fd/baseline.json` | Preserved FAILED ambient-toolchain run; `4c34e566c5b50084c8f319caac2be9f2fbf9659af3cc0a60abdae369ba39f01c` |
| `product-baseline-explicit-toolchain-28605fd/baseline.json` | Preserved FAILED transport run; `b27c216248e0d757723fb603c739c6deffed40385bbcfd2d63ca0177e105ec7f` |
| `dev-baseline-e1bbacdf4/baseline.json` | Preserved FAILED cold helper build; successful rows are explicitly reused |
| `dev-completed-e1bbacdf4/baseline.json` | Preserved FAILED outside-tree report display |
| `frontend-helper-e1bbacdf4.log` | Explicit preparation cost after cold timeout; hash bound from completed contributor report |

## Merged Record — DX.2 (2026-09-17)

- **State:** complete and merged in [PR #3842](https://github.com/sifr-lang/sifr/pull/3842).
  Candidate c6841ab44a6c0e738045586f32e36c5fa1155a26; merge
  59be2f046f71e0e3ac626f51e8ffd2614573f8bd; base
  8fb424984351768116251ed5a5e577373603c533. Owned remote checkout remains
  /home/yaser5/projects/sifr/compiler-dx-orchestration; implementation branch
  codex/dx2, record branch codex/dx2-record.
- **Scope:** role-separated canonical identities; outer product input tracking;
  dependency-scoped compiled test tokens; immutable compiler context injection;
  context-owned stdlib caches; pinned inspectable Cargo/rustc/configuration;
  native key separation and constructor/entrypoint inventories. The explicitly
  authorized Python qualification prerequisite pins the probed libpython loader
  and preserves original import errors. No DX.3 implementation.
- **Named validation:** I01/I02/I07/I08 PASS through actual Cargo rebuilds.
  Same-HEAD dirty LSP changes alter product identity while frontend/sysroot remain
  fresh. Compiled dependency/feature changes alter test and product identities;
  inventory add/delete, archive inputs and source-independent installed inspection
  pass. Deterministic encoding, explicit overrides, unavailable tools, redaction,
  frontend/context ownership and native key tests pass. Required workspace Clippy,
  formatting and HIR/file-size checks pass.
- **Evidence reuse:** final Rust implementation is f912025c15cb1259203ca053b7e181dc2f15fd84;
  c6841ab44 only adjusts two verification guards. Its actual rebuild receipt and
  focused compiler/Python results remain applicable. The complete Git-free source
  archive build at 82e6fa2d7 is reused for unchanged archive/identity mechanics.
  Final warm generation-only median is **15.526 seconds** (first sample excluded),
  not a product target or claimed speedup.
- **Review:** final scoped Claude Opus 5 review is SATISFIED, no blockers.
  [Published response](https://github.com/sifr-lang/sifr/pull/3842#issuecomment-5709069845);
  SHA-256 27b7424514cbb38fc982d0e440c754e798f3b06e3b465f21ab49c11c9c9ef59d.
- **Preserved broader evidence:** original gates remain failed, including
  dependency/taxonomy/guard integration failures and the Python loader failure.
  The final c6841ab44 aggregate attempt failed at an unprofiled Linux-versus-Mac
  performance comparison after preceding functional areas passed. Authorized
  focused performance qualification passed with the immutable matching Linux
  reference and all governors restored. Missing nested editor and Node/npm
  preparation was completed; affected distribution checks passed. Cold sysroot
  packaging exceeded its original deadline; prepared installed-smoke and boundary
  equivalence passed under unchanged limits. Further collected project/package,
  stdlib, regression, fuzz and ecosystem results pass. On the user's prospective
  policy change, owned SQL execution was stopped safely; full crate/E2E steps
  were not started. **No full-gate pass is claimed.**
- **Follow-ups:** [separate identity/review observations](ad-hoc-dx2-identity-review-followups.md).
  The [Python qualification owner](ad-hoc-python-interop-qualification-dependencies.md)
  records the bounded prerequisite. Full final-implementation validation remains
  required at phase end; actual release checkpoints retain their qualification.
- **Blocker:** none for DX.2 acceptance.
- **Exact next action:** stop after this record update. Start DX.3 only in a new
  bounded session. Local unrelated modifications remain untouched.

Evidence host/root:
yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx2-evidence/.

| Artifact | Result / SHA-256 |
| --- | --- |
| evidence-index-c6841ab44.json | Full receipt/log digest index; ffc8d68fa638d5f58624e0809ee8c2fa7e8d69e31c499de126eb11fa411e0da5 |
| rebuild-f912025c1/receipt.json | Actual I01/I02/I07/I08, native context and overhead proof |
| validation-summary-c6841ab44.md | Current acceptance, evidence reuse and prospective policy |
| python-affected-examples.log, python-qualified/ | Nine affected example variants PASS |
| merge-c6841ab44.report.json | Original aggregate FAILED; retained unchanged |
| performance-named-c6841ab44.json | Corrected named-reference performance PASS |
| merge-c6841ab44-*-completion.json | Provenance-bound focused results; original failures retained |
| gate-policy-stop.json | Owned execution terminated after prospective policy change |

## Handoff Format

After each bounded item, record only the current milestone/item, branch and candidate, PR state, relevant validation/review evidence, unresolved blocker if any, and exact next action. Keep historical logs and full reports in their evidence locations. An older issue, branch or review must explicitly identify its successor when superseded, so a new session cannot revive an obsolete requirement.

## Planning And Repository References

The architecture's source appendix contains the pinned Rust/TypeScript/Cargo mechanisms and current Sifr integration paths. Repository formatting and workflow references for this phase are [Phase 35](../../phases/35_performance_benchmarking_and_budgets.md), [Phase 36](../../phases/36_developer_tooling_and_ecosystem_hooks.md), [AGENTS.md](../../../AGENTS.md), and the [phase-closure skill](../../../.cursor/skills/phase-closure-loop/SKILL.md). These references preserve ownership and execution conventions; they do not authorize changing unrelated release or language requirements.

## Historical Handoff — DX.3 (2026-09-17)

Superseded by the merged DX.4 handoff below; its next-action instruction is historical.

- **State:** complete and merged in [PR #3844](https://github.com/sifr-lang/sifr/pull/3844).
  Candidate `63640031f7b66e067b2cbb26849277faed725668`; merge
  `f7b733875ecd2a1bd86431daed4056249dad9212`; base
  `f22e6686f12be874d4e30d89c51d76d13d832d03`. Owned remote checkout:
  /home/yaser5/projects/sifr/compiler-dx-orchestration; implementation branch
  codex/dx3, record branch codex/dx3-record.
- **Scope:** persistent generated native/test artifact storage; same-filesystem
  private staging, immutable publication, OS leases, winner validation and
  active-reader protection; owner-scoped pressure pruning and auxiliary-root
  protection; compiler process outcomes, bounded raw streams, cancellation and
  descendant cleanup; complete streamed user-program output; independent safety
  deadlines/performance contracts; runner-owned status and retained child metrics;
  initial doctor/cache/timing surfaces. No semantic generation GC or DX.4 work.
- **Named validation:** C01–C03, C05–C07, B10 and R01–R03 PASS through real
  filesystem/subprocess injection and actual native/test consumers. Eight Rust
  tests cover killed producers at publication boundaries, surviving descendants,
  concurrent readers/winners, active older-entry prune exclusion, distinct TMPDIR
  filesystem, invalid payloads/ownership, positive pressure cleanup, retained
  useful large storage, timeout/cancellation and raw output. Ten Python tests cover
  status forgery, preserved metrics, separate functional/performance outcomes,
  safety deadlines, cancellation, bounded output and detached observers.
- **Actual consumers:** final-candidate cold/warm CLI tests under umask 002 pass;
  warm finalized payload mtimes stay unchanged while tests rerun, failing tests
  remain failures, missing executables rebuild, pressure removes eligible older
  entries while protecting newest state, and user output retains all 9,000,001
  bytes. Cache inspect/dry-run, timings and actionable doctor errors pass.
- **Focused checks:** CLI schema parity, production driver Clippy, selected-crate
  formatting, step-budget self-test and 900-line guardrail pass. Unchanged native
  consumer reuse/invalidation evidence is reused. Optional all-targets Clippy
  exposed three pre-existing test lints; its failed output remains preserved and
  is owned by the separate follow-up below.
- **Before/after evidence:** retained DX.2 binary identity is tied to its actual
  rebuild receipt. Baseline and final cold/warm preparation and cleanup receipts
  remain separate. These observations are functional evidence, not a
  host-sensitive performance claim; no cold-cache run is relabeled as warm.
  Existing approximately 116–130 GiB target was retained with sufficient free
  space (approximately 207–220 GiB); cleanup qualification used owned cache entries.
- **Review:** final scoped Claude Opus 5 review is SATISFIED, no blockers.
  [Published response](https://github.com/sifr-lang/sifr/pull/3844#issuecomment-5711325637);
  SHA-256 `2c4f1f652364f1490db1d807119cb1566d171b94bf985870baed91c173ea0345`.
  The initial NOT SATISFIED review and failed intermediate evidence remain
  preserved; all four blockers were remediated in one batch and requalified.
- **Policy:** no intermediate create-PR/merge full gate was run under the
  prospective phase-end policy. No full-gate pass is claimed. One full gate remains
  required on the final phase implementation; actual release qualification remains
  conditional on a release request.
- **Follow-ups:** [separate storage/process review observations](ad-hoc-dx3-storage-process-review-followups.md).
- **Blocker:** none for DX.3 acceptance.
- **Exact next action:** stop after this record update. Start DX.4 only in a new
  bounded session. Local unrelated modifications remain untouched.

Evidence host/root:
yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx3-evidence/.

| Artifact | Result / SHA-256 |
| --- | --- |
| evidence-index-63640031f7b66e067b2cbb26849277faed725668.json | Receipt/log digest index; `c5fc8c0bdf4cff0ad6f17031121561fb1a18969a590182b39f926cdc7be7e6df` |
| review-candidate-acceptance.log | Eight Rust acceptance/regression tests PASS |
| remediation-final-process.log | Ten process/reporting regression tests PASS; unchanged Python inputs reused |
| consumer-63640031f/consumer-receipt.json | Baseline/final cold/warm actual consumers, umask 002, full user output PASS |
| consumer-63640031f/failure-injection-receipt.json | Actual missing-payload recovery and positive pressure cleanup PASS |
| review-candidate-file-size.log | 3954 files checked; 900-line guardrail PASS |
| review-candidate-clippy.log / review-candidate-fmt.log / review-candidate-budget.log | Production Clippy, formatting and unchanged budget parser checks PASS |
| reviews/63640031f7b66e067b2cbb26849277faed725668/response.md | Final read-only Opus review SATISFIED |

## Historical Handoff — DX.4 (2026-09-17)

Superseded by the DX.5 handoff below; do not restart DX.5 from this historical record.

- **State:** complete and merged in [PR #3846](https://github.com/sifr-lang/sifr/pull/3846).
  Candidate `fa8083ccfea3ec1b8d0fe8a5290c9a3d21efe834`; merge
  `41675c336e562ff43c4339630097a4533d321de7`; base
  `d28e84b64ad911cba41e553654f69681237060b1`. Owned remote checkout:
  /home/yaser5/projects/sifr/compiler-dx-orchestration; implementation branch
  codex/dx4, record branch codex/dx4-record.
- **Scope:** shared area fixture selection/preparation/reporting, Cargo-authoritative
  binary resolution, exact failed-case reruns, independent failure collection and
  dependent blocked outcomes, explicit native application profiles/assertions,
  existing-path comparison hooks, diagnostic examples and safe fix/recheck.
  No metadata-reader qualification, CLI-default change or DX.5 implementation.
- **Named validation:** E07, R04, R05, R07, R08 and existing-path Q01 PASS.
  Final foundation run re-executes all relevant earlier harness seeds and 16 DX.4
  seeds, including missing-tool/timeout negative controls, blocked-independent
  collection, warm assertion execution, independent wrong-result detection,
  metadata-check versus native-link/runtime failure separation, exact selection,
  declared exit-2 blessing, structural/add/change/remove input identity and
  consumed-document/config versus unconsumed-prose/phase-record controls.
- **Actual consumers:** safe text edit applies and the next document version
  rechecks; 12 diagnostic recovery tests, 148 codegen/IR invariant tests and
  185 diagnostic variants pass, including five executable error-document/explain
  pairs. Seven codegen smoke gates include two explicit release compile/link/run
  cases with independent expected stdout. A real warm artifact rerun still
  executes both runtime assertions. Per-case stage outcomes remain in reports.
- **Evidence reuse:** these compiler/diagnostic/native executions are retained
  from their actual earlier candidates through the final `reused-evidence.json`
  chain. Only affected runner/inventory controls were rerun after their changes.
  No unchanged execution is relabeled as a fresh run. File-size and full-base
  whitespace checks pass; retained formatting/HIR checks cover unchanged inputs.
  The installed uv launcher mismatch (0.12.5 versus pinned 0.12.10) is preserved;
  validation used the existing locked verification virtual environment through
  the canonical Python entrypoint.
- **Before/after:** all 2,436 original source fixtures and original area-manifest
  assertions remain selected; profile manifests are unchanged. Added assertions
  are diagnostic examples and native release/runtime companions. Final inventory
  has 52,276 inputs, including arbitrary generator assets, vendor/demo inputs,
  submodule identities/dirty contents and actual declared document/config inputs;
  no prior inputs were removed and no phase records are selected. Final
  `assertion-inventory-diff.json` and `changed-paths.json` carry the exact
  candidate's full changed-path inventory; historical copies remain intact.
- **Review/adjudication:** final scoped Claude Opus 5 review is SATISFIED, no blockers.
  [Published response](https://github.com/sifr-lang/sifr/pull/3846#issuecomment-5712346406);
  raw response SHA-256
  `742d68130962e8ea4ecde8cc6fe1553570d275686b2bf9ce5dac46457e573559`.
  Initial NOT SATISFIED reviews identified the expected-exit blessing regression
  and incomplete inventory authority. Parent adjudications explicitly authorized
  bounded root-cause remediation beyond the default review loop: structural
  ownership/submodules, actual consumed docs/configs, and taxonomy-declared root
  inputs. Intermediate SATISFIED reviews retain their original nonblocking
  classifications; their related input omissions were repaired. All responses,
  failed intermediate evidence and adjudications remain outside the Git tree.
- **Storage/policy:** the useful approximately 127 GiB owned target was retained
  with approximately 207 GiB free; no size-only cleanup. No intermediate
  create-PR/merge full gate was run or claimed. One full gate remains required on
  the final phase implementation; actual release qualification is conditional
  on a release request.
- **Follow-ups:** [separate fixture/inventory review observations](ad-hoc-dx4-fixture-review-followups.md).
- **Blocker:** none for DX.4 acceptance.
- **Exact next action:** stop after this record update. Start DX.5 only in a new
  bounded session. Local unrelated modifications remain untouched.

Evidence host/root:
yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx4-evidence/.
Final candidate directory: `fa8083ccfea3ec1b8d0fe8a5290c9a3d21efe834/`.


## Historical Handoff — DX.5 (2026-09-17)

- **State:** complete and merged in [PR #3848](https://github.com/sifr-lang/sifr/pull/3848).
  Final candidate `c7734025c242631a326901ce08c846ac42afd7ae`; merge
  `22a72c6c6c9f95242e3e60ba83e473f0edd60d11`; base
  `3f070fcdbc8a205104291d32b91edd8e3c59b12f`. Sole owned remote checkout:
  `/home/yaser5/projects/sifr/compiler-dx-orchestration`; implementation branch
  `codex/dx5`, record branch `codex/dx5-record`.
- **Scope:** private v1 indexed container with explicit 134-record type/HIR,
  declaration, binder, source, descriptor, interop, fragment and template schema;
  canonical typed references and shared lazy decoded owners; bounded header,
  directory, digest, reference, source-range and graph validation; cache pressure
  evicts only unreferenced records. The checked [consumer/field inventory](../../../internal_docs/compiler_dx_metadata_consumers.md)
  records exact current source symbols plus mutation, clone and lifetime migration.
  No producer command, source-backed overlay migration or normal CLI/LSP metadata
  consumer was activated. DX.6/DX.7 retain those implementation boundaries.
- **Named validation:** I05, I06, M06 and M16 PASS on the exact clean final candidate:
  `RUSTUP_TOOLCHAIN=1.98.1 cargo test -p sifr_sysroot metadata:: -- --nocapture`
  runs 14 tests, including all record kinds/variants, generated valid type graphs,
  shuffled insertion ordering, relocation, same-name package/version/source
  declarations, binder/partial-view distinction, malformed headers/offsets/IDs/
  digests, mutation seeds, structural-cycle/depth rejection, nominal/mutual
  recursion, and active-handle retention/eviction.
- **Scaling evidence:** a 500-field class referenced by 10 versus 1,000 signatures
  produces 167,449 versus 677,209 encoded bytes. Demanded signature lookup retains
  11 versus 1,001 records without loading the class view; decoder allocation-budget
  charges are 150,688 versus 14,089,888 bytes. These are conservative bounds, not
  measured RSS or a product performance improvement claim.
- **Focused checks:** scoped `cargo clippy -p sifr_sysroot --all-targets -- -D warnings`,
  `cargo fmt -p sifr_sysroot -- --check`, `python3 scripts/check_dx_metadata_inventory.py`,
  `python3 scripts/check_hir_maintainability_guardrails.py` and `git diff --check`
  pass. The touched-source file-size audit passes; the largest source is 790 lines.
  Existing Cargo dependencies were retained; the intentional lockfile change adds
  the already pinned `serde_json` dependency to `sifr_sysroot`.
- **Review:** read-only Claude Opus 5 returned SATISFIED with no blocking findings.
  [Published review](https://github.com/sifr-lang/sifr/pull/3848#issuecomment-5712915895).
  The reviewer independently checked record-kind coverage, IR enum variants and
  the Type mapping in addition to the checked-in inventory guardrail. Review
  evidence is outside the approved Git tree, keyed by final candidate SHA.
- **Storage/policy:** approximately 202 GiB was free with a useful 127 GiB owned
  target before the focused work; no cleanup was required. All implementation,
  checks and review ran on the owned Linux host; the unrelated local checkout was
  untouched. No intermediate full create-PR/merge gate was run or claimed. The
  single final-phase merge gate remains required; no release was requested.
- **Follow-ups:** [separate nonblocking metadata review observations](ad-hoc-dx-metadata-review-followups.md)
  retain encoder/decoder limit alignment, future enum/Type drift coverage and
  pressure-path accounting observations under their later owning milestones.
- **Blocker:** none for DX.5 acceptance.
- **Exact next action:** stop after this record update. Start DX.6 only in a new
  bounded session. Do not repeat unchanged DX.5 validation or review.

Evidence host/root:
`yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx5-evidence/`.
Final candidate directory: `c7734025c242631a326901ce08c846ac42afd7ae/`.

| Evidence | SHA-256 / result |
| --- | --- |
| `receipt.json` | `081e9b0cf4af349245dae2a689fa12357d2c9116dc4e15d990fdce1f86dcd276`; exact commands, exits and per-log digests |
| `check-0.log` | 14 named/focused metadata tests PASS, with M16 size/retention counters |
| `check-1.log` through `check-5.log` | Scoped clippy, formatting, inventory, HIR maintainability and whitespace checks PASS |
| `file-size.json` | Touched hand-maintained first-party source files below 900 lines; maximum 790 |
| `review-response.md` | SATISFIED; `71aea7ddf381607ee9b137b477827efa89406ae396509188d839698f05a2b5bd` |
| `review-prompt.md` | `3573ce41dd2ea8c15e506b3180e652d5099f7a5a06f7c18deaafd01823fab10f`; exact base/candidate, paths, scope and evidence |

The earlier implementation attempts and lint/test receipts remain in the evidence
root with their original outcomes. Only the final candidate directory is closure
validation evidence; no earlier outcome is relabeled as a fresh final-candidate run.


## Historical Handoff — DX.6 (2026-09-17)

- **State:** complete and merged in [PR #3850](https://github.com/sifr-lang/sifr/pull/3850).
  Final candidate `5581af0884edcb46d88419b9b7d66d1ce97531df`; merge
  `d53899694e8b65f47f7105fda34fec039459549a`; base
  `03c4b0d0671afe7e96f054cf0a39dcab63e56db9`. Sole owned remote checkout:
  `/home/yaser5/projects/sifr/compiler-dx-orchestration`; implementation branch
  `codex/dx6`, record branch `codex/dx6-record`.
- **Scope:** canonical 89-module source producer with exhaustive typed semantic,
  HIR, generic/default, interop and Rust payload projection; one write-through
  ensure for development launch, linked driver tests, verification preparation and
  native packaging. Successful owners are keyed by compiled compiler/test,
  semantic target and complete captured input identities. Per-key OS locks,
  complete validation, source recapture, fsync and atomic publication coordinate
  callers and preserve retryable failures. No DX.7 metadata consumer was activated.
- **Named validation:** M09, M12–M14 and the full inventory producer pass in
  `cargo test -p sifr_driver --lib dx6_ -- --nocapture`: 11 passed, one ignored
  subprocess-protocol helper (exercised by its parent acceptance tests). This
  covers exact repeated output bytes, checked-source semantic/HIR/Rust parity,
  initial absence, stale/corrupt/missing entries, strict overrides, source mutation,
  four concurrent threads, four cold/four warm processes, two independent
  configurations, producer death/failure/cancellation, read-only source, writable
  cache, and output errors. Every successful waiter runs independent positive and
  negative checking assertions; identical-family cold requests publish once.
- **R10:** full bare `cargo test -p sifr_driver` ran with both normal debug/release
  CLI binary paths unavailable and no injected verification environment. Empty
  matching metadata cache: 649 passed / 78 ignored, 266.45 s wall, 1,941,448 KiB
  maximum RSS. Prepared cache: the same 649 passed / 78 ignored, 242.41 s wall,
  1,821,740 KiB maximum RSS. Regenerated and prepared artifact identities match the
  linked preparation report. Normal generated-program Cargo tests remain intact;
  no recursive Cargo or preexisting CLI is used by the metadata producer.
- **Configuration evidence:** the real CLI produces distinct metadata identities
  for the four supported semantic targets and reuses each warm entry without
  production. Actual Cargo-reported executable preparation passes. Native Linux
  fixture packaging and archive verification pass; separately producing from the
  staged source snapshot gives exactly the source-tree metadata identity.
  Host/target execution distinctions are preserved; no foreign binary was run.
  These are observed preparation measurements, not a latency-contract or native
  qualification claim for the other three targets.
- **Focused checks:** constructor-shaped nominal-view decoder regression,
  verification adapter policy tests, descriptor/foreign-binary/error tests and
  existing all-target/stable/corrupt-archive distribution fixtures pass. Scoped
  driver clippy, generated-encoder check, exact site/schema inventory, file-size
  guardrail, format and diff checks pass. Unchanged schema/Python/fixture inputs
  reuse their recorded evidence; real CLI/native packaging and driver acceptance
  were validated on the final candidate.
- **Review:** read-only Claude Opus 5 returned SATISFIED with no blocking findings.
  [Published review](https://github.com/sifr-lang/sifr/pull/3850#issuecomment-5717512054)
  is keyed to the exact candidate; its original response and SHA-256 receipts
  remain outside the approved Git tree. Nonblocking observations are retained in
  the [metadata review follow-up issue](ad-hoc-dx-metadata-review-followups.md).
- **History/storage/policy:** the remote interruption, initial producer ordering
  failure, initial lint failures and one CLI test-harness package-boundary error
  remain historical evidence, not passes. About 202 GiB was initially free with
  the useful 128 GiB private target; 183 GiB remained during final native packaging.
  No pressure cleanup was necessary. All implementation, tests and review ran
  remotely; the unrelated local checkout was untouched. No intermediate full
  create-PR/merge gate or release qualification was run or claimed. The single
  final-phase gate remains required.
- **Blocker:** none for DX.6 acceptance.
- **Exact next action:** stop after this record update. Start DX.7 only in a new
  bounded session. Assess its recorded suggestions against DX.7 scope; do not
  repeat unchanged DX.6 validation or review.

Evidence host/root:
`yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx6-evidence/`.
Final candidate directory: `5581af0884edcb46d88419b9b7d66d1ce97531df/`.
The root `final-evidence.json` and candidate `evidence.json` enumerate exact
configuration identities, timing reports and per-file SHA-256 receipts.

| Evidence | SHA-256 |
| --- | --- |
| `driver-final.log` | `796f721fbe2ab5d46e8bacd1289b299e8c045842a9980d4a3860e4c6fd708b26` |
| `bare-empty.log` | `fb7149e613a8c44a6fa8bcab93d05159f0c3a7aae039b958bbef982054c3ea36` |
| `bare-warm.log` | `15f01ceda9d95350b7a3d67694050c18b369edb5613bbdc5e68b2c59f205dd82` |
| `cli-configurations.json` | `0a270530f90ebbcd37c4ab39b6bf63e97a617199c1ecb4c25c44f0dc79e44731` |
| `native-package-parity.log` | `3d6572f615914a11d79d93d51e1620de283216959fa1be031139ba8435a94533` |
| `clippy-focused-final.log` | `88ebf1ad1b5ef5f9ee922ee89a4175f4918b402a3e35f90b09c88e869f2012f9` |
| Original Opus response | `ae9dab23539f905748bda3efbce525c74e7c3fd74b2c9b6208e3d70c09d2afb2` |


## Historical Handoff — DX.7 (2026-09-17)

- **State:** complete and merged. Implementation [PR #3852](https://github.com/sifr-lang/sifr/pull/3852);
  final candidate `036e69592164b87d309f31ba43ee406a58ed79ca`, merge
  `fb6d432be20d945e0266030b076dbc76ce4c8a50`; base
  `963eed6419d8947822030451017fde6212a1b86f`. Sole remote checkout:
  `yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/compiler-dx-orchestration`;
  implementation branch `codex/dx7`.
- **Source-first boundary:** checkpoint `e7e89bab2703d66256400aadfd112fc522b6f8af`
  established the immutable baseline and mutable project overlay over the source
  provider. Lookup/visibility, failed/deleted/repaired exports and two-project
  isolation passed before metadata activation. Export collection and invalidation
  mutate only project overlays; lowering performs no provider I/O.
- **Consumer boundary:** ordinary CLI, bare-test, frontend, analysis/LSP and
  codegen paths demand metadata closures through the existing semantic authority.
  Complete nominal fields/methods/variants share immutable storage; contextual
  mutation detaches. Artifact-local nominal projections are reused. Checking
  loads selected semantics, building loads selected HIR/Rust/templates, navigation
  reads selected source on demand, and installed operation has no source fallback.
  Transformed/specialized/final Rust validation remains active. Transient selection
  failures are retryable and explicit artifact overrides have separate owners.
- **Named/focused validation:** M01, M02, M04 and M17 pass, including decoded/read
  counters and 1,000 nominal references sharing one projection without mutation
  leakage. Selected runs pass: source sharing 1, baseline parity 4, source overlay
  2, driver DX.7 acceptance 6, defaults 15, typed descriptors 11, stdlib exports 4,
  analysis 9, LSP 5, syntax validation 6 and source-map origins 1.
  Format, generated encoder/decoder, exact inventory, file-size/maintainability
  and diff checks pass on the final candidate.
- **Installed/native/M15:** six representative math, bisect, collections,
  datetime, JSON and regex programs preserve exact public emit bytes, final
  newlines and stdlib preamble markers against both source production and the
  installed source reference. Twelve actual source/metadata native assertion
  executions and the installed native test runner pass. Selected-source removal
  does not break checking; missing, corrupt and incompatible metadata reject;
  restoration retries successfully. The installed report has 30 command rows.
  Full corpus/target qualification remains DX.8.
- **Q09:** the same reference host, Rust 1.98.1 and optimized installed artifacts
  execute the frozen DX.1 sources. Each primary lane has 21 fresh checks and
  21 LSP open/diagnose/close samples, with the first warmup excluded; separate
  21-sample allocator lanes do not supply latency claims. Fresh checks improve
  10.76% (1,168.256 → 1,042.541 ms); the LSP sequence improves 10.67%
  (1,850.225 → 1,652.757 ms), with timing CV below 1.3%. Candidate peak stays
  below the scoped 128 MiB cap. Every sample preserves the exact existing
  `SIFR-PACKAGE-0103` diagnostic in the frozen DX.1 workload; this is not a
  clean-package editor correctness claim or a new expected-difference baseline.
- **Memory evidence:** the following medians are paired measurements, not a
  historical RSS estimate. Candidate counters show 13 semantic modules and
  13 shared nominal projections, with zero HIR-module or Rust payload reads.

| Q09 metric | Source reference | Metadata candidate | Delta |
| --- | ---: | ---: | ---: |
| Steady/post-close RSS | 128.262 MiB | 65.350 MiB | −49.05% |
| Peak RSS | 128.264 MiB | 95.766 MiB | −25.34% |
| Steady allocated chunks | 89.364 MiB | 44.054 MiB | −50.70% |
| Post-close allocated chunks | 72.234 MiB | 43.258 MiB | −40.11% |
| Peak allocated chunks | 98.948 MiB | 82.823 MiB | −16.30% |
| Empty allocated chunks | 0.070 MiB | 0.177 MiB | +0.107 MiB |

The separate glibc `mallinfo2` sampler measures allocated chunks/direct mappings,
including allocator/thread-cache retention; it is not exact object attribution.
The conservative 58,952,288-byte decoded bound is a separate counter, not a heap
measurement. Reduced retained state is consistent with demanded semantics and
shared nominal payloads. The pinned metadata container/directory, transient
record decoding and allocator high-water retention remain; peak improves less
than steady. Empty allocation grows slightly in absolute terms.

- **Review:** read-only Claude Opus 5 returned SATISFIED, no blocking findings,
  for the exact candidate. [Published review](https://github.com/sifr-lang/sifr/pull/3852#issuecomment-5719741485).
  Nonblocking observations and prior DX.6 dispositions remain in the
  [metadata follow-up issue](ad-hoc-dx-metadata-review-followups.md).
- **History/storage/policy:** earlier compile/fixture failures and superseded
  incomplete `product-e824bd` preparation remain historical, not passes.
  The first installed capture used an outside-package fixture with the repository
  as cwd and failed; only its external harness cwd was corrected for the passing
  v2 capture. Final installed preparation took 741.18 s; warm build artifacts
  were reused, with about 150 GiB free and no pressure cleanup required. All
  implementation, tests and review ran remotely; local source was untouched.
  No intermediate full create-PR/merge gate, clippy gate or release qualification
  was run or claimed. The single full phase-end gate remains required.
- **Blocker:** none for DX.7 acceptance.
- **Exact next action:** stop after this record update. Start DX.8 only in a new
  bounded session. Reuse unchanged DX.7 evidence; do not repeat its review.

Evidence host/root:
`yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx7-evidence/`.
The root `final-evidence.json` indexes candidate
`036e69592164b87d309f31ba43ee406a58ed79ca/evidence.json`, including exact commands,
artifact identities, raw reports, scripts and per-file digests.

| Evidence | SHA-256 |
| --- | --- |
| Candidate `evidence.json` | `ca2da8bf220515f694c8043a41be008760fd87022fdc7e719c2415ec4fb9ae1b` |
| `installed-036e69-v2/report.json` | `19aef4c93dd47491baccb7adc5b2a5bde9cab1c40fc67398c71e62900c798e45` |
| `q09-comparison.json` | `ebd89eddce113fc6252310baf86fb064b8e9b31ab8decb9ac16c6fb225aa3861` |
| Original Opus response | `1e159c464005c26e3df1beb40711b6783b78b2878f7cb08e9173be4fb1e91d79` |


## Historical Handoff — DX.8 (2026-09-18)

- **State:** complete and merged. Implementation [PR #3854](https://github.com/sifr-lang/sifr/pull/3854);
  final candidate `78f12c5cdd357f1601926f3fa78ec96f004108dd`, merge
  `1b55710f4a6e1d796be243c025b673f8958de72b`, base
  `e96028d97fdc2efe4389485dfcb88981d17f32a5`. Sole implementation/review checkout:
  `yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/compiler-dx-orchestration`,
  branch `codex/dx8`. This record uses `codex/dx8-record`.
- **Metadata/corpus:** M03, M05, M10, M15 and Q05 pass. Complete qualification
  traverses 89 canonical modules and 50,410 records for each semantic target.
  Each of the four target corpora passes all 727 exact source-versus-metadata
  Rust/source-map comparisons, including final newline and preamble positions.
  The corpus references 83 modules; six are structurally qualified without corpus
  references (`_sifr.python`, `sifr.meta`, `sifr.python`, `sifr.python_core`,
  `sifr.sql`, `sifr.sql.migration`). This does not claim exhaustive API or generic
  instantiation coverage.
- **Behavioral oracle:** the full native maintained corpus passes 727/727
  (185 groups, signature `ca4467fa7af12bd4`). After canonical NaN repair,
  `native-oracle-reuse.json` proves unchanged exact emitted Rust, source maps,
  fixture/runtime/runner inputs and module sets. Only unordered coverage-report
  module arrays are compared as sets; Rust and metadata are never normalized.
  Four focused installed native programs also pass.
- **Portable identity/M11:** actual native Linux x86_64 and actual native macOS
  arm64 producers, both Rust 1.98.1, produce byte-identical complete portable
  record streams for matching explicit semantic targets:
  `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
  `x86_64-apple-darwin`, `aarch64-apple-darwin`.
  Every record ID/kind/length/payload, including semantic/type/declaration
  identities, participates. Only the compiler header envelope and
  `FragmentValidation.compiler_identity` are excluded; an independent parser
  validates offsets/digests and matches the compiler digest before complete
  streams are compared. Actual cross-host qualification exposed CPU-dependent
  NaN HIR bits; the existing constant-folding owner now emits a canonical quiet
  NaN before identity construction. No semantic identity field is masked.
  Compiler envelopes remain strict. Native behavior was qualified on Linux;
  no foreign target binary was executed.
- **Mac exception:** the user authorized only M11 qualification in isolated
  `/tmp/sifr-dx8-m11-checkout`, private target `/tmp/sifr-dx8-m11-target` and
  evidence `/tmp/sifr-dx8-m11-evidence`. The existing local user checkout was
  untouched. The final candidate rebuilt a byte-identical native Mac binary and
  revalidated all four original metadata artifacts (`mac-final-78f/report.json`).
  All implementation, other tests and scoped review ran remotely.
- **Installed generations/M07/M08:** atomic selection of retained immutable
  generations pins matching compiler/metadata/source payloads for live sessions.
  Real LSP old/new generation navigation and rollback pass, as do archive
  relocation, two-store isolation and installer/self-update assertions.
  Receipts retain the stable install directory but pin physical binary/sysroot
  generation paths. Unsupported mutable roots reject with an empty-root remedy;
  no backward migration or retroactive live-session pinning is claimed.
- **Installed emission and recovery:** all 727 public `emit` stdout/newline/
  preamble/source-map comparisons passed in `installed-5628e142a/report.json`.
  That invocation later failed on an unsupported standalone `--offline` flag;
  its failed outcome remains preserved. The corrected committed helper uses
  Cargo's offline environment with ordinary standalone execution.
  `installed-native-current/report.json` passes native assertions, live LSP,
  missing/corrupt/incompatible rejection, restoration and lazy checking with
  private navigation source absent on the same qualified artifact bytes.
  `final-package-equivalence.json` proves all final packaged payload files
  unchanged except the manifest build-commit provenance label. Final installed
  doctor integrity and full metadata validation pass. These are explicitly
  joined successful stages, not a relabeled failed invocation.
- **R09/profile regressions:** development doctor missing/ready/corrupt/recovered
  behavior, identity and remedy text, and no implicit production/repair now have
  a committed real-CLI regression in the existing `metadata-structural` suite.
  Complete structural and corpus tiers remain profile-wired. The distribution
  representative runner passes all 59 cases, including the executable immutable
  generation fixture and generation-pinned receipt assertions, with pinned Node
  26.8.2/npm 12.0.2. Format, maintainability, diff and 900-line guardrails pass
  (4,046 files).
- **Q09:** fresh paired optimized installed cohorts use the unchanged frozen
  DX.1 workload on the reference Linux host. Each primary lane has 21 fresh
  checks and 21 LSP sequences, first warmup excluded; separate 21-sample heap
  cohorts do not supply latency claims. LSP median improves 9.88%
  (1,841.644 → 1,659.732 ms); fresh checking improves 10.40%
  (1,160.407 → 1,039.678 ms). Timing CV is below 2%; maximum candidate peak
  100,642,816 bytes remains below 128 MiB. All samples retain 13 demanded
  semantic modules/13 shared nominal projections and zero HIR/Rust payload reads.

| Q09 metric | Source reference | Metadata candidate | Delta |
| --- | ---: | ---: | ---: |
| Steady/post-close RSS | 128.359 MiB | 65.303 MiB | −49.13% |
| Peak RSS | 128.359 MiB | 95.713 MiB | −25.43% |
| Steady allocated chunks | 89.364 MiB | 44.051 MiB | −50.71% |
| Post-close allocated chunks | 72.233 MiB | 43.255 MiB | −40.12% |
| Peak allocated chunks | 98.721 MiB | 82.819 MiB | −16.11% |
| Empty allocated chunks | 0.070 MiB | 0.174 MiB | +0.104 MiB |

The frozen workload retains its exact existing `SIFR-PACKAGE-0103` diagnostic;
these measurements do not establish clean-package editor correctness. The
separate glibc `mallinfo2` lane includes allocated chunks/direct mappings and
allocator/thread-cache retention, not exact object attribution. Reduced retention
is consistent with demanded semantics/shared nominal payloads; pinned container
state, transient decoding and allocator high-water retention remain.

- **Review/remediation:** read-only Claude Opus 5 first identified a missing
  executable bit and absent committed development-doctor regression. The bounded
  correction also reconciled a stale mutable-root receipt assertion exposed by
  the real area runner. A first area run with that stale assertion and wrong
  ambient Node remains failed; the corrected pinned-toolchain run passes.
  Final [Opus review is SATISFIED, no blockers](https://github.com/sifr-lang/sifr/pull/3854#issuecomment-5722224549).
  Nonblocking observations are recorded in the
  [metadata follow-up issue](ad-hoc-dx-metadata-review-followups.md).
- **Evidence reuse/policy:** the final four-file remediation changes only
  verification files. Compiler, semantic/source/runtime/lock/vendor/installer
  and corpus inputs are unchanged from qualified product candidate
  `53e9a8f05ccebd794311cef39c34c0059ceb98b4`; its immutable receipts retain that
  provenance. Final wrapper `final-evidence-78f.json` adds the focused corrections
  and actual final Mac audit. Earlier failed/incomplete runs remain historical.
  Pressure checks retained about 100 GiB free during final validation; no
  foreign/shared target cleanup was needed. No intermediate full create-PR/merge
  gate or release qualification was run or claimed. The full phase-end gate
  remains required; no release was requested.
- **Blocker:** none for DX.8 acceptance.
- **Exact next action:** stop after this record update. Start DX.9 only in a new
  bounded session; no DX.9 implementation belongs to this handoff.

Evidence host/root:
`yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx8-evidence/`.

| Evidence | SHA-256 |
| --- | --- |
| Final `final-evidence-78f.json` | `2ac0d40ea6b501011a961145e613820d3fd48d031e52a7c3c57d59de492d5f0f` |
| Product `qualification-manifest.json` | `59c1578ad47ba67ef3898ca5852b7d8c579c3c56db9ff486aa85270cb2aa23b1` |
| `q09-comparison.json` | `df4a08b07b756baac0b11af320040c2cc690c101b5497c0203c2155a5056bbd3` |
| Final original Opus response | `3f859a2ef2524dda2648215792aba88033c44a04b666d0b5c4924f82316a0fb9` |

## Historical Handoff — DX.9 (2026-09-18)

- **Current state:** DX.9 implementation is merged in [PR #3856](https://github.com/sifr-lang/sifr/pull/3856).
  Candidate `2f1f3fb215d5263c0d1fb49d8c3346aafa299b33`;
  merge `455400351b3d989490191a09b3cb3d2c817404fa`.
  This record-only update is on `codex/dx9-record`.
- **Implemented:** owned compatible native Cargo families, stable generated
  roots and root-specific target names; family/publication leases span source
  mutation and capture. Ordinary binary/test/probe requests ask Cargo for
  freshness, preserve unchanged input mtimes and return captured final bundles.
  Current source lock authority changes reseed normal generated roots without
  discarding their ordinary Cargo updates when authority is unchanged.
  Native defaults remain release/test; no DX.10 profiles or project generations.
- **Named acceptance:** I04, B01–B04 and B07–B09 pass on the exact candidate.
  Nine DX.9 cases cover real same-name compile/lint errors, concurrent mutation
  and capture across native contexts, linker/runtime failures, independent
  runtime-library capture, build-script changes, lock-authority changes and
  trust revocation after a real native cache hit. Focused materialization,
  portable lock, normal-resolution, probe, native-context and test-runner checks
  pass, as do the actual alternate Python loader process and generated typed
  Python integration. File-size/HIR guardrails, formatting and diff checks pass.
- **Installed qualification:** real build/run/test requests confirm Cargo
  freshness, native flag repartition, invalid-target failure/recovery and
  repeated failing tests executing again. Native and Python source exports are
  independently built offline with `--locked` and run. Python export declares
  its selected external interpreter/library paths and SHA256; exposing an
  alternate library produces the concrete loader-mismatch error. The unpushed
  exact Git candidate was supplied through command-local test transport;
  exported manifests/locks were not rewritten for qualification.
- **Bounded repair:** actual Python export qualification exposed an unresolved
  normal-mode workspace seed lock. Local Cargo metadata now resolves the
  generated root before source/lock freezing. The original failed export and
  its independent diagnostic copy remain preserved; a new permanent seeded
  root regression and the final real export pass.
- **Observed paired performance:** three measured pairs after discarding the
  first warmup pair, optimized installed compilers on the same Linux host:

| Native request | Before median | After median | Cargo work after |
| --- | --- | --- | --- |
| Cold | 12.628 s | 12.707 s | 32 rebuilt units |
| No-op | 0.252 s | 0.513 s | 0 rebuilt, 32 fresh units |
| Edited | 12.389 s | 0.592 s | 1 rebuilt, 31 fresh units |

The prior edited request rebuilt 31 units; its no-op bypassed Cargo. The new
no-op cost includes the required current Cargo check. These are observed
latencies, not a universal host budget. The prior DX.8 installed compiler's
native/runtime sources match the phase base; later DX.8 changes were
documentation/verification only (`baseline-reuse-final.json`).

- **Review:** one read-only Opus 5 review returned
  [SATISFIED, no blockers](https://github.com/sifr-lang/sifr/pull/3856#issuecomment-5723374425). Suggestions are separate work in
  [native reuse follow-ups](ad-hoc-native-cargo-reuse-followups.md).
- **Policy and preserved failures:** no intermediate full create-PR/merge gate
  or release qualification was run or claimed. Supplemental feature-enabled
  Clippy still reports 498 pre-existing metadata/Python errors, with zero
  errors in changed source files; their owning issues retain the findings.
  Earlier failed/incomplete fixture, loader-environment, lint, test-transport
  and portable-lock attempts remain in the manifest. They are not passes.
  Validation retained at least 83 GiB free before final product preparation;
  no shared/foreign target cleanup was performed.
- **Blocker:** none for DX.9 acceptance.
- **Exact next action:** stop after this record update. Start DX.10 only in a
  new bounded session. The full phase-end gate remains required.

Evidence host/root:
`yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx9-evidence/`.

| Evidence | SHA-256 |
| --- | --- |
| `qualification-manifest.json` | `deedd24f44e6ada7896c963af6ce2a442a6e1df93351fd72d1ee8eeb187a62ac` |
| Original Opus response | `38c3078487ec037b914c18cdde22057a78ad2e96315113e62dc90b49dca462d7` |


## Historical Handoff — DX.10 (2026-09-18)

Historical next-action instructions below are superseded by the current DX.11
handoff. Original validation, failed observations and review history are retained.

- **Current state:** DX.10 implementation is merged in
  [PR #3858](https://github.com/sifr-lang/sifr/pull/3858).
  Candidate `d4b80812086538f6d26bf7830f30da54d52bffb0`;
  merge `65e870fced69b02c6833fa9f000d31d8c1bbefb8`.
  This record-only update is on `codex/dx10-record`.
- **Implemented:** default build/run select development applications, test uses
  the test profile, and explicit `--release` selects optimized applications.
  Generated manifests pin overflow/unwind behavior; captured overrides and
  actual generated-manifest workspace authority are checked. Native profile
  identity/reports follow Cargo's actual application configuration. Generated
  applications and bridge probes own their workspace. The distributed compiler
  remains optimized independently of application selection.
- **Grouping and retained selection:** compatible compiler integration
  preparations share the execution configuration and memoize duplicate builds.
  Minimal, feature, target and native exceptions remain isolated; before/after
  inventories retain every original assertion. Representative prepared/executed
  binaries and assertion lists match. E2E/native aliases and standalone release
  consumers retain explicit release; consumers use finalized `target/final`
  artifacts. Default-development tests are additional.
- **Named acceptance:** B05, B06, B11, B12 and R06 pass. The complete declared
  release run-pass corpus executed **727/727 fixtures through 175 hashed actual
  release native binaries**; an additional 5/5 development fixtures used debug
  artifacts. Installed compiler qualification covers actual dev/test/release
  overflow, assertion, error, ownership and async cleanup semantics, FFI panic
  mapping/redaction and call-scoped callbacks. The permanent caller-authority
  regression executes default/release run/test under both unsafe and malformed
  unrelated Cargo manifests with a Rust-backed HTML probe/cache nested beneath
  them (eight real requests).
- **Final consumer qualification:** 13 focused consumer/profile tests and eight
  generated preparation/selection tests pass. Twenty-one successful actual
  operations include 16 observed native release application profiles, benchmark
  final-artifact measurement/execution, sysroot boundary/compile and attached
  API execution/edit invalidation, Python live-package compilation, differential
  preparation/execution, and representative golden/hardening/namespace/readiness
  and metadata/capture commands. Actual Cargo receipts show opt3, debug assertions
  false and overflow checks true. Python live qualification did not start an
  external service. File-size and relevant formatting/syntax/diff guards pass.
- **Observed profile costs:** optimized installed compiler, 12 GB Linux host,
  Rust 1.98.1. Two isolated samples cover dev-only, release-only and alternating
  cold/no-op/edit/warm-edited requests. Sample 0 is retained but excluded from
  comparison; sample 1 is below. GNU time child RSS and 50 ms sampled summed
  process-tree RSS are recorded; shared pages count per process, not as PSS.

| Sample 1 workload | Cold seconds / rebuilt units | No-op seconds / rebuilt units | Edit seconds / rebuilt units | Warm edited seconds / rebuilt units | Allocated cache |
| --- | --- | --- | --- | --- | --- |
| Development only | 8.789 / 31 | 0.638 / 0 | 0.782 / 1 | 0.638 / 0 | 136.0–142.6 MiB |
| Release only | 12.323 / 32 | 0.615 / 0 | 0.741 / 1 | 0.616 / 0 | 119.3–124.7 MiB |
| Alternating, development | 8.749 / 31 | 0.630 / 0 | 0.780 / 1 | 0.669 / 0 | shared row below |
| Alternating, release | 11.976 / 32 | 0.625 / 0 | 0.735 / 1 | 0.620 / 0 | 248.7–261.8 MiB shared |

Cold peak summed RSS is about 653.8–703.2 MiB; edits about 218.4–248.0 MiB.
Alternating profiles retain 31+32 distinct cold configurations; every warm
return rebuilds zero units and each edit rebuilds one application unit. These
are observations for the small scalar-edit workload, not universal latency
budgets. The full release corpus's 5927.62 s cold serial run is correctness
evidence, not a host-sensitive performance comparison.

- **Review and adjudication:** the initial review found the release size consumer
  and generated-manifest authority defects. The second confirmed authority was
  fixed but found the same consumer migration missing at other first-party
  entry points. Both remain
  [NOT SATISFIED in the preserved history](https://github.com/sifr-lang/sifr/pull/3858#issuecomment-5725480175).
  Execution stopped for the skill's repeated-finding rule; the orchestrator
  explicitly adjudicated bounded completion of that mechanism and one additional
  review. The final exact-candidate
  [Opus review is SATISFIED, no blockers](https://github.com/sifr-lang/sifr/pull/3858#issuecomment-5725527463).
  Suggestions, profile-agnostic inventory decisions and the size observation
  are separate work in [DX.10 followups](ad-hoc-dx10-profile-review-followups.md).
  External LeetCode consumer migration is recorded in
  [its owning issue #49](https://github.com/sifr-lang/leetcode/issues/49).
- **Evidence reuse:** final d4 is verification-only relative to qualified
  installed compiler candidate `ff1f3b84b2b0ba9f174b2623a1837e90ff086ffb`.
  Exact crate/runtime/stdlib/Cargo/vendor identities prove unchanged compiler,
  fixture and profile inputs. The ff1 receipts retain their real provenance;
  earlier corpus reuse has its own tree/blob proof. Final consumer receipts,
  inventory and the inherited manifests are hashed under the final candidate.
  No compiler rebranding or broad repeat is claimed.
- **Preserved observations:** the separate actual-ref whole-file non-increase
  check remains **failed**: 6,634,688 → 6,641,408 bytes, +6720 (+0.10%).
  `.text`, data and bss are unchanged; nonallocated debug/string metadata
  increased. The size assertion remains unchanged. Predetermined real decrease/
  increase controls passed expected exits 0/2; no whole-file non-growth is
  claimed as DX.10 acceptance. Both remediation reviews confirmed this separate
  budget observation is not a new named criterion. Initial pipeline assumptions,
  extended probe and other failed/incomplete attempts remain historical.
  Ambient uv 0.12.5 initially failed the Python area's 0.12.10 pin; an owned,
  external installation of the pinned version corrected only the invocation
  environment, and the resumed actual build passed.
- **Resource/policy:** the private compiler target exceeded 220 GiB without
  size-triggered cleanup; final qualification retained about 43 GiB free.
  No shared/foreign target or active application profile was evicted. No
  intermediate full create-PR/merge gate or full release checkpoint was run.
  Named native release correctness qualification was completed; no release
  publication was requested. The full phase-end gate remains required.
- **Blocker:** none for DX.10 acceptance.
- **Exact next action:** stop after this record update. DX.11 is eligible only
  in a new bounded session; no DX.11 implementation belongs to this handoff.

Evidence host/root:
`yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx10-evidence/`.
Final candidate directory:
`d4b80812086538f6d26bf7830f30da54d52bffb0/`.

| Evidence | SHA-256 |
| --- | --- |
| Final qualification-manifest.json | `c22780206faa6168cf7f92ede0b742f652c732c05969de33b96d1eb1a90d7e93` |
| Original final Opus response | `1c7a0acb69c19f56fdffec0b6ca899e27f89b98f37aa87457fd77b2b0b0c1857` |
| Consumer profile/invocation inventory | `2dd6593ed7a8265a25266f393b745e03e411d55e6c50dc2bb60f90476ae2923f` |
| Inherited ff1 qualification manifest | `3aab3b08c10fe71077c18932f5f37cb3a66a06ba0b3094710ed322e4bfdba369` |
| Complete native release corpus | `618e863ec3cdc4dfc5978d7e44d1d2acabf06eb8371962a2ac9fc2709049301b` |

## Historical Handoff — DX.11 (2026-09-18)

- **Current state:** DX.11 implementation is merged in
  [PR #3860](https://github.com/sifr-lang/sifr/pull/3860).
  Final candidate `c0d490d460ca4f8a286226da804b31744b5a0b3c`;
  merge `24eec746b3b72d1b3bdbbb073986092f629b1ab9`. This docs-only update is on `codex/dx11-record`.
- **Ownership:** implementation, tests and Opus review ran only on the reference
  Linux host in `/home/yaser5/projects/sifr/compiler-dx-orchestration`.
  Original base was `67005d246ef2c261b5d93bbf308df51ef033891c`.
  The private target was reused; >220 GiB alone did not trigger cleanup, and
  sufficient operation reserve remained. No local Mac implementation or tests ran.
- **Implemented:** immutable snapshots pin source and compiler metadata ownership;
  host identity rejects cross-host snapshot reuse even with coincident revisions.
  Incremental edits now update the authoritative overlay as well as the frontend
  context, so reload cannot restore older bytes. Ingress generations independently
  guard request responses and diagnostics, including pending close clears.
  Superseded work terminates with ContentModified, without relying on cancellation.
  Closed projects release semantic/metadata and Python environment/target owners.
  Configuration explicitly re-resolves toolchains while preserving open overlays.
  Syntax queries avoid semantic initialization, and physical aliases retain a
  single owner while results use the logical document URI.
- **Named acceptance and focused regressions:** nine Rust DX.11 tests pass,
  including overlapping snapshot lifetime/release, independent stale rejection,
  atomic invalid edit batches, aliases, syntax-only metadata isolation, closed
  environment retention, deferred publications and incremental push progress.
  Actual installed E01–E06 pass: CLI saved-source/editor overlay isolation,
  20/20 stale terminal responses, real symlink URI and Linux case identity,
  Unicode/CRLF UTF-8/16/32 edits, 12 close/switch cycles, syntax with missing
  metadata, and missing/incompatible same-process recovery. Protocol smoke passes.
  The final installed two-document push regression verifies zero edit-driven
  progress events and retained begin/end workspace progress.
- **Evidence reuse:** the final remediation changes only whether an incremental
  reconciliation starts work-done progress; it preserves publication scheduling
  and all ownership/analysis code. Earlier exact-candidate evidence retains its
  real `dd282f9d84e038e4739e73dacdbac400388c8c6f` provenance. The source diff and
  capability/mode analysis in `evidence-reuse.json` establish applicability:
  Q09 clients do not advertise progress, the large fixture has diagnostics off,
  and actual toolchain upgrade/configuration keeps the same workspace path.
  No old measurement is relabelled as a newly measured final binary.
- **Scoped budgets and recovery:** the applicable large-session smoke has
  42 operations, p95 **7.128 ms**, peak RSS **53.1 MiB**, passing unchanged
  1000 ms/128 MiB limits. That diagnostics-off fixture does not bound multi-document
  push-diagnostics cost. Actual installed transport verifies pinned A across a
  selector upgrade, B in a new process, explicit same-process adoption of B, and
  rollback to A. File-size, maintainability, formatting and diff guards pass.
- **Q09 paired memory evidence:** optimized installed baseline
  `ff1f3b84b2b0ba9f174b2623a1837e90ff086ffb` has identical compiler inputs to
  the base (proof retained). Four normal/heap baseline/candidate cohorts each
  retain 21 samples with predetermined first-sample warmup excluded. The frozen
  demanded DX.1 trace is extended equally with edit/close/switch. These medians
  describe the same 12 GB Linux host, Rust 1.98.1; the heap lane records glibc
  allocated chunks plus mmap, not requested object sizes or heap-lane latency.

| Q09 median | Baseline | Candidate | Delta |
| --- | ---: | ---: | ---: |
| Active steady RSS | 65.273 MiB | 65.426 MiB | +0.152 MiB |
| Peak RSS | 95.674 MiB | 95.801 MiB | +0.127 MiB |
| Post-close RSS | 65.510 MiB | 47.805 MiB | −17.705 MiB (−27.03%) |
| Switched active RSS | 65.930 MiB | 67.414 MiB | +1.484 MiB |
| Switched closed RSS | 65.930 MiB | 49.613 MiB | −16.316 MiB |
| Active steady allocated heap | 44.049 MiB | 44.051 MiB | +0.002 MiB |
| Peak allocated heap | 82.818 MiB | 82.818 MiB | unchanged |
| Post-close allocated heap | 43.264 MiB | 0.506 MiB | −42.758 MiB (−98.83%) |
| Switched closed allocated heap | 43.292 MiB | 0.538 MiB | −42.754 MiB |

Active/peak RSS has **no demonstrated improvement**: the same metadata
container/index and 13 demanded modules are still required, with zero HIR/Rust
payload reads. After final close the metadata owner disappears. Allocator arena
medians remain 24,276,992 bytes, supporting the attribution that retained allocator
arenas account for much of the gap between released chunks and RSS. Switched
active RSS rises while live heap falls, consistent with allocator/file-residency
high-water effects, not an additional decoded record set. These are scoped
attributions, not exact object accounting. Normal trace median elapsed time is
1850.087 → 1834.750 ms (−0.83%), primary noise status observed; fresh checks are
comparable, with no product-target claim. Original DX.8 pre-migration evidence
remains the earlier baseline, not replaced by this incremental comparison.

- **Review:** initial exact-candidate review was
  [NOT SATISFIED](https://github.com/sifr-lang/sifr/pull/3860#issuecomment-5726155969):
  multi-document incremental changes incorrectly opened workspace progress.
  One bounded remediation suppresses that scope on edit reconciliation,
  corrects the misleading comment and tests both edit/workspace behavior.
  Final [Opus review](https://github.com/sifr-lang/sifr/pull/3860#issuecomment-5726309175) is SATISFIED without blockers.
  Suggestions are separate work in
  [DX.11 editor review followups](ad-hoc-dx11-editor-review-followups.md).
- **Evidence:** external root
  `/home/yaser5/projects/sifr/dx11-evidence`; final candidate-keyed
  `c0d490d460ca4f8a286226da804b31744b5a0b3c/qualification-manifest.json`,
  SHA-256 `8b29cf3f8599e4fa10cbb3e4c8c1c6155da81ddabe3a60202a99332755dccbad`.
  Final optimized installed receipt: `product-final/receipt.json`.
  Final review SHA-256: `7078be8a2a3a1ada0dbf645a4e1a802a9eb3e3fae5e18abd1c20931d5e10e158`.
  Q09 report: `q09-comparison.json`,
  SHA-256 `52ea04a875a1381e3b1342556fd1116a35eb4a0df32abd247c22f1535a78a4e3`.
  The manifest hashes the final focused tests, actual installed acceptance/
  protocol/progress reports, guards and explicit reuse proof.
- **Preserved observations:** preliminary unit/harness/protocol failures and
  original review remain negative evidence. The original frozen Q09 source
  retains its pre-existing SIFR-PACKAGE-0103; clean semantics are separately
  qualified by E01–E06. The heap baseline's latency noise is not a timing claim.
  DX.10 F6 remains **failed +6720 bytes**, with unchanged assertion and historical
  artifacts in [its owning issue](ad-hoc-dx10-profile-review-followups.md).
- **Policy and next action:** no intermediate full create-pr/merge gate or
  unrequested release checkpoint ran. The phase-end full gate remains required.
  No DX.12/DX.13 persistence was implemented. Blocker: **none**.
  Stop after this merged record. DX.12 is next only in a new bounded session.


## Historical Handoff — DX.12 (2026-09-18; superseded by DX.13 below)

- **Current state:** DX.12 is complete and merged in
  [PR #3862](https://github.com/sifr-lang/sifr/pull/3862).
  Final candidate `cc2874a319feef6f96f45954b91dc3bed79cc2e8`;
  merge `3959f0168f0630802548ad2b12d3623ab71f151c`.
  This record-only update is on `codex/dx12-record`.
- **Ownership:** implementation, named tests and read-only Opus review ran
  exclusively in the owned Linux worktree
  `/home/yaser5/projects/sifr/compiler-dx-orchestration`, with Rust 1.98.1.
  Base was `f1d0e1d72ad107e816c16690cefddfb19df6200f`.
  The private target was reused under the pressure policy; sufficient operation
  reserve remained, so no size-triggered cleanup ran. Local Mac activity was
  limited to Git bundle submission and GitHub administration.
- **Implemented:** the frontend persistence boundary pins captured source reads
  and records ordered resolution probes, absence, directory membership and
  canonical path observations. Package capture uses the existing graph and
  source-map builders with the same provider and records resolved Cargo metadata,
  lock and manifest observations. Explicit semantic context includes declared
  external completeness. Independent result families distinguish completed
  diagnostics (including deterministic source errors) from HIR, codegen, editor
  and executable readiness.
- **Typed and diagnostic transport:** checked HIR and complete semantic export/
  body inventories use the existing bounded indexed codec with package/source
  declaration anchors and an explicit checked source table. Decode validates
  source bytes and family references. Canonical diagnostic facts remap byte
  ranges and suggestion edits into current source maps. Interfaces retain
  conservative exact-input stamps, including consumed dependency summaries.
  These are usable transport/capture contracts; cross-process disk reuse remains
  disabled and belongs to DX.13. Unpersisted syntax, flow and rich editor queries
  continue through their existing owners.
- **Named acceptance and focused evidence:** the exact final candidate passed
  `INSTA_UPDATE=no cargo test -p sifr_frontend -p sifr_package -p sifr_driver -p sifr_lsp --lib dx12 --no-fail-fast`:
  **14 passed, 0 failed**. I03 covers restored facts under UTF-8/16/32 presentation.
  P03/P04 cover absent and higher-priority imports, actual resolver cycles,
  source-set/deletion/root/configuration/lock edits and conservative dependency
  invalidation. P07 includes a real separate-process writer between source capture
  and analysis. All five families have serialization/remapping coverage,
  including byte-identical typed re-encoding, source-table/reference rejection,
  independent completeness and an ordered-observation golden.
- **Guards and lint limits:** formatting, file-size, maintainability and diff
  guards pass. Supplemental strict lint attempts still fail in unchanged owners;
  the complete owner-only JSON scan reports zero new/touched-line diagnostics.
  Existing failures remain recorded in the
  [metadata follow-up issue](ad-hoc-dx-metadata-review-followups.md) and
  [profile follow-up issue](ad-hoc-dx10-profile-review-followups.md).
  This does not claim a passing full lint or merge gate.
- **Review:** the first completed exact-candidate
  [Opus review](https://github.com/sifr-lang/sifr/pull/3862#issuecomment-5726861173)
  is **SATISFIED**, with no blocking findings. Four suggestions are separate work
  in [DX.12 semantic review followups](ad-hoc-dx12-semantic-review-followups.md).
  No remediation or second review was needed.
- **Evidence:** external root `/home/yaser5/projects/sifr/dx12-evidence`;
  candidate-keyed `cc2874a319feef6f96f45954b91dc3bed79cc2e8/qualification-manifest.json`,
  SHA-256 `f9df54c5c70206e12f477d1d91a30a275c0c31504656684e7f9d2c6f65791e5d`.
  Validation manifest SHA-256
  `14aa937944a74ab95f0cfc868aec1b99bb8db18da2cfba1bb642baf52510649a`;
  review SHA-256
  `91e9f228268edef441c8769b5f132f367d4d4cf026f99f0d95c429158c921cdf`.
  Earlier development/setup failures remain in the evidence root and are not
  relabelled as final-candidate passes.
- **Gate policy:** no full create-PR/merge gate ran for this intermediate item.
  The explicit prospective phase-end policy remains authoritative. No release
  was requested or performed; no performance or active-memory improvement is
  claimed by these contract tests.
- **Blocker:** none for the named DX.12 scope.
- **Exact next action:** stop after this record update. DX.13 is eligible only
  in a new bounded session; no DX.13 implementation belongs to this handoff.


## Historical Handoff — DX.13 (2026-09-18)

- **Current state:** DX.13 is complete and merged in
  [PR #3864](https://github.com/sifr-lang/sifr/pull/3864).
  Final candidate `be67cc8dfa7d6db91f295557f08123d705cc79ff`;
  merge `9ee1d65db9a1f1851f46032f641d39dc27460bc0`.
  This record-only update is on `codex/dx13-record`.
- **Ownership:** implementation, tests and read-only Opus review ran exclusively
  in `/home/yaser5/projects/sifr/compiler-dx-orchestration` on Linux with Rust
  1.98.1. Base was `72e9490fc43a809648947d8cf6cc9594a30abd53`.
  The private target was reused; 16–19 GiB free space provided the operation
  reserve, so no size-triggered cleanup ran. Local Mac work was limited to
  Git bundle transport and GitHub administration.
- **Implemented:** a new CLI process restores completed shallow checking through
  the frontend canonical result contract after validating compiler, pinned
  metadata, resolved package context, captured sources and ordered resolver
  observations. Pure packages require the actual generated empty interop demand
  attestation; configured components, Python/native/external contexts retain
  ordinary owner execution. Deterministic source errors restore separately from
  success; transient, cancelled and changed-input work does not publish.
- **Generation protocol:** private stages, immutable inherited hard-linked or
  copied records, bounded digest-checked manifests, atomic publication, shared
  reader leases and pressure-driven project-only GC preserve A/B/A eligibility.
  Project leases no longer leak into unrelated child processes. Optional cache
  failure preserves semantic results; workspace hints remain optional and
  `--no-incremental` executes ordinary checking without project publication.
  Typed HIR, codegen and editor families remain explicitly Pending and compute
  through their normal owners when requested. No DX.14 propagation/editor work
  was implemented.
- **Named acceptance and focused evidence:** the exact final candidate passed
  C04, C08, C09, P01, P02 and P08–P10/P12: 12 project-cache tests with actual
  semantic payloads, partial ENOSPC injection, read-only/unavailable storage,
  killed writer subprocesses, active-reader/GC contention, immutable inheritance,
  error/fix/revert, cancellation/transient exclusion and source-map remapping.
  Focused regressions passed: 3 DX.12 driver, 8 DX.3 ownership and 9 frontend
  persistence tests, CLI schema/global tests. The acceptance runner made 31
  separate CLI invocations, including actual package imports/dependency edits,
  restored canonical error equality, live-context exclusion, hint removal and
  missing deeper-family computation. Format, file-size (4075 files),
  HIR maintainability and diff guards passed.
- **Measurements:** five prepared contributor-profile samples on the same
  candidate gave median process wall time 3.3715 s fresh / 3.1239 s restored;
  maximum sampled peak RSS 118092 / 80512 KiB. Restored runs reported one restored
  check and zero computed checks; median project validation was 1186 microseconds.
  Observed publication serialization costs were 19035–76287 microseconds;
  serialized payloads were 1776–4501 bytes. Payload bytes are encoded size, not
  retained heap. These descriptive samples do not claim optimized phase-end
  percentile or full project memory qualification.
- **Review:** the first completed exact-candidate
  [Opus review](https://github.com/sifr-lang/sifr/pull/3864#issuecomment-5727613414)
  is **SATISFIED**, with no blocking findings. No remediation or second review
  was required. Seven nonblocking observations are separate work in
  [DX.13 project reuse followups](ad-hoc-dx13-project-review-followups.md).
  Existing DX.12/metadata/profile lint ownership remains unchanged.
- **Evidence:** external root `/home/yaser5/projects/sifr/dx13-evidence`;
  candidate-keyed `be67cc8dfa7d6db91f295557f08123d705cc79ff/qualification-manifest.json`,
  SHA-256 `489d91b9810ad6cc0856fd68dacc6b3a3bdc6faf7afe1ec4a1409c8bc005a601`.
  Validation manifest SHA-256
  `f9a1f1d05762ef790ead5ab4c0cfe41af415a6ee8c26008fa09b12336aa1493c`;
  review SHA-256
  `3b769f729399e919a157be781789b375e0b6d4ff79da50dc5518ff16b1a0a78a`.
  Earlier API/fixture/lease development failures and the interrupted earlier
  candidate run remain preserved, not relabelled as passes.
- **Gate policy:** no create-PR/full merge gate ran for this intermediate item.
  The approved prospective phase-end gate remains authoritative. No release
  was requested or performed. Record-only changes require documentation checks,
  not another broad gate or external review.
- **Blocker:** none for DX.13.
- **Exact next action:** stop after this merged record. DX.14 is eligible only
  in a new bounded session; no DX.14 implementation belongs to this handoff.


## Current Handoff — DX.14 (2026-09-18)

- **Current state:** DX.14 is complete and merged in
  [PR #3866](https://github.com/sifr-lang/sifr/pull/3866).
  Final candidate `2c3f23127966850e7ed961155bcee817514d56f1`;
  merge `10e917056bc26d7a7b21c1361cc9cecfb1c7afb0`.
  This record-only update is on `codex/dx14-record`.
- **Ownership:** implementation, tests and read-only Opus review ran exclusively
  in /home/yaser5/projects/sifr/compiler-dx-orchestration on Linux with Rust
  1.98.1, from base b54f8985c50e4f0da4ab9bcc9886da8dab8dec53. Local Mac work
  was limited to Git bundle transport and GitHub administration. The private
  Cargo target was reused; 8.9–16 GiB free space provided the planned operation
  reserve, so no size-triggered cleanup ran.
- **Implemented:** changed modules are checked through the existing frontend;
  successful interface-stable bodies retain unchanged importer diagnostics.
  The positive proof deliberately covers only undecorated, nongeneric,
  synchronous zero-argument primitive-return functions with one pure return
  expression in flat non-package source graphs. Defaults, constants,
  generic/const bodies, effects, ownership and unknown constructs stay
  conservative. Full SHA-256 structural fingerprints keep body text out of
  resident signatures and graph cache keys. Native output still recomputes.
- **Editor boundary:** eligible saved diagnostics enter the existing captured
  generation only after compiler, metadata, context, sources and resolver
  observations agree. Unsaved overlays never publish CLI authority. Deeper
  typed, flow, index and codegen queries compute through their existing owners.
  --no-incremental disables the adapter and survives toolchain refresh.
  Ordinary package graphs retain exact-context reuse; positive edited-body
  reuse currently uses the legacy source workspace from a manifestless CLI
  invocation. Saved CLI and editor processes must share the pinned cwd context.
- **Named acceptance and focused evidence:** all checks ran afresh on the final
  remediation candidate: seven DX.14 tests, twelve DX.13 regressions, six
  frontend cache tests, nine DX.11 tests and the 2000-statement bounded-key
  regression. Twenty-four deterministic unit edits and sixteen CLI edit pairs
  compare independent fresh runs with expected semantic results; thirteen of
  the sixteen CLI edits retain eligible importers. Forty-four separate CLI
  invocations cover body/default/constant/error paths. Module family decisions
  identify computed/restored work with reasons. The native P06 supplement
  prints 1 then 2 after a helper edit while restoring the unchanged importer.
  Format, file-size (4080 files), HIR maintainability and diff guards pass.
- **Editor/Q09 evidence:** E01–E06 pass with persistence enabled and disabled,
  including actual saved restoration, stale saved records, unsaved overlays,
  late old requests, aliases/encoding, lifecycle and same-process setup recovery.
  Q09 uses one rebuilt optimized installed artifact in four 21-sample cohorts
  (normal/heap, enabled/disabled), excluding each predetermined first warmup
  from measurements. All 42 enabled samples restore one saved diagnostic
  module; all 42 disabled samples restore none. Diagnostics remain equal,
  typed HIR/Rust payload reads remain zero, and close releases metadata owners.
  The frozen trace retains its historical equal SIFR-PACKAGE-0103 LSP status.
- **Measurements:** median trace latency is 1848.98 ms disabled / 1843.96 ms
  enabled (CV approximately 0.7%); median steady RSS is 66.13 / 66.09 MiB;
  maximum enabled sampled peak RSS is 96.63 MiB. Separate allocator-heap
  samples and retained/edited/switched stages are in the comparison report.
  These are descriptive paired observations, not a performance or active-memory
  improvement claim or a substitute for DX.15 qualification.
- **Review:** the initial completed review identified one new regression:
  expanded AST Debug strings inflated resident interfaces and repeated cache
  keys. One bounded correction replaced them with full structural fingerprints
  and added the size-invariance regression. The exact-candidate
  [Opus remediation review](https://github.com/sifr-lang/sifr/pull/3866#issuecomment-5729004460)
  is **SATISFIED**, with no blocking findings. Ten nonblocking observations
  from both reviews are separate work in
  [DX.14 interface/editor followups](ad-hoc-dx14-interface-review-followups.md).
- **Evidence:** external root /home/yaser5/projects/sifr/dx14-evidence;
  candidate-keyed 2c3f23127966850e7ed961155bcee817514d56f1/qualification-manifest.json,
  SHA-256 4a3732ea151d5decd62ade500c1a6bb20ad7a7f88bc170231c6976ad35fe2c4b.
  Validation manifest SHA-256
  e94965a3ec3c804117533b59fa4fa6a45ac53e75a3ae3336b1a4e16ce92ea78a;
  final review SHA-256
  7195c7a65553aa175e588d6643dad94820d6de1c59ecbf3b086b449db211c5c8.
  Earlier failed setup/development records and the initial NOT SATISFIED review
  remain preserved; none is relabeled as passing.
- **Gate policy:** no create-PR/full merge gate ran for this intermediate item.
  The prospective phase-end policy remains authoritative. No release was
  requested or performed. Record-only changes require documentation checks,
  not another external review or broad gate.
- **Blocker:** none for DX.14.
- **Exact next action:** stop after this merged record. DX.15 is eligible only
  in a new bounded session; no DX.15 implementation belongs to this handoff.


### DX.15 final gate remediation scope (unmerged)

The phase-end merge facade ran once. Its failed reports and the interrupted
continuations remain immutable under the external `dx15-evidence` directory.
The continuation on `4f33dcbe77c6541738face6def01b5a8d34ff544` completed the
canonical 727-fixture E2E suite, but required qualification still has failures
and unrun configurations. Neither that result nor earlier product measurements
constitutes a final candidate pass.

The bounded repair batch preserves acceptance coverage and deadlines:

- Separate exact selected SQL, native Python example, diagnostic harness and
  release compiler preparation from timed assertions; retain all preparation
  cost and execute the original runtime checks afterward.
- Stage a version-matched source-development manifest before canonical metadata
  production, then finalize the installed target/archive integrity afterward.
  Use the same staged source inputs and release version for the complete source
  corpus reference; retain full portable-payload equality and strict
  producer/installed artifact identity checks. Synthetic governance fixtures now include the
  canonical metadata envelope and its verifier dependency.
- Use the owned process-tree executor for installed qualification, including
  bounded stdin/output and descendant cleanup on timeout.
- Correct stale test expectations, generated demo output and verification
  inventory/ownership checks. Rename delivery-numbered tests without changing
  their assertions; split oversized package tests by ownership responsibility.
- Enroll the existing identity crate and driver bigint dependency in the strict
  Cargo inventory, and record the driver tempfile/sysroot zstd declarations.
  Extend the registry audit with the official ring index/checksums observed on
  2026-09-19; preserve the original 2026-09-09 audit and existing release rows.
  Exact owner, version, source and checksum checks remain enabled.
- Permit only the regular generated `.sifrbuildinfo` hint in audit directories,
  retaining rejection of unknown files and symlinked hints.

These changes are necessary to finish the declared final integration checks,
including failures that predate DX.15. They do not waive required failures or
change numerical budgets. Compiler source changes (including test-source input
tokens) require newly bound final products and affected performance, metadata,
native package and desktop qualification. Earlier Mac and Linux reports remain
historical evidence with their original candidate identities. The documented
Mac desktop / Linux resource host split remains in effect; the literal combined
12 GB graphical-desktop scenario is not represented as executed. The existing read-only native four-runner qualification workflow is authorized
for unavailable target execution. Its extension consumes exact indexed archive
bytes and same-source older-version fixture packages. It performs actual
installation, version upgrade and forced downgrade, subsequent upgrade,
same-version no-op/reinstall, receipt-failure rollback, relocation,
development/release native reuse, and full source/metadata corpus comparison.
The updater's network transport is an allowlisted local fixture; no publication
or live channel mutation is performed. Both candidate and fixture archives and
installers are bound to the exact source, target, version and digest. Candidate
editor compatibility is checked before expensive native builds. Additional
native evidence is produced after the governed artifact index, so it cannot
change the existing publication evidence schema or acquire mutation authority.
All four native results remain required and are not yet represented as passing.

The final compiler integration repairs also restore reserved CLI namespace
protection for `cache`/`sysroot`, make test code consume the actual lazy codegen
projection and native generated source root, and certify warm Cargo freshness
under the original constrained lock modes. An owned, leased test-fixture path
lets explicit Arrow/tensor native preparation and its runtime assertion share
the same authoritative lock path; every acquisition recopies current fixture
bytes and serializes reset/build/capture. Production native authority and all
original deadlines remain unchanged. Existing runner fail-fast selection stops
at the first actionable failure; earlier exact-input passes remain reusable.

The first native four-target CI run preserved passing Linux primary packages
and exposed a macOS Bash 3.2 production-builder failure when expanding an empty
optional array under nounset. The bounded correction constructs a nonempty
metadata command array in both production and fixture modes, preserving the
production prohibition on fixture-script metadata. Both failed Mac logs remain
external evidence; all affected native package checks must pass on the repaired
candidate. Earlier strict workspace lint repairs and their failed/passing logs
are retained; this batch does not defer required lint cleanup to DX.16.

The final ignored native suite exposed a runner safety-boundary mismatch:
25 serial cases already exceeded the single child's 2,400-second deadline.
The canonical isolated Cargo selection now discovers its complete ignored
inventory and executes each exact case serially through the existing bounded
executor. Empty, duplicate, truncated or inconsistent inventories fail before
execution; each successful child must attest exactly one executed test. The
same feature/target/lock/offline flags, per-child safety deadline and complete
assertions remain required. Default fail-fast stops on the first failure;
explicit collection still executes remaining cases and retains failure. This
is scheduling of functional assertions, not an increased performance budget.
The original aggregate timeout and all individual preparation costs remain
preserved and are not reclassified as a passing aggregate invocation.

The first expanded native package execution completed the real installer
transitions and then rejected an incomplete qualification tool override:
the Cargo event wrapper lacked its mandatory paired rustc. Qualification now
selects both tools from the same admitted PATH and supplies the explicit pair;
missing tools reject before native execution. Compiler toolchain authority is
unchanged. The original CI failures are retained, and affected native profile
and corpus checks require repaired-candidate evidence.

The Python launcher self-test now uses a copy of the actual Cargo-selected
compiler for its positive override and separately requires rejection of the
empty mismatched fixture. The shared production resolver still validates
every override against the prepared candidate. This corrects the remaining
self-test fixture after strict compiler identity admission; it does not add
a test bypass or fallback. The original failure and focused repair evidence
remain external, and untouched package artifacts retain their original SHA.

Python explicit example preparation now builds the required src/main.sifr
entrypoint from the real package cwd and writes generated Cargo output to a
separate owned sibling directory. The CLI package/configuration, interpreter,
trust and native-family resolution remain authoritative; cached runtime
execution and its assertions still run separately. Any application rebuild
caused by the distinct generated root remains visible in execution cost.
Focused preparation checks retain certification immutability and fail-fast
ordering. The original missing-source CLI rejection remains preserved.

Native Mac package qualification now validates the exact otool binary-name
header separately from its dependency lines. A relocated installation beneath
the CI checkout is not itself a source-tree library dependency. Every library
line remains checked for missing or source-tree dependencies; unexpected
headers and empty evidence reject. The original ARM Mac failure, eight native
profile passes, and system-only dependency output remain preserved. Actual
package/corpus qualification continues on the repaired final candidate.

Installed metadata qualification now copies each exact corpus fixture into an
owned manifestless temporary workspace and uses that workspace for native and
editor inputs. Corpus files beneath the compiler checkout intentionally select
project-mode CLI emission, including its source-listing header; they cannot be
compared against a single-file producer contract. Source digests and complete
emitted-byte comparisons remain required, and a temporary directory beneath a
Sifr workspace is rejected. Both Linux original failures retain their successful
727-case provider comparison and eight native profile observations.

The native package workflow also places its complete workload under RUNNER_TEMP,
with collection bound to that same isolated root. Its constructor rejects an
output beneath the compiler checkout or another Sifr workspace before mutation.
Thus plain native profile checks exercise installed standalone behavior without
ambient source-workspace configuration, while package inputs and all compiled
artifacts remain exactly bound. Workflow permission and publication boundaries
are unchanged.

The final SQL component checks also detected stale checked-in guest artifacts:
DX identity integration changed 22 recorded source inputs for each SQL family.
MySQL, PostgreSQL and SQLite components are regenerated by their existing pinned
Rust/WASI SDK 34 and wasi-virt producers; the unrelated words component has no
changed producer inputs. Actual generated bytes, complete provenance and
negative mutation checks remain required. No source hash is manually waived or
substituted. The original provenance failures and all producer timing/digest
reports are retained. New component bytes require affected SQL execution and
new final package identities before final qualification can complete.
