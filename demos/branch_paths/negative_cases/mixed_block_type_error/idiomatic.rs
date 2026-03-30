//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because the mixed `try`/`if` block keeps a
//! reachable `else` branch that returns `&str` from an `int`-typed function.
//!
//! A direct Rust analogue would also be rejected:
//! ```compile_fail
//! fn bad(flag: bool) -> i64 {
//!     let result: Result<i64, ()> = try {
//!         if flag {
//!             1
//!         } else {
//!             "bad"
//!         }
//!     };
//!     result.unwrap_or(0)
//! }
//! ```

fn main() {}
