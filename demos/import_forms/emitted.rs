// src/main.rs
mod helper;

use crate::helper::value;

fn main() {
    println!("import_forms import-form semantics demo:");
    println!("{}", value());
}

// src/helper.rs
pub fn value() -> i64 {
    17_i64
}
