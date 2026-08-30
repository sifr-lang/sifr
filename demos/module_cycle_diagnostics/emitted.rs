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
pub use ::sifr_runtime::SifrInt;
pub fn fetch() -> SifrInt {
    &value() + &SifrInt::from_i64(1)
}

// src/z_provider.rs
pub use ::sifr_runtime::SifrInt;
pub fn value() -> SifrInt {
    SifrInt::from_i64(41)
}
