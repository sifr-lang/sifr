// src/main.rs
pub mod helper;
use crate::helper::value;
fn main() {
    println!("import_forms import-form semantics demo:");
    println!("{}", value());
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub const fn value() -> SifrInt {
    SifrInt::from_i64(17)
}
