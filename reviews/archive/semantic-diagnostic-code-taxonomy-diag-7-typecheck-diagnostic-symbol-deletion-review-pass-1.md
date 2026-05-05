# Review Pass 1 — milestone_diag_7 slice 2: TypeCheckDiagnostic Symbol Deletion

Scope under review: working-tree changes that delete the transitional `sifr_type_system::TypeError` / `TypeErrorKind` symbols, introduce `sifr_type_system::TypeCheckDiagnostic { code: DiagnosticCode, message: String }` as the canonical operator/type-check error payload, and rename the HIR consumer hook from `LowerCtx::type_error` to `LowerCtx::type_check_diagnostic`.

Files touched:

- [crates/sifr_type_system/src/lib.rs](../crates/sifr_type_system/src/lib.rs) — symbol replacement.
- [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) — all `Err(TypeError { … })` rewritten to `Err(TypeCheckDiagnostic { … })`; in-crate unit tests updated.
- [crates/sifr_hir/src/lower/mod.rs](../crates/sifr_hir/src/lower/mod.rs) — import + helper rename.
- [crates/sifr_hir/src/lower/aug_assign_lowering.rs](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:316), [crates/sifr_hir/src/lower/expressions.rs](../crates/sifr_hir/src/lower/expressions.rs:373) — call-site rename only.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:79](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:79) — slice 2 entry added as in-progress.

Read against the brief: delete the transitional `TypeError`/`TypeErrorKind` symbols; carry an active `DiagnosticCode` and message directly out of the operator helpers; keep call-sites in HIR explicit; do not change diagnostic text, codes, or `LoweringError` shape.

## Verdict

**MERGEABLE as a clean, narrow reviewable slice toward the milestone_diag_7 DoD bullet "`sifr_type_system::TypeError` and `TypeErrorKind` symbols no longer exist."** No correctness regressions, no behavioral drift, and no hidden TypeError adapter remains in either crate.

One non-blocker: the [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md) wording is now stale (it still names `TypeError`/`TypeErrorKind` and recommends "delete these symbols" as a future step) — see N1 below. The milestone_diag_7 DoD additionally says "the type-system adapter path is gone; type-checking code emits or returns canonical diagnostics directly" — `TypeCheckDiagnostic` is itself a thinner transitional adapter, so the *symbol-deletion* DoD bullet is satisfied by this slice but the *adapter-path-deleted* bullet still has a residual hop. That is consistent with the explicit slice scope ("no broad `LoweringError` cleanup in this slice") and is appropriate work for a follow-up slice — see N2.

Validation reported by the author (`cargo fmt --check`, `cargo test -p sifr_type_system`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo clippy -p sifr_type_system -p sifr_hir --no-deps -- -D warnings`, `cargo test -p sifr --test e2e test_e2e_fail`, `rg TypeError/TypeErrorKind` clean under both crates) is appropriate for the change surface. I cross-checked the `rg` claim independently and confirm there are no residual references in `crates/sifr_type_system/src` or `crates/sifr_hir/src`. The four `"TypeError"` string-literal matches under [crates/sifr_codegen](../crates/sifr_codegen) ([lib.rs:108](../crates/sifr_codegen/src/lib.rs:108), [stdlib_filter.rs:52](../crates/sifr_codegen/src/stdlib_filter.rs:52), [stmt_support_emitter.rs:40](../crates/sifr_codegen/src/stmt_support_emitter.rs:40), [intrinsic_method_emitters.rs:592](../crates/sifr_codegen/src/intrinsic_method_emitters.rs:592)) are unrelated — they reference Python's builtin `TypeError` exception class name as part of stdlib-filter codegen, not the deleted Rust symbol.

---

## Blockers

None.

---

## Major

None.

---

## Minor

### N1 — `internal_docs/diagnostic_emission_inventory.md` still names the deleted symbols

The inventory doc that catalogues this exact migration is now out of sync with the code:

- [internal_docs/diagnostic_emission_inventory.md:9](../internal_docs/diagnostic_emission_inventory.md:9) recommends `rg "TypeErrorKind::" crates/sifr_type_system/src crates/sifr_hir/src -g '*.rs'` to find construction sites — that ripgrep now returns zero matches.
- [internal_docs/diagnostic_emission_inventory.md:64](../internal_docs/diagnostic_emission_inventory.md:64): "`sifr_type_system::TypeError` and `TypeErrorKind` are transitional only … Migration should replace them with direct domain helper calls in HIR/type-checking code, then delete these symbols." That deletion has now happened; the guidance reads as future work.
- [internal_docs/diagnostic_emission_inventory.md:68-80](../internal_docs/diagnostic_emission_inventory.md:68) — the per-`TypeErrorKind`-variant routing table references variant names (`TypeErrorKind::TypeMismatch`, `TypeErrorKind::InvalidOperator`, etc.) that no longer exist in the source.
- [internal_docs/diagnostic_emission_inventory.md:119, 375](../internal_docs/diagnostic_emission_inventory.md:119) — both still describe `TypeError`-forwarding sites.

This is non-blocking because the inventory is internal documentation, not load-bearing for compiler correctness, and the same milestone (`milestone_diag_7`) is what's discharging that inventory. But it would be cleaner to either (a) update lines 9, 64, 68-80, 119, and 375 to refer to `TypeCheckDiagnostic` and the new call sites, or (b) note in the same slice that the inventory rows are now retired and strike them. Doing this in slice 2 keeps the doc consistent with the symbol state at every commit boundary.

### N2 — `TypeCheckDiagnostic` is itself a (thinner) adapter

The milestone_diag_7 DoD lists two bullets relevant to this slice:

1. "`sifr_type_system::TypeError` and `TypeErrorKind` symbols no longer exist." ✅ satisfied.
2. "The type-system adapter path is gone; type-checking code emits or returns canonical diagnostics directly." Partially satisfied: `TypeCheckDiagnostic` *is* a canonical (code, message) pair, but it is still a distinct type from `sifr_diagnostics::DiagnosticCode`-keyed `LoweringError` / `SifrDiagnostic`, and HIR still routes it through `LowerCtx::type_check_diagnostic` → `error_with_code` ([lower/mod.rs:231-242](../crates/sifr_hir/src/lower/mod.rs:231)).

The slice brief explicitly scopes out that second cleanup ("no broad `LoweringError` cleanup in this slice"), and I agree that a follow-up slice — where the operator helpers either return `(DiagnosticCode, String)` directly or are inverted so the HIR site builds the canonical diagnostic — is the right place for it. Worth flagging here so the milestone tracker doesn't mark milestone_diag_7 done after this slice on the strength of the symbol-deletion bullet alone.

### N3 — No direct unit test for `LowerCtx::type_check_diagnostic`

[crates/sifr_hir/src/lower/diagnostic_transport_tests.rs](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs) has focused tests for `error` (None code) and `error_with_code` (Some code) but nothing for the new `type_check_diagnostic` shim. The shim is a one-liner that delegates to `error_with_code`, so the contract is trivial — and integration coverage exists via the type-system unit tests in [check.rs:644-725](../crates/sifr_type_system/src/check.rs:644) plus the e2e fail corpus. So this is a stylistic minor only. A two-line test would lock in "TypeCheckDiagnostic always becomes a coded LoweringError" as part of the same transport-test bucket and would survive future refactors without re-relying on operator-level tests.

### N4 — `code` field tightening is correct; old `Option<DiagnosticCode>` was already dead

For the record (because it surprised me at first): the old `TypeError.code: Option<DiagnosticCode>` was *only* ever constructed with `Some(...)` in the codebase — every `Err(TypeError { … })` site in [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) used `Some(DiagnosticCode::…)`. Consequently the `if let Some(code) = error.code { … } else { self.error(error.message) }` branch in the previous `LowerCtx::type_error` was dead. The new `TypeCheckDiagnostic.code: DiagnosticCode` (non-`Option`) is therefore a sound type-level tightening, not a behavioral change, and it correctly forecloses the bug class where a future `TypeCheckDiagnostic` could regress to a code-less `LoweringError`. Good.

---

## Behavior preservation cross-check

For every error branch in [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) I cross-walked old → new:

| Helper | Branch | Old code | New code | Old message | New message | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `type_check_binary_op` | decimal/bigdecimal mix | `DECIMAL_MIXED_WITH_BIGDECIMAL` | same | "cannot mix 'decimal' and 'bigdecimal' …" | unchanged | `kind: InvalidOperator` payload dropped — never read |
| `type_check_binary_op` | float/decimal-family mix | `DECIMAL_FLOAT_MIXED` | same | unchanged | unchanged | same |
| `type_check_binary_op` | int/bigint mix | `TYPE_INT_BIGINT_MIXED` | same | unchanged | unchanged | same |
| `type_check_binary_op` | `+` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_binary_op` | `-`/`*` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_binary_op` | `/` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_binary_op` | `//`/`%` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_binary_op` | `**` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_binary_op` | `&`/`\|`/`^` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_binary_op` | `<<`/`>>` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_binary_op` | unknown op | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_comparison` | decimal/bigdecimal mix | `DECIMAL_MIXED_WITH_BIGDECIMAL` | same | unchanged | unchanged | `kind: TypeMismatch` payload dropped |
| `type_check_comparison` | float/decimal-family mix | `DECIMAL_FLOAT_MIXED` | same | unchanged | unchanged | same |
| `type_check_comparison` | `==`/`!=` int/bigint | `TYPE_INT_BIGINT_MIXED` | same | unchanged | unchanged | same |
| `type_check_comparison` | `==`/`!=` fallthrough | `TYPE_MISMATCH` | same | unchanged | unchanged | same |
| `type_check_comparison` | `<`/`>`/`<=`/`>=` int/bigint | `TYPE_INT_BIGINT_MIXED` | same | unchanged | unchanged | same |
| `type_check_comparison` | `<`/`>`/`<=`/`>=` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_comparison` | unknown op | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_unary_op` | `-`/`+` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_unary_op` | `not` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_unary_op` | `~` fallthrough | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_unary_op` | unknown op | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_bool_op` | non-truthy operands | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |
| `type_check_bool_op` | unknown op | `TYPE_UNSUPPORTED_OPERATOR` | same | unchanged | unchanged | same |

Every branch preserves both code and message. The `kind: TypeErrorKind::*` payloads were the only structured datum being deleted; I confirmed there are no readers of `kind` in `crates/sifr_hir`, `crates/sifr_codegen`, or `crates/sifr_driver` — the previous `LowerCtx::type_error` never inspected `kind`, and no other crate consumed `TypeErrorKind` ([rg `InvalidOperator|TypeMismatch|UndefinedVariable|UndefinedFunction|WrongArgumentCount|UseAfterMove|MissingTypeAnnotation|NotCallable` returns zero matches](#) under the three crates). So the `kind` payload was already dead data. Deleting it is correct and reduces a real source of drift between the type-system enum and the diagnostic registry.

The HIR call-site renames are mechanical:

- [aug_assign_lowering.rs:316](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:316) and [aug_assign_lowering.rs:322](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:322): both still inside the `(Type::Str, Type::Str)`/`(Type::List(_), Type::List(_))`/`(Type::Bytes, Type::Bytes)` short-circuit guard, so the early-accept paths for `+=` on those types are preserved unchanged.
- [expressions.rs:373](../crates/sifr_hir/src/lower/expressions.rs:373): still gated behind the `Type::Class { methods }` `__add__`/`__sub__`/etc. dunder lookup. Operator overloads for class types still suppress the `TYPE_UNSUPPORTED_OPERATOR` emission — no regression.
- [expressions.rs:518](../crates/sifr_hir/src/lower/expressions.rs:518): still gated behind the `__eq__`/`__lt__` dunder check for class operands in `lower_compare`. No regression.
- [expressions.rs:396](../crates/sifr_hir/src/lower/expressions.rs:396) and [expressions.rs:568](../crates/sifr_hir/src/lower/expressions.rs:568): plain emission, no overload gating, unchanged behavior.

## Symbol deletion verification

I independently re-ran the residual-search:

```
rg -n "TypeError|TypeErrorKind|type_error\b" --type rust crates/sifr_type_system crates/sifr_hir
```

Single match: [crates/sifr_hir/src/lower/type_alias_tests.rs:23](../crates/sifr_hir/src/lower/type_alias_tests.rs:23) — `fn test_recursive_type_alias_name_resolves_without_unknown_type_error`. That is a test function whose name describes a Sifr-level "unknown type" diagnostic regression (the `_error` suffix is part of the English phrase "unknown-type error"); it is not a reference to the Rust struct. No action needed.

`rg -n "TypeCheckDiagnostic|type_check_diagnostic"` returns the expected 35 hits across [sifr_type_system/src/lib.rs](../crates/sifr_type_system/src/lib.rs), [sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs), [sifr_hir/src/lower/mod.rs](../crates/sifr_hir/src/lower/mod.rs:5), [sifr_hir/src/lower/expressions.rs](../crates/sifr_hir/src/lower/expressions.rs), and [sifr_hir/src/lower/aug_assign_lowering.rs](../crates/sifr_hir/src/lower/aug_assign_lowering.rs) only. No leakage into `sifr_codegen`, `sifr_driver`, `sifr_diagnostics`, or any binary crate, which is correct: the symbol stays internal between operator helpers and HIR.

## Test coverage assessment

- `cargo test -p sifr_type_system` exercises [check.rs:587-749](../crates/sifr_type_system/src/check.rs:587), which now asserts on the unwrapped `code: DiagnosticCode` (previously `Some(DiagnosticCode::…)`). Three assertion sites updated; all preserve their previous code expectations. Good.
- `cargo test -p sifr_hir diagnostic_transport_tests` covers the *general* transport contract (`error_with_code` ↔ `Some(code)`, `error` ↔ `None`); it does not directly assert on `type_check_diagnostic` — see N3.
- `cargo test -p sifr --test e2e test_e2e_fail` covers end-to-end: the operator/type-mismatch fail fixtures (`bigint_int_mixed_arithmetic.sifr`, `decimal_bigdecimal_mixed_arithmetic.sifr`, `decimal_float_mixed_arithmetic.sifr`, `optional_arithmetic_without_narrowing.sifr`, etc.) implicitly verify that the same `SIFR-TYPE-*` / `SIFR-DECIMAL-*` codes still come out the back of the renderer. That is the right integration-level signal for "no behavioral regression."

## Reviewable-slice quality

This is a clean reviewable slice:

- One coherent change (delete two transitional symbols, introduce one slimmer replacement, rename one HIR helper, mechanical call-site updates).
- ~85 lines added / ~216 lines removed — net negative, dominated by removing the no-longer-needed `TypeErrorKind` enum and per-branch `kind: …` payloads.
- No mixed-in unrelated changes inside the `crates/` portion of the diff.
- Preserves diagnostic codes and message strings byte-for-byte at every call site, which is exactly the slice's stated invariant.

The untracked working-tree files reported by `git status` (the `issues/ad-hoc-signature-invalid-fixture-adaptation-*.md`, `issues/ownership-mutability-boundary-root-cause-2026-04-02.md`, `package.json`, `package-lock.json`, `reviews/ownership-mutability-boundary-*`, `verification/leetcode/`) are unrelated to this slice and should not be staged into the slice's PR.

## Recommended follow-ups (not blocking this slice)

1. Update [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md) entries that reference `TypeError`/`TypeErrorKind` (lines 9, 64, 68-80, 119, 375) so the inventory matches code reality — ideally inside this same slice's PR (see N1).
2. Plan a follow-up slice that fully discharges the `milestone_diag_7` DoD bullet "the type-system adapter path is gone; type-checking code emits or returns canonical diagnostics directly" by either having the operator helpers return `(DiagnosticCode, String)` (or `(DiagnosticCode, &'static str / Cow<str>)`) directly, or inverting control so HIR builds the diagnostic at the call site with the AST span and the type-system helper only returns the typed result. Either path lets `TypeCheckDiagnostic` be retired (see N2).
3. Optional: add a focused `type_check_diagnostic_records_structured_identity` test next to the existing two in [diagnostic_transport_tests.rs](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs) (see N3).

## Summary

Slice 2's *symbol-deletion* contract is satisfied cleanly, with no semantic-text or code drift, no hidden adapter, and no behavioral regressions across the operator surface. The thinner `TypeCheckDiagnostic` is a strict improvement (mandatory active code, no dead `kind` payload). The slice is appropriately narrow, the validation lane the author ran is sufficient for the change surface, and the only artifacts left to clean up are an internal inventory doc and a future slice that finishes retiring the adapter type itself — neither of which blocks merging this slice.
