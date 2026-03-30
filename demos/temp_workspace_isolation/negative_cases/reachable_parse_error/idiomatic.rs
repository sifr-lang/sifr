//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program must still fail under invocation-scoped isolation
//! because the reachable local dependency contains a syntax error:
//! `def value(:`.
//!
//! A direct Rust analogue would also fail during parsing of the imported helper:
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
