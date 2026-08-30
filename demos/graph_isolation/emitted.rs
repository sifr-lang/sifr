// src/main.rs
mod helper;

use crate::helper::value;

fn main() {
    println!("graph_isolation graph and isolation regression matrix demo:");
    println!("{}", value());
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
pub fn value() -> SifrInt {
    SifrInt::from_i64(55)
}
