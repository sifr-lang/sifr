// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    let mut items: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)];
    {
        let __idx_raw = -&SifrInt::from_i64(1);
        let __idx_norm = __idx_raw.normalize_index_or_len(items.len());
        if let Some(__elem) = items.get_mut(__idx_norm) {
            *__elem = SifrInt::from_i64(9);
        }
    }
    {
        let __idx_raw = -(SifrInt::from_i64(2));
        let __idx_norm = __idx_raw.normalize_index_or_len(items.len());
        if let Some(__elem) = items.get_mut(__idx_norm) {
            *__elem += SifrInt::from_i64(5);
        }
    }
    {
        let __idx_raw = -&SifrInt::from_i64(1);
        let __idx_norm = __idx_raw.normalize_index_or_len(items.len());
        if __idx_norm < items.len() {
            let _ = items.remove(__idx_norm);
        }
    }
    {
        let __idx_raw = -&SifrInt::from_i64(5);
        let __idx_norm = __idx_raw.normalize_index_or_len(items.len());
        if __idx_norm < items.len() {
            let _ = items.remove(__idx_norm);
        }
    }
    println!("indexing_rules indexing and semantics parity fixes demo:");
    println!("{:?}", items);
}
