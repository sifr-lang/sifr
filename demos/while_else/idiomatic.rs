fn classify(items: &[i64]) -> &'static str {
    if items.is_empty() {
        "else"
    } else {
        "broke"
    }
}

fn main() {
    println!("while_else while-else structured support demo:");
    println!("{}", classify(&[]));
    println!("{}", classify(&[1]));
}
