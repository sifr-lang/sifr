# Phase 14 Strict Finish Checklist (Code-Verified)

Last verified: 2026-02-24

This document is the source of truth for closing Phase 14 with no claimed-done gaps.
Every unchecked item below is a mandatory implementation slice.

Execution loop per slice:

1. Implement root-cause fix
2. Validate (`cargo test`, targeted e2e, `cargo clippy -- -D warnings`)
3. Demo verification
4. PR
5. Review
6. Merge
7. Update this checklist

---

## Milestone Status (Strict)

### milestone_rust_ir_types

status: **met**

- [x] `rust_ir.rs` exists with IR node families and `RawCode` variants
- [x] Derives present (`Debug`, `Clone`)
- [x] Export wiring in `lib.rs`
- [x] Representative unit tests present

### milestone_rust_ir_renderer

status: **met**

- [x] `render.rs` exists and renders all IR families
- [x] `RawCode` passthrough exists for item/stmt/expr/type
- [x] `insta` snapshot coverage present
- [x] `render_items([RawCode]) == raw` test present

### milestone_codegen_preamble_migration

status: **met**

- [x] IR preamble builders implemented (`error`, `file handles`, `logging`, imports)
- [x] `sifr_type_to_rust_type` implemented
- [x] Preamble validated/optimized via IR passes
- [x] Remove compatibility string scan for builtin error references from generated-code path
- [x] Error reachability now sourced from HIR + intrinsic usage metadata + stdlib preamble refs

### milestone_codegen_stmt_expr_migration

status: **partially met**

- [x] `lower_expr.rs`, `lower_stmt.rs`, `lower_item.rs`, `context.rs`, `preamble.rs` exist
- [x] Broad variant coverage exists in simple lowering paths
- [x] `expr_to_string` helper removed from production path
- [ ] Core pipeline still fallback-emitter first-class (`emit_*_fallback` remains active)
- [x] `emit_module` still string-emitter orchestration, not full `RustFile` assembly + single render
- [ ] `lower_*` contract is not full `Result<_, CodegenError>` end-to-end for production path
- [x] Production stmt lowering entry now has explicit `Result` contract (`try_lower_simple_stmt_with_scope_result`) with context validation
- [x] Production expr lowering entry now has explicit `Result` contract (`try_lower_leaf_expr_result`) with shape validation
- [x] Production module-constant item lowering entry now has explicit `Result` contract (`try_lower_simple_module_constant_item_result`) with name-shape validation
- [x] Production statement lowering now enters through `ScopeContext` (`try_lower_simple_stmt_with_scope` in `emit_stmt`)
- [x] Union enums still emitted as raw `enum_defs` strings, not `RustItem::Enum` nodes

### milestone_codegen_intrinsic_migration

status: **partially met**

- [x] Intrinsic registry exists (`intrinsics/mod.rs`) with dependency metadata
- [x] Method registry exists (`methods/mod.rs`)
- [x] Codegen returns `required_crates`
- [x] Legacy intrinsic fallback dispatcher remains huge and active (`emit_intrinsic_call` fallback arms)
- [x] Legacy method fallback path remains active (`emit_method_call` fallback arms)
- [x] `builtin_open` and related file-handle intrinsics still giant string literals
- [x] DoD constraint on long `self.write(...)` bodies (>100 chars) not met

### milestone_codegen_structural_passes

status: **partially met**

- [x] IR import pass exists (`ir_imports.rs`)
- [x] IR clone optimization exists (`ir_optimize.rs`)
- [x] IR validation exists (`ir_validate.rs`)
- [x] Old helper names (`filter_rust_code_to_needed`, `parse_rust_blocks`, etc.) removed
- [x] Boolean import flags removed from primary import selection path (imports now derived from structural IR needs)
- [x] Stdlib DCE migrated to structural `syn` item traversal (`stdlib_filter.rs` no longer uses text/chunk/token parsing)
- [x] `ir_imports` now uses structural `syn` traversal for `RawCode` payloads (text-token scan only as parse-failure fallback)
- [ ] `RawCode`-zero gate not enforced for all core production paths (IR type still carries bridge; fallback emitters remain)
- [ ] Structural passes not run over full user-code IR because full user-code IR assembly is not yet the production path

---

## Reopened Work Slices

## Slice 0: Metadata Dependency Parity in Test Runner

status: **done**

Root cause: `run_tests` built `Cargo.toml` manually and ignored codegen `required_crates`.

- [x] Route test-runner dependency generation through metadata-aware path
- [x] Aggregate `required_crates` across test files
- [x] Add unit tests for required crate inclusion and stdlib dependency preservation
- [x] Validate:
  - [x] `cargo test -p sifr_driver`
  - [x] `cargo clippy -p sifr_driver -- -D warnings`

## Slice 1: Eliminate Builtin Error Text Scan Shim

status: **done**

Root cause: `is_builtin_error_referenced` still scans generated Rust text.

- [x] Replace generated-code scan with structured metadata collection from HIR + intrinsic usage + stdlib preamble refs
- [x] Remove dependency on `helpers::is_builtin_error_referenced`
- [x] Keep behavior parity for conditional builtin error emission (validated by `sifr_codegen` test suite)
- [x] Cover type-position builtin errors (`Result[..., ValueError]`, class fields, constants) with regression tests
- [x] Validation:
  - [x] `cargo test -p sifr_codegen`
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`

## Slice 2: Make `generate_rust_test` Use Structural Import Collection

status: **done**

Root cause: test entrypoint still relies on emitter boolean flags for imports.

- [x] Build import set via structural pass on produced IR artifacts (not direct flags)
- [x] Remove direct `collection_needs/runtime_needs` import rendering in `entrypoints.rs`
- [x] Keep required crate metadata parity

## Slice 3: Replace `enum_defs` String Path with IR Items

status: **done**

Root cause: union enum generation remains string accumulation.

- [x] Refactor union enum generation to produce `Vec<RustItem>`
- [x] Render through common renderer path only
- [x] Remove `enum_defs: String` plumbing from `RustEmitter`

## Slice 4: Intrinsic Fallback Deletion (Registry-Only)

status: **done**

Root cause: giant legacy intrinsic fallback still in production.

- [x] Migrate remaining fallback-only intrinsics into registry modules
- [x] Delete legacy match arms from `emit_intrinsic_call`
- [x] Enforce no >100-char direct `self.write(...)` literal bodies in intrinsic lowering
- [x] Add guard test to prevent fallback reintroduction
- [x] Delete string-arg intrinsic lowering boundary; registry entrypoint now consumes IR expressions directly
- [x] Intrinsic registry caller now builds typed IR args (not blanket `RawCode` wrappers)

## Slice 5: Method Fallback Deletion (Registry-Only)

status: **done**

Root cause: `emit_method_call` still has broad fallback string emission.

- [x] Migrate remaining method lowering into registry modules
- [x] Reduce `emit_method_call` to registry dispatch + tightly-scoped non-registry semantics if unavoidable
- [x] Add guard test for registry-first/no-large-fallback growth
- [x] Delete string-arg method lowering boundary; method registry now consumes IR expressions directly

## Slice 6: Promote Full IR Module Assembly to Production Path

status: **done**

Root cause: production codegen still centers around string emitters + fallback.

- [x] Build `RustFile` for user code items in production path
- [x] Run structural passes on full `RustFile`
- [x] Render once at end (single renderer sink)
- [x] Keep parity tests green

## Slice 7: Structural DCE on IR (Replace Text-Token DCE)

status: **done**

Root cause: stdlib pruning is still text-token chunk parsing.

- [x] Replace `stdlib_filter` token/chunk DCE with IR item graph traversal
- [x] Keep transitive dependency behavior and order stability
- [x] Delete obsolete text/chunk parsing helpers after migration
- [x] Follow-up regression fix: macro-argument dependency tracking + file-handle lowering lifetime correction (PR #687)

## Slice 8: File-handle Open Intrinsic Template Cleanup

status: **done**

Root cause: `builtin_open` / `open_file` lowering still relied on monolithic raw string literals.

- [x] Centralize mode-branch generation into template helpers
- [x] Preserve runtime semantics while reducing giant inline string bodies for open paths
- [x] Migrate handle-result wrappers to typed IR closure/match structure (minimize monolithic `RawCode` wrappers)
- [x] Validate with full completion gate + demos
- [x] PR merged (#688)

---

## Completion Gate (Phase 14)

Phase 14 is complete only when all slices above are marked done and the following pass:

- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo test -p sifr --test e2e`
- [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [x] `cargo run -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`
