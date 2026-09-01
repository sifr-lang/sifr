// src/main.rs
pub mod consumer;
pub mod provider;
use crate::consumer::describe;
fn main() {
    println!("{}", describe());
}

// src/consumer.rs
pub use crate::provider::answer;
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn describe() -> SifrInt {
    &(&answer() + &crate::provider::sifr_generated_const_42415345()) - &SifrInt::from_i64(40)
}

// src/provider.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub const fn sifr_generated_const_42415345() -> SifrInt {
    SifrInt::from_i64(41)
}
#[must_use]
pub const fn answer() -> SifrInt {
    sifr_generated_const_42415345()
}
