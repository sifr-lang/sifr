# INT-1 Large Module `int` Constant SifrInt Codegen — Review Pass 1

**Verdict:** Satisfied with non-blocking suggestions.

## Scope reviewed

Working-tree diff against `342071de` on `int-1-large-module-int-const-codegen`:

- [crates/sifr_hir/src/lower/fixed_width_fitting.rs](crates/sifr_hir/src/lower/fixed_width_fitting.rs)
- [crates/sifr_hir/src/lower/module_constants_lowering.rs](crates/sifr_hir/src/lower/module_constants_lowering.rs)
- [crates/sifr_hir/src/lower/expressions_tests.rs](crates/sifr_hir/src/lower/expressions_tests.rs)
- [crates/sifr_codegen/src/lower_item.rs](crates/sifr_codegen/src/lower_item.rs)
- [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)

Reference docs:
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
- [internal_docs/integer_model.md](internal_docs/integer_model.md)
- [reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md) (the N4 follow-up this slice closes)

## Slice goal — closed

The pass-4 N4 follow-up was that an in-budget literal exceeding `i64` (e.g. `LIMIT: int = 999_999_999_999_999_999_999_999_999_999_999_999`) survived `lower_module_integer_const_expr` → `lower_integer_const_expr_simple` → `HirExpr::LargeIntLiteral`, then `try_lower_simple_module_constant_item_result_impl` returned `Ok(None)`, and the production emitter panicked at [crates/sifr_codegen/src/module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12) with `structured module constant emission missing for production path (...)`.

The diff closes that path on both sides:

1. HIR: [oversized_int_module_constant_literal_for_codegen](crates/sifr_hir/src/lower/module_constants_lowering.rs:109) folds any `Type::Int`-annotated or bare module constant whose `BigInt` value does not fit `i64` to a canonical `HirExpr::LargeIntLiteral(value.to_str_radix(10))`. This is wired into both [collect_annotated_constant](crates/sifr_hir/src/lower/module_constants_lowering.rs:55) and [collect_bare_constant](crates/sifr_hir/src/lower/module_constants_lowering.rs:94), gated by the existing `error_count_before_initializer` check on the annotated path so an over-budget or fixed-width-out-of-range diagnostic still suppresses the fold.

2. HIR: [remember_module_const_integer](crates/sifr_hir/src/lower/fixed_width_fitting.rs:78) was extended to return `Option<BigInt>` so the caller can reuse the already-evaluated value without re-evaluating. The implementation hashes the same `ConstIntegerValue::Value(value)` arm and inserts a clone, so this is a non-behavioral refactor on the `remember` side.

3. Codegen: [large_module_int_literal_decimal](crates/sifr_codegen/src/lower_item.rs:183) is checked in both the Result and non-Result dispatchers *before* the legacy `is_simple_module_primitive_const_type` branch, and emits a private helper [lower_large_module_int_const_item](crates/sifr_codegen/src/lower_item.rs:231) returning `SifrInt` via `SifrInt::parse_decimal(text, sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)`. The `Err(err)` arm panics with a `compiler emitted invalid integer literal for module constant {name}: {{}}` message — programmer-invariant, not user-data-driven.

I traced the production path end-to-end:

- `BIG_LIMIT: int = 10 ** 20` ⇒ parser → `BinOp(IntLiteral(10), **, IntLiteral(20))` → `lower_module_integer_const_expr` returns the BinOp HIR with `ty: Int` → `error_count_before_initializer` sees no validate error (annotated `int = int` is `NotConst`, types match, no `TYPE_MISMATCH`) → `remember_module_const_integer` evaluates to `100_000_000_000_000_000_000` and caches it → `oversized_int_module_constant_literal_for_codegen(&Type::Int, Some(&value))` rejects `i64::try_from`, returns `LargeIntLiteral("100000000000000000000")` → `try_lower_simple_module_constant_item_result_impl` matches the new branch → emits `fn __const_BIG_LIMIT() -> SifrInt { match SifrInt::parse_decimal(...) { Ok(v) => return v, Err(err) => panic!(...) } }`. Verified by hand-running `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/module_constants.sifr`, which reproduces exactly that shape, and by `cargo run -q -p sifr -- run …`, which prints `100000000000000000000`.

- `BIG: int = -(10 ** 20)` ⇒ inner BinOp evaluates to `100_000_000_000_000_000_000`, the negate path produces `UnaryOp("-", BinOp, Int)` HIR, `remember_module_const_integer` evaluates the `"-"` arm to `-100_000_000_000_000_000_000`, and the codegen fold replaces the entire `UnaryOp` shape with a canonical `LargeIntLiteral("-100000000000000000000")`. Verified end-to-end: emitted `parse_decimal("-100000000000000000000", …)` runs and prints `-100000000000000000000`.

- Module-public flag preserves correctness: [try_emit_lowered_module_constant_result](crates/sifr_codegen/src/module_constants.rs:33) rewrites the new `RustItem::Fn { visibility, .. }` to `Visibility::Pub` when `module_public` is true, same as the existing string/none helpers.

- Use-site rewriting works: `module_constants.insert(name, (Type::Int, "__const_BIG_LIMIT()"))` via [try_emit_lowered_module_constant_result](crates/sifr_codegen/src/module_constants.rs:44), and [parse_module_constant_expr](crates/sifr_codegen/src/expr_render_helpers.rs:1204) converts a referencing `Expr::Name` into a fn-call expression. `str(BIG_LIMIT)` ends up as `format!("{}", __const_BIG_LIMIT())`, which works because `SifrInt: Display` is impl'd in the runtime crate. The auto-import collector at [ir_imports.rs:436](crates/sifr_codegen/src/ir_imports.rs:436) handles both the `SifrInt` type and the `sifr_runtime` path, so the generated file picks up `use sifr_runtime::SifrInt;`.

- The `Err` panic is unreachable under canonical input. `BigInt::to_str_radix(10)` produces `[\\-]?[0-9]+`, [count_decimal_digits](crates/sifr_runtime/src/int.rs:426) accepts that shape, both `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET` ([fixed_width_fitting.rs:103](crates/sifr_hir/src/lower/fixed_width_fitting.rs:103), [integer_literal_diagnostics.rs:9](crates/sifr_hir/src/lower/integer_literal_diagnostics.rs:9)) and [DEFAULT_MAX_INTEGER_DIGITS](crates/sifr_runtime/src/int.rs:10) are `4096`, and the HIR fold only fires after `remember_module_const_integer` already passed `reject_if_over_budget`. So the runtime digit-limit check cannot reject what the compiler emits today.

## Determinism / regression check

- Annotated *fixed-width* over-budget paths are unchanged. `LIMIT: uint8 = 10 ** 5000` still emits one `SIFR-INT-0004` from `validate_fixed_width_initializer` and the `error_count_before_initializer` guard skips the new fold. The new `oversized_int_module_constant_literal_for_codegen` also short-circuits on `!matches!(ty.resolve_alias(), Type::Int)`, so even if the gate were missed, a `FixedInt` annotation would not trigger SifrInt codegen.
- Annotated `int` over-budget paths are unchanged. `LIMIT: int = 10 ** 5000` ⇒ `remember_module_const_integer` emits one `SIFR-INT-0004` and returns `None`; the fold sees `value: None` and returns `None`; the existing `LargeIntLiteral` HIR survives but the diagnostic short-circuits the build before codegen runs. The pinned [test_module_int_over_budget_const_expr_stays_hir_diagnostic](crates/sifr_hir/src/lower/expressions_tests.rs:466) still asserts `errors.len() == 1`.
- The `i64::try_from(value.clone()).is_ok()` gate keeps small `int` constants on the legacy `RustItem::Const { ty: I64, … }` path — `MAX_RETRIES: int = 3`, `BASE_LIMIT: int = 250`, `LIMIT: int = BASE_LIMIT + 4`, `NEGATIVE_LIMIT: int = -(MAX_RETRIES + 10)` all stay on `i64`, as visible in the emitted snapshot. Same-module `int` reuse and unary/binop folding from the INT-2B pass-4 fix still works (the e2e fixture pins `'254'` and `'-13'`).
- The non-Result dispatcher `try_lower_simple_module_constant_item` only has test-only callers (`lib_codegen_tests.rs:944` aside, all uses are tests in `lower_item.rs`), so the duplicated check at [lower_item.rs:222](crates/sifr_codegen/src/lower_item.rs:222) does not change production behavior; it only keeps the dispatcher paths symmetric. No drift.

## Scope drift

- `module_constants_lowering.rs` adds a single helper and two call sites mirrored across the annotated and bare paths. No drift into general expression lowering.
- `fixed_width_fitting.rs` change is a return-type extension on an existing helper; the only behavioral side effect is the now-required `value.clone()` to satisfy the `BigInt` return — `value` was previously consumed into the map insertion. `num_bigint::BigInt: Clone`, so the clone is allocator-bound but unavoidable here.
- `lower_item.rs` adds `RustMatchArm` to the imports, a single helper for the SifrInt parse-decimal call, the dispatcher pre-check, and the `lower_large_module_int_const_item` constructor. No edits to existing branches.
- The e2e fixture only adds one constant and one assertion. No deletions or renames.

## Tests

- [test_module_int_const_expr_above_i64_folds_to_large_literal_for_codegen](crates/sifr_hir/src/lower/expressions_tests.rs:507) pins the HIR fold on the annotated path: `LIMIT: int = 10 ** 20` lowers to `(LIMIT, Type::Int, LargeIntLiteral("100000000000000000000"))`. The `module.constants` shape is the contract the codegen new branch matches against, so this is the right invariant.
- [dispatcher_result_lowers_large_module_int_const_as_sifr_int_helper](crates/sifr_codegen/src/lower_item.rs:479) pins the codegen lowering shape: `Fn { name: "__const_limit", ret: Some(Named("SifrInt")), … }` with the rendered body containing `SifrInt::parse_decimal("100000000000000000000", sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)` and asserts `!rendered.contains(".unwrap(")` / `!rendered.contains(".expect(")`. Sufficient for the no-panic-via-unwrap invariant.
- [module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr) adds `BIG_LIMIT: int = 10 ** 20` and `assert str(BIG_LIMIT) == '100000000000000000000'`, smoke-testing the full pipeline (parse → HIR fold → codegen helper → runtime parse_decimal → Display).

## Non-blocking findings

### N1 — `BIG_LIMIT + 1` in a function body emits invalid Rust

This is the most impactful follow-up. With the new lowering, `BIG_LIMIT: int = 10 ** 20` succeeds, but if a downstream use site mixes it with a small `int` (e.g. `x: int = BIG_LIMIT + 1`), the emitter produces `__const_BIG_LIMIT() + (1 as i64)`, which `rustc` rejects with:

```
error[E0277]: cannot add `i64` to `SifrInt`
   --> src/main.rs:15:38
    |
15 |     let x: i64 = __const_BIG_LIMIT() + (1 as i64);
    |                                      ^ no implementation for `SifrInt + i64`
```

Reproduced locally with a minimal `BIG_LIMIT: int = 10 ** 20\n\ndef main():\n    x: int = BIG_LIMIT + 1\n    print(str(x))\n` driver. Before this slice the file would have died at compiler-panic time inside `emit_module_constants`, so this is *strictly* an incremental improvement — the failure mode moves from compiler crash to `rustc` error. But it leaves a real, narrow gap where Sifr accepts source that produces invalid generated Rust, which violates the spirit of the "if it compiles, it works" guarantee at the surface level.

This is consistent with the issue's framing ("Wire module-level `int` constants whose in-budget values exceed `i64` through `SifrInt` codegen, removing the current module-constant production panic path") and the pass-4 N4 note that broader `Type::Int` arithmetic SifrInt wiring belongs to INT-1/INT-3 wave 2. So the slice can land as-is for the panic-removal goal, but a follow-up should track this to closure either by a typed Sifr-level "int constant exceeds i64" diagnostic on use sites until full SifrInt arithmetic lands, or by routing any `int`-typed module constant through SifrInt regardless of size when the broader wave runs. Worth tracking explicitly in the issue's INT-1 checklist so it doesn't slip behind the panic-removal tick.

### N2 — Codegen-side `UnaryOp` branches in `large_module_int_literal_decimal` are dead from the production HIR pipeline

[large_module_int_literal_decimal](crates/sifr_codegen/src/lower_item.rs:187) handles `HirExpr::UnaryOp { op: "+", … }` and `HirExpr::UnaryOp { op: "-", … }`, but the HIR fold added in this slice always pre-flattens both shapes to a canonical `LargeIntLiteral("-…")` or `LargeIntLiteral("…")` via `bigint.to_str_radix(10)`. Both production callers (annotated and bare) go through the same `oversized_int_module_constant_literal_for_codegen` path, so the dispatcher's `UnaryOp` branches are only reachable if a future caller passes an un-folded shape.

Two options, neither blocking:
- Add a unit test that calls the codegen helper with an un-folded `UnaryOp("-", LargeIntLiteral("1…"))` shape to keep the branches alive and pinned.
- Drop the `UnaryOp` branches from `large_module_int_literal_decimal` since the HIR pipeline guarantees the canonical shape — leaning on the contract the new HIR test asserts.

I'd lean toward keeping the branches plus adding the test (so the codegen helper stays robust to non-canonical callers), but either is fine.

### N3 — Bare-constant fold path has no focused HIR test

The new test covers the annotated `LIMIT: int = 10 ** 20` shape. The bare `BIG = 10 ** 20` (no annotation) path was modified — `mut hir_value`, `oversized_int_module_constant_literal_for_codegen` call — but no HIR test specifically exercises it. Tracing it: `lower_module_integer_const_expr(BinOp(10, **, 20))` returns the BinOp with `ty: Int`, `remember_module_const_integer` evaluates and caches, the fold replaces `hir_value` with the canonical `LargeIntLiteral`. Functionally identical to the annotated path, just not pinned. A two-line addition mirroring `test_module_int_const_expr_above_i64_folds_to_large_literal_for_codegen` for the bare-assignment shape (`BIG = 10 ** 20\n\ndef main():\n    print(str(BIG))\n`) would close the gap.

### N4 — No "compile-time vs runtime digit budget" contract test

The unreachability of the `panic!` arm relies on `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET` (HIR) and `DEFAULT_MAX_INTEGER_DIGITS` (runtime) staying in lockstep at `4096`. They live in different crates and aren't structurally tied. A trivial assertion test in `sifr_codegen` (or an integration test) like

```rust
assert_eq!(
    sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
    sifr_hir::lower::INTEGER_EVAL_DECIMAL_DIGIT_BUDGET, // pub-export needed
);
```

would pin the invariant so a future change reducing the runtime limit (for example) cannot silently make the panic reachable from valid Sifr source. Optional — the values are unlikely to drift accidentally — but cheap insurance, and aligned with the INT-1 milestone scope ("Add generated-code panic-shape tests for runtime integer paths").

### N5 — Per-call re-parsing of the decimal text

`__const_BIG_LIMIT()` calls `SifrInt::parse_decimal(...)` on every reference, allocating a fresh `BigInt` each time. For ordinary module constants this is consistent with how the existing `__const_<name>()` helpers for `String`/`None` already work (the string helper does `to_string()` per call), but the SifrInt path is the first to do real work in the helper body. INT-8 perf gates will likely want a `OnceLock`/`LazyLock`-cached static or a `&'static SifrInt` accessor. Not a concern for this slice.

### N6 — Minor: string-literal expression construction in `sifr_int_parse_decimal_call`

[sifr_int_parse_decimal_call](crates/sifr_codegen/src/lower_item.rs:200) constructs the decimal-text argument as `RustExpr::Ident(format!("\"{}\"", decimal_text.escape_default()))`. `escape_default()` is fine because the canonical decimal text only contains ASCII digits and an optional `-`, neither of which `escape_default` mangles. Using `RustExpr::Literal(RustLiteral::Str(decimal_text.into()))` would be more idiomatic (the renderer already handles quoting/escaping for `Str` literals). Style nit, not behavioral.

## Validation

I re-traced rather than re-ran the listed validation. The cited results are consistent with the code:

- `cargo fmt` — diff is fmt-clean (no stray indentation against the existing style).
- `cargo test -p sifr_hir module_int_const_expr_above_i64 -- --nocapture` — the test source pins exactly the shape my trace describes.
- `cargo test -p sifr_codegen large_module_int_const -- --nocapture` — the rendered-substring assertions match the call-site I verified.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` — I reproduced this directly via `emit` + `run` and confirmed `100000000000000000000` and `-100000000000000000000` both round-trip.
- `cargo test -p sifr_hir module_ -- --nocapture` and `cargo test -p sifr_codegen module_const -- --nocapture` — broader regressions; the slice does not touch the cases pinned by the existing `test_module_*` set.
- `scripts/run_all_tests.sh --profile quick` — `report_signature=e1bf653aaa770517`, `wall_time=71.91s`, passed. The manifest-bound suite is the authoritative gate per AGENTS.md and is green.

## Verdict

**Satisfied with non-blocking suggestions.** The slice cleanly closes the pass-4 N4 follow-up: in-budget `int` module constants exceeding `i64` no longer crash the emitter, they lower through a `SifrInt` helper using `parse_decimal` against a programmer-invariant `panic!` arm that is unreachable while the HIR and runtime digit budgets agree. The HIR fold and codegen branch are mirrored across the annotated and bare paths and tested at both the HIR-shape level and end-to-end. Non-blocking suggestions: (N1) downstream `int + int` arithmetic mixing the new SifrInt helper with the legacy `i64` `const` shape produces `rustc` errors and should be tracked as the next sub-item under INT-1 wave 2; (N2) prune-or-test the codegen-side `UnaryOp` branches that the HIR fold makes dead; (N3) add a focused HIR test for the bare-constant fold; (N4) pin the `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET == DEFAULT_MAX_INTEGER_DIGITS` invariant with a contract test; (N5) consider lazy-static caching of the parse_decimal result for INT-8 perf; (N6) `RustLiteral::Str` is more idiomatic than `Ident` for the decimal-text argument. None gates merge.
