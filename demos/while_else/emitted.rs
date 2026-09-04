// src/main.rs
use ::sifr_runtime::SifrInt;
fn classify(items: &[SifrInt]) -> String {
    let mut result: String = "init".to_string();
    {
        let mut sifr_generated_broke: bool = false;
        while !items.is_empty() {
            result = "broke".to_string();
            sifr_generated_broke = true;
            break;
        }
        if !sifr_generated_broke {
            result = "else".to_string();
        }
    }
    result
}
fn main() {
    println!("while_else while-else structured support demo:");
    println!("{}", classify(&Vec::new()));
    println!("{}", classify(&[SifrInt::from_i64(1)]));
}
