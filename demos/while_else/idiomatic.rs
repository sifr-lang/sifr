fn classify(items: &[i64]) -> &'static str {
    if items.is_empty() {
        "else"
    } else {
        "broke"
    }
}

fn main() {
    println!("m21_2 while-else structured support demo:");
    println!("{}", classify(&[]));
    println!("{}", classify(&[1]));
}
