//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because arithmetic mixes `decimal` and
//! `bigdecimal` without an explicit conversion.
//!
//! This fixture exists to preserve the decimal mixed-precision arithmetic rule
//! and its deterministic `SIFR-DECIMAL-0004` diagnostic. The Rust-side analogue is still a
//! policy-level semantic error owned by Sifr's decimal checker, so this file
//! documents the rules rather than inventing a synthetic Rust compile-fail.

fn main() {}
