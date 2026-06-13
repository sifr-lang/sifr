# Review: INT-3 Fixed-Width Promotion Coverage (PR #1861)

**Review scope:** Test-coverage follow-up after PR #1860. Expands coverage for fitting fixed-width scalar `+`, `-`, and `*` promotion across `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, and `isize`. Keeps `uint64` and `usize` explicitly blocked until the broader `SifrInt` promotion path exists.

## 1. Verdict: APPROVED

## 2. Blocking Findings

None.

## 3. Non-Blocking Follow-Ups

**Test structure — explicit `uint8`/`uint16`/`uint32` assertions in HIR lowering test**

The HIR lowering test `test_fixed_width_scalar_add_sub_mul_promote_to_int` (expressions_tests.rs:504) adds declarations for `tiny` (int8), `small` (int16), `left` (int32), `wide` (int64), `byte` (uint8), `mid` (uint16), `large` (uint32), and `pointer` (isize) in the source (lines 508-515). Only `tiny_total`, `total`, `diff`, `product`, and `pointer_total` have explicit `assert!(matches!(...))` assertions. The `uint8`, `uint16`, and `uint32` variables (`byte`, `mid`, `large`) contribute to `diff` and `product` but lack individual assertion blocks. This is not a blocker — the design intent is clearly expressed and the behavior is covered via `diff` (uint16-uint8 → int) and `product` (2*uint32 → int) — but consider adding an explicit assertion for at least one pure-unsigned expression (e.g., `unsigned_total: int = byte + mid`) to make the unsigned coverage intent unambiguous.

**Follow-up item (tracked in issue INT-3 checklist):** Deduplicate the `fixed_width_promotes_to_current_int` policy between type checking and codegen once the broader `SifrInt` promotion path lands.

## 4. Validation Notes

- **Type system unit tests:** `cargo test -p sifr_type_system -- test_fixed_width` — 2 tests pass (88 total, all pass).
- **HIR unit tests:** `cargo test -p sifr_hir -- test_fixed_width` — 14 tests pass (451 total, all pass).
- **E2E fixture:** `fixed_width_scalar_arithmetic_promotion.sifr` compiles and runs to completion (exit 0), all 7 assert statements pass.
- **HIR maintainability guardrails:** `python3 scripts/check_hir_maintainability_guardrails.py` — PASS.
- **`isize` coverage:** `isize` is in the promoted set at `check.rs:648` and is explicitly asserted in `pointer_total` at `expressions_tests.rs:541-544`. Design-consistent: `isize` is covered like any other fitting fixed-width type in scalar promotion.
- **`uint64`/`usize` exclusion:** Both remain blocked via `!matches!(fixed, FixedIntType::U64 | FixedIntType::USize)` at `check.rs:31`. The renamed test `test_uint64_and_usize_integer_add_wait_for_sifrint_promotion` at `check.rs:665` covers both with an error result, matching the documented blocking condition.

## Summary

PR #1861 correctly broadens the test coverage established by PR #1860. The type system loop now covers all 8 fitting fixed-width types (`I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `ISize`) with `+`, `-`, and `*` promotion. The HIR lowering test exercises int8, int16, int32, int64, uint8, uint16, uint32, and isize scalar promotion to `int` with concrete values and assertions. `uint64` and `usize` are correctly excluded and explicitly tested as blocked. No regressions, no blockers.
