// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
}
use crate::sifr_generated_generated_support::floor;
fn main() {
    println!("auto_detection structural workspace demo:");
    println!("{}", floor(3.9_f64));
}
