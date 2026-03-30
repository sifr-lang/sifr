//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because a non-exiting `None` branch leaves
//! the function trying to return `Option<int>` where `int` is required.
//!
//! A direct Rust analogue would also be rejected:
//! ```compile_fail
//! fn pick_value(maybe: Option<i64>) -> i64 {
//!     if maybe.is_none() {
//!         println!("missing value");
//!     }
//!     maybe
//! }
//! ```

fn main() {}
