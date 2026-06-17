//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because a recursive function's reachable
//! branch calls `reccurse(...)` instead of the declared local function
//! `recurse(...)`.
//!
//! This fixture exists to preserve semantic analysis of recursive paths: the
//! typo is nested under a reachable `if n > 0` branch and must still resolve as
//! an undefined-function diagnostic. The Rust-side analogue is ordinary
//! name-resolution failure, so this file remains a rules scaffold.

fn main() {}
