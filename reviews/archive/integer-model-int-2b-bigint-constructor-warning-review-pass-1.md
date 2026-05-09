# INT-2B `bigint(...)` constructor warning — review pass 1

## Scope reviewed

Direct emission of `SIFR-INT-0011` ("bigint is a temporary transition alias…") when lowering a bare `bigint(...)` constructor call, plus the test rebalancing in `crates/sifr_driver/src/tests/single_file_frontend.rs`. Branch: `int-2b-bigint-constructor-warning`. Working-tree diff at the time of review:

- [crates/sifr_hir/src/lower/expressions.rs:857](crates/sifr_hir/src/lower/expressions.rs:857) — single-line addition `ctx.warn_bigint_transition_alias(call.func.range());` inside the existing `if func_name == "bigint"` branch.
- [crates/sifr_driver/src/tests/single_file_frontend.rs:256](crates/sifr_driver/src/tests/single_file_frontend.rs:256) — annotation-only fixture (`value: bigint = 1`) replacing the previous mixed `value: bigint = bigint(1)` form, and a new constructor-only test at [single_file_frontend.rs:284](crates/sifr_driver/src/tests/single_file_frontend.rs:284).

The slice answers the open follow-up flagged at [reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md:69](reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md:69) and [reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md:105](reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md:105) — that the `bigint(...)` constructor remained the strongest silent `bigint` mention in the language.

## What is correct in the new delta

1. **Emission point is the right one.** The branch at [expressions.rs:856](crates/sifr_hir/src/lower/expressions.rs:856) is reachable only when (a) the callee is a bare `Name`, (b) `func_name == "bigint"` after [resolve_bare_python_compat_call_alias](crates/sifr_hir/src/lower/compat_imports.rs:34) (which never aliases `bigint` to anything), and (c) the surrounding `if !builtin_is_shadowed` guard at [expressions.rs:421-424](crates/sifr_hir/src/lower/expressions.rs:421) holds — so a user-defined `bigint` binding (scope or function table) bypasses the branch entirely. There is no false-positive surface for attribute calls (`obj.bigint(...)`), method calls, or shadowed locals.

2. **Span is consistent with all other emit sites.** `call.func.range()` is the range of the bare `bigint` identifier itself (since `call.func` is `Expr::Name("bigint")` here), matching:
   - annotation form at [typing_and_functions.rs:439-440](crates/sifr_hir/src/lower/typing_and_functions.rs:439) (uses `name.range()`),
   - `isinstance(..., bigint)` form at [builtin_calls.rs:930-931](crates/sifr_hir/src/lower/builtin_calls.rs:930) (uses `n.range()`),
   - TypeVar/PEP 695 forms at [typevar_annotations.rs:36-146](crates/sifr_hir/src/lower/typevar_annotations.rs:36) (all `name.range()`).
   The user-visible span will underline the literal token `bigint` in `bigint(1)`, not the whole call. That is the right pointer for "this identifier is a transition alias."

3. **Warn-before-validate ordering is harmless.** The warning is pushed before the keyword/arity/type-mismatch checks at [expressions.rs:858-892](crates/sifr_hir/src/lower/expressions.rs:858). On error, [lower_module_impl](crates/sifr_hir/src/lower/mod.rs:1153) returns `Err(ctx.errors)` and discards `ctx.warnings`, so:
   - `bigint("x")` → still a clean `TYPE_MISMATCH` to existing assertions in [expressions_tests.rs:1127](crates/sifr_hir/src/lower/expressions_tests.rs:1127),
   - `bigint()` → still a clean `CALL_WRONG_POSITIONAL_COUNT` at [expressions_tests.rs:1057](crates/sifr_hir/src/lower/expressions_tests.rs:1057),
   - `bigint(value=1)` → still a clean `CALL_UNEXPECTED_KEYWORD` at [expressions_tests.rs:1086](crates/sifr_hir/src/lower/expressions_tests.rs:1086).
   Warnings only surface when the lowering succeeds, which is the meaningful case.

4. **Test rebalancing keeps the canonical fixture clean.** The existing strong-assertion test [test_type_check_source_surfaces_bigint_transition_warning](crates/sifr_driver/src/tests/single_file_frontend.rs:255) still anchors `code` / `severity` / `message` / `message_template` / `args` / `primary_span` for `SIFR-INT-0011` on the annotation path (now `value: bigint = 1`). It still asserts `diagnostics.len() == 1`, which would *fail* if the constructor warning leaked into the annotation-only fixture — so the count assertion is now load-bearing rather than incidental.

5. **New constructor-only test reuses the dedicated helper.** [test_type_check_source_warns_for_bigint_constructor_call](crates/sifr_driver/src/tests/single_file_frontend.rs:284) goes through `assert_single_bigint_transition_warning`, matching the pattern established for `isinstance` / TypeVar / PEP 695 coverage at [single_file_frontend.rs:318-388](crates/sifr_driver/src/tests/single_file_frontend.rs:318). It asserts exactly one warning, line 2, primary span non-empty, code `INT_BIGINT_TRANSITION_ALIAS`, severity `Warning` — all the load-bearing properties of an `SIFR-INT-0011` emit, without re-asserting the message template (covered by the canonical fixture).

6. **Guardrails still pass.** `python3 scripts/check_hir_maintainability_guardrails.py` reports PASS; [expressions.rs](crates/sifr_hir/src/lower/expressions.rs) grew by one line (3797 → 3798) against the 3800 cap at [scripts/check_hir_maintainability_guardrails.py:18](scripts/check_hir_maintainability_guardrails.py:18). See concerns below.

7. **No regressions in adjacent paths.** Re-running the targeted suite with `cargo test -p sifr_driver bigint -- --quiet` produced `10 passed; 0 failed`, matching the user-reported run. `cargo fmt --check` is clean.

## Concerns and gaps

**Non-blocking** — none of the items below are correctness defects on this slice:

1. **No combined-form test for non-deduplication.** The slice intentionally lets both the annotation warning and the constructor warning fire when both are present (which is precisely why the previous canonical fixture was changed from `value: bigint = bigint(1)` to `value: bigint = 1` — that prior source would now emit two `SIFR-INT-0011` warnings, not one). There is currently no positive lock-in for that two-warning outcome. A future change that accidentally folds the constructor emit under the annotation site (or vice versa) would not be caught by any existing test. Adding a single `assert_eq!(diagnostics.len(), 2)` test against `value: bigint = bigint(1)` would close that gap. The user's invocation explicitly scoped this slice to "annotation-only and constructor-only coverage," so this is a follow-up nicety, not a blocker.

2. **Guardrail headroom is now 2 lines.** [crates/sifr_hir/src/lower/expressions.rs](crates/sifr_hir/src/lower/expressions.rs) sits at 3798/3800. The next non-trivial addition to this file will bust the cap, which means the next contributor will be paying decomposition cost on top of their actual change. Worth noting in PR text so the reviewer of the *next* expressions.rs change isn't surprised. Not a blocker for this slice — the guardrail is currently passing.

3. **Phase-tracker hygiene.** The bundled follow-up at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:429](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:429) still lists *"decide whether `bigint(...)` constructor calls should warn before public `bigint` removal"* as a deferred decision. This slice settles that decision (yes, they warn). Two paperwork items remain for the PR-opening step:
   - Add a per-slice bullet under [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:422](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:422) referencing this review and the eventual PR number, analogous to lines 425 and 428.
   - Strike the `bigint(...)` constructor clause from line 429 (or rephrase it as decided).
   This is the same hygiene gap flagged at [reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md:60](reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md:60); same classification — non-blocking, resolvable at PR opening.

4. **No e2e fail/pass fixture.** Consistent with concern 7 of [reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md:72](reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md:72) ("not warranted for a diagnostics-only slice"). The driver-level test exercises the diagnostic path end-to-end through `type_check_source`. No new fixture is necessary.

5. **Demo noise.** [demos/integer_safety/main.sifr](demos/integer_safety/main.sifr) uses `bigint(...)` 17 times alongside 9 `: bigint` annotations; this demo will now emit roughly double the previous `SIFR-INT-0011` count when type-checked. This is intentional under [internal_docs/integer_model.md:69](internal_docs/integer_model.md:69) and [internal_docs/integer_model.md:460](internal_docs/integer_model.md:460) ("`bigint`… may exist only as a temporary parser/type alias with deprecation diagnostics"). The e2e harness at [crates/sifr/tests/e2e.rs](crates/sifr/tests/e2e.rs) does not assert on warnings, and there are no Sifr-side `.snap` files that would drift, so this is informational only.

## Cross-checks performed

- **Span verification:** confirmed `call.func` is the bare `Expr::Name("bigint")` at this site (the `func_name` extraction at [expressions.rs:390-400](crates/sifr_hir/src/lower/expressions.rs:390) only reaches the body via the `Name` arm; `Attribute`, `Subscript`, etc. either alias-resolve to a different name or hit the "only simple function calls are supported" early-return).
- **Shadowing protection:** the `if !builtin_is_shadowed` block opens at [expressions.rs:424](crates/sifr_hir/src/lower/expressions.rs:424) and closes at line 1138; the new emit at line 857 is fully nested inside.
- **No unintended alias false positives:** `resolve_bare_python_compat_call_alias` at [compat_imports.rs:34-52](crates/sifr_hir/src/lower/compat_imports.rs:34) returns either an unrelated synthetic alias (`defaultdict`, `deque`, `Counter`) or the bare original name; it never maps anything *to* the string `"bigint"`. So no other identifier can collapse into this branch.
- **Bigdecimal is unaffected.** Only `func_name == "bigint"` warns — the `BigDecimal`-constructor branch at [expressions.rs:783](crates/sifr_hir/src/lower/expressions.rs:783) and the bigdecimal lowering helper are untouched.
- **Diagnostic registry:** `SIFR-INT-0011` is already registered with `Severity::Warning` at [crates/sifr_diagnostics/src/codes.rs:65](crates/sifr_diagnostics/src/codes.rs:65) and rendered with the canonical message at [crates/sifr_driver/src/frontend/module_lowering.rs:183](crates/sifr_driver/src/frontend/module_lowering.rs:183). Nothing new is required from the registry.
- **Local validation:** `cargo fmt --check` clean; `python3 scripts/check_hir_maintainability_guardrails.py` PASS; `cargo test -p sifr_driver bigint` 10/10 passing in 0.49s. Wider validation (`scripts/run_all_tests.sh --profile quick`) reported in the invocation as `report_signature=e1bf653aaa770517, wall_time=64.10s`, plus the targeted `cargo clippy -p sifr_hir -p sifr_driver -- -D warnings` clean.

## Verdict-blocking summary

- Constructor emit at [expressions.rs:857](crates/sifr_hir/src/lower/expressions.rs:857): **correct site, correct span, correctly scoped under the shadowing guard.**
- Test rebalance at [single_file_frontend.rs:256](crates/sifr_driver/src/tests/single_file_frontend.rs:256) and [single_file_frontend.rs:284](crates/sifr_driver/src/tests/single_file_frontend.rs:284): **annotation canonical fixture stays load-bearing; constructor regression test added at appropriate strictness.**
- Guardrails: **passing** (with the headroom note above).
- Validation reported in the invocation: **all green.**
- Documentation/phase-tracker follow-ups (concerns 2 and 3): **non-blocking, defer to PR opening.**

VERDICT: SATISFIED
