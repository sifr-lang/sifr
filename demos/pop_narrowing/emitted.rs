// src/main.rs
use ::sifr_runtime::SifrInt;
fn drain(values: &mut Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    while !values.is_empty() {
        if let Some(item) = values.pop() {
            total = ::std::ops::Add::add(&total, &item);
        }
    }
    total
}
fn drain_front(values: &mut Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    while !values.is_empty() {
        if let Some(item) = {
            let sifr_generated_len = values.len();
            let sifr_generated_index =
                SifrInt::from_i64(0).normalize_index_or_len(sifr_generated_len);
            if sifr_generated_index >= sifr_generated_len {
                None
            } else {
                Some(values.remove(sifr_generated_index))
            }
        } {
            total = ::std::ops::Add::add(&total, &item);
        }
    }
    total
}
fn main() {
    assert_eq!(
        drain(&mut vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4)
        ]),
        SifrInt::from_i64(10)
    );
    assert_eq!(drain(&mut Vec::new()), SifrInt::from_i64(0));
    assert_eq!(
        drain_front(&mut vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4)
        ]),
        SifrInt::from_i64(10)
    );
    assert_eq!(drain_front(&mut Vec::new()), SifrInt::from_i64(0));
}
