// src/main.rs
fn main() {
    let items: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let value: Option<i64> = {
    let __sifr_index_list = &items;
    let __sifr_index_i = 1_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(value) = value {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("{}", value);
    } else {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("missing");
    }
}
