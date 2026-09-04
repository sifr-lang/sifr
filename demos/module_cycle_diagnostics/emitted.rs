// src/main.rs
pub mod a_consumer;
pub mod z_provider;
use crate::a_consumer::fetch;
fn main() {
    println!("module_cycle_diagnostics deterministic module graph and cycle diagnostics demo:");
    println!("{}", fetch());
}

// src/a_consumer.rs
pub use crate::z_provider::value;
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn fetch() -> SifrInt {
    ::std::ops::Add::add(&value(), &SifrInt::from_i64(1))
}

// src/z_provider.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub const fn value() -> SifrInt {
    SifrInt::from_i64(41)
}
