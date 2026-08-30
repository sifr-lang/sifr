// src/main.rs
mod helper;

use crate::helper::value;

fn main() {
    println!("temp_workspace_isolation invocation-scoped temp workspace isolation demo:");
    println!("{}", value());
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
pub fn value() -> SifrInt {
    SifrInt::from_i64(44)
}
