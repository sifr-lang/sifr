//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because arithmetic is attempted on
//! `int | None` before narrowing and the function therefore fails its declared
//! `-> int` return rules on every control-flow path.
//!
//! This fixture exists to preserve deterministic optional-arithmetic diagnostics.
//! The Rust-side analogue is a type-system rejection around `Option<i64>`
//! narrowing, so this file documents the rules instead of fabricating a
//! misleading compile-fail sample.

fn main() {}
