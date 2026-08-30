// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn active_indices(flags: &Vec<bool>) -> Vec<SifrInt> {
    let mut out: Vec<SifrInt> = vec![];
    for index in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from(flags.len()), SifrInt::from_i64(1)) {
        let Some(__sifr_checked_value_0) = ({
    let __sifr_checked_read_collection = &flags;
    let __sifr_checked_read_index = index.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            continue;
        };
        if __sifr_checked_value_0 {
            out.push(index.clone());
        }
    }
    out
}

fn main() {
    assert!((format!("{:?}", active_indices(&vec![true, false, true, true])) == "[0, 2, 3]"));
    assert!((format!("{:?}", active_indices(&vec![false, false])) == "[]"));
    println!("monotonic_indices: ok");
}
