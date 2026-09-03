// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
const PI: f64 = 3.141_592_653_589_793_f64;
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn main() {
    println!("local_imports stdlib cache local loops demo:");
    println!("{}", floor(PI));
}
