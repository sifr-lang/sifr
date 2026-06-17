//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program intentionally produces a run of repeated `int <- str`
//! assignment failures so the `--diagnostic-format compact` renderer can be
//! checked for deterministic grouping and ordering.
//!
//! A representative Rust-side analogue would be a file with repeated type
//! mismatches, but the important rules here is the renderer behavior rather
//! than the exact source syntax.

fn main() {}
