//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because `relay_missing[U: MissingBound]`
//! forwards `x: U` into `take_missing[T: MissingBound]`, but `MissingBound` is not
//! a known protocol and therefore the forwarded type cannot satisfy `T`.
//!
//! This fixture exists to preserve the protocol-bound forwarding diagnostic rather
//! than a plain parser failure. The Rust-side analogue would be an unsatisfied
//! trait bound on a generic call, so this file records the Sifr rules directly.

fn main() {}
