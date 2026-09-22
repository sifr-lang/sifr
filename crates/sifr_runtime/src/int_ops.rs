use super::{DEFAULT_MAX_INTEGER_DIGITS, IntegerParseError, SifrInt};
use num_bigint::{BigInt, Sign};
use num_traits::{Signed, ToPrimitive};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::Sum;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, MulAssign,
    Neg, Not, Shl, Shr, Sub, SubAssign,
};
use std::str::FromStr;

impl From<&SifrInt> for SifrInt {
    fn from(value: &SifrInt) -> Self {
        value.clone()
    }
}

impl From<SifrInt> for String {
    fn from(value: SifrInt) -> Self {
        match value {
            SifrInt::Small(value) => value.to_string(),
            SifrInt::Big(value) => value.to_string(),
        }
    }
}

impl From<i64> for SifrInt {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}

macro_rules! impl_from_signed {
    ($($source:ty),* $(,)?) => {
        $(
            impl From<$source> for SifrInt {
                fn from(value: $source) -> Self {
                    Self::from_i128(i128::from(value))
                }
            }
        )*
    };
}

macro_rules! impl_from_unsigned {
    ($($source:ty),* $(,)?) => {
        $(
            impl From<$source> for SifrInt {
                fn from(value: $source) -> Self {
                    Self::from_u128(u128::from(value))
                }
            }
        )*
    };
}

impl_from_signed!(i8, i16, i32);
impl_from_unsigned!(u8, u16, u32, u64);

impl From<isize> for SifrInt {
    fn from(value: isize) -> Self {
        Self::from_i128(value as i128)
    }
}

impl From<usize> for SifrInt {
    fn from(value: usize) -> Self {
        Self::from_u128(value as u128)
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

impl AddAssign for SifrInt {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
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

impl SubAssign for SifrInt {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs;
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

impl MulAssign for SifrInt {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
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

impl Sum for SifrInt {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::from_i64(0), Add::add)
    }
}

impl<'a> Sum<&'a SifrInt> for SifrInt {
    fn sum<I: Iterator<Item = &'a SifrInt>>(iter: I) -> Self {
        iter.fold(Self::from_i64(0), Add::add)
    }
}

macro_rules! impl_exact_binary_op {
    ($trait:ident, $method:ident, $operator:tt) => {
        impl $trait for SifrInt {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                Self::from_bigint(self.as_bigint() $operator rhs.as_bigint())
            }
        }

        impl $trait<&SifrInt> for SifrInt {
            type Output = Self;

            fn $method(self, rhs: &SifrInt) -> Self::Output {
                self $operator rhs.clone()
            }
        }

        impl $trait<SifrInt> for &SifrInt {
            type Output = SifrInt;

            fn $method(self, rhs: SifrInt) -> Self::Output {
                self.clone() $operator rhs
            }
        }

        impl $trait<&SifrInt> for &SifrInt {
            type Output = SifrInt;

            fn $method(self, rhs: &SifrInt) -> Self::Output {
                self.clone() $operator rhs.clone()
            }
        }
    };
}

impl_exact_binary_op!(BitAnd, bitand, &);
impl_exact_binary_op!(BitOr, bitor, |);
impl_exact_binary_op!(BitXor, bitxor, ^);

macro_rules! impl_exact_assign_op {
    ($trait:ident, $method:ident, $operator:tt) => {
        impl $trait for SifrInt {
            fn $method(&mut self, rhs: Self) {
                *self = self.clone() $operator rhs;
            }
        }
    };
}

impl_exact_assign_op!(BitAndAssign, bitand_assign, &);
impl_exact_assign_op!(BitOrAssign, bitor_assign, |);
impl_exact_assign_op!(BitXorAssign, bitxor_assign, ^);

impl Not for SifrInt {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::from_bigint(!self.as_bigint())
    }
}

impl Not for &SifrInt {
    type Output = SifrInt;

    fn not(self) -> Self::Output {
        !self.clone()
    }
}

impl Shl<usize> for SifrInt {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self::from_bigint(self.as_bigint() << rhs)
    }
}

impl Shl<usize> for &SifrInt {
    type Output = SifrInt;

    fn shl(self, rhs: usize) -> Self::Output {
        self.clone() << rhs
    }
}

impl Shr<usize> for SifrInt {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self::from_bigint(self.as_bigint() >> rhs)
    }
}

impl Shr<usize> for &SifrInt {
    type Output = SifrInt;

    fn shr(self, rhs: usize) -> Self::Output {
        self.clone() >> rhs
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

impl PartialEq<f64> for SifrInt {
    fn eq(&self, other: &f64) -> bool {
        self.partial_cmp_f64(*other) == Some(Ordering::Equal)
    }
}

impl PartialOrd<f64> for SifrInt {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.partial_cmp_f64(*other)
    }
}

impl PartialEq<SifrInt> for f64 {
    fn eq(&self, other: &SifrInt) -> bool {
        other == self
    }
}

impl PartialOrd<SifrInt> for f64 {
    fn partial_cmp(&self, other: &SifrInt) -> Option<Ordering> {
        other.partial_cmp_f64(*self).map(Ordering::reverse)
    }
}

macro_rules! impl_primitive_comparison {
    ($($primitive:ty),* $(,)?) => {
        $(
            impl PartialEq<$primitive> for SifrInt {
                fn eq(&self, other: &$primitive) -> bool {
                    self == &SifrInt::from(*other)
                }
            }

            impl PartialOrd<$primitive> for SifrInt {
                fn partial_cmp(&self, other: &$primitive) -> Option<Ordering> {
                    Some(self.cmp(&SifrInt::from(*other)))
                }
            }

            impl PartialEq<SifrInt> for $primitive {
                fn eq(&self, other: &SifrInt) -> bool {
                    other == self
                }
            }

            impl PartialOrd<SifrInt> for $primitive {
                fn partial_cmp(&self, other: &SifrInt) -> Option<Ordering> {
                    other.partial_cmp(self).map(Ordering::reverse)
                }
            }
        )*
    };
}

impl_primitive_comparison!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

macro_rules! impl_primitive_arithmetic {
    ($($primitive:ty),* $(,)?) => {
        $(
            impl Add<$primitive> for SifrInt {
                type Output = Self;

                fn add(self, rhs: $primitive) -> Self::Output {
                    self + Self::from(rhs)
                }
            }

            impl Add<$primitive> for &SifrInt {
                type Output = SifrInt;

                fn add(self, rhs: $primitive) -> Self::Output {
                    self + SifrInt::from(rhs)
                }
            }

            impl Add<SifrInt> for $primitive {
                type Output = SifrInt;

                fn add(self, rhs: SifrInt) -> Self::Output {
                    SifrInt::from(self) + rhs
                }
            }

            impl Sub<$primitive> for SifrInt {
                type Output = Self;

                fn sub(self, rhs: $primitive) -> Self::Output {
                    self - Self::from(rhs)
                }
            }

            impl Sub<$primitive> for &SifrInt {
                type Output = SifrInt;

                fn sub(self, rhs: $primitive) -> Self::Output {
                    self - SifrInt::from(rhs)
                }
            }

            impl Sub<SifrInt> for $primitive {
                type Output = SifrInt;

                fn sub(self, rhs: SifrInt) -> Self::Output {
                    SifrInt::from(self) - rhs
                }
            }

            impl Mul<$primitive> for SifrInt {
                type Output = Self;

                fn mul(self, rhs: $primitive) -> Self::Output {
                    self * Self::from(rhs)
                }
            }

            impl Mul<$primitive> for &SifrInt {
                type Output = SifrInt;

                fn mul(self, rhs: $primitive) -> Self::Output {
                    self * SifrInt::from(rhs)
                }
            }

            impl Mul<SifrInt> for $primitive {
                type Output = SifrInt;

                fn mul(self, rhs: SifrInt) -> Self::Output {
                    SifrInt::from(self) * rhs
                }
            }
        )*
    };
}

impl_primitive_arithmetic!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

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

impl fmt::Debug for SifrInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

pub(super) fn count_decimal_digits(text: &str) -> Result<usize, IntegerParseError> {
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

pub(super) fn normalized_magnitude_bytes(bytes: &[u8]) -> Vec<u8> {
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
