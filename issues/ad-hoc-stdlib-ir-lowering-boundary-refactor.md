# Ad Hoc Phase: Stdlib, IR, and Lowering Boundary Refactor

Context: ad hoc architecture refactor before the next stdlib/platform work adds network, async subprocess, signal handling, IPC/process-pool, typed validation, web, and interop surfaces.

Status: planned

## Objective

Create stable compiler boundaries for three concerns that are currently coupled:

- the reusable IR data consumed by codegen, lint, analysis, and tooling,
- the lowering/type/ownership machinery that produces that IR,
- the stdlib host contract used by lowering, codegen dependency selection, and driver stdlib bootstrap.

This phase is complete when:

- `sifr_hir` has been replaced by `sifr_ir` plus `sifr_lowering`,
- `sifr_stdlib` owns the stdlib host contract and intrinsic signature registry,
- codegen, lint, and analysis consume IR data without depending on lowering internals,
- generated Cargo dependency decisions come from one stdlib/runtime feature manifest,
- no vague `core`, `utils`, or speculative domain runtime crates are introduced.

## Source Of Truth

This file is the implementation contract for the ad hoc boundary-refactor phase. Implementation PRs must not widen the crate tree beyond the crates named here unless a reviewed planning update changes this file first.

Execution status, validation evidence, review artifacts, and merged PR links are tracked in [ad-hoc-stdlib-ir-lowering-boundary-refactor-execution.md](./ad-hoc-stdlib-ir-lowering-boundary-refactor-execution.md).

## Depends On

- Current `sifr_source` crate as the bottom dependency for source text, line maps, text positions, and position encodings.
- Current `sifr_syntax` parser facade.
- Current `sifr_type_system` type model.
- Current Phase 32 async model and Phase 35/36 frontend/tooling boundaries.
- Current Phase 37 Cargo-backed package model.

## Non-Goals And Deferrals

- Do not create `sifr_core`, `sifr_utils`, or any similarly vague shared crate.
- Do not create `sifr_runtime_async`; async remains a language/compiler/runtime feature across existing layers.
- Do not create `sifr_model` or `sifr_runtime_validation` in this phase. Phase 40 starts with modules inside existing crates and may split later only if dependency pressure or consumer count proves it.
- Do not create `sifr_runtime_web`, `sifr_runtime_db`, `sifr_runtime_ml`, `sifr_test`, or `sifr_ffi_codegen`.
- Do not migrate codegen preamble strings into `sifr_runtime` in this phase. That is a later target-runtime consolidation phase. This phase may only move generated dependency selection into the stdlib feature manifest.
- Do not implement new stdlib APIs, async network APIs, `Popen`, signal handling, process pools, IPC, FFI, typed validation, web, SQL, OTEL, SIMD, or WASM behavior.
- Do not add compatibility shims that survive phase exit. Temporary mechanical aliases are allowed only inside an implementation PR and must be removed before that milestone merges.

## Locked Architecture Decisions

1. `sifr_source` remains the model for small coherent foundational crates: one named concept, one dependency direction, no dumping ground.
2. `sifr_ir` owns reusable IR data contracts, not lowering policy.
3. `sifr_lowering` owns AST-to-IR production, name resolution, type checking, ownership analysis, async analysis/lowering, mutable lowering state, and semantic diagnostics emitted during lowering.
4. `sifr_stdlib` owns the compiler-side stdlib contract. It is not the target runtime and does not compile generated Rust.
5. `sifr_stdlib` may depend on `sifr_type_system`, `sifr_diagnostics`, and `sifr_source` if needed, but it must not depend on `sifr_lowering`, `sifr_frontend`, `sifr_codegen`, `sifr_driver`, `sifr_package`, `sifr_analysis`, `sifr_lsp`, or the CLI.
6. `sifr_lowering`, `sifr_codegen`, `sifr_frontend`, and `sifr_driver` may depend on `sifr_stdlib`.
7. Generated dependency decisions live in one manifest owned by `sifr_stdlib`; codegen emits feature requirements and driver renders Cargo dependencies from the manifest.
8. Codegen-owned intrinsic Rust emission remains in `sifr_codegen`. `sifr_stdlib` owns the signature and dependency contract; `sifr_codegen` owns the Rust implementation of each intrinsic.
9. CFG and flow-graph public data types live in `sifr_ir`; CFG/flow-graph construction, mutation, and effect derivation live in `sifr_lowering`.
10. Mutable lowering scope state lives in `sifr_lowering`. Only immutable public snapshots that downstream consumers need may live in `sifr_ir`.
11. `sifr_codegen` must depend on `sifr_ir`, not `sifr_lowering`.
12. `sifr_lint` may depend on `sifr_ir` for HIR inspection, but must not directly depend on `sifr_lowering`. A transitive `sifr_lint -> sifr_frontend -> sifr_lowering` path is acceptable while lint still uses frontend queries to obtain project analysis.
13. `sifr_analysis` may depend on `sifr_ir` for editor/query views, but must not directly depend on `sifr_lowering`. A transitive `sifr_analysis -> sifr_frontend -> sifr_lowering` path is acceptable while analysis obtains project facts through frontend queries.
14. `sifr_frontend` remains the canonical facade for parsing/lowering/type-checking project inputs. CLI, driver, analysis, and LSP paths must not invent a second lowering path.
15. `sifr_driver` owns build/run/check orchestration and generated project materialization. It should obtain lowered user modules through `sifr_frontend`. The only allowed direct stdlib-bootstrap exception is the embedded-stdlib compilation path that parses and lowers `sifr_stdlib` source inventory before user modules are compiled; this exception must not expand to general user-module lowering from driver.

## Target Crate Tree

```text
sifr_source          # source text, line maps, text positions
sifr_diagnostics     # diagnostic model, codes, renderers, schemas
sifr_syntax          # parser/AST facade over the Sifr Ruff fork
sifr_type_system     # language type model

sifr_ir              # HIR nodes, public IR ids/views, CFG/flow graph data contracts
sifr_lowering        # AST -> IR, name/type/ownership/async lowering and diagnostics
sifr_stdlib          # host stdlib contract: sources, intrinsic signatures, feature deps

sifr_frontend        # canonical parse/lower/type-check/project query facade
sifr_codegen         # IR -> Rust and intrinsic Rust implementation emission
sifr_runtime         # target-side runtime library used by generated Rust
sifr_format
sifr_lint
sifr_analysis
sifr_lsp
sifr_package
sifr_driver
sifr                 # CLI
```

## `sifr_ir` Contract

`sifr_ir` owns data that can be consumed without knowing how lowering produced it:

- HIR node types and public IR ids.
- Lowered module/function/class/import views.
- Public lowering outcome/result data needed by frontend/codegen/analysis.
- CFG node/edge/view data and debug/fingerprint surfaces.
- Flow-graph node/edge/effect data and debug/fingerprint surfaces.
- Immutable narrowing/scope snapshots only when they are part of a public IR view.

`sifr_ir` must not own:

- Ruff AST lowering.
- Name resolution.
- Type inference or type checking.
- Ownership/move/borrow analysis.
- Async effect computation.
- CFG/flow-graph construction algorithms.
- Mutable scope state.
- Stdlib intrinsic signature lookup.
- Diagnostics emitted from lowering policy.

## `sifr_lowering` Contract

`sifr_lowering` owns all producer-side semantic work:

- AST-to-HIR lowering.
- Import resolution and local/external definition collection.
- Name resolution.
- Function/class/module lowering.
- Type inference and type checking.
- Ownership, borrow, move, narrowing, and flow diagnostics.
- Async lowering, task-boundary validation, async effect summaries, and offload diagnostics.
- CFG and flow-graph construction.
- Mutable scope/lowering state.
- Calls into `sifr_stdlib` for stdlib intrinsic signatures and stdlib-known workload classifications.

`sifr_lowering` must return `sifr_ir` data. It must not expose mutable lowering internals to codegen, lint, analysis, LSP, or the CLI.

## `sifr_stdlib` Contract

`sifr_stdlib` owns compiler-host knowledge about stdlib and intrinsics:

- Embedded `lib/sifr/*.sifr` source inventory and module names.
- Classification of embedded stdlib source versus user source for `_sifr.*` trust-boundary checks.
- Intrinsic module/member signatures currently represented under `crates/sifr_hir/src/stdlib`.
- Stdlib-known workload classifications used by async offload diagnostics.
- Stdlib module/member to feature requirements.
- Feature requirements to generated Cargo dependency specs.
- Stable feature ids used by codegen and driver.

`sifr_stdlib` must not own:

- Lowering implementation.
- Type checking implementation.
- Generated Rust implementation of intrinsics.
- Generated Rust project rendering.
- Rustc/Cargo process invocation.
- Target-side runtime behavior.

Expected public concepts:

```rust
pub struct StdlibSource {
    pub module: StdlibModuleName,
    pub source: &'static str,
}

pub struct IntrinsicSignature {
    pub module: IntrinsicModuleName,
    pub name: IntrinsicName,
    pub params: Vec<sifr_type_system::Type>,
    pub return_type: sifr_type_system::Type,
    pub workload: Option<WorkloadClass>,
    pub features: Vec<StdlibFeature>,
}

pub struct StdlibFeatureSpec {
    pub feature: StdlibFeature,
    pub cargo_dependencies: Vec<GeneratedCargoDependency>,
}
```

Names may change during implementation, but the same concepts must exist and must be tested.

## Generated Dependency Contract

Today, generated Cargo dependency decisions are spread through codegen/driver logic. This phase centralizes the decision table:

1. Lowering records stdlib/intrinsic usage through stable module/member or feature ids.
2. Codegen records required target features while emitting Rust.
3. Driver resolves those features through `sifr_stdlib`.
4. Driver renders deterministic generated Cargo dependencies.

Intrinsic contract consistency rules:

- Every intrinsic signature in `sifr_stdlib` must have either a matching codegen implementation in `sifr_codegen` or an explicit unsupported/deferred diagnostic path.
- Every codegen intrinsic implementation must have a signature in `sifr_stdlib`.
- Every intrinsic implementation that requires an external crate must report a stable `StdlibFeature` resolved through `sifr_stdlib`.
- Tests must fail if a signature, implementation, or feature dependency is added without the corresponding contract entry.

Dependency rendering rules:

- Generated dependencies are sorted deterministically.
- Duplicate dependencies are deduplicated by normalized package name and feature set.
- Dependency specs are snapshot-tested.
- Existing generated dependencies for current stdlib modules remain byte-for-byte equivalent unless the phase contract explicitly records an intentional cleanup.
- Async/tokio dependency activation remains tied to actual async/runtime feature usage, not to the mere existence of async support in the compiler.

## Milestones

### milestone_stdlib_boundary_1: Create `sifr_stdlib` Contract Crate

Scope:

- Add `crates/sifr_stdlib` to the workspace.
- Move stdlib intrinsic signatures out of `crates/sifr_hir/src/stdlib` into `sifr_stdlib`.
- Move embedded `lib/sifr/*.sifr` source inventory metadata into `sifr_stdlib`.
- Keep stdlib bootstrap compilation in `sifr_driver`; driver reads source inventory from `sifr_stdlib`.
- Update lowering to call `sifr_stdlib` for intrinsic signatures.
- Preserve all current import/type-check/codegen behavior.

Validation:

- `cargo check -p sifr_stdlib`
- `cargo test -p sifr_stdlib`
- focused import/stdin/stdout stdlib E2E fixtures that prove intrinsic signatures still resolve
- `cargo tree -p sifr_stdlib --depth 5` checked against the forbidden dependency set from this phase
- `cargo test -p sifr -- stdlib`
- `scripts/run_all_tests.sh --profile quick`

Definition of done:

- No `crates/sifr_hir/src/stdlib` module remains.
- `sifr_hir` or `sifr_lowering` no longer owns intrinsic signature definitions.
- `sifr_stdlib` has no dependency on lowering, frontend, codegen, driver, package, analysis, LSP, or CLI crates.

### milestone_stdlib_boundary_2: Centralize Stdlib Feature And Dependency Manifest

Scope:

- Add stable stdlib/runtime feature ids to `sifr_stdlib`.
- Move generated Cargo dependency mapping for stdlib/intrinsic usage into `sifr_stdlib`.
- Update codegen to emit required feature ids instead of owning dependency specs.
- Update driver generated-project rendering to resolve feature ids through `sifr_stdlib`.
- Add parity tests that prove intrinsic signatures, codegen implementations, and feature requirements stay synchronized.
- Preserve existing generated Cargo dependency behavior.

Validation:

- snapshot tests for every generated dependency case currently covered by stdlib/intrinsic modules
- intrinsic contract parity tests covering signature-without-codegen, codegen-without-signature, and feature-without-dependency negative cases
- focused generated-project tests for JSON, time, random, regex, compression/archive, bigint, decimal, async/tokio, and `sifr_runtime`
- `cargo test -p sifr_codegen`
- `cargo test -p sifr_driver`
- `scripts/check_codegen_binary_size.sh`
- `scripts/run_all_tests.sh --profile quick`

Definition of done:

- No generated Cargo dependency table remains in `sifr_codegen`.
- Driver-generated Cargo dependencies are rendered from `sifr_stdlib` feature specs.
- Codegen still owns intrinsic Rust implementation emission, but not dependency policy.

### milestone_ir_boundary_1: Extract `sifr_ir` Data Crate

Scope:

- Add `crates/sifr_ir`.
- Move reusable HIR data contracts out of `sifr_hir`:
  - HIR node types,
  - public lowered module/function/class/import views,
  - public lowering outcome/result data,
  - CFG and flow-graph data structs,
  - immutable public snapshots needed by downstream consumers.
- Keep construction algorithms and mutable state in the current lowering crate during this milestone.
- Update `sifr_codegen`, `sifr_lint`, `sifr_analysis`, and read-only downstream consumers to import IR data from `sifr_ir`.

Validation:

- `cargo check -p sifr_ir`
- `cargo test -p sifr_ir`
- `cargo tree -p sifr_codegen --depth 2` shows no parser/lowering dependency through IR
- `cargo tree -p sifr_lint --depth 1` shows no direct lowering dependency
- `cargo test -p sifr_codegen`
- `cargo test -p sifr_lint`
- `scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`

Definition of done:

- `sifr_codegen` depends on `sifr_ir`, not the lowering crate.
- `sifr_lint` depends on `sifr_ir`, not the lowering crate.
- CFG/flow-graph data types are reusable without depending on parser/lowering internals.
- CFG/flow-graph construction logic has not moved into `sifr_ir`.

### milestone_ir_boundary_2: Rename Remainder To `sifr_lowering`

Scope:

- Replace the remaining producer-side `sifr_hir` crate with `sifr_lowering`.
- Move lowering modules, mutable scope state, CFG/flow-graph construction, type/ownership/async analyses, and lowering diagnostics into `sifr_lowering`.
- Update `sifr_frontend` to depend on `sifr_lowering` for producing `sifr_ir`.
- Remove all workspace references to `sifr_hir`.
- Update architecture docs and guardrails to use the new crate names.

Validation:

- `rg "sifr_hir|crates/sifr_hir" Cargo.toml Cargo.lock crates internal_docs docs issues scripts verification` returns only historical/archive references or explicitly updated migration notes
- `cargo tree -p sifr_codegen --depth 2` confirms codegen has no lowering/parser dependency path through `sifr_ir`
- `cargo tree -p sifr_lint --depth 1` confirms lint has no direct lowering dependency
- `cargo check --workspace`
- `cargo test -p sifr_lowering`
- `cargo test -p sifr_frontend`
- `scripts/check_hir_maintainability_guardrails.py` updated or replaced to enforce `sifr_lowering` decomposition
- `scripts/run_all_tests.sh --profile quick`

Definition of done:

- No crate named `sifr_hir` remains in the workspace.
- The public architecture names are `sifr_ir` and `sifr_lowering`.
- Downstream read-only consumers cannot call lowering internals.

### milestone_ir_boundary_3: Dependency Direction Guardrails

Scope:

- Add or update guardrail scripts so future changes cannot reintroduce the old coupling.
- Enforce that:
  - `sifr_ir` does not depend on parser, syntax, frontend, lowering, stdlib, codegen, driver, package, analysis, LSP, or CLI crates.
  - `sifr_stdlib` does not depend on lowering, frontend, codegen, driver, package, analysis, LSP, or CLI crates.
  - `sifr_codegen` does not depend on `sifr_lowering`.
  - `sifr_lint` does not directly depend on `sifr_lowering`.
  - `sifr_analysis` does not directly depend on `sifr_lowering`.
  - generated dependency specs are owned by `sifr_stdlib`.
- Wire the guardrails into local quick validation.

Validation:

- new guardrail self-tests covering positive and negative dependency examples
- `scripts/run_all_tests.sh --profile quick`
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`

Definition of done:

- Dependency-direction rules are enforced locally and in CI-equivalent quick validation.
- Future accidental `sifr_codegen -> sifr_lowering`, `sifr_lint -> sifr_lowering`, or `sifr_analysis -> sifr_lowering` dependencies fail before PR review.

### milestone_ir_boundary_4: Documentation And Phase Closeout

Scope:

- Update `internal_docs/architecture.md`.
- Update relevant phase docs that still name `sifr_hir` as the HIR/lowering owner.
- Update build/test guidance if crate names appear in docs.
- Update execution checklist with validation evidence and review artifacts.

Validation:

- docs grep for stale current-state crate names
- `scripts/run_all_tests.sh --profile quick`
- file-size guardrail

Definition of done:

- Architecture docs describe the final crate tree.
- Execution checklist records validation evidence for every milestone.
- Review artifacts show final implementation-readiness review is `READY`.

## Quality Contract

- The phase must not change user-facing language behavior.
- All current pass/fail fixtures must preserve behavior unless a reviewed milestone explicitly records an intentional diagnostic wording or dependency-snapshot cleanup.
- No user-triggerable panic paths may be introduced.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` may be introduced into user runtime paths.
- Hand-maintained files touched by this phase must remain under the 900-line guardrail.
- Every milestone must run `scripts/run_all_tests.sh --profile quick` before PR.
- Full `scripts/run_all_tests.sh` is required before phase closeout.
- Each milestone must include at least one positive validation and one negative validation for the boundary it claims.

## Exit Gate

The phase exits only when:

- `sifr_stdlib`, `sifr_ir`, and `sifr_lowering` exist with the ownership described above.
- `sifr_hir` no longer exists as a workspace crate.
- Generated Cargo dependency decisions are centralized in `sifr_stdlib`.
- Codegen and lint consume `sifr_ir` without depending on lowering internals.
- Dependency-direction guardrails are part of quick validation.
- Architecture docs and execution checklist are updated.
