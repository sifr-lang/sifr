// src/main.rs
mod a_consumer;
mod z_provider;

use crate::a_consumer::fetch;

fn main() {
    println!("module_cycle_diagnostics deterministic module graph and cycle diagnostics demo:");
    println!("{}", fetch());
}

// src/a_consumer.rs
pub use crate::z_provider::value;
pub fn fetch() -> i64 {
    value() + (1_i64)
}

// src/z_provider.rs
pub fn value() -> i64 {
    41_i64
}
