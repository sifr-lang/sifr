//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because `bad()` declares `-> int` but
//! returns the string literal `"oops"`.
//!
//! This fixture exists to preserve the direct return-type mismatch diagnostic in
//! the smallest possible compiled-expression example. The Rust-side analogue is
//! straightforward type mismatch, but the user-facing rules belongs to Sifr's
//! checker, so this file records that rules rather than fabricating a separate
//! compile-fail sample.

fn main() {}
