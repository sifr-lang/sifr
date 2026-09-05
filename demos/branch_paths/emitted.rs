// src/main.rs
pub mod helper;
use crate::helper::evaluate;
use ::sifr_runtime::SifrInt;
fn main() {
    println!("hir analysis consolidation regression matrix demo:");
    println!("{}", evaluate(&SifrInt::from_i64(10)));
    println!("{}", evaluate(&SifrInt::from_i64(0)));
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn evaluate(n: &SifrInt) -> SifrInt {
    if n > &SifrInt::from_i64(0) {
        if n > &SifrInt::from_i64(10) {
            (*n).clone()
        } else {
            ::std::ops::Add::add(n, &SifrInt::from_i64(10))
        }
    } else {
        SifrInt::from_i64(45)
    }
}
