// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn reversed_values(values: &Vec<SifrInt>) -> Vec<SifrInt> {
    let mut out: Vec<SifrInt> = vec![];
    for i in SifrRange::new_known_nonzero(&SifrInt::from(values.len()) - &SifrInt::from_i64(1), -(SifrInt::from_i64(1)), -(SifrInt::from_i64(1))) {
        let Some(__sifr_checked_value_0) = ({
    let __sifr_checked_read_collection = &values;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        out.push(__sifr_checked_value_0.clone());
    }
    out
}

fn main() {
    assert!((format!("{:?}", reversed_values(&vec![SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)])) == "[6, 5, 4]"));
    assert!((format!("{:?}", reversed_values(&vec![])) == "[]"));
    println!("reverse_indices: ok");
}
