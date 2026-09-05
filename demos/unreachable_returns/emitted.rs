// src/main.rs
use ::sifr_runtime::SifrInt;
const fn inferred(flag: bool) -> SifrInt {
    if flag {
        return SifrInt::from_i64(1);
    }
    SifrInt::from_i64(2)
}
fn consume(n: &SifrInt) -> SifrInt {
    ::std::ops::Add::add(n, &SifrInt::from_i64(1))
}
fn main() {
    println!("unreachable_returns diagnostics and consumer integration demo:");
    println!("{}", consume(&inferred(true)));
    println!("{}", consume(&inferred(false)));
}
