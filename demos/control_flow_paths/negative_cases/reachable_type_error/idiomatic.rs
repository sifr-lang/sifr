//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because a reachable branch returns `&str`
//! from a function declared as `int -> int`.
//!
//! A direct Rust analogue would also be rejected:
//! ```compile_fail
//! fn safe(seed: i64) -> i64 {
//!     if seed > 0 {
//!         return seed;
//!     }
//!     "bad"
//! }
//! ```

fn main() {}
