//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because the imported helper `adjusted`
//! declares `int -> int` but returns `str`.
//!
//! A direct Rust analogue would also be rejected in the dependency module:
//! ```compile_fail
//! mod helper {
//!     pub fn adjusted(value: i64) -> i64 {
//!         let _ = value;
//!         "bad"
//!     }
//! }
//!
//! fn main() {
//!     println!("{}", helper::adjusted(5));
//! }
//! ```

fn main() {}
