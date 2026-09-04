// src/main.rs
mod sifr_generated_generated_support {
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
}
use crate::sifr_generated_generated_support::*;
fn main() {
    println!("auto_detection structural workspace demo:");
    println!("{}", floor(3.9_f64));
}
