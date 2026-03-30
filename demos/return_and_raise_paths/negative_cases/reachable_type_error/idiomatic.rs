//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because the explicit `else` branch returns
//! `&str` even though the function is declared to return `int`.
//!
//! A direct Rust analogue would also be rejected:
//! ```compile_fail
//! fn classify(flag: bool) -> i64 {
//!     if flag {
//!         return 1;
//!     } else {
//!         return "bad";
//!     }
//! }
//! ```

fn main() {}
