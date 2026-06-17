//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because the generator body remains fully
//! reachable through `try`/`except`: the function is declared as `-> list[int]`
//! even though it yields values, and the `except` arm also yields an undefined
//! name `missing_value`.
//!
//! This fixture exists to preserve traversal completeness for reachable
//! generator branches inside exception handling. The Rust-side analogue would
//! require both iterator-shape lowering and name-resolution failure, so this
//! file records the rules instead of manufacturing a misleading Rust error.

fn main() {}
