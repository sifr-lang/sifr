// src/main.rs
use ::sifr_runtime::SifrInt;

fn classify(items: &[SifrInt]) -> String {
    let mut result: String = "init".to_string();
    {
        let mut _broke: bool = false;
        while !items.is_empty() {
            result = "broke".to_string();
            _broke = true;
            break;
        }
        if !(_broke) {
            result = "else".to_string();
        }
    }
    result
}

fn main() {
    println!("while_else while-else structured support demo:");
    println!("{}", classify(&vec![]));
    println!("{}", classify(&vec![SifrInt::from_i64(1)]));
}
