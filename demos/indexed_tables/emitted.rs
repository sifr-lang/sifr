// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn write_indices(size: SifrInt) -> Vec<SifrInt> {
    let mut out: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), size.clone(), SifrInt::from_i64(1)) {
        __sifr_list_comp.push(SifrInt::from_i64(0));
    }
    __sifr_list_comp
};
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from(out.len()), SifrInt::from_i64(1)) {
        {
            let __assign_value = &i + &SifrInt::from_i64(1);
            {
                let __index_raw = i.clone();
                let __index_normalized = __index_raw.normalize_index_or_len(out.len());
                if let Some(__elem) = out.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    out
}

fn main() {
    assert!((format!("{:?}", write_indices(SifrInt::from_i64(4))) == "[1, 2, 3, 4]"));
    assert!((format!("{:?}", write_indices(SifrInt::from_i64(0))) == "[]"));
    println!("indexed_tables: ok");
}
