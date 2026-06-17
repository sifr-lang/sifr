//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because safe indexing returns `T | None`,
//! but the generic helper `first(items: list[T])` promises plain `T`.
//!
//! This fixture exists to preserve the generic return-type mismatch rules
//! `expected 'T', got 'T | None'`. The Rust-side analogue is the need to unwrap
//! or propagate `Option<T>` explicitly, so this file documents the Sifr
//! type-system rule instead of inventing a different Rust error.

fn main() {}
