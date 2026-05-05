# Review Pass 2 — milestone_diag_7 slice 2: TypeCheckDiagnostic Symbol Deletion

Scope under review: same working-tree slice as pass 1 (delete `sifr_type_system::TypeError` / `TypeErrorKind`, replace with `TypeCheckDiagnostic { code: DiagnosticCode, message: String }`, rename `LowerCtx::type_error` → `LowerCtx::type_check_diagnostic`), plus the follow-ups added since pass 1:

- New direct HIR transport test [crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:30](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:30) — `type_check_diagnostic_records_structured_identity`.
- Inventory rewrite [internal_docs/diagnostic_emission_inventory.md:9](../internal_docs/diagnostic_emission_inventory.md:9), [:62-74](../internal_docs/diagnostic_emission_inventory.md:62), [:113](../internal_docs/diagnostic_emission_inventory.md:113), [:369](../internal_docs/diagnostic_emission_inventory.md:369) — replaces all `TypeError` / `TypeErrorKind` mentions and rebuilds the type-system surface table around `TypeCheckDiagnostic`.
- Issue checklist [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:79](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:79) — slice 2 is recorded as in-progress.

This pass re-checks pass 1 findings (N1, N2, N3, N4), audits the new doc, and looks for behavioral regressions.

## Verdict

**SATISFIED — mergeable.**

All three pass 1 findings that called for action are discharged in the slice as it stands now:

- **N1 (stale inventory):** the section that previously catalogued `TypeError` / `TypeErrorKind` is rewritten to describe `TypeCheckDiagnostic` and the residual cleanup target ([:62-66](../internal_docs/diagnostic_emission_inventory.md:62)). The grep recommendation now reads as "symbols have been deleted" ([:9](../internal_docs/diagnostic_emission_inventory.md:9)). The "Type-check diagnostic forwarding" row in the public-code-mechanism table ([:113](../internal_docs/diagnostic_emission_inventory.md:113)) and the span/related-span note ([:369](../internal_docs/diagnostic_emission_inventory.md:369)) are both updated to the new surface name.
- **N2 (residual adapter):** the rewritten doc explicitly calls out that `TypeCheckDiagnostic` is itself the residual transport and names "retire `TypeCheckDiagnostic` itself" as the next cleanup target ([:66](../internal_docs/diagnostic_emission_inventory.md:66)). That keeps the slice's scope honest — the symbol-deletion DoD bullet is satisfied, the "adapter path is gone" bullet remains explicitly future work.
- **N3 (no direct test for `type_check_diagnostic`):** the new `type_check_diagnostic_records_structured_identity` test ([diagnostic_transport_tests.rs:30](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:30)) asserts the exact contract that was missing — `TypeCheckDiagnostic { code, message }` becomes a coded `LoweringError` with `Some(code)` and the literal message, single-error count. It correctly imports `sifr_type_system::TypeCheckDiagnostic`, which proves the public re-export at [sifr_type_system/src/lib.rs:32](../crates/sifr_type_system/src/lib.rs:32) survives.
- **N4 (`Option<DiagnosticCode>` was already dead):** no further action expected; confirmed once again by walking [check.rs](../crates/sifr_type_system/src/check.rs) — every error site uses a literal `DiagnosticCode::…` constant, and the field is now non-`Option`, so the dead branch is gone for good.

Validation lane the author reported (`cargo fmt --check`, `cargo test -p sifr_type_system`, `cargo test -p sifr_hir diagnostic_transport_tests -- --nocapture`, `cargo clippy -p sifr_type_system -p sifr_hir --no-deps -- -D warnings`, `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture`) is appropriate for the change surface and is unchanged from pass 1.

No blockers. Two minor doc-only nits below; both are non-blocking and can be folded into the slice PR or left for a follow-up tidy.

---

## Blockers

None.

---

## Major

None.

---

## Minor

### N1 — inventory uses incorrect `DiagnosticCode` constant names in three rows

The new type-system surface table lists `TypeCheckDiagnostic { code: <CONST>, ... }` examples whose constant names do not match the actual `sifr_diagnostics::DiagnosticCode` constants. The `SIFR-*` code strings are right; only the Rust constant names are wrong:

| inventory row ([internal_docs/diagnostic_emission_inventory.md:71-74](../internal_docs/diagnostic_emission_inventory.md:71)) | constant printed in doc | actual constant in [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs) |
| --- | --- | --- |
| float compared with decimal family | `DECIMAL_MIXED_FLOAT` | `DECIMAL_FLOAT_MIXED` ([codes.rs:47](../crates/sifr_diagnostics/src/codes.rs:47)) |
| decimal/bigdecimal comparison | `DECIMAL_MIXED_DECIMAL` | `DECIMAL_MIXED_WITH_BIGDECIMAL` ([codes.rs:48](../crates/sifr_diagnostics/src/codes.rs:48)) |
| int/bigint requires conversion | `TYPE_NUMERIC_CONVERSION_REQUIRED` | `TYPE_INT_BIGINT_MIXED` ([codes.rs:33](../crates/sifr_diagnostics/src/codes.rs:33)) |

The `TYPE_MISMATCH` ([codes.rs:29](../crates/sifr_diagnostics/src/codes.rs:29)) and `TYPE_UNSUPPORTED_OPERATOR` ([codes.rs:32](../crates/sifr_diagnostics/src/codes.rs:32)) rows are correct. The actual call sites in [check.rs:36](../crates/sifr_type_system/src/check.rs:36), [:45](../crates/sifr_type_system/src/check.rs:45), [:57](../crates/sifr_type_system/src/check.rs:57), [:335](../crates/sifr_type_system/src/check.rs:335), [:344](../crates/sifr_type_system/src/check.rs:344), [:356](../crates/sifr_type_system/src/check.rs:356), [:404](../crates/sifr_type_system/src/check.rs:404) use the right constants — only the doc is wrong.

This is non-blocking because the inventory is internal documentation and the `SIFR-*` codes themselves are correct, so any reader cross-referencing back into the code will still land on the right diagnostic. But the table is the canonical map between current construction and active code, so future readers running `rg DECIMAL_MIXED_FLOAT` will get zero hits and could conclude the row is stale. Suggested fix: either substitute the real constant names in those three rows, or replace the constant column with the active `SIFR-*` string only (the active-code column already has it).

### N2 — collapsed "Current message category" wording loses the arithmetic case for decimal-family mixing

The pass 1 inventory split decimal-family mixing into two rows: a `TypeMismatch`-row for "decimal/bigdecimal comparison" and an `InvalidOperator`-row for "decimal-family mixed arithmetic" with codes `SIFR-DECIMAL-0003` / `SIFR-DECIMAL-0004`. The pass 2 rewrite collapses to one row each per active code, but the surviving wording only mentions the comparison case:

- [internal_docs/diagnostic_emission_inventory.md:71](../internal_docs/diagnostic_emission_inventory.md:71): "float compared with decimal family" — but [check.rs:42-46](../crates/sifr_type_system/src/check.rs:42) emits `DECIMAL_FLOAT_MIXED` for arithmetic too ("cannot mix 'float' with decimal numeric types in arithmetic"), and [check.rs:343-347](../crates/sifr_type_system/src/check.rs:343) for comparison.
- [internal_docs/diagnostic_emission_inventory.md:72](../internal_docs/diagnostic_emission_inventory.md:72): "decimal/bigdecimal comparison" — but [check.rs:33-39](../crates/sifr_type_system/src/check.rs:33) emits `DECIMAL_MIXED_WITH_BIGDECIMAL` for arithmetic too.

Both decimal codes fire from arithmetic *and* comparison helpers; the doc currently implies only the comparison helper. Same kind of cosmetic drift as N1 — easy to repair with a wording tweak ("arithmetic and comparison with float / decimal family", "arithmetic and comparison mixing decimal/bigdecimal"). Not behaviorally relevant.

### N3 — `e: TypeCheckDiagnostic` parameter name reads as legacy `TypeError`

Stylistic only: the rename from `type_error` to `type_check_diagnostic` was applied at the helper boundary, but two call sites still spell the binding `e` (`Err(e) => { ctx.type_check_diagnostic(e); ... }`) — for example [expressions.rs:393-396](../crates/sifr_hir/src/lower/expressions.rs:393), [aug_assign_lowering.rs:315-322](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:315). `e` is fine and matches existing convention for `Result` matches in the file, so this is not worth changing in this slice; just noting it for completeness because the helper-level test ([diagnostic_transport_tests.rs:31](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:31)) uses the more readable `TypeCheckDiagnostic { code, message }` literal form, and it would not hurt to rename `e` to `diag` if the call sites get touched again in the follow-up that retires `TypeCheckDiagnostic`.

---

## Cross-checks performed in this pass

1. **Symbol residue.** Re-ran `rg -n "TypeError|TypeErrorKind|fn type_error\b|ctx\.type_error\b" --type rust crates/`. The only hits are the four `"TypeError"` string-literal references in [crates/sifr_codegen](../crates/sifr_codegen) ([lib.rs:108](../crates/sifr_codegen/src/lib.rs:108), [stdlib_filter.rs:52](../crates/sifr_codegen/src/stdlib_filter.rs:52), [stmt_support_emitter.rs:40](../crates/sifr_codegen/src/stmt_support_emitter.rs:40), [intrinsic_method_emitters.rs:592](../crates/sifr_codegen/src/intrinsic_method_emitters.rs:592)) — these are Python-builtin exception class names baked into stdlib codegen filtering, not the deleted Rust symbol. No regression.

2. **New symbol scope.** `rg -n "TypeCheckDiagnostic|type_check_diagnostic" crates/ --type rust` is bounded to `sifr_type_system` (definition + helper signatures + in-crate unit tests) and `sifr_hir/src/lower/{mod,expressions,aug_assign_lowering,diagnostic_transport_tests}.rs`. No leakage into `sifr_codegen`, `sifr_driver`, or `sifr_diagnostics`. Correct: the residual transport stays internal to the operator helpers ↔ HIR boundary.

3. **Behavior preservation.** I re-walked [check.rs](../crates/sifr_type_system/src/check.rs) and confirmed every error-emitting branch retains the same `(DiagnosticCode, message)` pair documented in the pass 1 cross-walk table. No code or message string changed in pass 2. The pass 1 cross-walk remains authoritative; nothing in the pass-2 commits invalidated it.

4. **New transport test mechanics.** [diagnostic_transport_tests.rs:30-48](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:30) constructs a `TypeCheckDiagnostic` with the full struct literal (so the `{ code, message }` field set is exercised at compile time and any future field addition forces an update here), then asserts `errors.len() == 1`, `errors[0].code == Some(TYPE_UNSUPPORTED_OPERATOR)`, and the literal message. Combined with the existing `error_with_code_records_structured_identity` and `legacy_error_records_no_structured_identity` tests, the transport-tests bucket now covers all three current emission paths into `LoweringError` (raw uncoded, coded direct, coded via `TypeCheckDiagnostic`). The brief's "direct HIR regression test for `LowerCtx::type_check_diagnostic`" is satisfied.

5. **Issue tracker entry.** [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:79](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:79) marks slice 2 as `in progress` with a one-line summary that matches the implemented surface ("delete the transitional `sifr_type_system::TypeError`/`TypeErrorKind` symbols and carry active type-check diagnostic code/message data directly from `sifr_type_system` into HIR lowering"). Consistent with the other slice rows.

6. **No-fallback / no-retired-symbol contract.** `TypeCheckDiagnostic.code: DiagnosticCode` is non-`Option` ([sifr_type_system/src/lib.rs:33](../crates/sifr_type_system/src/lib.rs:33)); `LowerCtx::type_check_diagnostic` ([mod.rs:231-233](../crates/sifr_hir/src/lower/mod.rs:231)) unconditionally calls `error_with_code(error.code, error.message)` — no `error()` (uncoded) fallback path remains for type-check diagnostics. The retired-symbol contract holds as established in pass 1.

---

## Test coverage assessment (delta vs. pass 1)

- New direct test for `type_check_diagnostic` closes pass 1 N3.
- Existing `cargo test -p sifr_type_system` assertions (e.g. [check.rs:643-666](../crates/sifr_type_system/src/check.rs:643), [check.rs:712-725](../crates/sifr_type_system/src/check.rs:712)) still assert directly on the unwrapped `code: DiagnosticCode` shape, locking the non-`Option` invariant from inside the producing crate too.
- `cargo test -p sifr --test e2e test_e2e_fail` continues to back-stop the `SIFR-TYPE-*` and `SIFR-DECIMAL-*` integration codes coming out of the renderer.

Coverage is now end-to-end on the path: `Err(TypeCheckDiagnostic { … })` from operator helper → `LowerCtx::type_check_diagnostic` → `LoweringError` with `Some(code)` → fail-fixture renderer-level `expect-error` assertion.

---

## Recommended follow-ups (not blocking this slice)

1. Fix the three constant names in [internal_docs/diagnostic_emission_inventory.md:71-74](../internal_docs/diagnostic_emission_inventory.md:71) so the table matches [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs) (N1).
2. Tighten the "Current message category" wording for the two decimal-family rows so it covers both arithmetic and comparison cases (N2).
3. Plan the next slice that retires `TypeCheckDiagnostic` itself — either by having the operator helpers return `(DiagnosticCode, Cow<'static, str>)` directly or by inverting control so HIR builds the canonical `SifrDiagnostic` at the call site with the AST span and structured args. The inventory at [internal_docs/diagnostic_emission_inventory.md:66](../internal_docs/diagnostic_emission_inventory.md:66) and [:113](../internal_docs/diagnostic_emission_inventory.md:113) already names this as the next target — good.

---

## Summary

Slice 2 cleanly satisfies the symbol-deletion contract and the brief's "no fallback / no retired symbol" requirement. Pass 1's three actionable findings (N1 inventory rewrite, N3 direct test, transport-test bucket completeness) are all addressed. Pass 1's N2 (residual adapter) is acknowledged in the rewritten inventory as the next cleanup target rather than silently absorbed. Pass 1's N4 was a confirmation only and does not need action.

The two remaining minor doc nits (incorrect constant names in three table rows, slightly narrowed wording for decimal-family categories) are cosmetic and can be folded into this PR or addressed in the follow-up that retires `TypeCheckDiagnostic`. They do not affect correctness, behavior, or the diagnostic registry.

I am satisfied. No blockers.
