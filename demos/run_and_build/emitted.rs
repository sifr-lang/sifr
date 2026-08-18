// src/main.rs
mod helper;

use crate::helper::msg;

fn main() {
    println!("run_and_build run/build alignment demo:");
    println!("{}", msg());
}

// src/helper.rs
pub fn msg() -> String {
    "aligned".to_string()
}
