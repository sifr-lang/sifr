// src/main.rs
mod sifr_generated_generated_support {
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
}
pub mod worker;
use crate::worker::call;
fn main() {
    println!("external_modules non-main externals demo:");
    println!("{}", call());
}

// src/worker.rs
use crate::sifr_generated_generated_support::*;
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn call() -> SifrInt {
    floor(3.9_f64)
}
