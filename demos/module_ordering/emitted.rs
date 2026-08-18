// src/main.rs
mod consumer;
mod provider;

use crate::consumer::value;

fn main() {
    println!("module_ordering dependency-safe module ordering demo:");
    println!("{}", value());
}

// src/consumer.rs
pub use crate::provider::provided;
pub fn value() -> i64 {
    provided()
}

// src/provider.rs
pub fn provided() -> i64 {
    19_i64
}
