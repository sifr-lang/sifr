// src/main.rs
use ::sifr_runtime::SifrInt;

fn payload_size(data: &[SifrInt]) -> SifrInt {
    SifrInt::from(data.len())
}

fn main() {
    println!("{}", payload_size(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]));
    println!("recursive alias names resolved");
}
