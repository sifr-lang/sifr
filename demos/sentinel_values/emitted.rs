// src/main.rs
use ::sifr_runtime::SifrInt;

fn smallest_or_zero(values: &Vec<SifrInt>) -> SifrInt {
    let mut best: SifrInt = SifrInt::from_i64(9223372036854775807);
    for value in values.iter().cloned() {
        if &value < &best {
            best = value.clone();
        }
    }
    if &best != &SifrInt::from_i64(9223372036854775807) { best } else { SifrInt::from_i64(0) }
}

fn main() {
    assert!((&smallest_or_zero(&vec![SifrInt::from_i64(8), SifrInt::from_i64(3), SifrInt::from_i64(7)]) == &SifrInt::from_i64(3)));
    assert!((&smallest_or_zero(&vec![]) == &SifrInt::from_i64(0)));
    println!("sentinel_values: ok");
}
