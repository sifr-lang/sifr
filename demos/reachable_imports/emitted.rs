// src/main.rs
pub mod helper;
use crate::helper::compute;
use ::sifr_runtime::SifrInt;
fn main() {
    println!("reachable_imports import-closure discovery demo:");
    println!("{}", compute(SifrInt::from_i64(6)));
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn compute(x: SifrInt) -> SifrInt {
    &x * &SifrInt::from_i64(7)
}
