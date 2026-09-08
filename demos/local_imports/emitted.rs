// src/main.rs
mod sifr_generated_generated_support {
    use ::sifr_runtime::SifrInt;
    #[expect(
        clippy::approx_constant,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub const PI: f64 = 3.141_592_653_589_793_f64;
    #[must_use]
    pub fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
}
use crate::sifr_generated_generated_support::{PI, floor};
fn main() {
    println!("local_imports stdlib cache local loops demo:");
    println!("{}", floor(PI));
}
