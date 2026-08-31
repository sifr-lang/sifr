// src/main.rs
use ::sifr_runtime::SifrInt;

fn accumulate(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    let mut apply = || {
    for value in values.iter().cloned() {
        total += value;
    }
};
    apply();
    total.clone()
}

fn main() {
    assert!((format!("{}", accumulate(&vec![SifrInt::from_i64(3), SifrInt::from_i64(1), SifrInt::from_i64(4), SifrInt::from_i64(1), SifrInt::from_i64(5)])) == "14"));
}
