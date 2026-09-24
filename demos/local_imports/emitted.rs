// src/main.rs
mod sifr_generated_generated_support {
    use ::sifr_runtime::SifrInt;
    #[expect(
        clippy::approx_constant,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner emitted-Rust quality; remove when the Rust ABI can differ without changing Sifr semantics"
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
