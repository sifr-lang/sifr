// src/main.rs
use ::sifr_runtime::SifrInt;
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn main() {
    println!("auto_detection structural workspace demo:");
    println!("{}", floor(3.9_f64));
}
