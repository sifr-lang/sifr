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
pub use ::sifr_runtime::SifrInt;
pub fn describe() -> SifrInt {
    &(&answer() + &BASE) - &SifrInt::from_i64(40)
}

// src/provider.rs
pub use ::sifr_runtime::SifrInt;
pub fn __const_BASE() -> SifrInt {
    SifrInt::from_i64(41)
}
pub fn answer() -> SifrInt {
    __const_BASE()
}
