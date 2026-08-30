// src/main.rs
use ::sifr_runtime::SifrInt;

fn identity(x: SifrInt) -> SifrInt {
    x.clone()
}

fn main() {
    let value: SifrInt = identity(SifrInt::from_i64(17));
    println!("type_checking frontend-only check path demo:");
    println!("{}", value);
}
