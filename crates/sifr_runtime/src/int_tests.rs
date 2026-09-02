use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash(value: &SifrInt) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn fitting_values_stay_small() {
    let value = SifrInt::from_i64(42);
    assert!(matches!(value, SifrInt::Small(42)));
    assert_eq!(value.to_string(), "42");
}

#[test]
fn large_decimal_text_spills_to_big() {
    let value = SifrInt::parse_decimal("9223372036854775808", DEFAULT_MAX_INTEGER_DIGITS)
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(matches!(value, SifrInt::Big(_)));
    assert_eq!(value.to_string(), "9223372036854775808");
}

#[test]
fn parse_enforces_digit_limit_without_panicking() {
    let err = SifrInt::parse_decimal("12345", 4)
        .expect_err("digit limit should reject oversized integer text");
    assert_eq!(
        err,
        IntegerParseError::DigitLimitExceeded {
            limit: 4,
            actual: 5
        }
    );
}

#[test]
fn equality_ordering_and_hashing_are_normalized() {
    let small = SifrInt::Small(1);
    let big = SifrInt::Big(Box::new(BigInt::ONE));

    assert_eq!(small, big);
    assert_eq!(small.cmp(&big), Ordering::Equal);
    assert_eq!(hash(&small), hash(&big));
}

#[test]
fn arithmetic_keeps_small_results_inline() {
    let result = SifrInt::from_i64(7) + SifrInt::from_i64(5);
    assert!(matches!(result, SifrInt::Small(12)));

    let product = &SifrInt::from_i64(6) * &SifrInt::from_i64(7);
    assert!(matches!(product, SifrInt::Small(42)));
}

#[test]
fn arithmetic_spills_on_i64_overflow() {
    let result = SifrInt::from_i64(i64::MAX) + SifrInt::from_i64(1);
    assert!(matches!(result, SifrInt::Big(_)));
    assert_eq!(result.to_string(), "9223372036854775808");
}

#[test]
fn exponentiation_promotes_and_normalizes() {
    let large = SifrInt::from_i64(2).pow(100);
    assert!(matches!(large, SifrInt::Big(_)));
    assert_eq!(large.to_string(), "1267650600228229401496703205376");

    let small = SifrInt::from_i64(2).pow(10);
    assert_eq!(small, SifrInt::Small(1024));
}

#[test]
fn subtraction_multiplication_and_negation_spill_on_i64_overflow() {
    let underflow = SifrInt::from_i64(i64::MIN) - SifrInt::from_i64(1);
    assert!(matches!(underflow, SifrInt::Big(_)));
    assert_eq!(underflow.to_string(), "-9223372036854775809");

    let product = SifrInt::from_i64(i64::MAX) * SifrInt::from_i64(2);
    assert!(matches!(product, SifrInt::Big(_)));
    assert_eq!(product.to_string(), "18446744073709551614");

    let negated = -SifrInt::from_i64(i64::MIN);
    assert!(matches!(negated, SifrInt::Big(_)));
    assert_eq!(negated.to_string(), "9223372036854775808");
}

#[test]
fn overflow_then_cancel_can_return_to_small() {
    let result = (SifrInt::from_i64(i64::MAX) + SifrInt::from_i64(1)) - SifrInt::from_i64(i64::MAX);

    assert!(matches!(result, SifrInt::Small(1)));
}

#[test]
fn checked_floor_division_matches_exact_integer_semantics() {
    let cases = [
        (7, 3, 2),
        (-7, 3, -3),
        (7, -3, -3),
        (-7, -3, 2),
        (6, 3, 2),
        (-6, 3, -2),
    ];

    for (left, right, expected) in cases {
        let result = SifrInt::from_i64(left)
            .checked_floor_div(&SifrInt::from_i64(right))
            .expect("non-zero divisor should divide");
        assert_eq!(result, SifrInt::from_i64(expected));
    }
}

#[test]
fn checked_floor_modulo_matches_exact_integer_semantics() {
    let cases = [
        (7, 3, 1),
        (-7, 3, 2),
        (7, -3, -2),
        (-7, -3, -1),
        (6, 3, 0),
        (-6, 3, 0),
    ];

    for (left, right, expected) in cases {
        let result = SifrInt::from_i64(left)
            .checked_floor_mod(&SifrInt::from_i64(right))
            .expect("non-zero divisor should produce a remainder");
        assert_eq!(result, SifrInt::from_i64(expected));
    }
}

#[test]
fn checked_floor_division_and_modulo_return_none_for_zero_divisor() {
    let left = SifrInt::from_i64(7);
    let zero = SifrInt::from_i64(0);

    assert_eq!(left.checked_floor_div(&zero), None);
    assert_eq!(left.checked_floor_mod(&zero), None);
}

#[test]
fn checked_floor_division_and_modulo_normalize_large_results() {
    let left = SifrInt::parse_decimal("100000000000000000000", DEFAULT_MAX_INTEGER_DIGITS)
        .unwrap_or_else(|err| panic!("{err}"));
    let divisor = SifrInt::from_i64(3);

    assert_eq!(
        left.checked_floor_div(&divisor)
            .expect("non-zero divisor")
            .to_string(),
        "33333333333333333333"
    );
    assert_eq!(
        left.checked_floor_mod(&divisor)
            .expect("non-zero divisor")
            .to_string(),
        "1"
    );
}

#[test]
fn known_nonzero_floor_division_and_modulo_match_checked_results() {
    let cases = [(7, 3), (-7, 3), (7, -3), (-7, -3), (6, 3)];

    for (left, right) in cases {
        let left = SifrInt::from_i64(left);
        let right = SifrInt::from_i64(right);
        assert_eq!(
            left.floor_div_known_nonzero(&right),
            left.checked_floor_div(&right).expect("non-zero divisor")
        );
        assert_eq!(
            left.floor_mod_known_nonzero(&right),
            left.checked_floor_mod(&right).expect("non-zero divisor")
        );
    }
}

#[test]
fn fixed_width_hash_helpers_normalize_signed_and_unsigned_values() {
    assert_eq!(
        NormalizedIntegerHash::from_signed(255),
        NormalizedIntegerHash::from_unsigned(255)
    );
    assert_ne!(
        NormalizedIntegerHash::from_signed(-1),
        NormalizedIntegerHash::from_unsigned(255)
    );
}

#[test]
fn exact_integer_converts_to_fitting_fixed_width_targets() {
    let value = SifrInt::from_i64(42);

    assert_eq!(value.try_to_i8().expect("fits i8"), 42);
    assert_eq!(value.try_to_i16().expect("fits i16"), 42);
    assert_eq!(value.try_to_i32().expect("fits i32"), 42);
    assert_eq!(value.try_to_i64().expect("fits i64"), 42);
    assert_eq!(value.try_to_i128().expect("fits i128"), 42);
    assert_eq!(value.try_to_isize().expect("fits isize"), 42);
    assert_eq!(value.try_to_u8().expect("fits u8"), 42);
    assert_eq!(value.try_to_u16().expect("fits u16"), 42);
    assert_eq!(value.try_to_u32().expect("fits u32"), 42);
    assert_eq!(value.try_to_u64().expect("fits u64"), 42);
    assert_eq!(value.try_to_u128().expect("fits u128"), 42);
    assert_eq!(value.try_to_usize().expect("fits usize"), 42);
}

#[test]
fn exact_zero_divided_by_negative_integer_preserves_negative_zero() {
    let quotient = SifrInt::from_i64(0)
        .checked_true_div(&SifrInt::from_i64(-2))
        .expect("nonzero exact divisor should succeed");

    assert_eq!(quotient.to_bits(), (-0.0_f64).to_bits());
    assert!(quotient.is_sign_negative());
}

#[test]
fn exact_integer_conversion_reports_typed_range_errors() {
    let too_large = SifrInt::parse_decimal(
        "340282366920938463463374607431768211456",
        DEFAULT_MAX_INTEGER_DIGITS,
    )
    .unwrap_or_else(|err| panic!("{err}"));
    let negative = SifrInt::from_i64(-1);

    let i64_err = too_large
        .try_to_i64()
        .expect_err("oversized exact int should not fit i64");
    assert_eq!(i64_err.target(), "i64");
    assert_eq!(i64_err.value(), "340282366920938463463374607431768211456");

    let u8_err = SifrInt::from_i64(256)
        .try_to_u8()
        .expect_err("256 should not fit u8");
    assert_eq!(u8_err.target(), "u8");
    assert_eq!(u8_err.value(), "256");

    let usize_err = negative
        .try_to_usize()
        .expect_err("negative exact int should not fit usize");
    assert_eq!(usize_err.target(), "usize");
    assert_eq!(usize_err.value(), "-1");
}

#[test]
fn signed_big_values_order_correctly() {
    let negative = SifrInt::parse_decimal("-9223372036854775809", DEFAULT_MAX_INTEGER_DIGITS)
        .unwrap_or_else(|err| panic!("{err}"));
    let small = SifrInt::from_i64(-10);

    assert!(negative < small);
}
