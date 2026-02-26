# Phase 14 Gap Cleanup Execution Plan (Implementation Blueprint)

Date: 2026-02-26  
Status: In Progress  
Owner: Codegen architecture closeout  
Primary scope:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md`
2. `issues/217-phase14-remove-fallback-first-class-pipeline.md`
3. `issues/218-phase14-promote-full-ir-module-assembly.md`
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
5. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`

---

## Why this rewrite is needed

The remaining work is not “routing cleanup” only. The core gap is that major emitters are still `.write()`-based string emitters.  
True closeout requires migrating these emitters to structured IR construction (`lower_*` style), then removing bridge/fallback paths.

This plan is dependency-ordered to avoid fake “done” states.

---

## Locked architecture decisions

1. User-code generation target is IR-first, not string-first.
2. Legacy fallback/bridge is temporary migration scaffolding only and must be removed from production path.
3. `RustItem::SynItem` policy:
4. `User code`: forbidden in production assembly.
5. `External stdlib compiled Rust text`: allowed only behind explicit boundary until replaced; must be documented and hard-gated from user-code paths.
6. Epic `216` cannot close while production user-code paths still depend on fallback bridge.

---

## Scope of emitter migration (actual bulk of work)

These are the real migration targets that must move from `.write()` emission to IR-building:

1. `crates/sifr_codegen/src/stmt_emitter.rs` (all `HirStmt` shapes)
2. `crates/sifr_codegen/src/expr_emitter.rs` (all `HirExpr` shapes)
3. `crates/sifr_codegen/src/class_emitter.rs` (struct/impl/Display/Error/operator-related class output)
4. `crates/sifr_codegen/src/function_emitter.rs` (fn signatures/body/generics/generator path)
5. `crates/sifr_codegen/src/class_method_emitter.rs`
6. `crates/sifr_codegen/src/match_emitter.rs`
7. `crates/sifr_codegen/src/method_call_emitter.rs`
8. `crates/sifr_codegen/src/operator_protocol_emitters.rs`
9. `crates/sifr_codegen/src/slice_emitter.rs`
10. `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
11. `crates/sifr_codegen/src/expr_ref_emitter.rs`
12. `crates/sifr_codegen/src/expr_render_helpers.rs` (legacy render fallback helpers)

Support/orchestration files:

1. `crates/sifr_codegen/src/lib.rs`
2. `crates/sifr_codegen/src/module_body.rs`
3. `crates/sifr_codegen/src/module_constants.rs`
4. `crates/sifr_codegen/src/lower_expr.rs`
5. `crates/sifr_codegen/src/lower_stmt.rs`
6. `crates/sifr_codegen/src/lower_item.rs`
7. `crates/sifr_codegen/src/entrypoints.rs`
8. `crates/sifr_codegen/src/ir_validate.rs`
9. `crates/sifr_codegen/src/ir_imports.rs`
10. `crates/sifr_codegen/src/stdlib_filter.rs`
11. `crates/sifr_codegen/src/rust_ir.rs`
12. `crates/sifr_codegen/src/render.rs`
13. `crates/sifr_codegen/Cargo.toml`
14. `crates/sifr_codegen/src/lib_codegen_tests.rs`

---

## Dependency graph (strict order)

1. Build/expand structured lowering coverage for remaining production-reachable stmt/expr/item shapes.
2. Migrate top-level module item assembly (class/function/method/operator/protocol/generator) to IR-first.
3. Remove production default fallback routing and migration wrappers.
4. Enforce no-raw/no-opaque gates (`RawCode` + user-path `SynItem`) in production assembly.
5. Finalize structural-pass hard gate and dependency rationale.
6. Close epic docs/checklists with evidence.

No step may skip prerequisites.

---

## Workstreams and PR slices

## WS0: Baseline quantification and coverage inventory (prerequisite)

Deliverables:
1. Add variant coverage inventory to `issues/217-phase14-remove-fallback-first-class-pipeline.md`:
2. all `HirExpr` variants with status: `structured-ready` / `legacy-dependent`
3. all `HirStmt` variants with status: `structured-ready` / `legacy-dependent`
4. production reachability marker per variant based on e2e/demo corpus
5. Add emitter ownership matrix to `issues/218-phase14-promote-full-ir-module-assembly.md` mapping each emitter file to migrated IR entrypoints.

PR slice:
1. PR-WS0-doc-baseline (docs + guard assertions only).

Completion gate:
1. Coverage inventory is committed and referenced by later PRs.

---

## WS1: Structured expression lowering expansion (core migration)

Target:
1. Move production-reachable expression shapes from legacy emitter paths into structured lowering (`lower_expr` + helpers).

Primary files:
1. `crates/sifr_codegen/src/lower_expr.rs`
2. `crates/sifr_codegen/src/expr_emitter.rs`
3. `crates/sifr_codegen/src/method_call_emitter.rs`
4. `crates/sifr_codegen/src/slice_emitter.rs`
5. `crates/sifr_codegen/src/expr_ref_emitter.rs`
6. `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
7. `crates/sifr_codegen/src/expr_render_helpers.rs`
8. `crates/sifr_codegen/src/lib.rs`
9. `crates/sifr_codegen/src/lib_codegen_tests.rs`

PR slices:
1. PR-WS1A: compound expr + calls + method/slice/ref shapes.
2. PR-WS1B: comprehensions/match/lambda/remaining production-reachable expr shapes.

Completion gate:
1. Production-reachable `HirExpr` variants are `structured-ready`.
2. No production wrapper callsite requires expression legacy bridge.

---

## WS2: Structured statement lowering expansion (core migration)

Target:
1. Move production-reachable statement shapes into structured lowering (`lower_stmt` + helpers).

Primary files:
1. `crates/sifr_codegen/src/lower_stmt.rs`
2. `crates/sifr_codegen/src/stmt_emitter.rs`
3. `crates/sifr_codegen/src/match_emitter.rs`
4. `crates/sifr_codegen/src/stmt_support_emitter.rs`
5. `crates/sifr_codegen/src/helpers.rs`
6. `crates/sifr_codegen/src/lib.rs`
7. `crates/sifr_codegen/src/lib_codegen_tests.rs`

PR slices:
1. PR-WS2A: control-flow stmt shapes (`if/while/for/match/try-except`) structured migration.
2. PR-WS2B: assignment/unpack/with/delete/nested function stmt shapes structured migration.

Completion gate:
1. Production-reachable `HirStmt` variants are `structured-ready`.
2. Generator-init path is structured-only (no legacy bridge dependency).

---

## WS3: Top-level item migration (class/function/method/operator/protocol/generator)

Target:
1. Replace string-emitter orchestration with item-first lowering for user module body.
2. Explicitly migrate generator function assembly from string closure template to structured IR.
3. Explicitly migrate trait/operator/protocol/class method emission paths.

Primary files:
1. `crates/sifr_codegen/src/module_body.rs`
2. `crates/sifr_codegen/src/class_emitter.rs`
3. `crates/sifr_codegen/src/function_emitter.rs`
4. `crates/sifr_codegen/src/class_method_emitter.rs`
5. `crates/sifr_codegen/src/operator_protocol_emitters.rs`
6. `crates/sifr_codegen/src/lower_item.rs`
7. `crates/sifr_codegen/src/lib.rs`
8. `crates/sifr_codegen/src/entrypoints.rs`
9. `crates/sifr_codegen/src/lib_codegen_tests.rs`

PR slices:
1. PR-WS3A: remove drain-parse `SynItem` for user class/function body.
2. PR-WS3B: class/method/operator/protocol structured item assembly.
3. PR-WS3C: generator function structured lowering and parity tests.

Completion gate:
1. User code class/function/method/operator/protocol/generator paths are item-first.
2. `module_body` no longer uses user-code drain->parse->`SynItem` flow.

---

## WS4: Bridge/fallback decommission (Issue 217 finalization)

Target:
1. Remove default production fallback routing.
2. Remove temporary migration wrappers and bridge emitters from production path.

Primary files:
1. `crates/sifr_codegen/src/lib.rs`
2. `crates/sifr_codegen/src/legacy_bridge_emitters.rs`
3. Remaining emitter callsites listed in WS1/WS2/WS3.
4. `crates/sifr_codegen/src/lib_codegen_tests.rs`
5. `issues/217-phase14-remove-fallback-first-class-pipeline.md`

PR slices:
1. PR-WS4A: strict production wrappers + guards.
2. PR-WS4B: delete/disable legacy bridge from production flow after coverage gate.

Completion gate:
1. No production routing from `emit_stmt`/`emit_expr` to legacy fallback emitters.
2. Guard tests fail on any reintroduction.

---

## WS5: RawCode/SynItem production gate hardening (Issue 219)

Target:
1. Keep `RawCode` zero in production assembled file.
2. Enforce user-path `SynItem` zero in production assembled file.
3. Explicitly handle stdlib preamble boundary.

Primary files:
1. `crates/sifr_codegen/src/lib.rs`
2. `crates/sifr_codegen/src/module_constants.rs`
3. `crates/sifr_codegen/src/module_body.rs`
4. `crates/sifr_codegen/src/ir_validate.rs`
5. `crates/sifr_codegen/src/rust_ir.rs`
6. `crates/sifr_codegen/src/lib_codegen_tests.rs`
7. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`

Policy implementation:
1. Add explicit validation rule:
2. production final `file_items` cannot contain `RustItem::RawCode`
3. production final `file_items` cannot contain `RustItem::SynItem` from user-code assembly path
4. If stdlib preamble still uses external compiled text, keep it in a dedicated boundary with explicit marker and test coverage.

PR slices:
1. PR-WS5A: module constants + user module body no-opaque cleanup.
2. PR-WS5B: production validation hard gate for raw/opaque leakage.

Completion gate:
1. Production user-code assembly is `RawCode`-zero and `SynItem`-zero.
2. Any allowed stdlib boundary is explicit, tested, and documented.

---

## WS6: Structural-pass hard gate completion (Issue 220)

Target:
1. Structural passes must not rely on raw-text fallback behavior in production.
2. Structural passes must not rely on opaque user-code payload parsing in production.

Primary files:
1. `crates/sifr_codegen/src/ir_imports.rs`
2. `crates/sifr_codegen/src/ir_validate.rs`
3. `crates/sifr_codegen/src/lib.rs`
4. `crates/sifr_codegen/src/entrypoints.rs`
5. `crates/sifr_codegen/src/stdlib_filter.rs`
6. `crates/sifr_codegen/Cargo.toml`
7. `crates/sifr_codegen/src/lib_codegen_tests.rs`
8. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`

PR slice:
1. PR-WS6A: structural-pass policy and dependency rationale hardening.

Completion gate:
1. Production structural passes do not parse raw text fallback payloads.
2. If `syn` remains runtime dependency, rationale is documented with exact usage.

---

## WS7: Epic closeout (Issue 216)

Primary files:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md`
2. `issues/217-phase14-remove-fallback-first-class-pipeline.md`
3. `issues/218-phase14-promote-full-ir-module-assembly.md`
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
5. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`
6. `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md`
7. `.cursor/plans/main/phases/14_codegen_architecture.md`
8. `.cursor/plans/main/architecture.md`
9. `.cursor/plans/main/roadmap.md`

To-do:
1. Link merged PRs in order.
2. Record completion evidence from local validation commands.
3. State final `SynItem` policy outcome explicitly.
4. Mark checklist entries only when code evidence exists on `main`.

Completion gate:
1. All child issues done with code evidence and local validation evidence.

---

## Execution loop per issue/PR

1. Implement scoped slice.
2. Run local validations.
3. Open PR.
4. Review against issue acceptance and this plan.
5. Merge.
6. Update issue + phase docs immediately.
7. Proceed to next slice.

---

## Local validation gate (required per PR)

1. `cargo test -p sifr_codegen`
2. `cargo clippy -p sifr_codegen -- -D warnings`
3. `cargo run -q -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
4. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`
5. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`

No CI waiting required for progression.

---

## Risk controls / anti-regression checks

1. Add grep-style decomposition tests for bridge/fallback reintroduction in production wrappers.
2. Add assembly tests asserting user-code module body does not produce `SynItem`.
3. Add final production assembly tests asserting no `RawCode` and no user-path `SynItem`.
4. Add structural-pass tests asserting raw fallback paths panic in production mode.
5. Keep temporary migration exceptions documented with removal owner and deadline.

---

## Issue-to-workstream mapping

1. `217`: WS0, WS1, WS2, WS3, WS4
2. `218`: WS0, WS3
3. `219`: WS5
4. `220`: WS6
5. `216`: WS7

---

## PR and merge policy

1. One issue focus at a time, but multiple PRs per issue are expected.
2. Every PR must include:
3. root cause
4. scope boundaries
5. exact files changed
6. local validation command results
7. residual risks and follow-ups
8. Merge only after local gate passes.
