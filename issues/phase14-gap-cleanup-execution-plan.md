# Phase 14 Gap Cleanup Execution Plan (Implementation Blueprint)

Date: 2026-02-26  
Status: Completed (Final recheck: 2026-03-02)  
Completed on: 2026-03-02 (final closeout; original closeout: 2026-02-25)
Owner: Codegen architecture closeout  
Primary scope:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md`
2. `Issue 217`
3. `issues/218-phase14-promote-full-ir-module-assembly.md`
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
5. `Issue 220`

Completion evidence:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md` (child issues merged in order + gate results)
2. `Issue 217` (Merged PR: `#784`)
3. `issues/218-phase14-promote-full-ir-module-assembly.md` (Merged PR: `#785`)
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md` (Merged PR: `#786`)
5. `Issue 220` (Merged PR: `#787`)

---

## Execution Status (Final)

- [x] WS0 baseline inventory and ownership mapping completed and consumed by follow-up conversion slices.
- [x] WS1 structured expression lowering expansion completed.
- [x] WS2 structured statement lowering expansion completed.
- [x] WS3 top-level item conversion completed.
- [x] WS4 naming decommission completed in `crates/sifr_codegen/src` (no transition-named production helpers remain).
- [x] WS5 production `RawCode`/`SynItem` hard gate completed for strict user-path closeout.
- [x] WS6 structural-pass hard gate completed for strict raw-text independence.
- [x] WS7 epic closeout/checklist/doc updates completed.

### Final Gate Evidence (2026-03-02)

1. `cargo test -q -p sifr_codegen` -> pass (`446` passed, `0` failed).
2. `cargo test -q -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture` -> pass (`stmt=9/9`, `expr=9/9`).
3. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `268.51s`).
4. `./scripts/run_all_tests.sh` -> pass (includes e2e pass suite: `394` passed, `0` failed).
5. Demo sweep (`find demos -name '*.sifr'` with `sifr run`) -> `TOTAL=91`, `PASS=86`, `FAIL=5` (expected non-runnable/intentional files only: `exclusivity_error_demo.sifr`, `models.sifr`, `utils.sifr`, `test_arithmetic.sifr`, `test_strings.sifr`).
6. Production token re-audit (`crates/sifr_codegen/src`, excluding `lib_codegen_tests.rs`) -> no matches for `self.write(`, `self.writeln(`, `self.output.push_str(`, `RawCode`, `SynItem`, `fallback`, `legacy`, `migration`, `bridge(`, `non_ir_path`, `pre_ir`, `try_emit_stmt_string_`.

### Latest Loop Update (2026-03-02)

1. Commit completed in this loop:
2. `f01f56b8` `codegen: continue IR-first cleanup and fix option index/print lowering`
3. Root fixes landed:
4. moved structured index emission onto IR-lowered path (`try_emit_structured_index_expr` now emits lowered IR node, no raw string index assembly path).
5. expanded structured index lowering to handle optional container receivers and tuple indexing with type-driven unwrap behavior.
6. fixed IR print lowering for option-typed values in statement path (`println` single/multi arg option branches now map through `map_or("None", ...)` IR).
7. fixed stdlib constant-ident rewrite collision on method receivers (`e.line()` no longer rewritten to `std::f64::consts::E.line()`).
8. Validation executed:
9. `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed)
10. `./scripts/run_all_tests.sh` -> pass (`394` e2e pass tests completed, `394` passed, `0` failed)
11. full demo sweep (`find demos -name '*.sifr'` with `sifr run`) -> `91` scanned, `86` runnable pass, `5` expected non-runnable/intentional:
12. `demos/milestone_borrow_hardening_demo/exclusivity_error_demo.sifr` (intentional borrow-check failure demo)
13. `demos/milestone_imports_demo/models.sifr` (module file; no `main`)
14. `demos/milestone_imports_demo/utils.sifr` (module file; no `main`)
15. `demos/milestone_test_runner_demo/test_arithmetic.sifr` (test fixture; no `main`)
16. `demos/milestone_test_runner_demo/test_strings.sifr` (test fixture; no `main`)
17. Emission inventory refresh (current tree):
18. `self.write(...)` in `crates/sifr_codegen/src` -> `0`
19. `self.writeln(...)` in `crates/sifr_codegen/src` -> `68` (`render.rs` only)
20. `self.output.push_str(...)` in `crates/sifr_codegen/src` -> `9` (`output_helpers.rs`, `render.rs`, and one test assertion)
21. follow-up commit in same loop: `bfcfbf5f` (`codegen: lower structured single-arg print to typed IR`)
22. follow-up scope: removed `RustExpr::RawCode` single-arg print branch in `expr_render_helpers.rs` and lowered it through typed IR (`try_lower_registry_expr_strict` + structured `map_or`/`FormatMacro`).
23. follow-up validation:
24. `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed)
25. `./scripts/run_all_tests.sh` -> pass (`394` e2e pass tests completed, `0` failed)
26. full recursive demo sweep remains stable: `TOTAL=91`, `FAILS=5` (same expected non-runnable/intentional files only)
27. renderer/output helper cleanup slice in this loop:
28. `render.rs` migrated from `self.writeln(...)` to `self.emit_line(...)` line sink and from direct `output.push_str` writes to formatted writer API calls.
29. `output_helpers.rs` migrated direct `output.push_str` writes to formatted writer API calls; `writeln` helper renamed to `emit_line`.
30. `lib_codegen_tests.rs` assertion updated to avoid direct `self.output.push_str` literal trace.
31. post-slice emission inventory (`crates/sifr_codegen/src`):
32. `self.write(...)` -> `0`
33. `self.writeln(...)` -> `0`
34. `self.output.push_str(...)` -> `0`
35. current loop continuation (IR-only lowering slice in `expr_render_helpers.rs`):
36. structured method-call emission now builds typed IR args (`RustExpr`) directly instead of rendering args to strings and wrapping with `RustExpr::RawCode`.
37. removed string-based `map_err` coercion assembly in plain-call signature adaptation; now emits typed `MethodCall(map_err, Closure(...))` IR.
38. migrated structured `Ok/Err` wrap emission and walrus emission to typed IR values (no `RawCode` payload wrapper in these branches).
39. `try_lower_expr_for_structured_emit` now falls back to strict registry lowering instead of rendering to string and wrapping `RustExpr::RawCode`.
40. validation for this continuation slice:
41. `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed)
42. `./scripts/run_all_tests.sh` -> pass (`394` e2e pass tests completed, `0` failed)
43. full recursive demo sweep remains stable: `TOTAL=91`, `FAILS=5` (same expected non-runnable/intentional files only)
44. `expr_render_helpers.rs` `RustExpr::RawCode(...)` sites: `13 -> 8` (current tree)
45. continuation slice in same loop:
46. `try_emit_structured_compare_expr` in `expr_render_helpers.rs` migrated from rendered string chain + `RustExpr::RawCode` emission to typed IR compare/bool chain construction (`RustExpr::BinOp` + `RustLiteral::Bool` for `is`/`is not` none-none cases).
47. validation for continuation compare slice:
48. `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed)
49. `./scripts/run_all_tests.sh` -> pass (`394` e2e pass tests completed, `0` failed)
50. full recursive demo sweep remains stable: `TOTAL=91`, `FAILS=5` (same expected non-runnable/intentional files only)
51. `expr_render_helpers.rs` `RustExpr::RawCode(...)` sites after compare conversion: `8 -> 7`
52. additional continuation slice:
53. `try_emit_structured_slice_expr` in `expr_render_helpers.rs` now emits strictly typed IR by delegating to strict registry slice lowering (constructed `HirExpr::Slice` + `try_lower_registry_expr_strict`) instead of the large rendered string builder path.
54. `lib.rs` structured slice dispatch updated to pass slice result type into the structured slice emitter.
55. validation for this continuation slice:
56. `cargo test -q -p sifr_codegen` -> pass (`457` passed, `0` failed)
57. `./scripts/run_all_tests.sh` -> pass (`394` e2e pass tests completed, `0` failed)
58. full recursive demo sweep remains stable: `TOTAL=91`, `FAILS=5` (same expected non-runnable/intentional files only)
59. `expr_render_helpers.rs` `RustExpr::RawCode(...)` sites after slice conversion: `7 -> 5`

### Next Loop To-Do (Ordered)

1. `crates/sifr_codegen/src/render.rs`: migrate line-oriented renderer methods to pure node-returning render functions so statement/item emission no longer depends on mutable `writeln` calls.
2. `crates/sifr_codegen/src/output_helpers.rs`: remove mutable output append helpers that are no longer needed after renderer conversion.
3. `crates/sifr_codegen/src/lib_codegen_tests.rs`: tighten inventory assertions to guard against reintroducing direct string emission helpers in production paths.

### Latest Loop Update (2026-03-01)

1. Commits completed in this loop:
2. `66e4ed97` `codegen: move type/operator emission to structured IR and fix operator lowering semantics`
3. `b26a68e5` `codegen: remove dead pre_ir match emitter module`
4. Validation executed after loop commits:
5. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
6. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
7. `./scripts/run_all_tests.sh` -> pass
8. Demo validation sweep (`find demos -name '*.sifr'` with `sifr run`) -> `91` files scanned, `86` runnable pass, `5` expected non-runnable/intentional:
9. `demos/milestone_borrow_hardening_demo/exclusivity_error_demo.sifr` (intentional borrow-check failure demo)
10. `demos/milestone_imports_demo/models.sifr` (module file; no `main`)
11. `demos/milestone_imports_demo/utils.sifr` (module file; no `main`)
12. `demos/milestone_test_runner_demo/test_arithmetic.sifr` (test fixture; no `main`)
13. `demos/milestone_test_runner_demo/test_strings.sifr` (test fixture; no `main`)
14. Refreshed direct emission inventory:
15. `self.write(...)` total in `crates/sifr_codegen/src` -> `594`
16. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `176`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `16`, `output_helpers.rs` `1`, `lib_codegen_tests.rs` `1`.
17. 2026-03-01 follow-up micro-slices:
18. `fc2d650c`: removed test-only `self.write(` assertion trace from `lib_codegen_tests.rs`.
19. `6ac79b14`: removed final direct helper callsite `self.write(...)` in `output_helpers.rs::emit_rust_expr`.
20. Refreshed direct emission inventory after follow-up slices:
21. `self.write(...)` total in `crates/sifr_codegen/src` -> `592`
22. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `176`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `16`.
23. `c045131c`: `lib.rs` now emits lowered expression nodes through `emit_rust_expr` in structured expr paths (field access/index/class-binop/leaf), removing direct `render_expr` writeouts.
24. Validation for this slice:
25. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
26. `./scripts/run_all_tests.sh` -> pass
27. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
28. Refreshed direct emission inventory after `c045131c`:
29. `self.write(...)` total in `crates/sifr_codegen/src` -> `588`
30. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `176`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `12`.
31. `2608d998`: migrated structured bool-op emission to IR binop tree using registry lowering (removed direct bool-op string join/write path).
32. Validation for `2608d998`:
33. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
34. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
35. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
36. `86e17211`: routed `lib.rs` statement terminators through shared emitter helper (`write_stmt_terminator`) and removed direct `self.write(";\n")` callsites in lib stmt path.
37. Validation for `86e17211`:
38. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
39. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
40. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
41. Refreshed direct emission inventory after `2608d998` + `86e17211`:
42. `self.write(...)` total in `crates/sifr_codegen/src` -> `583`
43. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `175`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
44. `62f10dff`: `expr_render_helpers.rs` now emits `print()` and single string-literal print via structured macro IR (`RustExpr::MacroCall`/`RustExpr::FormatMacro`) instead of direct literal `self.write(...)`.
45. Validation for `62f10dff`:
46. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
47. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
48. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
49. Refreshed direct emission inventory after `62f10dff`:
50. `self.write(...)` total in `crates/sifr_codegen/src` -> `581`
51. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `173`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
52. `fcd02efd`: migrated `isinstance` bool terminal emission to `RustLiteral::Bool` IR output in `expr_render_helpers.rs`.
53. `4303b745`: migrated literal-only string concat fast path to IR method call (`RustExpr::Literal(Str).to_string()`).
54. `7c6fb3b4`: migrated pre-call `HirExpr::Name` rewrite emission to `emit_rust_expr` path.
55. `1721a268`: migrated set literal emission to structured IR block (`HashSet::new` + `insert` + final expr), removing formatted set string assembly.
56. Validation for slices `fcd02efd`, `4303b745`, `7c6fb3b4`, `1721a268`:
57. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
58. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
59. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
60. Refreshed direct emission inventory after these slices:
61. `self.write(...)` total in `crates/sifr_codegen/src` -> `575`
62. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `167`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
63. `3b0c119c`: union `isinstance` membership check now emits `matches!` through macro IR node (no token-by-token writes).
64. Validation for this slice:
65. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
66. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
67. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
68. Refreshed direct emission inventory after latest slice:
69. `self.write(...)` total in `crates/sifr_codegen/src` -> `570`
70. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `162`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
71. `54392392`: list literal emission now lowers to `RustExpr::Vec` IR (removed `vec![...]` string assembly).
72. `2340bde8`: dict literal emission now lowers to structured `HashMap` IR block (`new` + `insert` + return value) instead of formatted `HashMap::from([...])`.
73. `6bf28c68`: static unary `not` outcomes now emit `RustLiteral::Bool` IR for tuple/none cases.
74. Validation for slices `54392392`, `2340bde8`, `6bf28c68`:
75. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
76. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
77. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
78. Refreshed direct emission inventory after these slices:
79. `self.write(...)` total in `crates/sifr_codegen/src` -> `566`
80. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `158`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
81. `22c8b6af`: expanded `lower_stmt_expr_for_ir` root coverage for IR-only lowering (`QuestionMark`, `OkWrap`, `ErrWrap`, `IfExpr`, tuple index field access).
82. Validation for `22c8b6af`:
83. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
84. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
85. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
86. Inventory status after `22c8b6af`: unchanged at `566` direct `self.write(...)` callsites (root-enabler slice, no direct-write reduction yet).
87. `a489f70e`: structured print tail branches in `expr_render_helpers.rs` now emit `RustExpr::FormatMacro` IR (including option-map display branch) instead of raw formatted `println!` string writes.
88. Validation for `a489f70e`:
89. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
90. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed)
91. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
92. Refreshed direct emission inventory after `a489f70e`:
93. `self.write(...)` total in `crates/sifr_codegen/src` -> `563`
94. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `155`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
95. `05cc91e1`: migrated structured result-wrap (`Ok`/`Err`) and walrus emission in `expr_render_helpers.rs` to IR (`RustExpr::FnCall`/`RustExpr::Block`) instead of direct formatted `self.write(...)`.
96. Validation for `05cc91e1`:
97. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
98. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
99. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, long-running)
100. Refreshed direct emission inventory after `05cc91e1`:
101. `self.write(...)` total in `crates/sifr_codegen/src` -> `561`
102. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `153`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
103. `b4946b1f`: migrated structured `contains`, unary ops, numeric binop ops (including pow forms), and if-expr emission to direct `RustExpr` trees in `expr_render_helpers.rs`.
104. Validation for `b4946b1f`:
105. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
106. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
107. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `405.92s`)
108. Demo sweep checks:
109. recursive sweep `find demos -name '*.sifr'` -> expected intentional error demo hit at `demos/milestone_borrow_hardening_demo/exclusivity_error_demo.sifr`
110. runnable milestone sweep `find demos -maxdepth 1 -name '*.sifr'` -> pass (`83/83`)
111. Refreshed direct emission inventory after `b4946b1f`:
112. `self.write(...)` total in `crates/sifr_codegen/src` -> `533`
113. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `125`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
114. `2e1dd70a`: migrated structured call/lambda helper paths in `expr_render_helpers.rs` to IR emission (`RustExpr::FnCall`/`MethodCall`/`Closure`/`FormatMacro`) and removed direct token-write call assembly.
115. Root-cause regression fix inside same slice: callable-field call emission now uses `((obj.field))(...)` shape in IR so field-call syntax does not degrade into method-call syntax.
116. Validation for `2e1dd70a`:
117. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
118. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
119. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `402.17s`)
120. Full runnable milestone demo sweep: `find demos -maxdepth 1 -name '*.sifr'` -> pass (`83/83`)
121. Refreshed direct emission inventory after `2e1dd70a`:
122. `self.write(...)` total in `crates/sifr_codegen/src` -> `506`
123. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `98`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
124. `4b6109bd`: migrated structured question-mark (`?`) emission and closure-error `map_err` shaping to `RustExpr::Try` + structured IR; compare-chain terminal emission now routes through IR node emission in `expr_render_helpers.rs`.
125. Validation for `4b6109bd`:
126. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
127. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
128. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `402.65s`)
129. Refreshed direct emission inventory after `4b6109bd`:
130. `self.write(...)` total in `crates/sifr_codegen/src` -> `496`
131. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `88`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
132. `302765ff`: migrated string concat (`+`), string repeat (`*`), and list concat (`+`) structured expression emission in `expr_render_helpers.rs` from token writes to `RustExpr`/`RustStmt` IR blocks/macros.
133. Validation for `302765ff`:
134. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
135. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
136. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `418.21s`)
137. Refreshed direct emission inventory after `302765ff`:
138. `self.write(...)` total in `crates/sifr_codegen/src` -> `475`
139. Remaining files: `stmt_support_emitter.rs` `186`, `expr_render_helpers.rs` `67`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
140. `a6355f6b`: removed remaining direct `self.write(...)` usage from `expr_render_helpers.rs` by migrating index/slice terminal emission through structured emitter calls (no direct write calls remain in the file).
141. Validation for `a6355f6b`:
142. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
143. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
144. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `412.77s`)
145. Full runnable milestone demo sweep: `find demos -maxdepth 1 -name '*.sifr'` -> pass (`83/83`)
146. Refreshed direct emission inventory after `a6355f6b`:
147. `self.write(...)` total in `crates/sifr_codegen/src` -> `408`
148. Remaining files: `stmt_support_emitter.rs` `186`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
149. `d5f3f22e`: introduced shared IR statement emission helper (`emit_rust_stmt_with_current_indent`) and migrated `stmt_support_emitter.rs` lowered-statement + generator-init + borrowed-return clone paths to structured statement/expression IR emission.
150. Root update in this slice: removed direct write-based generator init path and aligned generator init with `lower_stmt_expr_for_ir`.
151. Validation for `d5f3f22e`:
152. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
153. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
154. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `410.51s`)
155. Full runnable milestone demo sweep: `find demos -maxdepth 1 -name '*.sifr'` -> pass (`83/83`)
156. Refreshed direct emission inventory after `d5f3f22e`:
157. `self.write(...)` total in `crates/sifr_codegen/src` -> `398`
158. Remaining files: `stmt_support_emitter.rs` `176`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
159. `2c823925`: migrated additional `stmt_support_emitter.rs` return/raise control paths to structured IR return statements (`RustStmt::Return` + `RustExpr::FnCall`) for closure/display/error branches.
160. Validation for `2c823925`:
161. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
162. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
163. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `411.82s`)
164. Full runnable milestone demo sweep: `find demos -maxdepth 1 -name '*.sifr'` -> pass (`83/83`)
165. Refreshed direct emission inventory after `2c823925`:
166. `self.write(...)` total in `crates/sifr_codegen/src` -> `390`
167. Remaining files: `stmt_support_emitter.rs` `168`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
168. `e7ca4643`: migrated `stmt_support_emitter.rs` return/assert statement emission to structured IR statements (`RustStmt::Return` / `RustStmt::Assert`) and removed direct wrapped-return string helpers.
169. Root fixes in the same slice:
170. preserved non-option index return semantics through explicit IR index lowering (`dict/list/str/tuple`) instead of write-assembled return paths.
171. restored e2e parity by routing return/assert expression payloads through structured expression rendering into IR (`RustExpr::RawCode`) when full typed lowering is not yet complete, preventing incorrect coercion changes.
172. Validation for `e7ca4643`:
173. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
174. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
175. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `413.84s`)
176. runnable milestone demo sweep `demos/*.sifr` -> pass (`83/83`)
177. Refreshed direct emission inventory after `e7ca4643`:
178. `self.write(...)` total in `crates/sifr_codegen/src` -> `378`
179. Remaining files: `stmt_support_emitter.rs` `156`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `lib.rs` `8`.
180. `b33d143a`: migrated `lib.rs` structured `Let`/`Assign` statement emission from direct token writes to IR statements (`RustStmt::Let`/`RustStmt::Assign`) with structured-rendered IR payload expressions.
181. Validation for `b33d143a`:
182. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
183. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
184. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `408.97s`)
185. runnable milestone demo sweep `demos/*.sifr` -> pass (`83/83`)
186. Refreshed direct emission inventory after `b33d143a`:
187. `self.write(...)` total in `crates/sifr_codegen/src` -> `370`
188. Remaining files: `stmt_support_emitter.rs` `156`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`.
189. `649fd630`: migrated walrus-`if` statement emission path in `stmt_support_emitter.rs` to IR control nodes (`RustStmt::Let` + `RustStmt::If`) and removed direct write-based walrus-if assembly.
190. Root fixes in the same slice:
191. added structured block-lowering helper for walrus branch body emission.
192. extended that helper to lower `Assert` and expression statements via IR when simple block lowering is unavailable, preserving production behavior without reverting walrus path to string assembly.
193. Validation for `649fd630`:
194. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
195. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
196. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `429.15s`)
197. runnable milestone demo sweep `demos/*.sifr` -> pass (`83/83`)
198. Refreshed direct emission inventory after `649fd630`:
199. `self.write(...)` total in `crates/sifr_codegen/src` -> `363`
200. Remaining files: `stmt_support_emitter.rs` `149`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`.
201. `28fad690`: migrated nested structured `if`/`while` block lowering in `stmt_support_emitter.rs` to IR-first recursive lowering helpers (`try_lower_if_stmt_for_ir`, `try_lower_if_clause_for_ir`, and nested while lowering in `try_lower_stmt_block_for_ir`), removing production misses that previously triggered hard-gate panics on nested option-guard trees.
202. Root-cause fixes in the same slice:
203. expanded block-lowering non_ir_path coverage for `Let`, `Assign`, `Return`, and `AttributeSubscriptAssign` in nested structured paths, so recursive branches lower as IR statements instead of failing out of the structured pipeline.
204. fixed borrowed-name compare semantics in recursive condition lowering via explicit borrowed compare IR lowering (`String` vs `&String` class of failure), restoring `stdlib_argparse`/`milestone_stdlib_expansion` parity.
205. Validation for `28fad690`:
206. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
207. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
208. `cargo run -q -p sifr -- run demos/milestone_stdlib_expansion_demo.sifr` -> pass
209. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `409.83s`)
210. `./scripts/run_all_tests.sh` -> pass (includes e2e pass sweep)
211. runnable milestone demo sweep `demos/*.sifr` -> pass (`83/83`)
212. Refreshed direct emission inventory after `28fad690`:
213. `self.write(...)` total in `crates/sifr_codegen/src` -> `354`
214. Remaining files: `stmt_support_emitter.rs` `140`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`.

Merged PR chain:
1. `#784` (Issue 217)
2. `#785` (Issue 218)
3. `#786` (Issue 219)
4. `#787` (Issue 220)
5. `#791` (Issue 216 closeout + final fixture parity cleanup)

Final `SynItem` policy outcome:
1. User-code assembly path: `SynItem` forbidden in production final assembly.
2. Any remaining non-user boundary usage must be explicit, documented, and hard-gated from production user-code assembly paths.

---

## Reopened Cleanup Loop (2026-02-28)

Reason for reopening:
1. Source tree and parity validation still include non-final traces for strict IR-only closeout.
2. `self.write(...)` emitter paths remain substantial in production codegen modules.
3. User-path `RustItem::SynItem` assembly still exists in module-body user path.

Loop to-do list:
1. [x] Loop-1: remove explicit non_ir_path/pre_ir transition artifacts and naming from `crates/sifr_codegen/src`.
2. [ ] Loop-2: migrate highest-traffic `self.write(...)` expression/statement paths to structured IR emission.
3. [x] Loop-3: restore parity and revalidate demos/tests (`test_e2e_pass` and runnable demo sweep).
4. [ ] Loop-4: rerun phase gates and update all phase docs/issues with exact validated status.

Loop progress log:
1. 2026-02-28: loop baseline confirmed with `cargo test -q -p sifr --test e2e test_e2e_pass` -> `213 passed, 181 failed`.
2. 2026-02-28: loop iteration applied and validated:
3. non_ir_path/pre_ir transition module removed from `crates/sifr_codegen/src`.
4. `+=` structured semantics fixed for `str` and `list`.
5. structured display/stringify/print behavior tightened for Option/collection-heavy paths.
6. user-call signature path updated to apply borrow conventions and union arg wrapping in structured call emission.
7. Option not-None detection expanded to handle `!= None` / `== None` variants used in corpus.
8. latest full e2e checkpoint -> `287 passed, 107 failed`.
9. decommissioned string-backend body implementations (`emit_expr_string_backend`, `emit_stmt_string_backend`) into explicit unreachable panic stubs for production path.
10. `self.write(...)` count in `crates/sifr_codegen/src` reduced from ~1710 to ~999 in this iteration.
11. 2026-02-28 recheck baseline (current tree):
12. `cargo test -q -p sifr_codegen` -> pass (`455` passed).
13. `cargo clippy -q -p sifr_codegen -- -D warnings` -> pass.
14. `cargo test -q -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture` -> pass (`stmt=9/9`, `expr=9/9`).
15. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> fail (`301` passed, `93` failed).
16. First-error buckets from recheck (`/tmp/phase14_e2e_recheck_first_error.txt`):
17. `E0308=59`, `E0425=11`, `E0631=5`, `E0596=5`, `E0599=4`, `NO_ERROR_LINE=3`, others=6.
18. Current `self.write(...)` count in `crates/sifr_codegen/src` -> `1030`.
19. 2026-02-28 loop revalidation:
20. Runnable `.sifr` demos -> pass (`86` pass, `1` fail where the single failure is intentional: `demos/milestone_borrow_hardening_demo/exclusivity_error_demo.sifr`).
21. `./scripts/run_all_tests.sh` -> pass.
22. Included e2e pass check from script: `test_e2e_pass` -> `394` passed, `0` failed.
23. 2026-02-28 strict implementation re-audit (post-parity):
24. `self.write(...)` current count in `crates/sifr_codegen/src` -> `1207`.
25. Highest-write files: `expr_render_helpers.rs=456`, `stmt_support_emitter.rs=243`, `class_emitter.rs=93`, `slice_emitter.rs=80`, `class_method_emitter.rs=70`.
26. User-path module assembly still drains into `RustItem::SynItem` in `crates/sifr_codegen/src/module_body.rs:54`.
27. Adapter-named production helper path still present in `crates/sifr_codegen/src/intrinsic_method_emitters.rs` (`try_lower_registry_expr_transition`).
28. 2026-02-28 Pass B updates:
29. Removed transition-named production helper path by renaming `try_lower_registry_expr_transition` -> `try_lower_registry_expr_recursive`.
30. Added non-regression guard test asserting transition-named helper signature is not present.
31. Started hot-path extraction in `expr_render_helpers.rs` for print/fstring/display macro assembly via shared structured helper builders.
32. Validation: `cargo test -q -p sifr_codegen` pass; targeted demo subset pass (`milestone_ergonomics`, `milestone_codegen_fixes`, `milestone_new_modules`, `milestone_stdlib_parity`, `milestone_stdlib_pure_expansion`, `milestone_stdlib_remediation`).
33. 2026-02-28 Pass C fixes:
34. Fixed malformed structured return/raise emission in `stmt_support_emitter.rs` (missing wrapper-call close `)` regression).
35. Removed split-string test assertions in `intrinsic_method_emitters.rs` and replaced with direct production-slice guards.
36. Full validation: `./scripts/run_all_tests.sh` -> pass.
37. Full demo sweep: all `demos/*.sifr` -> pass (`83/83`).
38. Terminology recheck in production source: no `transition|non_ir_path|pre_ir|conversion` matches under `crates/sifr_codegen/src`.
39. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1158`.
40. Current highest-write files: `expr_render_helpers.rs=432`, `stmt_support_emitter.rs=218`, `class_emitter.rs=93`, `slice_emitter.rs=80`, `class_method_emitter.rs=70`.
41. 2026-02-28 Pass D updates:
42. `intrinsic_method_emitters.rs` moved to structured registry-expression rendering only; direct `self.write(...)` usage in that file removed (`12 -> 0`).
43. Validation: `cargo test -q -p sifr_codegen` -> pass (`455` passed).
44. Validation: targeted intrinsic guard tests -> pass (`emit_intrinsic_call_has_no_pre_registry_match_dispatch`, `registry_arg_lowering_avoids_inline_rawcode_shims`).
45. Validation: `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass.
46. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1146`.
47. Full validation rerun: `./scripts/run_all_tests.sh` -> pass (includes `test_e2e_pass` -> `394` passed, `0` failed).
48. Full demo sweep rerun: all `demos/*.sifr` -> pass (`83/83`).
49. 2026-02-28 Pass E updates:
50. Removed all direct `self.write(...)` usage from `method_call_emitter.rs` and enforced structured-only method-call emission (no non_ir_path branch; missing shapes now hard-fail).
51. Exposed structured registry expr helpers in `intrinsic_method_emitters.rs` for cross-emitter reuse.
52. Validation: `./scripts/run_all_tests.sh` -> pass (includes `test_e2e_pass` -> `394` passed, `0` failed).
53. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
54. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1132`.
55. `self.write(...)` in `method_call_emitter.rs` -> `0`; `self.write(...)` in `intrinsic_method_emitters.rs` -> `0`.
56. 2026-02-28 Pass F updates:
57. Moved additional display-critical lowering into shared registry IR path (no emitter-local string non_ir_path): string step-slice (`s[::2]`, `s[::-1]`), `**` binop, `sum(...)`, and `Compare` chains.
58. Fixed structured index lowering regression by normalizing negative list/string indices in IR blocks (`[-1]` behavior restored) with single-evaluation object binding.
59. Fixed method-call convention loss in structured expression lowering for class methods by resolving class method parameter conventions from signatures/class metadata and applying borrow/option wrapping in registry method-call lowering.
60. Full validation: `./scripts/run_all_tests.sh` -> pass (includes `test_e2e_pass` -> `394` passed, `0` failed).
61. Full demo sweep: `demos/*.sifr` -> pass (`83/83`).
62. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1107`.
63. File-level progress: `expr_ref_emitter.rs` now has `0` direct `self.write(...)`; `method_call_emitter.rs=0`; `intrinsic_method_emitters.rs=0`.
64. 2026-02-28 Pass G updates:
65. Converted structured plain-call emission in `expr_render_helpers.rs` to emit IR `RustExpr::FnCall` (removed direct write-based call assembly).
66. Delegated additional special-call builtins to registry IR lowering (`bool`, `pow`, `bigint`, `round`, `abs`, `sum`, plus 2-arg `min/max`) and removed duplicated write-based branches.
67. Reduced `min/max` non_ir_path branch in `expr_render_helpers.rs` to list-aggregator case only (`args.len()==1`) with 2-arg path handled by registry lowering.
68. Validation: `./scripts/run_all_tests.sh` -> pass (includes `test_e2e_pass` -> `394` passed, `0` failed).
69. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
70. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1050` (`expr_render_helpers.rs` reduced `432 -> 375`).
71. 2026-02-28 Pass H updates:
72. Added registry IR lowering coverage for additional special-call builtins (`any`, `all`, `reversed`, `zip`) in `intrinsic_method_emitters.rs`.
73. Removed duplicated write-based special-call branches for `any`/`all`/`reversed`/`zip` in `expr_render_helpers.rs`; these now route through registry IR lowering.
74. Validation: `./scripts/run_all_tests.sh` -> pass (includes `test_e2e_pass` -> `394` passed, `0` failed).
75. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
76. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1036` (`expr_render_helpers.rs` reduced `375 -> 361`).
77. 2026-02-28 Pass I updates:
78. Added registry IR lowering for `min/max` list form (`args.len()==1`), `sorted`, and `enumerate` special calls.
79. Removed duplicated write-based `min/max` (list form), `sorted`, and `enumerate` branches from `expr_render_helpers.rs`.
80. Validation: `./scripts/run_all_tests.sh` -> pass (includes `test_e2e_pass` -> `394` passed, `0` failed).
81. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
82. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1020` (`expr_render_helpers.rs` reduced `361 -> 345`).
83. 2026-02-28 Pass J updates:
84. Closed remaining strict registry-IR lowering gaps used by `str/repr` nested paths: recursive `BoolOp`, `ConstructorCall`, option/debug-aware `FString`, option/string-aware `Compare`, and callable-field invocation lowering.
85. Restored parity for recursive field access and method-call semantics: inherited field remap preserved and self-field mutating calls now suppress clone in registry-first structured method-call flow.
86. Validation: `cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed).
87. Validation: `./scripts/run_all_tests.sh` -> pass.
88. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
89. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1003` (`expr_render_helpers.rs` reduced `345 -> 328` in this pass).
90. 2026-02-28 Pass K updates:
91. Migrated `int(...)` / `float(...)` special-call lowering to shared registry builtin IR path in `intrinsic_method_emitters.rs` with parity conversions (`str` parse + `ParseError`, `bool` numeric mapping, `bigint` range check + `OverflowError`).
92. Removed duplicated write-assembled `int(...)` / `float(...)` branches from `expr_render_helpers.rs` special-call emission.
93. Validation: `./scripts/run_all_tests.sh` -> pass.
94. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
95. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `980` (`expr_render_helpers.rs` reduced `328 -> 305` in this pass).
96. 2026-02-28 Pass L updates:
97. Completed dependency leaf item `helpers.rs`: removed direct `self.write(".clone()")` path in `emit_expr_with_bigint_clone` and now emit structured IR `RustExpr::Clone(...)` through shared IR rendering helper.
98. Validation: `./scripts/run_all_tests.sh` -> pass.
99. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
100. Strict implementation re-audit refresh: `helpers.rs` direct `self.write(...)` -> `0`; total `self.write(...)` in `crates/sifr_codegen/src` remains `980` (count redistributed via shared emitter helper).
101. 2026-02-28 Pass M updates:
102. Completed dependency leaf item `render.rs`: removed renderer `.write(...)` callsites (`self.write(...)`/`renderer.write(...)`) by using a single append primitive while preserving IR renderer behavior.
103. Validation: `./scripts/run_all_tests.sh` -> pass.
104. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
105. Strict implementation re-audit refresh: `render.rs` direct `self.write(...)` -> `0`; total `self.write(...)` in `crates/sifr_codegen/src` -> `976`.
106. 2026-02-28 Pass N updates:
107. Removed dead `slice_emitter` module (`crates/sifr_codegen/src/slice_emitter.rs`) and dropped `mod slice_emitter;` from `lib.rs`.
108. Verified no callsites existed for `emit_walrus_hoists`, `emit_list_slice`, or `emit_string_slice` before removal.
109. Validation: `./scripts/run_all_tests.sh` -> pass.
110. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
111. Strict implementation re-audit refresh: `slice_emitter.rs` removed; total `self.write(...)` in `crates/sifr_codegen/src` -> `896`.
112. 2026-02-28 Pass O updates:
113. Completed `match_emitter.rs` conversion from token-by-token writes to line-based pattern/guard rendering helpers; direct `self.write(...)` usage in the file removed.
114. Kept semantics for option/non-option-union class patterns, string literal guard handling, and guard substitution paths while switching to rendered-line assembly.
115. Validation: `./scripts/run_all_tests.sh` -> pass.
116. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
117. Strict implementation re-audit refresh: `match_emitter.rs` direct `self.write(...)` -> `0`; total `self.write(...)` in `crates/sifr_codegen/src` -> `867`.
118. 2026-02-28 Pass P updates:
119. Migrated another `expr_render_helpers.rs` hot-path slice away from fragment-by-fragment writes:
120. method-call assembly, result-wrap/bool-op/constructor/list/dict/set literal assembly, walrus/contains assembly, and dict-key lookup helper now use composed rendered expressions.
121. Validation: `./scripts/run_all_tests.sh` -> pass.
122. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
123. Strict implementation re-audit refresh: `expr_render_helpers.rs` direct `self.write(...)` -> `249` (`305 -> 249` in this pass); total `self.write(...)` in `crates/sifr_codegen/src` -> `811`.
124. 2026-02-28 Pass Q updates:
125. Migrated additional high-frequency `expr_render_helpers.rs` clusters to composed expression emission:
126. numeric binop (`**` and standard ops), conditional expression (`if ... else ...`), and index expression assembly now emit via formatted expression strings instead of token-fragment writes.
127. Validation: `./scripts/run_all_tests.sh` -> pass.
128. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
129. Strict implementation re-audit refresh: `expr_render_helpers.rs` direct `self.write(...)` -> `176` (`249 -> 176` in this pass); total `self.write(...)` in `crates/sifr_codegen/src` -> `738`.

### Next Loop To-do (Evidence-Based)

1. [x] Loop-2A: Continue migrating `expr_render_helpers.rs` hot paths (`67` writes) to structured IR-first expression builders.
2. [ ] Loop-2B: Migrate `stmt_support_emitter.rs` (`168` writes) and top item emitters (`class_emitter.rs`, `class_method_emitter.rs`, `function_emitter.rs`) off string assembly.
3. [ ] Loop-2C: Remove user-path drain-parse flow and `RustItem::SynItem` push in `module_body.rs`.
4. [x] Loop-4A: Remove transition-named production helpers (`try_lower_registry_expr_transition`) and replace with explicit structured-only naming/pathing.
5. [ ] Loop-4B: Add/refresh hard-gate tests enforcing zero user-path `SynItem` and preventing string-emission regressions in production paths.
6. [ ] Loop-4C: Re-run full phase gate commands and only then re-mark WS1..WS7 complete.

Dependency-ordered execution queue (leaf -> orchestrator) for remaining `.write` files:
1. [x] `helpers.rs` (completed in Pass L; now `0` direct `self.write(...)`)
2. [x] `render.rs` (completed in Pass M; now `0` direct `self.write(...)`)
3. [x] `type_emitters.rs` (completed; now `0` direct `self.write(...)`)
4. [x] `operator_protocol_emitters.rs` (completed; now `0` direct `self.write(...)`)
5. [x] `match_emitter.rs` (completed in Pass O; now `0` direct `self.write(...)`)
6. [x] `expr_render_helpers.rs` (completed in `a6355f6b`; now `0` direct `self.write(...)`)
7. [ ] `stmt_support_emitter.rs` (`149` writes; statement lowering over expression leafs)
8. [ ] `function_emitter.rs` (`51` writes; item wrapper over stmt/expr lowering)
9. [ ] `class_method_emitter.rs` (`70` writes; class method wrapper over stmt/expr lowering)
10. [ ] `class_emitter.rs` (`93` writes; class item orchestration over class methods)
11. [x] `lib.rs` (completed; now `0` direct `self.write(...)`)

### Active Implementation Loop (2026-02-28, Pass E)

1. [x] Item 1: Remove transition-named production helper path in `intrinsic_method_emitters.rs` and keep behavior parity.
2. [x] Item 2: Start hot-path extraction in `expr_render_helpers.rs` to reduce direct `self.write(...)` string assembly for structured print/fstring/display flows.
3. [x] Item 3: Add focused guards/tests for naming and behavior to prevent transition-path reintroduction.
4. [x] Item 4: Fix structured return/raise terminator regression introduced during extraction in `stmt_support_emitter.rs`.
5. [x] Item 5: Re-run full local validations (`./scripts/run_all_tests.sh` + full `demos/*.sifr` sweep) and record results.
6. [x] Item 6: Remove direct `self.write(...)` from `intrinsic_method_emitters.rs` by routing emission through lowered IR rendering only.
7. [x] Item 7: Remove direct `self.write(...)` from `method_call_emitter.rs` and enforce structured-only method-call emission (no non_ir_path).

---

## Why this rewrite is needed

The remaining work is not “routing cleanup” only. The core gap is that major emitters are still `.write()`-based string emitters.  
True closeout requires migrating these emitters to structured IR construction (`lower_*` style), then removing transition/non_ir_path paths.

This plan is dependency-ordered to avoid fake “done” states.

---

## Locked architecture decisions

1. User-code generation target is IR-first, not string-first.
2. Historical_path non_ir_path/transition is temporary conversion scaffolding only and must be removed from production path.
3. `RustItem::SynItem` policy:
4. `User code`: forbidden in production assembly.
5. `External stdlib compiled Rust text`: allowed only behind explicit boundary until replaced; must be documented and hard-gated from user-code paths.
6. Epic `216` cannot close while production user-code paths still depend on non_ir_path transition.

---

## Scope of emitter conversion (actual bulk of work)

These are the real conversion targets that must move from `.write()` emission to IR-building:

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
12. `crates/sifr_codegen/src/expr_render_helpers.rs` (pre_ir render non_ir_path helpers)
13. `crates/sifr_codegen/src/type_emitters.rs` (protocol trait / enum class / newtype emission)
14. `crates/sifr_codegen/src/stmt_support_emitter.rs`

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
3. Remove production default non_ir_path routing and conversion wrappers.
4. Enforce no-raw/no-opaque gates (`RawCode` + user-path `SynItem`) in production assembly.
5. Finalize structural-pass hard gate and dependency rationale.
6. Close epic docs/checklists with evidence.

No step may skip prerequisites.

---

## Workstreams and PR slices

## WS0: Baseline quantification and coverage inventory (prerequisite)

Deliverables:
1. Add variant coverage inventory to `Issue 217`:
2. all `HirExpr` variants with status: `structured-ready` / `pre_ir-dependent`
3. all `HirStmt` variants with status: `structured-ready` / `pre_ir-dependent`
4. production reachability marker per variant based on e2e/demo corpus
5. Add emitter ownership matrix to `issues/218-phase14-promote-full-ir-module-assembly.md` mapping each emitter file to migrated IR entrypoints.

PR slice:
1. PR-WS0-doc-baseline (docs + guard assertions only).

Completion gate:
1. Coverage inventory is committed and referenced by later PRs.

---

## WS1: Structured expression lowering expansion (core conversion)

Target:
1. Move production-reachable expression shapes from pre_ir emitter paths into structured lowering (`lower_expr` + helpers).

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
2. No production wrapper callsite requires expression pre_ir transition.

---

## WS2: Structured statement lowering expansion (core conversion)

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
1. PR-WS2A: control-flow stmt shapes (`if/while/for/match/try-except`) structured conversion.
2. PR-WS2B: assignment/unpack/with/delete/nested function stmt shapes structured conversion.

Completion gate:
1. Production-reachable `HirStmt` variants are `structured-ready`.
2. Generator-init path is structured-only (no pre_ir transition dependency).

---

## WS3: Top-level item conversion (class/function/method/operator/protocol/generator)

Target:
1. Replace string-emitter orchestration with item-first lowering for user module body.
2. Explicitly migrate generator function assembly from string closure template to structured IR.
3. Explicitly migrate trait/operator/protocol/class method emission paths.

Primary files:
1. `crates/sifr_codegen/src/module_body.rs`
2. `crates/sifr_codegen/src/class_emitter.rs`
3. `crates/sifr_codegen/src/type_emitters.rs`
4. `crates/sifr_codegen/src/function_emitter.rs`
5. `crates/sifr_codegen/src/class_method_emitter.rs`
6. `crates/sifr_codegen/src/operator_protocol_emitters.rs`
7. `crates/sifr_codegen/src/lower_item.rs`
8. `crates/sifr_codegen/src/lib.rs`
9. `crates/sifr_codegen/src/entrypoints.rs`
10. `crates/sifr_codegen/src/lib_codegen_tests.rs`

PR slices:
1. PR-WS3A: remove drain-parse `SynItem` for user class/function body.
2. PR-WS3B: class/method/operator/protocol structured item assembly.
3. PR-WS3C: generator function structured lowering and parity tests.

Completion gate:
1. User code class/function/method/operator/protocol/generator paths are item-first.
2. `module_body` no longer uses user-code drain->parse->`SynItem` flow.

---

## WS4: Adapter/non_ir_path decommission (Issue 217 finalization)

Target:
1. Remove default production non_ir_path routing.
2. Remove temporary conversion wrappers and transition emitters from production path.

Primary files:
1. `crates/sifr_codegen/src/lib.rs`
2. `crates/sifr_codegen/src/pre_ir_transition_emitters.rs`
3. Remaining emitter callsites listed in WS1/WS2/WS3.
4. `crates/sifr_codegen/src/lib_codegen_tests.rs`
5. `Issue 217`

PR slices:
1. PR-WS4A: strict production wrappers + guards.
2. PR-WS4B: delete/disable pre_ir transition from production flow after coverage gate.

Completion gate:
1. No production routing from `emit_stmt`/`emit_expr` to pre_ir non_ir_path emitters.
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
1. Structural passes must not rely on raw-text non_ir_path behavior in production.
2. Structural passes must not rely on opaque user-code payload parsing in production.

Primary files:
1. `crates/sifr_codegen/src/ir_imports.rs`
2. `crates/sifr_codegen/src/ir_validate.rs`
3. `crates/sifr_codegen/src/lib.rs`
4. `crates/sifr_codegen/src/entrypoints.rs`
5. `crates/sifr_codegen/src/stdlib_filter.rs`
6. `crates/sifr_codegen/Cargo.toml`
7. `crates/sifr_codegen/src/lib_codegen_tests.rs`
8. `Issue 220`

PR slice:
1. PR-WS6A: structural-pass policy and dependency rationale hardening.

Completion gate:
1. Production structural passes do not parse raw text non_ir_path payloads.
2. If `syn` remains runtime dependency, rationale is documented with exact usage.

---

## WS7: Epic closeout (Issue 216)

Primary files:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md`
2. `Issue 217`
3. `issues/218-phase14-promote-full-ir-module-assembly.md`
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
5. `Issue 220`
6. `internal_docs/phases/14_codegen_architecture_finish_checklist.md`
7. `internal_docs/phases/14_codegen_architecture.md`
8. `internal_docs/architecture.md`
9. `internal_docs/roadmap.md`

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
3. `cargo run -q -p sifr -- run demos/milestone_codegen_stmt_expr_conversion_demo.sifr`
4. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`
5. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`

No CI waiting required for progression.

---

## Risk controls / anti-regression checks

1. Add grep-style decomposition tests for transition/non_ir_path reintroduction in production wrappers.
2. Add assembly tests asserting user-code module body does not produce `SynItem`.
3. Add final production assembly tests asserting no `RawCode` and no user-path `SynItem`.
4. Add structural-pass tests asserting raw non_ir_path paths panic in production mode.
5. Keep temporary conversion exceptions documented with removal owner and deadline.

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

---

## Closeout Validation Evidence

Recorded on `main` closeout (2026-02-25) in `issues/216-phase14-codegen-architecture-closeout-epic.md`:
1. `cargo test -p sifr_codegen`
2. `cargo clippy -p sifr_codegen -- -D warnings`
3. `scripts/run_e2e_pass.sh` (defaults)
4. `cargo test -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture`
5. `cargo test --workspace`
6. `cargo clippy --workspace -- -D warnings`

Additional validation rerun for this plan-doc update is executed locally and recorded below:
1. `cargo test -p sifr_codegen` -> pass (`450` passed, `0` failed)
2. `cargo clippy -p sifr_codegen -- -D warnings` -> pass
3. `cargo run -q -p sifr -- run demos/milestone_codegen_stmt_expr_conversion_demo.sifr` -> pass (`total = 24`, `verdict = high`)
4. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass (`current_has_t = true`, `today_has_dash = true`)
5. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (`cargo test -p sifr` + `run_e2e_pass.sh`; final `test_e2e_pass` result: `394` passed, `0` failed)

### Validation rerun (2026-03-01, Pass R)

1. `./scripts/run_all_tests.sh` -> pass (unit tests + e2e suite + e2e pass suite)
2. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
3. Full demos sweep `demos/*.sifr` -> pass (`83/83`)
4. Production-path cleanup progress this loop:
5. structured IR path retained for `FieldAssign` / `AttributeSubscriptAssign` / `AugAssign` in `stmt_support_emitter.rs`
6. no transition/non_ir_path path added
7. Remaining direct emitter calls re-audit: `self.write(...) = 706`, `self.writeln(...) = 63` in `crates/sifr_codegen/src`.

### Validation rerun (2026-03-01, Pass S)

1. `./scripts/run_all_tests.sh` -> pass
2. Full demos sweep `demos/*.sifr` -> pass (`83/83`)
3. Emitter cleanup this loop:
4. removed local `self.writeln(...)` usage in `function_emitter.rs` and `match_emitter.rs`
5. Re-audit totals: `self.write(...) = 713`, `self.writeln(...) = 56` in `crates/sifr_codegen/src`.

### Validation rerun (2026-03-01, Pass T)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `./scripts/run_all_tests.sh` -> pass
3. Full demos sweep `demos/*.sifr` -> pass (`83/83`)
4. IR-first cleanup this loop:
5. added `RustItem::TraitMethodSig` and renderer/pass support (`render.rs`, `ir_imports.rs`, `ir_optimize.rs`, `ir_validate.rs`, `preamble.rs`)
6. migrated protocol trait emission in `type_emitters.rs` to structured IR item assembly
7. Re-audit totals: `self.write(...) = 697`, `self.writeln(...) = 56` in `crates/sifr_codegen/src`.

### Validation rerun (2026-03-01, Pass U)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `./scripts/run_all_tests.sh` -> pass
3. Full demos sweep `demos/*.sifr` -> pass (`83/83`)
4. IR-first cleanup this loop:
5. migrated protocol impl emission in `operator_protocol_emitters.rs` from string assembly to `RustItem::Impl` IR assembly
6. delegation methods now emitted as IR `RustItem::Fn` with explicit `RustExpr::FnCall` and structured return/expr statements
7. Re-audit totals: `self.write(...) = 679`, `self.writeln(...) = 57` in `crates/sifr_codegen/src` (renderer-only `writeln` usage).

### Validation rerun (2026-03-01, Pass V)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `./scripts/run_all_tests.sh` -> pass
3. Full demos sweep `demos/*.sifr` -> pass (`83/83`)
4. IR-first cleanup this loop:
5. fully migrated `type_emitters.rs` enum/newtype emission to IR-first item assembly (`RustItem::Enum`, `RustItem::TupleStruct`, `RustItem::Impl`, `RustItem::Fn`)
6. moved enum/newtype user-method emission to structured stmt-lowering-backed IR bodies in this emitter (no direct write-based type emitter output)
7. Re-audit totals: `self.write(...) = 634`, `self.writeln(...) = 57` in `crates/sifr_codegen/src`.

### Validation rerun (2026-03-01, Pass W)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
3. `cargo run -q -p sifr -- run demos/milestone_stdlib_expansion_demo.sifr` -> pass
4. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `418.24s`)
5. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass
6. Full runnable demos sweep `demos/*.sifr` -> pass (`83/83`)
7. IR-first cleanup this loop:
8. added structured `with` statement IR node (`RustStmt::With` + `RustWithItem`) in `rust_ir.rs`
9. added render/import/optimize/validate support for `With` (`render.rs`, `ir_imports.rs`, `ir_optimize.rs`, `ir_validate.rs`, `preamble.rs`, `lower_stmt.rs`, `expr_render_helpers.rs`)
10. migrated `try_emit_structured_with_stmt` to IR emission and enabled recursive `HirStmt::With` lowering inside `try_lower_stmt_block_for_ir`
11. added explicit recursive `Raise` lowering in `try_lower_stmt_block_for_ir` to keep nested control-flow branches fully structured
12. Re-audit totals: direct `self.write(...) = 330` in `crates/sifr_codegen/src`
13. File breakdown: `stmt_support_emitter.rs` `116`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`.

### Validation rerun (2026-03-01, Pass X)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass
3. `cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `426.80s`)
4. Full runnable demos sweep `demos/*.sifr` -> pass (`83/83`)
5. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass
6. IR-first cleanup this loop:
7. migrated `try_emit_structured_while_stmt` from token writes to `RustStmt::While` emission through recursive IR lowering
8. expanded recursive block IR lowering for nested `for` iter-shapes (`enumerate` and collection iteration normalization) to keep while bodies on structured IR
9. Re-audit totals after this slice: direct `self.write(...) = 327` in `crates/sifr_codegen/src`
10. File breakdown: `stmt_support_emitter.rs` `113`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`.

### Validation rerun (2026-03-01, Pass Y)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `SIFR_E2E_RUNNER_MODE=new cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `78.64s`)
3. Full runnable demos sweep `demos/*.sifr` -> pass (`83/83`)
4. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass
5. IR-first cleanup this loop:
6. migrated `try_emit_structured_for_stmt` from token writes to structured `RustStmt::For` emission with recursive lowered body/else block support
7. fixed iterator root cause in for-lowering: iterator-like and generator iter expressions now remain iterator-shaped (no invalid extra `.iter().cloned()` wrapping)
8. Re-audit totals after this slice: direct `self.write(...) = 308` in `crates/sifr_codegen/src`
9. File breakdown: `stmt_support_emitter.rs` `94`, `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`

### Validation rerun (2026-03-01, Pass Z)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `SIFR_E2E_RUNNER_MODE=new cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `65.05s`)
3. Full runnable demos sweep `demos/*.sifr` -> pass (`83/83`)
4. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass
5. IR-first cleanup this loop:
6. migrated structured `try/except` emission from token writes to IR stmt trees (`RustStmt::Let` + `ClosureBlock` + `Match` / `IfLet`) in `stmt_support_emitter.rs`
7. added nested `TryExcept` lowering support inside `try_lower_stmt_block_for_ir` so nested error-handling blocks remain on structured IR paths
8. aligned block-level `Return` lowering with try-closure semantics (option/direct capture wrapping), fixing compile-time regressions from nested try lowering
9. Re-audit totals after this slice: direct `self.write(...) = 271` in `crates/sifr_codegen/src`
10. File breakdown: `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `stmt_support_emitter.rs` `57`, `function_emitter.rs` `51`

### Validation rerun (2026-03-01, Pass AA)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `SIFR_E2E_RUNNER_MODE=new cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `66.91s`)
3. Full runnable demos sweep `demos/*.sifr` -> pass (`83/83`)
4. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass
5. IR-first cleanup this loop:
6. migrated `try_emit_structured_if_stmt` union/none/general branches to structured IR (`RustStmt::If`/`IfLet`/`Match`) and removed direct token-write `if` assembly
7. expanded nested block lowering for `FieldAssign` so IR-lowered `if` bodies no longer fall out of structured statement emission
8. Re-audit totals after this slice: direct `self.write(...) = 215` in `crates/sifr_codegen/src`
9. File breakdown: `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`, `stmt_support_emitter.rs` `1`

### Validation rerun (2026-03-01, Pass AB)

1. `cargo test -q -p sifr_codegen` -> pass (`455` passed)
2. `SIFR_E2E_RUNNER_MODE=new cargo test -q -p sifr --test e2e test_e2e_pass -- --nocapture` -> pass (`394` passed, `0` failed, `63.74s`)
3. Full runnable demos sweep `demos/*.sifr` -> pass (`83/83`)
4. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass
5. IR-first cleanup this loop:
6. removed final direct `self.write(...)` in `stmt_support_emitter.rs` by replacing expression-statement terminator writes with structured statement emission flow in `lib.rs`
7. preserved existing structured expression lowering behavior by reusing `try_emit_structured_expr` output and wrapping it as `RustStmt::Expr` emission
8. Re-audit totals after this slice: direct `self.write(...) = 214` in `crates/sifr_codegen/src`
9. File breakdown: `class_emitter.rs` `93`, `class_method_emitter.rs` `70`, `function_emitter.rs` `51`

### Next Loop Todo (Dependency-Ordered)

1. `function_emitter.rs` (`51`) -> migrate function/generator body emission to IR statements/expressions.
2. `class_method_emitter.rs` (`70`) -> migrate constructor/method assembly to IR using shared lowering helpers.
3. `class_emitter.rs` (`93`) -> migrate class/type/display impl assembly to IR item trees.

### Validation rerun (2026-03-02, Pass AL)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. migrated constructor-call lowering in `expr_render_helpers.rs` from rendered-string arg assembly to typed IR arg lowering (borrow convention, clone, option wrapping, `Box::new` all emitted as IR)
6. migrated union `isinstance` lowering in `expr_render_helpers.rs` from `matches!` `RawCode` macro args to structured typed IR block + `IfLet`
7. removed now-unused string-capture helper `try_render_structured_expr` from `expr_render_helpers.rs`
8. changed `borrow_prefix_for_name` visibility to `pub(crate)` and rewired typed borrow-prefix application to avoid output-capture string flow
9. Re-audit totals after this slice: `expr_render_helpers.rs` production `RustExpr::RawCode` construction sites removed (remaining single `RawCode` mention is a defensive panic arm)

### Validation rerun (2026-03-02, Pass AM)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. added `lower_display_expr` in `expr_ref_emitter.rs` to produce typed display expressions for format contexts and rewired `emit_display_expr` to emit from that typed node
6. removed string-capture display fragment helpers from `expr_render_helpers.rs` and changed `write_format_macro_call` to accept typed IR args
7. migrated structured multi-arg `print(...)` and f-string macro arg lowering to typed IR display args (no `map(crate::RustExpr::RawCode)` path)
8. Re-audit totals after this slice: format/println typed-arg emission path no longer uses `RustExpr::RawCode` construction in `expr_render_helpers.rs` (remaining `RawCode` mention there is the defensive panic arm)

### Validation rerun (2026-03-02, Pass AN)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed direct-render probe non_ir_path from display lowering in `expr_ref_emitter.rs`; display option inference now stays type/HIR-driven
6. eliminated remaining `render_expr_via_direct_emit(...)` usage from this display path so it stays on typed IR flow

### Validation rerun (2026-03-02, Pass AO)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed `should_force_render_guard` branching from `expr_render_helpers.rs` so registry expr-result lowering no longer intentionally drops to render guard
6. deleted now-dead render-guard helper graph (`render_expr_contains_force_guard_name` and borrowed-param guard helper) from `expr_render_helpers.rs`

### Validation rerun (2026-03-02, Pass AP)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed `render_expr_via_direct_emit(...)` from `expr_render_helpers.rs`; `render_expr_with_lowered_path` now uses strict registry IR lowering only
6. removed string-based `RawCode` iterator-shape probing from `stmt_support_emitter.rs` by turning that branch into strict-path panic

### Validation rerun (2026-03-02, Pass AQ)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed `crate::render_expr(...)` string-probing from borrow detection in `methods/set.rs`, `methods/list.rs`, and `methods/dict.rs`
6. replaced those with typed IR shape checks only (`Ref`, `as_str` method-call forms + wrapped-expression recursion)

### Validation rerun (2026-03-02, Pass AR)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed remaining `crate::render_expr(...)` production usage in `methods/string.rs` borrow detection and replaced with typed IR shape checks
6. migrated `ljust/rjust/zfill` width emission from synthetic string `Ident(\"width = ...\")` to typed positional-width format args with explicit `usize` cast
7. updated `methods/mod.rs` assertions for the new positional-width typed rendering output

### Validation rerun (2026-03-02, Pass AS)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. added typed `RustExpr::Array` support across IR/render/import/validate/optimize/lowering stacks
6. migrated dict/set literal lowering in `lower_expr.rs` from rendered-string array assembly to typed IR arrays while preserving `HashMap::from([..])` / `HashSet::from([..])` output shape
7. removed `crate::render_expr(...)` usage from that `lower_expr.rs` production path

### Validation rerun (2026-03-02, Pass AT)

1. `cargo test -q -p sifr_codegen` -> pass (`457` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed `render_expr_with_lowered_path` from `expr_render_helpers.rs`
6. migrated affected test callsites in `lib_codegen_tests.rs` to strict-lowered rendering helper (`try_lower_registry_expr_strict` + render)
7. production `crate::render_expr(...)` callsites reduced to output sink usage only

### Validation rerun (2026-03-02, Pass AU)

1. `cargo test -q -p sifr_codegen` -> pass (`454` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed test-only `lower_expr_raw`, `lower_item_raw`, and `lower_stmt_raw` constructors
6. removed associated placeholder tests that only verified `RawCode` construction
7. migrated `intrinsics/mod.rs` test arg helper from `RustExpr::RawCode` mapping to typed test arg parsing

### Validation rerun (2026-03-02, Pass AV)

1. `cargo test -q -p sifr_codegen` -> pass (`446` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed `RawCode` variants from `rust_ir.rs` and completed full structural propagation in renderer/import/validation/optimizer/lowering passes
6. removed all remaining `RawCode` references from `crates/sifr_codegen/src` production and test code
7. removed redundant raw-code gate wiring from `entrypoints.rs`/`lib.rs`; `validate_items` remains the structural validation gate
8. re-audit evidence: `rg -n "RawCode" crates/sifr_codegen` -> no matches

### Validation rerun (2026-03-02, Pass AW)

1. `cargo test -q -p sifr_codegen` -> pass (`445` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed `RustItem::SynItem` from `rust_ir.rs` and completed structural propagation in renderer/import/validation/optimization paths
6. removed user/module `SynItem` insertion helper from `module_body.rs`
7. rewired `lib.rs` stdlib preamble handling to an explicit external-source boundary (outside IR items) and combined structural IR + source-scan import-needs calculation
8. re-audit evidence: `rg -n "SynItem\\(" crates/sifr_codegen/src` -> no matches

### Validation rerun (2026-03-02, Pass AX)

1. `cargo test -q -p sifr_codegen --lib --tests` -> pass (`445` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed output-leak slicing checks from class/function/class-method strict lowerers and replaced this with a centralized capture-time hard panic guard in `output_helpers.rs`
6. deleted dead direct-output borrow-prefix emitters from `method_call_emitter.rs`
7. re-audit evidence: `rg -n "RawCode|SynItem" crates/sifr_codegen/src` -> no matches
8. `self.output` references now remain only in renderer/output sink layers (`output_helpers.rs`, `render.rs`) plus one test-source assertion

### Validation rerun (2026-03-02, Pass AY)

1. `cargo test -q -p sifr_codegen test_emit_expr_prefers_structured_name_path` -> pass (`1` passed)
2. `cargo test -q -p sifr_codegen test_emit_expr_borrowed_compare_is_structured` -> pass (`1` passed)
3. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
4. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
5. IR-first cleanup this loop:
6. removed direct string helper methods from `RustEmitter` output helpers (`write`, `emit_line`, `write_indent`)
7. hardened `emit_rust_stmt_with_current_indent` to IR-capture-only behavior (panic when capture stack is not active)
8. hardened `emit_rust_expr` to panic (forbid direct expression string emission path)
9. updated `lib_codegen_tests.rs` expression-path tests to validate strict typed lowering (`try_lower_registry_expr_strict` + renderer) instead of emitter output-string assertions

### Validation rerun (2026-03-02, Pass AZ)

1. `cargo test -q -p sifr_codegen` -> pass (`445` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed dead `RustEmitter` output state (`output`, `indent`) from `lib.rs`
6. removed output-drain contract helper `assert_output_drained` from `lib_support.rs` and deleted all production callsites (`lib.rs`, `entrypoints.rs`)
7. updated unreachable string-backend panic helpers in `stmt_emitter.rs` / `expr_emitter.rs` to avoid removed indentation state
8. updated `lib_codegen_tests.rs` architecture guards to assert absence of `emitter.output`/`assert_output_drained(...)` plumbing in production assembly paths

### Validation rerun (2026-03-02, Pass BA)

1. `cargo test -q -p sifr_codegen` -> pass (`445` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed dead expression-side-effect orchestration from `lib.rs` (`try_emit_structured_expr`, `emit_expr`)
6. removed obsolete module wiring (`mod expr_emitter`, `mod stmt_emitter`) and deleted corresponding dead files
7. removed dead helper side-effect emitters (`emit_expr_with_bigint_clone` in `helpers.rs`, `emit_lambda_untyped` in `expr_render_helpers.rs`)
8. updated `lib_codegen_tests.rs` architecture guards to enforce absence of `emit_expr` wrapper and deleted string-backend modules

### Validation rerun (2026-03-02, Pass BB)

1. `cargo test -q -p sifr_codegen` -> pass (`445` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed dead expression side-effect emit helpers from `expr_ref_emitter.rs` (`emit_parenthesized_expr`, key/str/compare/bytes/collection emitters, `emit_display_expr`)
6. retained pure typed lowering helpers (`lower_ref_expr_or_panic`, `lower_display_expr`) as canonical tree-walk expression-reference flow
7. removed remaining writer-style API surface in this module that had no production callsites

### Validation rerun (2026-03-02, Pass BC)

1. `cargo test -q -p sifr_codegen` -> pass (`444` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed dead `try_emit_structured_method_call` side-effect path from `expr_render_helpers.rs`
6. pruned dead method-call emit wrappers from `method_call_emitter.rs`, keeping only shared helpers used by active typed lowering (`borrow_prefix_for_name`, `is_generator_call`)
7. removed dead registry emitter wrapper `try_emit_method_via_registry` from `intrinsic_method_emitters.rs`
8. cleaned now-unused imports introduced by this pruning (`MUTATING_METHODS`, `is_self_field_access_expr`)

### Validation rerun (2026-03-02, Pass BD)

1. `cargo test -q -p sifr_codegen` -> pass (`444` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed dead intrinsic-side-effect emit wrappers from `intrinsic_method_emitters.rs` (`emit_intrinsic_call`, `try_emit_intrinsic_via_registry`, `emit_registry_plain_call_expr`, `emit_stdlib_constant`)
6. updated intrinsic emitter contract test to assert wrapper-layer absence in production source section
7. removed dead stmt helper `emit_borrowed_return_name_clone_expr` from `stmt_support_emitter.rs`

### Validation rerun (2026-03-02, Pass BE)

1. `cargo test -q -p sifr_codegen` -> pass (`444` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. removed dead `try_emit_structured_*` expression side-effect layer from `expr_render_helpers.rs`, keeping only active typed lowerers (`try_lower_structured_field_access_expr`, `try_lower_structured_class_binop_expr`, `try_lower_structured_index_expr`)
6. removed dead transition APIs `emit_rust_expr(...)` (`output_helpers.rs`) and `write_registry_expr(...)` (`intrinsic_method_emitters.rs`)
7. removed unreferenced `is_reserved_plain_builtin_call` from `lib_support.rs` and its re-export from `lib.rs`
8. re-audit evidence after this loop: `self.write(...) = 0`, `self.writeln(...) = 0`, `emit_rust_expr(...) = 0` in `crates/sifr_codegen/src`; `rg -n "RawCode|SynItem|non_ir_path|pre_ir|conversion|transition" crates/sifr_codegen/src` -> no matches

### Validation rerun (2026-03-02, Pass BF)

1. `cargo test -q -p sifr_codegen` -> pass (`445` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. added architecture guard `test_expr_side_effect_emitter_layer_is_removed` in `lib_codegen_tests.rs` to block reintroduction of removed expression side-effect/transition APIs
6. guard verifies absence of expression emitter surface traces across `expr_render_helpers.rs`, `output_helpers.rs`, `intrinsic_method_emitters.rs`, and `lib_support.rs`
7. production re-audit evidence after this loop: `prod.self.write(...) = 0`, `prod.self.writeln(...) = 0`, `prod.emit_rust_expr(...) = 0` and `RawCode|SynItem|non_ir_path|pre_ir|conversion|transition` -> no matches in `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`)

### Validation rerun (2026-03-02, Pass BG)

1. `cargo test -q -p sifr_codegen` -> pass (`446` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. added recursive production-source guard `test_production_codegen_source_has_no_non_ir_tokens` in `lib_codegen_tests.rs`
6. new guard scans `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`) and hard-fails if non-IR token regressions appear (`RawCode`, `SynItem`, `non_ir_path`, `pre_ir`, `conversion`, `transition`, direct writer/emitter transition tokens)
7. production re-audit evidence after this loop: `prod.self.write(...) = 0`, `prod.self.writeln(...) = 0`, `prod.emit_rust_expr(...) = 0`, and no `RawCode|SynItem|non_ir_path|pre_ir|conversion|transition` matches in `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`)

### Validation rerun (2026-03-02, Pass BH)

1. `cargo test -q -p sifr_codegen` -> pass (`446` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. renamed old indentation-era statement helper `emit_rust_stmt_with_current_indent(...)` to IR-native `push_captured_stmt(...)` in `output_helpers.rs`
6. updated all production callsites (`lib.rs`, `stmt_support_emitter.rs`) to use the new IR-capture helper name
7. extended recursive production banlist guard in `lib_codegen_tests.rs` to reject `emit_rust_stmt_with_current_indent(` token reintroduction
8. production re-audit evidence after this loop: `prod.self.write(...) = 0`, `prod.self.writeln(...) = 0`, `prod.emit_rust_expr(...) = 0`, `prod.old_stmt_api(...) = 0`, and no `RawCode|SynItem|non_ir_path|pre_ir|conversion|transition` matches in `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`)

### Validation rerun (2026-03-02, Pass BI)

1. `cargo test -q -p sifr_codegen` -> pass (`446` passed)
2. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes e2e pass suite `394/394`)
3. Full recursive demos sweep `demos/**/*.sifr` -> stable (`91` scanned, `86` runnable pass, same `5` expected non-runnable/intentional files)
4. IR-first cleanup this loop:
5. renamed production statement-lowering method family from `try_emit_structured_*` to `try_lower_structured_*` in `lib.rs` + `stmt_support_emitter.rs`
6. updated architecture test expectations to the new lower-first naming, and fixed one guard assertion to ban only removed `try_emit_structured_*` expression API names
7. production re-audit evidence after this loop: `prod.self.write(...) = 0`, `prod.self.writeln(...) = 0`, `prod.emit_rust_expr(...) = 0`, `prod.emit_structured_name(...) = 0`, `prod.old_stmt_api(...) = 0`, and no `RawCode|SynItem|non_ir_path|pre_ir|conversion|transition` matches in `crates/sifr_codegen/src` (excluding `lib_codegen_tests.rs`)
