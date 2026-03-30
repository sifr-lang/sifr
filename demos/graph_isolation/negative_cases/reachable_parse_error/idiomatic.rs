//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program reaches a local dependency import, then fails before
//! graph analysis can proceed because `helper.sifr` contains a syntax error:
//! `def value(:`.
//!
//! A direct Rust analogue would also fail during parsing of the imported module:
//! ```compile_fail
//! mod helper {
//!     fn value( {}
//! }
//!
//! fn main() {
//!     println!("{}", helper::value());
//! }
//! ```

fn main() {}
