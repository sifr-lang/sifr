

Now I have a complete picture of all changed code. Let me write the review.

---

# PR #1855 Review: Lower proven exact-int floor modulo literals

**PR:** https://github.com/sifr-lang/sifr/pull/1855
**Branch:** `int-1-sifrint-proven-floor-mod-codegen`
**Author's validation:** `scripts/run_all_tests.sh --profile quick` (report_signature=e1bf653aaa770517, wall_time=86.98s), plus targeted unit/e2e runs.

---

## Changed files

| File | Change |
|------|--------|
| `crates/sifr_runtime/src/int.rs` | Add `floor_div_known_nonzero` / `floor_mod_known_nonzero` to `SifrInt`, each with `#[must_use]` and `debug_assert!(!rhs.is_zero())` |
| `crates/sifr_codegen/src/expr_render_helpers.rs` | Codegen rewrite: detect `SifrInt`-shaped `//`/`%` with syntactically proven-nonzero integer literal RHS, emit method call to known-nonzero helpers |
| `crates/sifr/tests/e2e/pass/exact_int_floor_mod_literals.sifr` | New e2e fixture covering oversized module constant, unary receiver, and derived local as floor-div/modulo LHS with literal divisor `3` |
| `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` | Tracker update marking the literal-divisor slice done, with residual scoped to `//=`/`%=` augmented assignment and HIR `Result[int, DivisionError]` integration |

---

## Blocking Findings

**None identified.**

The implementation is sound: runtime helpers delegate to the shared `floor_div_bigint`/`floor_mod_bigint` path already validated in PR #1853, the codegen rewrite fires only when both the LHS is `SifrInt`-shaped and the RHS is a syntactically proven-nonzero integer expression, and the `debug_assert!` in both new helpers guards the compiler-proof contract at development/test time without any user-triggerable panic path in release builds (per `#[must_use]` + no `panic!` or `unwrap` in the implementation body).

---

## Design Alignment

### Integer model contract

- `floor_div_known_nonzero` / `floor_mod_known_nonzero` return `Self` (not `Option<Self>`), consistent with the design intent that a compiler-proven non-zero divisor removes the fallibility.
- The `debug_assert!` in both helpers makes the proof obligation explicit and visible to developers, not users.
- `#[must_use]` on both helpers follows the same convention as `checked_floor_div`/`checked_floor_mod` from PR #1853.
- The codegen rewrite fires before the general `is_sifr_int_operand_coercion_op` path in the same function, so proven-nonzero literal divisors no longer fall through to `i64`-based Rust division/modulo.

### Ownership / value semantics

- The codegen emits the receiver through `coerce_expr_to_sifr_int_method_receiver` and the argument through `coerce_expr_to_sifr_int_comparison_operand` (which takes a `Ref`), matching the ownership pattern used by the existing comparison-coercion path.
- For `SifrInt`-registered locals, the receiver is passed by reference without clone; for other expressions (including casts from `i64`), the helper applies `sifr_int_from_i64_expr`. This is consistent with established coercion patterns in the codegen.

### Runtime panic risks

- No `panic!`, `unwrap`, or `expect` in user-triggerable paths in either new helper. The only guard is the `debug_assert!`, which is compile-time-only in release builds.
- The `floor_div_bigint` and `floor_mod_bigint` shared helpers were introduced in PR #1853 and are already covered by the existing `checked_floor_div`/`checked_floor_mod` runtime tests. The new known-nonzero variants simply elide the `Option`-wrapping.

### Tracker accuracy

- The updated checklist entry ("`SifrInt`-shaped `//` and `%` expressions with syntactically non-zero integer literal divisors now lower through compiler-proven non-zero floor division/modulo runtime helpers") accurately describes this PR's scope.
- The residual item ("unsupported exact-int augmented-assignment codegen for `//=` and `%=` plus HIR `Result[int, DivisionError]` / `SIFR-INT-0005` proven-nonzero integration") correctly scopes what remains.

---

## Non-Blocking Notes

### 1. Tracker residual wording could be tightened

The residual says "unsupported exact-int codegen for `//`, `%`, `//=`, and `%=`". This PR addresses `//` and `%` with **literal** divisors. The residual could more precisely say "unsupported exact-int augmented-assignment codegen for `//=` and `%=` plus HIR `Result[int, DivisionError]` / `SIFR-INT-0005` proven-nonzero integration" — or alternatively, "exact-int `//`/`%` codegen for non-literal-division cases". Minor wording drift; not a blocker.

### 2. `is_sifr_int_checked_floor_op` naming

The predicate function `is_sifr_int_checked_floor_op` returns true for `/` and `%`. In Python/Sifr semantics, `/` is floor division for `int` operands. The name uses "checked" but this path is for **known-nonzero** (not checked/Options-returning) dispatch. A more descriptive name would be `is_sifr_int_floor_div_mod_op` or simply `is_floor_division_or_modulo_op`. Not a blocker; naming is internal and can be improved in a follow-up.

### 3. Unary receiver coercion — no `Deref` in `coerce_expr_to_sifr_int_method_receiver`

The helper does not handle `crate::RustExpr::Deref`, but `is_sifr_int_expr` in the same file also returns `false` for `Deref` expressions. The implementation is consistent with the existing codebase. If a future PR adds deref-support for `SifrInt`, this helper will need corresponding updating.

### 4. E2E fixture does not cover negative literal divisors or zero literal divisors

The fixture exercises `BIG_LIMIT // 3` and `BIG_LIMIT % 3`. It does not include:
- `BIG_LIMIT // -3` (negative divisor) — `is_proven_nonzero_integer_expr` would return `true` for the negation of a proven-nonzero expression, so this path **would** be rewritten correctly; the omission is test coverage, not a gap.
- `BIG_LIMIT // 0` — this is correctly **not** rewritten (the rewrite guard fires only when `is_proven_nonzero_integer_expr` returns `true`); a zero literal divisor would not pass the `value != 0` check, so it would fall through to the general coercion path or produce a compile error downstream.

Coverage could be broadened to include negative literal divisors for completeness, but this is a hardening note, not a finding.

### 5. E2E fixture uses string-comparison assertions

The fixture uses `assert str(quotient) == "33333333333333333333"` rather than direct equality with a `SifrInt` literal. This is appropriate for e2e readability and avoids needing a `const` declaration for the expected `SifrInt` value; the approach is sound.

### 6. Unit test for known-nonzero helpers covers small values only

The runtime test `known_nonzero_floor_division_and_modulo_match_checked_results` only covers `i64`-sized inputs: `[(7, 3), (-7, 3), (7, -3), (-7, -3), (6, 3)]`. It does not directly test oversized `SifrInt::Big` receivers. However, both `floor_div_known_nonzero` and `checked_floor_div` delegate to the same `floor_div_bigint` helper for all values, so correctness for large receivers follows from the existing `checked_floor_division_and_modulo_normalize_large_results` test covering oversized inputs. This is acceptable.

### 7. `coerce_expr_to_sifr_int_comparison_operand` takes a `Ref` for the divisor argument

The divisor argument to the known-nonzero methods is passed through `coerce_expr_to_sifr_int_comparison_operand`, which wraps in `Ref { mutable: false, ... }`. This matches the comparison-coercion pattern used elsewhere in the codebase. The generated Rust signature accepts `&Self` for both methods, so passing a reference is correct.

---

## Summary

PR #1855 is clean. The runtime helpers are correct thin wrappers over the already-validated `floor_div_bigint`/`floor_mod_bigint` shared logic. The codegen rewrite correctly guards on a syntactically proven non-zero integer divisor and emits a direct method call, eliminating the prior path that would have fallen through to invalid `i64` division. Tracker updates are accurate. No blocking issues. Non-blocking notes are hardening suggestions, none of which represent gaps in the current implementation scope.
