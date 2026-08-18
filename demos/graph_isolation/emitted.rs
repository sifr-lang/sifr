// src/main.rs
mod helper;

use crate::helper::value;

fn main() {
    println!("graph_isolation graph and isolation regression matrix demo:");
    println!("{}", value());
}

// src/helper.rs
pub fn value() -> i64 {
    55_i64
}
