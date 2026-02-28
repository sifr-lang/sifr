# Phase 14 Closeout Epic: Eliminate Remaining Legacy Bridges in Codegen

Date: 2026-02-25  
Status: In Progress (Reopened 2026-02-28)  
Phase: 14 `codegen_architecture`

---

## Why This Epic Exists

At epic creation time, Phase 14 was marked `done` in planning docs, but the codebase still had structural gaps against strict finish criteria.  
This epic tracked the implementation work required to bring the codebase to the intended end-state.

Primary source criteria:
- `.cursor/plans/main/phases/14_codegen_architecture.md`
- `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md`

Unchecked strict checklist items at epic creation time (now resolved):
- `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md:57`
- `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md:162`
- `.cursor/plans/main/phases/14_codegen_architecture_finish_checklist.md:163`

---

## Verified Gaps (Historical Baseline Evidence)

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

All baseline gaps above are resolved through merged child issues `#784` -> `#787` and final closeout `#791`.

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

---

## Completion Summary

Child issues merged in required order:
- 217 via `#784`
- 218 via `#785`
- 219 via `#786`
- 220 via `#787`

Working-loop completion record (per AGENTS flow):
1. Plan/to-do defined per child issue (`217` -> `220`) with strict dependency order.
2. Root-cause implementation completed per issue scope.
3. Local validations executed per issue acceptance + phase gate.
4. PR opened and reviewed for each child issue.
5. PRs merged in order.
6. Phase docs/checklists updated after merge.
7. Epic closeout finalized in `#791`.

Open items:
- Structured IR migration still incomplete in high-traffic `.write(...)` emitters.
- User-path `RustItem::SynItem` still appears in module body assembly.
- Full closeout gate rerun remains pending until the remaining `.write(...)`/`SynItem` migration work is completed.

Completion gate validated on 2026-02-25:
- `cargo test -p sifr_codegen`
- `cargo clippy -p sifr_codegen -- -D warnings`
- `scripts/run_e2e_pass.sh` (defaults)
- `cargo test -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

WS0 closeout evidence documented on 2026-02-27:
- `issues/217-phase14-remove-fallback-first-class-pipeline.md` now includes `HirExpr`/`HirStmt` coverage inventory with phase corpus reachability markers.
- `issues/218-phase14-promote-full-ir-module-assembly.md` now includes emitter ownership matrix mapped to migrated IR entrypoints.

Re-validation run on 2026-02-27:
- `cargo test -p sifr_codegen` (pass)
- `cargo clippy -p sifr_codegen -- -D warnings` (pass)
- `cargo test -q -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture` (pass; `stmt=8/9`, `expr=1/1`)
- `cargo run -q -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr` (pass)
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` (pass)
- `scripts/run_e2e_pass.sh` (pass; `394` pass tests)

Recheck run on 2026-02-28 (current working tree):
- `cargo test -q -p sifr_codegen` (pass; `455` passed)
- `cargo clippy -q -p sifr_codegen -- -D warnings` (pass)
- `cargo test -q -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture` (pass; `stmt=9/9`, `expr=9/9`)
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` (fail; `301` passed, `93` failed)
- Current first-error distribution from `/tmp/phase14_e2e_recheck_first_error.txt`: `E0308=59`, `E0425=11`, `E0631=5`, `E0596=5`, `E0599=4`, others=9.

Recheck run on 2026-02-28 (Pass C, current tree):
- `./scripts/run_all_tests.sh` (pass; includes `test_e2e_pass` -> `394` passed, `0` failed)
- Full demo sweep `demos/*.sifr` (pass; `83/83`)
- Production source terminology scan in `crates/sifr_codegen/src` for `bridge|fallback|legacy|migration` (no matches)
- Current `.write(...)` count in `crates/sifr_codegen/src`: `1132`

Closeout decision:
- Epic cannot be marked done again until user-path `SynItem` and remaining high-traffic string emitters are migrated to structured IR and all strict completion conditions are revalidated.

Latest validation loop (2026-02-28):
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Included e2e pass check from script: `test_e2e_pass` -> `394` passed, `0` failed.
- Strict re-audit evidence: `self.write(...)` remains high (`1132` total in `crates/sifr_codegen/src`), and user-path `RustItem::SynItem` is still emitted in `crates/sifr_codegen/src/module_body.rs:54`.

Latest validation loop (2026-02-28, Pass F):
- Root-cause regressions fixed in structured expression lowering:
- string step-slice lowering in registry IR (`s[::2]`, `s[::-1]`),
- exponent binop lowering (`**`),
- builtin call lowering for `sum(...)` and display compare chains,
- class-method argument convention application in registry method-call lowering.
- Negative list/string indexing semantics restored in structured index IR blocks (no direct cast-to-`usize` regression).
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Included e2e pass check from script: `test_e2e_pass` -> `394` passed, `0` failed.
- Strict re-audit evidence: `self.write(...)` reduced to `1107` total in `crates/sifr_codegen/src`; remaining highest-write files are still `expr_render_helpers.rs` and `stmt_support_emitter.rs`.
