# Phase 39: Rust Interop

status: completed, audited

> Phase 39 closed on 2026-06-22 through PRs #2702-#2728 and closeout [PR #2729](https://github.com/sifr-lang/sifr/pull/2729). Runtime/ecosystem certification beyond the supported matrix remains tracked by [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../issues/active/rust-interop-runtime-ecosystem-certification.md). Verification-matrix hardening completed through [PR #3018](https://github.com/sifr-lang/sifr/pull/3018), [#3019](https://github.com/sifr-lang/sifr/pull/3019), [#3020](https://github.com/sifr-lang/sifr/pull/3020), [#3022](https://github.com/sifr-lang/sifr/pull/3022), and [#3023](https://github.com/sifr-lang/sifr/pull/3023), with its final review and validation recorded in the [archived closeout](../issues/archive/rust-interop-verification-matrix-hardening.md).

## Objective
Deliver production-grade Rust interop as declaration-level Cargo integration.

This phase makes Rust-backed Sifr APIs feel like ordinary Sifr packages while preserving Sifr's panic-safety, type-safety, ownership, diagnostics, async/offload, package, and verification guarantees.

The canonical design is [`internal_docs/rust_interop_architecture.md`](../../internal_docs/rust_interop_architecture.md). That document is normative for syntax, lowering, build planning, trust policy, panic boundaries, async behavior, zero-copy, callbacks, and verification.

Rust interop intentionally lands before stable GA because stable promotion must not bless a package ecosystem whose Rust-backed package contract is still undefined. Phase 40 can promote stable only after Phase 39 has locked and verified the Cargo-backed Rust package surface that stable users and package authors will rely on.

## Scope

Phase 39 owns the full Rust interop implementation:

- `@rust(...)` declaration decorators and structured dotted-path resolution,
- direct Cargo crate bindings when signatures are bridge-compatible,
- package-local Rust bridge modules under `src/bridges`,
- shared bridge crate support through ordinary Cargo dependencies,
- opaque Rust-backed Sifr classes and handles,
- bridge-compatible type checking and conversion,
- panic boundary handling,
- explicit async, blocking I/O, and CPU-heavy classification on Sifr's Tokio runtime,
- zero-copy/view contracts for bytes, Arrow-style columnar data, tensors, and custom views,
- callback lifetime/threading/backpressure contracts,
- Cargo metadata, trust policy, cache-key, and build-plan integration,
- LSP/tooling support for Rust target paths,
- first-class `verification/areas/rust_interop` coverage.

## Out of Scope

- Python interop.
- Raw C ABI interop.
- Rust dynamic ABI loading.
- Rejected `extern rust` syntax. <!-- rust-interop-rejected -->
- Rejected runtime `dlopen` of Rust functions. <!-- rust-interop-rejected -->
- Compatibility shims for earlier draft interop syntax.
- Silent copy fallback for zero-copy APIs.
- Hidden Tokio runtimes, generated `block_on`, or implicit offload.

## Depends on

- Phase 38.
- Existing Cargo-backed package graph and generated project materialization.
- Existing Sifr async/offload diagnostics and Tokio runtime substrate.
- Existing Phase 27 safety and diagnostic non-regression contracts.

This phase is otherwise self-contained for the Rust interop contract it claims
as supported: declaration syntax, structured metadata, Cargo build-plan
integration, diagnostics, trust policy, tooling, documentation, and the
machine-checkable compatibility matrix. Runtime or ecosystem surfaces that do
not have passing positive and negative evidence must not be claimed as Phase 39
support. They must be categorized as `future-owned-by-separate-phase` and
linked to an active follow-up issue or phase before closeout.

Phase 39 has internal implementation checkpoints, not separate release phases
for the supported core contract. Core Rust interop must land before any stable
release can claim Rust-backed package support. Advanced runtime/ecosystem
surfaces that remain future-owned are tracked by
[`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../issues/active/rust-interop-runtime-ecosystem-certification.md)
and cannot be advertised as supported by Phase 40 stable promotion while they
remain future-owned.

## Milestones

### milestone_39_0: Architecture Lock and Verification Scaffold

- Scope:
  - Land the Rust interop architecture as the normative design.
  - Create `verification/areas/rust_interop`.
  - Add fixture matrix, tier definitions, README, and runner skeletons.
  - Add diagnostic family inventory and documentation placeholders for `SIFR-RUST-*`.
  - Search active docs for stale Rust interop drafts and update or remove `extern rust`, `from rust import`, `native = [`, panic examples without `RustPanicError` or explicit panic policy, `@rust(crate=..., path=...)`, and Python code fences for Sifr interop examples. <!-- rust-interop-rejected -->
- Definition of done:
  - The verification area exists and names every fixture required by the design.
  - The architecture document and phase file agree on supported capabilities and rejected designs.
  - No old `extern rust`, Rust `dlopen`, or fallback syntax remains in active planning docs. <!-- rust-interop-rejected -->

### milestone_39_1: Declaration Syntax, Lowering, and Build Plan Metadata

Status: implemented in [PR #2702](https://github.com/sifr-lang/sifr/pull/2702); local `create-pr` validation passed and reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-1-review-round3.md`.

- Scope:
  - Parse and validate `@rust(...)`, `@rust.opaque(...)`, `@rust.async(...)`, `@rust.zero_copy(...)`, and `@rust.view(...)`.
  - Represent Rust decorator targets as structured dotted-path AST nodes, not strings.
  - Implement the fixed Rust interop decorator value grammar for booleans, identifier symbols, integers, bounded/custom policy calls, and structured Rust target paths.
  - Lower Rust interop declarations into HIR with source spans, effect classifications, and ABI requirements.
  - Extend codegen outputs with `InteropBuildPlan { rust: RustInteropPlan }`.
  - Reject malformed decorators, string targets, unsupported roots, and prior draft syntax with stable diagnostics.
- Definition of done:
  - Codegen dependency metadata comes from interop build-plan structures, not emitted Rust scanning.
  - Every parsed Rust interop declaration is visible to check/build/run paths before Cargo execution.
  - Negative fixtures prove invalid syntax cannot silently compile.

### milestone_39_2: Cargo Resolution, Trust, Cache, and Probe Infrastructure

Status: implemented in [PR #2703](https://github.com/sifr-lang/sifr/pull/2703); local `create-pr` validation passed and reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-2-review-round4.md`.

- Scope:
  - Resolve direct Cargo dependency roots, same-workspace crates, shared bridge crates, and package-local bridge roots.
  - Preserve Cargo as the source of truth for package IDs, source IDs, features, target triples, lockfiles, `--locked`, `--offline`, and `--frozen`.
  - Add trust gates for `rust-build-scripts`, `rust-proc-macros`, `native-links`, `unsafe-rust-bridges`, `build-env`, `rust-no-panic`, and `rust-panic-abort`.
  - Resolve `rust-no-panic` and `rust-panic-abort` entries through canonical Sifr dotted target paths, not lowered Rust `::` paths.
  - Split trust validation into pre-execution evidence that rejects known build scripts/proc macros before Cargo execution and post-execution evidence for trusted build-script link output before final artifact acceptance.
  - Include Rust interop requirements, bridge source digests, Cargo metadata, selected Cargo profile, resolved panic strategy, codegen-affecting profile settings, trust policy, target triple, target features, rustc/Cargo versions, bridge-version schema, and declared build environment values in cache keys.
  - Generate `RustBridgeProbePlan` metadata and isolated probe modules that run rustc item-existence probes for direct Cargo dependency targets; record async, receiver-mode, opaque, Send, and Sync obligations in the plan for milestone_39_5 signature assertions.
- Definition of done:
  - Cargo metadata failures become Sifr diagnostics with spans and remediation.
  - Known untrusted build scripts and proc macros are rejected before Cargo execution; native link evidence emitted by trusted build scripts is validated before final artifact acceptance.
  - Cache invalidation changes when any bridge declaration, bridge source, Cargo lock state, target, feature, or trust input changes.
  - Direct Cargo item-existence probe failures map rustc diagnostics to `SIFR-RUST-RESOLVE-*` or `SIFR-RUST-TYPE-*` diagnostics at the original decorator span.
- Carry-forward note:
  - Mixed Rust and Python interop fixtures remain outside M39.2; the first mixed fixture must either declare Python runtime native link evidence in Rust interop trust or add a Python-runtime trust source before native-link validation is considered complete for that combined build shape.

### milestone_39_3: Package-Local and Shared Bridge Modules

Status: implemented in [PR #2704](https://github.com/sifr-lang/sifr/pull/2704); focused validation passed and reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-3-review-round3.md`.

- Scope:
  - Generate and maintain Sifr-owned projection entries for `src/bridges/mod.rs`, Sifr-managed `src/lib.rs`, and generated `crate::__sifr_bridge`.
  - Use bridge-versioned deterministic Rust module-name mangling for generated `crate::__sifr_bridge::<sifr_module_path>` paths.
  - Discover user-authored `src/bridges/*.rs` files without overwriting them silently.
  - Support shared bridge crates as normal Cargo dependencies while enforcing that shared bridge crates cannot import package-specific generated bridge types.
  - Validate bridge module target paths, exported functions, managed projection conflicts, and package archive contents.
  - Add fixtures for local bridge, shared bridge crate, generated projection ownership, bridge-version mismatch, plain cargo-check limitations, package archive validation, and bridge module conflict behavior.
- Definition of done:
  - User bridge files remain owned by users.
  - Generated glue can call package-local bridge functions deterministically.
  - Bridge projection conflicts fail with stable diagnostics instead of clobbering user files.
  - Shared bridge crates expose only stable Rust/runtime interop types or their own opaque types.

### milestone_39_4: Bridge Type Contract and Conversion Runtime

Status: implemented in [PR #2705](https://github.com/sifr-lang/sifr/pull/2705); local `create-pr` validation passed and reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-4-review-round2.md`.

- Scope:
  - Implement checked bridge mappings for booleans, fixed-width integers, exact integers through `sifr_runtime::interop::SifrIntBridge`, floats, strings, bytes, lists, order-preserving dicts, `Option`, `Result`, closed enums, records, opaque handles through `sifr_runtime::interop::Handle<T>`, callbacks, and errors.
  - Generate explicit bridge types for records, closed enums, and errors under `crate::__sifr_bridge::<sifr_module_path>::<Name>Bridge`.
  - Reject source-level exact `int` where a fixed-width or explicit exact representation is required.
  - Add conversion diagnostics for width, overflow, invalid UTF-8, unsupported container shapes, record layout mismatches, invalid enum discriminants, and unsupported containers such as `set`/`tuple`.
- Definition of done:
  - Supported type mappings roundtrip through Rust bridge calls; full runtime value-roundtrip certification for `bridge_type_matrix` is future-owned by [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../issues/active/rust-interop-runtime-ecosystem-certification.md) and is not claimed as Phase 39 support until both evidence directions pass.
  - Unsupported mappings fail before final generated binary build.
  - Every conversion failure has a stable Sifr error surface.

### milestone_39_5: Direct Cargo Crate Bindings

Status: implemented in [PR #2707](https://github.com/sifr-lang/sifr/pull/2707); `scripts/run_all_tests.sh --profile create-pr` passed with only the warm wall-time advisory, and reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-5-review-round1.md`.

- Scope:
  - Implement direct binding for Cargo dependency functions whose public Rust signatures are bridge-compatible under milestone_39_4.
  - Extend milestone_39_2 item-existence probes to validate visibility, arity, parameter types, return types, receiver mode, asyncness, panic policy, Send/Sync obligations, and `Result`/`Option` shape.
  - Reject arbitrary lifetimes, borrowed returns, raw pointers, trait objects, unconstrained generics, closures, `unsafe fn`, and unsupported `Pin`/self-referential surfaces.
  - Add direct binding fixtures for simple crates such as `crc32fast`, `blake3`, `sha2`, `uuid`, and `regex`, including compatible direct signatures, incompatible signatures, reserved-root conflicts, no-panic trust, and probe diagnostic mapping.
- Definition of done:
  - Direct crate binding works without package-local bridge code for compatible signatures.
  - Incompatible third-party APIs require an explicit bridge and produce actionable diagnostics.
  - Same-workspace crates work only when declared as ordinary Cargo dependencies.

### milestone_39_6: Opaque Rust Handles and Resource Cleanup

Status: implemented in [PR #2709](https://github.com/sifr-lang/sifr/pull/2709); focused driver/runtime validation covers opaque contract parsing, Send/Sync/Copy probes, explicit close/aclose contract diagnostics, and runtime closed/poisoned handle transitions. Full ecosystem resource wrappers are future-owned by [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../issues/active/rust-interop-runtime-ecosystem-certification.md) through the `opaque_resource_matrix` compatibility row.

- Scope:
  - Implement `@rust.opaque(...)` classes with ownership, borrowing, clone, close, `Send`, `Sync`, and thread-affinity metadata.
  - Generate handle wrappers that prevent use-after-close and double-close, enforce closed/poisoned state transitions, use private generated-glue tokens for state mutation, use `PoisonOnPanic` guards for owned handles, and expose only generated-safe accessors.
  - Require either safe `Drop` cleanup or explicit `close`/`aclose` contracts for owning handles.
  - Add diagnostics for leaking explicitly-closed handles where ownership analysis can prove the leak.
  - Cover sync close, async close, borrowed handle, exclusive handle, clone, and non-clone paths with resource-shaped crates such as `reqwest`, `rusqlite`, `tokio-postgres`, and `redis`.
- Definition of done:
  - Opaque handles preserve Sifr ownership rules at the Rust boundary.
  - Use-after-close and invalid aliasing produce stable errors.
  - Handle cleanup does not rely on fallible `Drop` behavior.

### milestone_39_7: Async, Blocking, and Tokio Integration

Status: implemented in [PR #2711](https://github.com/sifr-lang/sifr/pull/2711); local `create-pr` validation passed and reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-7-review-round3.md`. Focused validation covers `@rust.async(...)` lowering on `async def`, rejection on sync declarations, default `Send` future probe obligations, `thread_affinity=tokio_current_thread` non-`Send` opt-out, invalid async affinity diagnostics, and rejection of blocking/CPU-heavy classifications on async Rust declarations. The follow-on `async_runtime_reqwest` certification executes borrowed-input reqwest futures on the generated current-thread runtime, observes runtime reuse and cancellation cleanup against an ephemeral loopback server, and rejects package-local nested runtime construction or `block_on`.

- Scope:
  - Support async Rust bridge functions using Sifr's existing Tokio runtime model.
  - Reject hidden runtime creation, generated `block_on`, and assumptions that `rt-multi-thread` is available.
  - Enforce explicit `@blocking_io` and `@cpu_heavy` annotations for blocking or CPU-heavy Rust calls.
  - Reject `@blocking_io` and `@cpu_heavy` on `async def` Rust interop declarations.
  - Certify borrowed inputs inside generated async wrapper futures through the runtime-observed `async_runtime_reqwest` follow-on row; the original M39.7 compile-time contract alone does not carry that runtime claim.
  - Require explicit Sifr offload APIs when classified calls are used from async Sifr code.
  - Allow non-`Send` futures only when explicitly pinned to the current Sifr Tokio runtime through `thread_affinity=tokio_current_thread`; reject non-`Send` futures that may leave that runtime.
  - Map cancellation and shutdown behavior to stable Sifr errors.
  - Cover async ecosystem shapes with `tokio`, `futures`, `reqwest`, `tower`, `http`, and `http-body`.
- Definition of done:
  - Async Rust interop composes with current-thread Tokio entrypoints.
  - Blocking and CPU-heavy calls cannot accidentally run on async scheduler paths.
  - Negative fixtures prove hidden runtime and hidden blocking designs are rejected.

### milestone_39_8: Panic Boundary and Rust Error Surface

Status: merged in [PR #2717](https://github.com/sifr-lang/sifr/pull/2717); focused validation covers built-in `RustPanicError` signatures, explicit `RustPanicError`/`panic=map_error(...)` Result panic surfaces, rejection of missing or invalid panic policies, rejection of `panic=map_error(...)` on non-Result APIs, abort policy trust gates, abort-profile strategy validation, and redacted `RustPanicErrorBridge` panic payload conversion. The follow-on [`certification_2` work](../issues/active/rust-interop-runtime-ecosystem-certification.md#certification_2-panic-boundary-wrapper-emission) completes synchronous generated wrapper execution for package-local calls: target and mapper panics are contained and redacted, ordinary Rust `Err` values remain distinct, mapper signatures are Cargo-probed, mapper-panic fallback is runtime-observed, and async mapper policies remain rejected pending their own wrapper certification.

- Scope:
  - Wrap Rust bridge calls in unwind boundaries where recoverable.
  - Convert Rust panics into `RustPanicError` without exposing Rust panic payload details unsafely.
  - Validate `panic=map_error(...)` adapter signatures and mapper-panic fallback behavior.
  - Reject `panic = "abort"` for recoverable bridge builds unless explicitly opted into through `[trust].rust-panic-abort` and documented.
  - Preserve Sifr user error semantics for Rust `Result` values.
  - Add diagnostics for panic strategy mismatch, unreachable panic containment, and poisoned opaque handles after caught panics.
- Definition of done:
  - Panicking Rust bridge functions cannot panic through Sifr user code in recoverable builds.
  - Abort-profile behavior is explicit and covered by negative validation.
  - Rust user errors and Rust panics remain distinguishable.

Follow-on closure evidence is owned by the
`panic_boundary_wrapper_emission` compatibility row and its dedicated
`certification_2` PR. It does not promote the separately future-owned async,
callback, opaque-handle, or ecosystem runtime rows.

### milestone_39_9: Zero-Copy and Core Views

Status: merged in [PR #2720](https://github.com/sifr-lang/sifr/pull/2720); focused validation covers explicit `@rust.zero_copy(...)` and `@rust.view(...)` contract shapes, required paired view contracts for zero-copy returns, owner/lifetime/mutability checks, rejection of `lifetime=call` returned views, rejection of mutable views from shared-borrow owners, rejection of copy-fallback declarations, async borrowed-view suspension rejection, and view Send/Sync probe metadata. Runtime-observed crate-backed certification for `bytes`, `memmap2`, `bytemuck`, and `zerocopy` is future-owned by [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../issues/active/rust-interop-runtime-ecosystem-certification.md) while contract-only zero-copy rows remain supported, and reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-9-review-round3.md`.

- Scope:
  - Implement explicit `@rust.zero_copy(...)` and `@rust.view(...)` contracts.
  - Require borrowed zero-copy returns to declare both no-copy behavior and view lifetime/thread policy.
  - Enforce owner/view lifetime rules, returned-view lifetime restrictions, aliasing, mutable exclusivity, Send/Sync declarations, and async suspension restrictions.
  - Support zero-copy bytes views.
  - Provide separate copy APIs for copy behavior; never silently copy for a zero-copy declaration.
  - Add positive and negative fixtures for borrowed views, mutable exclusivity, owner lifetime, copy-fallback rejection, and real view-backed crates such as `bytes`, `memmap2`, `bytemuck`, and `zerocopy`.
- Definition of done:
  - Zero-copy fixtures include positive and negative ownership/lifetime cases.
  - Copy fallback attempts are rejected with `SIFR-RUST-ZC-*` diagnostics.

### milestone_39_10: Advanced Data Bridges

Status: merged in [PR #2722](https://github.com/sifr-lang/sifr/pull/2722); focused validation covers contract-only `@rust.view(...)` advanced data metadata for Arrow/dataframe and tensor/DLPack views, required Arrow schema identity, tensor dtype/rank/shape/layout/strides/device metadata, CPU-only device constraints, explicit DLPack transfer ownership with owned owner parameters, and matching shared bridge crate roots. Runtime-observed crate-backed certification for `arrow`, `datafusion`, `polars`, `ndarray`, and `candle` is future-owned by [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../issues/active/rust-interop-runtime-ecosystem-certification.md) and is not claimed as Phase 39 support until both evidence directions pass.

- Scope:
  - Add Arrow-compatible record batch/array bridge contracts through shared bridge crates.
  - Add tensor buffer contracts with dtype, shape, layout, strides, device, and ownership metadata.
  - Support DLPack-style tensor handoff through shared bridge crates where the ownership contract is explicit.
  - Certify advanced data fixtures with `arrow`, `datafusion`, `polars`, `ndarray`, and `candle`.
- Definition of done:
  - Arrow and tensor bridge fixtures validate metadata, ownership, and dtype behavior.
  - Shared bridge crates for Arrow/tensor work do not import package-specific generated bridge types.

### milestone_39_11: Callback Contracts

Status: contract-only callback declaration, validation, and bridge planning support merged in [PR #2724](https://github.com/sifr-lang/sifr/pull/2724). Focused validation covers `@rust.callback(...)` lowering, required thread-safe callback `backpressure`/`overflow`/`shutdown` policy, and rejection of missing/invalid/duplicate callback contracts with `SIFR-RUST-CB-0001`. The follow-on `certification_3` work promotes plain top-level `Callable` parameters as call-scoped, runtime-observed bridges: generated adapters are borrowed and non-thread-safe, callback panics use the redacted outer panic boundary, and Cargo probing rejects storage or thread escape. Subscription handle cleanup, cross-thread capture enforcement for registered callbacks, and `tokio-tungstenite`/`redis`/`notify` ecosystem certification remain future-owned by [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../issues/active/rust-interop-runtime-ecosystem-certification.md) through `callback_subscription_ecosystem`.

- Scope:
  - Implement call-scoped callbacks that cannot be stored, called after return, or called from unmanaged threads.
  - Implement thread-safe callback registration with cancellation/subscription handles and required `@rust.callback(...)` backpressure/overflow/shutdown policy.
  - Enforce Sifr task-spawn/offload ownership requirements for callbacks that may cross threads: no borrowed stack-local values, no non-send opaque handles, and no current-thread/current-OS-thread-affine captures.
  - Require explicit backpressure, cancellation, and shutdown policy for async or thread-safe callbacks.
  - Add panic-to-error handling around callback invocation.
  - Exercise thread-safe callback and subscription behavior with crates such as `tokio-tungstenite`, `redis` pub/sub, and `notify`.
- Definition of done:
  - Callback fixtures cover call-scoped and thread-safe callback behavior.
  - Invalid callback storage, threading, capture, and backpressure declarations fail at check/build time.
  - Registered callbacks clean up deterministically during shutdown.

### milestone_39_12: Tooling, Diagnostics, and Documentation

Status: merged in [PR #2726](https://github.com/sifr-lang/sifr/pull/2726); focused validation covers `sifr bridge check` routing through package check, decorator-specific Rust interop completion candidates through analysis/LSP, Rust diagnostic family documentation, package-author workflows, user-facing examples, and rejected-design docs. Reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-12-review-round2.md`. The `create-pr` facade passed through the previously failing audit suites on a warmed rerun but the final full-facade attempt was interrupted by the execution session before a final lane report; selected relevant areas were rerun directly and passed.

- Scope:
  - Add LSP completion and validation for Rust decorator dotted paths.
  - Add diagnostics documentation for `SIFR-RUST-CONFIG-*`, `SIFR-RUST-RESOLVE-*`, `SIFR-RUST-TRUST-*`, `SIFR-RUST-TYPE-*`, `SIFR-RUST-HANDLE-*`, `SIFR-RUST-ASYNC-*`, `SIFR-RUST-ZC-*`, `SIFR-RUST-CB-*`, `SIFR-RUST-PANIC-*`, and `SIFR-RUST-CARGO-*`.
  - Document package-author workflows for direct bindings, local bridges, shared bridge crates, opaque handles, async, zero-copy, callbacks, and trust policy.
  - Document `sifr bridge check`, `sifr repair --check`, and `sifr repair` workflows for managed projections and local bridge authoring.
  - Document user-facing examples for `crc32fast`, `blake3`, tokenizer handles, async HTTP, Arrow, tensor/DLPack, and callback registration.
- Definition of done:
  - Tooling surfaces the same target resolution and diagnostics as the compiler.
  - Public and internal docs are aligned with the architecture document.
  - Invalid examples are documented as rejected designs, not alternate forms.

### milestone_39_13: Ecosystem Certification and Closeout

Status: merged in [PR #2728](https://github.com/sifr-lang/sifr/pull/2728); local `create-pr` validation passed with only the warm wall-time advisory, focused post-review validation passed for taxonomy, Rust interop matrix checks, and completion-quality helpers, and reviewer sign-off is recorded in `plans/reviews/active/rust-interop-milestone39-13-review-round5.md`. The verification-matrix hardening follow-up completed through [PR #3018](https://github.com/sifr-lang/sifr/pull/3018), [#3019](https://github.com/sifr-lang/sifr/pull/3019), [#3020](https://github.com/sifr-lang/sifr/pull/3020), [#3022](https://github.com/sifr-lang/sifr/pull/3022), and [#3023](https://github.com/sifr-lang/sifr/pull/3023); its final review and validation are recorded in the [archived closeout](../issues/archive/rust-interop-verification-matrix-hardening.md).

- Scope:
  - Certify representative packages across direct binding, local bridge, shared bridge, opaque handle, zero-copy, async, callbacks, build script, proc macro, native link, and locked/offline Cargo behavior.
  - Publish a Rust interop compatibility matrix with `supported`, `supported-through-bridge`, `unsupported-by-design`, and `future-owned-by-separate-phase` categories.
  - Include required fixture evidence for direct binding, bridge types, async, resource handles, blocking/CPU-heavy calls, zero-copy views, Arrow/dataframe exchange, tensor exchange, callbacks, proc macros, build scripts, native links, and locked/offline Cargo behavior.
  - Run production-grade review rounds until no `SIFR-RUST-*` diagnostic family, verification tier, bridge-type contract, runtime safety rule, or package/build-plan contract has an open specification gap.
- Definition of done:
  - Every supported design capability has a passing positive fixture and a deliberate negative fixture.
  - The compatibility matrix matches actual verification evidence.
  - Phase closeout leaves no undocumented Rust interop gaps: incomplete fixture
    families must have positive and negative evidence placeholders, a
    `future-owned-by-separate-phase` compatibility row, and a concrete active
    issue or phase owner.

## Verification Area

Phase 39 must create and maintain the exact `verification/areas/rust_interop` tree listed in [`internal_docs/rust_interop_architecture.md`](../../internal_docs/rust_interop_architecture.md#verification-area). The architecture document is the normative source for fixture names, runner names, and area layout.

Verification tiers:

- Tier 0: parser, lowering, metadata, and diagnostics without Cargo build.
- Tier 1: direct crate and local bridge build fixtures.
- Tier 2: opaque handles, panic boundary, async/blocking, callbacks, and zero-copy.
- Tier 3: build scripts, proc macros, native linking, and locked/offline Cargo behavior.
- Tier 4: production examples and compatibility matrix.

### Crate Verification Matrix

Phase 39 verifies Rust interop behavior with representative real crates. This matrix is fixture guidance, not a promise that Sifr ships first-party package wrappers for every crate listed. A crate can count only when the fixture records the exact interop shape it exercises, the positive/negative evidence, the Cargo feature set, target triple, lock state, and any trust policy required.

Required core fixtures:

| Capability | Required crates | Verification purpose |
| --- | --- | --- |
| Direct compatible functions | `crc32fast`, `blake3`, `sha2`, `uuid`, `regex` | Dotted-path direct binding, bridge-compatible signatures, panic policy, incompatible signature rejection, and probe diagnostic mapping. |
| Bridge type generation and conversion | `serde`, `serde_json`, `thiserror`, `bytes`, `indexmap` | Generated records/enums/errors, ordered dictionaries, owned bytes, explicit error types, and conversion diagnostics. |
| Build and proc-macro trust | `serde_derive`, `prost-build` | Pre-execution proc-macro/build-script trust rejection, trusted execution evidence, and cache-key sensitivity. |
| Native/build links | `cc`, `bindgen`, `cxx`, `zstd` | Trusted build-script link output, native-link evidence, unsafe bridge policy, and post-execution artifact acceptance. |
| Async/Tokio ecosystem | `tokio`, `futures`, `reqwest`, `tower`, `http`, `http-body` | Async function probing, future output conversion, Send/non-Send diagnostics, cancellation, and service-shaped public types on Sifr's Tokio runtime. |
| Opaque resources | `reqwest::Client`, `rusqlite`, `tokio-postgres`, `redis` | Owned handles, borrowed/exclusive receivers, close/aclose contracts, Send/Sync policy, poisoned handles, and resource cleanup. |
| Blocking and CPU-heavy calls | `rusqlite`, `rayon`, `flate2` | `@blocking_io`, `@cpu_heavy`, explicit offload requirements, and rejection of accidental async-scheduler blocking. |
| Zero-copy core views | `bytes`, `memmap2`, `bytemuck`, `zerocopy` | Owner/view lifetimes, mutable exclusivity, no-copy guarantees, static/call/owner lifetimes, and copy-fallback rejection. |

Required advanced fixtures:

| Capability | Required crates | Verification purpose |
| --- | --- | --- |
| Arrow and dataframe exchange | `arrow`, `datafusion`, `polars` | Arrow schema identity, record batches, columnar ownership metadata, shared bridge crate limits, and dataframe boundary behavior. |
| Tensor and array exchange | `ndarray`, `candle` | Dtype, shape, layout, strides, device metadata, ownership transfer, and DLPack-style bridge contracts. |
| Thread-safe callbacks and subscriptions | `tokio-tungstenite`, `redis` pub/sub, `notify` | Callback registration, cancellation handles, backpressure, overflow policy, shutdown, thread-safety, and callback panic mapping. |

Ecosystem certification fixtures:

| Area | Representative crates | Purpose |
| --- | --- | --- |
| Backend/service certification | `axum`, `tower-http`, `sqlx` | Prove real Rust-backed service packages can compile and probe through the canonical package model without adding web-framework-specific rules. |
| CLI/tooling certification | `clap`, `tracing`, `tracing-subscriber`, `anyhow` | Prove common package-author dependencies work with generated bridge crates, diagnostics, and runtime integration. |

CLI/tooling certification is implemented as exact-pinned
`supported-through-bridge` evidence: the package executes clap parsing and a
filtered tracing event, maps internal `anyhow::Error` context into a declared
bridge error, and rejects a direct `anyhow::Error` return with
`SIFR-RUST-TYPE-0001`.

Backend/service certification is implemented as exact-pinned
`supported-through-bridge` evidence. The package executes an Axum server on
`127.0.0.1:0`, observes a tower-http response header, expands a SQLx query
macro from checked-in metadata, and completes deterministic shutdown. Sifr
forces SQLx offline compilation and removes `DATABASE_URL`; missing and stale
metadata are rejected as `SIFR-RUST-CARGO-0001` before the armed database
sentinel can observe a connection. This remains package-model certification,
not product-level web framework support.

Pinned fixture feature policy:

- `reqwest`: `default-features = false`, `features = ["rustls-tls", "json"]`; do not enable `blocking` in async fixtures.
- `tokio-postgres`: `default-features = false`, `features = ["runtime"]`; TLS is not part of the primary opaque-resource fixture.
- `rusqlite`: `features = ["bundled"]`; the unbundled system-sqlite variant is intentionally not certified in Phase 39.
- `redis`: `default-features = false`, `features = ["tokio-comp"]`; pub/sub fixtures use loopback service infrastructure.
- `tokio-tungstenite`: `default-features = false`; add `features = ["rustls-tls-webpki-roots"]` only for explicit network/TLS coverage.
- `sqlx`: `default-features = false`, `features = ["runtime-tokio-rustls", "postgres", "macros"]`; this is ecosystem certification, not the primary opaque-resource fixture, and query-macro fixtures must use checked-in `.sqlx/` offline artifacts instead of requiring `DATABASE_URL` during Cargo execution.
- `axum`: the backend certification uses only `http1` and `tokio`.
- `tower-http`: the backend certification disables defaults and uses only
  `set-header` so middleware execution is directly observable.
- `tracing-subscriber`: include `env-filter` for the CLI/tooling certification fixture.
- `flate2`: `default-features = false`, `features = ["rust_backend"]`.
- `candle`: CPU-only default backend; GPU and accelerator backend features are out of scope for Phase 39.
- `prost-build`: use default features and a checked-in `.proto` fixture; generated output must be deterministic.

Fixture execution policy:

- Compile/probe-only fixtures are valid for syntax, lowering, Cargo metadata, signature, feature, trust, and diagnostics coverage.
- Resource-behavior fixtures must use loopback services, local filesystem inputs, or explicit local service configuration so close/aclose, cancellation, subscription, and shutdown behavior is actually observed.
- `reqwest`, `tokio-tungstenite`, and `notify` should prefer loopback or local filesystem inputs.
- `tokio-postgres` and `redis` require explicit local service configuration when runtime behavior is under test; a fixture can be tier-gated, but it cannot silently degrade into compile-only coverage while claiming resource behavior evidence.

Out of scope for required Phase 39 verification:

- Game, GUI, desktop, and rendering crates such as `bevy`, `wgpu`, `egui`, `tauri`, and `iced`.
- Embedded and `no_std` ecosystems such as `embedded-hal`, `embassy`, and `defmt`.
- Full product-level web framework support. Phase 39 may certify `axum`/`tower-http` package compilation and probing, but web framework product workflows belong to separate Sifr web work.
- Creating Sifr standard wrappers for every crate above. Phase 39 proves the interop contract and package model; wrapper packages can be authored independently after the contract is implemented.

## Quality Contract

- Entry criteria: Phase 38 is completed and docs/planning for the Rust interop architecture are canonical.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract; canonical/lossless JSON diagnostics with human and compact renderers as views only; deterministic recovery; and enforced exit-code and CLI stability contracts.
- Existing async/offload diagnostics remain authoritative. Rust interop adds classifications to that model instead of creating a parallel runtime model.
- Cargo remains authoritative for Rust dependency resolution. Sifr must not implement a second Rust resolver.
- No fallback, migration, or legacy compatibility code is allowed. Implement the canonical architecture directly.
- No silent behavior downgrades are allowed. A zero-copy declaration cannot copy, an async declaration cannot block, and a direct binding cannot use an adapter that was not declared.
- All implementations must be production-grade compiler/runtime/package code with explicit invariants, deterministic behavior, and stable diagnostics.
- Every milestone must include positive and negative verification mapped to the milestone validation goals.
- Validation evidence must be recorded in the phase execution checklist issue before merge.

## Validation Planning Goals

- `milestone_39_0`: architecture, verification area, fixture matrix, tiers, diagnostic family inventory, and stale interop-design removal.
- `milestone_39_1`: decorator parsing/lowering, decorator value grammar, structured target metadata, HIR representation, build-plan output, and invalid syntax diagnostics.
- `milestone_39_2`: Cargo metadata, trust gates, canonical Sifr dotted trust target paths, pre-execution and post-execution trust evidence, signature probe infrastructure, lock/offline/frozen behavior, profile and panic-strategy inputs, cache invalidation, build-script/proc-macro/native-link evidence with crates such as `serde_derive`, `prost-build`, `cc`, `bindgen`, `cxx`, and `zstd`.
- `milestone_39_3`: package-local bridge generation, shared bridge crates, projection ownership, `src/lib.rs`/`src/bridges/mod.rs` management, `crate::__sifr_bridge` reservation, deterministic bridge module path mangling, bridge-version mismatch rejection, package archive validation, projection conflicts, and same-workspace dependency behavior.
- `milestone_39_4`: supported bridge type roundtrips, generated bridge type naming, order-preserving dicts, exact-integer bridges, opaque `Handle<T>` representation, closed enum representation, unsupported containers, and unsupported bridge type diagnostics.
- `milestone_39_5`: direct binding success with `crc32fast`, `blake3`, `sha2`, `uuid`, and `regex`; direct binding rejection for unsupported Rust signatures; probe diagnostic mapping; reserved-root conflict behavior; and no-panic trust behavior.
- `milestone_39_6`: opaque handles, close/aclose, clone policy, Send/Sync policy, state transitions, use-after-close, double-close, poisoned-handle behavior, leak diagnostics, and resource-shaped crates such as `reqwest`, `rusqlite`, `tokio-postgres`, and `redis`.
- `milestone_39_7`: async Rust functions, borrowed-input wrapper futures, blocking/CPU-heavy classification, rejection of async blocking/CPU-heavy decorator conflicts, explicit offload, Tokio current-thread compatibility, current-thread non-`Send` futures, invalid non-`Send` rejection, and async ecosystem fixtures with `tokio`, `futures`, `reqwest`, `tower`, `http`, and `http-body`.
- `milestone_39_8`: panic containment, Rust user errors, panic strategy rejection, poisoned handle behavior, and abort opt-in evidence.
- `milestone_39_9`: zero-copy bytes, required combined zero-copy/view contracts for borrowed returns, core view contracts, owner/view lifetime rejection, mutable exclusivity, copy-fallback rejection, and view-backed crates such as `bytes`, `memmap2`, `bytemuck`, and `zerocopy`.
- `milestone_39_10`: Arrow record batches, dataframe exchange, tensor/DLPack handoff, shared bridge crate data boundaries, metadata validation, schema identity, dtype behavior, and advanced fixtures with `arrow`, `datafusion`, `polars`, `ndarray`, and `candle`.
- `milestone_39_11`: call-scoped callbacks, thread-safe callbacks, cancellation handles, backpressure, shutdown, invalid capture/threading diagnostics, and subscription/event fixtures with `tokio-tungstenite`, `redis` pub/sub, and `notify`.
- `milestone_39_12`: LSP completions, diagnostic documentation, package-author docs, `sifr bridge check`, `sifr repair --check`, `sifr repair`, user examples, and rejected-design docs.
- `milestone_39_13`: ecosystem compatibility matrix, fixture evidence, review closure, and phase closeout.
  Future-owned runtime/ecosystem certification rows are tracked by
  [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../issues/active/rust-interop-runtime-ecosystem-certification.md)
  and are not Phase 39 supported surfaces until both evidence directions pass.

## Exit Gate

Core gate:

- Rust-backed Sifr packages can expose package-local bridges, shared bridge crates, direct Cargo bindings, bridge-compatible records/enums/errors, opaque handles, async functions, blocking/CPU-heavy classifications, panic surfaces, and zero-copy core views under the canonical declaration model.
- Cargo resolution, Rust signature probing, trust policy, cache keys, managed projections, diagnostics, and verification tiers 0-2 are implemented with positive and negative fixtures.

Advanced gate:

- Rust-backed Sifr packages can expose the advanced data, callback, and
  zero-copy contracts that the compatibility matrix marks as `supported` or
  `supported-through-bridge`.
- Runtime/ecosystem certification rows that remain
  `future-owned-by-separate-phase` are explicitly excluded from Phase 39 support
  claims and are blocked from stable-release advertising until their active
  follow-up issue lands passing evidence.
- Verification tiers 3-4 and the compatibility matrix are backed by local
  validation evidence for supported rows, and future-owned rows are backed by
  concrete follow-up ownership.

Full Phase 39 exit:

- Every Rust interop path lowers through structured metadata into generated Rust and Cargo build plans.
- Every unsafe, build-time, native, blocking, CPU-heavy, callback, panic, and zero-copy hazard has a stable trust/diagnostic/verification surface.
- The verification area contains positive and negative fixtures for every supported capability.
- The compatibility matrix is backed by local validation evidence for supported
  rows and active follow-up ownership for future-owned rows.
- Phase 27 non-regression remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic in user runtime paths, and stable diagnostics/renderer/exit-code behavior.
