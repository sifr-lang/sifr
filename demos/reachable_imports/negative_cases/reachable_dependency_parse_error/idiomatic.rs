//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because the imported dependency is
//! reachable and contains a syntax error: `def compute(:`.
//!
//! A direct Rust analogue would also fail while parsing the dependency module:
//! ```compile_fail
//! mod helper {
//!     fn compute( {}
//! }
//!
//! fn main() {
//!     println!("{}", helper::compute(1));
//! }
//! ```

fn main() {}
