//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because the imported helper `area_like`
//! declares `float -> float` but returns `str`.
//!
//! A direct Rust analogue would also be rejected in the dependency module:
//! ```compile_fail
//! mod helper {
//!     pub fn area_like(r: f64) -> f64 {
//!         let _ = r;
//!         "bad"
//!     }
//! }
//!
//! fn main() {
//!     println!("{}", helper::area_like(3.0));
//! }
//! ```

fn main() {}
