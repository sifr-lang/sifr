// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
}
pub mod helper;
use crate::helper::value;
use ::sifr_runtime::SifrInt;
fn main() {
    println!("mode_consistency parity regression matrix demo:");
    println!("{}", value(&SifrInt::from_i64(1)));
}

// src/helper.rs
use crate::sifr_generated_generated_support::floor;
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub const fn sifr_generated_const_42415345() -> SifrInt {
    SifrInt::from_i64(5)
}
#[must_use]
pub fn value(x: &SifrInt) -> SifrInt {
    ::std::ops::Add::add(
        &::std::ops::Add::add(&sifr_generated_const_42415345(), &floor(2.9_f64)),
        x,
    )
}
