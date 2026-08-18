// src/main.rs
mod helper;

use crate::helper::doubled;

fn main() {
    println!("diagnostic_exit_codes cross-mode diagnostic and exit behavior demo:");
    println!("{}", doubled(21_i64));
}

// src/helper.rs
pub fn doubled(x: i64) -> i64 {
    x * (2_i64)
}
