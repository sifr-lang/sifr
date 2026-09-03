// src/main.rs
pub mod worker;
use crate::worker::call;
fn main() {
    println!("external_modules non-main externals demo:");
    println!("{}", call());
}

// src/worker.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
#[must_use]
pub fn call() -> SifrInt {
    floor(3.9_f64)
}
