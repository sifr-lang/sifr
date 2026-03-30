//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because the final branch stays reachable
//! and returns `&str` from a function declared to return `int`.
//!
//! A direct Rust analogue would also be rejected:
//! ```compile_fail
//! fn classify(flag: bool) -> i64 {
//!     if flag {
//!         return 5;
//!     }
//!     "bad"
//! }
//! ```

fn main() {}
