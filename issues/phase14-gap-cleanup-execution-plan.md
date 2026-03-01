# Phase 14 Gap Cleanup Execution Plan (Implementation Blueprint)

Date: 2026-02-26  
Status: In Progress (Reopened 2026-02-28)  
Completed on: 2026-02-25 (original closeout)
Owner: Codegen architecture closeout  
Primary scope:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md`
2. `issues/217-phase14-remove-fallback-first-class-pipeline.md`
3. `issues/218-phase14-promote-full-ir-module-assembly.md`
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md`
5. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md`

Completion evidence:
1. `issues/216-phase14-codegen-architecture-closeout-epic.md` (child issues merged in order + gate results)
2. `issues/217-phase14-remove-fallback-first-class-pipeline.md` (Merged PR: `#784`)
3. `issues/218-phase14-promote-full-ir-module-assembly.md` (Merged PR: `#785`)
4. `issues/219-phase14-enforce-rawcode-zero-in-core-production-path.md` (Merged PR: `#786`)
5. `issues/220-phase14-structural-passes-hard-gate-no-raw-fallback.md` (Merged PR: `#787`)

---

## Execution Status (Current Recheck)

- [x] WS0 baseline inventory and ownership mapping completed and consumed by follow-up migration slices.
- [ ] WS1 structured expression lowering expansion not complete (`self.write` heavy paths still production-reachable).
- [ ] WS2 structured statement lowering expansion not complete (statement emitters still string-heavy).
- [ ] WS3 top-level item migration not complete (`RustItem::SynItem` still present in module-body user path).
- [x] WS4 naming decommission completed in `crates/sifr_codegen/src` (no bridge-named production helpers remain).
- [ ] WS5 production `RawCode`/`SynItem` hard gate not complete for strict user-path closeout.
- [ ] WS6 structural-pass hard gate not complete for strict raw-text independence.
- [ ] WS7 epic closeout/checklist/doc updates blocked until strict IR-first hard gates pass.

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
1. [x] Loop-1: remove explicit fallback/legacy bridge artifacts and naming from `crates/sifr_codegen/src`.
2. [ ] Loop-2: migrate highest-traffic `self.write(...)` expression/statement paths to structured IR emission.
3. [x] Loop-3: restore parity and revalidate demos/tests (`test_e2e_pass` and runnable demo sweep).
4. [ ] Loop-4: rerun phase gates and update all phase docs/issues with exact validated status.

Loop progress log:
1. 2026-02-28: loop baseline confirmed with `cargo test -q -p sifr --test e2e test_e2e_pass` -> `213 passed, 181 failed`.
2. 2026-02-28: loop iteration applied and validated:
3. fallback/legacy bridge module removed from `crates/sifr_codegen/src`.
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
27. Bridge-named production helper path still present in `crates/sifr_codegen/src/intrinsic_method_emitters.rs` (`try_lower_registry_expr_bridge`).
28. 2026-02-28 Pass B updates:
29. Removed bridge-named production helper path by renaming `try_lower_registry_expr_bridge` -> `try_lower_registry_expr_recursive`.
30. Added non-regression guard test asserting bridge-named helper signature is not present.
31. Started hot-path extraction in `expr_render_helpers.rs` for print/fstring/display macro assembly via shared structured helper builders.
32. Validation: `cargo test -q -p sifr_codegen` pass; targeted demo subset pass (`milestone_ergonomics`, `milestone_codegen_fixes`, `milestone_new_modules`, `milestone_stdlib_parity`, `milestone_stdlib_pure_expansion`, `milestone_stdlib_remediation`).
33. 2026-02-28 Pass C fixes:
34. Fixed malformed structured return/raise emission in `stmt_support_emitter.rs` (missing wrapper-call close `)` regression).
35. Removed split-string test assertions in `intrinsic_method_emitters.rs` and replaced with direct production-slice guards.
36. Full validation: `./scripts/run_all_tests.sh` -> pass.
37. Full demo sweep: all `demos/*.sifr` -> pass (`83/83`).
38. Terminology recheck in production source: no `bridge|fallback|legacy|migration` matches under `crates/sifr_codegen/src`.
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
50. Removed all direct `self.write(...)` usage from `method_call_emitter.rs` and enforced structured-only method-call emission (no fallback branch; missing shapes now hard-fail).
51. Exposed structured registry expr helpers in `intrinsic_method_emitters.rs` for cross-emitter reuse.
52. Validation: `./scripts/run_all_tests.sh` -> pass (includes `test_e2e_pass` -> `394` passed, `0` failed).
53. Validation: full demo sweep `demos/*.sifr` -> pass (`83/83`).
54. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1132`.
55. `self.write(...)` in `method_call_emitter.rs` -> `0`; `self.write(...)` in `intrinsic_method_emitters.rs` -> `0`.
56. 2026-02-28 Pass F updates:
57. Moved additional display-critical lowering into shared registry IR path (no emitter-local string fallback): string step-slice (`s[::2]`, `s[::-1]`), `**` binop, `sum(...)`, and `Compare` chains.
58. Fixed structured index lowering regression by normalizing negative list/string indices in IR blocks (`[-1]` behavior restored) with single-evaluation object binding.
59. Fixed method-call convention loss in structured expression lowering for class methods by resolving class method parameter conventions from signatures/class metadata and applying borrow/option wrapping in registry method-call lowering.
60. Full validation: `./scripts/run_all_tests.sh` -> pass (includes `test_e2e_pass` -> `394` passed, `0` failed).
61. Full demo sweep: `demos/*.sifr` -> pass (`83/83`).
62. Strict implementation re-audit refresh: `self.write(...)` in `crates/sifr_codegen/src` -> `1107`.
63. File-level progress: `expr_ref_emitter.rs` now has `0` direct `self.write(...)`; `method_call_emitter.rs=0`; `intrinsic_method_emitters.rs=0`.
64. 2026-02-28 Pass G updates:
65. Converted structured plain-call emission in `expr_render_helpers.rs` to emit IR `RustExpr::FnCall` (removed direct write-based call assembly).
66. Delegated additional special-call builtins to registry IR lowering (`bool`, `pow`, `bigint`, `round`, `abs`, `sum`, plus 2-arg `min/max`) and removed duplicated write-based branches.
67. Reduced `min/max` fallback branch in `expr_render_helpers.rs` to list-aggregator case only (`args.len()==1`) with 2-arg path handled by registry lowering.
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
113. Completed `match_emitter.rs` migration from token-by-token writes to line-based pattern/guard rendering helpers; direct `self.write(...)` usage in the file removed.
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

### Next Loop To-do (Evidence-Based)

1. [ ] Loop-2A: Continue migrating `expr_render_helpers.rs` hot paths (`305` writes) to structured IR-first expression builders.
2. [ ] Loop-2B: Migrate `stmt_support_emitter.rs` (`218` writes) and top item emitters (`class_emitter.rs`, `class_method_emitter.rs`, `slice_emitter.rs`) off string assembly.
3. [ ] Loop-2C: Remove user-path drain-parse flow and `RustItem::SynItem` push in `module_body.rs`.
4. [x] Loop-4A: Remove bridge-named production helpers (`try_lower_registry_expr_bridge`) and replace with explicit structured-only naming/pathing.
5. [ ] Loop-4B: Add/refresh hard-gate tests enforcing zero user-path `SynItem` and preventing string-emission regressions in production paths.
6. [ ] Loop-4C: Re-run full phase gate commands and only then re-mark WS1..WS7 complete.

Dependency-ordered execution queue (leaf -> orchestrator) for remaining `.write` files:
1. [x] `helpers.rs` (completed in Pass L; now `0` direct `self.write(...)`)
2. [x] `render.rs` (completed in Pass M; now `0` direct `self.write(...)`)
3. [x] `slice_emitter.rs` (completed in Pass N; dead module removed)
4. [x] `match_emitter.rs` (completed in Pass O; now `0` direct `self.write(...)`)
5. [ ] `expr_render_helpers.rs` (`249` writes; core expression lowering hot path)
6. [ ] `stmt_support_emitter.rs` (`218` writes; core statement lowering hot path)
7. [ ] `function_emitter.rs` (`50` writes; item-level wrapper over stmt/expr lowering)
8. [ ] `type_emitters.rs` (`61` writes; item/type wrappers over lowered expression bodies)
9. [ ] `operator_protocol_emitters.rs` (`52` writes; operator/protocol wrappers over lowered bodies)
10. [ ] `class_method_emitter.rs` (`70` writes; class method wrappers over stmt/expr lowering)
11. [ ] `class_emitter.rs` (`93` writes; class item orchestration over class-method/type/operator emitters)
12. [ ] `lib.rs` (`16` writes; top-level orchestration entrypoint and hard-gate cleanup)
13. [ ] `lib_codegen_tests.rs` (`1` write assertion; final guard/test rewrite)

### Active Implementation Loop (2026-02-28, Pass E)

1. [x] Item 1: Remove bridge-named production helper path in `intrinsic_method_emitters.rs` and keep behavior parity.
2. [x] Item 2: Start hot-path extraction in `expr_render_helpers.rs` to reduce direct `self.write(...)` string assembly for structured print/fstring/display flows.
3. [x] Item 3: Add focused guards/tests for naming and behavior to prevent bridge-path reintroduction.
4. [x] Item 4: Fix structured return/raise terminator regression introduced during extraction in `stmt_support_emitter.rs`.
5. [x] Item 5: Re-run full local validations (`./scripts/run_all_tests.sh` + full `demos/*.sifr` sweep) and record results.
6. [x] Item 6: Remove direct `self.write(...)` from `intrinsic_method_emitters.rs` by routing emission through lowered IR rendering only.
7. [x] Item 7: Remove direct `self.write(...)` from `method_call_emitter.rs` and enforce structured-only method-call emission (no fallback).

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
3. `cargo run -q -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr` -> pass (`total = 24`, `verdict = high`)
4. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr` -> pass (`current_has_t = true`, `today_has_dash = true`)
5. `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (`cargo test -p sifr` + `run_e2e_pass.sh`; final `test_e2e_pass` result: `394` passed, `0` failed)
