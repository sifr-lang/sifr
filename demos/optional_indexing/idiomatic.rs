fn main() {
    let items = [10_i64, 20, 30];
    let value = items.get(1).copied();
    if let Some(value) = value {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("{}", value);
    } else {
        println!("optional_indexing remove data-dependent unwrap/expect demo:");
        println!("missing");
    }
}
