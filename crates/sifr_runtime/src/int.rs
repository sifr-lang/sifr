use num_bigint::BigInt;
use num_bigint::Sign;
use num_traits::{Signed, ToPrimitive};
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
        let big = SifrInt::Big(Box::new(BigInt::from(1_i64)));

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
    fn signed_big_values_order_correctly() {
        let negative = SifrInt::parse_decimal("-9223372036854775809", DEFAULT_MAX_INTEGER_DIGITS)
            .unwrap_or_else(|err| panic!("{err}"));
        let small = SifrInt::from_i64(-10);

        assert!(negative < small);
    }
}
