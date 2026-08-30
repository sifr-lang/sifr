// src/main.rs
mod helper;

use crate::helper::value;

fn main() {
    println!("resolver_triggers explicit workspace import demo:");
    println!("{}", value());
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
pub fn value() -> SifrInt {
    SifrInt::from_i64(18)
}
