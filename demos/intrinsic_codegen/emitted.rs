// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ValueError;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn ceil(x: f64) -> SifrInt {
    ::sifr_stdlib::math::ceil(x).into_sifr_int()
}
fn atan2(y: f64, x: f64) -> f64 {
    ::sifr_stdlib::math::atan2(y, x)
}
const fn isfinite(x: f64) -> bool {
    ::sifr_stdlib::math::isfinite(x)
}
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let base: f64 = 9.0_f64;
    let root: f64 = sqrt(base);
    let rounded_down: SifrInt = floor(3.9_f64);
    let rounded_up_value_c84a77e463db860a: SifrInt = ceil(3.1_f64);
    let powered: f64 = (2.0_f64 as f64).powf(3.0_f64 as f64);
    let mut rounded: SifrInt = SifrInt::from_i64(0);
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let converted_rounded: SifrInt = SifrInt::from_f64_trunc(3.6_f64.round_ties_even())
            .ok_or_else(|| ValueError {
                message: "cannot round non-finite float to int".to_string(),
            })?;
        rounded = converted_rounded;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        assert!(false);
    }
    let angle: f64 = atan2(1.0_f64, 1.0_f64);
    let finite: bool = isfinite(powered);
    println!("root = {root}");
    assert_eq!(format!("root = {root}"), "root = 3");
    println!("rounded_down = {rounded_down}");
    assert_eq!(format!("rounded_down = {rounded_down}"), "rounded_down = 3");
    println!("rounded_up = {rounded_up_value_c84a77e463db860a}");
    assert_eq!(
        format!("rounded_up = {rounded_up_value_c84a77e463db860a}"),
        "rounded_up = 4"
    );
    println!("powered = {powered}");
    assert_eq!(format!("powered = {powered}"), "powered = 8");
    println!("rounded = {rounded}");
    assert_eq!(format!("rounded = {rounded}"), "rounded = 4");
    println!("angle_positive = {}", angle > 0.0_f64);
    assert_eq!(
        format!("angle_positive = {}", angle > 0.0_f64),
        "angle_positive = true"
    );
    println!("finite = {finite}");
    assert_eq!(format!("finite = {finite}"), "finite = true");
}
