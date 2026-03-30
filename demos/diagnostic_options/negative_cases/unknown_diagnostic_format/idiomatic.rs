//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! This fixture does not test the Sifr source itself. It exists to ensure the
//! CLI rejects `--diagnostic-format <unknown>` with usage exit code `2` before
//! semantic analysis of `main.sifr` begins.
//!
//! The Rust-side analogue is CLI-argument validation, not a compile-fail Rust
//! program, so this file remains a documentation scaffold.

fn main() {}
