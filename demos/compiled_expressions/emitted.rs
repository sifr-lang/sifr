// src/main.rs
mod sifr_generated_generated_support {
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
fn add(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}
fn main() {
    let total: SifrInt = add(SifrInt::from_i64(10), SifrInt::from_i64(11));
    if &total > &SifrInt::from_i64(20) {
        println!("compiled_expressions lower decomposition demo:");
    }
    println!("{total}");
    println!("{}", floor(3.9_f64));
}
