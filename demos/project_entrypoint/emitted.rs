// src/main.rs
mod sifr_generated_generated_support {
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
}
pub mod helper;
use crate::helper::adjusted;
use ::sifr_runtime::SifrInt;
fn main() {
    println!("project_entrypoint canonical frontend entry path demo:");
    println!("{}", adjusted(SifrInt::from_i64(5)));
}

// src/helper.rs
use crate::sifr_generated_generated_support::*;
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn adjusted(value: SifrInt) -> SifrInt {
    &value + &floor(2.9_f64)
}
