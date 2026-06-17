//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because a `list[int]` is indexed with a
//! `str` key instead of an integer index.
//!
//! This fixture exists to preserve deterministic invalid-index diagnostics. The
//! Rust-side analogue is still an index-type mismatch, but the exact user-facing
//! rules belongs to Sifr's checker, so this file remains an explanatory
//! scaffold instead of a synthetic Rust compile-fail.

fn main() {}
