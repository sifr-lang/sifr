//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because `list[int]` is not accepted where
//! `list[int | str]` is required. Lists remain invariant, so widening the element
//! type at the call boundary is not allowed.
//!
//! This fixture exists to preserve the list-variance diagnostic for mutable
//! container types. The Rust-side analogue is the same invariance pressure around
//! `Vec<T>`, but the user-facing wording is owned by Sifr's checker, so this file
//! stays as a rules scaffold.

fn main() {}
