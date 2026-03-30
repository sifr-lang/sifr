//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because `BigDecimal(1.25)` attempts to
//! construct an exact decimal value from a binary float.
//!
//! The Rust-side analogue should use exact textual input instead:
//! ```rust
//! # use bigdecimal::BigDecimal;
//! # use std::str::FromStr;
//! let value = BigDecimal::from_str("1.25").unwrap();
//! # let _ = value;
//! ```
//!
//! This fixture exists to preserve the same exact-construction rule for
//! `bigdecimal` values.

fn main() {}
