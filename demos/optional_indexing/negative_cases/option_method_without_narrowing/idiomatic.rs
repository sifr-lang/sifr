//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because a method is called on
//! `list[int] | None` before narrowing, so the optional value has no reachable
//! `.len()` member.
//!
//! This fixture exists to preserve deterministic optional-method diagnostics.
//! The Rust-side analogue is the requirement to unwrap or pattern-match
//! `Option<Vec<i64>>` before method access, so this file records the rules
//! rather than inventing a different Rust compile error.

fn main() {}
