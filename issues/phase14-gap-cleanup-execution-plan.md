# Phase 14 Gap Cleanup Execution Plan

Date: 2026-02-26  
Status: In Progress  
Scope:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md`
2. `issues/217-phase14-remove-fallback-first-class-pipeline.md`
3. `issues/218-phase14-promote-full-ir-module-assembly.md`
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
5. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`

---

## Execution Order and Loop

1. Plan and finalize to-do list for one issue.
2. Implement root-cause fix.
3. Run local demo + tests.
4. Open PR for that issue.
5. Review and merge PR.
6. Update docs/checklists.
7. Move to next issue.

No waiting on CI. Local validation is the gate.

---

## Global Validation Gate Per Issue

1. `cargo test -p sifr_codegen`
2. `cargo clippy -p sifr_codegen -- -D warnings`
3. `cargo run -q -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
4. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`
5. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`

---

## Cross-Issue Architecture Decisions (Locked)

1. `Issue 217` is dependency-ordered: wrapper fallback removal is not considered complete until structured lowering coverage is functionally complete for production-reachable stmt/expr shapes.
2. Generator lowering is a first-class migration track, not an implicit subtask.
3. `RustItem::SynItem` is treated as legacy/opaque. Target for this cleanup: zero `SynItem` in production `file_items`.
4. `lib.rs` stdlib preamble insertion via `push_syn_items_from_source(&stdlib_preamble, ...)` is explicitly in scope and must be removed or replaced by structured item assembly.
5. Any temporary exception must be explicitly documented in issue docs with exact file/line path and a blocking follow-up ticket before epic closeout.

---

## Issue 217 Plan: Remove fallback-first-class production routing

Scope baseline (from current bridge surface):
1. `HirExpr` variants currently covered by legacy bridge: 35
2. `HirStmt` variants currently covered by legacy bridge: 27
3. Immediate implication: removing fallback without coverage expansion is not valid.

To-do:
1. Keep `emit_stmt` and `emit_expr` strict structured-only in production path.
2. Remove default routing from production wrappers to legacy bridge/fallback.
3. Add structured coverage map by stmt/expr variant and mark production-reachable gaps.
4. Migrate generator-init and high-frequency emission callsites away from legacy wrappers.
5. Add or update guard tests to assert:
6. wrappers do not route to legacy bridge by default
7. new bridge/fallback routing cannot be reintroduced silently
8. Record temporary bridge surface explicitly with owner and removal checkpoint.
9. Remove temporary migration wrappers after structured coverage is complete.

PR target:
1. PR A: wrapper hardening + guard tests + coverage map.
2. PR B: structured lowering expansion for remaining production-reachable variants.
3. PR C: remove temporary migration wrappers/bridge once coverage reaches completion gate.

Done condition:
1. Production wrappers do not default to legacy bridge.
2. Production code path no longer depends on legacy bridge for reachable stmt/expr shapes.
3. Tests enforce regression protection.

Planned change files:
1. `crates/sifr_codegen/src/lib.rs`
2. `crates/sifr_codegen/src/legacy_bridge_emitters.rs`
3. `crates/sifr_codegen/src/stmt_support_emitter.rs`
4. `crates/sifr_codegen/src/function_emitter.rs`
5. `crates/sifr_codegen/src/stmt_emitter.rs`
6. `crates/sifr_codegen/src/expr_emitter.rs`
7. `crates/sifr_codegen/src/class_emitter.rs`
8. `crates/sifr_codegen/src/class_method_emitter.rs`
9. `crates/sifr_codegen/src/match_emitter.rs`
10. `crates/sifr_codegen/src/method_call_emitter.rs`
11. `crates/sifr_codegen/src/expr_ref_emitter.rs`
12. `crates/sifr_codegen/src/expr_render_helpers.rs`
13. `crates/sifr_codegen/src/helpers.rs`
14. `crates/sifr_codegen/src/operator_protocol_emitters.rs`
15. `crates/sifr_codegen/src/slice_emitter.rs`
16. `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
17. `crates/sifr_codegen/src/lib_codegen_tests.rs`
18. `issues/217-phase14-remove-fallback-first-class-pipeline.md`

---

## Issue 218 Plan: Promote full IR module assembly

To-do:
1. Remove module drain-and-parse `SynItem` assembly path for class/function body.
2. Introduce structured item lowering entrypoints for class/function top-level items.
3. Make module body assembly item-first only (`RustItem` nodes).
4. Add explicit generator-function lowering path:
5. migrate `function_emitter.rs` generator closure emission (`yield` pipeline) to structured item/body lowering.
6. Add explicit class trait/operator scope:
7. migrate class methods + operator impls + protocol impls + Display/Error impl generation into structured item assembly.
8. Ensure `generate_rust`, `generate_rust_test`, `generate_rust_multi` follow same item-first contract.
9. Add tests proving class/function/generator outputs are structured items and not string-drain wrappers.

PR target:
1. One or more PRs if needed:
2. PR A for module_body orchestration refactor.
3. PR B for class/function/method/operator/protocol item lowering migration.
4. PR C for generator lowering migration and parity tests.

Done condition:
1. User class/function/generator module body no longer depends on drain/parse `SynItem` wrapping.
2. Trait/operator/protocol impl emission is on structured assembly path.

Planned change files:
1. `crates/sifr_codegen/src/module_body.rs`
2. `crates/sifr_codegen/src/lib.rs`
3. `crates/sifr_codegen/src/entrypoints.rs`
4. `crates/sifr_codegen/src/class_emitter.rs`
5. `crates/sifr_codegen/src/function_emitter.rs`
6. `crates/sifr_codegen/src/lower_item.rs`
7. `crates/sifr_codegen/src/stmt_support_emitter.rs`
8. `crates/sifr_codegen/src/rust_ir.rs`
9. `crates/sifr_codegen/src/render.rs`
10. `crates/sifr_codegen/src/lib_codegen_tests.rs`
11. `issues/218-phase14-promote-full-ir-module-assembly.md`
12. `crates/sifr_codegen/src/class_method_emitter.rs`
13. `crates/sifr_codegen/src/operator_protocol_emitters.rs`

---

## Issue 219 Plan: Enforce RawCode-zero in core production path

To-do:
1. Remove remaining production string escape insertions for module constants and preamble.
2. Keep hard validation gates for `RustItem::RawCode`, `RustStmt::RawCode`, `RustExpr::RawCode`.
3. Ensure production-reachable type mapping never creates `RustType::RawCode`.
4. Add production gate for opaque items: fail if `RustItem::SynItem` reaches final production `file_items`.
5. Remove or test-gate `push_syn_items_from_source` in production assembly paths.
6. Add regression tests for production raw/opaque leakage.
7. Keep test-only raw helpers explicitly gated.

PR target:
1. PR A: module constants + preamble production path cleanup.
2. PR B: no-raw + no-opaque production validation hard gates.

Done condition:
1. Core production assembly path is raw-free and no-opaque (`SynItem`-free) and validated.

Planned change files:
1. `crates/sifr_codegen/src/module_constants.rs`
2. `crates/sifr_codegen/src/lib.rs`
3. `crates/sifr_codegen/src/module_body.rs`
4. `crates/sifr_codegen/src/ir_validate.rs`
5. `crates/sifr_codegen/src/expr_render_helpers.rs`
6. `crates/sifr_codegen/src/stmt_support_emitter.rs`
7. `crates/sifr_codegen/src/lower_expr.rs`
8. `crates/sifr_codegen/src/lower_stmt.rs`
9. `crates/sifr_codegen/src/lower_item.rs`
10. `crates/sifr_codegen/src/preamble.rs`
11. `crates/sifr_codegen/src/lib_codegen_tests.rs`
12. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
13. `crates/sifr_codegen/src/rust_ir.rs`

---

## Issue 220 Plan: Structural passes hard-gate

To-do:
1. Keep structural passes hard-failing when raw nodes appear in production.
2. Remove production fallback text-scanning reliance for raw handling paths.
3. Restrict any remaining fallback collectors to test-only if still needed.
4. Ensure structural import/validation passes do not depend on parsing opaque `SynItem` in production.
5. Keep `syn` runtime usage only where needed for structural AST traversal of explicit structured sources (for example stdlib IR filtering), not opaque item fallback.
6. Document dependency rationale if `syn` cannot be removed from main deps.

PR target:
1. One focused PR for structural pass hard-gate cleanup and dependency rationale.

Done condition:
1. Production structural passes do not rely on raw-text fallback behavior.
2. Production structural passes are not forced to parse opaque `SynItem` payloads.

Planned change files:
1. `crates/sifr_codegen/src/ir_imports.rs`
2. `crates/sifr_codegen/src/ir_validate.rs`
3. `crates/sifr_codegen/src/lib.rs`
4. `crates/sifr_codegen/src/entrypoints.rs`
5. `crates/sifr_codegen/src/stdlib_filter.rs`
6. `crates/sifr_codegen/Cargo.toml`
7. `crates/sifr_codegen/src/lib_codegen_tests.rs`
8. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`

---

## Issue 216 Plan: Closeout epic

To-do:
1. Verify merged PR list and completion gate evidence.
2. Reconcile epic completion conditions against current code state.
3. Update:
4. `issues/216-phase14-codegen-architecture-closeout-epic.md`
5. `issues/217-phase14-remove-fallback-first-class-pipeline.md`
6. `issues/218-phase14-promote-full-ir-module-assembly.md`
7. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
8. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`
9. `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md`
10. `.cursor/plans/main/phases/14_codegen_architecture.md` (if required by acceptance wording)
11. `.cursor/plans/main/architecture.md` and `.cursor/plans/main/roadmap.md` if milestone status changed.
12. Add explicit statement on `SynItem` final status and remaining exceptions (must be zero for closeout unless approved with blocking follow-up).

PR target:
1. Final closeout PR after all gap PRs are merged.

Done condition:
1. Epic completion criteria are true in code, not only in docs.

Planned change files:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md`
2. `issues/217-phase14-remove-fallback-first-class-pipeline.md`
3. `issues/218-phase14-promote-full-ir-module-assembly.md`
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
5. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`
6. `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md`
7. `.cursor/plans/main/phases/14_codegen_architecture.md`
8. `.cursor/plans/main/architecture.md`
9. `.cursor/plans/main/roadmap.md`

---

## 217 Completion Gate Quantification

1. Variant inventory table is added to `issues/217-phase14-remove-fallback-first-class-pipeline.md` with:
2. all `HirExpr` variants
3. all `HirStmt` variants
4. status per variant: structured-ready, partial, legacy-dependent
5. Production reachability marker per variant from e2e/demo coverage.
6. `Issue 217` is only marked done when production-reachable variants are no longer legacy-dependent.

---

## PR and Merge Policy

1. One issue at a time.
2. One or multiple PRs per issue only when split is necessary.
3. Every PR includes root cause, implementation, local validation output summary, and residual risk notes.
4. Merge after local validation is green. Do not wait for CI completion.
