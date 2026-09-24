// src/main.rs
use ::sifr_runtime::SifrInt;
fn safe_add_one(x: Option<&SifrInt>) -> SifrInt {
    let x: Option<SifrInt> = x.cloned();
    let Some(x) = x else {
        return SifrInt::from_i64(0);
    };
    ::std::ops::Add::add(&x, &SifrInt::from_i64(1))
}
fn main() {
    println!("optional_arithmetic optional arithmetic soundness demo:");
    println!("{}", safe_add_one(Some(&SifrInt::from_i64(5))));
    println!("{}", safe_add_one(None));
    let total: Option<SifrInt> = Some(SifrInt::from_i64(9));
    let count: Option<SifrInt> = Some(SifrInt::from_i64(3));
    if let Some(_total) = total
        && let Some(_count) = count
    {
        println!("{}", 9.0 / 3.0);
    }
    let missing_total: Option<SifrInt> = None;
    if missing_total.is_none() {
        println!("{}", SifrInt::from_i64(0));
    }
}
