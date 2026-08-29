use num_bigint::BigInt;
use num_bigint::Sign;
use num_traits::{FromPrimitive, Signed, ToPrimitive, Zero};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

pub const DEFAULT_MAX_INTEGER_DIGITS: usize = 4096;
pub const DEFAULT_MAX_INTEGER_OUTPUT_BITS: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerArithmeticError {
    NegativeOperand,
    LimitExceeded { limit: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerDivisionError {
    DivisionByZero,
    FloatOverflow,
    FloatPrecisionLoss,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerFloatConversionError {
    Overflow,
    PrecisionLoss,
}

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

#[derive(Clone, Eq)]
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

    /// Builds an exact integer from compiler-validated decimal literal text.
    ///
    /// This entry point is reserved for generated code. Source-controlled text must use
    /// [`Self::parse_decimal`] and handle its error.
    #[must_use]
    pub fn from_decimal_literal(text: &str) -> Self {
        match Self::parse_decimal(text, DEFAULT_MAX_INTEGER_DIGITS) {
            Ok(value) => value,
            Err(error) => panic!("invalid compiler-validated integer literal: {error}"),
        }
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
    pub fn into_bigint(self) -> BigInt {
        match self {
            Self::Small(value) => BigInt::from(value),
            Self::Big(value) => *value,
        }
    }

    /// Returns the canonical two's-complement big-endian representation.
    ///
    /// This byte bridge lets generated adapters cross between independently
    /// versioned `num-bigint` dependencies without narrowing or text parsing.
    #[must_use]
    pub fn to_signed_bytes_be(&self) -> Vec<u8> {
        match self {
            Self::Small(value) => BigInt::from(*value).to_signed_bytes_be(),
            Self::Big(value) => value.to_signed_bytes_be(),
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

    pub fn checked_true_div(&self, rhs: &Self) -> Result<f64, IntegerDivisionError> {
        let denominator = rhs.as_bigint();
        if denominator.is_zero() {
            return Err(IntegerDivisionError::DivisionByZero);
        }
        let numerator = self.as_bigint();
        exact_bigint_ratio_to_f64(&numerator, &denominator)
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

    pub fn checked_pow(&self, exponent: &Self) -> Result<Self, IntegerArithmeticError> {
        if exponent.is_negative() {
            return Err(IntegerArithmeticError::NegativeOperand);
        }
        if exponent.is_zero() {
            return Ok(Self::from_i64(1));
        }
        if self.is_zero() {
            return Ok(Self::from_i64(0));
        }
        let base = self.as_bigint();
        if base == BigInt::ONE {
            return Ok(Self::from_i64(1));
        }
        if base == -BigInt::ONE {
            return Ok(
                if exponent.as_bigint() % BigInt::from(2_u8) == BigInt::ONE {
                    Self::from_i64(-1)
                } else {
                    Self::from_i64(1)
                },
            );
        }
        let mut remaining =
            exponent
                .try_to_u64()
                .map_err(|_| IntegerArithmeticError::LimitExceeded {
                    limit: DEFAULT_MAX_INTEGER_OUTPUT_BITS,
                })?;
        let mut result = BigInt::ONE;
        let mut factor = base;
        while remaining != 0 {
            if remaining & 1 == 1 {
                result *= &factor;
                ensure_integer_output_budget(&result)?;
            }
            remaining >>= 1;
            if remaining != 0 {
                factor = &factor * &factor;
                ensure_integer_output_budget(&factor)?;
            }
        }
        Ok(Self::from_bigint(result))
    }

    /// Executes exponentiation after compile-time validity and output-budget proof.
    #[must_use]
    pub fn pow_known_valid(&self, exponent: &Self) -> Self {
        match self.checked_pow(exponent) {
            Ok(value) => value,
            Err(error) => {
                panic!("compiler exact-integer exponentiation proof was invalid: {error:?}")
            }
        }
    }

    pub fn checked_shl(&self, shift: &Self) -> Result<Self, IntegerArithmeticError> {
        if shift.is_negative() {
            return Err(IntegerArithmeticError::NegativeOperand);
        }
        if self.is_zero() || shift.is_zero() {
            return Ok(self.clone());
        }
        let shift = shift
            .try_to_u64()
            .map_err(|_| IntegerArithmeticError::LimitExceeded {
                limit: DEFAULT_MAX_INTEGER_OUTPUT_BITS,
            })?;
        if self.as_bigint().bits().saturating_add(shift) > DEFAULT_MAX_INTEGER_OUTPUT_BITS {
            return Err(IntegerArithmeticError::LimitExceeded {
                limit: DEFAULT_MAX_INTEGER_OUTPUT_BITS,
            });
        }
        let shift = usize::try_from(shift).map_err(|_| IntegerArithmeticError::LimitExceeded {
            limit: DEFAULT_MAX_INTEGER_OUTPUT_BITS,
        })?;
        Ok(Self::from_bigint(self.as_bigint() << shift))
    }

    pub fn checked_shr(&self, shift: &Self) -> Result<Self, IntegerArithmeticError> {
        if shift.is_negative() {
            return Err(IntegerArithmeticError::NegativeOperand);
        }
        let bit_len = self.as_bigint().bits();
        if shift.as_bigint() >= BigInt::from(bit_len) {
            return Ok(if self.is_negative() {
                Self::from_i64(-1)
            } else {
                Self::from_i64(0)
            });
        }
        let shift = shift
            .try_to_usize()
            .map_err(|_| IntegerArithmeticError::LimitExceeded {
                limit: DEFAULT_MAX_INTEGER_OUTPUT_BITS,
            })?;
        Ok(Self::from_bigint(self.as_bigint() >> shift))
    }

    /// Executes a left shift after compile-time validity and output-budget proof.
    #[must_use]
    pub fn shl_known_valid(&self, shift: &Self) -> Self {
        match self.checked_shl(shift) {
            Ok(value) => value,
            Err(error) => {
                panic!("compiler exact-integer left-shift proof was invalid: {error:?}")
            }
        }
    }

    /// Executes a right shift after compile-time validity proof.
    #[must_use]
    pub fn shr_known_valid(&self, shift: &Self) -> Self {
        match self.checked_shr(shift) {
            Ok(value) => value,
            Err(error) => {
                panic!("compiler exact-integer right-shift proof was invalid: {error:?}")
            }
        }
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        match self {
            Self::Small(value) => *value == 0,
            Self::Big(value) => value.is_zero(),
        }
    }

    #[must_use]
    pub fn is_negative(&self) -> bool {
        match self {
            Self::Small(value) => value.is_negative(),
            Self::Big(value) => value.is_negative(),
        }
    }

    #[must_use]
    pub fn abs(&self) -> Self {
        if self.is_negative() {
            -self
        } else {
            self.clone()
        }
    }

    /// Narrows after generated code has established the inclusive `u8` bounds.
    #[must_use]
    pub fn to_u8_proven_in_range(&self) -> u8 {
        match self.try_to_u8() {
            Ok(value) => value,
            Err(error) => {
                panic!("compiler exact-integer u8 range proof was invalid: {error}")
            }
        }
    }

    /// Narrows after generated code has established non-negative, addressable bounds.
    #[must_use]
    pub fn to_usize_proven_in_bounds(&self) -> usize {
        match self.try_to_usize() {
            Ok(value) => value,
            Err(error) => {
                panic!("compiler exact-integer usize range proof was invalid: {error}")
            }
        }
    }

    /// Applies Python's unit-step slice-bound normalization against an addressable length.
    ///
    /// The result is always in `0..=len`, including for integers outside every fixed-width
    /// representation, so generated indexing code can use the returned `usize` directly.
    #[must_use]
    pub fn clamp_slice_bound(&self, len: usize) -> usize {
        let len_exact = BigInt::from(len);
        let normalized = if self.is_negative() {
            self.as_bigint() + &len_exact
        } else {
            self.as_bigint()
        };
        if normalized <= BigInt::ZERO {
            return 0;
        }
        if normalized >= len_exact {
            return len;
        }
        SifrInt::from_bigint(normalized).to_usize_proven_in_bounds()
    }

    /// Normalizes a Python-style sequence index, using `len` as an out-of-bounds sentinel.
    #[must_use]
    pub fn normalize_index_or_len(&self, len: usize) -> usize {
        let len_exact = BigInt::from(len);
        let normalized = if self.is_negative() {
            self.as_bigint() + &len_exact
        } else {
            self.as_bigint()
        };
        if normalized < BigInt::ZERO || normalized >= len_exact {
            return len;
        }
        Self::from_bigint(normalized).to_usize_proven_in_bounds()
    }

    #[must_use]
    pub fn to_f64_exact(&self) -> Option<f64> {
        let value = self.as_bigint();
        let converted = value.to_f64()?;
        (BigInt::from_f64(converted).as_ref() == Some(&value)).then_some(converted)
    }

    pub fn checked_to_f64(&self) -> Result<f64, IntegerFloatConversionError> {
        let value = self.as_bigint();
        let converted = value
            .to_f64()
            .filter(|candidate| candidate.is_finite())
            .ok_or(IntegerFloatConversionError::Overflow)?;
        if BigInt::from_f64(converted).as_ref() == Some(&value) {
            Ok(converted)
        } else {
            Err(IntegerFloatConversionError::PrecisionLoss)
        }
    }

    /// Converts a value whose exact float representability was proved by the compiler.
    #[must_use]
    pub fn to_f64_proven_exact(&self) -> f64 {
        match self.to_f64_exact() {
            Some(value) => value,
            None => panic!("compiler exact-integer float proof was invalid"),
        }
    }

    #[must_use]
    pub fn from_f64_trunc(value: f64) -> Option<Self> {
        value
            .is_finite()
            .then(|| value.trunc())
            .and_then(BigInt::from_f64)
            .map(Self::from_bigint)
    }

    /// Compares an exact integer with the exact mathematical value represented by an `f64`.
    ///
    /// No integer-to-float conversion occurs. `NaN` is unordered, matching Rust and Python
    /// floating-point comparison behavior.
    #[must_use]
    pub fn partial_cmp_f64(&self, other: f64) -> Option<Ordering> {
        if other.is_nan() {
            return None;
        }
        if other == f64::INFINITY {
            return Some(Ordering::Less);
        }
        if other == f64::NEG_INFINITY {
            return Some(Ordering::Greater);
        }

        let bits = other.to_bits();
        let negative = bits >> 63 != 0;
        let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
        let fraction = bits & ((1_u64 << 52) - 1);
        let (mantissa, exponent) = if raw_exponent == 0 {
            (fraction, -1022 - 52)
        } else {
            ((1_u64 << 52) | fraction, raw_exponent - 1023 - 52)
        };
        let mut signed_mantissa = BigInt::from(mantissa);
        if negative {
            signed_mantissa = -signed_mantissa;
        }

        let integer = self.as_bigint();
        Some(if exponent >= 0 {
            integer.cmp(&(signed_mantissa << exponent.unsigned_abs()))
        } else {
            (integer << exponent.unsigned_abs()).cmp(&signed_mantissa)
        })
    }

    #[must_use]
    pub fn eq_f64(&self, other: f64) -> bool {
        self.partial_cmp_f64(other) == Some(Ordering::Equal)
    }

    #[must_use]
    pub fn lt_f64(&self, other: f64) -> bool {
        self.partial_cmp_f64(other) == Some(Ordering::Less)
    }

    #[must_use]
    pub fn le_f64(&self, other: f64) -> bool {
        matches!(
            self.partial_cmp_f64(other),
            Some(Ordering::Less | Ordering::Equal)
        )
    }

    #[must_use]
    pub fn gt_f64(&self, other: f64) -> bool {
        self.partial_cmp_f64(other) == Some(Ordering::Greater)
    }

    #[must_use]
    pub fn ge_f64(&self, other: f64) -> bool {
        matches!(
            self.partial_cmp_f64(other),
            Some(Ordering::Greater | Ordering::Equal)
        )
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

fn exact_bigint_ratio_to_f64(
    numerator: &BigInt,
    denominator: &BigInt,
) -> Result<f64, IntegerDivisionError> {
    if numerator.is_zero() {
        return Ok(0.0);
    }
    let negative = numerator.sign() != denominator.sign();
    let mut numerator = numerator.abs();
    let mut denominator = denominator.abs();
    let divisor = bigint_gcd(numerator.clone(), denominator.clone());
    numerator /= &divisor;
    denominator /= divisor;

    let one = BigInt::ONE;
    if (&denominator & (&denominator - &one)) != BigInt::ZERO {
        return Err(IntegerDivisionError::FloatPrecisionLoss);
    }
    let denominator_power = denominator.bits().saturating_sub(1);
    let mut numerator_twos = 0_u64;
    while (&numerator & &one) == BigInt::ZERO {
        numerator >>= 1_usize;
        numerator_twos += 1;
    }
    let significand_bits = numerator.bits();
    if significand_bits > 53 {
        return Err(IntegerDivisionError::FloatPrecisionLoss);
    }
    let exponent = i128::from(numerator_twos) - i128::from(denominator_power);
    let highest_exponent = exponent + i128::from(significand_bits) - 1;
    if highest_exponent > 1023 {
        return Err(IntegerDivisionError::FloatOverflow);
    }
    if exponent < -1074 {
        return Err(IntegerDivisionError::FloatPrecisionLoss);
    }
    let significand = numerator
        .to_f64()
        .ok_or(IntegerDivisionError::FloatOverflow)?;
    let exponent = i32::try_from(exponent).map_err(|_| IntegerDivisionError::FloatOverflow)?;
    let value = significand * 2.0_f64.powi(exponent);
    Ok(if negative { -value } else { value })
}

fn bigint_gcd(mut left: BigInt, mut right: BigInt) -> BigInt {
    while !right.is_zero() {
        let remainder = left % &right;
        left = right;
        right = remainder;
    }
    left
}

fn ensure_integer_output_budget(value: &BigInt) -> Result<(), IntegerArithmeticError> {
    if value.bits() > DEFAULT_MAX_INTEGER_OUTPUT_BITS {
        Err(IntegerArithmeticError::LimitExceeded {
            limit: DEFAULT_MAX_INTEGER_OUTPUT_BITS,
        })
    } else {
        Ok(())
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

#[path = "int_ops.rs"]
mod ops;
use ops::{count_decimal_digits, normalized_magnitude_bytes};
#[cfg(test)]
#[path = "int_tests.rs"]
mod tests;
