// src/main.rs
use ::sifr_runtime::SifrInt;
fn classify(items: &[SifrInt]) -> String {
    if items.is_empty() {
        "else".to_string()
    } else {
        "broke".to_string()
    }
}
fn main() {
    println!("while_else while-else structured support demo:");
    println!("{}", classify(&[]));
    println!("{}", classify(&[SifrInt::from_i64(1)]));
}
