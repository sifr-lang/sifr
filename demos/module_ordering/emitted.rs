// src/main.rs
pub mod consumer;
pub mod provider;
use crate::consumer::value;
fn main() {
    println!("module_ordering dependency-safe module ordering demo:");
    println!("{}", value());
}

// src/consumer.rs
pub use crate::provider::provided;
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn value() -> SifrInt {
    provided()
}

// src/provider.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub const fn provided() -> SifrInt {
    SifrInt::from_i64(19)
}
