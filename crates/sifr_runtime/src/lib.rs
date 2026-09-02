//! Runtime support shared by generated Sifr projects.
#![cfg_attr(test, allow(clippy::expect_used))]

pub mod async_cleanup;
mod byte_ops;
pub mod cancellation;
mod conversion;
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
mod nonempty_vec;
#[cfg(feature = "python")]
pub mod python;
mod range;
mod slice;
mod string_padding;
#[cfg(any(feature = "net", feature = "tls", feature = "http", test))]
mod timeouts;
#[cfg(feature = "tls")]
pub mod tls;
#[cfg(feature = "unicode")]
pub mod unicode;
#[cfg(feature = "unicode")]
mod unicode_data;

pub use byte_ops::count_byte;
pub use conversion::{ProvenUsize, to_usize_proven};
pub use int::{
    DEFAULT_MAX_INTEGER_DIGITS, DEFAULT_MAX_INTEGER_OUTPUT_BITS, IntegerArithmeticError,
    IntegerDivisionError, IntegerFloatConversionError, IntegerParseError, IntegerRangeError,
    NormalizedIntegerHash, SifrInt,
};
pub use nonempty_vec::SifrNonEmptyVec;
pub use range::SifrRange;
pub use slice::SifrSliceIndices;
pub use string_padding::{checked_center, checked_ljust, checked_rjust, checked_zfill};
