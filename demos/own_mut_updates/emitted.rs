// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn increment_all(mut values: Vec<SifrInt>) -> Vec<SifrInt> {
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from(values.len()), SifrInt::from_i64(1)) {
        let Some(__sifr_checked_value_0) = ({
    let __sifr_checked_read_collection = &values;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            continue;
        };
        {
            let __assign_value = &__sifr_checked_value_0.clone() + &SifrInt::from_i64(1);
            {
                let __index_raw = i.clone();
                let __index_normalized = __index_raw.normalize_index_or_len(values.len());
                if let Some(__elem) = values.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    values
}

fn clear_all(mut values: Vec<SifrInt>) -> Vec<SifrInt> {
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from(values.len()), SifrInt::from_i64(1)) {
        {
            let __assign_value = SifrInt::from_i64(0);
            {
                let __index_raw = i.clone();
                let __index_normalized = __index_raw.normalize_index_or_len(values.len());
                if let Some(__elem) = values.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    values
}

fn main() {
    assert!((format!("{:?}", increment_all(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)])) == "[2, 3, 4]"));
    assert!((format!("{:?}", clear_all(vec![SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)])) == "[0, 0, 0]"));
    println!("own_mut_updates: ok");
}
