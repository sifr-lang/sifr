//! Tier-2 Rust scaffold for the Sifr negative fixture in `main.sifr`.
//!
//! The paired Sifr program is rejected because the reachable `else` branch returns
//! a list, which is not a member of the declared `int | str` return union.
//!
//! A direct Rust analogue would also be rejected:
//! ```compile_fail
//! enum Value {
//!     Int(i64),
//!     Text(String),
//! }
//!
//! fn bad(flag: bool) -> Value {
//!     if flag {
//!         Value::Int(1)
//!     } else {
//!         vec![1]
//!     }
//! }
//! ```

fn main() {}
