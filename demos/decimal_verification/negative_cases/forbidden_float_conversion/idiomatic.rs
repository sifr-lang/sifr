//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because `Decimal(1.5)` attempts to build
//! an exact decimal value from a floating-point literal.
//!
//! This fixture exists to preserve the decimal exact-construction rule and its
//! deterministic `SIFR-DECIMAL-0005` diagnostic directing users to `Decimal("...")`. The
//! Rust-side analogue depends on Sifr's exact-construction policy rather than a
//! direct Rust type error, so this file remains a rules scaffold.

fn main() {}
