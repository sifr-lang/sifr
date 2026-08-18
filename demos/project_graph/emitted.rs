// src/main.rs
mod consumer;
mod provider;

use crate::consumer::describe;

fn main() {
    println!("{}", describe());
}

// src/consumer.rs
pub use crate::provider::BASE;
pub use crate::provider::answer;
pub fn describe() -> i64 {
    (answer() + BASE) - (40_i64)
}

// src/provider.rs
pub const BASE: i64 = 41_i64;
pub fn answer() -> i64 {
    BASE
}
