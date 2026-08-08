// src/main.rs
mod helper;

use crate::helper::value;

fn main() {
    println!("resolver_triggers explicit workspace import demo:");
    println!("{}", value());
}

// src/helper.rs
pub fn value() -> i64 {
    18_i64
}
