// src/main.rs
use ::sifr_runtime::SifrInt;
fn sum_iterable(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for value in values.iter() {
        total = ::std::ops::Add::add(&total, value);
    }
    total
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(2),
        SifrInt::from_i64(4),
        SifrInt::from_i64(6),
    ];
    println!("{}", sum_iterable(&nums));
}
