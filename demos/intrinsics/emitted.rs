// src/main.rs
pub mod sifr_generated_generated_support {
    #[expect(
        clippy::approx_constant,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) const PI: f64 = 3.141_592_653_589_793_f64;
    pub(super) fn sqrt(x: f64) -> f64 {
        ::sifr_stdlib::math::sqrt(x)
    }
}
use crate::sifr_generated_generated_support::{PI, sqrt};
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::assertions_on_constants,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
#[expect(
    clippy::approx_constant,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    assert_eq!(
        ::std::ops::Add::add(&SifrInt::from_i64(1), &SifrInt::from_i64(1)),
        SifrInt::from_i64(2)
    );
    assert_eq!("hello world".to_string(), "hello world");
    assert!(true);
    let result: f64 = sqrt(16.0_f64);
    assert_eq!(result, 4.0_f64);
    assert!(PI > 3.14_f64);
    println!("intrinsics demo: all checks passed!");
    assert_eq!(
        "intrinsics demo: all checks passed!".to_string(),
        "intrinsics demo: all checks passed!"
    );
}
