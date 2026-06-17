//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because `T` is constrained to `(int, str)`
//! but `echo(1.5)` tries to instantiate it with `float`.
//!
//! This fixture exists to preserve the constrained-typevar diagnostic for an
//! unsatisfied argument type. The Rust-side analogue is a trait-bound or enum-like
//! constraint failure, but the exact user-facing rules is specific to Sifr's
//! type-parameter checker, so this file remains a rules scaffold.

fn main() {}
