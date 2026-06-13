# Phase 14 Closeout Epic: Codegen Architecture Closeout

Date: 2026-02-25  
Status: Completed (Final recheck: 2026-03-02)  
Phase: 14 `codegen_architecture`

---

## Final Gate Evidence (2026-03-02)

1. `cargo test -q -p sifr_codegen` -> pass (`446` passed, `0` failed).
2. `cargo test -q -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture` -> pass (`stmt=9/9`, `expr=9/9`).
3. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `268.51s`).
4. `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite: `394` passed, `0` failed).
5. Demo sweep (`find demos -name '*.sifr'` with `sifr run`) -> `TOTAL=91`, `PASS=86`, `FAIL=5` (expected non-runnable/intentional files only: `exclusivity_error_demo.sifr`, `models.sifr`, `utils.sifr`, `test_arithmetic.sifr`, `test_strings.sifr`).
6. Production token re-audit (`crates/sifr_codegen/src`, excluding `lib_codegen_tests.rs`) -> no matches for `self.write(`, `self.writeln(`, `self.output.push_str(`, `RawCode`, `SynItem`, `fallback`, `legacy`, `migration`, `bridge(`, `non_ir_path`, `pre_ir`, `try_emit_stmt_string_`.

---

## Why This Epic Exists

At epic creation time, Phase 14 was marked `done` in planning docs, but the codebase still had structural gaps against strict finish criteria.  
This epic tracked the implementation work required to bring the codebase to the intended end-state.

Primary source criteria:
- `internal_docs/phases/14_codegen_architecture.md`
- `internal_docs/phases/14_codegen_architecture_finish_checklist.md`

Unchecked strict checklist items at epic creation time (now resolved):
- `internal_docs/phases/14_codegen_architecture_finish_checklist.md:57`
- `internal_docs/phases/14_codegen_architecture_finish_checklist.md:162`
- `internal_docs/phases/14_codegen_architecture_finish_checklist.md:163`

---

## Verified Gaps (Historical Baseline Evidence)

1. Alternate_path emitters are still production-first-class:
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

4. Structural passes still rely on raw-code non_ir_path scanning:
- `crates/sifr_codegen/src/ir_imports.rs:34`
- `crates/sifr_codegen/src/ir_imports.rs:98`
- `crates/sifr_codegen/src/ir_imports.rs:165`
- `crates/sifr_codegen/src/ir_imports.rs:309`

5. Generator-init emission still string-based and transitively non_ir_path-coupled:
- `crates/sifr_codegen/src/stmt_support_emitter.rs:6`
- `crates/sifr_codegen/src/function_emitter.rs:209`

6. Type-level raw transition and downstream raw passthrough branches still exist:
- `crates/sifr_codegen/src/ir_imports.rs:274` (`RustType::RawCode` handling)
- `crates/sifr_codegen/src/stmt_support_emitter.rs:34` (`emit_lowered_stmts` raw stmt branch)
- `crates/sifr_codegen/src/expr_render_helpers.rs:224` (raw expr passthrough rewrite branch)
- `crates/sifr_codegen/src/intrinsics/mod.rs:309` (test helper raw args; test-only carveout required)
- `crates/sifr_codegen/Cargo.toml:12` (`syn` currently in main dependencies)

All baseline gaps above are resolved through merged child issues `#784` -> `#787` and final closeout `#791`.

---

## Child Issues

1. `Issue 217`
2. `issues/218-phase14-promote-full-ir-module-assembly.md`
3. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
4. `Issue 220`

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

1. No production routing from `emit_stmt`/`emit_expr` directly to pre_ir non_ir_path emitters.
2. `emit_module` produces module output from full IR assembly, not string drain-to-`RustItem::RawCode`.
3. No `RustItem::RawCode` / `RustStmt::RawCode` / `RustExpr::RawCode` in core production output path.
4. No production leakage of `RustType::RawCode` (including through `sifr_type_to_rust_type`).
5. Structural passes (`ir_imports` and related) no longer depend on raw-text non_ir_path parsing for production outputs.
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
- None. Final recheck gates are green as of 2026-03-02.

Completion gate validated on 2026-02-25:
- `cargo test -p sifr_codegen`
- `cargo clippy -p sifr_codegen -- -D warnings`
- `scripts/run_e2e_pass.sh` (defaults)
- `cargo test -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

WS0 closeout evidence documented on 2026-02-27:
- `Issue 217` now includes `HirExpr`/`HirStmt` coverage inventory with phase corpus reachability markers.
- `issues/218-phase14-promote-full-ir-module-assembly.md` now includes emitter ownership matrix mapped to migrated IR entrypoints.

Re-validation run on 2026-02-27:
- `cargo test -p sifr_codegen` (pass)
- `cargo clippy -p sifr_codegen -- -D warnings` (pass)
- `cargo test -q -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture` (pass; `stmt=8/9`, `expr=1/1`)
- `cargo run -q -p sifr -- run demos/milestone_codegen_stmt_expr_conversion_demo.sifr` (pass)
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
- Production source terminology scan in `crates/sifr_codegen/src` for `transition|non_ir_path|pre_ir|conversion` (no matches)
- Current `.write(...)` count in `crates/sifr_codegen/src`: `1132`

Closeout decision:
- Epic cannot be marked done again until user-path `SynItem` and remaining high-traffic string emitters are migrated to structured IR and all strict completion conditions are revalidated.

Latest loop update (2026-03-01):
- Landed `66e4ed97`: structured IR expansion for type/operator emission with root semantic fixes in operator lowering.
- Landed `b26a68e5`: removed dead pre_ir `match_emitter` module from production build graph.
- Validation:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed)
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
- `./scripts/run_all_tests.sh` -> pass
- Demo sweep (`demos/**/*.sifr` via `sifr run`) confirmed no new runnable regressions; remaining failures are fixture-only (`no main`) or intentional error demo.
- Refreshed strict inventory: `self.write(...)` in `crates/sifr_codegen/src` is now `594`.
- Follow-up micro-slices in same loop:
- `fc2d650c`: removed test-only `self.write(` assertion trace in `lib_codegen_tests.rs`.
- `6ac79b14`: removed remaining direct helper callsite in `output_helpers.rs`.
- Refreshed strict inventory after follow-up slices: `self.write(...)` in `crates/sifr_codegen/src` is now `592`.
- Additional loop slice:
- `c045131c`: `lib.rs` structured expr paths now emit lowered IR expressions via `emit_rust_expr` instead of direct `render_expr` string writes.
- Validation for `c045131c`: `cargo test -q -p sifr_codegen` pass, `./scripts/run_all_tests.sh` pass, demo smoke pass.
- Refreshed strict inventory after `c045131c`: `self.write(...)` in `crates/sifr_codegen/src` is now `588` (`lib.rs` `16 -> 12`).
- Additional loop slices:
- `2608d998`: bool-op structured emission now lowers to IR binop tree via registry lowering (removed bool-op string assembly path).
- `86e17211`: `lib.rs` statement terminators now route through shared stmt helper (`write_stmt_terminator`), removing direct terminator writes in lib stmt path.
- Validation for both slices: `cargo test -q -p sifr_codegen` pass (`455`), `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`), and `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- Refreshed strict inventory after these slices: `self.write(...)` in `crates/sifr_codegen/src` is now `583` (`expr_render_helpers.rs` `176 -> 175`, `lib.rs` `12 -> 8`).
- Additional loop slice:
- `62f10dff`: `expr_render_helpers.rs` now emits `print()` and single string-literal `print` through structured macro IR (`RustExpr::MacroCall`/`RustExpr::FormatMacro`) instead of direct string write emission.
- Validation: `cargo test -q -p sifr_codegen` pass (`455`), `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`), and milestone structural-pass demo pass.
- Refreshed strict inventory after `62f10dff`: `self.write(...)` in `crates/sifr_codegen/src` is now `581` (`expr_render_helpers.rs` `175 -> 173`).
- Additional loop slices:
- `fcd02efd`: `isinstance` terminal boolean branches now emit `RustLiteral::Bool` IR instead of string booleans.
- `4303b745`: literal-only string concat fast path now emits IR method call (`Str(...).to_string()`).
- `7c6fb3b4`: pre-call rewritten `HirExpr::Name` emission now routes via `emit_rust_expr`.
- `1721a268`: set literals now emit structured IR block (`HashSet::new` + `insert` + return value) instead of formatted `HashSet::from([...])` string assembly.
- Validation for these slices: `cargo test -q -p sifr_codegen` pass (`455`), `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`), and milestone structural-pass demo pass.
- Refreshed strict inventory after these slices: `self.write(...)` in `crates/sifr_codegen/src` is now `575` (`expr_render_helpers.rs` `173 -> 167`).
- Additional loop slice:
- `3b0c119c`: union `isinstance` membership emission now routes through structured `matches!` macro IR node rather than token-by-token write assembly.
- Validation: `cargo test -q -p sifr_codegen` pass (`455`), `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`), and milestone structural-pass demo pass.
- Refreshed strict inventory after this slice: `self.write(...)` in `crates/sifr_codegen/src` is now `570` (`expr_render_helpers.rs` `167 -> 162`).
- Additional loop slices:
- `54392392`: list literal emission now lowers to `RustExpr::Vec` IR (replacing formatted `vec![...]` string assembly).
- `2340bde8`: dict literal emission now lowers to structured `HashMap` IR block (`new` + `insert` + final expr) instead of formatted `HashMap::from([...])`.
- `6bf28c68`: static unary `not` outcomes now emit `RustLiteral::Bool` IR for tuple/none branches.
- Validation for these slices: `cargo test -q -p sifr_codegen` pass (`455`), `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`), and milestone structural-pass demo pass.
- Refreshed strict inventory after these slices: `self.write(...)` in `crates/sifr_codegen/src` is now `566` (`expr_render_helpers.rs` `162 -> 158`).
- Additional root-enabler slice:
- `22c8b6af`: expanded statement-expression IR lowering coverage in `stmt_support_emitter.rs` for `QuestionMark`, `OkWrap`, `ErrWrap`, `IfExpr`, and tuple index lowering.
- Validation: `cargo test -q -p sifr_codegen` pass (`455`), `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`), milestone structural-pass demo pass.
- Inventory impact: unchanged (`566` direct `self.write(...)`) because this slice is coverage-first infrastructure for the upcoming `lib.rs` let/assign IR conversion.
- Additional loop slice:
- `a489f70e`: structured print tail branches now emit `RustExpr::FormatMacro` IR nodes (including option-map display formatting) instead of raw formatted `println!` string writes.
- Validation: `cargo test -q -p sifr_codegen` pass (`455`), `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`), milestone structural-pass demo pass.
- Refreshed strict inventory after this slice: `self.write(...)` in `crates/sifr_codegen/src` is now `563` (`expr_render_helpers.rs` `158 -> 155`).
- Additional loop slices (2026-03-01):
- `05cc91e1`: moved structured result-wrap and walrus emission to IR (`RustExpr::FnCall`/`RustExpr::Block`) in `expr_render_helpers.rs`.
- `b4946b1f`: moved structured `contains`, unary ops, numeric binops (including `**` forms), and if-expr emission to `RustExpr` trees in `expr_render_helpers.rs`.
- Validation for both slices:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `405.92s`).
- Demo sweeps: recursive `demos/**/*.sifr` only hits known intentional error demo (`exclusivity_error_demo.sifr`); runnable milestone sweep `demos/*.sifr` passes (`83/83`).
- Refreshed strict inventory after `b4946b1f`: `self.write(...)` in `crates/sifr_codegen/src` is now `533` (`expr_render_helpers.rs` `155 -> 125`).
- Additional loop slice (2026-03-01):
- `2e1dd70a`: migrated structured call/lambda helper paths in `expr_render_helpers.rs` to IR emission (`FnCall`/`MethodCall`/`Closure`/`FormatMacro`) and removed direct token-write call assembly.
- Root semantic fix in this slice: callable struct-field invocation is emitted as callable-field form (`((obj.field))(...)`) instead of method-call form.
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `402.17s`).
- Runnable milestone demo sweep `demos/*.sifr` pass (`83/83`).
- Refreshed strict inventory after `2e1dd70a`: `self.write(...)` in `crates/sifr_codegen/src` is now `506` (`expr_render_helpers.rs` `125 -> 98`).
- Additional loop slice (2026-03-01):
- `4b6109bd`: migrated structured question-mark (`?`) emission and closure-error `map_err` shaping to `RustExpr::Try` + structured IR in `expr_render_helpers.rs`; compare-chain terminal emission now routes through IR node emission.
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `402.65s`).
- Refreshed strict inventory after `4b6109bd`: `self.write(...)` in `crates/sifr_codegen/src` is now `496` (`expr_render_helpers.rs` `98 -> 88`).
- Additional loop slice (2026-03-01):
- `302765ff`: migrated string concat (`+`), string repeat (`*`), and list concat (`+`) structured emission in `expr_render_helpers.rs` from direct token writes to structured `RustExpr`/`RustStmt` IR shapes.
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `418.21s`).
- Refreshed strict inventory after `302765ff`: `self.write(...)` in `crates/sifr_codegen/src` is now `475` (`expr_render_helpers.rs` `88 -> 67`).
- Additional loop slice (2026-03-01):
- `a6355f6b`: removed remaining direct `self.write(...)` usage from `expr_render_helpers.rs` (index/slice helpers now emit through structured emitter calls; no direct writes remain in that file).
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `412.77s`).
- runnable milestone demo sweep `demos/*.sifr` pass (`83/83`).
- Refreshed strict inventory after `a6355f6b`: `self.write(...)` in `crates/sifr_codegen/src` is now `408` (`expr_render_helpers.rs` `67 -> 0`).
- Additional loop slice (2026-03-01):
- `d5f3f22e`: added shared IR statement emission helper (`emit_rust_stmt_with_current_indent`) and migrated `stmt_support_emitter.rs` lowered-statement/generator-init/borrowed-return-clone paths to structured `RustStmt`/`RustExpr` emission.
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `410.51s`).
- runnable milestone demo sweep `demos/*.sifr` pass (`83/83`).
- Refreshed strict inventory after `d5f3f22e`: `self.write(...)` in `crates/sifr_codegen/src` is now `398` (`stmt_support_emitter.rs` `186 -> 176`).
- Additional loop slice (2026-03-01):
- `2c823925`: migrated more `stmt_support_emitter.rs` return/raise paths to structured IR return statements (`RustStmt::Return` with `Ok/Err/Some` function-call IR), reducing direct statement-string emission in closure/display/error branches.
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `411.82s`).
- runnable milestone demo sweep `demos/*.sifr` pass (`83/83`).
- Refreshed strict inventory after `2c823925`: `self.write(...)` in `crates/sifr_codegen/src` is now `390` (`stmt_support_emitter.rs` `176 -> 168`).
- Additional loop slice (2026-03-01):
- `e7ca4643`: migrated `stmt_support_emitter.rs` return/assert emission to structured IR statements (`RustStmt::Return`/`RustStmt::Assert`) and removed wrapped-return string helper assembly.
- Root-cause parity fix in the same slice: return/assert payload expressions now preserve structured expression semantics through IR-embedded rendered expressions where typed lowering coverage is not yet complete.
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `413.84s`).
- runnable milestone demo sweep `demos/*.sifr` pass (`83/83`).
- Refreshed strict inventory after `e7ca4643`: `self.write(...)` in `crates/sifr_codegen/src` is now `378` (`stmt_support_emitter.rs` `168 -> 156`).
- Additional loop slice (2026-03-01):
- `b33d143a`: migrated `lib.rs` structured `Let`/`Assign` emission to IR statements (`RustStmt::Let`/`RustStmt::Assign`) with structured-rendered IR expression payloads; removed remaining direct `self.write(...)` in `lib.rs`.
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `408.97s`).
- runnable milestone demo sweep `demos/*.sifr` pass (`83/83`).
- Refreshed strict inventory after `b33d143a`: `self.write(...)` in `crates/sifr_codegen/src` is now `370` (`lib.rs` `8 -> 0`).
- Additional loop slice (2026-03-01):
- `649fd630`: migrated walrus-`if` statement emission in `stmt_support_emitter.rs` to IR control nodes (`RustStmt::Let` + `RustStmt::If`) and removed direct write-assembled walrus-if path.
- Root-cause follow-up in same slice: added IR block-lowering helper support for `Assert` and expression statements when simple block lowering is unavailable, keeping walrus branches on the structured path.
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `429.15s`).
- runnable milestone demo sweep `demos/*.sifr` pass (`83/83`).
- Refreshed strict inventory after `649fd630`: `self.write(...)` in `crates/sifr_codegen/src` is now `363` (`stmt_support_emitter.rs` `156 -> 149`).
- Additional loop slice (2026-03-01):
- `28fad690`: migrated nested structured `if`/`while` block lowering in `stmt_support_emitter.rs` to recursive IR lowering (`try_lower_if_stmt_for_ir` / `try_lower_if_clause_for_ir` + nested while lowering) so nested option-guard trees remain on the structured IR path instead of falling out and hard-failing production emission.
- Root-cause fixes in the same slice:
- expanded nested block non_ir_path coverage for `Let`, `Assign`, `Return`, and `AttributeSubscriptAssign` to keep recursive branch bodies lowerable through IR.
- fixed borrowed-name compare lowering in recursive conditions (resolved `String` vs `&String` regression seen in `stdlib_argparse` and `milestone_stdlib_expansion` paths).
- Validation:
- `cargo test -q -p sifr_codegen` pass (`455`).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` pass.
- `cargo run -q -p sifr -- run demos/milestone_stdlib_expansion_demo.sifr` pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` pass (`394/394`, `409.83s`).
- `./scripts/run_all_tests.sh` pass.
- runnable milestone demo sweep `demos/*.sifr` pass (`83/83`).
- Refreshed strict inventory after `28fad690`: `self.write(...)` in `crates/sifr_codegen/src` is now `354` (`stmt_support_emitter.rs` `149 -> 140`).

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

Latest validation loop (2026-02-28, Pass G):
- Structured plain-call emission migrated to IR `FnCall` in `expr_render_helpers.rs`.
- Additional builtin special-calls now route through shared registry IR lowering (`bool`, `pow`, `bigint`, `round`, `abs`, `sum`, and 2-arg `min/max`), with duplicate write-based branches removed.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Included e2e pass check from script: `test_e2e_pass` -> `394` passed, `0` failed.
- Strict re-audit evidence: `self.write(...)` now `1050` total in `crates/sifr_codegen/src` (`expr_render_helpers.rs` `432 -> 375` in this pass).

Latest validation loop (2026-02-28, Pass H):
- Shared registry IR lowering expanded for special-call builtins `any`, `all`, `reversed`, `zip`.
- Duplicate write-based emission branches for these builtins were removed from `expr_render_helpers.rs`.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Included e2e pass check from script: `test_e2e_pass` -> `394` passed, `0` failed.
- Strict re-audit evidence: `self.write(...)` now `1036` total in `crates/sifr_codegen/src` (`expr_render_helpers.rs` `375 -> 361` in this pass).

Latest validation loop (2026-02-28, Pass I):
- Registry IR lowering expanded for special-call builtins `min/max` list form (`args.len()==1`), `sorted`, and `enumerate`.
- Duplicate write-based special-call branches for those operations were removed from `expr_render_helpers.rs`.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Included e2e pass check from script: `test_e2e_pass` -> `394` passed, `0` failed.
- Strict re-audit evidence: `self.write(...)` now `1020` total in `crates/sifr_codegen/src` (`expr_render_helpers.rs` `361 -> 345` in this pass).

Latest validation loop (2026-02-28, Pass J):
- Closed strict registry-IR lowering gaps in recursive expression lowering used by `str/repr` nested paths (`BoolOp`, `ConstructorCall`, option/debug-aware `FString`, option/string-aware `Compare`, callable-field invocation).
- Restored parity for recursive field access and self-field mutation semantics in registry-first method-call flow:
- inherited parent-field remap preserved in recursive `FieldAccess` lowering,
- self-field mutating calls now suppress clone in structured registry path (no mutation-on-clone regression).
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Focused e2e validation: `cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed).
- Strict re-audit evidence: `self.write(...)` now `1003` total in `crates/sifr_codegen/src` (`expr_render_helpers.rs` `345 -> 328` in this pass).

Latest validation loop (2026-02-28, Pass K):
- `int(...)` / `float(...)` special-call semantics migrated to shared registry builtin IR lowering in `intrinsic_method_emitters.rs`.
- Behavior parity preserved in registry IR path:
- string parse branches return `ParseError` via `map_err`,
- bool coercions map to numeric `0/1` / `0.0/1.0`,
- bigint->int conversion preserves overflow mapping to `OverflowError`.
- Removed duplicated write-assembled `int(...)` / `float(...)` branches from `expr_render_helpers.rs`.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Strict re-audit evidence: `self.write(...)` now `980` total in `crates/sifr_codegen/src` (`expr_render_helpers.rs` `328 -> 305` in this pass).

Latest validation loop (2026-02-28, Pass L):
- Started dependency-ordered emitter cleanup from the leaf utility file (`helpers.rs`) before higher-level orchestrators.
- Removed direct `.write` usage in `emit_expr_with_bigint_clone` by emitting structured IR `RustExpr::Clone` and rendering through shared IR emit helper.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Strict re-audit evidence: `helpers.rs` now has `0` direct `self.write(...)`; total `self.write(...)` remains `980` in `crates/sifr_codegen/src` pending follow-up conversion of higher-traffic emitters (`expr_render_helpers.rs`, `stmt_support_emitter.rs`, `slice_emitter.rs`).

Latest validation loop (2026-02-28, Pass M):
- Continued dependency-ordered leaf cleanup with `render.rs`.
- Removed all direct renderer `.write(...)` callsites (`self.write` / `renderer.write`) while preserving IR renderer output behavior.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Strict re-audit evidence: `render.rs` now has `0` direct `self.write(...)`; total `self.write(...)` is now `976` in `crates/sifr_codegen/src`.

Latest validation loop (2026-02-28, Pass N):
- Continued dependency-ordered cleanup by removing dead string-emitter module `slice_emitter.rs` and removing `mod slice_emitter;` from `lib.rs`.
- Verified module methods had no repo callsites before deletion (`emit_walrus_hoists`, `emit_list_slice`, `emit_string_slice`).
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Strict re-audit evidence: `slice_emitter.rs` removed; total `self.write(...)` now `896` in `crates/sifr_codegen/src`.

Latest validation loop (2026-02-28, Pass O):
- Migrated `match_emitter.rs` from fragmented `.write(...)` emission to rendered pattern/guard line assembly helpers.
- Preserved match semantics for option/union patterns, class capture substitution in guards, and string-pattern guard behavior.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Strict re-audit evidence: `match_emitter.rs` now has `0` direct `self.write(...)`; total `self.write(...)` is now `867` in `crates/sifr_codegen/src`.

Latest validation loop (2026-02-28, Pass P):
- Continued `expr_render_helpers.rs` IR-first cleanup for high-traffic expression assembly paths:
- method-call callsite string assembly was consolidated,
- bool-op/result-wrap/constructor/list/dict/set literal emitters moved to composed rendered expressions,
- walrus/contains and dictionary key lookup assembly were simplified with shared rendered-arg helper return values.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Strict re-audit evidence: `expr_render_helpers.rs` reduced `self.write(...)` from `305` to `249`; total `self.write(...)` is now `811` in `crates/sifr_codegen/src`.

Latest validation loop (2026-02-28, Pass Q):
- Continued `expr_render_helpers.rs` cleanup for additional expression hot paths:
- numeric binop emission (`**` and non-`**`), structured `if` expression emission, and structured index emission moved to composed expression-string assembly.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Strict re-audit evidence: `expr_render_helpers.rs` reduced `self.write(...)` from `249` to `176`; total `self.write(...)` is now `738` in `crates/sifr_codegen/src`.

Latest validation loop (2026-03-01, Pass R):
- Continued structured statement-path IR conversion in `stmt_support_emitter.rs` without transition/non_ir_path paths.
- Migrated structured handling for:
- `FieldAssign` -> IR `RustStmt::Assign` (including deque `_data` `VecDeque::new/from` handling),
- `AttributeSubscriptAssign` -> IR `MethodCall("insert", ...)`,
- `AugAssign` specialized string/list/pow and generic ops -> IR forms.
- Expanded statement-expression IR lowering helper coverage used by those statement emitters for non-leaf expression shapes encountered in production/e2e (constructor calls, compares, list literals, option-aware and direct indexing, builtin/special calls, string concat, generic binops).
- Restored assert-path behavioral parity by keeping canonical structured expression renderer path for `assert(...)` emission while preserving production hard-gate behavior.
- Removed remaining local `self.writeln(...)` usage in `stmt_support_emitter.rs` break/continue emission.
- Validation evidence:
- `./scripts/run_all_tests.sh` -> pass (unit + full e2e + e2e pass suite all green).
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- Focused crate validation: `cargo test -q -p sifr_codegen` -> pass (`455` passed).
- Strict re-audit evidence: direct `self.write(...)` count is now `706` in `crates/sifr_codegen/src` (`self.writeln(...)` count `63`).

Latest validation loop (2026-03-01, Pass S):
- Next-loop cleanup removed remaining non-render `self.writeln(...)` usage in:
- `crates/sifr_codegen/src/function_emitter.rs` (function close brace emission),
- `crates/sifr_codegen/src/match_emitter.rs` (match header/arm/body brace + destructure lines).
- Replaced with explicit indent + line write emission while preserving behavior.
- Validation evidence:
- `./scripts/run_all_tests.sh` -> pass.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- Re-audit evidence after this loop: direct `self.write(...) = 713`, direct `self.writeln(...) = 56` in `crates/sifr_codegen/src`.

Latest validation loop (2026-03-01, Pass T):
- Added explicit IR support for protocol trait method signatures:
- `RustItem::TraitMethodSig` introduced in `rust_ir.rs`,
- renderer support added in `render.rs`,
- IR import/optimize/validate/preamble raw-code counter passes updated to handle the new item shape.
- Migrated protocol class emission in `type_emitters.rs` to push structured `RustItem::Trait` with `TraitMethodSig` entries (no direct protocol string emission via `self.write(...)`).
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- Re-audit evidence after this loop: direct `self.write(...) = 697`, direct `self.writeln(...) = 56` in `crates/sifr_codegen/src`.

Latest validation loop (2026-03-01, Pass U):
- Migrated protocol implementation emission (`impl Protocol for Class`) in `operator_protocol_emitters.rs` to direct IR item assembly:
- removed string-assembled protocol impl blocks,
- now emits `RustItem::Impl` + method `RustItem::Fn` entries with explicit IR call/return nodes.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- Re-audit evidence after this loop: direct `self.write(...) = 679`, direct `self.writeln(...) = 57` in `crates/sifr_codegen/src` (all `self.writeln` remains confined to IR renderer).

Latest validation loop (2026-03-01, Pass V):
- Fully migrated `type_emitters.rs` from string assembly to IR-first item construction:
- enum/newtype item definitions now emit `RustItem::Enum` / `RustItem::TupleStruct` directly,
- enum/newtype Display and helper impls now emit structured `RustItem::Impl` + `RustItem::Fn`,
- enum/newtype user-defined methods now lower via structured stmt lowering into IR method bodies (no direct write path in this emitter).
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass.
- Full demo sweep `demos/*.sifr` -> pass (`83/83`).
- Re-audit evidence after this loop: direct `self.write(...) = 634`, direct `self.writeln(...) = 57` in `crates/sifr_codegen/src`.

Latest validation loop (2026-03-01, Pass W):
- Added explicit structured IR support for `with` statements:
- introduced `RustStmt::With` + `RustWithItem` in `rust_ir.rs`,
- added renderer and pass support (`render.rs`, `ir_imports.rs`, `ir_optimize.rs`, `ir_validate.rs`, `preamble.rs`, `lower_stmt.rs`, `expr_render_helpers.rs`).
- Migrated `stmt_support_emitter.rs` `try_emit_structured_with_stmt` from token writes to IR stmt emission and enabled recursive `HirStmt::With` lowering in `try_lower_stmt_block_for_ir`.
- Added recursive `Raise` lowering in block IR helper to keep nested control-flow branches on structured IR.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass.
- `cargo run -q -p sifr -- run demos/milestone_stdlib_expansion_demo.sifr` -> pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394/394`, `418.24s`).
- `./scripts/run_all_tests.sh` -> pass.
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- Re-audit evidence after this loop: direct `self.write(...) = 330` in `crates/sifr_codegen/src` with remaining files:
- `stmt_support_emitter.rs` (`116`), `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `function_emitter.rs` (`51`).

Latest validation loop (2026-03-01, Pass X):
- Migrated `try_emit_structured_while_stmt` to structured IR emission (`RustStmt::While`) instead of direct token-write assembly.
- Expanded recursive block IR lowering to cover nested `for` iteration forms (including `enumerate` lowering and collection iteration normalization) so while-body lowering remains in structured IR paths.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass.
- `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394/394`, `426.80s`).
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Re-audit evidence after this loop: direct `self.write(...) = 327` in `crates/sifr_codegen/src` with remaining files:
- `stmt_support_emitter.rs` (`113`), `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `function_emitter.rs` (`51`).

Latest validation loop (2026-03-01, Pass Y):
- Migrated `try_emit_structured_for_stmt` to structured IR emission (`RustStmt::For`) with recursive lowered body/else block emission, removing direct for-loop token assembly.
- Root-cause iterator fix in same slice: preserved generator/iterator pipelines in for-iter lowering so iterator-shaped expressions are not wrapped again with `.iter().cloned()`.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `SIFR_E2E_RUNNER_MODE=new cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394/394`, `78.64s`).
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Re-audit evidence after this loop: direct `self.write(...) = 308` in `crates/sifr_codegen/src` with remaining files:
- `stmt_support_emitter.rs` (`94`), `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `function_emitter.rs` (`51`).

Latest validation loop (2026-03-01, Pass Z):
- Migrated structured `try/except` emission in `stmt_support_emitter.rs` from direct token writes to IR stmt trees (`RustStmt::Let` + closure call + `Match`/`IfLet` handler dispatch).
- Added nested `TryExcept` support to block IR lowering (`try_lower_stmt_block_for_ir`) so nested try/except statements stay on structured IR paths.
- Root-cause parity fix in same slice: aligned block-level `Return` lowering with try-closure wrapping semantics used by production return emission, fixing nested try/except compile regressions in e2e groups.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `SIFR_E2E_RUNNER_MODE=new cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394/394`, `65.05s`).
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Re-audit evidence after this loop: direct `self.write(...) = 271` in `crates/sifr_codegen/src` with remaining files:
- `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `stmt_support_emitter.rs` (`57`), `function_emitter.rs` (`51`).

Latest validation loop (2026-03-01, Pass AA):
- Migrated `try_emit_structured_if_stmt` to IR-first lowering for union narrowing + option guards + generic branches (`RustStmt::If`/`IfLet`/`Match`), removing direct token-write if/elif/else assembly.
- Added nested block support for `FieldAssign` in `try_lower_stmt_block_for_ir` so IR-lowered `if` bodies remain production-reachable without non_ir_path panic.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `SIFR_E2E_RUNNER_MODE=new cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394/394`, `66.91s`).
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Re-audit evidence after this loop: direct `self.write(...) = 215` in `crates/sifr_codegen/src` with remaining files:
- `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `function_emitter.rs` (`51`), `stmt_support_emitter.rs` (`1`).

Latest validation loop (2026-03-01, Pass AB):
- Removed the final direct `self.write(...)` usage from `stmt_support_emitter.rs` by switching expression-statement termination to structured statement emission flow in `lib.rs`.
- Preserved structured expression behavior by reusing existing `try_emit_structured_expr` generation and wrapping emitted expression output as statement IR emission.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`455` passed, `0` failed).
- `SIFR_E2E_RUNNER_MODE=new cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394/394`, `63.74s`).
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass.
- Re-audit evidence after this loop: direct `self.write(...) = 214` in `crates/sifr_codegen/src` with remaining files:
- `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `function_emitter.rs` (`51`).

Next loop todo (dependency-ordered):
1. `function_emitter.rs` (`51`) IR conversion.
2. `class_method_emitter.rs` (`70`) IR conversion.
3. `class_emitter.rs` (`93`) IR conversion.

Latest validation loop (2026-03-01, Pass AC):
- Added missing renderer support for function generic bounds in IR (`RustItem::Fn` now renders `T: Bound` constraints instead of dropping bounds).
- Added renderer regression coverage for bounded generic function signatures (`render.rs` snapshot test).
- This is prerequisite plumbing for direct-IR conversion of `function_emitter.rs`/`class_*` emitters, which currently require accurate bounded generic rendering.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`456` passed, `0` failed).
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass (including e2e pass suite `394/394`).
- Re-audit evidence after this loop: direct `self.write(...) = 214` in `crates/sifr_codegen/src` (unchanged), remaining files:
- `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `function_emitter.rs` (`51`).

Latest validation loop (2026-03-01, Pass AD):
- Added structured statement-capture infrastructure in `RustEmitter`:
- new `capture_structured_stmts(...)` helper + `stmt_capture_stack` state.
- `emit_rust_stmt_with_current_indent` now supports IR-capture mode (collect `RustStmt` nodes without writing text output).
- Added regression coverage to ensure capture returns IR statements and does not write to `emitter.output`.
- This is the next prerequisite for migrating `function_emitter.rs`/`class_method_emitter.rs` off `self.write(...)` while preserving current structured stmt semantics.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass (including e2e pass suite `394/394`).
- Re-audit evidence after this loop: direct `self.write(...) = 214` in `crates/sifr_codegen/src` (unchanged), remaining files:
- `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `function_emitter.rs` (`51`).

Latest validation loop (2026-03-01, Pass AE):
- Exposed reusable registry method-call IR lowerer (`try_lower_registry_method_call_expr`) and registry plain-call signature lowerer (`try_lower_registry_plain_call_with_signature`) for statement-path integration.
- Integrated statement-path call lowering updates in `stmt_support_emitter.rs`:
- `print(...)` call lowering now emits structured `println!` IR (`MacroCall`/`FormatMacro`) instead of generic function-call IR in statement contexts.
- statement call lowering now consults signature-aware registry plain-call lowering before generic path.
- Kept loop iterator semantics stable by not globally switching method-call lowering in `lower_stmt_expr_for_ir` (regression was detected and fixed in-loop before final validation).
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- Full runnable demo sweep `demos/*.sifr` -> pass (`83/83`).
- `./scripts/run_all_tests.sh` -> pass (including e2e pass suite `394/394`).
- Re-audit evidence after this loop: direct `self.write(...) = 214` in `crates/sifr_codegen/src` (unchanged), remaining files:
- `class_emitter.rs` (`93`), `class_method_emitter.rs` (`70`), `function_emitter.rs` (`51`).

Latest validation loop (2026-03-02, Pass AF):
- Landed `f01f56b8`: continued IR-first cleanup with root-cause fixes in structured index/print lowering.
- Root fixes in this slice:
- removed the raw-string structured index emission path; `try_emit_structured_index_expr` now emits only the IR-lowered expression.
- expanded `try_lower_structured_index_expr` for option-receiver container indexing + tuple indexing with result-type unwrap behavior.
- fixed statement-path `print(...)` IR lowering for option-typed arguments (`map_or("None", ...)`) to avoid `Option<T>: Display` compile errors in demos/e2e.
- fixed stdlib-constant identifier rewrite on method receivers (exception variable `e` no longer rewritten into `std::f64::consts::E` when used as receiver).
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
- Full recursive demo sweep (`demos/**/*.sifr`) -> `91` scanned, `86` runnable pass, `5` expected non-runnable/intentional:
- `demos/milestone_borrow_hardening_demo/exclusivity_error_demo.sifr` (intentional type error demo).
- `demos/milestone_imports_demo/models.sifr` and `demos/milestone_imports_demo/utils.sifr` (module files, no `main`).
- `demos/milestone_test_runner_demo/test_arithmetic.sifr` and `demos/milestone_test_runner_demo/test_strings.sifr` (test fixtures, no `main`).
- Re-audit evidence after this loop:
- direct `self.write(...) = 0` in `crates/sifr_codegen/src`.
- remaining direct string-emitter helper is `self.writeln(...) = 68` (all in `render.rs`).

Latest validation loop (2026-03-02, Pass AG):
- Landed `bfcfbf5f`: migrated single-argument structured `print` lowering in `expr_render_helpers.rs` from `RawCode` string payloads to typed IR expression lowering (`try_lower_registry_expr_strict` + IR `map_or`/`FormatMacro` composition).
- Scope:
- removed `RustExpr::RawCode` usage in this branch for option and non-option single-arg print emission.
- preserved option display behavior with IR `map_or("None".to_string(), |__v| format!(...))`.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> `91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AH):
- Landed renderer/output sink cleanup for the remaining direct string helper traces:
- `render.rs`: `self.writeln(...)` replaced with `self.emit_line(...)`, and direct `output.push_str` writes replaced with formatted writer API usage.
- `output_helpers.rs`: direct `output.push_str` writes replaced with formatted writer API usage; helper renamed from `writeln` to `emit_line`.
- `lib_codegen_tests.rs`: removed direct `self.output.push_str` assertion literal trace.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.
- Re-audit evidence after this loop:
- direct `self.write(...) = 0` in `crates/sifr_codegen/src`.
- direct `self.writeln(...) = 0` in `crates/sifr_codegen/src`.
- direct `self.output.push_str(...) = 0` in `crates/sifr_codegen/src`.

Latest validation loop (2026-03-02, Pass AI):
- Continued IR-first cleanup in `expr_render_helpers.rs` with root-cause removal of string-backed argument/value lowering paths:
- structured method-call emission now lowers arguments to typed IR nodes directly (removed rendered-arg string pipeline and `RawCode` arg wrapping).
- plain-call signature `Result` error coercion now emits typed IR `map_err` closure instead of string interpolation into `RustExpr::RawCode`.
- structured `Ok/Err` wrapper emission and walrus emission now consume typed lowered IR values instead of rendered-string `RawCode`.
- `try_lower_expr_for_structured_emit` now falls back to strict registry lowering (typed IR) rather than rendering expression strings into `RawCode`.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.
- Re-audit evidence after this loop:
- `expr_render_helpers.rs` `RustExpr::RawCode(...)` sites reduced `13 -> 8`.

Latest validation loop (2026-03-02, Pass AJ):
- Continued IR-first cleanup in `expr_render_helpers.rs` compare lowering:
- `try_emit_structured_compare_expr` no longer builds rendered string chains for compare sequences; now emits typed IR compare chain (`RustExpr::BinOp`) including option wrapping and string `as_str()` normalization paths.
- no `RustExpr::RawCode` emission remains in this compare path.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.
- Re-audit evidence after this loop:
- `expr_render_helpers.rs` `RustExpr::RawCode(...)` sites reduced `8 -> 7`.

Latest validation loop (2026-03-02, Pass AK):
- Continued IR-first cleanup in slice emission:
- replaced `try_emit_structured_slice_expr` string-builder path in `expr_render_helpers.rs` with strict typed registry lowering of a constructed `HirExpr::Slice`.
- updated `lib.rs` structured slice callsite to pass the slice result type through to the slice emitter.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.
- Re-audit evidence after this loop:
- `expr_render_helpers.rs` `RustExpr::RawCode(...)` sites reduced `7 -> 5`.

Latest validation loop (2026-03-02, Pass AL):
- Continued IR-first cleanup in `expr_render_helpers.rs` with root-cause removal of the remaining string-backed constructor/isinstance emission path.
- `try_emit_structured_constructor_call_expr` now lowers args as typed IR, applies borrow conventions with typed IR (`Ref`/`Clone`), and applies `Option`/`Option<Box<_>>` wrapping using typed IR function calls (`Some`, `Box::new`) instead of rendered-string argument assembly and `RustExpr::RawCode`.
- `isinstance` union branch now emits a typed IR block + `IfLet` tree, removing `matches!(...)` macro arg `RawCode` construction.
- removed now-unused `try_render_structured_expr` helper (string-capture path) from `expr_render_helpers.rs`.
- moved borrow-prefix decision reuse to typed helper visibility (`borrow_prefix_for_name` is now `pub(crate)`), and `apply_borrow_prefix_expr` now uses it directly (no output capture).
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.
- Re-audit evidence after this loop:
- `expr_render_helpers.rs` no longer constructs `RustExpr::RawCode(...)` for production emission paths (remaining single `RawCode` occurrence is a defensive panic arm in AST rewrite traversal).

Latest validation loop (2026-03-02, Pass AM):
- Continued IR-first cleanup in display/format emission:
- introduced `lower_display_expr` in `expr_ref_emitter.rs` to produce typed display IR expressions directly (with existing option/debug behavior preserved), and rewired `emit_display_expr` to emit from this typed node.
- removed string-capture display fragments from `expr_render_helpers.rs` (`capture_emitted_fragment`/`render_display_expr_fragment`) and changed `write_format_macro_call` to consume typed IR args.
- migrated structured `print(...)` multi-arg and f-string macro arg lowering to use typed display IR args directly, removing `map(crate::RustExpr::RawCode)` format arg construction.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.
- Re-audit evidence after this loop:
- `expr_render_helpers.rs` no longer contains `map(crate::RustExpr::RawCode)` emission for `format!/println!` args; remaining `RustExpr::RawCode` mention stays a defensive panic arm only.

Latest validation loop (2026-03-02, Pass AN):
- Continued IR-first cleanup in display-path non_ir_path removal:
- removed direct-render probe non_ir_path from `expr_ref_emitter.rs` display lowering; `lower_display_expr` now relies on structured type/HIR-based option-inner inference only.
- this removes remaining `render_expr_via_direct_emit(...)` usage in `expr_ref_emitter.rs` display path and keeps display lowering on typed IR-only flow.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AO):
- Continued IR-first cleanup by removing force-render guard scaffolding from `expr_render_helpers.rs`.
- `try_lower_registry_expr_result` now relies directly on structured leaf lowering + stdlib-ident rewrite (no `should_force_render_guard` branch to intentionally return `None`).
- removed now-dead guard helper logic (`render_expr_contains_force_guard_name`, `render_expr_uses_borrowed_param`, and associated constant helper) to keep lowering path clean and typed.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AP):
- Continued IR-first cleanup in expression/string-render transition removal:
- removed `render_expr_via_direct_emit(...)` from `expr_render_helpers.rs`; `render_expr_with_lowered_path` now enforces strict registry IR lowering only and panics when lowering is missing.
- removed remaining string-shape probing non_ir_path from statement iterator analysis: `is_iterator_like_expr_for_ir` now panics on `RustExpr::RawCode` instead of inspecting string snippets.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AQ):
- Continued IR-first cleanup in registry method lowerers:
- removed `crate::render_expr(...)` string inspection from `methods/set.rs`, `methods/list.rs`, and `methods/dict.rs` borrow-shaping helpers.
- borrow detection now uses typed IR shape checks only (`RustExpr::Ref` / `MethodCall { method: "as_str" }` plus recursive wrapper unwrapping), eliminating string parsing from these production lowering paths.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AR):
- Continued IR-first cleanup in string method lowerers:
- removed final `crate::render_expr(...)` production usage from `methods/string.rs` borrow-shaping helper and replaced with typed IR shape checks (`Ref` / `as_str` / wrapper recursion).
- removed string interpolation for width formatting in `ljust/rjust/zfill`; these now pass typed `usize` cast arg via positional width format (`{:<1$}`, `{:>1$}`, `{:0>1$}`) instead of synthetic `Ident(\"width = ...\")`.
- updated string method lowering expectations in `methods/mod.rs` to match the typed positional-width IR rendering.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AS):
- Continued IR-first cleanup for typed collection literals:
- introduced typed IR `RustExpr::Array(Vec<RustExpr>)` in `rust_ir.rs` and renderer/validation/import/optimization/lowering support across `render.rs`, `expr_render_helpers.rs`, `lower_stmt.rs`, `preamble.rs`, `ir_imports.rs`, `ir_optimize.rs`, and `ir_validate.rs`.
- migrated `lower_expr` dict/set literal lowering from rendered-string array assembly to typed IR arrays while preserving `HashMap::from([..])` / `HashSet::from([..])` output form.
- removed the remaining `crate::render_expr(...)` production usage from `lower_expr.rs` dict/set literal assembly path.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AT):
- Continued IR-first cleanup in expr rendering helpers and tests:
- removed `render_expr_with_lowered_path` from `expr_render_helpers.rs` (non-sink helper path that rendered lowered exprs to strings).
- migrated affected tests in `lib_codegen_tests.rs` to a strict helper built on `try_lower_registry_expr_strict` + renderer call, preserving strict IR-only lowering assertions without reintroducing helper non_ir_path paths.
- Re-audit evidence after this loop:
- production `crate::render_expr(...)` callsites now reduced to output sink usage only (`output_helpers.rs`).
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AU):
- Continued RawCode eradication in test scaffolding:
- removed test-only raw placeholder constructors:
  - `lower_expr_raw` from `lower_expr.rs`
  - `lower_item_raw` from `lower_item.rs`
  - `lower_stmt_raw` from `lower_stmt.rs`
- removed associated placeholder tests that only asserted `RawCode` construction.
- replaced `intrinsics/mod.rs` test helper argument mapping from `RustExpr::RawCode` to typed test arg parsing (`Int`/`Float`/`Path`/`Ident`) so intrinsic lowering tests stay IR-typed.
- Validation evidence:
- `cargo test -q -p sifr_codegen` -> pass (`454` passed, `0` failed).
- `./scripts/run_all_tests.sh` -> pass (`394/394` e2e pass suite).
- Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AV):
- Continued IR-only cleanup with zero `RawCode` usage in codegen IR and passes.
- removed `RawCode` variants from core IR and propagated structural updates across:
  - `render.rs`
  - `ir_imports.rs`
  - `ir_validate.rs`
  - `ir_optimize.rs`
  - `lower_stmt.rs`
  - `expr_render_helpers.rs`
  - `stmt_support_emitter.rs`
  - `preamble.rs`
  - `entrypoints.rs`
  - `lib.rs`
- removed remaining `RawCode` references from crate tests (`intrinsics/mod.rs`, `lib_codegen_tests.rs`, `intrinsic_method_emitters.rs`) and updated assertions to IR-structural checks only.
- Re-audit evidence after this loop:
  - `rg -n "RawCode" crates/sifr_codegen` -> no matches.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`446` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AW):
- Continued IR-first cleanup by removing opaque `SynItem` from production IR.
- removed `RustItem::SynItem` from `rust_ir.rs` and propagated structural updates across:
  - `module_body.rs`
  - `render.rs`
  - `ir_imports.rs`
  - `ir_optimize.rs`
  - `ir_validate.rs`
  - `preamble.rs`
  - `lib.rs`
  - `lib_codegen_tests.rs`
- stdlib preamble is now handled as an explicit external-source boundary string in `lib.rs` (imports are computed by combining structural IR import-needs and source-scan import-needs), while user/module emission remains item-first IR.
- Re-audit evidence after this loop:
  - `rg -n "SynItem\\(" crates/sifr_codegen/src` -> no matches.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`445` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AX):
- Continued IR-first cleanup by tightening structured-capture strictness and reducing non-renderer output state usage.
- removed per-call output-leak slicing checks from:
  - `class_emitter.rs`
  - `function_emitter.rs`
  - `class_method_emitter.rs`
- added a hard guard in `output_helpers.rs` that panics if any string emission (`write`/`emit_rust_expr`/`emit_line`/`write_indent`) occurs while structured statement capture is active.
- removed dead direct-output borrow-prefix emitters from `method_call_emitter.rs` (`emit_borrow_prefix*`), keeping call-lowering on typed IR flow.
- Re-audit evidence after this loop:
  - `rg -n "RawCode|SynItem" crates/sifr_codegen/src` -> no matches.
  - `self.output` references are now constrained to renderer/output sink layers (`output_helpers.rs`, `render.rs`) plus one test-source assertion line.
- Validation evidence:
  - `cargo test -q -p sifr_codegen --lib --tests` -> pass (`445` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AY):
- Continued IR-first cleanup by removing the remaining non-capture output path in `RustEmitter` helper plumbing:
  - `output_helpers.rs`:
    - removed direct string output helper methods (`write`, `emit_line`, `write_indent`) from `RustEmitter`.
    - `emit_rust_stmt_with_current_indent` now hard-requires active statement capture and panics otherwise (no non_ir_path render-to-output branch).
    - `emit_rust_expr` now hard-panics to block direct expression string emission and enforce typed IR attachment to statements/items.
- updated `lib_codegen_tests.rs` tests that previously exercised `emit_expr -> output string` to assert strict typed lowering via `try_lower_registry_expr_strict` + renderer.
- Re-audit evidence after this loop:
  - no production `self.write(...)`/`self.writeln(...)` callsites remain in `crates/sifr_codegen/src`.
  - `output_helpers.rs` no longer contains direct render-to-output non_ir_path logic for statements.
- Validation evidence:
  - `cargo test -q -p sifr_codegen test_emit_expr_prefers_structured_name_path` -> pass (`1` passed).
  - `cargo test -q -p sifr_codegen test_emit_expr_borrowed_compare_is_structured` -> pass (`1` passed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass AZ):
- Continued IR-first cleanup by removing dead emitter output state and residual output-contract plumbing:
  - removed `output` and `indent` fields from `RustEmitter` in `lib.rs`.
  - removed `assert_output_drained` helper from `lib_support.rs`.
  - removed `assert_output_drained` callsites from:
    - `lib.rs` (`generate_rust_with_stdlib`, `generate_rust_multi`)
    - `entrypoints.rs` (`generate_rust_test`)
  - updated unreachable string-backend panic paths in `stmt_emitter.rs` and `expr_emitter.rs` to stop referencing deleted indentation state.
- updated architecture guard tests in `lib_codegen_tests.rs` to enforce the new contract:
  - no `emitter.output`/`assert_output_drained(...)` wiring in production assembly paths.
  - assembly remains IR item/stmt based and rendered only through `Renderer`.
- Re-audit evidence after this loop:
  - `RustEmitter` no longer owns mutable output-buffer/indent string state.
  - remaining output string ownership is isolated to `render.rs` renderer sink only.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`445` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BA):
- Continued IR-first cleanup by deleting dead expression-side-effect orchestration and obsolete string-backend modules:
  - removed `try_emit_structured_expr(...)` and `emit_expr(...)` wrappers from `lib.rs`.
  - removed obsolete module wiring from `lib.rs`: `mod expr_emitter;` and `mod stmt_emitter;`.
  - deleted dead files:
    - `crates/sifr_codegen/src/expr_emitter.rs`
    - `crates/sifr_codegen/src/stmt_emitter.rs`
- removed now-dead helper emission code paths:
  - removed `emit_expr_with_bigint_clone` from `helpers.rs`.
  - removed `emit_lambda_untyped` from `expr_render_helpers.rs` (unused side-effect emitter path).
- updated architecture guards in `lib_codegen_tests.rs` to enforce the stricter contract:
  - no `emit_expr` wrapper in `lib.rs`.
  - no `expr_emitter` / `stmt_emitter` module linkage.
  - `emit_stmt` remains the only production orchestration wrapper for structured statement lowering.
- Re-audit evidence after this loop:
  - no `emit_expr(` callsites remain in production code paths.
  - no `emit_expr_string_backend` / `emit_stmt_string_backend` traces remain in source modules.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`445` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BB):
- Continued IR-first cleanup by deleting dead expression side-effect emit helpers from `expr_ref_emitter.rs`.
- removed unreferenced `emit_*` writer-style methods that previously emitted directly through registry/output paths:
  - `emit_parenthesized_expr`
  - `emit_key_ref_expr`
  - `emit_str_ref_expr`
  - `emit_expr_as_str_ref`
  - `emit_expr_for_compare`
  - `emit_expr_with_parens_for_compare`
  - `emit_expr_as_bytes`
  - `emit_collection_expr`
  - `emit_display_expr`
- kept pure lowering helpers (`lower_ref_expr_or_panic`, `lower_display_expr`) as the canonical IR-first path for display/value lowering.
- Re-audit evidence after this loop:
  - no remaining callsites for removed `expr_ref_emitter` side-effect emit methods.
  - expression reference handling now stays on typed lowering helpers instead of writer-style APIs.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`445` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BC):
- Continued IR-first cleanup by pruning dead method-call side-effect emission plumbing.
- `expr_render_helpers.rs`:
  - removed dead `try_emit_structured_method_call(...)` side-effect emission path.
- `method_call_emitter.rs`:
  - removed dead registry/class/generic method-call emit wrappers and associated internal helpers.
  - retained only shared, still-used helpers:
    - `borrow_prefix_for_name(...)`
    - `is_generator_call(...)`
- `intrinsic_method_emitters.rs`:
  - removed dead `try_emit_method_via_registry(...)` emitter wrapper.
- cleanup follow-up:
  - removed now-unused imports caused by this prune (`MUTATING_METHODS` in `expr_render_helpers.rs`, `is_self_field_access_expr` re-export in `lib.rs`).
- Re-audit evidence after this loop:
  - no production references remain for removed method-call side-effect emit wrappers.
  - production call-site behavior continues through typed lowering paths only.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`444` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BD):
- Continued IR-first cleanup by removing dead intrinsic-side-effect emit wrappers and an unused stmt-side-effect helper:
  - `intrinsic_method_emitters.rs`:
    - removed dead wrappers:
      - `emit_intrinsic_call(...)`
      - `try_emit_intrinsic_via_registry(...)`
      - `emit_registry_plain_call_expr(...)`
      - `emit_stdlib_constant(...)`
    - updated internal contract test to assert wrapper-layer absence in production section.
  - `stmt_support_emitter.rs`:
    - removed dead `emit_borrowed_return_name_clone_expr(...)` side-effect helper.
- Re-audit evidence after this loop:
  - no production callsites remain for removed intrinsic wrapper emit APIs.
  - structured intrinsic behavior remains via typed lowering (`try_lower_registry_*`) paths.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`444` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BE):
- Continued IR-first cleanup by deleting the remaining dead expression side-effect emitter layer and its transition APIs:
  - `expr_render_helpers.rs`:
    - removed all dead `try_emit_structured_*` expression side-effect methods.
    - removed dead helper stack used only by that layer (`try_lower_expr_for_structured_emit`, callable/dict/string-emit helpers, f-string side-effect helper).
    - kept only actively-used typed lowering APIs (`try_lower_structured_field_access_expr`, `try_lower_structured_class_binop_expr`, `try_lower_structured_index_expr`) and identifier rewrite/lowering utilities.
  - `output_helpers.rs`:
    - removed dead `emit_rust_expr(...)` API entirely (no direct expression-emission surface remains).
  - `intrinsic_method_emitters.rs`:
    - removed dead `write_registry_expr(...)` transition wrapper.
  - `lib.rs` / `lib_support.rs`:
    - removed dead `is_reserved_plain_builtin_call` re-export and function (unreferenced after emitter-layer deletion).
- Re-audit evidence after this loop:
  - `self.write(...) = 0`, `self.writeln(...) = 0`, `emit_rust_expr(...) = 0` in `crates/sifr_codegen/src`.
  - `rg -n "RawCode|SynItem" crates/sifr_codegen/src` -> no matches.
  - `rg -n "non_ir_path|pre_ir|conversion|transition" crates/sifr_codegen/src` -> no matches.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`444` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BF):
- Continued IR-first hardening by adding architecture guards to prevent reintroduction of deleted expression side-effect/transition APIs:
  - `lib_codegen_tests.rs`:
    - added `test_expr_side_effect_emitter_layer_is_removed` asserting absence of:
      - expression-side-effect emitter surface in `expr_render_helpers.rs` (`try_emit_structured_*` / f-string side-effect helper traces),
      - direct expression-emission helper traces in `output_helpers.rs`,
      - registry write-transition helper trace in `intrinsic_method_emitters.rs`,
      - reserved plain builtin helper trace in `lib_support.rs`.
- Re-audit evidence after this loop (production source only):
  - `prod.self.write(...) = 0`
  - `prod.self.writeln(...) = 0`
  - `prod.emit_rust_expr(...) = 0`
  - `rg -n "RawCode|SynItem|non_ir_path|pre_ir|conversion|transition" crates/sifr_codegen/src --glob '!**/lib_codegen_tests.rs'` -> no matches.
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`445` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BG):
- Continued IR-first hardening by adding a recursive production-source banlist gate for non-IR token regressions:
  - `lib_codegen_tests.rs`:
    - added `test_production_codegen_source_has_no_non_ir_tokens`.
    - recursively scans `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`) and fails on any of:
      - `RawCode`, `SynItem`, `non_ir_path`, `pre_ir`, `conversion`, `transition`,
      - `self.write(`, `self.writeln(`, `emit_rust_expr(`, `write_registry_expr(`.
- Re-audit evidence after this loop (production source only):
  - `prod.self.write(...) = 0`
  - `prod.self.writeln(...) = 0`
  - `prod.emit_rust_expr(...) = 0`
  - no matches for `RawCode|SynItem|non_ir_path|pre_ir|conversion|transition` in `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`).
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`446` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BH):
- Continued IR-first cleanup by removing old indentation-era statement helper naming from production source:
  - `output_helpers.rs`:
    - renamed `emit_rust_stmt_with_current_indent(...)` to `push_captured_stmt(...)` to reflect pure IR-capture semantics.
  - updated all production callsites in:
    - `lib.rs`
    - `stmt_support_emitter.rs`
  - extended recursive banlist guard in `lib_codegen_tests.rs` to reject reintroduction of `emit_rust_stmt_with_current_indent(` token.
- Re-audit evidence after this loop (production source only):
  - `prod.self.write(...) = 0`
  - `prod.self.writeln(...) = 0`
  - `prod.emit_rust_expr(...) = 0`
  - `prod.old_stmt_api(...) = 0` for `emit_rust_stmt_with_current_indent(...)`
  - no matches for `RawCode|SynItem|non_ir_path|pre_ir|conversion|transition` in `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`).
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`446` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.

Latest validation loop (2026-03-02, Pass BI):
- Continued IR-first cleanup by removing remaining emitter-era naming from production statement-lowering APIs:
  - renamed production statement pipeline methods from `try_emit_structured_*` to `try_lower_structured_*` in:
    - `lib.rs`
    - `stmt_support_emitter.rs`
  - updated internal callsites and architecture test expectations accordingly.
  - fixed a guard regression where a test accidentally banned valid `try_lower_structured_*` helper names in `expr_render_helpers.rs`; the guard now correctly bans only removed `try_emit_structured_*` expression APIs.
- Re-audit evidence after this loop (production source only):
  - `prod.self.write(...) = 0`
  - `prod.self.writeln(...) = 0`
  - `prod.emit_rust_expr(...) = 0`
  - `prod.emit_structured_name(...) = 0` for `try_emit_structured_*`
  - `prod.old_stmt_api(...) = 0` for `emit_rust_stmt_with_current_indent(...)`
  - no matches for `RawCode|SynItem|non_ir_path|pre_ir|conversion|transition` in `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`).
- Validation evidence:
  - `cargo test -q -p sifr_codegen` -> pass (`446` passed, `0` failed).
  - `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`).
  - Full recursive demo sweep (`demos/**/*.sifr`) -> stable: `91` scanned, `86` runnable pass; same `5` expected non-runnable/intentional files.
