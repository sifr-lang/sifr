//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because `Decimal(1.5)` constructs a
//! decimal from a binary float, which loses the exact-string construction rule.
//!
//! The Rust-side analogue should use exact textual input instead:
//! ```rust
//! # use rust_decimal::Decimal;
//! let value = Decimal::from_str_exact("1.5").unwrap();
//! # let _ = value;
//! ```
//!
//! This fixture exists to preserve the ban on float-based decimal construction.

fn main() {}
