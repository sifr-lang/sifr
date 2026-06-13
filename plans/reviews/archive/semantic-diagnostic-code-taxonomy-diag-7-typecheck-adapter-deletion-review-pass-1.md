# Review Pass 1 — milestone_diag_7 slice 3: TypeCheckDiagnostic Adapter Deletion

Scope under review: working-tree changes that delete the residual `sifr_type_system::TypeCheckDiagnostic` adapter struct (introduced in slice 2 as a thinner replacement for `TypeError`/`TypeErrorKind`) and replace it with direct `(DiagnosticCode, String)` failure data returned from the four operator helpers, destructured at the HIR call sites and routed through `LowerCtx::error_with_code`. The `LowerCtx::type_check_diagnostic` shim is also removed.

Files inspected:

- [crates/sifr_type_system/src/lib.rs](../crates/sifr_type_system/src/lib.rs) — `TypeCheckDiagnostic` struct + `Display`/`Error` impls deleted. No re-export remains.
- [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) — `Err(TypeCheckDiagnostic { code, message })` rewritten to `Err((code, message))` at all 14 emission sites; private `type TypeCheckResult = Result<Type, (DiagnosticCode, String)>` alias added; in-crate unit tests adjusted from `.code` to `.0`.
- [crates/sifr_hir/src/lower/mod.rs](../crates/sifr_hir/src/lower/mod.rs) — `type_check_diagnostic` shim deleted; `TypeCheckDiagnostic` import dropped.
- [crates/sifr_hir/src/lower/expressions.rs](../crates/sifr_hir/src/lower/expressions.rs) — four call sites (`lower_binop`, `lower_unaryop`, `lower_compare`, `lower_boolop`) destructure `(code, message)` and call `ctx.error_with_code(...)`.
- [crates/sifr_hir/src/lower/aug_assign_lowering.rs](../crates/sifr_hir/src/lower/aug_assign_lowering.rs) — four call sites updated; two of them previously dropped the code via `ctx.error(error.message)` (see Behavioral Upgrades below).
- [crates/sifr_hir/src/lower/container_literal_specialization.rs](../crates/sifr_hir/src/lower/container_literal_specialization.rs) — two call sites updated; both previously dropped the code via `ctx.error(error.message)` (see Behavioral Upgrades below).
- [crates/sifr_hir/src/lower/diagnostic_transport_tests.rs](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs) — `type_check_diagnostic_records_structured_identity` test removed (helper no longer exists); `TypeCheckDiagnostic` import dropped.
- [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md) — type-system section, code-mechanism row, and span/related-span note all forwarded one step; surface table rewritten from `TypeCheckDiagnostic { code, ... }` to `(DiagnosticCode::CONSTANT, message)` form.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) — slice-3 in-progress checkbox added at line 81.

---

## Verdict: SATISFIED

The slice cleanly accomplishes the claimed contract: `TypeCheckDiagnostic` and `type_check_diagnostic` are gone, the operator helpers return canonical diagnostic data directly, and the HIR ↔ type-system boundary now uses the same coded transport (`error_with_code`) that the rest of HIR uses. No blockers. There are three optional follow-ups (none of them blocking this slice) called out in the *Follow-ups* section below.

---

## What Holds Up

### Adapter deletion is real, not just renamed

`rg -n "TypeCheckDiagnostic|type_check_diagnostic" --type rust crates/` returns nothing in `sifr_type_system` or `sifr_hir`. Outside the `crates/` tree, the only matches are:

- Documentation (`internal_docs/diagnostic_emission_inventory.md`, `issues/...`) referring to the historical state.
- Slice-2 review files that describe the now-deleted symbol.
- `third_party/ruff/.../TypeCheckDiagnostics` (plural) in unrelated Ruff vendored code — name collision only, not an alias.

So the deletion is not just a `pub use` rename; the struct, its `Display`/`Error` impls, and the `LowerCtx` shim are all physically gone.

### No fallback / no Option-coded edge

Every `Err(...)` site in [check.rs](../crates/sifr_type_system/src/check.rs) constructs the tuple with a literal `DiagnosticCode::<CONST>` constant — there is no variant-to-code mapping, no `Option<DiagnosticCode>`, no message-substring classifier. The fact that the error type is `(DiagnosticCode, String)` (not `(Option<DiagnosticCode>, String)`) makes the "code is required" contract a compile-time guarantee at every emission site:

- [check.rs:32-36](../crates/sifr_type_system/src/check.rs:32) `DECIMAL_MIXED_WITH_BIGDECIMAL`
- [check.rs:41-45](../crates/sifr_type_system/src/check.rs:41) `DECIMAL_FLOAT_MIXED`
- [check.rs:53-57](../crates/sifr_type_system/src/check.rs:53) `TYPE_INT_BIGINT_MIXED`
- Multiple `TYPE_UNSUPPORTED_OPERATOR` and `TYPE_MISMATCH` sites across the binary/comparison/unary/bool helpers.

HIR records the data through `error_with_code`, which unconditionally sets `LoweringError.code = Some(code)` ([mod.rs:231-238](../crates/sifr_hir/src/lower/mod.rs:231)). There is no code-less edge from operator-helper failure → HIR.

### `TypeCheckResult` alias is not a residual adapter

[check.rs:7](../crates/sifr_type_system/src/check.rs:7) introduces `type TypeCheckResult = Result<Type, (DiagnosticCode, String)>` as a private (no `pub`) module-scoped abbreviation. It is purely cosmetic — callers in `sifr_hir` see and destructure the bare tuple `(DiagnosticCode, String)`. This is not a transport or adapter type. Acceptable, and equivalent in effect to inlining the type in each helper signature. The slice's "no residual adapter" requirement holds.

### Doc updates land cleanly and accurately

- The type-system section at [inventory.md:62-66](../internal_docs/diagnostic_emission_inventory.md:62) correctly describes the new shape and explicitly names the next cleanup target (canonical `SifrDiagnostic` with declared args + spans from the semantic owner). The slice's DoD bullet ("type-check adapter path is gone") is now actually satisfied — what remained at the end of slice 2 was the `TypeCheckDiagnostic` adapter; with it deleted, the only remaining work is the args+spans uplift, which is correctly scoped to a future slice.
- The five surface-table rows ([:68-74](../internal_docs/diagnostic_emission_inventory.md:68)) name the actual `sifr_diagnostics::DiagnosticCode` constants. I cross-checked each constant by grepping [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs) and against the active emission sites in [check.rs:32](../crates/sifr_type_system/src/check.rs:32), [:41](../crates/sifr_type_system/src/check.rs:41), [:53](../crates/sifr_type_system/src/check.rs:53), [:381](../crates/sifr_type_system/src/check.rs:381), and the various `TYPE_UNSUPPORTED_OPERATOR` paths. Every constant resolves; every active code in the table is actually constructed by a real `Err((<const>, ...))` site.
- The "Public-code mechanisms to remove" row at [:113](../internal_docs/diagnostic_emission_inventory.md:113) updates the mechanism description from "calls `ctx.type_check_diagnostic(error)`" to "destructures `(code, message)` and calls `ctx.error_with_code(code, message)`", which is now factually correct.
- The span/related-span note at [:369](../internal_docs/diagnostic_emission_inventory.md:369) is updated to "Type-check helper failure sites" and still correctly flags args+spans as outstanding work.
- The issue checkbox at [:81](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:81) is the in-progress marker — appropriate while the slice is unmerged.

### Test coverage is intact at the boundary that survived

- `cargo test -p sifr_type_system` passes (asserted): the in-crate unit tests at [check.rs:631-713](../crates/sifr_type_system/src/check.rs:631) still lock the helper contract by asserting the canonical code via `.0` instead of `.code`. The tested codes (`TYPE_UNSUPPORTED_OPERATOR`, `TYPE_INT_BIGINT_MIXED`) are the same set tested before — no coverage was lost.
- `cargo test -p sifr_hir diagnostic_transport_tests`: `error_with_code_records_structured_identity` and `legacy_error_records_no_structured_identity` continue to lock the LowerCtx → LoweringError edge that this slice now routes operator-helper failures through. The slice-2-specific `type_check_diagnostic_records_structured_identity` test is correctly removed because the helper it tested no longer exists.
- E2E coverage of operator-helper-driven codes is intact:
  - `SIFR-TYPE-0005` (TYPE_UNSUPPORTED_OPERATOR) — `optional_arithmetic_without_narrowing.sifr`, `optional_arithmetic_reachable_after_partial_narrowing.sifr`.
  - `SIFR-TYPE-0006` (TYPE_INT_BIGINT_MIXED) — `bigint_int_mixed_arithmetic.sifr`, `bigint_int_mixed_comparison.sifr`.
  - `SIFR-DECIMAL-0003` (DECIMAL_FLOAT_MIXED) — `decimal_float_mixed_arithmetic.sifr`.
  - `SIFR-DECIMAL-0004` (DECIMAL_MIXED_WITH_BIGDECIMAL) — `decimal_bigdecimal_mixed_arithmetic.sifr`, `decimal_forbidden_mixed_arithmetic_seeded.sifr`.

  Each of these fixtures asserts the same code that the rewritten helper returns, so the canonical-identity contract is end-to-end exercised.

### `nested_function_inference.rs` discard site is unaffected

[nested_function_inference.rs:1440-1441](../crates/sifr_hir/src/lower/nested_function_inference.rs:1440) calls `type_check_binary_op(...).unwrap_or_else(|_| infer_numeric_result_type(...))`. This is type *inference* fallback, not error reporting, so it intentionally drops the error closure. The signature change is transparent here (the closure binds the whole error tuple as `_` either way). No behavioral change. Confirmed correct.

---

## Behavioral Upgrades Captured Incidentally By This Slice

In four formerly-uncoded sites, the previous code called `ctx.error(error.message)` — explicitly discarding the `TypeCheckDiagnostic.code` and emitting an uncoded `LoweringError`. By destructuring `(code, message)` and routing through `ctx.error_with_code(code, message)`, this slice silently *upgrades* those sites to carry the canonical `SIFR-TYPE-*` / `SIFR-DECIMAL-*` code that the helper produced.

The four sites:

1. [aug_assign_lowering.rs:130-135](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:130) — nested subscript aug-assign type mismatch (`obj[a][b] += rhs`).
2. [aug_assign_lowering.rs:199-204](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:199) — attribute-subscript aug-assign type mismatch (`obj.field[i] += rhs`).
3. [container_literal_specialization.rs:140-141](../crates/sifr_hir/src/lower/container_literal_specialization.rs:140) — list element aug-assign in `validate_subscript_augassign_target` (`xs[i] += rhs`).
4. [container_literal_specialization.rs:153-168](../crates/sifr_hir/src/lower/container_literal_specialization.rs:153) — dict value aug-assign (`d[k] += rhs`) in the non-empty-dict-specialization branch.

This is a strictly better outcome — previously the type-system helper computed e.g. `TYPE_UNSUPPORTED_OPERATOR`, the call site discarded it, and the resulting `LoweringError` reached the renderer with `code = None`. Now the canonical code rides through. It is in keeping with the spirit of the slice (no code-dropping fallback edges anywhere on the operator-helper boundary).

Two notes on this:

- It is technically *out of literal scope* of "delete `TypeCheckDiagnostic`". Because the destructured tuple makes the code visible at the call site, the choice between `ctx.error(message)` and `ctx.error_with_code(code, message)` becomes obvious — consistency with the other six call sites argues for `error_with_code`. I think this is fine for the slice and is correctly handled. Just worth calling out in the PR description so reviewers don't read it as an unintentional behavioral drift.
- There is **no e2e fixture coverage** for any of the four upgraded sites today. `rg "+=" crates/sifr/tests/e2e/fail/` returns only `nested_function_recursive_nonlocal_unsupported.sifr` and `own_parameter_augassign_requires_mut.sifr`, neither of which exercises a type mismatch on a subscript aug-assign LHS. So although the upgrade is correct, no regression test would catch a future re-introduction of the "drop the code" bug. See follow-up F1.

---

## Findings (Non-Blocker)

### N1 — TYPE_MISMATCH inventory row fixture column is imprecise (pre-existing)

[inventory.md:70](../internal_docs/diagnostic_emission_inventory.md:70) maps `(DiagnosticCode::TYPE_MISMATCH, message)` to `crates/sifr/tests/e2e/fail/type_mismatch.sifr` and `union_type_mismatch.sifr`. But the operator helpers' only `TYPE_MISMATCH` emission is the equality-comparison fallback at [check.rs:381-388](../crates/sifr_type_system/src/check.rs:381) ("cannot compare 'X' and 'Y' with =="). The two listed fixtures actually exercise general assignment-shape `TYPE_MISMATCH` (e.g. `x: int = "hello"`), which is emitted by HIR lowering paths *outside* the operator helpers. So the row's fixture column is documenting "fixtures that test the SIFR-TYPE-0002 code in general" rather than "fixtures that exercise the helper's TYPE_MISMATCH emission specifically".

This is a doc-only imprecision carried over from slice 2 (slice-2 review pass-3 signed off on the constant names but did not re-walk the fixture column for behavioral specificity). Not introduced by slice 3 and not a blocker. Worth fixing in a follow-up by either (a) adding a fixture that exercises `int == str` or similar comparison-fallback emission from the helper, or (b) softening the column header from "Representative fixture" to "Representative fixture for the SIFR code (may be emitted from non-helper paths)".

### N2 — Lost dedicated boundary test for the operator-helper → HIR transport edge

Slice 2 added `type_check_diagnostic_records_structured_identity` ([diagnostic_transport_tests.rs](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs) historical line 30) specifically to lock the `TypeCheckDiagnostic` → coded `LoweringError` contract from the consumer side. With `TypeCheckDiagnostic` deleted, that test correctly went away, but no replacement was added. The contract is still locked end-to-end via:

- The in-crate unit tests at [check.rs:631-713](../crates/sifr_type_system/src/check.rs:631) (helper returns the right tuple).
- The transport tests at [diagnostic_transport_tests.rs:5-27](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:5) (`error_with_code` records the code).
- The e2e fail fixtures listed above (the renderer sees the canonical code).

Composing those three is sufficient to assert "operator-helper failure ⇒ coded `LoweringError`", but no single test asserts the *destructure-and-route* step at the HIR call site. A two-line test that calls (e.g.) `lower_binop` for `int - str` and asserts the resulting `LoweringError` has `Some(TYPE_UNSUPPORTED_OPERATOR)` would lock the new boundary contract directly and would survive future restructuring of the operator helpers. Optional, not a blocker.

### N3 — `TypeCheckResult` alias readability

[check.rs:7](../crates/sifr_type_system/src/check.rs:7) `type TypeCheckResult = Result<Type, (DiagnosticCode, String)>` is a private cosmetic alias. A reader greppingfor the helper return type sees `TypeCheckResult` and has to glance one line up to learn the shape. Inlining the bare `Result<Type, (DiagnosticCode, String)>` in each helper signature is a wash readability-wise (more characters, but no jump-to-definition). I would slightly prefer the inline form because it surfaces the tuple shape at the call boundary, but the current alias is fine and idiomatic. Not a blocker.

---

## Follow-Ups (Not Blocking This Slice)

- **F1 (most useful).** Add at least one e2e fail fixture per upgraded site:
  - `xs[0] += "string"` where `xs: list[int]` — covers [container_literal_specialization.rs:140-141](../crates/sifr_hir/src/lower/container_literal_specialization.rs:140) (list element).
  - `d["k"] += 3.14` where `d: dict[str, int]` — covers [container_literal_specialization.rs:153-168](../crates/sifr_hir/src/lower/container_literal_specialization.rs:153) (dict value, non-empty-specialization branch).
  - `m[0][0] += "x"` where `m: list[list[int]]` — covers [aug_assign_lowering.rs:130-135](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:130) (nested subscript).
  - `obj.field[i] += "x"` with `field: list[int]` — covers [aug_assign_lowering.rs:199-204](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:199) (attribute-subscript).

  Each `expect-error` line should assert the canonical `SIFR-TYPE-0005` (or whichever code the helper actually produces for the operands), so future regressions to the code-dropping bug fail the e2e fail suite.

- **F2.** Fix N1 — either point the `TYPE_MISMATCH` inventory row at a comparison-fallback fixture or relax the column header to acknowledge that the fixtures listed exercise the SIFR code in general, not necessarily the helper site. Doc-only.

- **F3 (optional).** Add a HIR-level transport test asserting "operator-helper Err ⇒ `LoweringError` with `Some(code)`" to replace the slice-2 boundary test that was removed (see N2). Two lines of code, rounds out the coverage shape.

- **F4 (next planned slice).** The cleanup target named at [inventory.md:66](../internal_docs/diagnostic_emission_inventory.md:66) — moving the helper failures from `(DiagnosticCode, String)` to full canonical `SifrDiagnostic` with declared args and AST spans owned by the HIR call site — is the natural next slice. Either invert control so HIR builds the diagnostic at the call site (helper returns only the typed result), or have the helper return an explicit `(DiagnosticCode, MessageTemplate, &[Arg])` triple that HIR composes against the AST span. The latter is closer to the registry shape; the former is more localized.

---

## Validation Cross-Check

The author's stated validation lane covers exactly the code that changed:

- `cargo fmt --check` — appropriate; no whitespace drift.
- `cargo test -p sifr_type_system` — covers the rewritten helper signatures and the unit tests that assert `.0 == EXPECTED_CODE`.
- `cargo test -p sifr_hir diagnostic_transport_tests -- --nocapture` — locks the LowerCtx → LoweringError transport (which is the survival path for the destructured tuple).
- `cargo clippy -p sifr_type_system -p sifr_hir --no-deps -- -D warnings` — covers all crates that touch the new tuple shape. Adequate for the change surface.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` — exercises the canonical-code contract end-to-end through the renderer; non-trivial because the e2e harness rejects non-canonical codes (per slice 5).
- `rg "TypeCheckDiagnostic|type_check_diagnostic" crates/sifr_type_system/src crates/sifr_hir/src -g '*.rs'` — confirms the symbols are physically gone from the affected crates.

I would additionally recommend running `scripts/run_all_tests.sh --profile quick` before opening the PR (the AGENTS.md gate). The author's earlier validation lane note in the slice-2 issue checkbox already mentions running it; assuming the same is run here, the local validation lane is appropriate for the change surface.

---

## Bottom Line

`TypeCheckDiagnostic` and `type_check_diagnostic` are gone, with no residual adapter, no fallback path, no `Option<DiagnosticCode>` edge, and no message-substring classifier. The operator helpers and HIR boundary now use a flat `(DiagnosticCode, String)` tuple destructured at the call site and routed through the same `error_with_code` transport that the rest of HIR uses. As a side benefit, four formerly-uncoded subscript / attribute-subscript aug-assign sites now correctly carry their canonical codes through to the renderer.

Reviewer is satisfied. The three optional follow-ups (F1 fixture coverage for the upgraded sites, F2 inventory-row fixture-column tightening, F3 HIR-level boundary test) are non-blocking polish — they can land in this PR or in a small cleanup PR after merge. F4 is the next slice and is correctly named in the inventory's cleanup-target section.
