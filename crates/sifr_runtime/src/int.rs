use num_bigint::BigInt;
use num_bigint::Sign;
use num_traits::{Signed, ToPrimitive, Zero};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Mul, Neg, Sub};
use std::str::FromStr;

pub const DEFAULT_MAX_INTEGER_DIGITS: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerParseError {
    Empty,
    InvalidDigit,
    DigitLimitExceeded { limit: usize, actual: usize },
}

impl fmt::Display for IntegerParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("integer text is empty"),
            Self::InvalidDigit => f.write_str("integer text contains an invalid digit"),
            Self::DigitLimitExceeded { limit, actual } => {
                write!(
                    f,
                    "integer text has {actual} digits, exceeding the configured limit of {limit}"
                )
            }
        }
    }
}

impl std::error::Error for IntegerParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerRangeError {
    target: &'static str,
    value: String,
}

impl IntegerRangeError {
    #[must_use]
    pub const fn new(target: &'static str, value: String) -> Self {
        Self { target, value }
    }

    #[must_use]
    pub const fn target(&self) -> &'static str {
        self.target
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for IntegerRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "integer value {} does not fit {}",
            self.value, self.target
        )
    }
}

impl std::error::Error for IntegerRangeError {}

#[derive(Clone, Debug, Eq)]
pub enum SifrInt {
    Small(i64),
    Big(Box<BigInt>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NormalizedIntegerHash {
    negative: bool,
    magnitude_be: Vec<u8>,
}

impl NormalizedIntegerHash {
    #[must_use]
    pub fn from_signed(value: i128) -> Self {
        Self {
            negative: value.is_negative(),
            magnitude_be: normalized_magnitude_bytes(&value.unsigned_abs().to_be_bytes()),
        }
    }

    #[must_use]
    pub fn from_unsigned(value: u128) -> Self {
        Self {
            negative: false,
            magnitude_be: normalized_magnitude_bytes(&value.to_be_bytes()),
        }
    }
}

macro_rules! try_to_fixed_width {
    ($name:ident, $target_ty:ty, $target_name:literal, $to_primitive:ident) => {
        pub fn $name(&self) -> Result<$target_ty, IntegerRangeError> {
            self.as_bigint()
                .$to_primitive()
                .ok_or_else(|| self.range_error($target_name))
        }
    };
}

impl SifrInt {
    #[must_use]
    pub const fn from_i64(value: i64) -> Self {
        Self::Small(value)
    }

    #[must_use]
    pub fn from_i128(value: i128) -> Self {
        i64::try_from(value).map_or_else(|_| Self::Big(Box::new(BigInt::from(value))), Self::Small)
    }

    #[must_use]
    pub fn from_u128(value: u128) -> Self {
        i64::try_from(value).map_or_else(|_| Self::Big(Box::new(BigInt::from(value))), Self::Small)
    }

    #[must_use]
    pub fn from_bigint(value: BigInt) -> Self {
        match value.to_i64() {
            Some(small) => Self::Small(small),
            None => Self::Big(Box::new(value)),
        }
    }

    pub fn parse_decimal(text: &str, max_digits: usize) -> Result<Self, IntegerParseError> {
        let digit_count = count_decimal_digits(text)?;
        if digit_count > max_digits {
            return Err(IntegerParseError::DigitLimitExceeded {
                limit: max_digits,
                actual: digit_count,
            });
        }
        let parsed = BigInt::from_str(text).map_err(|_| IntegerParseError::InvalidDigit)?;
        Ok(Self::from_bigint(parsed))
    }

    #[must_use]
    pub fn normalized_hash_key(&self) -> NormalizedIntegerHash {
        match self {
            Self::Small(value) => NormalizedIntegerHash::from_signed(i128::from(*value)),
            Self::Big(value) => {
                let (sign, magnitude_be) = value.to_bytes_be();
                NormalizedIntegerHash {
                    negative: sign == Sign::Minus,
                    magnitude_be: normalized_magnitude_bytes(&magnitude_be),
                }
            }
        }
    }

    #[must_use]
    pub fn as_bigint(&self) -> BigInt {
        match self {
            Self::Small(value) => BigInt::from(*value),
            Self::Big(value) => value.as_ref().clone(),
        }
    }

    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Small(value) => Some(*value),
            Self::Big(_) => None,
        }
    }

    #[must_use]
    pub fn checked_floor_div(&self, rhs: &Self) -> Option<Self> {
        let rhs = rhs.as_bigint();
        if rhs.is_zero() {
            return None;
        }
        Some(Self::from_bigint(floor_div_bigint(&self.as_bigint(), &rhs)))
    }

    #[must_use]
    pub fn checked_floor_mod(&self, rhs: &Self) -> Option<Self> {
        let rhs = rhs.as_bigint();
        if rhs.is_zero() {
            return None;
        }
        Some(Self::from_bigint(floor_mod_bigint(&self.as_bigint(), &rhs)))
    }

    #[must_use]
    pub fn floor_div_known_nonzero(&self, rhs: &Self) -> Self {
        debug_assert!(!rhs.as_bigint().is_zero());
        Self::from_bigint(floor_div_bigint(&self.as_bigint(), &rhs.as_bigint()))
    }

    #[must_use]
    pub fn floor_mod_known_nonzero(&self, rhs: &Self) -> Self {
        debug_assert!(!rhs.as_bigint().is_zero());
        Self::from_bigint(floor_mod_bigint(&self.as_bigint(), &rhs.as_bigint()))
    }

    #[must_use]
    pub fn pow(&self, exponent: u32) -> Self {
        Self::from_bigint(self.as_bigint().pow(exponent))
    }

    try_to_fixed_width!(try_to_i8, i8, "i8", to_i8);
    try_to_fixed_width!(try_to_i16, i16, "i16", to_i16);
    try_to_fixed_width!(try_to_i32, i32, "i32", to_i32);
    try_to_fixed_width!(try_to_i64, i64, "i64", to_i64);
    try_to_fixed_width!(try_to_i128, i128, "i128", to_i128);
    try_to_fixed_width!(try_to_isize, isize, "isize", to_isize);
    try_to_fixed_width!(try_to_u8, u8, "u8", to_u8);
    try_to_fixed_width!(try_to_u16, u16, "u16", to_u16);
    try_to_fixed_width!(try_to_u32, u32, "u32", to_u32);
    try_to_fixed_width!(try_to_u64, u64, "u64", to_u64);
    try_to_fixed_width!(try_to_u128, u128, "u128", to_u128);
    try_to_fixed_width!(try_to_usize, usize, "usize", to_usize);

    fn range_error(&self, target: &'static str) -> IntegerRangeError {
        IntegerRangeError::new(target, self.to_string())
    }
}

fn floor_div_bigint(left: &BigInt, right: &BigInt) -> BigInt {
    let quotient = left / right;
    let remainder = left % right;
    if needs_floor_adjustment(&remainder, right) {
        quotient - BigInt::ONE
    } else {
        quotient
    }
}

fn floor_mod_bigint(left: &BigInt, right: &BigInt) -> BigInt {
    let remainder = left % right;
    if needs_floor_adjustment(&remainder, right) {
        remainder + right
    } else {
        remainder
    }
}

fn needs_floor_adjustment(remainder: &BigInt, divisor: &BigInt) -> bool {
    !remainder.is_zero() && (remainder.is_negative() != divisor.is_negative())
}

impl From<i64> for SifrInt {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}

impl From<i128> for SifrInt {
    fn from(value: i128) -> Self {
        Self::from_i128(value)
    }
}

impl From<u128> for SifrInt {
    fn from(value: u128) -> Self {
        Self::from_u128(value)
    }
}

impl From<BigInt> for SifrInt {
    fn from(value: BigInt) -> Self {
        Self::from_bigint(value)
    }
}

impl Add for SifrInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Small(left), Self::Small(right)) => left.checked_add(right).map_or_else(
                || Self::from_i128(i128::from(left) + i128::from(right)),
                Self::Small,
            ),
            (left, right) => Self::from_bigint(left.as_bigint() + right.as_bigint()),
        }
    }
}

impl Add<&SifrInt> for SifrInt {
    type Output = Self;

    fn add(self, rhs: &SifrInt) -> Self::Output {
        self + rhs.clone()
    }
}

impl Add<SifrInt> for &SifrInt {
    type Output = SifrInt;

    fn add(self, rhs: SifrInt) -> Self::Output {
        self.clone() + rhs
    }
}

impl Add<&SifrInt> for &SifrInt {
    type Output = SifrInt;

    fn add(self, rhs: &SifrInt) -> Self::Output {
        self.clone() + rhs.clone()
    }
}

impl Sub for SifrInt {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Small(left), Self::Small(right)) => left.checked_sub(right).map_or_else(
                || Self::from_i128(i128::from(left) - i128::from(right)),
                Self::Small,
            ),
            (left, right) => Self::from_bigint(left.as_bigint() - right.as_bigint()),
        }
    }
}

impl Sub<&SifrInt> for SifrInt {
    type Output = Self;

    fn sub(self, rhs: &SifrInt) -> Self::Output {
        self - rhs.clone()
    }
}

impl Sub<SifrInt> for &SifrInt {
    type Output = SifrInt;

    fn sub(self, rhs: SifrInt) -> Self::Output {
        self.clone() - rhs
    }
}

impl Sub<&SifrInt> for &SifrInt {
    type Output = SifrInt;

    fn sub(self, rhs: &SifrInt) -> Self::Output {
        self.clone() - rhs.clone()
    }
}

impl Mul for SifrInt {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Small(left), Self::Small(right)) => left.checked_mul(right).map_or_else(
                || Self::from_i128(i128::from(left) * i128::from(right)),
                Self::Small,
            ),
            (left, right) => Self::from_bigint(left.as_bigint() * right.as_bigint()),
        }
    }
}

impl Mul<&SifrInt> for SifrInt {
    type Output = Self;

    fn mul(self, rhs: &SifrInt) -> Self::Output {
        self * rhs.clone()
    }
}

impl Mul<SifrInt> for &SifrInt {
    type Output = SifrInt;

    fn mul(self, rhs: SifrInt) -> Self::Output {
        self.clone() * rhs
    }
}

impl Mul<&SifrInt> for &SifrInt {
    type Output = SifrInt;

    fn mul(self, rhs: &SifrInt) -> Self::Output {
        self.clone() * rhs.clone()
    }
}

impl Neg for SifrInt {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Small(value) => value
                .checked_neg()
                .map_or_else(|| Self::from_i128(-i128::from(value)), Self::Small),
            Self::Big(value) => Self::from_bigint(-*value),
        }
    }
}

impl Neg for &SifrInt {
    type Output = SifrInt;

    fn neg(self) -> Self::Output {
        -self.clone()
    }
}

impl FromStr for SifrInt {
    type Err = IntegerParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse_decimal(text, DEFAULT_MAX_INTEGER_DIGITS)
    }
}

impl PartialEq for SifrInt {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Small(left), Self::Small(right)) => left == right,
            (Self::Big(left), Self::Big(right)) => left == right,
            (Self::Small(left), Self::Big(right)) | (Self::Big(right), Self::Small(left)) => {
                right.to_i64() == Some(*left)
            }
        }
    }
}

impl PartialOrd for SifrInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SifrInt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Small(left), Self::Small(right)) => left.cmp(right),
            (Self::Big(left), Self::Big(right)) => left.cmp(right),
            (Self::Small(left), Self::Big(right)) => match right.to_i64() {
                Some(right) => left.cmp(&right),
                None if right.is_negative() => Ordering::Greater,
                None => Ordering::Less,
            },
            (Self::Big(left), Self::Small(right)) => match left.to_i64() {
                Some(left) => left.cmp(right),
                None if left.is_negative() => Ordering::Less,
                None => Ordering::Greater,
            },
        }
    }
}

impl Hash for SifrInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Small(value) => {
                hash_normalized_integer_parts(
                    value.is_negative(),
                    &value.unsigned_abs().to_be_bytes(),
                    state,
                );
            }
            Self::Big(value) => {
                let (sign, magnitude_be) = value.to_bytes_be();
                hash_normalized_integer_parts(sign == Sign::Minus, &magnitude_be, state);
            }
        }
    }
}

impl fmt::Display for SifrInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Small(value) => fmt::Display::fmt(value, f),
            Self::Big(value) => fmt::Display::fmt(value, f),
        }
    }
}

fn count_decimal_digits(text: &str) -> Result<usize, IntegerParseError> {
    let unsigned = text
        .strip_prefix('-')
        .or_else(|| text.strip_prefix('+'))
        .unwrap_or(text);
    if unsigned.is_empty() {
        return Err(IntegerParseError::Empty);
    }
    if !unsigned.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(IntegerParseError::InvalidDigit);
    }
    Ok(unsigned.len())
}

fn normalized_magnitude_bytes(bytes: &[u8]) -> Vec<u8> {
    let normalized = strip_leading_zero_bytes(bytes);
    if normalized.is_empty() {
        Vec::new()
    } else {
        normalized.to_vec()
    }
}

fn hash_normalized_integer_parts<H: Hasher>(negative: bool, magnitude_be: &[u8], state: &mut H) {
    let magnitude = strip_leading_zero_bytes(magnitude_be);
    let normalized_negative = negative && !magnitude.is_empty();
    normalized_negative.hash(state);
    magnitude.len().hash(state);
    state.write(magnitude);
}

fn strip_leading_zero_bytes(bytes: &[u8]) -> &[u8] {
    let first_non_zero = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len());
    &bytes[first_non_zero..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

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
        let result =
            (SifrInt::from_i64(i64::MAX) + SifrInt::from_i64(1)) - SifrInt::from_i64(i64::MAX);

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
}
