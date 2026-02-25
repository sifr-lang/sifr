# Phase 14 Closeout Epic: Eliminate Remaining Legacy Bridges in Codegen

Date: 2026-02-25  
Status: Open  
Phase: 14 `codegen_architecture`

---

## Why This Epic Exists

Phase 14 is marked `done` in planning docs, but the current codebase still has structural gaps against the strict finish criteria.  
This epic tracks the remaining implementation work needed to make the codebase match the intended end-state.

Primary source criteria:
- `.cursor/plans/main/phases/14_codegen_architecture.md`
- `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md`

Unchecked strict checklist items (currently real):
- `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md:57`
- `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md:162`
- `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md:163`

---

## Verified Gaps (Code-Evidence)

1. Fallback emitters are still production-first-class:
- `crates/sifr_codegen/src/lib.rs:1162`
- `crates/sifr_codegen/src/lib.rs:1166`
- `crates/sifr_codegen/src/lib.rs:1179`
- `crates/sifr_codegen/src/lib.rs:1183`

2. Module assembly is not fully IR-first:
- `crates/sifr_codegen/src/lib.rs:946`
- `crates/sifr_codegen/src/lib.rs:954`
- `crates/sifr_codegen/src/lib.rs:955`
- `crates/sifr_codegen/src/function_emitter.rs:8`
- `crates/sifr_codegen/src/class_emitter.rs:9`

3. `RawCode` remains in core production assembly:
- `crates/sifr_codegen/src/module_body.rs:39`
- `crates/sifr_codegen/src/module_constants.rs:19`
- `crates/sifr_codegen/src/lib.rs:411`

4. Structural passes still rely on raw-code fallback scanning:
- `crates/sifr_codegen/src/ir_imports.rs:34`
- `crates/sifr_codegen/src/ir_imports.rs:98`
- `crates/sifr_codegen/src/ir_imports.rs:165`
- `crates/sifr_codegen/src/ir_imports.rs:309`

5. Generator-init emission still string-based and transitively fallback-coupled:
- `crates/sifr_codegen/src/stmt_support_emitter.rs:6`
- `crates/sifr_codegen/src/function_emitter.rs:209`

6. Type-level raw bridge and downstream raw passthrough branches still exist:
- `crates/sifr_codegen/src/ir_imports.rs:274` (`RustType::RawCode` handling)
- `crates/sifr_codegen/src/stmt_support_emitter.rs:34` (`emit_lowered_stmts` raw stmt branch)
- `crates/sifr_codegen/src/expr_render_helpers.rs:224` (raw expr passthrough rewrite branch)
- `crates/sifr_codegen/src/intrinsics/mod.rs:309` (test helper raw args; test-only carveout required)
- `crates/sifr_codegen/Cargo.toml:12` (`syn` currently in main dependencies)

---

## Child Issues

1. `issues/217-phase14-remove-fallback-first-class-pipeline.md`
2. `issues/218-phase14-promote-full-ir-module-assembly.md`
3. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
4. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`

Execution order is strict: 217 -> 218 -> 219 -> 220.

---

## Completion Gate (Epic)

This epic is complete only when all child issues are merged and the following pass on `main`:

1. `cargo test -p sifr_codegen`
2. `cargo clippy -p sifr_codegen -- -D warnings`
3. `scripts/run_e2e_pass.sh` (defaults)
4. `cargo test -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture`
5. `cargo test --workspace`
6. `cargo clippy --workspace -- -D warnings`

And these conditions are true:

1. No production routing from `emit_stmt`/`emit_expr` directly to legacy fallback emitters.
2. `emit_module` produces module output from full IR assembly, not string drain-to-`RustItem::RawCode`.
3. No `RustItem::RawCode` / `RustStmt::RawCode` / `RustExpr::RawCode` in core production output path.
4. No production leakage of `RustType::RawCode` (including through `sifr_type_to_rust_type`).
5. Structural passes (`ir_imports` and related) no longer depend on raw-text fallback parsing for production outputs.
6. Test-only `RawCode` usage is explicitly carved out and documented; production hard gates do not fail test fixtures for that.

---

## Required Working Loop Per Child Issue

1. Implement root-cause fix (no compatibility shims unless explicitly justified).
2. Validate locally (tests + clippy + demos).
3. Open PR.
4. Self-review against acceptance criteria.
5. Merge.
6. Update phase docs/checklists in same PR or immediate follow-up PR.
