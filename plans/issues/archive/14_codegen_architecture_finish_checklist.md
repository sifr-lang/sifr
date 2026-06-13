# Phase 14 Strict Finish Checklist (Code-Verified)

Last verified: 2026-02-27

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

status: **met**

- [x] `lower_expr.rs`, `lower_stmt.rs`, `lower_item.rs`, `context.rs`, `preamble.rs` exist
- [x] Broad variant coverage exists in simple lowering paths
- [x] `expr_to_string` helper removed from production path
- [x] Core pipeline fallback-emitter first-class routing removed from production wrappers (`emit_*_fallback` no longer a default production path)
- [x] `emit_module` now assembles via `RustFile` item flow with a single render sink
- [x] Production `lower_*` contract is `Result<_, CodegenError>` end-to-end across production call sites (guarded by decomposition test against non-result helper regressions)
- [x] Production stmt lowering entry now has explicit `Result` contract (`try_lower_simple_stmt_with_scope_result`) with context validation
- [x] Production stmt lowering `Result` entry now validates nested stmt/expr shapes before fallback routing
- [x] Production expr lowering entry now has explicit `Result` contract (`try_lower_leaf_expr_result`) with shape validation
- [x] Production emit wrappers now route through explicit `Result`-based structured-attempt helpers (`try_emit_structured_stmt`, `try_emit_structured_expr`) before fallback
- [x] Core structured stmt/expr emission now rewrites lowered special-name idents (stdlib math constants + module constants) before render, so these names no longer force fallback in main emit-path gating
- [x] Core structured expr emission now attempts intrinsic and registry-method call lowering before fallback (`HirExpr::Call` via `try_emit_intrinsic_via_registry`, `HirExpr::MethodCall` via `try_emit_method_via_registry`)
- [x] Core structured expr emission now attempts signature-safe plain function-call lowering before fallback (`HirExpr::Call` via `try_emit_structured_plain_call_with_signature` for by-value, type-matching args)
- [x] Core stmt/expr wrappers now attempt structured lowering before force-fallback gating; borrowed-param compare/bool expressions remain explicitly guarded to fallback semantics
- [x] Legacy recursive force-fallback gating helpers were removed from core wrappers (`should_force_*_fallback`, `expr_contains_force_fallback_name`), keeping fallback as a pure post-structured sink
- [x] Structured stmt emission now bridges non-leaf expression statements through `try_emit_structured_expr` before full stmt fallback (with proper `;`/newline sink)
- [x] Structured stmt emission now bridges copy-typed `Assign` RHS expressions through `try_emit_structured_expr` before full stmt fallback
- [x] Structured stmt emission now bridges copy-typed `Let` RHS expressions through `try_emit_structured_expr` before full stmt fallback
- [x] Structured stmt emission now bridges copy-typed `Return` RHS expressions through `try_emit_structured_expr` before full stmt fallback (outside display/generator contexts)
- [x] Expression fallback no longer enforces subtree-wide legacy-only recursion (`fallback_depth` removed), allowing nested fallback subexpressions to still attempt structured lowering
- [x] Production helper rendering paths now consume expr `Result` contract (`expr_render_helpers`, `intrinsic_method_emitters`)
- [x] Registry arg/object lowering now shares one explicit expr `Result` helper path (`try_lower_registry_expr_result`) with strict no-inline-`RawCode` fallback shims in registry emit paths
- [x] Production module-constant item lowering entry now has explicit `Result` contract (`try_lower_simple_module_constant_item_result`) with name-shape validation
- [x] Production module-constant item lowering `Result` path now propagates leaf-lowering errors (not `None`-collapse)
- [x] Production module-constant emission now routes through explicit `Result` helper (`try_emit_lowered_module_constant_result`) before fallback
- [x] Production statement lowering now enters through `ScopeContext` (`try_lower_simple_stmt_with_scope` in `emit_stmt`)
- [x] Union enums emit as structured `RustItem::Enum` nodes through `enum_items` assembly (legacy raw `enum_defs` string path removed)

### milestone_codegen_intrinsic_migration

status: **met**

- [x] Intrinsic registry exists (`intrinsics/mod.rs`) with dependency metadata
- [x] Method registry exists (`methods/mod.rs`)
- [x] Codegen returns `required_crates`
- [x] Legacy intrinsic fallback dispatcher removed from production path (`emit_intrinsic_call` registry-first)
- [x] Legacy method fallback path removed from production path (`emit_method_call` registry-first)
- [x] `builtin_open` and related file-handle intrinsics now lower through structured IR helpers (no giant monolithic string templates)
- [x] Logging intrinsics (`set_global_level`, `get_global_level`) now lower via structured IR nodes (no `RawCode` emission path)
- [x] `file_close` intrinsic now lowers via structured IR (typed arg + `remove(&__hid)`), removing monolithic `RawCode` template
- [x] `builtin_open` / `open_file` now lower through structured IR blocks/match arms (no monolithic string-template assembly)
- [x] File-handle read/write lowerers now build structured match arm stmt vectors (no per-intrinsic `String` body assembly)
- [x] File-handle `read`/`write`/`read_bytes`/`write_bytes` now use structured IR trait-call (`std::io::*`) + structured `Err`/`Ok` returns (no per-path raw `use`/`Err(...)` templates)
- [x] File-handle `readline`/`readlines` now lower via structured IR loops/conditionals/trait-call paths (`std::io::BufRead::*`) without raw body templates/import stubs
- [x] `open_file` now emits structured success returns and uses fully qualified `std::io::BufReader/BufWriter` constructors (no raw open-path import stubs)
- [x] `builtin_open` now routes through closure-`Result` + `?` with structured success returns (removing raw open-arm success emission)
- [x] File-handle `owned_str` and wrapper plumbing now use structured IR only (`to_string` method call + no dead raw-import shim in handle wrappers)
- [x] File-handle ID allocation now uses preamble IR static+helper (`__SIFR_NEXT_FILE_HANDLE_ID`, `__sifr_next_file_handle_id`) so open lowerers no longer inject AtomicI64 `RawCode` blocks
- [x] Intrinsic parenthesization now has first-class IR support (`RustExpr::Paren`), and `math`/`hash`/`hashlib`/`sys` lowerers no longer use `RawCode(format!("({})", ...))` shims
- [x] Registry/helper expression lowering now rewrites stdlib math constants (`pi`/`e`/`tau`/`inf`/`nan`) from lowered idents into canonical constant paths, so these arg expressions stay IR-first instead of forced fallback
- [x] Registry/helper expression lowering now rewrites module constant idents (`CONST_NAME` / `__const_name()`) from lowered idents into canonical const/helper-call IR forms, removing module-constant forced fallback in helper paths
- [x] Method registry borrowed-arg helpers now avoid `RawCode` variant handling; tuple helpers use structured literals/casts
- [x] Math intrinsic `ldexp` now lowers via typed structured IR (`f64` cast + `2.0.powi(i32)`), removing its monolithic `RawCode` template
- [x] Math intrinsic `sumprod` now lowers via structured IR block/for-loop accumulation (no monolithic `RawCode` template)
- [x] Math intrinsic `modf` now lowers via structured IR conditional block shape (no monolithic `RawCode` template)
- [x] Math intrinsic `ulp` now lowers via structured IR conditional/numeric path (no monolithic `RawCode` template)
- [x] Math intrinsic `nextafter` now lowers via structured IR branch path (no monolithic `RawCode` template)
- [x] Math intrinsic `remainder` now lowers via structured IR branch/rounding path (no monolithic `RawCode` template)
- [x] Math intrinsic `dist` now lowers via structured IR loop/scale accumulation path (no monolithic `RawCode` template)
- [x] Math intrinsic `fsum` now lowers via structured IR compensated-sum loop path (no monolithic `RawCode` template)
- [x] Math intrinsic `erf` now lowers via structured IR polynomial path (no monolithic `RawCode` template)
- [x] Math intrinsic `erfc` now lowers via structured IR complementary polynomial path (no monolithic `RawCode` template)
- [x] Math intrinsic `frexp` now lowers via structured IR bit-decompose path (no monolithic `RawCode` template)
- [x] Math intrinsic `gamma` now lowers via structured IR Lanczos/reflection path (no monolithic `RawCode` template)
- [x] Math intrinsic `lgamma` now lowers via structured IR log-gamma path (no monolithic `RawCode` template)
- [x] Core scalar math intrinsic lowerers (`sqrt`..`isqrt`) now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `sys`/`platform`/`hash`/`hashlib` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `test` assertion intrinsics and `uuid4` now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `calendar` and `toml` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `env` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `bytes` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `base32` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `base64` (`base64_*`, `urlsafe_b64*`) intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `json` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `html` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `gzip` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `subprocess` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `datetime` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `zipfile` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `pathlib` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `random` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `time` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `os` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `re` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `io` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `collections` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] `file_handles` intrinsic lowerers now take typed IR args in registry dispatch (no string-arg dispatcher path)
- [x] DoD constraint on long `self.write(...)` bodies (>100 chars) met for intrinsic/method lowering paths

### milestone_codegen_structural_passes

status: **met**

- [x] IR import pass exists (`ir_imports.rs`)
- [x] IR clone optimization exists (`ir_optimize.rs`)
- [x] IR validation exists (`ir_validate.rs`)
- [x] Old helper names (`filter_rust_code_to_needed`, `parse_rust_blocks`, etc.) removed
- [x] Boolean import flags removed from primary import selection path (imports now derived from structural IR needs)
- [x] Stdlib DCE migrated to structural `syn` item traversal (`stdlib_filter.rs` no longer uses text/chunk/token parsing)
- [x] `ir_imports` now uses structural `syn` traversal for `RawCode` payloads (text-token scan only as parse-failure fallback)
- [x] Production and test codegen entrypoints now run `validate_items` on assembled `file_items` (including `RawCode` wrappers) before render
- [x] `generate_rust_multi` now uses assembled `RustFile` + `validate_items` + single renderer sink instead of manual string assembly
- [x] `generate_rust_multi` module-import prelude now lowers as structured `RustItem::Use`/`RustItem::UseAlias` items (no raw import-string prelude block)
- [x] Module constant emission now routes into assembled body-item lists (`RustItem`/`RawCode` entries) instead of writing constant definitions directly into `emitter.output` strings
- [x] Module class/function body emission now drains per-item raw chunks into assembled body-item lists (`RustItem::RawCode`) instead of retaining monolithic `emitter.output` accumulation
- [x] Top-level assembly path now enforces drained output contract (`assert_output_drained`) and no longer appends residual `emitter.output` as fallback `RawCode` in `generate_rust_with_stdlib`/`generate_rust_multi`/`generate_rust_test`
- [x] Union-enum `Display` impl generation now uses structured IR (`RustType::Ref` + `RustStmt::Match` + `write!` macro call) instead of `RawCode` type/stmt shims
- [x] Union-enum `Display` format argument now lowers as `RustLiteral::Str` (no `RustExpr::RawCode` shim for format spec literals)
- [x] `RawCode`-zero gate enforced for core production paths via explicit validation and production hard-fail on raw leakage
- [x] Structural passes now run over full user-code IR in the production assembly path

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
