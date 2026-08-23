//! Runtime support shared by generated Sifr projects.
#![cfg_attr(test, allow(clippy::expect_used))]

pub mod cancellation;
pub mod encoding;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "i18n")]
pub mod i18n;
mod int;
pub mod interop;
mod interop_callbacks;
pub mod json;
#[cfg(feature = "net")]
pub mod net;
#[cfg(feature = "python")]
pub mod python;
#[cfg(any(feature = "net", feature = "tls", feature = "http", test))]
mod timeouts;
#[cfg(feature = "tls")]
pub mod tls;
#[cfg(feature = "unicode")]
pub mod unicode;
#[cfg(feature = "unicode")]
mod unicode_data;

pub use int::{
    DEFAULT_MAX_INTEGER_DIGITS, IntegerParseError, IntegerRangeError, NormalizedIntegerHash,
    SifrInt,
};
