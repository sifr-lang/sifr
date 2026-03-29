fn main() {
    let items = [10_i64, 20, 30];
    let value = items.get(1).copied();
    if let Some(value) = value {
        println!("m27_1 remove data-dependent unwrap/expect demo:");
        println!("{}", value);
    } else {
        println!("m27_1 remove data-dependent unwrap/expect demo:");
        println!("missing");
    }
}
