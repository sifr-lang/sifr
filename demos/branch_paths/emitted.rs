// src/main.rs
mod helper;

use crate::helper::evaluate;

fn main() {
    println!("hir analysis consolidation regression matrix demo:");
    println!("{}", evaluate(10_i64));
    println!("{}", evaluate(0_i64));
}

// src/helper.rs
pub fn evaluate(n: i64) -> i64 {
    if n > (0_i64) {
        if n > (10_i64) {
            return n;
        } else {
            return n + (10_i64);
        }
    } else {
        return 45_i64;
    }
}
