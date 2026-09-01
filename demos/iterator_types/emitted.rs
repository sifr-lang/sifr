// src/main.rs
use ::sifr_runtime::SifrInt;
fn sum_iterable(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for value in values.iter().cloned() {
        total = &total + &value;
    }
    total.clone()
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(2),
        SifrInt::from_i64(4),
        SifrInt::from_i64(6),
    ];
    println!(
        "{}",
        sum_iterable(&nums.iter().cloned().collect::<Vec<_>>())
    );
}
