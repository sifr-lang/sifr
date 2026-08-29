// src/main.rs
mod helper;

use crate::helper::evaluate;

use ::sifr_runtime::SifrInt;

fn main() {
    println!("hir analysis consolidation regression matrix demo:");
    println!("{}", evaluate(SifrInt::from_i64(10)));
    println!("{}", evaluate(SifrInt::from_i64(0)));
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
pub fn evaluate(n: SifrInt) -> SifrInt {
    if &n > &SifrInt::from_i64(0) {
        if &n > &SifrInt::from_i64(10) {
            return n.clone();
        } else {
            return &n + &SifrInt::from_i64(10);
        }
    } else {
        return SifrInt::from_i64(45);
    }
}
