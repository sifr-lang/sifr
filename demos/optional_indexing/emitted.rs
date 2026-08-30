// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    let items: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    let value: Option<SifrInt> = {
    let __sifr_checked_read_collection = &items;
    let __sifr_checked_read_index = SifrInt::from_i64(1);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
};
    if let Some(value) = value.clone() {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("{}", value);
    } else {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("missing");
    }
}
