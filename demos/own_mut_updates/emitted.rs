// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn increment_all(mut values: Vec<SifrInt>) -> Vec<SifrInt> {
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from(values.len()), SifrInt::from_i64(1)) {
        {
            let __assign_value = &values[::sifr_runtime::to_usize_proven(&(i))].clone() + &SifrInt::from_i64(1);
            {
                let __idx_raw = i.clone();
                let __idx_norm = __idx_raw.normalize_index_or_len(values.len());
                if let Some(__elem) = values.get_mut(__idx_norm) {
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
            let __idx_raw = i.clone();
            let __idx_norm = __idx_raw.normalize_index_or_len(values.len());
            if let Some(__elem) = values.get_mut(__idx_norm) {
                *__elem = SifrInt::from_i64(0);
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
