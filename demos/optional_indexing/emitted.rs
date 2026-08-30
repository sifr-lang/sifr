// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    let items: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    let value: Option<SifrInt> = {
    let __sifr_index_list = &items;
    let __sifr_index_i = SifrInt::from_i64(1);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    if let Some(value) = value.clone() {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("{}", value);
    } else {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("missing");
    }
}
