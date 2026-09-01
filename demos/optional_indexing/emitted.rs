// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
    ];
    let value: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &items;
        let sifr_generated_checked_read_index = SifrInt::from_i64(1);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(value) = value.clone() {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("{value}");
    } else {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("missing");
    }
}
