//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because the imported helper `doubled`
//! declares `int -> int` but returns `str`.
//!
//! A direct Rust analogue would also be rejected in the dependency itself:
//! ```compile_fail
//! mod helper {
//!     pub fn doubled(x: i64) -> i64 {
//!         let _ = x;
//!         "bad"
//!     }
//! }
//!
//! fn main() {
//!     println!("{}", helper::doubled(21));
//! }
//! ```

fn main() {}
