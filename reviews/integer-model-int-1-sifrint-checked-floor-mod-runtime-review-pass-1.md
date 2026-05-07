# INT-1: SifrInt checked floor division/modulo runtime primitives
## Review pass 1 — PR #1853 (`int-1-sifrint-floor-mod-support`)

**Author:** Yaser Alnajjar
**Commit:** `6ac7d7ef` ("Add checked SifrInt floor division primitives")
**Files changed:** `crates/sifr_runtime/src/int.rs` (+109, -1)

---

## Summary

Adds `checked_floor_div` and `checked_floor_mod` to `SifrInt` returning `Option<Self>`,
with zero-divisor guard returning `None`. No codegen wiring. No HIR typed-failure
behavior. Scope is intentionally limited to the runtime substrate slice.

---

## Blocking findings

None.

---

## Non-blocking notes

### 1. `MIN // -1` / `MIN % -1` i64 overflow not exercised

`i64::MIN / -1` overflows i64 (produces `i64::MAX + 1 = 9223372036854775808`), and
`num_bigint::BigInt` handles this without issue. The test suite does not include a
case like `SifrInt::from_i64(i64::MIN).checked_floor_div(&SifrInt::from_i64(-1))`.
Since `SifrInt::from_bigint` upcasts to `Big` when the value does not fit `Small(i64)`,
this is safe in practice, but adding an explicit test case would harden the
coverage boundary.

**Recommendation (non-blocking):** Add a test case:
```rust
#[test]
fn checked_floor_div_min_int_overflow() {
    let result = SifrInt::from_i64(i64::MIN)
        .checked_floor_div(&SifrInt::from_i64(-1))
        .expect("should succeed with Big result");
    assert_eq!(result.to_string(), "9223372036854775808");
}
```

### 2. `#[must_use]` on private helpers

`floor_div_bigint`, `floor_mod_bigint`, and `needs_floor_adjustment` are private
non-`#[must_use]` functions. This is fine and consistent — callers are always
within the same module. No action needed.

### 3. `as_bigint()` clone cost on `Big` variant

Both `checked_floor_div` and `checked_floor_mod` call `rhs.as_bigint()` and
`self.as_bigint()`, each cloning the `BigInt` for the `Big` variant. This is the
established pattern in the crate (e.g., existing arithmetic ops), so it is not a
new concern. If performance becomes relevant later, a borrowing API could be
introduced, but that is out of scope for this slice.

### 4. `num_traits::Zero` import

The PR adds `Zero` to the `num_traits` import but only uses `is_zero()`.
`num_bigint::BigInt` also has an inherent `is_zero()` method, so the `Zero` trait
import is unnecessary for the current call sites. It may be intended for future
use. No action required.

---

## Design alignment check

| Concern | Design rule | Implementation | Status |
|---|---|---|---|
| Floor semantics for negative operands | Python/exact-integer floor div/mod identity | `needs_floor_adjustment` checks remainder/divisor sign mismatch | Correct |
| Zero divisor | Return typed failure, not panic | `None` on `is_zero()` | Correct |
| API shape | Fallible division returns Option or Result | `Option<Self>` | Correct |
| No codegen wiring | Scope is runtime only | No HIR/codegen changes | Correct |
| `#[must_use]` | Fallible APIs must warn on discard | On both public methods | Correct |
| `num_bigint` for arbitrary precision | `BigInt` handles overflow | `BigInt` division used | Correct |

Python floor division/modulo semantics verified directly against Python interpreter:
- `7 // 3 = 2`, `7 % 3 = 1` — matches test expectation
- `-7 // 3 = -3`, `-7 % 3 = 2` — matches test expectation
- `7 // -3 = -3`, `7 % -3 = -2` — matches test expectation
- `-7 // -3 = 2`, `-7 % -3 = -1` — matches test expectation
- `6 // 3 = 2`, `-6 // 3 = -2` — matches test expectation
- Floor identity `q * divisor + r == dividend` holds for all cases
- Remainder sign always matches divisor sign (or remainder is zero)

---

## Test coverage assessment

| Scenario | Covered | Notes |
|---|---|---|
| Positive div/mod positive | Yes | `7 // 3`, `7 % 3` |
| Negative div/mod positive | Yes | `-7 // 3`, `-7 % 3` |
| Positive div/mod negative | Yes | `7 // -3`, `7 % -3` |
| Negative div/mod negative | Yes | `-7 // -3`, `-7 % -3` |
| Divisible cases | Yes | `6 // 3`, `-6 // 3` |
| Zero divisor div | Yes | Returns `None` |
| Zero divisor mod | Yes | Returns `None` |
| Large BigInt normalization | Yes | 20-digit number |

Missing (see note 1 above): `MIN // -1` overflow case.

---

## Scope alignment

This PR is the **runtime substrate slice only** — `checked_floor_div` and
`checked_floor_mod` are added to `SifrInt` with no HIR changes, no codegen
emission, and no typed-failure wiring. The PR description explicitly states that
codegen and HIR typed-failure behavior is the next slice. The implementation
matches this boundary precisely.

From INT-1 milestone description:
> "Add canonical `SifrInt` runtime type with `Small(i64)` and `Big(Box<num_bigint::BigInt>)`"

This PR adds floor division/modulo runtime primitives to that canonical type.

---

## Validation status

Author reported:
- `cargo fmt --check` — passed
- `git diff --check` — passed
- `cargo test -p sifr_runtime checked_floor -- --nocapture` — passed
- `scripts/run_all_tests.sh --profile quick` — report `e1bf653aaa770517`, wall time 54.03s — passed

Quick validation was run by the author. No independent re-run was performed as part
of this review.

---

## Verdict

**APPROVED** — no blocking findings.

The implementation correctly implements Python/exact-integer floor division and
floor modulo semantics, safely handles zero divisors via `Option`, and is
correctly scoped to runtime only. One non-blocking note recommends adding an
explicit `MIN // -1` test case for defensive coverage, but the current behavior
is sound.
