//! Runtime support shared by generated Sifr projects.
#![cfg_attr(test, allow(clippy::expect_used))]

mod int;

pub use int::{IntegerParseError, NormalizedIntegerHash, SifrInt, DEFAULT_MAX_INTEGER_DIGITS};
