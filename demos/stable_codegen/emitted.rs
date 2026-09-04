// src/main.rs
use ::sifr_runtime::SifrInt;
fn summarize(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for value in values.iter() {
        if value > SifrInt::from_i64(10) {
            total = ::std::ops::Add::add(&total, value);
        } else {
            total = ::std::ops::Add::add(&total, &SifrInt::from_i64(1));
        }
    }
    total
}
fn main() {
    println!("stable_codegen analysis/emission boundary hardening demo:");
    println!(
        "{}",
        summarize(&[
            SifrInt::from_i64(3),
            SifrInt::from_i64(12),
            SifrInt::from_i64(20)
        ])
    );
}
