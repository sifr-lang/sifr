// src/main.rs
use ::sifr_runtime::SifrInt;

fn drain(values: &mut Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    while !values.is_empty() {
        if let Some(item) = values.pop() {
            total = &total + &item;
        }
    }
    total.clone()
}

fn drain_front(values: &mut Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    while !values.is_empty() {
        if let Some(item) = {
    let __len = values.len();
    let __index = SifrInt::from_i64(0).normalize_index_or_len(__len);
    if __index >= __len { None } else { Some(values.remove(__index)) }
} {
            total = &total + &item;
        }
    }
    total.clone()
}

fn main() {
    assert!((&drain(&mut vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)]) == &SifrInt::from_i64(10)));
    assert!((&drain(&mut vec![]) == &SifrInt::from_i64(0)));
    assert!((&drain_front(&mut vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)]) == &SifrInt::from_i64(10)));
    assert!((&drain_front(&mut vec![]) == &SifrInt::from_i64(0)));
}
