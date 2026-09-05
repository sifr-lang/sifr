# Ad-Hoc World-Class Verification — Wave 2.1 Review (Pass 1)

Reviewer: code-review (agent)
Date: 2026-06-14
Scope: close all 20 `proposed_pr_slice: 2.1` stale-expectation rows in the `sifr_codegen` red-blocker inventory.

## Verdict

**Approve for merge** — no blockers. Two non-blocking follow-ups noted below.

The diff is the smallest possible test-only refresh: no compiler/codegen logic was touched. Each test now asserts the current normalized literal spelling (`N_i64` / `F_f64`) and the current tail-expression rendering for final returns. The inline snapshot for `renders_function_type_param_bounds` was blessed and the matching `.pending-snap` file removed. The inventory, triage doc, and phase issue were updated consistently with the new pass/fail counts.

## 1. Are the 20 closures genuine stale-expectation updates?

Yes — every test still exercises the same HIR input as before; only the expected substring/snapshot was refreshed.

Two flavors of refresh:

- **Literal spelling**: `(N as i64)` → `N_i64`, `(F.F as f64)` → `F.F_f64`. Mechanical and lossless; semantics unchanged.
- **Final-return tail-expression**: `return EXPR;` → `EXPR` (no keyword, no trailing `;`). This matches `prettyplease`'s tail-expression rendering for terminal `return` statements.

Spot-checks:

- `lowers_extended_math_intrinsics_via_registry` (`registry_extended_tests.rs:498`): `(2.0 as f64).powi` → `(2.0_f64).powi`. The negative assertions guarding `__p.len().min(__q.len())` and `__x.is_nan()` (sumprod / modf semantics) are untouched.
- `test_round_parenthesizes_cast_receiver` / `test_float_min_max_parenthesize_cast_receivers` (`async_runtime_codegen_tests.rs:323,388,392`): the parenthesization contract — the whole reason these tests exist — is preserved (`((1_i64) as f64).min(...)`). Only the inner literal spelling moved.
- `test_generate_rust_recursive_constructor_argument_wraps_optional_box_field` (`iterators_and_generators_codegen_tests.rs:253`): preserves the exact `Some(Box::new(Entry::new(...)))` nesting contract — only the integer-literal spelling changed.
- `test_structured_aug_assign_uses_string_and_list_methods` (`iterators_and_generators_codegen_tests.rs:714`): still asserts `s.push_str(`, `items.extend(vec![2_i64])`, and the negative assertions that the operator-form lowering (`s += `, `items += `) did not leak.
- `test_structured_stmt_path_rewrites_module_constant_name` and `test_generate_rust_multi_exports_non_main_items` (`classes_and_basics_codegen_tests.rs:354,527`): the `const NAME: i64 = ...` contract is preserved end-to-end; only the RHS literal spelling shifted.
- `renders_function_type_param_bounds` (`render_helpers.rs:319`): the bounds-rendering contract (`T: Clone + std::fmt::Display`) — the test's actual reason for existing — is unchanged. The bless removed the explicit `return value;` in favor of the `value` tail expression, which is the renderer's current contract.

No assertion was deleted, weakened to a no-op, or replaced by a counter-only check. The `lowering_stats.stmt_structured >= N` assertions on the structured-lowering tests are intact.

## 2. Pending-snap deletion

`crates/sifr_codegen/src/render/.render_helpers.rs.pending-snap` (deleted) contained exactly the JSON record of the inline snapshot that was blessed in `render_helpers.rs:319`:

- `new.snapshot`: `pub fn identity<T: Clone + std::fmt::Display>(value: T) -> T {\n    value\n}` — matches the new inline `assert_snapshot!`.
- `old.snapshot`: the prior `return value;` form.

Once an inline snapshot is updated to match `new`, insta no longer needs the `.pending-snap` queue file; the deletion is the correct follow-up and is required for `cargo test` to stay clean (otherwise insta will treat the pending entry as an outstanding decision).

## 3. Inventory consistency

`verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`:

- `test_result`: `{passed: 675, failed: 32, ignored: 0, total: 707}` — matches the reported local run.
- `red_blocker.failure_count`: `32` — matches `test_result.failed`.
- Row-level status counts: 20 closed, 32 open, 52 total. All 20 closed rows are `proposed_pr_slice: "2.1"`; the 32 open rows split 16 (2.2) / 6 (2.3) / 6 (2.4) / 4 (2.5), consistent with the triage doc.
- Each closed row's `test_id` corresponds to a test touched in this diff (1:1 mapping verified for all 20).

## 4. Docs

- `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md`: Wave 2.0 status updated to "merged in PR #2561"; a new "Wave 2.1 Implementation Notes" section documents scope, validation, and artifacts. The pass/fail delta (655→675 passed, 52→32 failed) is stated and matches the inventory.
- `plans/issues/active/codegen-test-triage.md`: Wave 2.0 result frozen in place for history, Wave 2.1 current result noted alongside, and the remaining 32 rows are correctly attributed to Waves 2.2–2.5.

Wording is accurate; risk framing is honest (Wave 2.1 is purely a test-expectation refresh, no compiler change).

## 5. Contract loss check

The one place where the new assertion is **strictly narrower** than the old one is the final-return tail-expression group. Six tests now assert only the expression payload (e.g. `Box::new(items.into_iter())`) where they used to assert `return Box::new(items.into_iter());`.

In every case the test's function body is a single `Return` statement, so the materialization can only appear in tail position — i.e. the contract is still effectively enforced by construction. The codegen-side proof that "final return renders as tail expression" is carried separately by the inline snapshot in `render_helpers.rs:319`. So while the textual assertion is narrower, no contract is actually unprotected by the suite as a whole.

This is a non-blocking follow-up, called out below.

## Non-blocking follow-ups

1. **Lock down tail-position contract on the affected return tests** (low priority). For the six tests where `return EXPR;` was reduced to `EXPR`, add an explicit `assert!(!rust_code.contains("return "))` (or an `EXPR\n}` suffix check) so that a future regression which re-introduces an explicit `return` statement is caught at the test site, not just by the render-helper snapshot. Affected:
   - `async_control_codegen_tests.rs:235` (`test_arithmetic_codegen`)
   - `iterators_and_generators_codegen_tests.rs:200` (`test_generate_rust_iterable_return_from_iterator_materializes_for_signature`)
   - `iterators_and_generators_codegen_tests.rs:210` (`test_generate_rust_iterator_return_consumes_local_list_binding`)
   - `iterators_and_generators_codegen_tests.rs:220` (`test_generate_rust_iterator_return_consumes_owned_param_binding`)
   - `structured_lowering_codegen_tests.rs:420` (`test_structured_stmt_path_handles_copy_typed_return_expr`)
   - (the render-helper snapshot already proves tail-expression at the renderer level)

2. **Soften brittle full-string match in `test_generate_rust_recursive_constructor_argument_wraps_optional_box_field`** (`iterators_and_generators_codegen_tests.rs:253`) — the assertion is one long substring containing both `Some(...)` nesting and three literal spellings. If the literal spelling changes again in a future renderer pass, this row will reappear in the red-blocker inventory. Consider splitting into structural assertions: one for the `Some(Box::new(Entry::new(...)))` nesting depth, one for the literal values. Not in Wave 2.1 scope; just note for whichever wave touches the wrap-optional-box-field renderer.

## Validation cross-check

The validation runs reported in the task brief match what the changed files imply:

- `cargo test -p sifr_codegen -- --nocapture`: 675/32/707 — matches inventory.
- `cargo test -p sifr_codegen render::render_helpers::tests::renders_function_type_param_bounds -- --exact --nocapture`: pass — confirms the snapshot bless.
- `cargo fmt --check`: pass — no formatting drift in the touched test files.
- `python3 scripts/check_file_size_guardrails.py`: pass — no monolith introduced; touched files are far below the 900-line cap.
- `scripts/run_all_tests.sh --profile create-pr`: pass — confirms no cross-suite regression. Wall-time 508.98s is acceptable on a cold e2e cache.

## Out-of-scope confirmation

No Wave 2.2 / 2.3 / 2.4 / 2.5 rows are touched in the diff, no non-Wave-2.1 test was modified, and no production code under `crates/sifr_codegen/src/` outside of test files was changed (the `render_helpers.rs` edit is to the `mod tests` block on line 319). The phase issue's stated scope is respected.
