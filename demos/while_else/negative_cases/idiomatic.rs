//! Tier-2 Rust scaffold for the Sifr guard fixture in this folder.
//!
//! The paired Sifr program is expected to run successfully and print `ok` because
//! the `else` arm of a `while` loop must not execute after `break`.
//!
//! This fixture exists to preserve the control-flow guard against regressions in
//! `while`-`else` lowering. The Rust-side analogue is a runtime-behavior rules
//! rather than a compile-fail shape, so this file remains a minimal explanatory
//! scaffold.

fn main() {}
