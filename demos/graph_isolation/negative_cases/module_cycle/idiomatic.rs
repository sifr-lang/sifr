//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because the reachable import graph forms a
//! three-node cycle: `main -> a -> b -> c -> a`.
//!
//! This fixture exists to preserve deterministic module-cycle detection during
//! graph isolation. The Rust-side analogue is a build/import graph cycle rather
//! than a normal Rust type error, so this file remains a documentation scaffold.

fn main() {}
