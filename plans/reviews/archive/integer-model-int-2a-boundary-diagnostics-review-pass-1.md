# INT-2A — Large Literal Boundary Diagnostics — Review Pass 1

Reviewer: agent (agent), 2026-05-06.
Branch: `int-2a-large-literal-boundary-diagnostics`.
Prior review: [reviews/integer-model-int-2a-large-literal-defaults-parity-review-pass-1b.md](reviews/integer-model-int-2a-large-literal-defaults-parity-review-pass-1b.md).
Issue: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), Milestone INT-2A, follow-up bullet "Carry INT-2A malformed/over-budget literal diagnostics and parsed/constructed HIR parity in the next INT-2A slice."

## Scope under review

This slice closes the INT-2A milestone by landing the boundary-diagnostic and parsed-vs-constructed parity work that pass 1b explicitly deferred. Eight files touched, all small, no Ruff submodule changes, no codegen changes, no fixtures, no schema churn:

- [crates/sifr_diagnostics/src/codes.rs](crates/sifr_diagnostics/src/codes.rs:63) — adds `DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED` (`SIFR-INT-0004`, Severity::Error) plus its `active_entry!` and `ACTIVE_DIAGNOSTIC_CODES` registration. Declared args `digits` and `max_digits`, owner `sifr_hir::lower::integer_literal_diagnostics`, fixture pointer to the new unit test.
- [crates/sifr_hir/src/lower/integer_literal_diagnostics.rs](crates/sifr_hir/src/lower/integer_literal_diagnostics.rs:1) — new 53-line module with `validate_module_integer_literals` plus a `Visitor`-based `IntegerLiteralBudgetVisitor` that flags any `Expr::NumberLiteral`/`Number::Int` whose canonical-decimal representation exceeds `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET = 4096` characters.
- [crates/sifr_hir/src/lower/mod.rs:50](crates/sifr_hir/src/lower/mod.rs:50), [:513](crates/sifr_hir/src/lower/mod.rs:513) — wires the new module and runs `validate_module_integer_literals(stmts, &mut ctx)` once at the very top of `lower_module_impl`, immediately after `register_builtins`.
- [crates/sifr_hir/src/lower/expressions_tests.rs](crates/sifr_hir/src/lower/expressions_tests.rs:159) — adds `test_large_integer_literal_over_budget_has_int_code` (4097-digit `1`-repeat literal in a function body) and `test_constructed_and_parsed_large_integer_literals_match_hir` (parsed `0xFFFF_FFFF_FFFF_FFFF_FFFF` vs. constructed `Int::from_str_radix(...)`).
- [crates/sifr_driver/src/tests/single_file_frontend.rs:127](crates/sifr_driver/src/tests/single_file_frontend.rs:127) — adds `test_parse_source_surfaces_malformed_integer_token_as_typed_diagnostic` over `def main():\n    value = 0123\n`, asserting `PARSE_LEXICAL_OR_STRING` with `parser_category == "lexical_other"` and a `reason` containing `"Invalid decimal integer literal"`.
- [docs/errors/SIFR-INT-0004.md](docs/errors/SIFR-INT-0004.md), [docs/errors/diagnostic-codes.md](docs/errors/diagnostic-codes.md), [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md) — generated registry doc, public catalog row, and internal catalog row for `SIFR-INT-0004`.

I locally re-ran:

- `cargo test -p sifr_hir test_large_integer_literal_over_budget_has_int_code` — passes.
- `cargo test -p sifr_hir test_constructed_and_parsed_large_integer_literals_match_hir` — passes.
- `cargo test -p sifr_driver test_parse_source_surfaces_malformed_integer_token_as_typed_diagnostic` — passes.

The validation report signature `e1bf653aaa770517` is the same one carried by the previous slice; that is consistent with this slice making no e2e fixture/codegen changes.

---

## Correctness analysis

### `validate_module_integer_literals` — visitor design

The module-level pass uses Ruff's `sifr_python_ast::visitor::Visitor`. `walk_body` recurses into `walk_stmt` → `walk_expr` and through every statement form (function defs, class bodies, type aliases, ann/aug-assign, for/while/if/with/try/match), every expression form (BinOp/UnaryOp/Lambda/If/Dict/Set/List/Tuple/comprehensions/calls/subscript/attr/named/starred/f-string interpolation/format spec/t-string), every parameter/keyword/default, every type-param `bound`/`default`, every match-case pattern (`Pattern::MatchValue` is walked via `visit_expr`), every with-item, every except handler. So integer literals appearing in:

- module statements, nested function bodies, class field defaults, method/constructor defaults
- function/lambda parameter defaults
- type-alias values, decorator expressions, generator/comprehension subjects/predicates/elements
- f-string and t-string interpolated expressions and format-spec interpolations
- match-case literal patterns (positive or negative-via-USub)
- with-item context expressions, `except` exception types, ann-assign annotations, aug-assign rhs
- `TypeVar`/`TypeVarTuple`/`ParamSpec` defaults and bounds

are all visited exactly once. I traced this through [third_party/ruff/crates/ruff_python_ast/src/visitor.rs:135](third_party/ruff/crates/ruff_python_ast/src/visitor.rs:135) (`walk_stmt`), [:365](third_party/ruff/crates/ruff_python_ast/src/visitor.rs:365) (`walk_expr`), [:677](third_party/ruff/crates/ruff_python_ast/src/visitor.rs:677) (`walk_parameters` — walks defaults before annotations), [:761](third_party/ruff/crates/ruff_python_ast/src/visitor.rs:761) (`walk_pattern`), and [:823](third_party/ruff/crates/ruff_python_ast/src/visitor.rs:823) (`walk_interpolated_string_element`). No double-visits, no missed locations. Good.

### `validate_integer_literal_budget` — value-canonicalization branch

The flow is:

1. `value.as_i64().is_some()` short-circuits anything fitting `i64` ([crates/sifr_hir/src/lower/integer_literal_diagnostics.rs:36](crates/sifr_hir/src/lower/integer_literal_diagnostics.rs:36)). Cheap, exact, and avoids the BigUint allocation for the common case.
2. Calls `canonical_large_int_literal_text(value)` — the same helper that pass 2 normalized to canonical decimal ([crates/sifr_hir/src/lower/integer_literals.rs:4](crates/sifr_hir/src/lower/integer_literals.rs:4)). Its fallback `.map_or(text, …)` keeps it panic-free if `BigUint::parse_bytes` ever fails on a presumed-valid token.
3. `digits = canonical.len()` — bytes equals chars for ASCII decimal output, so the comparison is in decimal characters as intended.
4. `digits <= 4096` returns silently; otherwise emits via `ctx.error_with_code_at(DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED, message, range)` with `range = number_literal.range()`.

The skip on `as_i64().is_some()` is provably safe for `u64::MAX` and below because `i64::MAX` (9223372036854775807) has 19 decimal digits and `u64::MAX` (18446744073709551615) has 20 — both far below 4096. Anything that could exceed 4096 decimal digits is necessarily stored by Ruff as `Number::Big(token)`, where `Display` writes the original token text and `parse_unsigned_integer_literal_text` strips `0x`/`0o`/`0b` prefixes and `_` separators before re-encoding. So decimal/hex/octal/binary literals are all canonicalized consistently before the digit count is taken.

I checked the radix conversion math: a 4097-binary-digit literal is ≈ `4097 / 3.322 ≈ 1234` decimal digits and is *not* over budget; a 4097-hex-digit literal is ≈ `4097 × 1.204 ≈ 4933` decimal digits and *is* over budget. The implementation correctly compares against the *canonical decimal* digit count, not source-text length, so radix mixing produces the right answer.

### Integration with `lower_module_impl`

Running `validate_module_integer_literals` immediately after `register_builtins` ([crates/sifr_hir/src/lower/mod.rs:513](crates/sifr_hir/src/lower/mod.rs:513)) is the right placement. It:

- Runs on every successful parse — the parser must already have produced a `Vec<Stmt>` for `lower_module_impl` to be called, so we never visit a partial/recovered tree here.
- Pushes errors into `ctx.errors`, which the `if ctx.errors.is_empty() { Ok(...) } else { Err(ctx.errors) }` gate at [crates/sifr_hir/src/lower/mod.rs:1153](crates/sifr_hir/src/lower/mod.rs:1153) honors. The over-budget case therefore short-circuits codegen — `lower_source` returns `Err`, and the `LargeIntLiteral` HIR that subsequent lowering would have produced never reaches `try_lower_leaf_expr`.
- Also runs through `lower_module_with_externals` ([crates/sifr_hir/src/lower/mod.rs:497](crates/sifr_hir/src/lower/mod.rs:497)) since it shares `lower_module_impl`, so external-bearing entry points are covered too.

Lowering continues after the visitor pushes errors. That is intentional and consistent with the rest of the lowering pass — multiple over-budget literals in the same module will each surface their own diagnostic, and unrelated lowering errors continue to surface alongside. The `errors.iter().any(...)` shape of the test correctly tolerates unrelated companion diagnostics if any future change emits them.

### Diagnostic registry, owner, and docs

- `INT_EVAL_BUDGET_EXCEEDED` is added to the `DiagnosticCode` namespace block ([crates/sifr_diagnostics/src/codes.rs:63](crates/sifr_diagnostics/src/codes.rs:63)) and to `ACTIVE_DIAGNOSTIC_CODES` ([crates/sifr_diagnostics/src/codes.rs:1574](crates/sifr_diagnostics/src/codes.rs:1574)). Both spots match the existing INT-0003 convention.
- The `active_entry!` block ([crates/sifr_diagnostics/src/codes.rs:754](crates/sifr_diagnostics/src/codes.rs:754)) uses owner `sifr_hir::lower::integer_literal_diagnostics`, which matches the new module's path; family `INT`; severity `Error`; representative fixture pointing at `expressions_tests.rs::test_large_integer_literal_over_budget_has_int_code`; declared args `digits (message+json)` and `max_digits (message+json)`; dedupe args `digits, max_digits`. Consistent with `SIFR-INT-0003` shape.
- The generated `docs/errors/SIFR-INT-0004.md` and the catalog rows in `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` line up exactly with the registry entry — placement after `SIFR-INT-0003`, table column alignment preserved, no stray edits to neighboring rows. The "Generated by ... Do not edit by hand" header is intact in `SIFR-INT-0004.md`.
- The reported validation set includes `python3 scripts/check_diagnostic_docs_sync.py` and `python3 scripts/check_diagnostic_code_coverage.py`, both passing — that exactly covers the registry-doc-coverage trio.

### Message template vs. emitted message

Registry template: `integer literal exceeds compile-time evaluation budget: {digits} decimal digits (max {max_digits})`
Emission ([:46-52](crates/sifr_hir/src/lower/integer_literal_diagnostics.rs:46)): `format!("integer literal exceeds compile-time evaluation budget: {digits} decimal digits (max {INTEGER_EVAL_DECIMAL_DIGIT_BUDGET})")`

The template's `{max_digits}` placeholder is satisfied at emission time by inlining the `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET` constant. The test pins the literal "max 4096" form, so any future budget tweak forces a coordinated test+doc+constant update — that is mildly brittle but matches how existing INT-0003 and `TYPE_*` HIR diagnostics couple their constants to their tests.

Note: like every other HIR-level error in this codebase, `error_with_code_at` does not carry structured args through to `RenderedDiagnostic`. Driver-level rendering ([crates/sifr_driver/src/frontend/module_lowering.rs:86-93](crates/sifr_driver/src/frontend/module_lowering.rs:86)) re-wraps the rendered string in `[("message", DiagnosticArg::String(message))]` with template `"{message}"`, so the JSON channel sees `args.message`, not `args.digits` / `args.max_digits`. The registry's `digits (message+json)` / `max_digits (message+json)` declaration is therefore aspirational at the HIR layer (same as INT-0003 with `name`). This pre-dates the slice and is not a regression — see "Non-blocking" item D below for a tracked follow-up.

### Parser-driver malformed-token test

`def main():\n    value = 0123\n` triggers Ruff's "leading zero on decimal" lexer error path ([third_party/ruff/crates/ruff_python_parser/src/lexer.rs:1214](third_party/ruff/crates/ruff_python_parser/src/lexer.rs:1214) emits `OtherError("Invalid decimal integer literal")`). That funnels into `lexical_or_string_details` ([crates/sifr_driver/src/frontend/parser_diagnostics.rs:258](crates/sifr_driver/src/frontend/parser_diagnostics.rs:258)) where `LexicalErrorType::OtherError(_)` maps to `parser_category = "lexical_other"`, code `PARSE_LEXICAL_OR_STRING`, template `"lexical error: {reason}"`, and the `reason` arg carries the ruff-side message string. The new test asserts both shape pieces (`parser_category`, `reason` substring) — exactly the typed-diagnostic contract that the existing `test_parse_source_classifies_parser_error_categories` asserts for other lexical examples. The `reason.contains("Invalid decimal integer literal")` substring assertion is correctly tolerant of incidental lexer-message tweaks.

This test does not (and should not) overlap with the over-budget HIR diagnostic — it covers the malformed-token-text branch the issue calls out separately. The two together complete the negative parser/frontend coverage required by the INT-2A acceptance bullet.

### Parsed-vs-constructed parity test

`Int::from_str_radix("FFFFFFFFFFFFFFFFFFFF", 16, "0xFFFF_FFFF_FFFF_FFFF_FFFF")` ([third_party/ruff/crates/ruff_python_ast/src/int.rs:48](third_party/ruff/crates/ruff_python_ast/src/int.rs:48)) tries `u64::from_str_radix("FFFFFFFFFFFFFFFFFFFF", 16)` first, hits `IntErrorKind::PosOverflow` (20 hex digits > u64), and stores `Int::big("0xFFFF_FFFF_FFFF_FFFF_FFFF")`. `lower_expr_simple` ([crates/sifr_hir/src/lower/classes.rs:1247](crates/sifr_hir/src/lower/classes.rs:1247)) takes the `else` branch and produces `LargeIntLiteral(canonical_large_int_literal_text(i))`. Canonicalization strips `0x`, drops `_`, parses as hex, re-encodes as decimal, and yields exactly `"1208925819614629174706175"` (= 2^80 − 1).

The parsed path goes through full module lowering → `lower_number_literal` ([crates/sifr_hir/src/lower/expressions.rs](crates/sifr_hir/src/lower/expressions.rs)), the same `else` branch, the same canonicalization helper. Both literals are far below the 4096-digit budget, so `validate_module_integer_literals` does not fire on the parsed source — and the pattern match on `(constructed, parsed)` confirms structural equality of the two `LargeIntLiteral` values.

This test pins the INT-2A acceptance bullet "constructed-AST path and parsed-source path produce equivalent HIR literal representations" with a non-trivial radix (hex with `_` separators) and verifies the canonical decimal value is what downstream consumers see. Good.

### Range-targeting in the over-budget test

`literal = "1".repeat(4097)`; source = `format!("def main():\n    value = {literal}\n")`. `range_for(&source, &literal)` finds the first occurrence of "1...1" in source. The prefix `def main():\n    value = ` contains no `1`, so the first match is at byte offset 24 — the literal's source-text start. `number_literal.range()` is the parser's exact span for the literal token, which equals that exact byte range. The test pins primary range correctly.

`canonical_large_int_literal_text` for a `Big("1...1")` value: strips no prefix → radix 10, filters underscores (none) → 4097 ones, `BigUint::parse_bytes` succeeds, `to_str_radix(10)` returns the same 4097 ones → `digits = 4097 > 4096` → fire. The asserted message `"integer literal exceeds compile-time evaluation budget: 4097 decimal digits (max 4096)"` matches exactly.

---

## Test coverage assessment

| Acceptance criterion (INT-2A)                                          | Test                                                                                                    | Status      |
|-----------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------|-------------|
| Negative parser/frontend test for malformed integer token text        | `test_parse_source_surfaces_malformed_integer_token_as_typed_diagnostic` (driver)                       | Covered     |
| Constructed-AST and parsed-source paths produce equivalent HIR        | `test_constructed_and_parsed_large_integer_literals_match_hir` (HIR unit)                               | Covered     |
| Over-budget literal emits `SIFR-INT-0004`                             | `test_large_integer_literal_over_budget_has_int_code` (HIR unit)                                        | Covered     |
| Reserved `int128`/`uint128` produce reserved-width diagnostic         | `test_reserved_integer_width_annotations_have_int_code`                                                  | Covered (prior slice) |
| Parser/frontend tests for decimal/hex/octal/binary beyond `i64`       | `test_large_integer_literals_lower_losslessly_from_source`                                               | Covered (prior slice) |
| Ruff submodule unmodified                                             | `git diff` shows no `third_party/ruff` edits                                                             | Verified    |

That accounts for every numbered INT-2A bullet in the issue's "Validation" and "Acceptance criteria" lists. The slice intentionally does not add an e2e `.sifr` fail fixture for over-budget literals (which would also be valid coverage but is heavier than necessary). The pass-1b-style "carry an e2e fail fixture" follow-up still applies but is not blocking here, same as it wasn't blocking for the INT-0003 slice.

### Coverage gaps (non-blocking)

1. **Boundary case at exactly 4096 digits**. `"1".repeat(4096)` should *not* fire, and a positive test would lock the off-by-one. The current `if digits <= INTEGER_EVAL_DECIMAL_DIGIT_BUDGET { return; }` is correct — `4096 <= 4096` is true and returns silently — but a test would prevent a future tweak to `<` from regressing without a CI signal. Cheap to add.
2. **Negative over-budget literals**. `def f(x: int = -("1".repeat(4097)))` — the visitor visits the `Expr::NumberLiteral` operand of `Expr::UnaryOp(USub)`, fires once on the magnitude. No test pins this.
3. **Hex/octal/binary over-budget**. The radix math is the only path-divergent piece (canonicalization branch on `0x`/`0o`/`0b` prefix). A single hex over-budget literal would lock that branch.
4. **Multiple literals → multiple diagnostics**. `errors.iter().any(...)` style accepts both 1 and N matches; a test that asserts both literals trigger would lock per-literal emission.
5. **Over-budget literal in a position other than `value = ...` (default arg, type-alias rhs, comprehension element, match-case `case <literal>`, f-string format-spec interpolation)**. These all share the same visitor leg by construction, but a single representative test per position would harden against future refactors that, say, replace `walk_body` with a custom traversal.
6. **Module-level constant (`BIG: int = 9_223_372_036_854_775_808` and the over-budget version)**. Pass 1b raised this for the previous slice and it is still open: no test asserts a top-level constant flows through the new diagnostic visitor.
7. **No e2e `.sifr` fail fixture** under `crates/sifr/tests/e2e/fail/` referenced from the registry's `representative_fixture` slot. INT-0003 also defers this; carry to INT-7 cleanup or to the const-fitting INT-2B slice.

None of these gaps justifies blocking. The existing visitor traversal is uniform, and the three new tests cover the path's correctness end-to-end at the unit level.

---

## Slice scope discipline

Verified the slice is precisely as advertised:

- No HIR variant additions, no codegen changes, no Ruff submodule changes, no fixtures, no schema churn, no PR file moves. `git diff --stat` reports 8 files, 151 lines added, 1 line removed.
- The new module is a small (53-line) focused file — well under the 1200-line `mod.rs` and the 1400-line `classes.rs` HIR maintainability guardrails ([scripts/check_hir_maintainability_guardrails.py:12](scripts/check_hir_maintainability_guardrails.py:12)). It does not appear in `BANNED_MONOLITHS`. No `MAX_LINES_BY_FILE` entry needs updating because the new file is not on the tracked list — that is consistent with how previous slice modules (e.g. `min_max_validation.rs`, `flow_diagnostics.rs`) are handled.
- All four pass-1b non-blocking carry items map cleanly to either this slice (`SIFR-INT-0004` is no longer "deferred") or to INT-2B (`N3` codegen diagnostic, `N4` tuple compile-time index/slice diagnostics, helper deduplication for the `Number::Int` lowering arm, the boundary-case test gap, and class-field/module-constant/method-default tests). Nothing has been quietly dropped.
- The validator runs on success-parsed modules only; no risk of running over a partially-recovered AST.

---

## Non-blocking observations

These are observations for follow-up slices, not requests for this PR.

- **A. Over-budget literal lifetime / representation cost.** `canonical_large_int_literal_text` allocates a `BigUint` and a `String` per visit. For an over-budget literal, the canonical text is also re-allocated when `lower_number_literal` runs the same helper a second time. A `Cow`/cache through `LowerCtx` would deduplicate, but the win is only measurable for sources with many large literals; not worth the complexity yet.

- **B. Validator runs unconditionally.** Modules with all small integers still pay an O(AST size) traversal. Ruff doesn't currently expose a "module contains a `Big` int" hint; a one-bit `Parsed` flag would let `lower_module_impl` skip the visitor entirely on small-int-only sources. Defer until a perf signal demands it.

- **C. Lifetime annotation style.** `impl<'a> Visitor<'a> for IntegerLiteralBudgetVisitor<'_>` mixes a named AST lifetime with an anonymous ctx lifetime. Reads slightly oddly; `impl<'ast, 'ctx> Visitor<'ast> for IntegerLiteralBudgetVisitor<'ctx>` is one line longer but clearer about who borrows whom. Cosmetic.

- **D. Structured args at the HIR-error layer.** `INT-0004` declares `digits (message+json)` / `max_digits (message+json)` but `error_with_code_at` only carries a single `message` arg in the rendered envelope (template `"{message}"`, args `[("message", String(rendered))]`). This is the same mismatch INT-0003 ships, so it is not a regression — but the integer model's diagnostic-contract surface (issue §"Diagnostics" + design doc §"Diagnostics") implies `digits`/`max_digits` should be machine-readable on the JSON channel. Tracking item for a generalized HIR-args refactor; out of scope here.

- **E. Doc comment for `validate_module_integer_literals`.** The new function has no rustdoc. A two-line comment noting (a) it runs once per module, before the rest of `lower_module_impl`, and (b) it intentionally only flags *literal* nodes — computed const expressions like `10 ** 5000` are INT-2B/INT-3 work — would help the next person who extends it.

- **F. Constant duplication.** `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET = 4096` is module-private to `integer_literal_diagnostics.rs` but its concrete value is also baked into the registry template `(max {max_digits})` substitution and the test assertion. Future const-evaluator work in INT-2B will need the same constant. Hoisting to a shared `sifr_diagnostics::limits` or `sifr_hir::config` module before INT-2B would prevent drift.

- **G. f-string format-spec coverage.** The visitor walks format-spec interpolated elements (verified via `walk_interpolated_string_element`), so `f"{x:0{1...4097-ones}}"` would fire `SIFR-INT-0004`. That is a strange surface but legal Python; a defensive test would lock it down. Cheap, non-blocking.

- **H. `walk_body` vs `walk_module`.** `validate_module_integer_literals` calls `visitor::walk_body(&mut visitor, stmts)`. That walks the module-level statement list and recurses into nested function/class bodies. Equivalent behavior, but `walk_body` reads as "this is not the root" — using the explicit module-level helper, if Ruff adds one, would communicate intent more clearly. Stylistic.

---

## Pass-1b follow-ups status

| Pass-1b carry-item                                                              | Status after this slice                                                                                                                |
|---------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------|
| Class field / method / module-constant default coverage gap                     | Still open. INT-2B/INT-7 home; not in this slice's scope.                                                                              |
| Helper deduplication for the `Number::Int` lowering arm                         | Still open. Pure refactor; no behavioral implication for INT-2A close.                                                                  |
| `-(-LargeIntLiteral)` / `+LargeIntLiteral` rejection diagnostic clarity         | Still open. Const-fitting territory.                                                                                                    |
| `compile_error!` from codegen for `LargeIntLiteral` (N3)                        | Still open by design; INT-2B / INT-3.                                                                                                   |
| Tuple compile-time index/slice diagnostics ignore `LargeIntLiteral` (N4)        | Still open.                                                                                                                             |
| Over-budget / malformed-literal diagnostics (this slice's bullet)               | **Resolved.**                                                                                                                            |
| Parsed-vs-constructed HIR parity (this slice's bullet)                          | **Resolved.**                                                                                                                            |

The slice closes the two issue-tracked carry items it set out to close, defers the rest precisely where they belong (INT-2B, INT-3, INT-7), and does not regress any prior INT-2A acceptance.

---

## Final verdict

**SATISFIED.**

The slice introduces a correctly-scoped HIR-level visitor that emits `SIFR-INT-0004` for source integer literals canonicalizing to more than 4096 decimal digits, registers the diagnostic with consistent metadata across the Rust registry and the public/internal catalogs, surfaces the existing parser-side malformed-token-text path as a typed diagnostic test, and pins parsed-vs-constructed HIR parity for large literals. The visitor traversal covers every AST position where an `Expr::NumberLiteral` can appear, the canonicalization branch is radix-correct and panic-safe, and the new error path short-circuits codegen via the standard `ctx.errors` gate. The validation report on this exact diff (`cargo fmt --check`, `git diff --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir large_integer`, `cargo test -p sifr_driver test_parse_source_surfaces_malformed_integer_token_as_typed_diagnostic`, `python3 scripts/check_diagnostic_docs_sync.py`, `python3 scripts/check_diagnostic_code_coverage.py`, `cargo clippy -p sifr_hir -p sifr_diagnostics -p sifr_driver -- -D warnings`, `scripts/run_all_tests.sh --profile quick` with e2e signature `e1bf653aaa770517`) covers the relevant gates and matches what I re-ran locally for the three new tests.

The non-blocking suggestions (A–H) and the carry-coverage gaps (1–7 in "Coverage gaps") are appropriate INT-2B / INT-7 work — none justifies holding this slice. With this PR landing, every INT-2A acceptance bullet and validation bullet in the issue is satisfied, and the issue's INT-2A subsection is ready to be ticked complete.
