//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because `decimal.round()` is called with
//! scale `29`, outside the supported `0..=28` range.
//!
//! This fixture exists to preserve the exact decimal diagnostic rules,
//! including the explicit bound check and deterministic `SIFR-DECIMAL-0007` error. The
//! Rust-side analogue would depend on Sifr-specific decimal policy rather than a
//! standard library failure, so this file records the rules instead.

fn main() {}
