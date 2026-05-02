# Review Pass 3 — milestone_diag_7 slice 2: TypeCheckDiagnostic Symbol Deletion

Scope under review: same working-tree slice as passes 1 and 2 (delete `sifr_type_system::TypeError` / `TypeErrorKind`, replace with `TypeCheckDiagnostic { code: DiagnosticCode, message: String }`, rename `LowerCtx::type_error` → `LowerCtx::type_check_diagnostic`), plus the doc-only nits called out in pass 2:

- Three corrected `DiagnosticCode` constant names in the type-system surface table at [internal_docs/diagnostic_emission_inventory.md:71-74](../internal_docs/diagnostic_emission_inventory.md:71).
- Decimal-row wording widened from comparison-only to "arithmetic or comparison" at [internal_docs/diagnostic_emission_inventory.md:71](../internal_docs/diagnostic_emission_inventory.md:71), [:72](../internal_docs/diagnostic_emission_inventory.md:72), [:74](../internal_docs/diagnostic_emission_inventory.md:74).

This pass re-verifies the pass 2 nits are discharged, audits the surrounding inventory rows once more, and checks that the no-fallback / no-retired-symbol contract still holds end-to-end.

## Verdict

**SATISFIED — mergeable. I remain satisfied; no new blockers and no new findings.**

The two non-blocking pass 2 nits are now both addressed:

- **Pass 2 N1 (incorrect constant names)** — fixed. The four `TypeCheckDiagnostic { code: <CONST>, ... }` rows now name the actual Rust constants from [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs):
  - `DECIMAL_FLOAT_MIXED` (was `DECIMAL_MIXED_FLOAT`) → matches [codes.rs:47](../crates/sifr_diagnostics/src/codes.rs:47).
  - `DECIMAL_MIXED_WITH_BIGDECIMAL` (was `DECIMAL_MIXED_DECIMAL`) → matches [codes.rs:48](../crates/sifr_diagnostics/src/codes.rs:48).
  - `TYPE_INT_BIGINT_MIXED` (was `TYPE_NUMERIC_CONVERSION_REQUIRED`) → matches [codes.rs:33](../crates/sifr_diagnostics/src/codes.rs:33).
  - The unchanged rows (`TYPE_MISMATCH` at [codes.rs:29](../crates/sifr_diagnostics/src/codes.rs:29), `TYPE_UNSUPPORTED_OPERATOR` at [codes.rs:32](../crates/sifr_diagnostics/src/codes.rs:32)) were already correct in pass 2 and remain correct.

  I cross-checked each constant by `rg`-ing the names against [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs) and against the active emission sites in [crates/sifr_type_system/src/check.rs:36](../crates/sifr_type_system/src/check.rs:36), [:45](../crates/sifr_type_system/src/check.rs:45), [:57](../crates/sifr_type_system/src/check.rs:57), [:335](../crates/sifr_type_system/src/check.rs:335), [:344](../crates/sifr_type_system/src/check.rs:344), [:356](../crates/sifr_type_system/src/check.rs:356), [:404](../crates/sifr_type_system/src/check.rs:404). Every constant in the doc now resolves to a real symbol, and every active code in the table is actually constructed by an `Err(TypeCheckDiagnostic { code: <const>, ... })` site under [check.rs](../crates/sifr_type_system/src/check.rs). A future reader running `rg DECIMAL_FLOAT_MIXED` / `rg DECIMAL_MIXED_WITH_BIGDECIMAL` / `rg TYPE_INT_BIGINT_MIXED` now lands on real definitions and emission sites.

- **Pass 2 N2 (decimal wording loses arithmetic case)** — fixed. The two decimal-family rows and the int/bigint row now read:
  - [internal_docs/diagnostic_emission_inventory.md:71](../internal_docs/diagnostic_emission_inventory.md:71): "arithmetic or comparison mixing float with the decimal family" — covers both [check.rs:42-47](../crates/sifr_type_system/src/check.rs:42) (`type_check_binary_op`, message "cannot mix 'float' with decimal numeric types in arithmetic") and [check.rs:340-346](../crates/sifr_type_system/src/check.rs:340) (`type_check_comparison`, message "cannot compare 'float' with decimal numeric types").
  - [internal_docs/diagnostic_emission_inventory.md:72](../internal_docs/diagnostic_emission_inventory.md:72): "arithmetic or comparison mixing decimal and bigdecimal" — covers both [check.rs:32-38](../crates/sifr_type_system/src/check.rs:32) (`type_check_binary_op`) and [check.rs:331-338](../crates/sifr_type_system/src/check.rs:331) (`type_check_comparison`).
  - [internal_docs/diagnostic_emission_inventory.md:74](../internal_docs/diagnostic_emission_inventory.md:74): "int/bigint arithmetic or comparison requires conversion" — covers both [check.rs:53-60](../crates/sifr_type_system/src/check.rs:53) (`type_check_binary_op` arithmetic), [check.rs:352-358](../crates/sifr_type_system/src/check.rs:352) (`type_check_comparison` `==`/`!=`), and [check.rs:400-406](../crates/sifr_type_system/src/check.rs:400) (`type_check_comparison` `<`/`>`/`<=`/`>=`).

  Each wording change accurately matches the set of helpers that fire each code; nothing has been over- or under-stated.

The pass 2 verdict (SATISFIED / mergeable) therefore continues to hold, now with the cosmetic doc tightening folded in. The pass 1 cross-walk between `Err(TypeCheckDiagnostic { code, message })` construction sites in [check.rs](../crates/sifr_type_system/src/check.rs) and `SIFR-TYPE-*` / `SIFR-DECIMAL-*` codes is unchanged and still authoritative.

Author-reported validation (`cargo fmt --check` clean, `rg` confirming inventory constants match [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs)) is appropriate for a doc-only follow-up. I independently re-ran `cargo fmt --check` on the working tree and confirm it passes; I also re-ran the constant-existence `rg` cross-check and reproduced the author's result.

## Blockers

None.

## Major

None.

## Minor

None new in this pass.

The pass 2 N3 stylistic note (call sites still bind the helper return as `e: TypeCheckDiagnostic` rather than e.g. `diag`) was explicitly flagged as not worth changing in this slice and remains untouched. That is fine and consistent with the existing convention for `Result` matches in the file ([expressions.rs:373](../crates/sifr_hir/src/lower/expressions.rs:373), [:396](../crates/sifr_hir/src/lower/expressions.rs:396), [:518](../crates/sifr_hir/src/lower/expressions.rs:518), [:568](../crates/sifr_hir/src/lower/expressions.rs:568); [aug_assign_lowering.rs:316](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:316), [:322](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:322)). Not a blocker, not an action item for this slice.

## Cross-checks performed in this pass

1. **Constant correctness sweep.** Re-ran `rg -n "TYPE_MISMATCH|DECIMAL_FLOAT_MIXED|DECIMAL_MIXED_WITH_BIGDECIMAL|TYPE_UNSUPPORTED_OPERATOR|TYPE_INT_BIGINT_MIXED" crates/sifr_diagnostics/src/codes.rs` and confirmed each of the five constants printed in the inventory table corresponds to a real `pub const` declaration with the matching `SIFR-*` string ([codes.rs:29](../crates/sifr_diagnostics/src/codes.rs:29), [:32](../crates/sifr_diagnostics/src/codes.rs:32), [:33](../crates/sifr_diagnostics/src/codes.rs:33), [:47](../crates/sifr_diagnostics/src/codes.rs:47), [:48](../crates/sifr_diagnostics/src/codes.rs:48)) and is registered in the active codes list at [codes.rs:1326-1341](../crates/sifr_diagnostics/src/codes.rs:1326).

2. **Wording correctness sweep.** Walked every `Err(TypeCheckDiagnostic { code: <X>, ... })` site in [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) and confirmed each updated row in the inventory truthfully reflects which helpers (binary op, comparison, or both) emit the code. No row over-claims behavior the helpers do not implement, and no row under-claims by leaving out the arithmetic or comparison case.

3. **Symbol residue.** `rg -n "TypeError\b|TypeErrorKind\b|fn type_error\b|ctx\.type_error\b" --type rust crates/` returns only the four `"TypeError"` string-literal references in [crates/sifr_codegen](../crates/sifr_codegen) ([lib.rs:108](../crates/sifr_codegen/src/lib.rs:108), [stdlib_filter.rs:52](../crates/sifr_codegen/src/stdlib_filter.rs:52), [stmt_support_emitter.rs:40](../crates/sifr_codegen/src/stmt_support_emitter.rs:40), [intrinsic_method_emitters.rs:592](../crates/sifr_codegen/src/intrinsic_method_emitters.rs:592)) — Python builtin exception class names baked into stdlib codegen filtering, unrelated to the deleted Rust symbol. The pass 1 / pass 2 finding still holds: the deletion DoD bullet is fully satisfied.

4. **New-symbol scope.** `rg -n "TypeCheckDiagnostic|type_check_diagnostic" --type rust crates/` shows the new symbols are bounded to `sifr_type_system` (definition + emission sites + in-crate unit tests at [check.rs:643-725](../crates/sifr_type_system/src/check.rs:643)) and `sifr_hir/src/lower/{mod,expressions,aug_assign_lowering,diagnostic_transport_tests}.rs`. No leakage into `sifr_codegen`, `sifr_driver`, `sifr_diagnostics`, or `sifr` (the CLI). The residual transport stays confined to the operator-helper ↔ HIR boundary as intended.

5. **Behavior preservation.** No code or message string changed in this pass. The only changes between pass 2 and pass 3 are the three constant-name corrections and three wording widenings inside [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md). The compiled tree under `crates/` is byte-identical to pass 2's compiled tree apart from the doc file. `cargo fmt --check` passes (re-verified locally).

6. **No-fallback / no-retired-symbol contract.** Re-confirmed end-to-end:
   - `TypeCheckDiagnostic.code: DiagnosticCode` is non-`Option` ([sifr_type_system/src/lib.rs:33](../crates/sifr_type_system/src/lib.rs:33)).
   - Every `Err(TypeCheckDiagnostic { ... })` construction site in [check.rs](../crates/sifr_type_system/src/check.rs) supplies a literal `DiagnosticCode::…` constant — no variant-to-code mapper layer, no `Option`-typed code field, no message-substring classifier.
   - `LowerCtx::type_check_diagnostic` ([lower/mod.rs:231-233](../crates/sifr_hir/src/lower/mod.rs:231)) unconditionally calls `error_with_code(error.code, error.message)`. There is no `error()` (uncoded) fallback path on the type-check diagnostic edge.
   - The HIR transport test [diagnostic_transport_tests.rs:30-48](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:30) locks the contract by asserting `errors[0].code == Some(TYPE_UNSUPPORTED_OPERATOR)` after a `TypeCheckDiagnostic` is recorded, so any future regression that drops the code or routes through a fallback path would fail the unit test.

7. **Stale-doc audit (broader sweep).** `rg -n "TypeError|TypeErrorKind" --type md internal_docs/ issues/` shows only:
   - The expected forward-looking DoD/inventory references in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) (the issue records what the milestone is deleting and what the DoD is — these are historical/forward statements, not stale claims that the symbols still exist).
   - Python builtin `TypeError` exception class references in stdlib docs ([internal_docs/architecture.md:114](../internal_docs/architecture.md:114), [internal_docs/phases/07_stdlib_parity.md:247](../internal_docs/phases/07_stdlib_parity.md:247), several archived issues) — these are the Python language's exception name and are not affected by this slice.
   - The updated [internal_docs/diagnostic_emission_inventory.md:9](../internal_docs/diagnostic_emission_inventory.md:9) bullet that explicitly says the symbols have been deleted.

   No additional doc still claims the deleted symbols exist.

8. **Issue tracker entry.** [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:79](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:79) still records slice 2 as `in progress` with the original one-line summary; the pass 3 doc cleanup is part of the same slice and does not require a separate row. The forward-looking DoD bullet at [:1086](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1086) ("`sifr_type_system::TypeError` and `TypeErrorKind` symbols no longer exist") is now satisfied by the slice as a whole; the second adapter-deletion DoD bullet still belongs to a follow-up slice, exactly as inventory line [:66](../internal_docs/diagnostic_emission_inventory.md:66) names.

## Test coverage assessment (delta vs. pass 2)

No code change in this pass, so test coverage is unchanged from pass 2:

- `cargo test -p sifr_type_system` (in particular [check.rs:643-666](../crates/sifr_type_system/src/check.rs:643), [check.rs:712-725](../crates/sifr_type_system/src/check.rs:712)) asserts directly on the unwrapped `code: DiagnosticCode` shape and locks the non-`Option` invariant from inside the producing crate.
- [diagnostic_transport_tests.rs:30-48](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:30) locks the `LowerCtx::type_check_diagnostic` contract from the consumer side.
- `cargo test -p sifr --test e2e test_e2e_fail` continues to back-stop the `SIFR-TYPE-*` and `SIFR-DECIMAL-*` integration codes coming out of the renderer.

End-to-end coverage on `Err(TypeCheckDiagnostic { … })` from operator helper → `LowerCtx::type_check_diagnostic` → `LoweringError` with `Some(code)` → fail-fixture renderer-level `expect-error` assertion remains intact.

## Recommended follow-ups (not blocking this slice)

Unchanged from pass 2; recorded here for continuity:

1. Plan the next slice that retires `TypeCheckDiagnostic` itself — either by having the operator helpers return `(DiagnosticCode, Cow<'static, str>)` directly or by inverting control so HIR builds the canonical `SifrDiagnostic` at the call site with the AST span and structured args. The inventory at [internal_docs/diagnostic_emission_inventory.md:66](../internal_docs/diagnostic_emission_inventory.md:66) and [:113](../internal_docs/diagnostic_emission_inventory.md:113) names this as the next target.
2. Optionally rename the `e: TypeCheckDiagnostic` bindings at the HIR call sites to `diag` for readability when those sites are touched by the follow-up slice (pass 2 N3, stylistic only).

## Summary

Slice 2 still cleanly satisfies the symbol-deletion DoD bullet and the brief's "no fallback / no retired symbol" requirement. The pass 2 doc nits (incorrect constant names in three table rows, narrowed wording for decimal-family categories) are now fully addressed; every constant printed in the inventory table resolves to a real symbol in [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs), and every wording line truthfully reflects the helpers that fire each code in [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs).

Correctness, behavior, and the diagnostic registry are unaffected by this pass — it is a documentation-only follow-up.

I remain satisfied. No blockers, no new minor findings.
