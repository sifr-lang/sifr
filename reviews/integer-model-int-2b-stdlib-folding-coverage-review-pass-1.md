# Review — INT-2B Stdlib Const Folding Integration Coverage (pass 1)

Branch: `int-2b-stdlib-const-folding-coverage`
Reference: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), [internal_docs/integer_model.md](../internal_docs/integer_model.md)
Predecessor review: [reviews/integer-model-int-2b-stdlib-const-values-review-pass-1.md](integer-model-int-2b-stdlib-const-values-review-pass-1.md) (PR #1802 merged)
Reviewer scope: correctness of the new integration test, whether it actually exercises the stdlib `constant_integer_values` path, module placement, guardrail health, missing assertions, PR readiness.

## Scope verified

The slice adds exactly one integration test, no production-code changes:

- [crates/sifr_driver/src/tests/stdlib_exports.rs:23](../crates/sifr_driver/src/tests/stdlib_exports.rs:23) — new `stdlib_integer_constants_fold_in_project_fixed_width_initializers`.
- Top-of-file imports updated to bring in `parse_suite`, `collect_project_hir_modules`, `HirExpr`, `HirStmt`, and `HashMap`.

`git status` shows only `crates/sifr_driver/src/tests/stdlib_exports.rs` modified. No other source, doc, or guardrail files changed.

## Does the test prove the intended path?

The predecessor review's actionable follow-up read:

> A minimal addition would be a project-graph test that imports a real stdlib constant (e.g. `from sifr.calendar import MONDAY` or `from sifr.logging import DEBUG`) and asserts the fitted let body collapses to a literal in a `uint8` / `uint32` slot.

This test is precisely that addition, and the chain it exercises is intact end-to-end:

1. **Stdlib bootstrap populates the export.** `lib/sifr/logging.sifr:6` declares `DEBUG: int = 10` at module scope. `compile_stdlib()` runs `lower_module_stdlib_with_externals` for `sifr.logging`, recording `LoweringResult.constant_integer_values["DEBUG"] = BigInt(10)`. The bootstrap's public-name filter at [crates/sifr_driver/src/stdlib/bootstrap.rs:147](../crates/sifr_driver/src/stdlib/bootstrap.rs:147) keeps `DEBUG` (no `_` prefix), and [bootstrap.rs:336](../crates/sifr_driver/src/stdlib/bootstrap.rs:336) inserts `("sifr.logging", {"DEBUG": BigInt(10)})` into `stdlib_defs.constant_integer_values`. `defs` is what the test passes downstream.
2. **Project import propagates value into lowering context.** `from sifr.logging import DEBUG` is handled by [crates/sifr_hir/src/lower/imports.rs:104-118](../crates/sifr_hir/src/lower/imports.rs:104), which both defines the local symbol from `externals.constants[…]` *and* copies `externals.constant_integer_values["sifr.logging"]["DEBUG"]` into `ctx.const_integer_values["DEBUG"]`. The latter line is the only place an *imported* integer constant becomes foldable — if it ever silently broke, this test catches it.
3. **Const folding evaluates `DEBUG + 1`.** When lowering `value: uint8 = DEBUG + 1`, [crates/sifr_hir/src/lower/fixed_width_fitting.rs:16](../crates/sifr_hir/src/lower/fixed_width_fitting.rs:16) routes the binop through `const_integer_value`, which resolves `HirExpr::Name { name: "DEBUG" }` via `ctx.const_integer_values` (line 113-117) to `BigInt(10)`, then `BinOp` returns `10 + 1 = 11`.
4. **Fixed-width fit replaces the expression.** Because `target` is `Type::FixedInt(U8)` and `11` lies in `0..=255`, [fixed_width_fitting.rs:32](../crates/sifr_hir/src/lower/fixed_width_fitting.rs:32) returns `Fits(bigint_to_hir_integer_literal(&11))` = `HirExpr::IntLiteral(11)` (small-literal branch at line 291). The let's `ty` slot retains `uint8`.

The test's two assertions — `ty.display_name() == "uint8"` and `matches!(value, HirExpr::IntLiteral(11))` — together pin every link in that chain. There is no path that produces `IntLiteral(11)` typed `uint8` *without* the imported integer value flowing through `ExternalDefs.constant_integer_values`: a regression in any of the four steps above either yields a non-literal `BinOp` (test panics on `matches!`), a type-mismatch error from `validate_annotated_constant_initializer` (test panics on `expect("project lowering should fit…")`), or a wrong literal value.

The test is the *symmetric* stdlib-import-side counterpart to [project_graph.rs:602 `test_project_lowering_fits_imported_integer_constants`](../crates/sifr_driver/src/tests/project_graph.rs:602), which proves the same chain for an in-project sibling module. The structural parallel is exact (same fixture shape, same final assertion form), with only the source of the constant swapped from a hand-written `BASE: int = 250 + 4` module to a real stdlib `.sifr` declaration.

## Module placement and guardrail health

`crates/sifr_driver/src/tests/project_graph.rs` is currently 678 lines; the `TEST_RS_MAX_LINES` limit in [scripts/check_sifr_driver_maintainability_guardrails.py:20](../scripts/check_sifr_driver_maintainability_guardrails.py:20) is 700. Adding the ~33-line test there would push the file to roughly 708 and break the guardrail. The author's choice to land it in `stdlib_exports.rs` (now 56 lines, ample headroom) is the correct call.

Naming-wise, `stdlib_exports.rs` is defensible for this test because the *load-bearing* link the test newly covers is the stdlib bootstrap export — `imports.rs` propagation is already covered by the in-project sibling test. The file is no longer purely about "which symbols are exported"; it is now about "stdlib export shape and its consumption contract". That reading is consistent with the existing `stdlib_heapq_exports_allowlisted_private_max_heap_helpers` already in the file, which also asserts a downstream-visible contract rather than a structural property.

I ran `python3 scripts/check_sifr_driver_maintainability_guardrails.py` against the working tree — PASS. The HIR guardrail script is untouched (no HIR files modified). No banned-monolith risk; no `lib.rs` shape regression; no CHECKLIST doc drift.

## Correctness of the test itself

- **Determinism.** The fixture has exactly two top-level statements in `main`: the typed let and the return. Lowering does not synthesize a prelude statement before user code in this shape, so `body[0]` is unambiguous. The pattern matches the established `body[0]` destructure used at [project_graph.rs:637](../crates/sifr_driver/src/tests/project_graph.rs:637).
- **`expect` messages.** Each `.expect(...)` carries a useful message describing the contract being relied on (`"stdlib should compile"`, `"project lowering should fit imported stdlib integer constants"`, `"main module should lower"`, `"main function should lower"`). On regression, the failure message points at the specific link.
- **Fit-vs-fold conflation.** A natural worry is that the test could pass via `Type::Int → Type::FixedInt(U8)` fitting at the *literal* level alone, without proving that `DEBUG`'s imported value matters. This is not the case: if `DEBUG` were *not* in `ctx.const_integer_values`, `const_integer_value` would short-circuit to `ConstIntegerValue::Unsupported` at line 117, the binop would also return `Unsupported`, the fitter would return `NotConst`, and `validate_annotated_constant_initializer` would fall through to the `is_assignable_to` check — `int` is not `uint8`-assignable, producing a `TYPE_MISMATCH` and failing `expect`. The `IntLiteral(11)` outcome is therefore strictly diagnostic of the imported-value path.
- **Choice of fixture (`DEBUG`/`uint8`).** `DEBUG = 10` keeps the post-fold value (`11`) trivially inside every fixed-width range, so the test does not also blur into a range-edge fit assertion. Choosing `uint8` is the strictest target where fitting is non-vacuous, which is the right pick for an integration smoke test. A larger-target variant (e.g. `MONDAY`/`uint32`) would not add coverage; it would only restate the same assertion at a different width.
- **Hard-coded `11`.** If `lib/sifr/logging.sifr` ever changed `DEBUG = 10` to a different value, this test would break. That is acceptable and arguably desirable — the change would be a CPython-compat-meaningful event and warrants a deliberate update.

## What the test does not assert (and whether it should)

I considered three additional assertions and concluded each is unnecessary:

1. **`name == "value"` on the let.** Defensive; fixture is unambiguous. Skipping it keeps the test focused on the integer-model contract, not on irrelevant lowering metadata.
2. **`is_mutable == false`.** Same reasoning. The mutability rule is tested elsewhere; not part of this slice's contract.
3. **Body length.** A `body.len() == 2` check would guard against the lowering ever reordering or inserting statements ahead of the let. There is no current code path that does so for this fixture, and the test's `expect` message + `matches!` assertion already fails loudly on any deviation. Leaving it out matches the predecessor test's discipline at [project_graph.rs:637](../crates/sifr_driver/src/tests/project_graph.rs:637).

The test does *not* cover the other consumer sites of `externals.constant_integer_values` flagged in the predecessor review:

- [crates/sifr_hir/src/lower/mod.rs:929](../crates/sifr_hir/src/lower/mod.rs:929) and [mod.rs:1083](../crates/sifr_hir/src/lower/mod.rs:1083) (broader name-resolution paths)
- [crates/sifr_hir/src/lower/compat_imports.rs:158](../crates/sifr_hir/src/lower/compat_imports.rs:158)

Whether those merit symmetric stdlib-side integration tests is a separate question — the predecessor review only requested *one* such test as a high-leverage sentinel. The chosen path through `imports.rs` is the dominant one; the others fire on rarer constructs (compat re-exports, qualified-name access). Adding more here would expand scope beyond the slice's stated intent.

## Style and minor notes (non-blocking)

1. **Test-name prefix.** The new test omits the `test_` prefix, matching the existing `stdlib_heapq_exports_allowlisted_private_max_heap_helpers` in the same file. `project_graph.rs` uses the `test_` prefix uniformly. The intra-file convention is consistent — fine to leave as-is.
2. **Fixture indentation.** The raw string uses leading-newline + flush-left content, identical to the predecessor test in `project_graph.rs`. Matches local style.
3. **Imports ordering.** `use std::collections::HashMap;` sits below the crate-internal imports. The existing `project_graph.rs` puts `std::collections::HashMap` last as well — consistent.
4. **Use of `find` over `.functions[0]`.** The function lookup uses `find(|f| f.name == "main")` rather than indexing. Good — avoids coupling to declaration order across modules.

None of the above are blockers.

## Validation review

The user-reported local validation set covers what this slice can break:

- `cargo fmt` — clean.
- `cargo test -p sifr_driver stdlib_integer_constants_fold_in_project_fixed_width_initializers -- --nocapture` — passes. This is the new test in isolation.
- `scripts/run_all_tests.sh --profile quick` — `report_signature=e1bf653aaa770517`, `wall_time=58.42s`. The signature matches the predecessor review's quick-profile signature exactly, which is reassuring: a test-only addition should not perturb the deterministic suite signature.

I additionally re-ran `python3 scripts/check_sifr_driver_maintainability_guardrails.py` — PASS. No HIR file modified, so the HIR guardrail script is not in scope.

The validation set is sufficient. Asking for `cargo clippy --workspace -- -D warnings` would be belt-and-braces; given this is a test-only diff with no new patterns or lints triggered, the quick-profile run already covers it.

## Readiness

The change is small, surgical, and addresses the precise non-blocking gap flagged in the predecessor review. It uses a real stdlib constant (`DEBUG` in `sifr.logging`), proves the full bootstrap → import → const-fold → fixed-width-fit chain, lives in the only test file that has guardrail headroom, and matches the structural shape of the existing in-project sibling test for ease of mental linkage. The lowering-context propagation step at `imports.rs:114` now has a regression-blocking integration test for the stdlib boundary specifically.

No production code changed; risk is bounded to test-suite semantics. The fixture choice does couple the test to `DEBUG = 10` in `lib/sifr/logging.sifr`, but that is an intentional integration-coverage tradeoff and any change to that constant should be a deliberate decision anyway.

VERDICT: SATISFIED
