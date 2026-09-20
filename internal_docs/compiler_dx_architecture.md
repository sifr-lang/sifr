# Compiler DX and Toolchain Reuse Architecture

status: final target design  
implementation status: required implementation and trace remediation merged; DX.16 documentation closure ready for merge
implementation baseline: `sifr-lang/sifr@0f819c2f04bf5b2891074c55ba26369ddf4f13bd`  
design date: 2026-09-16  

## 1. Purpose and authority

Make Sifr's normal development loop fast, predictable, resource-bounded, and correct: install a toolchain, check a program, edit it, obtain diagnostics, build or run it, and repeat without reconstructing unchanged compiler or native-build work.

The target architecture requires prebuilt standard-library metadata, native artifact reuse, persistent project results, editor lifecycle improvements, and verification improvements. Implementation proceeds sequentially through the companion phase. Measurements establish whether each mechanism achieves its objectives and identify the optimizations needed to meet the completion criteria.

This document owns architectural behavior and invariants. The companion implementation plan owns scope, sequencing, status, and acceptance evidence. Neither document owns a running experiment diary. Existing subsystem documents remain authoritative for language semantics, package trust, SQL and Python contracts, runtime safety, and release authorization. A planned integration must update a conflicting contract explicitly rather than bypass it.

The command and ownership contracts below remain required. The [canonical phase evidence](../plans/issues/active/ad-hoc-compiler-dx-and-toolchain-reuse.md#execution-status) records merged capabilities and qualification; whole-phase review identified the missing `--trace-dir` surface, now implemented and qualified by [remediation #3871](https://github.com/sifr-lang/sifr/pull/3871). The baseline comparison is historical. Naming changes must preserve capabilities and ownership boundaries.

### 1.1 Historical implementation baseline

The baseline has the necessary frontend, analysis, source, sysroot, codegen, package, and verification owners. It also contains the following relevant integration points:

| Baseline surface | Observed behavior | Required change |
| --- | --- | --- |
| `sifr_driver::stdlib` | Source bootstrap constructs definitions, emitted support, templates, and HIR; a process-local `OnceLock<Result<...>>` retains the outcome. | Replace normal installed bootstrap with a versioned metadata store; retain source production as an explicit toolchain-development operation. [S1] [S2] |
| `ExternalDefs` / `StdlibEmissionCode` | Owned aggregates include more than signatures, including defaults, const bodies, codegen templates, and emitted Rust. | Introduce demand-driven read access without dropping existing semantic information. [S3] [S4] |
| Frontend cache identities | Compiler identity uses crate version/policy labels; the basic package identity projection contains root/entrypoint paths. | Establish complete compatibility and semantic-input identities before persisted result reuse. [S5] |
| `.sifrbuildinfo` model | Verifies compiler/package/source observations, but does not itself restore checked module payloads. | Persist and restore explicitly defined result families. [S6] |
| Native materialization | Uses Cargo JSON output, has some environment controls, and selects `--release`. | Consolidate invocation context, native freshness, compatible storage, and development/test/release profiles. [S7] |
| Generated artifact storage | Final and staging locations derive from temporary storage; publication uses rename. | Move persistent state and its staging to the same durable cache filesystem. [S8] |
| Contributor rules | Include automatic cleanup of an idle private target above 20 GiB. | Replace size-triggered invalidation with pressure-based, owner-scoped cleanup. [S9] |

Historical session measurements motivate this design; they are not performance results for its implementation. In particular, eliminating stdlib bootstrap does not prove a specific future check latency or memory reduction.

## 2. Goals, non-goals, and invariants

### 2.1 Goals

Provide fast fresh-process checks and incremental edit loops without a mandatory daemon. Reuse expensive stdlib and native-build work without weakening diagnostics, ownership checking, interop validation, or runtime safety. Support the qualified 12 GB Linux reference machine without requiring an upgrade. Make compiler-contributor verification selective during iteration, comprehensive at the appropriate gate, and understandable when it fails.

### 2.2 Non-goals

This design does not change Sifr syntax or language semantics, implement a replacement checker/query engine, add a public compiler-service protocol, create a remote/distributed cache, ship a universal precompiled native runtime for every configuration, redesign SQL/Python trust, add unsupported targets, or change release authorization. It does not promise compatibility between private cache formats from different compiler builds. It does not make every possible stdlib input or generic instantiation exhaustively testable.

### 2.3 Required invariants

| ID | Invariant |
| --- | --- |
| DX-I01 | CLI, LSP, formatting/lint integration, and verification use the existing canonical semantic owners. No second checker or stdlib signature authority is introduced. |
| DX-I02 | Normal installed commands never parse, lower, type-check, emit, or bootstrap-validate the whole stdlib from source. |
| DX-I03 | Checking may load selected semantic bodies, defaults, and templates. "No stdlib source bootstrap" does not mean "signatures are always sufficient." |
| DX-I04 | Reuse is valid only for the complete relevant inputs. Source, configuration, resolved dependencies, toolchain identity, and current authorization remain authoritative. |
| DX-I05 | A validated stdlib fragment is not a proof that rewritten, specialized, or assembled application Rust is valid. |
| DX-I06 | Required installed metadata and optional project caches have different error contracts. A broken installation is an error; an invalid optional cache is rejected and recomputed. |
| DX-I07 | Readers never observe partially published state. Finalized generations are immutable, and cleanup cannot delete an actively referenced generation. |
| DX-I08 | Build artifacts, semantic results, live environment observations, and validation receipts are distinct. One cannot stand in for another. |
| DX-I09 | A request analyzes one captured source/configuration/toolchain snapshot. An outdated result cannot overwrite newer editor state. |
| DX-I10 | Native builds use an explicitly resolved context and Cargo's freshness engine. Feature configurations are combined only when their validation contracts permit it. |
| DX-I11 | Selected qualification assertions execute even when their input binaries or metadata are cached. Old test success is not execution of a new test selection. |
| DX-I12 | Functional results, safety deadlines, performance verdicts, and final qualification are separately reported. |
| DX-I13 | Repeated build requests do not rewrite unchanged source/manifest files, rename ordinary projects on every edit, or invalidate caches merely because of an unrelated documentation commit. |
| DX-I14 | Performance improvements are measured end to end, including relevant input validation, cache loading/writing, preparation, and subprocess work. |

## 3. State layers and ownership

### 3.1 State layers

| Layer | Contents | Authority and lifecycle |
| --- | --- | --- |
| Authoritative inputs | User/stdlib source, manifests, lockfiles, configuration, selected toolchain and declared external semantic inputs | Authoring and resolution authority; captured into each compilation snapshot |
| Installed stdlib metadata | Prebuilt semantic records, reusable emitted Rust, required typed templates/bodies and indexed dependencies | Required immutable artifact of a selected toolchain generation |
| Live workspace session | In-memory queries, open overlays, current project graph, editor indexes | Existing frontend/analysis session; bounded lifetime and retention |
| Persistent project results | Selected module-level semantic results and dependency observations | Optional derived local state; versioned, rejectable, and safe to remove |
| Native artifacts | Cargo intermediate state plus Sifr-owned finalized executable/support bundles | Cargo owns native freshness; Sifr owns invocation and publication |
| Validation evidence | Test selection, commands, inputs, outcomes, logs, measurements and review references | Evidence for a specified candidate and validation input set; not a compiler cache |

### 3.2 Subsystem ownership

| Owner | Responsibility |
| --- | --- |
| `sifr_source` / `sifr_syntax` | Captured text, source identities/ranges, parser substrate, path/URI and position conversion |
| `sifr_ir` / `sifr_type_system` | Existing typed semantic structures and explicit persistence-facing record/identity definitions where appropriate |
| `sifr_lowering` | Semantic checking and external-definition lookup contract; no sysroot discovery or Cargo execution |
| `sifr_frontend` | Module graph, semantic inputs, dependency observations, query validity, interface summaries, result-family export/restore |
| `sifr_analysis` | Editor queries derived from frontend results, active project lifetime, source/metadata residency |
| `sifr_lsp` | Protocol transport, request scheduling/cancellation, synchronization, result-version checks and presentation conversion |
| `sifr_sysroot` | Toolchain resolution, generation identity, metadata container/index/compatibility access and installed layout; no second type checker |
| `sifr_driver::stdlib` | Canonical metadata producer orchestration and typed reader/provider adapters |
| `sifr_codegen` | Rust emission, reusable support contracts, specialization, structural policy, generated-code invariants |
| `sifr_package` | Resolved dependency graph, lock/source policy, package/compiler-component identity, current trust and environment contracts |
| `sifr_driver` | CLI/build/test orchestration, project-persistence I/O adapter, native build context, local artifact storage and process lifecycle |
| `crates/sifr` | Outer embedded compiler identity injection plus CLI surfaces/presentation, producer, doctor, timings and profile selection |
| `verification/runner/sifr_verify` | Declarative check/build selection, preparation DAG, execution reports, test-policy and resource coordination |
| Distribution scripts | Build matching producer/consumer artifacts, package exact qualified bytes, verify archive/install/update behavior |

Keep dependency direction intact. The frontend defines a result-store port and accepts an adapter; it must not depend back on the driver to persist state. Similarly, lowering accepts external semantic providers without importing the driver or reading the sysroot itself. Codegen accepts its own support-provider interface. A shared low-level format utility may be extracted only where required to avoid a cycle; this is not permission to introduce a second compiler platform.

## 4. Identity model

### 4.1 Explicit identities

Use domain-separated SHA-256 over canonical, length-delimited records for persisted identities. Ordered semantic sequences retain their order; unordered maps/sets are sorted before encoding. Never use display strings joined with an ambiguous separator as the format contract.

| Identity | Relevant inputs |
| --- | --- |
| `CompilerBuildId` | Compiler-relevant source bytes, source inventory, parser/submodule inputs, manifests/lockfile, build scripts/generation inputs, private wire/schema versions, effective compiler build features/configuration and Rust toolchain identity |
| `TargetSemanticId` | Explicit semantic target, target layout/ABI assumptions and target-dependent stdlib/configuration facts |
| `StdlibInputsId` | Public/private stdlib inventory and source bytes, canonical intrinsic/interop mappings, target semantics and producer options; excludes generated metadata outputs |
| `MetadataId` | Digest of the produced metadata container; stored in the external sysroot manifest, not recursively inside its own hash input |
| `SysrootGenerationId` | Canonical installed manifest inputs, compiler/package integrity identities, metadata identity, selected runtime/source/vendor identities; excludes its own digest field |
| `WorkspaceId` | Logical project identity plus normalized root/source-policy identity; distinguishes worktrees and filesystem/path rules |
| `SemanticContextId` | Compiler/metadata identity, effective language options, resolved package/configuration/lock state, compiler-component identities and declared external semantic context |
| `ModuleInputId` | Captured source bytes and identity plus the resolver/semantic observations required by the result family |
| `ModuleInterfaceId` | Complete exported semantic contract, including required compile-time bodies, defaults, constants, effects and ownership facts |
| `NativeBuildId` | Generated root/source identity, toolchain, host/target, profile, flags, resolved dependencies/features, source configuration and relevant native environment |
| `ValidationInputsId` | Exact tested artifact identities, fixture/assertion selection, harness/policy/configuration versions, relevant source and lock inputs |

Whole-workspace semantic context is recorded for provenance, but a module cache key must not indiscriminately include the contents of every unrelated project module. Per-module dependency observations identify which part of the context affects that module. Start conservatively and narrow only through tested dependency rules.

Presentation-only choices such as terminal color and width do not invalidate checking. Diagnostic policy and lint configuration do affect the relevant result families. Position encoding is applied at presentation, not baked into typed semantic records.

### 4.2 Compiler identity generation

Generate the application-wide `CompilerBuildId` in the existing outer executable package, `crates/sifr/build.rs`, beside `SIFR_BUILD_VERSION` and `SIFR_BUILD_TARGET`. Do not put an identity that changes for every compiler edit in a low-level library depended on by the rest of the workspace. The stateless hashing/inventory helper may be shared; a volatile workspace-wide identity constant may not. [S13]

The executable embeds the value at compilation and supplies an immutable `CompilerIdentity` through driver, sysroot, frontend and analysis context constructors. `CompilerFingerprint` derives from that supplied identity plus relevant policy inputs, rather than reading an independently embedded global constant. Library code does not import the executable, read a mutable identity environment variable on each query, or accept a package-version-only default. Two contexts may carry distinct identities without changing process-global state.

Use one maintained inventory of compiler-relevant roots and inputs, with build-system change tracking for additions, deletions, contents, submodules, feature/configuration changes, and non-Git source archives. Dirty source changes matter even when package version and HEAD are unchanged. Exclude docs-only paths unless they are actual compilation inputs. Rebuild the code that genuinely changed and its normal dependents, plus the outer identity-bearing target; an LSP-only change must not rebuild frontend/sysroot libraries solely to propagate the aggregate identity. Prove this with Cargo artifact/freshness evidence, not a claim that only one crate can ever rebuild.

A small build-time identity helper is permitted. It must not run stdlib compilation or recursively invoke Cargo/compiler execution from `build.rs`. Installed invocations read the embedded ID; they do not hash the current executable or scan the repository. A path/size/mtime tuple may optimize an inventory operation, but it cannot be the sole proof that changed bytes are unchanged. Change detection follows the qualified build-system contract; reproducibility and dirty-input tests include additions/deletions and actual rebuilds.

The binary digest remains separate packaging/evidence provenance. Different native producer binaries can generate identical portable semantic payloads under equivalent declared inputs; their full envelopes may legitimately differ. Cross-host payload reproducibility does not imply arbitrary cross-compiler metadata compatibility.

### 4.2.1 Library-test and embedded-caller identities

An outer executable ID cannot be assumed inside `cargo test -p sifr_driver`, library doctests, or a different executable embedding the compiler. Every real-stdlib test context therefore receives an explicit test compiler identity; a constant such as `source-tree-test`, the source checkout HEAD alone, or a freshly scanned source tree unrelated to the running code is forbidden.

For library harnesses, compose `TestCompilerIdentity` from compiled, dependency-scoped input tokens and the effective test configuration. The common stateless build helper emits each participating package's **local** token over that package's actual source/generation inputs; wrapper-owned vendored parser inputs and locked external dependencies are included at their owning boundary. A package-local token never watches a downstream package such as the LSP. The test composition includes the linked semantic dependency closure, relevant features/toolchain/target, wire policies, and whether test-specific compilation changes an implementation. A shared low-level crate must not embed the aggregate. Adding the local token must not create rebuild edges beyond the actual compiler dependency graph. Test/benchmark-only and product identities are explicitly domain-separated.

This is the documented source-tree test identity rule, not a second semantic algorithm. Different harnesses can share metadata only when their complete producer/reader implementation and configuration identities are equal; `cfg(test)` or feature differences may legitimately require separate artifacts. Record the concrete test target and assertion inputs separately from the semantic metadata family. Qualification of installed artifacts always uses the real executable's product identity; a caller cannot relabel test metadata as an official installation.

Library API tests needing no real stdlib continue to inject small fixture providers. Tests requiring the real producer obtain it from their existing outer driver/integration owner, without reverse production dependencies. Build-time token helpers do not compile stdlib. Byte-change, feature-change and dependency-change tests must demonstrate identity correctness for bare Cargo tests as well as the CLI. [D5]

### 4.2.2 Implemented constructor inventory (DX.2)

The stateless sifr_identity crate owns canonical SHA-256 records and distinct
compiler, semantic-target, stdlib-input, metadata, native-build and validation
types. Its build-only module walks sorted source inventories without Git.
Package-local build scripts emit source/configuration tokens and Cargo links
metadata composes the actual dependency closure. The outer sifr/build.rs
embeds the aggregate CLI identity; each SQL migration tool binary also
embeds an outer identity because it directly invokes the compiler.

| Constructor owner | Identity and lifetime |
| --- | --- |
| sifr/src/main.rs::compiler_context; CLI build/check/run/test/emit/trace | Embedded product identity; unit builds use the compiled CLI dependency closure and cfg(test) token |
| SQL MySQL/PostgreSQL/SQLite tool main.rs::compiler_context | Embedded outer product identity passed into the library migration API; unit context uses linked driver tokens |
| sifr_driver::CompilerContext::new | Caller-supplied immutable identity and pinned sysroot; process-local stdlib ownership keyed by both |
| sifr_driver::CompilerContext::for_test[_tokens]; driver unit/integration callers | Compiled driver producer closure, optionally extended by the owning harness; no installed CLI or recursive producer build |
| sifr_analysis::AnalysisHost::open_* and internal constructors | Required compiler context propagated into the frontend session before queries; analysis tests extend driver tokens with the analysis closure |
| sifr_lsp::run_stdio_with_identity; workspace/session | CLI product identity pinned for the session and document/project replacement; embedded library convenience entrypoint uses the compiled LSP test-family identity |
| sifr_frontend::FrontendContext loaders and WorkspaceSession constructors | Pure-library fixture family from compiled frontend tokens; real compiler/analysis owners inject their context through consuming with_compiler_identity before use |
| CompilerFingerprint::current | Pure-frontend fixture fingerprint; production reuse derives for_identity from its owning context |

All direct driver real-stdlib APIs require a CompilerContext. Fixture-only
frontend, lint and format consumers do not acquire driver dependencies.
Dependency tokens include per-crate cfg(test) distinctions and build feature,
target, profile, flags and rustc inputs. Source inputs are read only by build
scripts; installed identity inspection never scans the checkout or hashes the
running Sifr executable. Test/product identity overrides must match both kind
and compiled digest. Actual binary integrity remains a separate package digest.

### 4.3 Stable semantic identity

Persist artifact-local declaration/type/source IDs with explicit tables. Decode and intern them through normal compiler identity rules. Include package/version/source identity where nominal semantics require it; a bare class or module name is insufficient. No persisted pointer, arena index, allocation-order ID, or process-specific source-map number is an identity contract.

Never treat a hash as authentication. Installed integrity is established by the existing distribution trust mechanism. Project caches are local derived state, not trusted package declarations from another user.

## 5. Prebuilt stdlib artifact

### 5.1 Artifact role and layout

Produce one indexed `stdlib.sifrmeta` for each required compiler/target-semantic configuration. Install it at `<sysroot>/lib/sifr/metadata/stdlib.sifrmeta`, with the sysroot manifest naming the relative path. Retain packaged `.sifr` sources for navigation, documentation, and toolchain development.

The container is a private compiler format with a magic/header, explicit schema version, compatibility IDs, and a small bounded directory of sections. Directory entries contain section kind, module/record identity, offset, encoded size, decoded-size limit, and payload digest. Use fixed-width integer encodings for container offsets and a deterministic, bounded record codec. The existing `postcard` transport may encode explicit records; do not derive serialization over the entire live HIR graph merely because the serializer permits it.

DX.15 physical format version 4 keeps the compatibility header and uses three
bounded Zstandard frames: the logical header/directory, module/name catalog
records, and remaining record payload bytes. The clear outer compatibility header must equal its decoded counterpart.
All frame content lengths and the complete physical/expanded lengths are
checked before allocation. The reader rejects old versions, missing/extra
frames, trailing bytes, mismatched expansion lengths and invalid directory
bounds. It verifies the entire installed artifact digest before opening it.
Only the directory frame expands when opening the store. Each payload frame
expands once on demand. Module and name validation still occurs when the
provider opens; grouping these records avoids expanding unrelated payloads.
Record offsets follow catalog-first then remaining-record order, with canonical
record-ID order inside each group and exact nonoverlapping bounds. Record
digests, canonical JSON records and all
semantic/graph checks remain authoritative on demand. Startup physical
decompression and index construction are measured separately; cumulative
demanded payload decompression is exposed as physical_payload_decode_us.
Physical expansion never itself materializes semantic, type, HIR or Rust records.
The validated sorted directory uses contiguous entries with binary search.
This private format has no compatibility fallback.


| Section | Required content |
| --- | --- |
| Module/declaration directory | Complete public/private module inventory, canonical exports/re-exports and stable IDs |
| Semantic exports | Types/signatures, receiver/parameter conventions, error contracts, generic bounds, constants and declaration metadata |
| Semantic bodies | Const-evaluable bodies, argument/field defaults, descriptors and other typed payloads consumed during checking |
| Dependency/interop summaries | Direct semantic/body/codegen dependencies, retained intrinsic identities, native bridge contracts, feature/support demand summaries |
| Reusable Rust payloads | Per-module emitted Rust bytes, names/mappings, source attribution, generator inventories, validation identity |
| Codegen templates | Generic classes/functions and late project-policy templates that cannot be frozen to one application-independent output |
| Source/navigation data | Relative packaged source paths, content identities, declaration ranges and documentation references |

Every field currently consumed from stdlib definitions, templates, HIR inventory and emission metadata must be classified as persisted semantic data, persisted codegen data, derivable data with a documented derivation, or deliberately removed dead data. The consumer inventory is part of producer acceptance, not an informal checklist.

### 5.1.1 Indexed type, declaration and string tables

The record codec must not recursively serialize a live `Type::Class` at each signature occurrence. The baseline representation contains fields, methods and type arguments by value; naive repeated encoding would duplicate those structures. [S14]

Persist `TypeRecordId`, `DeclarationRecordId`, binder-scoped type-parameter IDs, string IDs and reference arrays in indexed tables. Function parameters/returns, fields, default expressions and body nodes refer to these IDs. A nominal instance refers to its declaring package/module/symbol identity and type-argument IDs; the declaration's field/method inventory is stored once. References include table/store identity when crossing a module chunk; an offset or integer index is never itself nominal identity.

Intern only canonically equivalent records using the existing type system's equality/normalization rules. Same spelling in another package, another generic binder, or another sysroot does not make a type equal. If the current compiler distinguishes a partial structural view from a full declaration, encode the view explicitly rather than merging it solely by nominal name. This is a storage representation change, not a change to Sifr subtyping or equality.

Assign deterministic declaration/binder anchors before encoding references. Preserve language-permitted recursive types with explicit graph references; reject invalid graphs without rejecting every cycle or infinitely expanding a permitted recursion. Decode directory/identity handles before demand-decoding their records, validate references and bounded work, and retain each decoded type/declaration once per store. Do not eagerly expand a compact type table back into full class copies for every loaded signature. Providers return shared handles; any compatibility projection into existing owned types is bounded to the demanded compilation context and measured.

Deduplication must not destroy laziness. A small type/record directory may be eager; unrelated class bodies and method inventories remain demand-loaded. Acceptance includes many signatures referencing one large class, mutually recursive type references, same-named declarations from distinct packages, different generic binders and repeated lookup. Encoded bytes and decoded retained memory must scale with distinct records plus references, rather than the number of references multiplied by declaration size. The applicable precedent is interning and indexed lazy data, not copying rustc's entire type engine. [R1] [R8]

### 5.2 Emitted-code validation contract

The producer emits and validates reusable stdlib Rust once. Its validation record binds the exact bytes, their parser/grammar version, container compatibility, and fragment-kind/boundary rules. Only a successfully validated complete production can publish the record.

An installed build reuses those bytes without rerunning whole-stdlib `StdlibSyntaxSession` bootstrap. It can still parse the selected fragment to perform a necessary transformation and must validate affected boundaries/final assembly. Rewriting, specialization, pruning, changed fragment placement, or changed grammar invalidates the relevant prior assurance. The final program still passes rustc checking/linking. A `validated = true` field alone never authorizes a bypass.

Preserve the emitted layout so source-produced and metadata-loaded routes can be compared byte for byte under identical inputs. Test public `sifr emit`, not only internal Rust strings: stdout bytes, final newline, stdlib preamble start/end markers and the source-map extraction that consumes those markers must be unchanged. Timings stay off emitted-source stdout. No normalization may remove a meaningful marker just to make the comparison pass. This design does not also redesign generated module layout. [S10] [S16]

### 5.3 Demand-driven consumer API

First separate immutable external definitions from mutable project exports; then introduce the metadata-backed provider. The baseline project collector accepts `mut external_defs` and folds each user module's exports into it. Replacing its owned aggregate with an `Arc` alone is incorrect. [S15]

Define a read-only `ExternalDefsView` (or equivalent final name) over an immutable baseline provider and a per-compilation `ExternalDefs` overlay. Lowering and queries accept this view, not the flat mutable maps. Export collection writes through a separate overlay API. Successful module exports are published as one consistent result; failures, deletions and reconfiguration remove/invalidate stale project exports rather than exposing last-good values as current. Preserve existing lookup precedence and package-qualified identity. Overlay lookup must not let ordinary user definitions override reserved/private stdlib authority.

Place the provider contract beside existing external definitions. It provides lazy queries for exports, a named declaration, required semantic body, source location and dependency summary; codegen has its corresponding support-provider contract. The driver supplies metadata-backed adapters. The canonical producer supplies a source-built reference view for qualification, not an alternate checker. Provider failures remain typed diagnostics at the owning boundary.

Decoded baseline values and indexed type records are shared immutable owners. Per-project discoveries and specialization results never mutate that baseline. Context-dependent provider memoization includes the context identity or remains in the overlay. An initial compatibility projection may materialize the complete required module closure, but may not materialize unrelated modules or deep-clone the entire stdlib. Two simultaneous project compilations over one store must not see one another's added/removed exports.

The DX.5 consumer inventory must enumerate read, enumeration, mutation, cloning and lifetime sites by exact repository path and symbol, including: `sifr_lowering::lower_module_with_externals_name_and_options` and sibling lowerers; frontend compile/query helpers and `collect_module_exports`; driver `project/frontend.rs` collection/single-module helpers, `build/entrypoint.rs`, test-runner preparation and stdlib accessors; analysis/LSP external-definition consumers; and codegen import/interop-demand readers. Derive the complete inventory from the pinned source tree and verify migration coverage for every recorded site. The first DX.7 implementation boundary is the layered view and mutation migration under the source-backed provider; only after its parity/isolation tests pass does the metadata adapter become the normal reader.

A small global name/module index is allowed for import completion and resolution. Discovering dependencies or interop demand must not scan/decode all stdlib bodies. Use producer-derived summaries and demand traversal. Private modules are available only through the existing trusted stdlib context, never made importable merely because they occur in an index.

`check` loads semantic exports and any required semantic bodies. `build`, `run`, and `emit` load additional selected Rust/templates. Pure syntax formatting does not load semantic stdlib state. Navigation reads the requested packaged source rather than keeping every source/AST/HIR resident.

### 5.4 Decoder and store lifecycle

Validate compatibility and directory bounds before decoding payloads. Validate every consumed payload's digest and internal references. Limit record counts, recursion, allocations, and decoded sizes; reject malformed offsets, unknown required section kinds and missing payloads. Corrupt required installed metadata is an actionable toolchain error, not a compiler crash or an invitation to source-bootstrap.

Use indexed reads and normal owned buffers as the baseline implementation. Memory mapping is optional only with immutable-file lifetime protection and demonstrated benefit. Laziness is required regardless of I/O mechanism.

Cache successful stores by the full resolved toolchain/sysroot/metadata/target identity. Two compatible stores may coexist in one process, but their nominal identities, lifetimes, and queries remain separated. Reject incompatible metadata. Do not retain transient initialization errors in an unqualified process-global slot forever; explicit re-resolution after correction retries initialization.

## 6. Metadata producer, development, and distribution

### 6.1 Producer command

The target CLI is:

```text
sifr sysroot build-metadata --source-root <root> --output <file> [--target <triple>]
```

The producer reads the canonical source inventory directly and calls the existing frontend/lowering/codegen pipeline. It does not require a previously built metadata file, consult a fallback signature table, or recursively invoke itself. The driver owns orchestration, semantic crates own checking, and codegen owns emitted support. Production may be optimized internally but remains a complete checked-in inventory traversal for the configured target.

Write output privately beside its destination, decode/validate it, and publish atomically. If input files change while producing, do not publish an artifact that claims a different source digest from the captured inputs; development preparation must detect the change and perform a bounded retry or fail with an actionable changed-input diagnostic.

### 6.2 Source-tree development

Define explicit `SourceTreeDevelopment` and `InstalledToolchain` modes in resolution. Installed mode never falls back to source production.

The normal development entrypoint performs `ensure_development_metadata`:

1. Resolve the built compiler and source root explicitly.
2. Compute/validate the source-input identity during preparation.
3. Acquire the generation's preparation ownership.
4. Reuse a matching complete artifact, or invoke the canonical producer if absent/stale.
5. Pass the exact artifact identity/path to consumers.

This write-through preparation is the primary development path. It is not a second checking implementation. A producer error stops the operation; there is no stale-metadata or signature-only substitute. Read-only source trees work with an already prepared artifact and a separate writable cache; a missing required artifact with no valid output location gets a precise remedy.

The verification entrypoint exposes preparation through its normal module/CLI interface. One gate ensures each required configuration once; selected subprocesses consume the same prepared artifact. Direct source-tree CLI/LSP launches use the same ensure operation. They must not silently discover an unrelated installed toolchain to mask stale development inputs.

Bare `cargo test -p sifr_driver` is also a first-class consumer. Its test adapter constructs the explicit test identity from section 4.2.1 and calls `ensure_development_metadata` before the first real-stdlib fixture. On a miss, it invokes the **already linked canonical producer library function in the test process**; it never assumes that `target/debug/sifr` exists, invokes recursive `cargo build/test`, or runs the producer from a build script. The CLI subcommand wraps this same producer function. Lower-level tests use fixture views or their existing integration-test owner rather than adding a dependency back from lowering to driver.

Ensure uses a process-local success registry keyed by complete identity and an OS-backed exclusive per-key preparation lock across processes. Check for a complete entry before locking and again after locking. One successful producer publishes; waiters validate and use that entry. Different identities never share a lock/key as though they were the same compiler. Lock-wait cancellation reports cancellation, not compilation failure. If the producer dies or fails, no partial entry or permanent cached failure is returned; a later caller can acquire ownership and retry. Shared readers or conservative no-delete-while-in-use ownership protect lazily accessed files.

Test ensure is configuration-selective. A parser-only test does not prepare stdlib. An explicitly supplied artifact must match the test context; stale overrides diagnose rather than trigger hidden substitution. Normal same-family fixtures reuse one store per process and one durable generation across eligible processes. Intentional source/metadata differential tests bypass reuse only for their stated producer assertions. Define and measure empty-cache, prepared-cache and simultaneous N-process runs, including distinct test configurations. Stock Cargo normally runs separate unit/integration targets serially while tests within a target use threads; concurrent Cargo invocations, doctests or other runners still require the multi-process contract. [D6]

### 6.3 Packaging and installation

Extend `scripts/distribution/build_release_artifacts.sh` and existing archive/install validators. Build the compiler first; run a matching producer on a supported execution host; qualify its metadata; package the exact compiler, metadata, source, runtime/vendor, lock and manifest inputs. Metadata's own digest stays outside its bytes, avoiding a self-referential hash.

Distinguish producer execution host from semantic target. For cross builds, run the appropriate host-native producer with explicit target semantics, or produce on the corresponding native qualification host. Never try to execute a foreign-target binary accidentally. Do not assert byte equality between different-target metadata. Require reproducible portable payloads for equivalent inputs/target, and whole-artifact reproducibility when the complete producer envelope matches.

DX.8 keys portable stdlib source inputs from captured source-only tokens, while the
required compiler compatibility envelope retains host/profile/rustc configuration.
Stable module-owned Rust/validation references keep that envelope from changing
semantic identities. Full-record portability comparison excludes only the compiler
header and FragmentValidation.compiler_identity; all semantic, type, declaration,
HIR, template, source and Rust records remain included.

Toolchain generations are immutable. Installation/update publishes a complete new generation and switches the selected generation atomically. Running CLI/LSP processes pin their resolved generation, including sources and codegen payloads. The installer publishes .sifr-generations and atomically switches .sifr-current;
the resolver canonicalizes the selected manifest once and derives every path from
that retained generation. Current cleanup conservatively retains all installed
generations. Generation-to-generation upgrades, forced downgrades and relocation
are supported. A legacy mutable or unmanaged populated root is rejected with an
empty-root installation remedy; no legacy migration or retroactive pinning of an
old running compiler is claimed. Cleanup retains active generations. A process either continues with its pinned generation or explicitly restarts/reloads; it cannot mix old decoded records with new files.

Integrity verification at installation covers the full package. Normal commands check compatibility and consumed metadata payloads without rehashing the entire runtime/vendor/source tree. An explicit doctor integrity operation performs the full check.

### 6.5 Implemented producer and preparation ownership (DX.6)

`metadata_producer::production::Inputs` captures the canonical public/private
stdlib inventory and passes those exact bytes to the existing source frontend and
codegen. Source record paths are relative to `sysroot.paths.stdlib_root`, so a
packaging snapshot can use `lib/sifr/stdlib` without embedding the build location.
The input identity also binds the source manifest, lockfile, compiled mapping
components, producer policy and explicit 64-bit little-endian semantic target.
The four currently supported triples have distinct target identities.

Typed encoders preserve the live semantic, HIR, generic/default, interop and Rust
payload families. `scripts/generate_metadata_encoders.py --check` verifies the
exhaustive generated projections; handwritten declaration, binder, export and
source-location ownership remains in the driver. Try-body error types are a set;
the encoder orders their complete references, avoiding ties in frontend display
names. Class-shaped synthetic constructor views retain the actual enum/newtype/
protocol declaration identity. Complete decoder validation precedes publication.

`ensure_development_metadata` is the one source write-through operation. Its
process store retains only successful immutable owners. The private durable entry
is keyed by compiler/test identity, semantic target and captured input identity;
an OS lock, validation before/after ownership, same-directory stage, fsync and
rename coordinate competing processes. Stale stages are replaced only under that
key's exclusive lock. Errors/cancellation release ownership and do not enter the
success store. Inputs are recaptured before publication, and changed inputs cause
an explicit retry diagnostic. A caller's explicit artifact override must match;
it cannot cause a substitute identity to be selected. Metadata owners currently
pin owned immutable bytes; native-artifact pruning does not prune metadata files.

Bare driver tests use `CompilerContext::for_test()` and the already linked source
producer. Verification runs that setup once per selected driver Cargo graph and
uses Cargo-reported product executables for CLI preparation; neither path assumes
a preexisting CLI for bare tests or invokes Cargo from inside the producer.
Other linked test configurations select their own keyed owner when initialized.
Preparation reports producer duration, compiler identity and metadata identity;
subprocess consumers revalidate the same durable entry.

Packaging invokes the exact native compiler over the staged source snapshot,
then seals metadata and `stdlib.metadata.json` into the installation. The descriptor
binds compiler bytes, compiler identity, semantic target, input identity and
metadata digest. Production packaging must run on each target's native
qualification host; it never runs a foreign binary or relabels a host compiler.
The explicit `--binary` script fixture protocol is limited to distribution tests.
Source-tree mode is explicit in the manifest; an installed snapshot is rejected
by the source producer. Ordinary metadata-backed consumer activation remains DX.7.

## 7. Local storage, publication, and cleanup

### 7.1 Storage boundaries

Use a user-owned persistent root: `${XDG_CACHE_HOME:-$HOME/.cache}/sifr` on Linux and `~/Library/Caches/sifr` on macOS, with explicit `SIFR_CACHE_DIR` override. These are target defaults for the currently supported host families, not a new Windows-support commitment.

The logical layout is:

```text
<cache-root>/
  metadata/<compiler-id>/<target-id>/<stdlib-inputs-id>/
  projects/<workspace-id>/<compiler-id>/generations/<generation-id>/
  native/roots/<root-identity>/
  native/targets/<compatible-family-id>/
  native/artifacts/<native-build-id>/
  locks/
```

Every published entry has ownership and identity metadata. A source-tree build may select a worktree-private equivalent root. Workspace caches do not share user semantic results across worktrees merely because filenames match. Native dependencies may share a compatible Cargo family as defined below.

Keep temporary scratch separate. Every entry that is promoted by rename is staged beneath the same destination filesystem, not under arbitrary `TMPDIR`. A path migration must update staging, required-artifact paths, identity/report fields, and tests together. [S8]

`.sifrbuildinfo` is a small ignored workspace manifest naming a local generation by validated identity. It is an optimization hint, not a permission grant or an arbitrary path to deserialize. Resolve generation IDs only under the selected cache namespace. If the workspace is read-only, the same generation can be discovered under its `WorkspaceId` in the user cache; writing the hint is not necessary for correctness.

### 7.2 Entry publication and project-generation protocol

Implement storage capabilities when their real consumer is introduced. DX.3 owns existing generated-artifact entries: same-filesystem private staging, atomic final publication, completeness/identity revalidation, per-entry or family ownership, and cleanup that skips active or contended entries. These safeguards are required immediately because the baseline already publishes reusable native/test artifacts. A directory existing after a rename race is not proof of a valid winner. [S8]

DX.6 adds per-key dev-metadata ensure and conservative reader protection; DX.8 adds pinning of installed toolchain generations; DX.9 adds ownership for shared native families and final executable capture. An immutable metadata file does not require project-record inheritance or a generalized lease manager. File locks and immutable handles suffice for these entry-level contracts.

Only DX.13 introduces the full **project** generation protocol below, alongside its first persisted semantic results. Do not implement hard-link inheritance, a graph of shared project records or project-generation garbage collection as an unused DX.3 framework. Share already-proven atomic/locking primitives, not a new cache service. [R3]

1. A writer acquires an OS-backed ownership lock and creates a unique working directory under the destination parent.
2. It opens a compatible finalized generation with a reader lease, revalidates its inputs, and retains still-needed complete records by immutable reference or hard-link/copy.
3. Changed records are written as new files. A hard-linked file is never truncated or modified in place.
4. Record payloads and the complete generation manifest are flushed; directory durability is handled by the supported local filesystem adapter.
5. The writer finalizes by a same-filesystem atomic publication. A `latest`/workspace pointer is updated separately and is only a hint.
6. Concurrent completed generations are independently valid only for their recorded inputs. A winner is not trusted merely because its directory exists: validate its manifest, required paths and identity before reuse.
7. The next run ignores incomplete working state. Cleanup removes it only after proving it has no live owner.

Readers keep a lease while they require an entry, including metadata or executables with dependent side files. GC requires exclusive ownership. A lease is a real lock/reference held through the relevant operation, not a PID string or a lock file's mere existence. Define lock ordering and test process death, contention and interrupted acquisition.

The supported persistence contract is for qualified local filesystems with working lock/atomic-publication semantics. Arbitrary shared/network filesystem behavior is not guaranteed. Failure to obtain safe project-cache persistence leaves normal uncached semantic computation available with a clear cache status. Required artifact preparation/materialization failures remain actionable errors. These are explicit supported behaviors, not alternative semantic implementations.

### 7.3 Pressure-based cleanup

Remove the automatic `target > 20 GiB → cargo clean` rule. Directory size is an observation, not an invalidation event.

Evaluate configured free-space reserve, upcoming preparation requirements, existing ownership and reusable active inputs. Reclaim abandoned working directories, obsolete project/metadata generations and obsolete generated roots before expensive dependency artifacts. Protect current toolchains and active candidate outputs. Never clean another session's worktree or a shared Cargo family without exclusive ownership.

Use supported Cargo cleanup operations for Cargo-owned artifacts. Do not delete individual internal Cargo fingerprint files. Cleanup exposes the selected scope, reason, bytes reclaimable and protected entries; a dry-run changes nothing. If safe reclamation cannot meet the reserve, report the resource blocker before launching a large build.

Resource limits live in the existing profile/resource-policy authority. They are calibrated for the supported machine and recorded with provenance, not duplicated across scripts. Cache-capacity policy cannot be changed to hide a correctness or performance regression.

### 7.4 Trust and privacy

Keep cache namespaces user-owned and reject traversal/symlink escapes at write and cleanup boundaries. Do not treat checksums as authorization. A cache hit never grants build-script, native-probe, compiler-component or runtime execution permission. Current trust and required environment checks still execute.

This design does not accept caches supplied by another user or a repository as privileged artifacts. Cache payloads can contain user code, types and paths; they are not committed, published in ordinary reports, or uploaded by default. Timings and error summaries redact secrets. Permission revocation or changed external context invalidates/restricts the corresponding operation even when bytes remain cached.

## 8. Native builds, probes, and application profiles

### 8.1 One resolved native build context

Resolve one immutable `NativeBuildContext` before preparing a generated project. It contains explicit Cargo/rustc executables or an explicit toolchain invocation, full toolchain identity, host/target, effective profile, flags, features and resolver policy, resolved package/source/lock inputs, relevant build-script/native environment, Python binding/loader identity where applicable, trust/sandbox policy and destination locations.

Internal temporary projects must not accidentally inherit an unrelated working directory's toolchain or Cargo configuration. Intentionally supported workspace/user configuration remains supported and appears in the effective context. Existing package/network/offline rules remain in force; no implicit online resolution is added to a check. Unknown build-affecting configuration cannot silently share a trusted-ready artifact.

`doctor` validates the selected toolchain, not an unrelated `rustc` earlier on shell PATH. Build, check-probe, metadata, test and portable-export paths consume the same context rather than independently constructing `Command::new("cargo")` policies.

DX.2 implements this boundary with sifr_sysroot::NativeToolchain and
NativeBuildContext. Resolve in the intentional caller/package root before
materializing temporary projects. SIFR_RUST_TOOLCHAIN selects an installed
rustup toolchain; paired absolute SIFR_CARGO/SIFR_RUSTC overrides select
explicit executables. A one-sided or unavailable selection fails. Cargo and
rustc versions, executable digests, target, supported native environment and
Cargo configuration bytes participate in identity. Only selected build
environment names are hashed; debug/CLI reports expose identities and tool
paths, not secret values.

Preparation, interop probes, generated builds/tests, package metadata and
host-tool Cargo commands consume resolved tool selections. Generated commands
use explicit manifests from the original configuration root; their private
target destination is explicit. Cargo-reported executable paths are published
at Sifr's stable artifact path. Application build remains release and generated
test remains Cargo's test profile. The --print native-context --json command
and doctor inspect the selected tools; build reports retain compiler and native
toolchain identities. Persistent metadata production/storage and changed
application profile defaults remain unchanged by this context contract. Selected Python shared-library directories are retained in generated Unix loader paths and in native artifact identity, so link-time interpreter selection also governs normal execution.

### 8.2 Compatible native storage

Cargo owns unit freshness, dependency artifacts and its internal target layout. Sifr groups storage by compatibility boundaries such as toolchain/host/target, dependency source configuration, native environment and trust domain. Distinct Cargo profiles/features remain represented in Cargo's own unit identities and in `NativeBuildId`; do not build a bespoke substitute for Cargo's fingerprint engine. [D1] [D2] [D3]

Separate compiler-development storage from generated applications/probes where registry/source replacement or feature/environment differences would invalidate or interfere with compiler artifacts. A single target per sysroot generation is not universally sufficient. Conversely, adding an unrelated root-source hash to the entire dependency-storage key would throw away valuable reuse.

Ordinary editable projects have stable root/package identity. Immutable test/probe materializations bind root identity to their complete source/manifest/contract configuration. Use distinct root target names, or equivalent serialization and output isolation, so two same-named binaries cannot overwrite one another's shared output before capture. Package naming alone is not the safety proof.

Serialize mutation of one generated root and the capture of its outputs. Cargo may parallelize its dependency graph. Independent compatible roots may share storage only under the defined family/ownership discipline. Do not allow one process to rewrite a manifest or source file while another process is compiling it. Write generated files only when bytes change.

### 8.3 Freshness and finalized artifacts

On ordinary local `build`, `run`, and `test`, ask Cargo to establish native freshness even when Sifr already has the generated source and a previous finalized artifact. This preserves Cargo's build-script/file/environment dependency checking. A Sifr metadata match and file existence alone do not justify bypassing those checks.

Read `compiler-artifact` executable/output paths and `compiler-message` diagnostics from Cargo JSON. Require process success in addition to protocol messages. Capture the exact output bundle into a private Sifr artifact generation, validate required executable/debug/runtime side files, and publish it under `NativeBuildId`. Do not return a mutable shared `target/...` path as a durable artifact handle. [D3]

A no-change request should perform a no-op Cargo freshness check, not recompile or relink. A build failure never runs the last good executable as an implicit fallback. The previous generation may remain in storage but is not the result of the failed request.

Local development uses resolved local sources. Portable export explicitly rewrites/freezes the appropriate publishable dependency identities and verifies the exported result. A local unpushed compiler candidate does not require a GitHub push merely to be tested.

DX.9 implements private native/families storage outside compiler targets.
A family file lease spans source mutation, Cargo execution and output capture.
Editable generated roots retain scope/name identity; direct probes bind their
materialization to the complete probe key. Generated binary/test target names
include the stable root identity. Byte-identical inputs retain mtimes, and
obsolete generated support files are removed. Ordinary binary, test and direct
probe requests consult Cargo even when a prior finalized result exists.
Executable bytes, Cargo-produced runtime libraries and platform debug side
files define the finalized bundle. Generated loader paths use the executable
directory for bundled runtime libraries; system/Python libraries retain their
declared external environment contract.
Capture copies these into leased immutable artifact entries and validates an
existing entry's bytes before returning it. Cargo failure cannot select the
previous generation. Cargo profiles and application defaults remain unchanged.

### 8.4 Application profiles

The optimized compiler executable and the optimization level of its generated application are separate decisions. Ship an optimized Sifr compiler. Adopt these application defaults:

| Command | Application profile |
| --- | --- |
| `sifr check` | No application code generation/linking; required semantic/interop checks retain their own contracts |
| `sifr build`, `sifr run` | Development |
| `sifr test` | Test, derived from development |
| `sifr build --release`, `sifr run --release`, `sifr test --release` | Explicit release configuration; test-harness requirements remain respected |
| Runtime-performance and release-qualification cases | Explicit declared profile, never an accidental default |

The initial pinned native profile policy is:

| Setting | Development/test | Release |
| --- | --- | --- |
| `opt-level` | `0` | `3` |
| `debug` | `1` (limited) | `line-tables-only` |
| `debug-assertions` | `true` | `false` |
| `overflow-checks` | `true` | `true` |
| `panic` | `unwind` on the qualified unwinding targets | Same |
| `incremental` | `true` | `false` |
| `lto` | `off` | `off` |
| `strip` | `none` | `none` |

These are explicit Sifr design defaults, not a claim that Cargo uses them all by default. Optimization/debug settings may be refined through measured, reviewed profile-policy changes. Supported programs must retain Sifr numeric, error, ownership, resource and panic-boundary semantics in every supported profile. No user data-dependent safety rule may rely on a debug assertion. Keep actual `assert` and required panic-boundary behavior intact.

Effective Cargo overrides are resolved and recorded. Reject incompatible settings that would invalidate a required boundary rather than silently accepting them. Test/bench harnesses have their own Cargo panic-strategy behavior; a release-like test is not a replacement for running the actual release executable in boundary qualification. [D1]

Compiler build profile, generated-application profile and verification selection are separate fields. Freeze the existing execution profile of each native/runtime fixture during DX.1/DX.4 inventory. Before changing the default `run` profile in DX.10, give each applicable assertion explicit `application_profiles`, target, assertion depth and effective profile-policy identity. A fixture that previously certified a release executable continues to run that release assertion; a development assertion is added, not silently substituted. Include explicit CLI-default tests to prove the new user-facing defaults.

The maintained release qualification must execute the complete declared release `run-pass` corpus through actual release application binaries, plus release panic/FFI/resource boundary cases. Its success is not inferred from `cargo test` in a development/test profile. Routine development E2E can use development applications only when selected explicitly and when the retained release selection remains present in the proper gate.

Development and release Cargo artifacts coexist in their respective profile storage. Measure dependency build count, elapsed cold setup, no-op/edited builds, peak memory, total disk bytes and live reserve for dev-only, release-only and alternating dev/release on the 12 GB machine. Do not claim a single dependency build or exactly doubled storage: Cargo can reuse some units and duplicate others. Changing profiles must not trigger automatic cleanup of the other useful profile or weaken qualified runtime semantics. [D1]

DX.10 implements this policy in `sifr_driver::ApplicationProfile` and emits all
three profile tables into each generated Cargo manifest. The test table derives
from dev; Cargo ignores an explicit test panic key, so the harness retains its
required unwinding behavior without an ignored setting. The outer CLI selects
one immutable application profile; build reports name it independently of compiler
identity. Captured Cargo configuration, environment and Rust flags are checked
for incompatible unwind/overflow overrides. Finalized executable paths use
`target/final` because those immutable copies are not Cargo profile storage;
Cargo's development and release directories remain separate within one family.
The maintained selection authority is `verification/runner/application_profiles.json`.
E2E release qualification rejects a development application override and executes
the complete declared release corpus in native binaries. Ordinary E2E retains
its development assertions; area-native commands explicitly retain release.

### 8.5 Feature-graph consolidation

Classify compiler-gate configurations by what they certify. Group ordinary compiler integration checks that support additive unification. Preserve minimal/default-feature checks, missing-feature regressions, native ABI/source-isolation cases and host/target distinctions in explicit separate groups.

A workspace-wide invocation is allowed for a named integration selection whose package/test/feature membership is intentionally equivalent. It is not a blanket replacement for all profile selections or ignored/native test tiers. Compare the executed assertion inventory before and after grouping. Generated applications use their resolved features, not a made-up all-features application. [D2]

### 8.6 Python/native environment

Bind Python declarations and native builds to the selected interpreter, ABI/library and environment contract. Thread it through probing, Cargo build inputs and actual runtime loading. A library with the same basename is not automatically equivalent. Do not hide original import/loader errors.

Development runtime lookup can use the selected local environment. Portable output requires an explicitly qualified deployment arrangement with relocatable loader behavior or a declared external runtime dependency. Do not embed an arbitrary developer path as an undocumented portable contract. Qualify a real process in every supported packaging mode; a successful link does not prove loader correctness.

Generated Python binaries additionally validate the actual library supplying
CPython symbols before interpreter initialization. A different canonical
same-basename library or changed library content produces a concrete loader-mismatch diagnostic. Python
source exports carry sifr-python-runtime.json: an explicit external-runtime
deployment contract naming the required interpreter and library paths, with
relocatable set to false. This mode requires that environment at deployment; it
does not advertise a relocatable bundled interpreter.

Do not cache live runtime certification or database/server observations as if they were immutable compiler facts. Deterministic SQL/component analysis may be cached only with its complete schema/provider/protocol/capability context and current authorization preserved.

## 9. Persistent project compilation state

### 9.1 Required capability and result families

Project-level disk reuse is part of the final scope. It follows stdlib and native-storage work so its benefit is measured independently. It persists selected module-level results, not a live process image and not a new query system. [R2] [R3] [T1]

| Result family | Persisted contract |
| --- | --- |
| Resolution observation | Source-file set, ordered resolution candidates, successful and failed lookups, configuration/package observations |
| Semantic interface | Canonical exported declarations plus the complete cross-module semantic information consumed by importers |
| Checked module | Typed representation and analysis summaries required by declared reuse consumers, with stable-ID remapping and phase invariants |
| Diagnostics | Completed canonical diagnostic facts, related locations and suggestions tied to exact source/dependency identities |
| Codegen handoff | References to required typed results/specializations; only reuse generated code if the codegen-specific identity is valid |

Syntax trees and rich editor indexes are not required to be serialized merely to restore a successful check. Recompute any unpersisted family through the existing engine when demanded. A manifest records which families are complete. "No diagnostics" is a valid completed diagnostic result, not a missing record; `pending`, `cancelled`, `crashed`, and `environment-blocked` are not reusable successful computations.

Completed deterministic source-error diagnostics may be stored when all relevant inputs are captured. A partially checked project cannot be represented as globally successful. On cancellation/crash, do not publish the in-progress generation. Native build/link/run failures never become a ready executable entry.

### 9.2 Cross-process algorithm

1. Resolve and pin toolchain/metadata, workspace options and package context.
2. Capture authoritative saved-source inputs for the command. Hash the same bytes passed to parsing/analysis, not a different read.
3. Read the workspace generation hint and reject incompatible schema/compiler/context before decoding result bodies.
4. Revalidate file-set, resolution and declared external observations. A filename/timestamp match alone is not proof of source equivalence across processes.
5. Reuse valid resolution and module results; rediscover/invalidate observations affected by edits, additions, removals, configuration changes or environment changes.
6. Process changed modules in the existing dependency-correct deterministic order, retaining the language's existing cycle policy.
7. Recompute the required result families; propagate changed semantic interfaces to importers as defined below.
8. Render the current command's diagnostic format from canonical facts. Compute missing build/editor families when requested.
9. Publish a complete immutable generation and update the workspace hint. Preserve valid unchanged records, including records not decoded in this run.

Cache decoding, invalidation and lookup overhead must be visible in timings. Do not persist a huge representation whose read/write cost exceeds the work it saves without an explicit measured reason. Keep the capability enabled by default where its qualified result families apply; uncacheable operations remain ordinary compiler work, not a separate semantic fallback.

### 9.2.1 Implemented checking consumer (DX.13)

The first restored family is the completed entrypoint checking query, including
its whole captured import closure and canonical diagnostics. Its module result
records resolution/diagnostics as complete and typed interface/HIR/codegen as
pending. This is deliberately conservative: any observed source/resolution/config
change recomputes the closure in that initial consumer. DX.14 implements the
interface propagation and editor restoration described below. Ordinary build/emit/editor requests compute absent
families with their existing owners; a no-error check never supplies a fake HIR.
The existing bounded typed codec remains available to deeper consumers.

`project_cache` is driver-owned storage; `persistence::CompletedCheck` is the
frontend-owned completion, source-observation and canonical remapping authority.
The ordinary saved-source CLI check calls the existing checker on a miss. It
pins and validates required stdlib metadata before reuse. Pure package graphs also reuse completed checking after the ordinary package
resolver runs. The complete resolved graph/source-map projections bind package,
lock/feature selection, namespace and source inclusion inputs. A nonempty
compiler-component, SQL, Python or native-backend inventory keeps the operation
uncached, preserving live owner checks instead of declaring missing context complete.
For otherwise eligible packages, the actual generated interop plan must also be
empty before a successful check can be published. This owner-produced attestation
prevents imported stdlib native demand from bypassing required live probes.

Each canonical workspace/context has a serialized writer and immutable generation
manifests naming content-bound records. Readers retain shared generation locks;
GC requires the writer lock and nonblocking exclusive generation locks. The
order is writer then generation; lock files are never unlinked. Inheritance
hard-links sealed payloads, or copies into new files when linking is unavailable.
No finalized payload is truncated. Schema/context/reference/digest validation
precedes publication-winner reuse. Interrupted private stages and finalized
orphans are misses until a complete pointer exists; GC only removes inactive
owners. Latest generations retain historical complete records, revalidated on
every reuse, including unused records and error/fix/revert states.

`.sifrbuildinfo` is an optional bounded hint; the user-cache pointer works in a
read-only workspace. Compiler-owned hint names are excluded from directory
observation identity (the actual resolver still receives unchanged entries).
`--no-incremental` skips all project reads/writes and does not alter stdlib/Cargo
storage. `--timings` reports completed-query reuse/computation counts, captured
source count, payload bytes, validation and serialization/publication wall cost.
The independent CLI acceptance runner records total wall time and peak RSS.
`cache prune-project <workspace> --reserve-bytes N [--dry-run]` reclaims inactive
predecessors/abandoned stages only under pressure, preserving active/current
project records, installed metadata and Cargo state.

### 9.3 Dependency observations

A semantic result records all relevant observations, including absence:

| Observation | Invalidated by |
| --- | --- |
| Source bytes/identity | Content changes, deletion, identity/path-policy change |
| Ordered import search | New higher-priority candidate, failed lookup becoming successful, changed search roots or package resolution |
| Project file set | Include/exclude/root rules and relevant directory membership changes |
| Imported interface/body | Changed exported types, defaults, constants, generic/const bodies, inferred effects or ownership facts actually consumed |
| Package/configuration | Effective manifest/lock/feature/semantic option and compiler-component changes |
| SQL/Python/interop semantic context | Relevant declared schema/provider/interpreter/declaration/ABI context changes, according to existing subsystem rules |

Watchers are invalidation notifications and performance hints, not complete cross-process correctness evidence. A file not present during an earlier check is still a dependency observation. Case sensitivity, symlink aliases and logical import names must follow the existing resolver/source identity policy.

A command is not promised a filesystem-wide atomic snapshot in the presence of arbitrary concurrent writers. It must, however, analyze one internally coherent captured input set and label/persist exactly those inputs. For tracked changes during capture, restart discovery within a bounded policy or report changed inputs. Never publish a mixed hash/content entry and call it a match for the new filesystem state.

### 9.4 Module-level invalidation and unchanged interfaces

Use the current module-granular dependency model. A changed source invalidates that module's affected local results. Recompute its dependency observations and interface. An importer may reuse checking only when its own captured inputs and every semantic dependency it consumed still match.

`ModuleInterfaceId` is not a hash of a pretty-printed function signature. It includes necessary generic/const/default payloads, nominal identity, re-exports, ownership/effects and declaration-derived rules. Ordinary implementation-body changes that provably do not affect importer semantics need not invalidate importer checking, but still invalidate the appropriate native build. Unknown or dynamic dependency scope is conservatively invalidated.

Tests must prove both semantic equivalence and reuse behavior. Merely returning the same diagnostics with caching disabled does not demonstrate this feature. Do not implement expression-level persistence, a second salsa database, or speculative dependency elision as part of this design.

### DX.14 proven checking propagation boundary

Completed saved-source diagnostics can seed the existing frontend queries after
revalidating the same compiler, metadata, source and ordered resolver observations.
The driver can recompute changed modules in a captured flat non-package graph
and retain unchanged importer diagnostics. The current positive interface proof
erases only undecorated, nongeneric, synchronous zero-argument functions with an
explicit primitive result and one pure return expression; checking changed
modules must still succeed. Stored interface identities are full SHA-256
fingerprints of the structural AST hash stream, so body size does not expand
resident signatures or graph cache keys. All other bodies, private declarations, defaults,
constants, class declarations, ownership and unknown effects remain conservative
dependencies. Pure package checks retain exact-context reuse; live external
authorities retain their ordinary owner execution.

The editor only restores matching saved-source diagnostic facts into its captured
generation. It does not publish overlays or claim typed HIR, flow, codegen or
editor-index readiness. Rich requests continue through the existing owners.
The --no-incremental flag also disables the editor adapter through the immutable
compiler context and survives explicit toolchain refresh. Module decisions report
the diagnostic family, computed/restored action and proof or invalidation reason.

### 9.5 Recovery and explicit controls

Missing, stale, corrupt or unsupported-format project records are ignored and recomputed from authoritative inputs. Cache-write failure preserves the actual semantic outcome and reports that persistence was unavailable; it cannot turn valid source into a type error. Required output/native materialization errors remain errors.

Add `--no-incremental` to bypass project semantic disk reads/writes for diagnosis and fresh-reference tests. It still uses installed stdlib metadata and Cargo's normal native behavior; it is not a whole-toolchain clean switch. Source-based stdlib equivalence uses the producer/test interface, not this flag.

Project state is local and private. Moving a workspace may cause a cache miss. Do not claim cross-machine/worktree portability for `.sifrbuildinfo`. Installed stdlib metadata, by contrast, must resolve correctly after moving an intact installation.

## 10. Editor session, source snapshots, and memory

Keep the existing `WorkspaceSession` / `AnalysisHost`. LSP handlers remain adapters; no editor-owned parser, type checker, module resolver, or code generator is added. [T2] [T3]

Each request captures an immutable generation containing source, configuration, dependency context, and a pinned toolchain/metadata store. Open-document overlays are the editor's source authority. Saved disk bytes are the ordinary CLI's authority. Open/change/save/close transitions update ownership explicitly; an unsaved editor result is not written as a saved-source project-cache hit. The LSP may persist a fully captured saved-source generation only when every represented input matches disk and no overlay contaminates that generation.

When a request completes, publication checks its captured generation/document version. Cancellation checkpoints reduce wasted work, but late or ignored cancellation cannot allow a stale diagnostic, completion, source edit or project registration to replace newer state. Every protocol request receives the required terminal response; diagnostics for deleted/closed/reconfigured documents are cleared according to the existing push/pull contract.

Keep logical URIs/paths distinct from canonical filesystem identity where the resolver requires it. Perform UTF-8/UTF-16/UTF-32 range conversion at the transport boundary. Test Unicode, CRLF, symlinked roots, relative roots, supported case rules and source relocation. Persisted diagnostic offsets are validated against their captured source identity.

Separate latency-sensitive syntax/editor work from expensive native/environment operations using the existing scheduler/cancellation boundary. Formatting and syntax-only requests do not force native compilation. Semantic requests that legitimately require an external contract retain that dependency and an explicit readiness state; they do not silently fabricate an answer to appear fast. Missing metadata keeps the server alive enough to publish the setup error and recover on explicit re-resolution, without providing counterfeit semantic results.

The stdio owner serializes analysis mutations. Its ingress generation lock also
orders source/configuration events against response and diagnostic publication;
late work returns `ContentModified` independently of cancellation. Analysis
snapshots retain captured source maps/overlays and their exact compiler metadata
owner. Incremental updates commit the same overlay bytes used by later reloads.
Configuration notifications explicitly re-resolve the toolchain and rebuild open
overlays; ordinary requests remain pinned. Closing projects releases their Python environment/inspection caches; closing
the last document releases the idle editor's decoded metadata ownership.
Logical client URIs remain distinct from physical source identity, including
new unsaved files under symlinked parents. Simultaneous aliases for one source
are rejected explicitly; close its existing URI before opening another alias. Formatting, folding and selection use
the existing analysis syntax APIs without constructing a semantic host; with
push diagnostics disabled, opening text itself does not load metadata.

### 10.1 Retention policy

Share immutable source/semantic/metadata values across requests. Reference-count active snapshots and release old generations when no request uses them. Keep bounded optional decoded-module/project caches; only unreferenced entries are evictable. Do not retain every stdlib HIR or every closed workspace indefinitely. [T3] [T4]

Measure both retained and peak memory: source capture, decoding, checking, completion, project switch, old/new snapshots coexisting, and cleanup. Size guards for `HirStmt`, `HirExpr`, `Type` and high-frequency structures prevent accidental layout expansion, but do not replace allocation and lifetime measurements. Box rare large variants and intern repeated identifiers only where total memory and latency measurements support the change; a small enum with thousands of extra allocations is not automatically better.

Maintain the existing 128 MiB large-session fixture cap where that contract applies and the named host-specific baselines. That cap is not a statement about total available system memory. The original qualification scenario combined the 12 GB machine, editor, normal background desktop load and bounded Cargo activity. On 2026-09-18 the user selected actual Mac desktop testing, so final qualification uses an explicit host split: the named 12 GB Linux reference must satisfy its fixed product latency and bounded editor/Cargo resource contracts, and the named Mac must separately run the editor/Cargo overlap under its real desktop load. Record each host’s actual hardware, effective memory/CPU/container limits, disk location, power policy and swapping pressure. The observed Mac has 32 GiB RAM; its result does not certify the original single-machine 12 GB graphical-desktop scenario, which remains recorded as not executed. This split does not raise the Linux supported-machine floor, waive its targets or apply Linux thresholds to Mac observations.

DX.1 also freezes a demanded-stdlib retained-memory workload, measured with comparable optimized installed LSP binaries before and after the reader migration. Record idle/session baseline, post-open settled RSS, peak load/edit RSS, decoded module/type/body counts, attributable retained allocations and memory after close/switch. Record absolute and relative changes plus the sampling noise envelope. Historical numbers near 130 MiB are motivation, not an interchangeable baseline. DX.7/DX.8 and DX.11 must show the demanded-state mechanism removes unrelated retained stdlib structures, report the paired RSS comparison and preserve active scoped caps. An unchanged or increased RSS requires attribution and nonregression assessment rather than a claim of an unmeasured memory win. Establish any additional numeric memory target prospectively in the named benchmark policy, never by selecting the best sample after the change.

## 11. Command surface, diagnostics, and observability

### 11.1 Implemented commands

| Surface | Contract |
| --- | --- |
| `sifr check <input>` | Canonical diagnostics; installed metadata plus eligible project reuse; no application binary produced |
| `sifr build/run/test <input>` | Same frontend inputs, then the appropriate declared native build/test/run stages |
| `--release` | Explicit generated-application release profile |
| `--no-incremental` | Bypass only project semantic disk reuse, retaining normal metadata/native behavior |
| `--timings` | Human timing/cache summary on stderr, without polluting emitted source or program output |
| `--trace-dir <dir>` | Opt-in versioned trace/artifact diagnostics with redaction and controlled size |
| `sifr sysroot build-metadata ...` | Explicit canonical metadata producer |
| `sifr sysroot validate-metadata [--source-root <root> --metadata <file> --target <triple>]` | Complete structural traversal of the selected installed artifact, or an explicit host-produced target artifact; reports full portable-record digest |
| `sifr doctor --verify-integrity` | Read-only complete metadata and installed package/compiler digest verification |
| `sifr doctor [--json] [--verify-integrity]` | Report selected compiler/native toolchain, sysroot/metadata identities, readiness and cache locations; full package integrity only when requested |
| `sifr cache inspect [--json]` | Read-only size, generation, ownership and scope summary |
| `sifr cache prune [--dry-run] [--reserve-bytes <bytes>]` | Pressure/obsolescence-based safe reclamation; no deletion of active generations or unrelated worktrees |
| `sifr cache prune-project <workspace> [--dry-run] [--reserve-bytes <bytes>]` | Reclaim inactive semantic generations for the exact workspace under free-space pressure |

The prune reserve defaults to zero, so default invocation does not force reclamation. Set the reserve required by the planned operation; active entries remain protected.

Reuse existing CLI spelling where an equivalent command already exists; converge to one surface instead of adding aliases with separate implementations. CLI flags must be covered by help/argument/JSON tests. Invocation preserves the current program-argument separator, run working-directory semantics and exit-status contract. A cache directory never becomes the user's working directory accidentally.

The directory sink requires a new directory whose parent exists and writes
`trace-v1.json` (schema version 1, at most 32 KiB and 64 owner reports).
Existing destinations are rejected without overwriting files. The initial marker
is `incomplete`; normal completion records the command exit status. Sink setup
failure exits 2 before command execution; finalization failure preserves a failed
command's status or changes success to exit 2. Abrupt termination may leave an
incomplete marker; no completion is inferred. Owner strings, paths, source,
environment, arguments and process output are omitted; known stage/status names
are allowlisted. Dropped reports and omitted stages are counted. Invocation wall
time includes setup and command preparation; measured tracing overhead includes
sink setup, report projection and serialization work before the final write.
Both timing fields explicitly exclude the final artifact write. Owner stage
intervals may overlap and must not be summed; absent measurements remain absent.

### 11.2 Diagnostic categories

Source diagnostics carry a Sifr code, applicable source span, related information and an explanation/fix where one is known. A failed inference/capture analysis must not create unrelated-looking cascades of `Unknown` diagnostics; dependent error state preserves the root cause.

Environment/toolchain diagnostics name expected and selected identities/paths, distinguish missing from stale/incompatible/corrupt state, and provide a concrete remedy. Omit an unavailable source-location label when there is no legitimate source span. A stale development artifact points to the preparation/producer action; broken installed metadata points to selecting/reinstalling a matching toolchain.

Internal compiler or generated-code defects are clearly labeled. Preserve backend diagnostics and generated sources in the reproduction evidence; do not misleadingly attribute compiler-generated Rust to the user or discard backend details necessary for FFI/link failures. Map a backend error to Sifr source only when the mapping is trustworthy.

Canonical facts are separate from rendering. `human`, compact, JSON and LSP views retain the same codes and semantic meaning. Code suggestions apply only to the source version they target. Tests apply machine-applicable fixes and recheck the result. Error-document examples have explicit compile/check/run expectations and are tested, rather than indiscriminately executed. [R5]

### 11.3 Timing and trace model

Extend existing `BuildReport`, frontend trace and verification reporting. Do not build a second observability framework.

Record startup, sysroot resolution, metadata header/payload reading, project discovery, input validation, cache restore, parsing/lowering/checking, generation, native preparation/build and runtime/test stages. Distinguish total wall duration, exclusive stage work, parallel aggregate work and queue/wait time. Do not add overlapping durations and call that elapsed time. Report native compile/link together when separate measurements are unavailable.

Cache events identify family, hit/miss/rejection reason, loaded module/record counts and rebuilt native units. Traces record enough identity to explain reuse without dumping credentials, complete environments or user source by default. Measure instrumentation overhead and keep high-detail tracing opt-in. Human stderr, program stdout, emitted Rust and LSP JSON-RPC transport remain separate.

## 12. Verification orchestration and evidence

### 12.1 One plan for preparation and execution

Extend `sifr_verify`; retain Python orchestration and Cargo builds. Named typed steps declare immutable inputs, prerequisites, concrete output artifacts, assertion selection, safety deadline, resource class and expected outcome. A step requests its dependency artifact rather than independently searching for `target/debug/sifr`. Identical preparation requests are memoized within a plan; native freshness across invocations remains Cargo's responsibility. [R4]

Run cheap static schema/ownership/inventory checks before expensive build steps when dependencies permit. Offline execution follows explicit dependency acquisition/preparation. Preparation time remains visible in total command cost and is never relabeled as free.

The plan validates area/suite identifiers centrally. Use canonical identities rather than deriving behavior through inconsistent string prefixes. A configured binary override must be checked against the selected candidate/configuration, not accepted merely because it exists.

### 12.2 Diagnostic collection and qualification

Support focused case/suite selection, `--no-fail-fast` for independent assertions, and rerun selection from an earlier failure report. Continue independent cases after a functional failure; mark dependent cases blocked after a prerequisite failure. Stop on unsafe execution, resource exhaustion or invalid required inputs. Collecting failures never changes a failed/blocked run into a pass.

During iteration: collect the bounded relevant failures, repair the approved related batch, and run failed/affected tests. Unknown change ownership selects a conservative broader set. The final merge/release profile retains all required coverage regardless of local affected-test shortcuts.

Keep build-result reuse separate from test-result reuse. A warm executable can be reused, but selected runtime assertions execute. Required lint diagnostics must not vanish behind stale root-package outputs. Deliberate negative controls prove the harness detects a second broken fixture after a first valid fixture passes.

### 12.3 Process execution contract

One execution abstraction owns argv/cwd, the resolved non-secret environment policy, stdout/stderr capture, exit/signal status, durations, deadlines, cancellation and descendants. Drain streams without deadlock and preserve bounded raw output plus truncation status. On failure, retain the original cause even if cleanup also fails.

Use platform process-group/descendant ownership on the supported hosts. Complete cancellation cleanup before deleting workspaces or starting conflicting mutation. A disconnected observer does not change the recorded run result; a machine-readable live status file and append-only event log are enough. No daemon/job service is required.

Runner-controlled events travel separately from arbitrary child text. Expected-negative tests succeed only when the expected stage/code/assertion occurs. Missing tools or setup failures cannot satisfy a language-negative test. Cargo's build-finished message alone does not establish that a subsequent test or executed program succeeded. [D3]

### 12.4 Evidence and review records

Emit one canonical report per run, with stable test IDs, selection digest, candidate/artifact identities, environment class, preparation/cache state, command outcomes, independent functional/performance status, and references to raw evidence. Use adapters for existing reports rather than duplicating facts into many competing authoritative files.

Evidence is keyed to the candidate and its complete validation inputs. An unrelated documentation-only commit may retain original evidence only with an explicit unchanged-input comparison and reference to the original run; do not manufacture a new execution record. A failed cold result remains recorded when a later warm run succeeds.

Apply the existing bounded implementation workflow: bounded item, targeted validation, one final required gate on the final implementation candidate, scoped review, and a compact handoff. Do not commit the approval into the commit it approves. Record-only updates do not trigger another broad gate or external review. Historical evidence stays linked outside the current-status summary. [S11]

## 13. Test architecture and full-stdlib qualification

### 13.1 Shared fixture model, distinct guarantees

Use one canonical fixture inventory and shared materialization orchestration. Preserve different assertions:

| Assertion | Guarantee |
| --- | --- |
| `check-pass` / `check-fail` | Sifr frontend accepts or reports the specified code/span/result |
| Rust check | Generated Rust passes native compiler checking |
| `build-pass` | Native compilation and linking succeed |
| `run-pass` / runtime-negative | The executable exhibits the expected behavior/exit outcome |
| Snapshot/quality | Emitted text, diagnostics, source maps, lints or safety structure meet their specific expectations |
| Incremental equivalence | Fresh, in-memory and restored paths agree after an edit sequence |
| Metadata equivalence | Source-produced and metadata-loaded semantic/codegen results agree |

Do not rename `cargo check` plus Clippy to `build-pass`; it does not prove linking. Do not replace all snapshots with native tests or vice versa. Existing demos/companions retained for documentation or audit remain explicit consumers of the shared generation output. [R5]

Normalized diagnostic baselines exclude only nonsemantic unstable paths/formatting. Independent assertions check codes, spans and selected values. `--bless`/snapshot acceptance is deliberate and scoped; acceptance never blesses a crash, timeout, missing test or disabled safety rule. Changes in allowed lint debt require the owning policy, not an automatic baseline refresh.

### 13.2 Complete metadata verification versus lazy E2E

Metadata qualification enumerates every canonical public/private module for the selected target, produces its artifact once, decodes every section, validates references and templates, and compares source-built versus decoded records. It validates all reusable emitted payloads and supported specialized/codegen cases. Production uses the exact artifact bytes covered by qualification.

Normal E2E tests continue through the ordinary lazy reader. Do not pre-touch every module in each test process; that would hide demand-loading bugs and erase the memory/startup benefit. Across the declared suite, maintain a map from stdlib/API/support families to behavioral/native fixtures. Full section decoding is not proof of complete API behavior. Uncovered supported surfaces are recorded as coverage gaps, not implicitly certified by the metadata inventory.

Full source production is performed when its complete inputs change and during explicit reproducibility/qualification exercises. Do not rebuild metadata before each fixture or repeatedly rebuild an unchanged candidate merely to warm a time budget. The release artifact is qualified after final production; packaging cannot replace it with newly generated, untested bytes.

### 13.3 Profiles

| Profile | Required selection |
| --- | --- |
| Create-PR | Static/preparation checks, metadata identity/index/full structural decoder validation for the candidate configuration, representative native/stdlib tests, affected families, cache negative controls and deterministic budget-policy tests |
| Merge | Full configured semantic/E2E and required native/lint suites, complete metadata source/reader equivalence for candidate configuration, runtime boundary matrix, relevant persistence/editor failure tests and controlled required performance selection |
| Release | Each supported target/toolchain package, exact distribution install/update/relocation behavior, full declared stdlib/native parity corpus and complete metadata payload validation, explicit production-profile and loader tests |
| Nightly/broad | Extended fuzzing, generated edit sequences, broader interaction/scale matrices and controlled long-running benchmarks; it never replaces mandatory correctness coverage in an earlier profile |

Keep existing selected guarantees while migrating the runner. Profile labels are not coverage proofs: reports enumerate actual assertions and blocked/not-run cases. A structural decoder test may intentionally load the full artifact in its own verification process; ordinary CLI/LSP/E2E must remain lazy.

### 13.4 Independent correctness and negative controls

Source-versus-metadata and fresh-versus-restored comparisons can agree while both are wrong. Retain independently specified semantic results, native compilation/linking, runtime behavior, expected errors, and existing IR invariants. Validate IR at the transformation boundaries that can introduce malformed references, ownership conventions, support demand or project layout. [R6] [T5]

Add seeded failures for stale compiler identity, corrupted records, same-name root aliasing, missing native outputs, stale diagnostics, trust revocation and fake child status events. Fuzz decoders and edit sequences within bounded resources. A changed baseline alone cannot explain away an incremental semantic discrepancy.

## 14. Performance and supported-machine contract

### 14.1 Measurement categories

Separate fresh installed checking, unchanged new-process checking, edited project checking, warm in-memory/LSP queries, native first build, native no-op build, edited native rebuild, and contributor gates. Record whether stdlib metadata, project state, Cargo artifacts, registry sources and filesystem pages are warm or cold. [R7]

Use the qualified Linux i7-4720HQ / 12 GB reference and each existing named host profile. Record actual CPU/core limits, memory limits, toolchain/compiler identity, disk/cache placement, power/governor state where available, source/corpus ID and interference. A Mac threshold is not silently applied to a different host. A tightly passing LSP cap is not proof that the entire machine is out of RAM.

Default the constrained contributor profile to the established bounded native-build concurrency; coordinate outer worker counts with Cargo/jobserver behavior rather than multiplying independent full-machine budgets. Memory safety concerns can serialize heavy cases without changing their test semantics. More resources are an optional faster profile, not a prerequisite to supporting the floor.

### 14.1.1 Compiler measurement lanes

DX.1 creates two explicit compiler measurement lanes before optimization:

| Lane | Executable and purpose |
| --- | --- |
| `product-installed-optimized` | Directly invoke the optimized packaged compiler/LSP with its matching installed sysroot. This lane owns the 300 ms and 50 ms CLI product targets and the paired product-memory comparison. |
| `contributor-dev` | Invoke the candidate development/test compiler and measure compiler rebuilds, bare Cargo tests, metadata preparation and verification workflow cost. This lane retains contributor-specific regression contracts. |

Record concrete executable path/digest, embedded compatibility identity, actual compiler build settings, sysroot identity, generated-application profile, verification selection and cache/page state separately. Do not hardcode `build_profile: dev` while running an optimized compiler or compare debug/dev samples to product thresholds. Keep benchmark/test output reproducible through validated artifact selection. A missing or mismatched identity/profile is an incomparable measurement, not a passing product sample. The baseline `reference_host.py` and binary-selection helpers are concrete migration sites. [S17]

Build/package preparation is reported separately but never omitted from the relevant whole-workflow measurement. Product `check` latency times the installed executable itself, not `cargo run` or a compiler build; contributor iteration includes the work actually required by that workflow. Reuse case definitions across lanes but assign disjoint result/budget identities. [R7]

### 14.2 Product targets

The following are acceptance targets. Actual measured scopes and results are recorded in the [DX.15 qualification](../plans/issues/active/ad-hoc-compiler-dx-and-toolchain-reuse.md#dx15-completed-qualification); the table does not extend those claims to other workloads:

| Workload | Target |
| --- | --- |
| Fixed small-program corpus, installed optimized compiler, no project cache, warm filesystem pages | p95 fresh-process `check` below 300 ms |
| Same corpus, unchanged saved inputs, compatible project state, new process | p95 `check` below 50 ms and demonstrated reused frontend work |
| Declared representative editor edit-to-diagnostics workload | Within the existing applicable 250 ms product target, with no stale results |
| Existing large-session memory fixture | Preserve its existing applicable 128 MiB cap and host-specific policy |
| Unchanged prepared native build | No unnecessary Rust recompilation or relinking; wall latency measured and compared |
| Contributor profiles | Preserve the reviewed profile-specific end-to-end DX objectives; account for preparation and distinguish operational deadlines from performance policy |

Larger and interop-heavy project workloads have their own fixed corpus and budgets. Freeze them before optimizing the candidate. Genuine cold-filesystem startup is measured separately, not called a warm-page result. Do not extrapolate a tiny-query microbenchmark to all projects or promise that metadata alone achieves the targets.

### 14.3 Comparison and gate policy

Use candidate-versus-pinned-base measurements on the same controlled host, supplemented by release/long-term anchors so repeated small regressions do not disappear into a moving baseline. Record baseline provenance and exact inputs. Instruction/work counters help attribute compiler work; wall latency, memory and I/O/wait behavior remain required DX measurements.

Take enough independent samples for the claimed percentile and uncertainty; record warmups separately, randomize/interleave comparison order where appropriate, and report median, p95, spread and invalid runs. Do not report a reliable p95 from a handful of samples. Host noise is an invalid/inconclusive performance measurement requiring policy-defined treatment, not a source-correctness failure or a justification to rerun until green.

Correctness steps have safety deadlines and resource bounds. Speed thresholds are evaluated as named performance contracts with explicit cold/warm workload definitions. Functional success and performance failure may coexist; qualification applies the selected policy and remains failed when a mandatory performance contract fails. Required performance contracts cannot be disabled to obtain release approval. Existing waiver governance remains explicit and may never waive stale results, safety or trust violations. [S12]

Cache setup, metadata decoding, input validation, serialization, artifact capture and environment probes stay in the relevant end-to-end measurements. Do not move work into an unreported prelude to meet a latency budget. Tracing overhead is measured separately.

#### Legacy threshold migration

Inventory every existing `budgets.json` entry and classify it by host, actual compiler profile, workload, cache state, metric, source authority and purpose. Old Mac/dev absolute captures remain immutable historical anchors and are not applied to Linux or optimized-product lanes. Superseded entries are explicitly marked historical/non-gating for the replacement lane with a recorded mapping; do not simply disable `budgets.json` or all absolute limits.

A correctly scoped active host/profile regression contract remains enforceable until a prospectively reviewed replacement is qualified. Product limits, safety deadlines and applicable memory caps remain active. Candidate-versus-pinned-base comparisons and valid long-term regression anchors use comparable lanes only. Missing comparator evidence is reported as unavailable/inconclusive under the selected qualification policy, never automatically accepted. A synthetic mismatch test proves that cross-host/profile data cannot accidentally gate or pass the wrong lane, while an actual seeded same-lane regression still fails. Historical failed results are not rewritten.

## 15. Migration and completion rules

Adoption is an explicit contract change to the existing performance policy's process-local-only deferral and to the old size-based cleanup rule. Preserve existing performance and editor semantic ownership and tooling protocols. Update `internal_docs/architecture.md`, the appropriate subsystem docs, profile policy and the phase index with links instead of copying this entire contract into each location.

Migrate consumers before removing old aggregate/cache interfaces. During migration, a test-only source-built reference path and the production metadata reader may coexist; they must use the same semantic compiler. At phase exit there is no normal installed source-bootstrap fallback and no unqualified global stdlib singleton.

Retire shadow binary resolvers, ad hoc Cargo launchers, duplicate identity calculators and legacy result caches only after all known consumers use the new contract. Compatibility with old private metadata/project-cache formats is not required: installed mismatches diagnose; optional project records miss. Existing public command semantics remain unchanged except for explicitly documented new capabilities and application profile defaults.

Completion requires the acceptance matrix, the sequential phase's definitions of done, and measured user/contributor workflows on supported configurations. It does not require recursive closure reviews or new governance. The phase status records actual implementation and evidence; this document remains the architecture reference.

Sequential implementation is not an all-or-nothing release barrier. The integrated metadata result after DX.8 and metadata/native/profile/editor result after DX.11 are release-eligible checkpoints once their exact artifacts satisfy existing publication qualification and their then-implemented public behavior. They do not assert that later project persistence exists or that Phase DX is complete. Reuse applicable candidate-bound evidence rather than add checkpoint-only duplicate reviews/gates. Actual release authorization/account operations remain governed elsewhere.

DX.12–DX.14 implement the required project persistence and interface propagation. The DX.8/DX.11 measurements guide tuning and identify remaining costs. Qualified improvements may ship at a checkpoint while the phase continues sequentially. Maintain implementation status and qualification evidence in the companion implementation plan.

## 16. Acceptance and failure-injection matrix

This is the canonical acceptance-case inventory. The implementation plan references these IDs instead of duplicating their semantic requirements. Each case needs executable coverage and evidence; the table defines required assertions; the canonical phase evidence records their qualified scopes.

| ID | Scenario | Required outcome | Owner |
| --- | --- | --- | --- |
| I01 | Change compiler implementation without changing package version or HEAD. | Embedded compatibility identity changes; incompatible metadata/project results are rejected. | compiler build / frontend |
| I02 | Change a compiler input, source inventory, parser submodule, build feature or generated schema input independently. | Build tracking updates the identity; source-archive and dirty-checkout builds cannot retain a stale ID. | compiler build |
| I03 | Change only diagnostic color, width or LSP position encoding. | Semantic results remain reusable; current presentation is rendered correctly. | frontend / diagnostics |
| I04 | Change target, Rust toolchain, lock/source configuration or relevant native environment. | The correct semantic/native family invalidates; no stale executable is accepted. | driver / package |
| I05 | Reorder unordered producer collections or move the producer checkout. | Canonical payloads are reproducible; no build-machine path or allocation-order identity leaks. | metadata producer |
| I06 | Add a nominally named declaration in another package/version. | Artifact-local identities remap without aliasing the existing declaration. | IR / lowering |
| I07 | Change only an LSP implementation input and rebuild the compiler. | Changed code and its normal dependents rebuild; a volatile aggregate identity does not force frontend/sysroot libraries to rebuild. Capture actual Cargo freshness/artifact evidence. | compiler build |
| I08 | Run a library test after changing its linked semantic implementation, features or test configuration without changing HEAD/version. | Its compiled test identity changes where required; no constant test ID or unrelated mutable source scan authorizes stale metadata. | compiler build / test adapters |
| Metadata case M01 | Run a tiny installed check with no project cache. | Zero stdlib source bootstrap/emission; only the required metadata closure is decoded. | driver / frontend |
| Metadata case M02 | Use defaults, const bodies, generic templates, descriptors and interop-heavy stdlib declarations. | All required semantic/codegen payloads load and agree with source production. | lowering / codegen |
| Metadata case M03 | Compile the maintained E2E corpus through source-built and artifact-loaded stdlib providers. | Generated Rust is byte-identical for identical declared inputs, with native and behavioral assertions retained. | codegen / verification |
| Metadata case M04 | Rewrite/specialize/reassemble a prevalidated Rust fragment; include invalid lexical joins. | Only unchanged valid fragment assurances are reused; required transformed/final validation catches the defect. | codegen |
| Metadata case M05 | Decode every canonical public/private module and required section. | Inventory, bounds, references, template and payload validation are complete; no module is silently absent. | sysroot / verification |
| Metadata case M06 | Truncate metadata or inject invalid lengths, offsets, IDs, recursion or payload digests. | Bounded actionable rejection; no panic, runaway allocation or false success. | sysroot / IR |
| Metadata case M07 | Move an intact installed toolchain. | Relative source/payload references and navigation still resolve. | distribution / source |
| Metadata case M08 | Upgrade/rollback during a live LSP session; open two compatible sysroot stores in one process. | Requests pin a consistent generation; stores cannot contaminate one another; incompatible artifacts reject. | sysroot / analysis |
| Metadata case M09 | Correct a missing or stale development artifact after an initialization error. | Explicit ensure/re-resolution retries the canonical producer; no permanent global error poisoning. | driver / sysroot |
| Metadata case M10 | Attempt production from a cross-build machine. | A supported host producer executes with explicit target semantics; no accidental execution of a foreign target binary. | distribution |
| Metadata case M11 | Produce on macOS and Linux under equivalent semantic inputs/target. | Portable payload bytes agree; legitimate producer-envelope/target differences are not falsely compared as identical. | metadata / distribution |
| Metadata case M12 | Modify producer inputs during metadata construction. | Output binds to captured inputs and is not selected as current for different live source; bounded retry or changed-input error. | driver / source |
| Metadata case M13 | Start N threads/processes ensuring the same dev-metadata key with an empty cache; repeat warm and with different configurations. | One successful production per identical key; all waiters validate complete output and run assertions. Incompatible identities remain isolated. | driver / test adapters |
| Metadata case M14 | Kill/fail/cancel the producer while other metadata-ensure callers wait. | No partial artifact or permanently cached failure escapes; ownership is released and a later eligible caller can retry without deadlock. | driver / storage |
| Metadata case M15 | Run public sifr emit through source-built and metadata-loaded providers over the maintained corpus. | Stdout bytes, final newline, stdlib preamble markers and derived source-map ranges match; timings do not contaminate emitted output. | CLI / codegen / source |
| Metadata case M16 | Encode/decode many signatures referring to a large class, recursive references and distinct binders/nominal views. | Storage/retention scales with unique records plus references; permitted recursion terminates and distinct semantic identities do not merge. | IR / type system / sysroot |
| Metadata case M17 | Compile two projects sharing one baseline, then fail/delete/reconfigure an export in only one. | Mutations remain in the correct overlay; no stale export, reserved-stdlib shadowing or cross-project contamination occurs. | lowering / frontend / driver |
| C01 | Kill a cache writer before, during and immediately after publication. | A complete valid entry or miss remains; partial state never becomes readable success. Requalify with project generations in DX.13. | driver storage |
| C02 | Run concurrent readers/writers and prune. | Entry-level OS ownership protects current native/test state and publication; extend to project generations in C09. | driver storage |
| C03 | Move persistent cache to a filesystem different from TMPDIR. | Staging remains under the destination filesystem; atomic publication is preserved. | driver storage |
| C04 | Use full, read-only, inaccessible or unsupported project-cache storage. | Actual semantic outcome survives with explicit unavailable-cache status; required native/artifact output failures remain errors. | driver / frontend |
| C05 | Remove a manifest-referenced payload or replace a concurrently published winner with an invalid entry. | Entry is revalidated and rejected; directory existence never proves completeness. | driver storage |
| C06 | Run on a target larger than 20 GiB with sufficient free space, then simulate real disk pressure. | No size-only clean; pressure cleanup reclaims only eligible inactive owned entries. | verification / storage |
| C07 | Inject path traversal, unsafe symlink destinations or foreign-owned cache entries. | Writes/pruning cannot escape the authorized cache scope; untrusted entries do not grant execution. | storage / package |
| C08 | Rotate A/B/A source generations without touching an eligible result in B. | Still-required valid records survive publication and can be reused in A subject to input validation. | frontend / storage |
| C09 | Use real inherited project-result generations with concurrent readers/writers and GC, including unused-but-valid records. | Finalized records remain immutable and active records survive; interrupted publication is a valid generation or miss and A/B/A eligibility is preserved. | frontend / driver storage |
| B01 | Repeat an unchanged native build/run. | Cargo confirms freshness; no needless file rewriting, Rust recompilation or relinking; the program still executes when requested. | driver |
| B02 | Compile two generated roots with identical display/package/binary names; make the second deliberately invalid. | Root/output isolation detects the intended error or lint and preserves each case diagnostics. | driver / codegen verification |
| B03 | Run builds concurrently while one root is edited or its output copied. | Owned mutation and artifact capture prevent source/output races; no shared mutable path is a durable result. | driver |
| B04 | Validate an unpushed local compiler candidate; separately export a portable project. | Local paths work locally; portable source/lock rewrite is explicit and independently validated. | driver / package |
| B05 | Exercise development, test and release applications, including actual release executables. | Required language, numeric, error, ownership and panic-boundary semantics hold; profile selection is explicit. | runtime / driver |
| B06 | Change a minimal/default/feature-isolation configuration while grouped integration tests pass. | Isolation regression remains detectable; grouping preserves the executed assertion/configuration inventory. | verification |
| B07 | Compile successfully but fail at native link; link successfully but fail at runtime. | Each required stage fails correctly; a shallower success cannot satisfy a deeper assertion. | driver / verification |
| B08 | Select Python library A at build time but expose library B at runtime. | Qualified loader behavior or a concrete mismatch diagnostic; no silent wrong-library success. | Python interop / driver |
| B09 | Change trust, build-script inputs, native flags or relevant compiler-component/environment inputs after a cache hit. | Current authorization and Cargo/environment freshness remain effective. | package / driver |
| B10 | Run nested Cargo/native work on the constrained resource profile. | Total concurrency remains bounded; cancellation leaves no owned descendants or source mutation behind. | verification / driver |
| B11 | Change the default run profile while executing the inventoried E2E/profile selections. | The release run-pass corpus and release boundary assertions remain explicit and execute; dev coverage is additional and defaults have their own tests. | verification / runtime |
| B12 | Alternate dev and release builds with both caches retained on the 12 GB host. | Measure time, memory, duplicate/reused units and actual disk use; useful profile artifacts remain valid and no size-only cleanup is triggered. | driver / performance |
| P01 | Check unchanged saved inputs in a new process. | Completed frontend work is actually restored; diagnostics agree with fresh mode and counters prove reuse. | frontend |
| P02 | Introduce an error, fix it, then revert. | Completed error and success results remain correct; stale diagnostics disappear and reappear appropriately. | frontend |
| P03 | Create a previously missing import or a higher-priority resolver candidate. | Resolution and all affected results update despite unchanged previous resolved files. | frontend / package |
| P04 | Delete/rename files, change source inclusion rules or switch package configuration. | File-set/dependency invalidation follows the current resolver and cycle semantics. | frontend |
| P05 | Change a default, constant, generic/const body, inferred effect or ownership contract without changing an obvious signature. | All semantically affected importers recheck; interface-only reuse never hides the change. | frontend / lowering |
| P06 | Change an ordinary private implementation body with an unchanged proven interface. | Local/codegen/native work invalidates; eligible importer checking is reused and fresh results still agree. | frontend |
| P07 | Hash and analyze a file while another process edits it. | Records describe the captured bytes; no mismatched source hash/result is published. | source / frontend |
| P08 | Restore a check then request missing typed build/editor data. | Missing families are computed normally; a cached successful check is not a complete process image or executable. | frontend / driver |
| P09 | Cancel/crash partway through checking or fail a transient environment query. | No incomplete aggregate is published as success; deterministic completed error records follow their distinct contract. | frontend |
| P10 | Use --no-incremental or remove .sifrbuildinfo. | Same semantic result through normal computation; installed stdlib metadata is still used and no cache requirement leaks into correctness. | CLI / frontend |
| P11 | Alter a SQL schema/provider identity, Python declaration/ABI context or compiler component. | Only complete declared semantic reuse applies; live certification/trust is not replayed as immutable authority. | frontend / package |
| P12 | Deserialize module results with changed local allocation order or relocated source-map IDs. | Stable IDs remap and canonical diagnostics point to the correct captured source. | IR / frontend |
| E01 | Maintain unsaved editor text different from disk while running CLI checks. | Each sees its own authoritative source snapshot; persisted saved-source state is uncontaminated. | analysis / LSP |
| E02 | Complete an older request after a newer edit/configuration change. | No stale result or registration overwrites newer state, even when cancellation arrives too late. | LSP |
| E03 | Exercise Unicode, CRLF, relative roots, symlinks and supported case rules. | Source identities, cached spans and negotiated LSP positions remain correct. | source / LSP |
| E04 | Open/edit/close/switch projects repeatedly with outstanding requests. | Referenced snapshots survive; unreferenced state is released; memory remains within its scoped budgets. | analysis |
| E05 | Request formatting or a syntax-only editor operation with no semantic project needed. | No unrelated metadata/native/environment work is forced. | format / analysis |
| E06 | Use a missing/incompatible installed metadata artifact from an editor. | Actionable setup diagnostics and recoverable session behavior, not fabricated semantic success. | LSP / sysroot |
| E07 | Apply a completion/fix/rename edit and immediately recheck. | Resulting text and versions are correct; safe fixes remove the intended error without introducing an unexpected one. | analysis / diagnostics |
| R01 | A child prints runner-like pass/fail events, including a negative self-test failure. | Runner-controlled aggregate status is unaffected by child text. | verification |
| R02 | Cargo emits build-finished success, then the test/program exits nonzero. | The overall required runtime/test result remains failed. | driver / verification |
| R03 | Timeout or cancel a process with nested children and partial binary/text output. | Original cause and bounded streams survive; owned descendants terminate before cleanup. | driver / verification |
| R04 | A negative fixture cannot launch because its compiler/tool is missing. | Setup failure cannot satisfy the expected language diagnostic. | verification |
| R05 | Run no-fail-fast with one failed prerequisite and two independent case failures. | Dependents are blocked, independent failures are collected, and final qualification remains nonpassing. | verification |
| R06 | Compare grouped/prepared execution with the original selected configurations. | Binary identity, effective arguments/environment and assertion inventory match; no hidden second compiler build. | verification |
| R07 | Reuse warm artifacts during a required qualification run. | Every selected assertion executes or is explicitly blocked/not-run; build cache receipts do not impersonate test results. | verification |
| R08 | Add a source fixture or change only records/documentation. | Cheap inventory checks catch mismatches early; valid unchanged-input evidence is preserved without fabricated reruns. | verification / phase workflow |
| R09 | Read help/doctor for stale source-tree and installed artifacts. | The correct remedy and selected artifact/toolchain identities are reported without a fake source location. | CLI / diagnostics |
| R10 | Run bare cargo test -p sifr_driver from a clean source checkout without a CLI binary or injected verification environment. | Relevant tests ensure matching metadata through their linked canonical producer/test identity; no recursive Cargo invocation or implicit installed fallback is needed. | driver / test adapters |
| Q01 | Both reference and optimized paths share the same compiler defect. | Independent expected results, IR invariants and native/runtime assertions still detect it. | verification |
| Q02 | Compare cold preparation, warm reuse, tracing-enabled and tracing-disabled runs. | Workload/overhead attribution stays explicit; total DX cost does not omit moved work. | performance |
| Q03 | Functional tests pass but a required performance target fails or sampling is inconclusive. | Separate outcomes are preserved and qualification applies explicit policy without rerunning until green. | performance / verification |
| Q04 | Install, move, update and roll back every supported release package. | The exact qualified compiler/metadata/runtime bytes work together; no source-tree path or stale generation leaks. | distribution |
| Q05 | Run full metadata traversal plus ordinary lazy E2E and native parity coverage. | Complete structural validation is explicit; lazy behavior remains tested and uncovered API families are not falsely certified. | verification |
| Q06 | Run the declared 12 GB developer workload and fixed latency corpus. | Recorded product/resource targets hold without upgrading the machine, weakening tests or hiding preparation. | performance |
| Q07 | Supply dev compiler results to product thresholds, or mislabeled compiler/application/verification profiles. | Measurement is incomparable/rejected; the actual optimized installed artifact is selected and separately identified for product targets. | performance / verification |
| Q08 | Apply a legacy Mac/dev anchor to Linux/product results; separately seed a valid same-lane regression. | Wrong-lane historical values cannot gate/pass the run, while active scoped regression/product/resource contracts still enforce failure. | performance policy |
| Q09 | Compare pre-migration and candidate demanded-stdlib LSP loading/retention under matching optimized installed conditions. | Report steady/peak RSS and allocation deltas with noise and decoded counts, preserve scoped caps and attribute changes; no historical 130 MiB claim substitutes for measurement. | analysis / performance |

## 17. Engineering precedent and source references

The normative requirements above define Sifr's target architecture. Pinned repository references identify the baseline integration points and engineering precedents. Performance and correctness claims require the qualification evidence specified in this document. Official documentation references can evolve.

| Pattern | Source and bounded lesson |
| --- | --- |
| Prebuilt lazy semantic data | [R1]: versioned metadata and lazy values/tables; Sifr additionally carries its own reusable Rust payloads. |
| Conservative compatibility and transactional reuse | [R2] [R3]: exact-version incremental acceptance, working/finalized generations and reader/GC coordination. |
| Dependency-aware orchestration | [R4]: typed step outputs and explicit prerequisites, not a general distributed build system. |
| Compiler validation | [R5] [R6]: distinct checking/linking/runtime assertions and internal representation invariants. |
| Selective project persistence | [T1]: restore declared build-info state, not the full live checker. |
| Editor lifetime and text ownership | [T2] [T3] [T4]: immutable snapshots, overlay ownership and reference-counted caches. |
| Incremental correctness tests | [T5]: compare edited incremental state with a fresh system, with independently specified outcomes retained. |
| Native build reuse | [D1] [D2] [D3] [D4]: explicit profiles, compatible feature unification, structured artifact messages, declared build-script inputs and jobserver coordination. |
| Performance assessment | [R7]: explicit compilation scenarios and compiler profiles. Numeric product targets here belong to Sifr. |

### Repository integration references

[S1]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_driver/src/stdlib/bootstrap.rs "Sifr stdlib bootstrap and definition projection"
[S2]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_driver/src/stdlib/cache.rs "Sifr process-local stdlib cache"
[S3]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_lowering/src/lower/external_defs.rs "External semantic definitions"
[S4]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_codegen/src/stdlib_codegen_metadata.rs "Codegen payloads and templates"
[S5]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_frontend/src/cache_keys.rs "Frontend identities"
[S6]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_frontend/src/workspace_residency.rs "Existing build-info verification"
[S7]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_driver/src/build/materialize.rs "Native materialization"
[S8]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_driver/src/build/workspace.rs "Artifact cache and staging"
[S9]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/AGENTS.md "Contributor rules and cleanup policy"
[S10]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_codegen/src/inline_syntax.rs "Existing fragment-boundary syntax validation"
[S11]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/AGENTS.md "Contributor workflow"
[S12]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/plans/phases/35_performance_benchmarking_and_budgets.md "Existing performance/query/cache contract"

[S13]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr/build.rs "Executable package identity injection point"
[S14]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_type_system/src/types/definitions.rs "Owned class and function type representation"
[S15]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_driver/src/project/frontend.rs "Project external definitions are mutated as exports are collected"
[S16]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/crates/sifr_driver/src/frontend/api.rs "Emitted support markers and source-map extraction"
[S17]: https://github.com/sifr-lang/sifr/blob/0f819c2f04bf5b2891074c55ba26369ddf4f13bd/verification/areas/performance/reference_host.py "Baseline dev-profile measurement metadata"

### Rust, TypeScript, and Cargo references

[R1]: https://github.com/rust-lang/rust/blob/215a8af4bb4c106cccf6d6535f84eaae91818265/compiler/rustc_metadata/src/rmeta/mod.rs "Rust metadata representation"
[R2]: https://github.com/rust-lang/rust/blob/215a8af4bb4c106cccf6d6535f84eaae91818265/compiler/rustc_incremental/src/persist/file_format.rs "Rust incremental compatibility"
[R3]: https://github.com/rust-lang/rust/blob/215a8af4bb4c106cccf6d6535f84eaae91818265/compiler/rustc_incremental/src/persist/fs.rs "Rust incremental generations and locks"
[R4]: https://github.com/rust-lang/rust/blob/215a8af4bb4c106cccf6d6535f84eaae91818265/src/bootstrap/src/core/builder/mod.rs "Rust bootstrap steps"
[R5]: https://rustc-dev-guide.rust-lang.org/tests/ui.html "Rust UI and rustfix testing"
[R6]: https://github.com/rust-lang/rust/blob/215a8af4bb4c106cccf6d6535f84eaae91818265/compiler/rustc_mir_transform/src/validate.rs "Rust MIR validation"
[R7]: https://rustc-dev-guide.rust-lang.org/profiling/with-rustc-perf.html "Compiler performance scenarios"
[T1]: https://github.com/microsoft/TypeScript/blob/d2b20b35034bd902bdda5974f522ff00352895e9/tsc/internal/execute/incremental/buildinfotosnapshot.go "TypeScript build-info restoration"
[T2]: https://github.com/microsoft/TypeScript/blob/d2b20b35034bd902bdda5974f522ff00352895e9/tsc/internal/project/session.go "TypeScript session"
[T3]: https://github.com/microsoft/TypeScript/blob/d2b20b35034bd902bdda5974f522ff00352895e9/tsc/internal/project/snapshot.go "TypeScript snapshots"
[T4]: https://github.com/microsoft/TypeScript/blob/d2b20b35034bd902bdda5974f522ff00352895e9/tsc/internal/project/refcountcache.go "TypeScript cache lifetime"
[T5]: https://github.com/microsoft/TypeScript/blob/d2b20b35034bd902bdda5974f522ff00352895e9/tsc/internal/execute/tsctests/runner.go "TypeScript incremental/fresh test oracle"
[D1]: https://doc.rust-lang.org/cargo/reference/profiles.html "Cargo profiles"
[D2]: https://doc.rust-lang.org/cargo/reference/features.html "Cargo feature unification"
[D3]: https://doc.rust-lang.org/cargo/reference/external-tools.html "Cargo structured artifact messages"
[D4]: https://doc.rust-lang.org/cargo/reference/build-scripts.html "Cargo build-script inputs and jobserver"

[R8]: https://rustc-dev-guide.rust-lang.org/memory.html "Interning and type memory ownership"
[D5]: https://doc.rust-lang.org/cargo/reference/build-scripts.html "Package-scoped build-script outputs and change tracking"
[D6]: https://doc.rust-lang.org/cargo/commands/cargo-test.html "Cargo tests: target execution, parallel test cases and no-fail-fast"

### DX.5 implemented transport boundary

The private v1 indexed schema and bounded record decoder live in
`sifr_sysroot::metadata`. [The pinned field/site inventory](compiler_dx_metadata_consumers.md)
documents stable-ID rules, explicit payload records, per-store shared handles and
the layered-view migration completed in DX.7. DX.6 activates canonical metadata
production and DX.7 activates normal CLI/LSP consumers.

### DX.12 implemented persistence-facing boundary

The frontend persistence module owns the ordered resolution journal, captured
source bytes, explicit semantic input contract, independent family completion
and conservative module-interface identity. CapturingSourceProvider must remain
alive across package discovery, resolution and parsing: repeated reads return
the captured bytes, and publication eligibility replays the actual observation
journal. Ordered absent candidates, directory membership, canonical aliases and
configuration reads are recorded without changing resolver precedence or cycles.
An environmental read failure cannot authorize a reusable record.

The package semantic_capture adapter uses the existing Cargo metadata parser,
package graph and source-map builder with that same provider. It records the
effective Cargo metadata and captured lock bytes, while manifest, namespace and
source-set observations come from actual resolver operations. The outer owner
supplies the compiler, metadata, target, language/diagnostic policy, component and
declared SQL/Python/interop identities. Required external observations must all
be resolved before input completeness; terminal presentation is excluded.

Canonical diagnostics store source identities and byte ranges, including
suggestion edits; the current source map allocates fresh source IDs and renders
current display paths, line/column and snippets. Typed interface and checked-HIR
families use the existing bounded indexed metadata codec through driver adapters.
Package/source identity anchors declarations; types and binders follow the
existing normal decode rules. Interface summaries include the complete semantic
export projection and a conservative exact-input stamp, so no unchanged-interface
optimization is claimed. Codegen handoffs identify checked inputs and required
specializations independently of executable materialization.

The declared checked-family consumers are typed HIR/codegen and semantic exports.
Syntax, flow-analysis queries and rich editor indexes remain separate unpersisted
families and are recomputed by their existing owners when demanded. These APIs alone do
not write project generations: the completed DX.13 adapters own cross-process
consumers, transaction boundaries and compatibility/miss policy, and DX.14 owns
proof-based narrowing of importer invalidation.
