//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because the nested `else` branch remains
//! reachable and returns `&str` from a function declared to return `int`.
//!
//! A direct Rust analogue would also be rejected:
//! ```compile_fail
//! fn broken(flag: bool) -> i64 {
//!     if flag {
//!         1
//!     } else {
//!         "bad"
//!     }
//! }
//! ```

fn main() {}
