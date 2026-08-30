use num_bigint::BigInt;
use num_traits::Zero;
use sifr_runtime::{
    DEFAULT_MAX_INTEGER_DIGITS, IntegerArithmeticError, IntegerDivisionError,
    IntegerFloatConversionError, SifrInt, SifrRange,
};
use std::str::FromStr;

fn exact(text: &str) -> SifrInt {
    SifrInt::parse_decimal(text, DEFAULT_MAX_INTEGER_DIGITS)
        .unwrap_or_else(|error| panic!("invalid test integer {text}: {error}"))
}

fn oracle_floor_div_mod(left: &BigInt, right: &BigInt) -> (BigInt, BigInt) {
    let mut quotient = left / right;
    let mut remainder = left % right;
    if !remainder.is_zero() && (remainder.sign() != right.sign()) {
        quotient -= 1;
        remainder += right;
    }
    (quotient, remainder)
}

#[test]
fn deterministic_exact_integer_properties_match_bigint_oracle() {
    let values = [
        "-340282366920938463463374607431768211457",
        "-18446744073709551617",
        "-9223372036854775809",
        "-9223372036854775808",
        "-2",
        "-1",
        "0",
        "1",
        "2",
        "9223372036854775807",
        "9223372036854775808",
        "18446744073709551616",
        "340282366920938463463374607431768211456",
    ];

    for left_text in values {
        let left = exact(left_text);
        let oracle_left = BigInt::from_str(left_text)
            .unwrap_or_else(|error| panic!("invalid oracle integer {left_text}: {error}"));
        assert_eq!((-&left).as_bigint(), -&oracle_left, "negation: {left_text}");

        for right_text in values {
            let right = exact(right_text);
            let oracle_right = BigInt::from_str(right_text)
                .unwrap_or_else(|error| panic!("invalid oracle integer {right_text}: {error}"));
            assert_eq!(
                (&left + &right).as_bigint(),
                &oracle_left + &oracle_right,
                "addition: {left_text}, {right_text}"
            );
            assert_eq!(
                (&left - &right).as_bigint(),
                &oracle_left - &oracle_right,
                "subtraction: {left_text}, {right_text}"
            );
            assert_eq!(
                (&left * &right).as_bigint(),
                &oracle_left * &oracle_right,
                "multiplication: {left_text}, {right_text}"
            );

            if oracle_right.is_zero() {
                assert_eq!(left.checked_floor_div(&right), None);
                assert_eq!(left.checked_floor_mod(&right), None);
                continue;
            }

            let (oracle_quotient, oracle_remainder) =
                oracle_floor_div_mod(&oracle_left, &oracle_right);
            assert_eq!(
                left.checked_floor_div(&right)
                    .unwrap_or_else(|| panic!("non-zero oracle divisor {right_text}"))
                    .as_bigint(),
                oracle_quotient,
                "floor division: {left_text}, {right_text}"
            );
            assert_eq!(
                left.checked_floor_mod(&right)
                    .unwrap_or_else(|| panic!("non-zero oracle divisor {right_text}"))
                    .as_bigint(),
                oracle_remainder,
                "floor modulo: {left_text}, {right_text}"
            );
        }
    }
}

#[test]
fn exact_arithmetic_crosses_every_primitive_boundary_without_wrapping() {
    assert_eq!(
        (SifrInt::from(i64::MAX) + 1_i64).to_string(),
        "9223372036854775808"
    );
    assert_eq!(
        (SifrInt::from(i64::MIN) - 1_i64).to_string(),
        "-9223372036854775809"
    );
    assert_eq!(
        (SifrInt::from(u64::MAX) * 2_u8).to_string(),
        "36893488147419103230"
    );
    assert_eq!((!SifrInt::from(0_i8)).to_string(), "-1");
}

#[test]
fn exact_floor_division_and_modulo_obey_python_sign_rules() {
    for (left, right, quotient, remainder) in [
        (-7, 3, -3, 2),
        (7, -3, -3, -2),
        (-7, -3, 2, -1),
        (7, 3, 2, 1),
    ] {
        let left = SifrInt::from(left);
        let right = SifrInt::from(right);
        assert_eq!(
            left.checked_floor_div(&right),
            Some(SifrInt::from(quotient))
        );
        assert_eq!(
            left.checked_floor_mod(&right),
            Some(SifrInt::from(remainder))
        );
    }
}

#[test]
fn integer_float_comparisons_compare_exact_values() {
    let above_f64_integer_precision = exact("9007199254740993");
    let rounded_float = 9_007_199_254_740_993_f64;

    assert_ne!(above_f64_integer_precision, rounded_float);
    assert!(above_f64_integer_precision > rounded_float);
    assert!(rounded_float < above_f64_integer_precision);
    assert_eq!(above_f64_integer_precision.partial_cmp_f64(f64::NAN), None);
}

#[test]
fn exact_float_conversion_distinguishes_precision_and_overflow() {
    assert_eq!(
        exact("1267650600228229401496703205376").checked_to_f64(),
        Ok(2_f64.powi(100))
    );
    assert_eq!(
        exact("9007199254740993").checked_to_f64(),
        Err(IntegerFloatConversionError::PrecisionLoss)
    );
    let enormous = SifrInt::from(2_i8)
        .checked_pow(&SifrInt::from(2000_i16))
        .unwrap_or_else(|error| panic!("2**2000 must fit the integer output budget: {error:?}"));
    assert_eq!(
        enormous.checked_to_f64(),
        Err(IntegerFloatConversionError::Overflow)
    );
}

#[test]
fn true_division_is_exact_or_reports_the_specific_failure() {
    assert_eq!(
        SifrInt::from(3_i8).checked_true_div(&SifrInt::from(2_i8)),
        Ok(1.5)
    );
    assert_eq!(
        SifrInt::from(1_i8).checked_true_div(&SifrInt::from(3_i8)),
        Err(IntegerDivisionError::FloatPrecisionLoss)
    );
    assert_eq!(
        SifrInt::from(1_i8).checked_true_div(&SifrInt::from(0_i8)),
        Err(IntegerDivisionError::DivisionByZero)
    );
}

#[test]
fn explosive_integer_operations_are_bounded_and_typed() {
    assert_eq!(
        SifrInt::from(2_i8).checked_pow(&SifrInt::from(-1_i8)),
        Err(IntegerArithmeticError::NegativeOperand)
    );
    assert!(matches!(
        SifrInt::from(2_i8).checked_shl(&SifrInt::from(1_000_000_u64)),
        Err(IntegerArithmeticError::LimitExceeded { .. })
    ));
    assert_eq!(
        SifrInt::from(-8_i8).checked_shr(&exact("100000000000000000000")),
        Ok(SifrInt::from(-1_i8))
    );
}

#[test]
fn exact_range_is_lazy_bidirectional_and_not_i64_bounded() {
    let start = exact("9223372036854775808");
    let end = exact("9223372036854775814");
    let mut range = SifrRange::new_known_nonzero(start.clone(), end.clone(), SifrInt::from(2_i8));

    assert_eq!(range.next(), Some(start));
    assert_eq!(range.next_back(), Some(exact("9223372036854775812")));
    assert_eq!(range.next(), Some(exact("9223372036854775810")));
    assert_eq!(range.next(), None);

    let descending =
        SifrRange::new_known_nonzero(end, exact("9223372036854775806"), SifrInt::from(-2_i8));
    assert_eq!(
        descending.collect::<Vec<_>>(),
        vec![
            exact("9223372036854775814"),
            exact("9223372036854775812"),
            exact("9223372036854775810"),
            exact("9223372036854775808"),
        ]
    );
}
