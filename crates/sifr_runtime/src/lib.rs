//! Runtime support shared by generated Sifr projects.
#![cfg_attr(test, allow(clippy::expect_used))]

pub mod encoding;
#[cfg(feature = "i18n")]
pub mod i18n;
mod int;
pub mod json;
#[cfg(feature = "net")]
pub mod net;
#[cfg(feature = "unicode")]
pub mod unicode;
#[cfg(feature = "unicode")]
mod unicode_data;

pub use int::{
    IntegerParseError, IntegerRangeError, NormalizedIntegerHash, SifrInt,
    DEFAULT_MAX_INTEGER_DIGITS,
};
