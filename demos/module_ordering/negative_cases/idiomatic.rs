//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because the reachable module ordering is
//! impossible: `main -> a -> b -> a` forms a two-node import cycle.
//!
//! This fixture exists to preserve cycle detection in the smaller ordering-only
//! graph. The Rust-side analogue is still a module-graph cycle, so this file is
//! an explanatory scaffold rather than a synthetic compile-fail program.

fn main() {}
