// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
}
use crate::sifr_generated_generated_support::floor;
use ::sifr_runtime::SifrInt;
fn add(a: &SifrInt, b: &SifrInt) -> SifrInt {
    ::std::ops::Add::add(a, b)
}
fn main() {
    let total: SifrInt = add(&SifrInt::from_i64(10), &SifrInt::from_i64(11));
    if total > SifrInt::from_i64(20) {
        println!("compiled_expressions lower decomposition demo:");
    }
    println!("{total}");
    println!("{}", floor(3.9_f64));
}
