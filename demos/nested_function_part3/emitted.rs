// src/main.rs
use ::sifr_runtime::SifrInt;
fn accumulate(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    let mut apply = || {
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for value in values.iter() {
            total = ::std::ops::Add::add(&total, value);
        }
    };
    apply();
    total.clone()
}
fn main() {
    assert_eq!(
        accumulate(&[
            SifrInt::from_i64(3),
            SifrInt::from_i64(1),
            SifrInt::from_i64(4),
            SifrInt::from_i64(1),
            SifrInt::from_i64(5)
        ])
        .to_string(),
        "14"
    );
}
