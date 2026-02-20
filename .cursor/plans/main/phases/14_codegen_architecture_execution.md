# Phase 14 Execution Checklist (Codegen Architecture)

This checklist operationalizes `14_codegen_architecture.md` into milestone-by-milestone execution loops.

Loop per milestone: **Work -> Validate -> Demo -> PR -> Review -> Merge -> Mark Done**

---

## Global Guards (apply to every milestone)

- [ ] Keep milestone scope limited to the current definition-of-done
- [ ] Preserve semantic parity: generated Rust must compile and behave the same
- [ ] Run `cargo test -p sifr_codegen` and `cargo clippy -p sifr_codegen -- -D warnings`
- [ ] Run targeted E2E parity checks for changed paths
- [ ] Create/update demo file in `demos/` named after the milestone
- [ ] Open one or multiple PRs for the milestone
- [ ] Review the PR(s), address findings, merge, and update roadmap/progress docs

---

## milestone_rust_ir_types

status: done

- [x] Add `crates/sifr_codegen/src/rust_ir.rs` with core IR node families:
  - [x] `RustFile`, `RustItem`, `RustStmt`, `RustExpr`
  - [x] `RustType`, `RustLiteral`, `RustParam`, `RustMatchArm`, `RustEnumVariant`, `RustTypeParam`, `Visibility`
  - [x] `RawCode(String)` variants at item/stmt/expr/type levels
- [x] Derive `Debug` and `Clone` for all IR types
- [x] Wire module exports in `crates/sifr_codegen/src/lib.rs`:
  - [x] `mod rust_ir;`
  - [x] `pub use rust_ir::*;`
- [x] Add representative IR construction tests in `rust_ir.rs`
- [x] Demo: `demos/milestone_rust_ir_types_demo.rs`
- [x] Open PR for milestone 1
- [x] Review and merge PR for milestone 1
- [x] Mark `milestone_rust_ir_types` status done in phase docs

---

## milestone_rust_ir_renderer

status: done

- [x] Add `crates/sifr_codegen/src/render.rs` with `Renderer`
- [x] Implement full render coverage for all IR variants
- [x] Add `render_items`, `render_stmts`, `render_expr` convenience functions
- [x] Wire exports in `lib.rs` (`mod render; pub use render::*;`)
- [x] Add renderer snapshot/unit tests (including `RawCode` passthrough)
- [x] Demo: `demos/milestone_rust_ir_renderer_demo.rs`
- [x] Open PR(s), review, merge
- [x] Mark done in phase docs

---

## milestone_codegen_preamble_migration

status: done

- [x] Move preamble emission to IR items (`error types`, `FileHandle`, logging, imports)
- [x] Add `sifr_type_to_rust_type(&Type) -> RustType`
- [x] Remove/reduce `is_builtin_error_referenced` string scanning
- [ ] Add differential old-vs-new codegen harness for parity
- [ ] Remove at least 5 clippy suppressions (including `format_push_string`)
- [x] Demo: `demos/milestone_codegen_preamble_migration_demo.sifr`
- [x] Open PR(s), review, merge
- [x] Mark done in phase docs

---

## milestone_codegen_stmt_expr_migration

status: in_progress

- [x] Introduce `context.rs` (`CodegenContext`, `ScopeContext`, `CodegenError`)
- [x] Add `lower_expr.rs`, `lower_stmt.rs`, `lower_item.rs`, `preamble.rs`
- [ ] Migrate `emit_*` to `lower_* -> IR -> render` dual path
- [ ] Implement semantic transforms (`elif`, `for/else`, `while/else`)
- [ ] Replace all `expr_to_string` call sites with structured lowering
- [ ] Convert >= 80% of stmt/expr arms to structured IR (`RawCode` only for remainder)
- [ ] Remove/replace at least 4 temporal coupling flags
- [ ] Add differential corpus parity tests
- [x] Demo: `demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [ ] Open PR(s), review, merge
- [ ] Mark done in phase docs

---

## milestone_codegen_intrinsic_migration

status: in_progress

- [x] PR1 scaffold: add `intrinsics/mod.rs` registry and `intrinsics/math.rs` domain lowerers (initial subset)
- [x] Route `emit_intrinsic_call` through registry-first dispatch with legacy match fallback
- [x] Demo: `demos/milestone_codegen_intrinsic_migration_demo.sifr` (initial registry path validation)
- [x] PR2 slice: add registry dependency metadata plumbing and migrate `json_loads/json_dumps` to registry lowering
- [x] PR3 slice: migrate expanded `sifr.math` intrinsic set into registry lowerers with parity checks
- [x] PR4 slice: migrate `sifr.env` intrinsic handlers into registry lowerers with env parity tests
- [x] PR5 slice: migrate `sifr.os` command/argv intrinsics into registry lowerers with OS parity tests
- [x] PR6 slice: migrate core `sifr.io` file/path intrinsics into registry lowerers with IO parity tests
- [x] PR7 slice: migrate additional `sifr.os` intrinsics (`chdir`, `getpid`, `cpu_count`, `stat_size`) into registry lowerers
- [x] PR8 slice: migrate remaining core `sifr.io` intrinsics (`append_text`, `walk_dir`) into registry lowerers
- [x] PR9 slice: migrate core `sifr.pathlib` intrinsics (`touch`, `resolve_path`, `iterdir`) into registry lowerers
- [x] PR10 slice: migrate `sifr.os.which` into registry lowerers
- [x] PR11 slice: migrate core `sifr.test` assertion intrinsics into registry lowerers
- [x] PR12 slice: migrate `sifr.collections` set intrinsics into registry lowerers
- [x] PR13 slice: migrate `sifr.collections` counter/defaultdict intrinsics into registry lowerers
- [ ] Add intrinsic registry (`intrinsics/mod.rs`) with metadata + dependency crates
- [ ] Split intrinsic lowerers into domain modules (`io`, `math`, `json`, etc.)
- [ ] Add method registry and type-specific method modules
- [ ] Remove driver string-scanning dependency detection; return `HashSet<String>` dependencies
- [ ] Convert all intrinsic/method lowering to structured IR (no `RawCode`)
- [ ] Reduce `lib.rs` by >= 2000 lines via decomposition
- [ ] Demo: `demos/milestone_codegen_intrinsic_migration_demo.sifr` (final milestone gate)
- [ ] Open PR(s), review, merge
- [ ] Mark done in phase docs

---

## milestone_codegen_structural_passes

status: pending

- [ ] Meet `RawCode`-zero gate (target zero; hard max 5 preamble-only documented)
- [ ] Add structural import collection pass from IR tree
- [ ] Replace `filter_rust_code_to_needed` with IR DCE pass
- [ ] Add conservative clone optimization pass
- [ ] Add IR validation pass for structural correctness
- [ ] Delete legacy string-parser helpers (`parse_rust_blocks`, `extract_top_level_item_name`, `count_braces`)
- [ ] Remove at least 20 clippy suppressions from file header
- [ ] Confirm generated binary size does not increase
- [ ] Demo: `demos/milestone_codegen_structural_passes_demo.sifr`
- [ ] Open PR(s), review, merge
- [ ] Mark phase 14 done in roadmap
