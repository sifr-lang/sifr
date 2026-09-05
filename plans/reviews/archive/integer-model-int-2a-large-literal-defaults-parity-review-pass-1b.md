# INT-2A — Large Integer Literal Defaults and Unary Parity — Review Pass 1b

Reviewer: agent (agent), 2026-05-06.
Branch: `int-2a-large-literal-defaults-parity`.
Prior review: [reviews/integer-model-int-2a-large-literal-hir-review-pass-2.md](reviews/integer-model-int-2a-large-literal-hir-review-pass-2.md).
Issue: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), Milestone INT-2A, follow-up bullet "Carry INT-2A default-argument large-literal parity, negative-large-literal unary coverage…".

## Scope under review

This slice picks up pass-2 N2 ("`lower_expr_simple` default-arg parity") plus the negative-literal unary coverage from pass-2 N1. Two files modified, no schema or wiring changes:

- [crates/sifr_hir/src/lower/classes.rs:1247](crates/sifr_hir/src/lower/classes.rs:1247) — `lower_expr_simple` now mirrors `lower_number_literal`'s `i.as_i64()` branch: `Some(IntLiteral(v))` if it fits, else `Some(LargeIntLiteral(canonical_large_int_literal_text(i)))`. Adds `use super::integer_literals::canonical_large_int_literal_text;`.
- [crates/sifr_hir/src/lower/classes.rs:1282](crates/sifr_hir/src/lower/classes.rs:1282) — the `Expr::UnaryOp(USub)` arm gains a `HirExpr::LargeIntLiteral(value) => Some(UnaryOp { op: "-", operand: Box::new(LargeIntLiteral(value)), ty: Type::Int })` case alongside the existing `IntLiteral` and `FloatLiteral` legs.
- [crates/sifr_hir/src/lower/expressions_tests.rs](crates/sifr_hir/src/lower/expressions_tests.rs) — adds `test_negative_large_integer_literal_lowers_as_unary_large_literal` and `test_large_integer_default_arguments_lower_losslessly`.

No HIR variant additions, no codegen changes, no Cargo or feature-flag changes, no churn in pass fixtures or snapshots. The diff is 16 net lines in `classes.rs` and 48 net lines in tests. The validation report signature `e1bf653aaa770517` is unchanged from pass 2 (expected — these are unit-only changes that don't touch e2e fixtures).

I re-ran `cargo test -p sifr_hir -- large_integer` locally and all three tests pass.

---

## Behavioral analysis

### Parity with the main `lower_expr` path — solid

The new `lower_expr_simple` branches mirror the main path in [crates/sifr_hir/src/lower/expressions.rs:116](crates/sifr_hir/src/lower/expressions.rs:116) (`lower_number_literal`) and [crates/sifr_hir/src/lower/expression_operators.rs:91](crates/sifr_hir/src/lower/expression_operators.rs:91) (`lower_unaryop`):

| Source                                           | Main `lower_expr`                                                          | New `lower_expr_simple`                                                     |
|--------------------------------------------------|----------------------------------------------------------------------------|-----------------------------------------------------------------------------|
| `9223372036854775808` (>= 2^63)                  | `LargeIntLiteral("9223372036854775808")` via `lower_number_literal`        | same, via the new `else`-branch                                              |
| `-9223372036854775809` (decimal `i64::MIN - 1`)  | `UnaryOp { op: "-", operand: LargeIntLiteral("9223372036854775809"), ty: Int }` via `type_check_unary_op("-", &Type::Int) → Ok(Type::Int)` | same shape, with `ty: Type::Int` set directly because the simple path has no `LowerCtx` to call `type_check_unary_op` from |
| `-0x8000000000000001` (hex `-(2^63 + 1)`)        | as above with canonical text `"9223372036854775809"`                       | same                                                                        |

The simple path's hard-coded `ty: Type::Int` is safe because `HirExpr::LargeIntLiteral.ty()` already returns `&Type::Int` ([crates/sifr_hir/src/hir_nodes.rs:553](crates/sifr_hir/src/hir_nodes.rs:553)) and the type-system's own unit test asserts `type_check_unary_op("-", &Type::Int).unwrap() == Type::Int` ([crates/sifr_type_system/src/check.rs:696](crates/sifr_type_system/src/check.rs:696)). Both paths therefore produce structurally identical `HirExpr` for the four call sites that consume `lower_expr_simple` output:

- module-level constants ([crates/sifr_hir/src/lower/mod.rs:1102](crates/sifr_hir/src/lower/mod.rs:1102), [:1118](crates/sifr_hir/src/lower/mod.rs:1118))
- top-level function defaults ([crates/sifr_hir/src/lower/default_args.rs:34](crates/sifr_hir/src/lower/default_args.rs:34) and [crates/sifr_hir/src/lower/typing_and_functions.rs:245](crates/sifr_hir/src/lower/typing_and_functions.rs:245)/[:263](crates/sifr_hir/src/lower/typing_and_functions.rs:263))
- class field defaults and method/constructor defaults ([crates/sifr_hir/src/lower/classes.rs:500](crates/sifr_hir/src/lower/classes.rs:500), [:556](crates/sifr_hir/src/lower/classes.rs:556), [:606](crates/sifr_hir/src/lower/classes.rs:606))
- class field type inference from `__init__` assignments ([crates/sifr_hir/src/lower/class_field_inference.rs:119](crates/sifr_hir/src/lower/class_field_inference.rs:119))

Every one of these call sites previously dropped a large literal through `lower_expr_simple`'s `i.as_i64()?` short-circuit. In every case the upstream caller responded by emitting `TYPE_UNSUPPORTED_DEFAULT_ARGUMENT` (or, for module constants, by silently skipping the constant). Both regressions are now closed by the same single-line change.

### Unary-minus folding behavior

The new `LargeIntLiteral` arm in the USub case rebuilds the variant by moving `value` into a fresh `HirExpr::LargeIntLiteral(value)`. Functionally equivalent to wrapping `inner` itself in `Box::new(inner)`, just stylistically different. Either form compiles. The existing `IntLiteral(-v)` and `FloatLiteral(-v)` arms still constant-fold their numeric negation, while the new `LargeIntLiteral` arm intentionally does **not** fold — it preserves the unsigned text and wraps it in `UnaryOp("-", …)`. That matches the issue's "negative large integer literals as a unary minus around a lossless positive `LargeIntLiteral`" criterion exactly.

### Edge cases I checked

- **`i64::MIN` literally written as `-9223372036854775808`**: Ruff's lexer parses the magnitude as `9223372036854775808 = 2^63`, which exceeds `i64::MAX`. So `i.as_i64()` returns `None`, and the result is `UnaryOp("-", LargeIntLiteral("9223372036854775808"))` rather than `IntLiteral(i64::MIN)`. Pass-2 review pre-emptively flagged this exact case (its sub-observation). It is pre-existing and not regressed by this slice; the const-fitting pass in INT-2B is the right place to peephole-fold it back to a small literal where the value would fit `i64`.
- **`-(-LargeIntLiteral)`**: outer USub recurses, gets `Some(UnaryOp(...))` back, the inner `match` falls through `_ => None`, and the whole default fails to lower. That falls through to `TYPE_UNSUPPORTED_DEFAULT_ARGUMENT`. Pre-existing behavior — `lower_expr_simple` has always rejected nested unary on non-literal results — and not introduced here.
- **`+LargeIntLiteral` (unary plus)**: `lower_expr_simple` only matches `UnaryOp::USub`, so `def f(x: int = +9_223_372_036_854_775_808)` continues to fail. Pre-existing.
- **`Number::Complex { .. }` defaults**: still return `None`. Unchanged.
- **`canonical_large_int_literal_text` panic surface**: the helper is called only on the `else`-branch where `i.as_i64()` already returned `None`, so it always sees a value with at least 19 decimal digits or 16/22/64 hex/octal/binary digits. Its `BigUint::parse_bytes` fallback is `.map_or(text, …)`, so even an unreachable parse failure preserves the Ruff display rather than panicking. Same defensive shape as in pass 2.

### Cross-check with broader codegen pipeline

- Codegen leaf-no-recurse / leaf-no-error-refs / leaf-no-result-flow arms in [crates/sifr_codegen/src/error_refs.rs:446](crates/sifr_codegen/src/error_refs.rs:446), [crates/sifr_codegen/src/hir_analysis/traversal.rs:282](crates/sifr_codegen/src/hir_analysis/traversal.rs:282), and [crates/sifr_codegen/src/lower_stmt.rs:571](crates/sifr_codegen/src/lower_stmt.rs:571)/[:1502](crates/sifr_codegen/src/lower_stmt.rs:1502) continue to include `LargeIntLiteral(_)` (set up in the prior slice). Default arguments now passing through `lower_expr_simple` reach the same consumers.
- `try_lower_leaf_expr` in [crates/sifr_codegen/src/lower_expr.rs:103](crates/sifr_codegen/src/lower_expr.rs:103) still does **not** lower `LargeIntLiteral` to a `RustExpr`. This is consistent with pass-2 N3 ("`compile_error!` from codegen for `LargeIntLiteral`"): the contract for this slice is to capture the literal in HIR; emitting it from codegen is INT-2B/INT-3 work. Generated code that actually relies on a large default will surface the codegen gap there. **Behavioral note (non-blocking):** before this slice, a large default produced a frontend-quality `TYPE_UNSUPPORTED_DEFAULT_ARGUMENT` diagnostic; after this slice, the same source compiles further into codegen before failing. The post-slice failure mode is still loud (codegen rejects), and the trade-off is intentional and consistent with the slice's "preserve literals through HIR" goal — but it is a behavioral shift worth being aware of when triaging future user reports.

---

## Test coverage analysis

| Test                                                                              | Path exercised                                                                 |
|-----------------------------------------------------------------------------------|--------------------------------------------------------------------------------|
| `test_negative_large_integer_literal_lowers_as_unary_large_literal`               | Main `lower_expr` → `lower_unaryop` (function body assignment, not the simple path). Asserts the parity invariant from the **opposite** side. |
| `test_large_integer_default_arguments_lower_losslessly`, `params[0]`              | `typing_and_functions.rs::collect_function_defaults` → `lower_expr_simple` `Number::Int` else-branch. |
| `test_large_integer_default_arguments_lower_losslessly`, `params[1]`              | `typing_and_functions.rs::collect_function_defaults` → `lower_expr_simple` USub-on-LargeIntLiteral arm. |

Both of the new code paths in `classes.rs` are covered by `params[0]` and `params[1]` of the second test. The first test does not exercise the new code, but it usefully pins the parity invariant from the main path so a future refactor can't drift the two paths apart silently. I cross-checked the canonical-decimal expected values:

- `9223372036854775808` = 2^63 ✓
- `0x8000000000000001` = 2^63 + 1 = `9223372036854775809` ✓ (verified via `python3 -c 'print(0x8000000000000001)'`)
- `9_223_372_036_854_775_809` (underscored decimal `i64::MIN - 1` magnitude) = `9223372036854775809` ✓

### Coverage gaps (non-blocking)

The new `lower_expr_simple` branches are uniform and the second test exercises both. These gaps are about defense-in-depth, not correctness:

1. **Class field defaults** ([crates/sifr_hir/src/lower/classes.rs:500](crates/sifr_hir/src/lower/classes.rs:500)). The slice description explicitly mentions "simple class fields", but no test exercises a `class Foo:\n    big: int = 9_223_372_036_854_775_808` shape. The code path is identical to function defaults, so the risk of a silent regression here is low, but a one-class snapshot would close the gap cheaply.
2. **Module-level constants** ([crates/sifr_hir/src/lower/mod.rs:1102](crates/sifr_hir/src/lower/mod.rs:1102)/[:1118](crates/sifr_hir/src/lower/mod.rs:1118)). Same uniform path; no test asserts that a top-level `BIG: int = 9_223_372_036_854_775_808` lands in `module.constants` as `LargeIntLiteral`.
3. **Constructor and method defaults inside classes** ([crates/sifr_hir/src/lower/classes.rs:556](crates/sifr_hir/src/lower/classes.rs:556)/[:606](crates/sifr_hir/src/lower/classes.rs:606)). Same path, no test.
4. **Negative-large-literal default for class field / method**. Logically subsumed by the function-default test, but adds confidence that the USub arm fires uniformly across consumers.
5. **No e2e fixture exercising a large-literal default end-to-end.** The unit tests stop at HIR; pass through codegen (where N3 still applies) is not asserted. Acceptable while N3 is open, but worth noting in the slice's PR description so reviewers don't expect e2e churn.

None of these justify holding the slice — the behavior they would test is the same `match` arm exercised twice already, and `cargo test -p sifr_hir -- large_integer` now reports three passing tests as expected.

---

## Pass-2 follow-ups status

| Pass-2 finding                                                              | Status after this slice                                                                                                                         |
|-----------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------|
| **N1** — extra test cases (negative beyond `i64::MIN`)                      | **Resolved.** `test_negative_large_integer_literal_lowers_as_unary_large_literal` covers `-9_223_372_036_854_775_809`; `test_large_integer_default_arguments_lower_losslessly` covers `-0x8000000000000001` through the simple path. |
| **N2** — `lower_expr_simple` default-arg parity                              | **Resolved.** This slice's central change. `lower_expr_simple` now matches `lower_number_literal` byte-for-byte on `Number::Int` and adds the missing USub-on-`LargeIntLiteral` arm. |
| **N3** — codegen diagnostic for `LargeIntLiteral`                           | Still open, by design. Out of scope for INT-2A; properly belongs to INT-2B/INT-3.                                                                |
| **N4** — tuple compile-time index/slice diagnostics ignore `LargeIntLiteral`| Still open. Out of scope; INT-2B follow-up.                                                                                                      |
| **N6** — clippy cleanup commit shape                                        | Untouched here; cosmetic only.                                                                                                                   |

The pass-2 verdict explicitly requested N2 as "the very next item" — this slice delivers exactly that and nothing else. Scope discipline is good.

---

## Non-blocking suggestions

These are observations for follow-up slices, not requests for changes here.

1. **Doc comment refresh**: the comment above `lower_expr_simple` ([crates/sifr_hir/src/lower/classes.rs:1245](crates/sifr_hir/src/lower/classes.rs:1245)) says "literal values only". Strictly true (a `LargeIntLiteral` is still a literal), but a one-line note that the function is now parity-locked with `lower_number_literal` and `lower_unaryop` for integer literals would help future maintainers spot drift faster.
2. **Helper deduplication**: the `Number::Int` arm now appears verbatim in two places ([crates/sifr_hir/src/lower/expressions.rs:118-126](crates/sifr_hir/src/lower/expressions.rs:118) and [crates/sifr_hir/src/lower/classes.rs:1250-1258](crates/sifr_hir/src/lower/classes.rs:1250)). A two-line `pub(super) fn lower_int_literal(i: &Int) -> HirExpr` in `integer_literals.rs` would prevent a future reviewer from having to mentally diff the two sites again. Truly trivial; mention only because the duplicated branch is the exact thing pass 1b/2 went through trouble normalizing.
3. **`-(-LargeIntLiteral)` and `+LargeIntLiteral` defaults**: still rejected. Document or fix in the const-fitting slice. Likely no real users, but the diagnostic when it fires today is `TYPE_UNSUPPORTED_DEFAULT_ARGUMENT`, which doesn't tell the user the issue is a nesting depth limit rather than the value.
4. **Class field / module constant test**: see "Coverage gaps" above. Cheap to add; would lock in the uniform-path claim.
5. **`Box::new(HirExpr::LargeIntLiteral(value))`** at [crates/sifr_hir/src/lower/classes.rs:1284](crates/sifr_hir/src/lower/classes.rs:1284) re-wraps the moved string into the same variant. Stylistically `Box::new(inner)` (with `inner` matched as `HirExpr::LargeIntLiteral(_)` rather than a destructured binding) reads slightly cleaner, but the current form is fine and parallels the `IntLiteral(-v)` and `FloatLiteral(-v)` arms structurally.

---

## Final verdict

**SATISFIED.**

The slice resolves pass-2 N2 (the last remaining `i64`-only assumption inside HIR lowering) with the minimum possible diff, picks up pass-2 N1's negative-literal coverage as a bonus, and matches the issue's "function defaults and simple class fields" / "negative large integer literals as a unary minus around a lossless positive `LargeIntLiteral`" criteria. Parity with the main `lower_expr`/`lower_unaryop` path is verified at the structural and type-system levels. No new blockers introduced. Validation already passed locally on this exact diff (`cargo test -p sifr_hir -- large_integer` re-confirmed in this review).

Carry the remaining INT-2A non-blockers (N3 codegen diagnostic, N4 tuple-index diagnostics, the test coverage gaps in the "Coverage gaps" section, and the helper-deduplication suggestion) into the next slice. Of those, none are urgent; N3 is the natural next milestone work but explicitly belongs to INT-2B/INT-3.
