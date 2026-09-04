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
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
pub fn doubled(x: SifrInt) -> SifrInt {
    ::std::ops::Mul::mul(&x, &SifrInt::from_i64(2))
}
