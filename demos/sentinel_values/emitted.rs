// src/main.rs
use ::sifr_runtime::SifrInt;
fn smallest_or_zero(values: &[SifrInt]) -> SifrInt {
    let mut best: SifrInt = SifrInt::from_i64(9_223_372_036_854_775_807);
    for value in values.iter().cloned() {
        if &value < &best {
            best = value.clone();
        }
    }
    if &best == &SifrInt::from_i64(9_223_372_036_854_775_807) {
        SifrInt::from_i64(0)
    } else {
        best
    }
}
fn main() {
    assert_eq!(
        &smallest_or_zero(&vec![
            SifrInt::from_i64(8),
            SifrInt::from_i64(3),
            SifrInt::from_i64(7)
        ]),
        &SifrInt::from_i64(3)
    );
    assert_eq!(&smallest_or_zero(&Vec::new()), &SifrInt::from_i64(0));
    println!("sentinel_values: ok");
}
