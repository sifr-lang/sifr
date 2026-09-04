// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn sum2(a: SifrInt, b: SifrInt) -> SifrInt {
    ::std::ops::Add::add(&a, &b)
}
fn main() {
    println!("{}", sum2(SifrInt::from_i64(20), SifrInt::from_i64(22)));
}
