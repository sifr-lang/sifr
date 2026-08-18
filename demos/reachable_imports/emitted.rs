// src/main.rs
mod helper;

use crate::helper::compute;

fn main() {
    println!("reachable_imports import-closure discovery demo:");
    println!("{}", compute(6_i64));
}

// src/helper.rs
pub fn compute(x: i64) -> i64 {
    x * (7_i64)
}
