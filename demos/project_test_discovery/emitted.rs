// src/main.rs
mod helper;
mod shared;

use crate::helper::value;

fn main() {
    println!("project_test_discovery project/test discovery parity behavior demo:");
    println!("{}", value());
}

// src/helper.rs
pub use crate::shared::BASE;
pub fn value() -> i64 {
    BASE
}

// src/shared.rs
pub const BASE: i64 = 42_i64;
