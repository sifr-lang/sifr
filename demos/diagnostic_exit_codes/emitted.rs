// src/main.rs
pub mod helper;
use crate::helper::doubled;
use ::sifr_runtime::SifrInt;
fn main() {
    println!("diagnostic_exit_codes cross-mode diagnostic and exit behavior demo:");
    println!("{}", doubled(SifrInt::from_i64(21)));
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn doubled(x: SifrInt) -> SifrInt {
    &x * &SifrInt::from_i64(2)
}
