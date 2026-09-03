// src/main.rs
mod sifr_generated_project_nominals {}
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
const PI: f64 = 3.141_592_653_589_793_f64;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    assert_eq!(
        &SifrInt::from_i64(1) + &SifrInt::from_i64(1),
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
