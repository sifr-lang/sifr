//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program intentionally triggers a simple `int <- str` type
//! mismatch so the canonical `--diagnostic-format json` schema can be validated.
//!
//! A representative Rust-side analogue would also be a plain assignment type
//! error, but the rules here is the structured JSON payload rather than the
//! exact surface syntax.

fn main() {}
