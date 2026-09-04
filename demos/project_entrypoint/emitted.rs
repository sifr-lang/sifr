// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn floor(x: f64) -> SifrInt {
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
use crate::sifr_generated_generated_support::floor;
pub use ::sifr_runtime::SifrInt;
#[must_use]
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
pub fn adjusted(value: SifrInt) -> SifrInt {
    ::std::ops::Add::add(&value, &floor(2.9_f64))
}
