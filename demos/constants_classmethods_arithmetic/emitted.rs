// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::approx_constant,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
const PI: f64 = 3.14159_f64;
const fn sifr_generated_const_4d41585f52455452494553() -> SifrInt {
    SifrInt::from_i64(3)
}
fn sifr_generated_const_4150505f4e414d45() -> String {
    "sifr".to_string()
}
const DEBUG: bool = true;
#[derive(Debug, Clone, PartialEq)]
struct Temperature {
    celsius: f64,
}
impl Temperature {
    const fn new(celsius: f64) -> Self {
        let sifr_generated_field_value_69a867ea0a4ed8a3_63656c73697573: f64 = celsius;
        Self {
            celsius: sifr_generated_field_value_69a867ea0a4ed8a3_63656c73697573,
        }
    }
}
impl Temperature {
    fn from_fahrenheit(f: f64) -> Self {
        Self::new((f - 32.0_f64) * 5.0_f64 / 9.0_f64)
    }
}
impl ::std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Temperature(celsius={})", self.celsius)
    }
}
fn circle_area(r: f64) -> f64 {
    PI * r * r
}
fn get_config() -> String {
    format!(
        "{} (debug={}, retries={})",
        sifr_generated_const_4150505f4e414d45(),
        DEBUG,
        sifr_generated_const_4d41585f52455452494553()
    )
}
fn main() {
    println!("{}", circle_area(5.0_f64));
    println!("{}", get_config());
    println!("{PI}");
    println!("{}", sifr_generated_const_4d41585f52455452494553());
    let t: Temperature = Temperature::new(100.0_f64);
    println!("{}", t.celsius);
    let t2: Temperature = Temperature::from_fahrenheit(212.0_f64);
    println!("{}", t2.celsius);
}
