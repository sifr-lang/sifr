//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because user code attempts to import
//! `_sifr.math`, which is reserved for internal compiler intrinsics. The
//! unresolved `sqrt` reference then surfaces as a secondary reachable diagnostic.
//!
//! This fixture exists to preserve the explicit frontend rule that `_sifr.*`
//! modules are not part of the user-visible standard library. The Rust-side
//! analogue is still compiler-owned import policy rather than a standalone Rust
//! compile error, so this file remains a rules scaffold.

fn main() {}
