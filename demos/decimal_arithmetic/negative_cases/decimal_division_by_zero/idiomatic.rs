//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program fails at runtime because it performs decimal division
//! by an exact zero divisor.
//!
//! A direct Rust analogue would also fail at runtime if left unchecked:
//! ```rust
//! # use rust_decimal::Decimal;
//! let x = Decimal::from_str_exact("1").unwrap();
//! let y = Decimal::from_str_exact("0").unwrap();
//! let _ = x / y;
//! ```
//!
//! The Sifr fixture exists to ensure this case reports a runtime error instead
//! of silently producing an invalid decimal result.

fn main() {}
