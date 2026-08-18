// src/main.rs
mod helper;

use crate::helper::value;

fn main() {
    println!("temp_workspace_isolation invocation-scoped temp workspace isolation demo:");
    println!("{}", value());
}

// src/helper.rs
pub fn value() -> i64 {
    44_i64
}
