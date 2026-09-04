// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn floor(x: f64) -> SifrInt {
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
use crate::sifr_generated_generated_support::floor;
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn call() -> SifrInt {
    floor(3.9_f64)
}
