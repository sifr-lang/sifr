//! Runtime support shared by generated Sifr projects.
#![cfg_attr(test, allow(clippy::expect_used))]

pub mod encoding;
mod int;
pub mod json;

pub use int::{
    IntegerParseError, IntegerRangeError, NormalizedIntegerHash, SifrInt,
    DEFAULT_MAX_INTEGER_DIGITS,
};
