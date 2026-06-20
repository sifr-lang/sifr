# Phase 39: Rust Interop

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
- `extern rust` syntax.
- Runtime `dlopen` of Rust functions.
- Compatibility shims for earlier draft interop syntax.
- Silent copy fallback for zero-copy APIs.
- Hidden Tokio runtimes, generated `block_on`, or implicit offload.

## Depends on

- Phase 38.
- Existing Cargo-backed package graph and generated project materialization.
- Existing Sifr async/offload diagnostics and Tokio runtime substrate.
- Existing Phase 27 safety and diagnostic non-regression contracts.

This phase is otherwise self-contained. Any compiler, runtime, package, or verification plumbing missing for Rust interop must be implemented inside Phase 39 rather than deferred to another interop phase.

## Milestones

### milestone_39_0: Architecture Lock and Verification Scaffold

- Scope:
  - Land the Rust interop architecture as the normative design.
  - Create `verification/areas/rust_interop`.
  - Add fixture matrix, tier definitions, README, and runner skeletons.
  - Add diagnostic family inventory and documentation placeholders for `SIFR-RUST-*`.
- Definition of done:
  - The verification area exists and names every fixture required by the design.
  - The architecture document and phase file agree on supported capabilities and rejected designs.
  - No old `extern rust`, Rust `dlopen`, or fallback syntax remains in active planning docs.

### milestone_39_1: Declaration Syntax, Lowering, and Build Plan Metadata

- Scope:
  - Parse and validate `@rust(...)`, `@rust.opaque(...)`, `@rust.async(...)`, `@rust.zero_copy(...)`, and `@rust.view(...)`.
  - Represent Rust decorator targets as structured dotted-path AST nodes, not strings.
  - Lower Rust interop declarations into HIR with source spans, effect classifications, and ABI requirements.
  - Extend codegen outputs with `InteropBuildPlan { rust: RustInteropPlan }`.
  - Reject malformed decorators, string targets, unsupported roots, and prior draft syntax with stable diagnostics.
- Definition of done:
  - Codegen dependency metadata comes from interop build-plan structures, not emitted Rust scanning.
  - Every parsed Rust interop declaration is visible to check/build/run paths before Cargo execution.
  - Negative fixtures prove invalid syntax cannot silently compile.

### milestone_39_2: Cargo Resolution, Trust, and Cache Integration

- Scope:
  - Resolve direct Cargo dependency roots, same-workspace crates, shared bridge crates, and package-local bridge roots.
  - Preserve Cargo as the source of truth for package IDs, source IDs, features, target triples, lockfiles, `--locked`, `--offline`, and `--frozen`.
  - Add trust gates for `rust-build-scripts`, `rust-proc-macros`, `native`, `unsafe-rust-bridges`, `build-env`, and `rust-panic-abort`.
  - Include Rust interop requirements, bridge source digests, Cargo metadata, selected Cargo profile, resolved panic strategy, codegen-affecting profile settings, trust policy, target triple, target features, rustc/Cargo versions, bridge-version schema, and declared build environment values in cache keys.
- Definition of done:
  - Cargo metadata failures become Sifr diagnostics with spans and remediation.
  - Trust failures are detected before executing untrusted build scripts or proc macros where possible.
  - Cache invalidation changes when any bridge declaration, bridge source, Cargo lock state, target, feature, or trust input changes.

### milestone_39_3: Direct Cargo Crate Bindings

- Scope:
  - Implement direct binding for Cargo dependency functions whose public Rust signatures are bridge-compatible.
  - Validate function existence, visibility, arity, parameter types, return types, and `Result`/`Option` shape.
  - Reject arbitrary lifetimes, borrowed returns, raw pointers, trait objects, unconstrained generics, closures, `unsafe fn`, and unsupported `Pin`/self-referential surfaces.
  - Add direct binding fixtures for simple crates such as `crc32fast`.
- Definition of done:
  - Direct crate binding works without package-local bridge code for compatible signatures.
  - Incompatible third-party APIs require an explicit bridge and produce actionable diagnostics.
  - Same-workspace crates work only when declared as ordinary Cargo dependencies.

### milestone_39_4: Package-Local and Shared Bridge Modules

- Scope:
  - Generate and maintain Sifr-owned projection entries for `src/bridges/mod.rs`.
  - Discover user-authored `src/bridges/*.rs` files without overwriting them silently.
  - Support shared bridge crates as normal Cargo dependencies.
  - Validate bridge module target paths and exported functions.
  - Add fixtures for local bridge, shared bridge crate, and bridge module conflict behavior.
- Definition of done:
  - User bridge files remain owned by users.
  - Generated glue can call package-local bridge functions deterministically.
  - Bridge projection conflicts fail with stable diagnostics instead of clobbering user files.

### milestone_39_5: Bridge Type Contract and Conversion Runtime

- Scope:
  - Implement checked bridge mappings for booleans, fixed-width integers, exact integers through `sifr_runtime::interop::SifrIntBridge`, floats, strings, bytes, lists, order-preserving dicts, `Option`, `Result`, closed enums, records, callbacks, and errors.
  - Generate explicit bridge types for records, closed enums, and errors under `crate::__sifr_bridge::<sifr_module_path>::<Name>Bridge`.
  - Reject source-level exact `int` where a fixed-width or explicit exact representation is required.
  - Add conversion diagnostics for width, overflow, invalid UTF-8, unsupported container shapes, and record layout mismatches.
- Definition of done:
  - Supported type mappings roundtrip through Rust bridge calls.
  - Unsupported mappings fail before generated Rust can reach an invalid Cargo build.
  - Every conversion failure has a stable Sifr error surface.

### milestone_39_6: Opaque Rust Handles and Resource Cleanup

- Scope:
  - Implement `@rust.opaque(...)` classes with ownership, borrowing, clone, close, `Send`, `Sync`, and thread-affinity metadata.
  - Generate handle wrappers that prevent use-after-close and double-close.
  - Require either safe `Drop` cleanup or explicit `close`/`aclose` contracts for owning handles.
  - Add diagnostics for leaking explicitly-closed handles where ownership analysis can prove the leak.
  - Cover sync close, async close, borrowed handle, exclusive handle, clone, and non-clone paths.
- Definition of done:
  - Opaque handles preserve Sifr ownership rules at the Rust boundary.
  - Use-after-close and invalid aliasing produce stable errors.
  - Handle cleanup does not rely on fallible `Drop` behavior.

### milestone_39_7: Async, Blocking, and Tokio Integration

- Scope:
  - Support async Rust bridge functions using Sifr's existing Tokio runtime model.
  - Reject hidden runtime creation, generated `block_on`, and assumptions that `rt-multi-thread` is available.
  - Enforce explicit `@blocking_io` and `@cpu_heavy` annotations for blocking or CPU-heavy Rust calls.
  - Require explicit Sifr offload APIs when classified calls are used from async Sifr code.
  - Allow non-`Send` futures only when explicitly pinned to the current Sifr Tokio runtime through `thread_affinity=tokio_current_thread`; reject non-`Send` futures that may leave that runtime.
  - Map cancellation and shutdown behavior to stable Sifr errors.
- Definition of done:
  - Async Rust interop composes with current-thread Tokio entrypoints.
  - Blocking and CPU-heavy calls cannot accidentally run on async scheduler paths.
  - Negative fixtures prove hidden runtime and hidden blocking designs are rejected.

### milestone_39_8: Panic Boundary and Rust Error Surface

- Scope:
  - Wrap Rust bridge calls in unwind boundaries where recoverable.
  - Convert Rust panics into `RustPanicError` without exposing Rust panic payload details unsafely.
  - Reject `panic = "abort"` for recoverable bridge builds unless explicitly opted into through `[trust].rust-panic-abort` and documented.
  - Preserve Sifr user error semantics for Rust `Result` values.
  - Add diagnostics for panic strategy mismatch, unreachable panic containment, and poisoned opaque handles after caught panics.
- Definition of done:
  - Panicking Rust bridge functions cannot panic through Sifr user code in recoverable builds.
  - Abort-profile behavior is explicit and covered by negative validation.
  - Rust user errors and Rust panics remain distinguishable.

### milestone_39_9: Zero-Copy, Views, Arrow, and Tensor Buffers

- Scope:
  - Implement explicit `@rust.zero_copy(...)` and `@rust.view(...)` contracts.
  - Enforce owner/view lifetime rules, aliasing, mutable exclusivity, Send/Sync declarations, and async suspension restrictions.
  - Support zero-copy bytes views.
  - Add Arrow-compatible record batch/array bridge contracts through shared bridge crates.
  - Add tensor buffer contracts with dtype, shape, layout, strides, device, and ownership metadata.
  - Support DLPack-style tensor handoff through shared bridge crates where the ownership contract is explicit.
  - Provide separate copy APIs for copy behavior; never silently copy for a zero-copy declaration.
- Definition of done:
  - Zero-copy fixtures include positive and negative ownership/lifetime cases.
  - Arrow and tensor bridge fixtures validate metadata, ownership, and dtype behavior.
  - Copy fallback attempts are rejected with `SIFR-RUST-ZC-*` diagnostics.

### milestone_39_10: Callback Contracts

- Scope:
  - Implement call-scoped callbacks that cannot be stored, called after return, or called from unmanaged threads.
  - Implement thread-safe callback registration with cancellation/subscription handles.
  - Enforce captured-value `Send + 'static` requirements for callbacks that may cross threads.
  - Require explicit backpressure, cancellation, and shutdown policy for async or thread-safe callbacks.
  - Add panic-to-error handling around callback invocation.
- Definition of done:
  - Callback fixtures cover call-scoped and thread-safe callback behavior.
  - Invalid callback storage, threading, capture, and backpressure declarations fail at check/build time.
  - Registered callbacks clean up deterministically during shutdown.

### milestone_39_11: Tooling, Diagnostics, and Documentation

- Scope:
  - Add LSP completion and validation for Rust decorator dotted paths.
  - Add diagnostics documentation for `SIFR-RUST-CONFIG-*`, `SIFR-RUST-RESOLVE-*`, `SIFR-RUST-TRUST-*`, `SIFR-RUST-TYPE-*`, `SIFR-RUST-HANDLE-*`, `SIFR-RUST-ASYNC-*`, `SIFR-RUST-ZC-*`, `SIFR-RUST-CB-*`, `SIFR-RUST-PANIC-*`, and `SIFR-RUST-CARGO-*`.
  - Document package-author workflows for direct bindings, local bridges, shared bridge crates, opaque handles, async, zero-copy, callbacks, and trust policy.
  - Document user-facing examples for `crc32fast`, `blake3`, tokenizer handles, async HTTP, Arrow, tensor/DLPack, and callback registration.
- Definition of done:
  - Tooling surfaces the same target resolution and diagnostics as the compiler.
  - Public and internal docs are aligned with the architecture document.
  - Invalid examples are documented as rejected designs, not alternate forms.

### milestone_39_12: Ecosystem Certification and Closeout

- Scope:
  - Certify representative packages across direct binding, local bridge, shared bridge, opaque handle, zero-copy, async, callbacks, build script, proc macro, native link, and locked/offline Cargo behavior.
  - Publish a Rust interop compatibility matrix with `supported`, `supported-through-bridge`, `unsupported-by-design`, and `future-owned-by-separate-phase` categories.
  - Run production-grade review rounds until no `SIFR-RUST-*` diagnostic family, verification tier, bridge-type contract, runtime safety rule, or package/build-plan contract has an open specification gap.
- Definition of done:
  - Every design capability has a passing positive fixture and a deliberate negative fixture.
  - The compatibility matrix matches actual verification evidence.
  - Phase closeout leaves no undocumented Rust interop gaps and no fixture family without both positive and negative evidence.

## Verification Area

Phase 39 must create and maintain the exact `verification/areas/rust_interop` tree listed in [`internal_docs/rust_interop_architecture.md`](../../internal_docs/rust_interop_architecture.md#verification-area). The architecture document is the normative source for fixture names, runner names, and area layout.

Verification tiers:

- Tier 0: parser, lowering, metadata, and diagnostics without Cargo build.
- Tier 1: direct crate and local bridge build fixtures.
- Tier 2: opaque handles, panic boundary, async/blocking, callbacks, and zero-copy.
- Tier 3: build scripts, proc macros, native linking, and locked/offline Cargo behavior.
- Tier 4: production examples and compatibility matrix.

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
- `milestone_39_1`: decorator parsing/lowering, structured target metadata, HIR representation, build-plan output, and invalid syntax diagnostics.
- `milestone_39_2`: Cargo metadata, trust gates, lock/offline/frozen behavior, profile and panic-strategy inputs, cache invalidation, build-script/proc-macro/native evidence.
- `milestone_39_3`: direct binding success and direct binding rejection for unsupported Rust signatures.
- `milestone_39_4`: package-local bridge generation, shared bridge crates, projection conflicts, and same-workspace dependency behavior.
- `milestone_39_5`: supported bridge type roundtrips, generated bridge type naming, order-preserving dicts, exact-integer bridges, and unsupported bridge type diagnostics.
- `milestone_39_6`: opaque handles, close/aclose, clone policy, Send/Sync policy, use-after-close, double-close, and leak diagnostics.
- `milestone_39_7`: async Rust functions, blocking/CPU-heavy classification, explicit offload, Tokio current-thread compatibility, current-thread non-`Send` futures, and invalid non-`Send` rejection.
- `milestone_39_8`: panic containment, Rust user errors, panic strategy rejection, poisoned handle behavior, and abort opt-in evidence.
- `milestone_39_9`: zero-copy bytes, Arrow record batches, tensor/DLPack handoff, owner/view lifetime rejection, mutable exclusivity, and copy-fallback rejection.
- `milestone_39_10`: call-scoped callbacks, thread-safe callbacks, cancellation handles, backpressure, shutdown, and invalid capture/threading diagnostics.
- `milestone_39_11`: LSP completions, diagnostic documentation, package-author docs, user examples, and rejected-design docs.
- `milestone_39_12`: ecosystem compatibility matrix, fixture evidence, review closure, and phase closeout.

## Exit Gate

- Rust-backed Sifr packages can expose direct Cargo bindings, package-local bridges, shared bridge crates, opaque handles, async functions, zero-copy views, tensors, Arrow-style data, and callbacks under the canonical declaration model.
- Every Rust interop path lowers through structured metadata into generated Rust and Cargo build plans.
- Every unsafe, build-time, native, blocking, CPU-heavy, callback, panic, and zero-copy hazard has a stable trust/diagnostic/verification surface.
- The verification area contains positive and negative fixtures for every supported capability.
- The compatibility matrix is backed by actual local validation evidence.
- Phase 27 non-regression remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic in user runtime paths, and stable diagnostics/renderer/exit-code behavior.
