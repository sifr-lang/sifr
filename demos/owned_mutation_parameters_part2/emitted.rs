// src/main.rs
use ::sifr_runtime::SifrInt;

fn mutate_and_return(mut items: Vec<SifrInt>) -> Vec<SifrInt> {
    if (&SifrInt::from(items.len()) > &SifrInt::from_i64(1)) {
        {
            let __assign_value = SifrInt::from_i64(9);
            {
                let __index_raw = SifrInt::from_i64(0);
                let __index_normalized = __index_raw.normalize_index_or_len(items.len());
                if let Some(__elem) = items.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                }
            }
        }
        {
            let __assign_value = SifrInt::from_i64(10);
            {
                let __index_raw = SifrInt::from_i64(1);
                let __index_normalized = __index_raw.normalize_index_or_len(items.len());
                if let Some(__elem) = items.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    items
}

fn mutate_borrowed(items: &mut Vec<SifrInt>) -> SifrInt {
    if (&SifrInt::from(items.len()) > &SifrInt::from_i64(0)) {
        {
            let __assign_value = SifrInt::from_i64(14);
            {
                let __index_raw = SifrInt::from_i64(0);
                let __index_normalized = __index_raw.normalize_index_or_len(items.len());
                if let Some(__elem) = items.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    SifrInt::from(items.len())
}

fn main() {
    let values: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)];
    let mut moved: Vec<SifrInt> = mutate_and_return(values);
    println!("{}", ({
    let __sifr_index_list = &moved;
    let __sifr_index_i = SifrInt::from_i64(0);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", ({
    let __sifr_index_list = &moved;
    let __sifr_index_i = SifrInt::from_i64(1);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", mutate_borrowed(&mut moved));
}
