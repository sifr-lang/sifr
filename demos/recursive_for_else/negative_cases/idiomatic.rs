//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because the recursive call in the
//! reachable `for`-`else` branch is misspelled as `recc(...)`.
//!
//! This fixture exists to preserve traversal completeness for `for`-`else`
//! lowering: the analyzer must descend into the `else` block and surface the
//! undefined-function diagnostic instead of treating that branch as dead or
//! skipped. The Rust-side analogue is still name-resolution failure, so this
//! file remains an explanatory scaffold.

fn main() {}
