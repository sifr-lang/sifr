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

## Issue 217 Plan: Remove fallback-first-class production routing

To-do:
1. Keep `emit_stmt` and `emit_expr` strict structured-only in production path.
2. Remove default routing from production wrappers to legacy bridge/fallback.
3. Migrate generator-init and high-frequency emission callsites away from legacy wrappers.
4. Add or update guard tests to assert no default wrapper fallback routing.
5. Record remaining temporary bridge surface explicitly (if any) and track follow-up removal.

PR target:
1. One focused PR for wrapper routing + callsite migration slice.

Done condition:
1. Production wrappers do not default to legacy bridge.
2. Tests enforce regression protection.

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
4. Ensure `generate_rust`, `generate_rust_test`, `generate_rust_multi` follow same item-first contract.
5. Add tests proving class/function outputs are structured items and not string-drain wrappers.

PR target:
1. One or more PRs if needed:
2. PR A for module_body orchestration refactor.
3. PR B for class/function item lowering migration.

Done condition:
1. User class/function module body no longer depends on drain/parse `SynItem` wrapping.

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

---

## Issue 219 Plan: Enforce RawCode-zero in core production path

To-do:
1. Remove remaining production string escape insertions for module constants and preamble.
2. Keep hard validation gates for `RustItem::RawCode`, `RustStmt::RawCode`, `RustExpr::RawCode`.
3. Ensure production-reachable type mapping never creates `RustType::RawCode`.
4. Add regression tests for production raw/opaque leakage.
5. Keep test-only raw helpers explicitly gated.

PR target:
1. One focused PR for production-path raw/opaque elimination and gate hardening.

Done condition:
1. Core production assembly path is raw-free and validated.

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

---

## Issue 220 Plan: Structural passes hard-gate

To-do:
1. Keep structural passes hard-failing when raw nodes appear in production.
2. Remove production fallback text-scanning reliance for raw handling paths.
3. Restrict any remaining fallback collectors to test-only if still needed.
4. Keep `syn` runtime usage only where needed for structural AST traversal.
5. Document dependency rationale if `syn` cannot be removed from main deps.

PR target:
1. One focused PR for structural pass hard-gate cleanup and dependency rationale.

Done condition:
1. Production structural passes do not rely on raw-text fallback behavior.

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

## PR and Merge Policy

1. One issue at a time.
2. One or multiple PRs per issue only when split is necessary.
3. Every PR includes root cause, implementation, local validation output summary, and residual risk notes.
4. Merge after local validation is green. Do not wait for CI completion.
