//! Rust-native standard library crate used by generated Sifr programs.
//!
//! The compiler-side metadata model lives in `sifr_stdlib_manifest`. This crate is
//! the generated-program dependency boundary. Public modules are gated by
//! narrow additive features and use `default-features = false` by default.

pub mod feature_contract;

#[cfg(feature = "base64")]
pub mod base64;
#[cfg(feature = "calendar")]
pub mod calendar;
#[cfg(feature = "encoding")]
pub mod encoding;
#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "gzip")]
pub mod gzip;
#[cfg(feature = "hash")]
pub mod hash;
#[cfg(feature = "html")]
pub mod html;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "i18n")]
pub mod i18n;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "logging")]
pub mod logging;
#[cfg(feature = "math")]
pub mod math;
#[cfg(feature = "net")]
pub mod net;
#[cfg(feature = "platform")]
pub mod platform;
#[cfg(feature = "process")]
pub mod process;
#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "random")]
pub mod random;
#[cfg(feature = "regex")]
pub mod regex;
#[cfg(feature = "runtime-observability")]
pub mod runtime_observability;
#[cfg(feature = "signals")]
pub mod signals;
#[cfg(feature = "sys")]
pub mod sys;
#[cfg(feature = "time")]
pub mod time;
#[cfg(feature = "tls")]
pub mod tls;
#[cfg(feature = "toml")]
pub mod toml;
#[cfg(feature = "unicode")]
pub mod unicode;
#[cfg(feature = "url")]
pub mod url;
#[cfg(feature = "uuid")]
pub mod uuid;
#[cfg(feature = "zipfile")]
pub mod zipfile;

#[must_use]
pub const fn crate_identity() -> &'static str {
    "sifr_stdlib"
}
