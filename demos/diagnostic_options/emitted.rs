// src/main.rs
use ::sifr_runtime::SifrInt;

fn sum2(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}

fn main() {
    println!("{}", sum2(SifrInt::from_i64(20), SifrInt::from_i64(22)));
}
