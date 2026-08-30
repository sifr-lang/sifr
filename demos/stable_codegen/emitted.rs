// src/main.rs
use ::sifr_runtime::SifrInt;

fn summarize(values: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for value in values.iter().cloned() {
        if &value > &SifrInt::from_i64(10) {
            total = &total + &value;
        } else {
            total = &total + &SifrInt::from_i64(1);
        }
    }
    total.clone()
}

fn main() {
    println!("stable_codegen analysis/emission boundary hardening demo:");
    println!("{}", summarize(&vec![SifrInt::from_i64(3), SifrInt::from_i64(12), SifrInt::from_i64(20)]));
}
