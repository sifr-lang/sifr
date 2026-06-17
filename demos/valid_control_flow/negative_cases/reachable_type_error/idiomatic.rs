//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because the final reachable return path
//! yields `&str` even though the function rules requires `int`.
//!
//! A direct Rust analogue would also be rejected:
//! ```compile_fail
//! fn compute(flag: bool) -> i64 {
//!     if flag {
//!         return 1;
//!     }
//!     "bad"
//! }
//! ```

fn main() {}
